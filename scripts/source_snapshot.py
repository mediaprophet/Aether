"""Verify the captured canonical source used by local qualification consumers."""
# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import tomllib
import reviewed_etalon_overlays as reviewed


AUTHORITY_TRANSPORTS = {
    "aether-chora-bridge-protocol": "packages/chora-bridge-protocol",
    "aether-chora-client": "packages/chora-client",
}


def validate_transport_profile(metadata: dict) -> None:
    """The library/pump snapshot excludes worker-backed test-fixture inputs."""
    features = {node["id"]: set(node.get("features", [])) for node in (metadata.get("resolve") or {}).get("nodes", [])}
    for package in metadata.get("packages", []):
        if package["name"] == "aether-chora-client" and "test-fixture" in features.get(package.get("id"), set()):
            raise ValueError("Captured client test-fixture requires uncaptured worker helper inputs; only library/pump profile is admitted")


def authority_transport_closure(host_manifest: Path, aether: Path) -> dict[str, Path]:
    """Resolve declared authority transport paths, including optional dependencies.

    Aether UI remains the established explicit live external boundary. These two
    authority-routing packages must stay inside the selected Aether source tree;
    a registry, Git alias or absolute live-tree escape is never a frozen source.
    """
    selected: dict[str, Path] = {}
    pending = [host_manifest.resolve()]
    while pending:
        manifest = pending.pop()
        parsed = tomllib.loads(manifest.read_text(encoding="utf-8-sig"))
        tables = [parsed.get(kind, {}) for kind in ("dependencies", "dev-dependencies", "build-dependencies")]
        for target in parsed.get("target", {}).values():
            tables.extend(target.get(kind, {}) for kind in ("dependencies", "dev-dependencies", "build-dependencies"))
        for table in tables:
            for alias, spec in table.items():
                name = spec.get("package", alias) if isinstance(spec, dict) else alias
                if name not in AUTHORITY_TRANSPORTS:
                    if isinstance(spec, dict) and "path" in spec:
                        local = (manifest.parent / spec["path"]).resolve()
                        if parsed.get("package", {}).get("name") in AUTHORITY_TRANSPORTS or (
                                name != "aether-core" and local.is_relative_to(aether.resolve())):
                            raise ValueError(f"Unreviewed authority transport path dependency: {name}")
                    continue
                expected = (aether / AUTHORITY_TRANSPORTS[name] / "Cargo.toml").resolve()
                if not isinstance(spec, dict) or "path" not in spec or "git" in spec or "registry" in spec or Path(spec["path"]).is_absolute():
                    raise ValueError(f"Authority transport requires its explicit captured path: {name}")
                actual = (manifest.parent / spec["path"] / "Cargo.toml").resolve()
                if not expected.is_relative_to(aether.resolve()) or actual != expected or not expected.is_file():
                    raise ValueError(f"Authority transport leaves selected source closure: {name}")
                identity = tomllib.loads(expected.read_text(encoding="utf-8-sig"))["package"]["name"]
                if identity != name:
                    raise ValueError(f"Authority transport package identity differs: {name}")
                if name not in selected:
                    selected[name] = expected
                    pending.append(expected)
    return selected


