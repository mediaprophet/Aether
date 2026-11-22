"""Content-bound, explicitly selected live integration sources.

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tomllib

ETALON_URL = "https://github.com/mediaprophet/etalon.git"
QUALIA_URL = "https://github.com/mediaprophet/qualiaDB.git"
VIBE_URL = "https://github.com/mediaprophet/vibe-script.git"


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def file_digest(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def inventory(root: Path, scopes: list[str]) -> dict:
    """Hash every tracked/nonignored source input in the selected scope.

    Git's index supplies names only: hashes always describe working-tree bytes.
    Missing tracked files and untracked additions are therefore observable too.
    """
    root = root.resolve()
    print(f"Inventorying live source {root} ...", file=sys.stderr, flush=True)
    names = subprocess.check_output(
        ["git", "-C", str(root), "ls-files", "-z", "--cached", "--others",
         "--exclude-standard", "--", *scopes]).decode("utf-8").split("\0")
    files = {}
    parents = {root: True}

    def reparse(info: os.stat_result) -> bool:
        return stat.S_ISLNK(info.st_mode) or bool(
            getattr(info, "st_file_attributes", 0) & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400))

    def valid_parent(path: Path) -> bool:
        if path in parents:
            return parents[path]
        if not path.is_relative_to(root):
            raise ValueError(f"Live source parent escapes the selected root: {path}")
        if not valid_parent(path.parent):
            parents[path] = False
            return False
        try:
            info = path.lstat()
        except FileNotFoundError:
            parents[path] = False
            return False
        if reparse(info) or not stat.S_ISDIR(info.st_mode):
            raise ValueError(f"Live source parent is a link, reparse point or non-directory: {path}")
        parents[path] = True
        return True

    byte_count = 0
    for name in sorted(set(names) - {""}):
        relative = Path(name)
        if not relative.parts or relative.anchor or ".." in relative.parts:
            raise ValueError(f"Live source inventory path escapes the selected root: {name}")
        path = root / relative
        info = None
        if valid_parent(path.parent):
            try:
                info = path.lstat()
            except FileNotFoundError:
                pass
        if info is not None and (reparse(info) or not stat.S_ISREG(info.st_mode)):
            raise ValueError(f"Live source is a link, reparse point or non-file: {path}")
        files[name] = file_digest(path) if info is not None else None
        byte_count += info.st_size if info is not None else 0
    payload = json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
    result = {"root": root.as_posix(), "scopes": scopes,
            "head": git(root, "rev-parse", "HEAD"),
            "status": git(root, "status", "--porcelain", "--untracked-files=all", "--", *scopes),
            "files": files, "sha256": hashlib.sha256(payload).hexdigest()}
    print(f"Inventoried {root}: {len(files)} paths, {byte_count} bytes", file=sys.stderr, flush=True)
    return result


def capture_sources(aether: Path, etalon: Path, qualia: Path, hypermedia: Path) -> dict:
    roots = {"aether": aether, "etalon": etalon, "qualia": qualia, "hypermedia": hypermedia}
    result = {}
    for name, root in roots.items():
        # Documentation, fixtures, licenses and CI can be build inputs too.
        # Exclude only explicit build output and Python bytecode caches.
        scopes = [".", ":(exclude)target",
                  ":(exclude)scripts/__pycache__", ":(exclude)scripts/tests/__pycache__"]
        if name == "etalon":
            scopes.append(":(exclude)target-container-standards")
        result[name] = inventory(root, scopes)
    return result


def verify_sources(states: dict) -> None:
    if set(states) != {"aether", "etalon", "qualia", "hypermedia"}:
        raise ValueError("Live profile requires all four source inventories")
    for name, expected in states.items():
        actual = inventory(Path(expected["root"]), expected["scopes"])
        if actual != expected:
            raise ValueError(f"Live source state changed since profile preparation: {name}")


def dependency_patches(hypermedia: Path, qualia: Path, etalon_packages: dict) -> dict:
    """Reproduce the dependency's local config without accepting foreign roots."""
    result = {}
    config = hypermedia / ".cargo/config.toml"
    if not config.exists():
        return result
    tables = tomllib.loads(config.read_text(encoding="utf-8-sig")).get("patch", {})
    for registry, packages in tables.items():
        if registry not in {QUALIA_URL, VIBE_URL, ETALON_URL}:
            raise ValueError(f"Live Hypermedia patch registry requires review: {registry}")
        for name, spec in packages.items():
            if not isinstance(spec, dict) or set(spec) != {"path"}:
                raise ValueError(f"Live dependency patch must select an inventoried path: {name}")
            path = (hypermedia / spec["path"]).resolve()
            expected = etalon_packages.get(name) if registry == ETALON_URL else qualia / "crates" / name
            if expected is None or path != expected.resolve():
                raise ValueError(f"Live dependency patch leaves the canonical package: {name}")
            identity = tomllib.loads((path / "Cargo.toml").read_text(encoding="utf-8-sig"))["package"]
            if identity["name"] != name:
                raise ValueError(f"Live dependency patch package identity differs: {name}")
            result.setdefault(registry, {})[name] = path
    return result


def verify_metadata(metadata: dict, states: dict, etalon_packages: dict,
                    consumer_manifest: Path | None = None) -> None:
    manifests = set()
    for state in states.values():
        manifests.update((Path(state["root"]) / name).resolve()
                         for name, digest in state["files"].items()
                         if digest is not None and Path(name).name == "Cargo.toml")
    canonical = Path(states["qualia"]["root"])
    errors = []
    for package in metadata["packages"]:
        name = package["name"]
        path = Path(package["manifest_path"]).resolve()
        is_consumer = (consumer_manifest is not None and path == consumer_manifest.resolve()
                       and name == "aether-local-integration")
        if package.get("source") is None and not is_consumer and path not in manifests:
            errors.append(f"Path package is outside live source inventories: {name} at {path}")
        if name.startswith("etalon") or (name in {"naga", "naga-types"} and not str(package.get("source", "")).startswith("registry+")):
            expected = etalon_packages.get(name)
            if expected is None or package.get("source") is not None or path != expected / "Cargo.toml":
                errors.append(f"Live GPU package does not select the canonical manifest: {name}")
        if name.startswith(("qualia-", "webizen-")) or name in {"vibe", "vibescript-abi"}:
            if package.get("source") is not None or path != canonical / "crates" / name / "Cargo.toml":
                errors.append(f"Live canonical package source differs: {name}")
    if errors:
        raise ValueError("\n".join(errors))


def verify_host_test_root(metadata: dict, host_source: Path) -> None:
    """Admit only the actual standalone host workspace as a live test root."""
    host_source = host_source.resolve()
    matches = [package for package in metadata["packages"]
               if package["name"] == "aether-qualia-host"]
    if len(matches) != 1:
        raise ValueError("Live host test root requires exactly one actual host package")
    host = matches[0]
    identity = host.get("id")
    if (host.get("source") is not None
            or Path(host["manifest_path"]).resolve() != host_source / "Cargo.toml"
            or not identity or set(metadata.get("workspace_members", [])) != {identity}
            or metadata["resolve"].get("root") != identity
            or Path(metadata.get("workspace_root", "")).resolve() != host_source):
        raise ValueError("Live host test root must be the exact standalone host workspace and resolver root")
