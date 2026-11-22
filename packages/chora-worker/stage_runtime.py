# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
"""Stage a successful Windows worker build using Cargo's exact artifact records.

This does not execute the worker. Keep cargo build --message-format=json output,
then supply that JSONL and the source-gated Windows metadata document. A unique
development stage preserves the original artifacts and matching ANGLE notices.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import tempfile

WORKER = Path(__file__).resolve().parent
REPOSITORY = WORKER.parents[1]


def digest(path):
    with path.open("rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def checked_output_directory(repository, destination):
    """Reject existing redirected output ancestors before creating/writing a stage."""
    root = repository.absolute()
    destination = destination.absolute()
    if ".." in destination.parts or not destination.is_relative_to(root):
        raise ValueError("Runtime stage destination must remain inside the declared repository")
    for ancestor in reversed((destination, *destination.parents)):
        try:
            info = ancestor.lstat()
        except FileNotFoundError:
            continue
        reparse = getattr(info, "st_file_attributes", 0) & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0)
        if stat.S_ISLNK(info.st_mode) or reparse or (hasattr(ancestor, "is_junction") and ancestor.is_junction()):
            raise ValueError(f"Redirected runtime stage ancestor: {ancestor}")
        if not stat.S_ISDIR(info.st_mode):
            raise ValueError(f"Runtime stage ancestor is not a directory: {ancestor}")
    if not root.is_dir() or not destination.resolve().is_relative_to(root.resolve()):
        raise ValueError("Runtime stage destination resolves outside the declared repository")
    return destination


def select_artifacts(messages, metadata, target_root):
    target_root = target_root.resolve()
    if Path(metadata.get("workspace_root", "")).resolve() != WORKER:
        raise ValueError("Metadata must describe the standalone worker workspace")
    worker_id = metadata.get("resolve", {}).get("root")
    packages = {package["id"]: package for package in metadata.get("packages", [])}
    if worker_id not in packages or Path(packages[worker_id]["manifest_path"]).resolve() != WORKER / "Cargo.toml":
        raise ValueError("Metadata root is not the actual worker package")
    angle = [package for package in packages.values() if package["name"] == "mozangle"]
    if len(angle) != 1:
        raise ValueError("Metadata must identify exactly one mozangle package")
    finished = [message for message in messages if message.get("reason") == "build-finished"]
    if len(finished) != 1 or finished[0].get("success") is not True:
        raise ValueError("One successful Cargo build-finished record is required")
    executables = {
        Path(message["executable"]).resolve()
        for message in messages
        if message.get("reason") == "compiler-artifact"
        and message.get("package_id") == worker_id
        and message.get("target", {}).get("name") == "aether-chora-worker"
        and message.get("target", {}).get("kind") == ["bin"]
        and message.get("executable")
    }
    out_dirs = {
        Path(message["out_dir"]).resolve()
        for message in messages
        if message.get("reason") == "build-script-executed"
        and message.get("package_id") == angle[0]["id"]
        and message.get("out_dir")
    }
    if len(executables) != 1 or len(out_dirs) != 1:
        raise ValueError("Build records must identify exactly one worker executable and matching ANGLE OUT_DIR")
    executable, out_dir = next(iter(executables)), next(iter(out_dirs))
    if executable.name != "aether-chora-worker.exe":
        raise ValueError("Only the actual Windows worker executable can be staged")
    if not executable.is_relative_to(target_root) or not out_dir.is_relative_to(target_root):
        raise ValueError("Cargo artifacts must remain within the explicitly selected target root")
    return executable, out_dir, angle[0]


def pe_imports(reader, path):
    result = subprocess.run([str(reader), "--coff-imports", str(path)], check=True, capture_output=True, text=True, encoding="utf-8")
    return sorted(set(re.findall(r"^\s*Name:\s*(\S+\.dll)\s*$", result.stdout, re.MULTILINE | re.IGNORECASE)))


def stage(messages_path, metadata_path, target_root, reader, native_archive_receipt=None):
    messages = [json.loads(line) for line in messages_path.read_text(encoding="utf-8-sig").splitlines() if line.strip()]
    metadata = json.loads(metadata_path.read_text(encoding="utf-8-sig"))
    executable, out_dir, angle = select_artifacts(messages, metadata, target_root)
    native_release_metadata = None
    if native_archive_receipt is not None:
        native = json.loads(native_archive_receipt.read_text(encoding="utf-8"))
        package = next(package for package in metadata["packages"] if package["name"] == "mozjs_sys")
        features = next(node["features"] for node in metadata["resolve"]["nodes"] if node["id"] == package["id"])
        if native["package"]["id"] != package["id"] or set(native["features"]) != set(features):
            raise ValueError("Native archive receipt differs from the actual SpiderMonkey package/features")
        archive = Path(native["archive"]["path"])
        actual_digest = digest(archive)
        if actual_digest != native["archive"]["sha256"] or "sha256:" + actual_digest != native["release_asset"]["digest"] or archive.stat().st_size != native["release_asset"]["size"]:
            raise ValueError("Native archive no longer matches its verified release digest/size")
        native_release_metadata = Path(native["release_metadata"]["path"])
        if digest(native_release_metadata) != native["release_metadata"]["sha256"]:
            raise ValueError("Native archive release metadata changed")
    sources = [executable, out_dir / "libEGL.dll", out_dir / "libGLESv2.dll"]
    for path in sources:
        if not path.is_file() or path.is_symlink() or not path.resolve().is_relative_to(target_root.resolve()):
            raise ValueError(f"Missing or redirected exact build artifact: {path}")
    if not reader.is_file():
        raise ValueError("An installed llvm-readobj executable is required for PE import inspection")
    imports = {path.name: pe_imports(reader, path) for path in sources}
    msvc_names = sorted({name for names in imports.values() for name in names if re.fullmatch(r"(?:vcruntime\d+.*|msvcp\d+.*|concrt\d+.*|ucrtbase)\.dll", name, re.IGNORECASE)})
    system_root = Path(os.environ.get("SystemRoot", "C:/Windows"))
    installed = {}
    for name in msvc_names:
        path = system_root / "System32" / name
        if not path.is_file():
            raise ValueError(f"Required MSVC runtime is not installed in System32: {name}; no system DLL was copied")
        installed[name] = {"path": str(path.resolve()), "sha256": digest(path)}

    base = checked_output_directory(REPOSITORY, REPOSITORY / "target/production-audit/chora-runtime")
    base.mkdir(parents=True, exist_ok=True)
    checked_output_directory(REPOSITORY, base)
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ-")
    destination = Path(tempfile.mkdtemp(prefix=timestamp, dir=base))
    checked_output_directory(REPOSITORY, destination)
    files = []

    def copy(source, relative):
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        # Each destination is new; source artifacts and unrelated files remain intact.
        if target.exists():
            raise ValueError(f"Duplicate stage destination: {relative}")
        before = digest(source)
        shutil.copy2(source, target)
        if before != digest(target) or before != digest(source):
            raise ValueError(f"Source changed during staging: {source}")
        files.append({"path": relative.as_posix(), "source": str(source.resolve()), "bytes": target.stat().st_size, "sha256": before})

    for source in sources:
        copy(source, Path(source.name))
    if native_archive_receipt is not None:
        copy(native_archive_receipt, Path("native-input-receipt.json"))
        copy(native_release_metadata, Path("native-input-release.json"))
    angle_root = Path(angle["manifest_path"]).parent
    notices = [path for path in angle_root.rglob("*") if path.is_file() and any(path.name.upper().startswith(prefix) for prefix in ("LICENSE", "COPYING", "NOTICE", "COPYRIGHT"))]
    if not (angle_root / "LICENSE") in notices or not (angle_root / "gfx/angle/checkout/LICENSE") in notices:
        raise ValueError("The actual mozangle and ANGLE license notices must be retained")
    for source in sorted(notices):
        if source.is_symlink() or not source.resolve().is_relative_to(angle_root.resolve()):
            raise ValueError(f"Redirected ANGLE notice: {source}")
        copy(source, Path("notices/mozangle") / source.relative_to(angle_root))
    for source, relative in [
        (REPOSITORY / "LICENSE", Path("notices/aether/LICENSE")),
        (REPOSITORY / "LICENSE-QUALIA", Path("notices/aether/LICENSE-QUALIA")),
        (REPOSITORY.parent / "chora/LICENSE", Path("notices/chora/LICENSE")),
    ]:
        if not source.is_file():
            raise ValueError(f"Missing repository license: {source}")
        copy(source, relative)
    inventory = [{key: package.get(key) for key in ("id", "name", "version", "source", "manifest_path", "license", "license_file")} for package in metadata["packages"]]
    inventory_path = destination / "dependency-licenses.json"
    inventory_path.write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
    receipt = {
        "purpose": "Local development runtime stage; not a complete distribution or runtime qualification",
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "worker_package_id": metadata["resolve"]["root"],
        "mozangle_package_id": angle["id"],
        "mozangle_out_dir": str(out_dir),
        "cargo_build_messages": {"path": str(messages_path.resolve()), "sha256": digest(messages_path)},
        "cargo_metadata": {"path": str(metadata_path.resolve()), "sha256": digest(metadata_path)},
        "worker_lock": {"path": str(WORKER / "Cargo.lock"), "sha256": digest(WORKER / "Cargo.lock")},
        "files": files,
        "dependency_license_inventory": {"path": inventory_path.name, "sha256": digest(inventory_path)},
        "pe_imports": imports,
        "installed_msvc_runtime_not_copied": installed,
        "remaining_release_gate": "Complete transitive distribution notices, installed artifacts and platform qualification",
    }
    receipt_path = destination / "stage-receipt.json"
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"executable": str(destination / executable.name), "receipt": str(receipt_path), "receipt_sha256": digest(receipt_path)}, indent=2))
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build_messages", type=Path)
    parser.add_argument("metadata", type=Path)
    parser.add_argument("--target-root", type=Path, default=REPOSITORY.parent / "chora/target")
    parser.add_argument("--llvm-readobj", type=Path, default=Path("C:/Program Files/LLVM/bin/llvm-readobj.exe"))
    parser.add_argument("--native-archive-receipt", type=Path, help="Retain and reverify the documented MOZJS_ARCHIVE input receipt")
    args = parser.parse_args()
    if os.name != "nt":
        raise SystemExit("This exact ANGLE staging profile targets Windows")
    stage(args.build_messages, args.metadata, args.target_root, args.llvm_readobj, args.native_archive_receipt)


if __name__ == "__main__":
    main()