def verify_source_snapshot(receipt_path: Path) -> dict:
    receipt_path = receipt_path.resolve()
    root = receipt_path.parent
    receipt = json.loads(receipt_path.read_text(encoding="utf-8-sig"))
    if receipt.get("schema") != "aether.qualia-source-snapshot.v1":
        raise ValueError("Unsupported canonical source snapshot receipt")
    # Cache shared ancestry only within this invocation. Each input leaf still
    # receives fresh link/file/hash checks, and the later inventory independently
    # rechecks every directory and file for links.
    checked_ancestors = set()

    def member(relative: str) -> Path:
        lexical = Path(relative)
        if lexical.is_absolute() or ".." in lexical.parts:
            raise ValueError(f"Snapshot path escapes its captured root: {relative}")
        candidate = root
        for index, part in enumerate(lexical.parts):
            candidate = candidate / part
            ancestor = index < len(lexical.parts) - 1
            if ancestor and candidate in checked_ancestors:
                continue
            if candidate.is_symlink() or (hasattr(candidate, "is_junction") and candidate.is_junction()):
                raise ValueError(f"Linked snapshot input: {relative}")
            if ancestor:
                checked_ancestors.add(candidate)
        path = (root / relative).resolve()
        if Path(relative).is_absolute() or not path.is_relative_to(root):
            raise ValueError(f"Snapshot path escapes its captured root: {relative}")
        return path

    expected = set()
    for entry in receipt["files"]:
        path = member(entry["path"])
        if path in expected or path == receipt_path:
            raise ValueError(f"Duplicate snapshot input: {entry['path']}")
        expected.add(path)
        for key in ("sha256", "source_sha256"):
            if not re.fullmatch(r"[0-9a-f]{64}", entry[key]):
                raise ValueError(f"Invalid snapshot digest for {entry['path']}")
        if not Path(entry["source"]).is_absolute():
            raise ValueError(f"Snapshot provenance must identify an absolute source: {entry['path']}")
        if entry["sha256"] != entry["source_sha256"] and not entry.get("transform"):
            raise ValueError(f"Snapshot transformation is not recorded: {entry['path']}")
        if not path.is_file() or path.is_symlink():
            raise ValueError(f"Missing or linked snapshot input: {entry['path']}")
        with path.open("rb") as captured:
            digest = hashlib.file_digest(captured, "sha256").hexdigest()
        if digest != entry["sha256"]:
            raise ValueError(f"Snapshot input changed: {entry['path']}")
    actual = set()
    for directory, directories, filenames in os.walk(root, followlinks=False):
        directories[:] = [name for name in directories if name != "target"]
        for name in directories + filenames:
            path = Path(directory) / name
            if path.is_symlink() or (hasattr(path, "is_junction") and path.is_junction()):
                raise ValueError(f"Linked snapshot input: {path.relative_to(root)}")
        actual.update((Path(directory) / name).resolve() for name in filenames)
    extra = actual - expected - {receipt_path}
    if extra:
        raise ValueError("Unrecorded snapshot inputs: " + ", ".join(str(p.relative_to(root)) for p in sorted(extra)))

    packages = {}
    for package in receipt["packages"]:
        path = member(package["path"])
        if path not in expected:
            raise ValueError(f"Snapshot package manifest was not captured: {package['name']}")
        identity = tomllib.loads(path.read_text(encoding="utf-8-sig"))["package"]["name"]
        if identity != package["name"] or identity in packages:
            raise ValueError(f"Snapshot package identity mismatch: {package['name']}")
        packages[identity] = path
    host = member(receipt["host_source"])
    qualia = member(receipt["qualia_source"])
    if packages.get("aether-qualia-host") != host / "Cargo.toml":
        raise ValueError("Snapshot host source does not match its captured package")
    transport = authority_transport_closure(host / "Cargo.toml", root / "Aether")
    for name, manifest in transport.items():
        if packages.get(name) != manifest:
            raise ValueError(f"Authority transport was not captured as a package: {name}")
    for name in AUTHORITY_TRANSPORTS.keys() & packages.keys():
        if packages[name] != (root / "Aether" / AUTHORITY_TRANSPORTS[name] / "Cargo.toml").resolve():
            raise ValueError(f"Snapshot authority transport source does not match {name}")
    required = {"vibe", "vibescript-abi", "qualia-core-db", "qualia-cooperative-core"}
    for name in required:
        if packages.get(name) != qualia / "crates" / name / "Cargo.toml":
            raise ValueError(f"Snapshot canonical source does not match {name}")
    overlays = receipt.get("reviewed_etalon_overlays",{})
    table = tomllib.loads((qualia / "Cargo.toml").read_text(encoding="utf-8-sig")).get("patch",{}).get(reviewed.ETALON_URL) if (qualia / "Cargo.toml").is_file() else None
    normalized = {}
    if overlays or table:
        paths = reviewed.verify_captured(root,overlays)
        if not isinstance(table,dict) or set(table) != reviewed.OVERLAY_NAMES:
            raise ValueError("Captured canonical Etalon patch table differs")
        for name,path in paths.items():
            spec = table[name]
            if packages.get(name) != path or not isinstance(spec,dict) or set(spec) != {"path"} or (qualia / spec["path"] / "Cargo.toml").resolve() != path:
                raise ValueError(f"Captured Etalon overlay package/patch identity differs: {name}")
            normalized[name] = {**overlays[name],"directory":str(path.parent),"manifest_path":str(path),"source_manifest_path":str(path)}
            indexed = {entry["path"]:entry for entry in receipt["files"]}
            for relative,digest in overlays[name]["source_files"].items():
                captured = overlays[name]["directory"] + "/" + ("Cargo.toml.source" if name == "etalon-model-io" and relative == "Cargo.toml" else relative)
                source = indexed.get(captured,{})
                if source.get("source_sha256") != digest or Path(source.get("source","")) != Path(overlays[name]["source_directory"]) / relative:
                    raise ValueError(f"Captured Etalon per-file source provenance differs: {name}/{relative}")
    return {"root": root, "host_source": host, "qualia_source": qualia, "packages": packages,"reviewed_overlays":normalized}
