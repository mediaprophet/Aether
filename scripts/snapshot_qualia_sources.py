#!/usr/bin/env python3
# Copyright (c) 2026 Timothy Charles Holborn.
# SPDX-License-Identifier: CC-BY-NC-ND-4.0
"""Capture real canonical sources for repeatable local qualification.

Never changes the source repositories. Copies only resolved canonical crate
directories, host sources, licensing/workspace metadata and referenced build
inputs. Hashes sources before/after capture and retries a changed inventory.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tomllib
from datetime import datetime, timezone
from pathlib import Path
from source_snapshot import authority_transport_closure, validate_transport_profile
import reviewed_etalon_overlays as reviewed


def git(root: Path, *args: str) -> bytes:
    return subprocess.check_output(["git", "--no-optional-locks", "-c", "core.safecrlf=false", "-C", str(root), *args])


def paths(raw: bytes) -> set[str]:
    return {p.decode("utf-8").replace("\\", "/") for p in raw.split(b"\0") if p}


def digest(path: Path) -> str:
    result = hashlib.sha256()
    with path.open("rb") as stream:
        while block := stream.read(1024 * 1024):
            result.update(block)
    return result.hexdigest()


def utc() -> str:
    return datetime.now(timezone.utc).isoformat()


FORBIDDEN = {".git", ".aws", ".ssh", ".codex", "target", "node_modules", "__pycache__"}
SOURCE_SUFFIXES = {
    ".rs", ".toml", ".md", ".json", ".txt", ".vibe", ".wgsl", ".glsl", ".hlsl",
    ".metal", ".h", ".hpp", ".c", ".cc", ".cpp", ".s", ".py", ".sh", ".ps1",
    ".bat", ".svg", ".png", ".bin", ".ftw", ".qlib", ".lock",
}
INCLUDE = re.compile(r'\binclude(?:_str|_bytes)?!\s*\(\s*"([^"\n]+)"')


def aether_capture_packages(aether: Path) -> dict[str, Path]:
    host = aether / "packages/qualia-host/Cargo.toml"
    if tomllib.loads(host.read_text(encoding="utf-8-sig"))["package"]["name"] != "aether-qualia-host":
        raise ValueError("Unexpected Aether host package identity")
    return {"aether-qualia-host": host, **authority_transport_closure(host, aether)}


def safe_source(root: Path, relative: str) -> Path:
    path = root / relative
    if not path.resolve().is_relative_to(root):
        raise ValueError(f"source escapes repository: {relative}")
    if any(part in FORBIDDEN for part in Path(relative).parts) or path.name.startswith(".env"):
        raise ValueError(f"non-source input refused: {relative}")
    return path


def inventory(root: Path, prefixes: list[str], extras: list[str]) -> tuple[set[str], set[str], set[str]]:
    tracked = paths(git(root, "ls-files", "-z", "--", *prefixes, *extras))
    candidates = paths(git(root, "ls-files", "-z", "--cached", "--others", "--exclude-standard",
                           "--exclude=target/", "--exclude=node_modules/", "--exclude=__pycache__/",
                           "--exclude=.env*", "--", *prefixes, *extras))
    modified = paths(git(root, "diff", "HEAD", "--name-only", "-z", "--", *prefixes, *extras))
    selected: set[str] = set()
    for relative in candidates | set(extras):
        # Build output can be unignored in a newly created nested workspace.
        # Exclude it explicitly instead of copying any compiler/cache artifacts.
        if any(part in FORBIDDEN for part in Path(relative).parts) or Path(relative).name.startswith(".env"):
            continue
        path = safe_source(root, relative)
        if path.is_file() and (relative in tracked or relative in extras or path.suffix.lower() in SOURCE_SUFFIXES):
            selected.add(relative)
    # Explicit literal compile inputs can live outside their package directories.
    pending = list(selected)
    while pending:
        relative = pending.pop()
        if not relative.endswith(".rs"):
            continue
        source = root / relative  # already validated when admitted to selected
        for match in INCLUDE.finditer(source.read_text(encoding="utf-8")):
            included = (source.parent / match.group(1)).resolve()
            if included.is_file() and included.is_relative_to(root):
                found = included.relative_to(root).as_posix()
                safe_source(root, found)
                if found not in selected:
                    selected.add(found)
                    pending.append(found)
    return selected, tracked, modified


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--resume", action="store_true", help="resume this task's partial capture; refuses an existing receipt")
    parser.add_argument("--aether", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--qualia", type=Path, default=Path(__file__).resolve().parents[2] / "qualiaDB")
    parser.add_argument("--etalon-source", type=Path, help="Immutable Git6ace object source for the exact reviewed model overlays")
    args = parser.parse_args()
    aether, qualia, output = args.aether.resolve(), args.qualia.resolve(), args.output.resolve()
    if not output.is_relative_to(aether / "target") or (output.exists() and not args.resume):
        raise ValueError("output must be a new directory under Aether/target, or an explicitly resumed partial capture")
    if (output / "source-receipt.json").exists():
        raise ValueError("a completed capture is immutable; do not resume it")
    metadata = json.loads(args.metadata.read_text(encoding="utf-8-sig"))
    if not metadata.get("resolve"):
        raise ValueError("Full resolved Cargo metadata is required for source capture")
    validate_transport_profile(metadata)
    canonical = []
    for package in metadata["packages"]:
        manifest = Path(package["manifest_path"]).resolve()
        if manifest.is_relative_to(qualia):
            parsed = tomllib.loads(manifest.read_text(encoding="utf-8"))
            if parsed["package"]["name"] != package["name"]:
                raise ValueError("metadata/package source identity changed; regenerate metadata")
            canonical.append((package["name"], manifest.relative_to(qualia).as_posix()))
    required = {"qualia-core-db", "qualia-cooperative-core", "vibe", "vibescript-abi"}
    if not required <= {name for name, _ in canonical}:
        raise ValueError("metadata does not contain the canonical SDK closure")
    # Workspace patches must remain resolvable when testing the real canonical
    # workspace, even when a particular consumer does not activate that package.
    workspace_manifest = tomllib.loads((qualia / "Cargo.toml").read_text(encoding="utf-8"))
    overlay_sources, overlay_objects = {}, {}
    if reviewed.ETALON_URL in workspace_manifest.get("patch", {}):
        if not args.etalon_source:
            raise ValueError("Reviewed model overlays require --etalon-source with immutable Git6ace objects")
        overlay_sources, overlay_objects = reviewed.verify_live(qualia,args.etalon_source.resolve())
    for registry,table in workspace_manifest.get("patch", {}).items():
        for name,specification in table.items():
            if registry == reviewed.ETALON_URL and name == "etalon-model-io":
                continue  # The exact external18-file closure is captured separately.
            if not isinstance(specification, dict) or "path" not in specification:
                continue
            directory = (qualia / specification["path"]).resolve()
            if not directory.is_relative_to(qualia):
                raise ValueError("canonical workspace patch escapes captured source root")
            manifest = directory / "Cargo.toml"
            package = tomllib.loads(manifest.read_text(encoding="utf-8"))["package"]
            captured = (package["name"], manifest.relative_to(qualia).as_posix())
            if captured not in canonical:
                canonical.append(captured)
    workspace_packages = [(name, manifest) for name, manifest in canonical
                          if Path(manifest).parent.parent == Path("crates")]
    roots = {"Aether": aether, "qualiaDB": qualia}
    if overlay_sources:
        roots["etalon-model-io"] = Path(overlay_sources["etalon-model-io"]["source_directory"])
    aether_packages = aether_capture_packages(aether)
    prefixes = {"Aether": [manifest.parent.relative_to(aether).as_posix() for manifest in aether_packages.values()],
                "qualiaDB": [str(Path(manifest).parent).replace("\\", "/") for name, manifest in canonical if not (overlay_sources and name == "etalon-model")]}
    extras = {label: [name for name in ("Cargo.toml", "Cargo.lock", "AGENTS.md", "LICENSE", "LICENSE-QUALIA", "NOTICE", "rust-toolchain.toml") if (root / name).is_file()]
              for label, root in roots.items()}
    # The frozen host is an independent package, so it needs licensing but not
    # the live Aether workspace's member/lock graph.
    extras["Aether"] = [p for p in extras["Aether"] if p not in {"Cargo.toml", "Cargo.lock"}]
    if overlay_sources:
        prefixes["etalon-model-io"] = []
        extras["qualiaDB"].extend("vendor/etalon-model-6ace6cc/" + name for name in overlay_sources["etalon-model"]["source_files"])
    if (aether / "upstream-licenses").is_dir():
        prefixes["Aether"].append("upstream-licenses")
    # qualia-core-db/build.rs stages these SDK inputs on Windows independently
    # of the optional DirectML feature. Preserve actual local bytes and notices.
    for name in ("dxcompiler.dll", "dxil.dll", "dxc.exe"):
        relative = f"vendor/dxc/bin/x64-win/{name}"
        if (qualia / relative).is_file():
            extras["qualiaDB"].append(relative)
    for path in (qualia / "vendor/dxc").glob("*"):
        if path.is_file() and (path.name.upper().startswith(("LICENSE", "NOTICE", "README"))):
            extras["qualiaDB"].append(path.relative_to(qualia).as_posix())
    started = utc()
    output.mkdir(parents=True, exist_ok=args.resume)
    previous = {(Path(base) / name).relative_to(output).as_posix()
                for base, _, names in os.walk(output) for name in names}
    if any(any(part in FORBIDDEN for part in Path(name).parts) for name in previous):
        raise ValueError("partial capture contains unexpected cache/build files")
    records = []
    def source_inventory(label):
        if label == "etalon-model-io":
            return set(overlay_sources[label]["source_files"]), set(), set()
        return inventory(roots[label],prefixes[label],extras[label])
    for attempt in range(1, 9):
        source_heads = {label: git(root, "rev-parse", "HEAD").decode().strip() for label, root in roots.items()}
        try:
            if overlay_sources:
                current_overlays,_ = reviewed.verify_live(qualia,args.etalon_source.resolve())
                if current_overlays != overlay_sources: raise ValueError("Reviewed overlay changed before capture")
            before = {label: source_inventory(label) for label in roots}
            selected = [(label, relative) for label, (names, _, _) in before.items() for relative in sorted(names)]
            print(f"Verifying {len(selected)} source/build-input files (attempt {attempt}).", flush=True)
            source_hashes = {(label, relative): digest(roots[label] / relative) for label, relative in selected}
            current = {f"{label}/{relative}" for label, relative in selected}
            for obsolete in previous - current:
                destination = output / obsolete
                if not destination.resolve().is_relative_to(output):
                    raise ValueError("invalid snapshot target")
                destination.unlink(missing_ok=True)  # exact previously-copied file only
            previous = current
            copied = 0
            for label, relative in selected:
                destination = output / label / relative
                if destination.is_file() and digest(destination) == source_hashes[label, relative]:
                    continue
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(roots[label] / relative, destination)
                copied += 1
            print(f"Copied {copied} changed files; checking the source again.", flush=True)
            after = {label: source_inventory(label) for label in roots}
            if overlay_sources:
                current_overlays,_ = reviewed.verify_live(qualia,args.etalon_source.resolve())
                if current_overlays != overlay_sources: raise ValueError("Reviewed overlay changed during capture")
            stable = all(before[label][0] == after[label][0] for label in roots)
            stable &= all((roots[label] / relative).is_file()
                          and digest(roots[label] / relative) == source_hashes[label, relative]
                          and digest(output / label / relative) == source_hashes[label, relative]
                          for label, relative in selected)
            stable &= all(git(root, "rev-parse", "HEAD").decode().strip() == source_heads[label]
                          for label, root in roots.items())
        except FileNotFoundError:
            print("A source file moved during capture; retrying the affected inventory.", flush=True)
            continue
        if stable:
            for label, relative in selected:
                _, tracked, modified = after[label]
                state = "tracked_modified" if relative in modified else "tracked" if relative in tracked else "untracked_or_explicit_build_input"
                records.append({"path": f"{label}/{relative}", "sha256": source_hashes[label, relative],
                                "source": str(roots[label] / relative), "source_sha256": source_hashes[label, relative], "git_state": state})
            break
        print(f"Source changed during capture attempt {attempt}; retrying explicit source inventory.", flush=True)
    else:
        raise RuntimeError("source did not stabilize; no qualification receipt was issued")

    by_path = {record["path"]: record for record in records}
    def transform(relative: str, rewritten: str, reason: str) -> None:
        target = output / relative
        original = target.with_name(target.name + ".source")
        shutil.copyfile(target, original)
        original_record = dict(by_path[relative], path=original.relative_to(output).as_posix())
        records.append(original_record)
        target.write_text(rewritten, encoding="utf-8", newline="\n")
        by_path[relative]["sha256"] = digest(target)
        by_path[relative]["transform"] = reason

    host = "Aether/packages/qualia-host/Cargo.toml"
    text = (output / host).read_text(encoding="utf-8")
    text, count = re.subn(r'aether-core\s*=\s*\{\s*path\s*=\s*"\.\./core"\s*\}',
                         f'aether-core = {{ path = "{(aether / "packages/core").as_posix()}" }}', text)
    if count != 1:
        raise ValueError("unexpected host UI dependency; review source mapping")
    transform(host, text, "Only aether-core path rewritten to its actual Aether source; canonical sibling SDK paths preserved.")
    workspace = "qualiaDB/Cargo.toml"
    text = (output / workspace).read_text(encoding="utf-8")
    members = [str(Path(manifest).parent).replace("\\", "/") for _, manifest in workspace_packages]
    text, count = re.subn(r'(?ms)^members\s*=\s*\[.*?^\]', "members = " + json.dumps(sorted(members), indent=4), text, count=1)
    if count != 1:
        raise ValueError("unexpected canonical workspace member declaration")
    if overlay_sources:
        original_io_path = workspace_manifest["patch"][reviewed.ETALON_URL]["etalon-model-io"]["path"]
        original_line = f'etalon-model-io = {{ path = "{original_io_path}" }}'
        if text.count(original_line) != 1: raise ValueError("Unexpected canonical model IO patch formatting")
        text = text.replace(original_line,'etalon-model-io = { path = "../etalon-model-io" }')
    transform(workspace, text, "Workspace members restricted to resolved canonical closure; exact reviewed external model IO patch relocated into its captured standalone package; all other dependency/source/features preserved.")
    # Cargo can otherwise discover the enclosing Aether workspace when this
    # captured tree lives below Aether/target, despite the inner member list.
    # Bind every captured package explicitly and retain its original manifest.
    for _, manifest in workspace_packages:
        relative = f"qualiaDB/{manifest}"
        text = (output / relative).read_text(encoding="utf-8")
        package = tomllib.loads(text)["package"]
        if "workspace" in package:
            if package["workspace"] != "../../":
                raise ValueError(f"unexpected canonical workspace binding: {manifest}")
            continue
        if Path(manifest).parent.parent != Path("crates") or text.count("[package]") != 1:
            raise ValueError(f"unexpected canonical package layout: {manifest}")
        text = text.replace("[package]", '[package]\nworkspace = "../../"', 1)
        transform(relative, text, "Explicit package.workspace binds the captured package to its canonical workspace, avoiding outer Aether workspace discovery; dependencies, source and features preserved.")
    captured_overlays = {}
    if overlay_sources:
        io_source = Path(overlay_sources["etalon-model-io"]["source_directory"])
        io_files = reviewed.io_materialization(io_source,overlay_objects)
        transform("etalon-model-io/Cargo.toml",io_files["Cargo.toml"].decode(),"Package/lints/registry dependency inheritance materialized only from immutable Git6ace workspace metadata; original reviewed manifest preserved; standalone workspace added.")
        pin = args.etalon_source.resolve()
        generated = {"PINNED-WORKSPACE.toml":"Cargo.toml",**{name:name for name in reviewed.review()["model_io"]["license_files"]}}
        for name,origin in generated.items():
            destination = output / "etalon-model-io" / name
            destination.parent.mkdir(parents=True,exist_ok=True)
            original_hash = digest(pin / origin)
            destination.write_bytes(io_files[name])
            if digest(pin / origin) != original_hash: raise ValueError("Pinned overlay locator changed during capture")
            records.append({"path":destination.relative_to(output).as_posix(),"sha256":digest(destination),
                "source":str(pin / origin),"source_sha256":original_hash,"git_state":"immutable_git_object_materialization",
                "transform":"Exact immutable Git6ace object bytes, preserving original license/metadata; explicit LF vs worktree-CRLF provenance",
                "git_revision":reviewed.REVISION,"git_object_path":origin,"git_blob_sha256":reviewed.sha(overlay_objects[origin])})
        for name,entry in overlay_sources.items():
            directory = "etalon-model-io" if name == "etalon-model-io" else "qualiaDB/vendor/etalon-model-6ace6cc"
            captured_overlays[name] = {**entry,"directory":directory,
                "files":reviewed.inventory(output / directory),"pinned_objects_verified":True}
    receipt = {
        "schema": "aether.qualia-source-snapshot.v1", "capture_started_utc": started,
        "capture_completed_utc": utc(), "stable_pre_post_hashes": True,
        "host_source": "Aether/packages/qualia-host", "qualia_source": "qualiaDB",
        "repositories": [{"root": str(root), "head": source_heads[label]} for label, root in roots.items()],
        "capture_tool": {"path": str(Path(__file__).resolve()), "sha256": digest(Path(__file__)),
                         "dependencies": [{"path": str(Path(__file__).with_name(name).resolve()),"sha256":digest(Path(__file__).with_name(name))}
                                          for name in ("source_snapshot.py","reviewed_etalon_overlays.py","reviewed_etalon_overlays.json")]},
        "packages": [{"name": name, "path": "Aether/" + manifest.relative_to(aether).as_posix()} for name, manifest in sorted(aether_packages.items())] + [{"name": name, "path": f"qualiaDB/{manifest}"} for name, manifest in sorted(canonical)] + ([{"name":"etalon-model-io","path":"etalon-model-io/Cargo.toml"}] if overlay_sources else []),
        "reviewed_etalon_overlays": captured_overlays,
        "profile": {"CARGO_PROFILE_DEV_DEBUG": "0", "CARGO_PROFILE_TEST_DEBUG": "0", "CARGO_BUILD_JOBS": "1", "qualia-core-db-default-features": False},
        "external_source_bindings": [{"name": "aether-core", "manifest": str(aether / "packages/core/Cargo.toml"), "manifest_sha256": digest(aether / "packages/core/Cargo.toml")}],
        "authority_transport_profile": {"kind": "library-pump", "excluded_features": {"aether-chora-client": ["test-fixture"]}, "worker_engine_captured": False},
        "files": sorted(records, key=lambda item: item["path"]),
    }
    receipt_path = output / "source-receipt.json"
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"receipt": str(receipt_path), "files": len(records), "packages": len(receipt["packages"]),
                      "bytes": sum((output / record["path"]).stat().st_size for record in records)}))


if __name__ == "__main__":
    main()
