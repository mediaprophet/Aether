"""Exact reviewed non-GPU Etalon overlays; never admit the mutable workspace."""
# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tomllib

REVISION = "6ace6cc6413a87ca10a1da58397ddc7c0b2fc41e"
ETALON_URL = "https://github.com/mediaprophet/etalon.git"
OVERLAY_NAMES = frozenset({"etalon-model-io", "etalon-model"})
REVIEW_PATH = Path(__file__).with_name("reviewed_etalon_overlays.json")

def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def reject_linked_ancestry(path: Path, boundary: Path) -> None:
    """Check lexical owned ancestors before resolve() can hide a junction."""
    candidate = boundary
    if not path.is_relative_to(boundary): raise ValueError("Overlay path escapes lexical source boundary")
    for part in (None,*path.relative_to(boundary).parts):
        if part == "..":
            if candidate == boundary: raise ValueError("Overlay path leaves lexical source boundary")
            candidate = candidate.parent
        elif part is not None: candidate = candidate / part
        if candidate.is_symlink() or (hasattr(candidate,"is_junction") and candidate.is_junction()):
            raise ValueError(f"Linked overlay source ancestry: {candidate}")

def review() -> dict:
    result = json.loads(REVIEW_PATH.read_text(encoding="utf-8"))
    if result["schema"] != "aether.reviewed-etalon-overlays.v1" or result["revision"] != REVISION:
        raise ValueError("Unknown Etalon overlay review")
    return result

def inventory(root: Path, excluded: frozenset[str] = frozenset()) -> dict[str, str]:
    """All files, regardless of suffix/ignore rules; only known root build dirs omitted."""
    if root.is_symlink() or (hasattr(root,"is_junction") and root.is_junction()):
        raise ValueError("Linked overlay root")
    result = {}
    for directory, directories, files in os.walk(root, followlinks=False):
        base = Path(directory)
        for name in directories + files:
            path = base / name
            if path.is_symlink() or (hasattr(path,"is_junction") and path.is_junction()):
                raise ValueError(f"Linked overlay input: {path}")
        if base == root:
            directories[:] = [name for name in directories if name not in excluded]
        for name in files:
            path = base / name
            result[path.relative_to(root).as_posix()] = sha(path.read_bytes())
    return result

def git_objects(pin: Path, names: list[str]) -> dict[str, bytes]:
    names = list(dict.fromkeys(names))
    if any(Path(name).is_absolute() or ".." in Path(name).parts or "\n" in name for name in names):
        raise ValueError("Invalid pinned overlay path")
    data = subprocess.check_output(["git", "-C", str(pin), "cat-file", "--batch"],
        input="".join(f"{REVISION}:{name}\n" for name in names).encode())
    if len(data) > 32 * 1024 * 1024:
        raise ValueError("Pinned overlay objects exceed review bound")
    result, position = {}, 0
    for name in names:
        end = data.index(b"\n",position)
        header = data[position:end].split()
        if len(header) != 3 or header[1] != b"blob":
            raise ValueError(f"Pinned overlay object missing: {name}")
        size = int(header[2]); position = end + 1
        if size > 4 * 1024 * 1024 or position + size >= len(data):
            raise ValueError("Invalid pinned overlay object size")
        result[name] = data[position:position+size]
        position += size + 1
    if position != len(data):
        raise ValueError("Unexpected pinned overlay object data")
    return result

def overlay_paths(qualia: Path, table: dict) -> dict[str, Path]:
    if set(table) != OVERLAY_NAMES:
        raise ValueError("Only the exact two reviewed Etalon overlay names are admitted")
    expected = {"etalon-model-io": qualia.parent / "etalon/crates/etalon-model-io",
                "etalon-model": qualia / "vendor/etalon-model-6ace6cc"}
    for name, spec in table.items():
        reject_linked_ancestry(expected[name],qualia.parent)
        if not isinstance(spec,dict) or set(spec) != {"path"} or (qualia / spec["path"]).resolve() != expected[name].resolve():
            raise ValueError(f"Etalon overlay source escapes reviewed location: {name}")
        reject_linked_ancestry((qualia / spec["path"]).absolute(),qualia.parent)
    return {name: path.resolve() for name,path in expected.items()}

def model_receipt(root: Path) -> tuple[dict, dict[str,str]]:
    approved = review()["model"]
    path = root / "BACKPORT-SHA256.json"
    raw = path.read_bytes()
    if sha(raw) != approved["backport_receipt_sha256"]:
        raise ValueError("Etalon model BACKPORT receipt differs from external review anchor")
    receipt = json.loads(raw)
    if receipt["origin"] != {"url":ETALON_URL,"revision":REVISION,"package":"crates/etalon-model"}:
        raise ValueError("Etalon model origin differs")
    if len(receipt["original_files"]) != approved["original_count"] or len(receipt["added_files"]) != approved["added_count"]:
        raise ValueError("Etalon model review inventory count differs")
    expected = {"BACKPORT-SHA256.json":sha(raw)}
    changed = []
    for entry in receipt["original_files"] + receipt["added_files"]:
        name = entry["path"]
        if Path(name).is_absolute() or ".." in Path(name).parts or name in expected:
            raise ValueError("Invalid or duplicate Etalon backport input")
        expected[name] = entry["backport_sha256"]
        if "pinned_sha256" in entry and entry["pinned_sha256"] != entry["backport_sha256"]:
            changed.append(name)
    if sorted(changed) != sorted(approved["changed_original_paths"]) or sorted(receipt["changed_original_paths"]) != sorted(changed):
        raise ValueError("Etalon model changed-original scope differs")
    if inventory(root,frozenset({"target-isolated"})) != expected:
        raise ValueError("Etalon model source inventory/hash drift")
    package = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8-sig"))
    kernel = package["dependencies"]["etalon-inference-kernel"]
    if package["package"]["name"] != "etalon-model" or package["package"]["version"] != "0.0.2" or kernel.get("git") != ETALON_URL or kernel.get("rev") != REVISION or "path" in kernel:
        raise ValueError("Etalon model inference-kernel identity differs")
    return receipt,expected

def verify_live(qualia: Path, pin: Path) -> tuple[dict,dict[str,bytes]]:
    table = tomllib.loads((qualia / "Cargo.toml").read_text(encoding="utf-8-sig")).get("patch",{}).get(ETALON_URL)
    if table is None: return {},{}
    paths = overlay_paths(qualia,table)
    approved = review()
    io_files = {name:entry["sha256"] for name,entry in approved["model_io"]["files"].items()}
    if inventory(paths["etalon-model-io"],frozenset({"target"})) != io_files:
        raise ValueError("Etalon model IO source inventory/hash drift")
    receipt,model_files = model_receipt(paths["etalon-model"])
    names = ["Cargo.toml",*approved["model_io"]["license_files"],
             *("crates/etalon-model-io/"+name for name in io_files),
             *("crates/etalon-model/"+entry["path"] for entry in receipt["original_files"])]
    objects = git_objects(pin,names)
    if sha(objects["Cargo.toml"]) != approved["model_io"]["workspace_sha256"]:
        raise ValueError("Pinned Etalon workspace metadata differs")
    for name,entry in approved["model_io"]["files"].items():
        original = objects["crates/etalon-model-io/"+name]
        if sha(original) != entry["pinned_sha256"]:
            raise ValueError("Pinned IO object differs")
        if entry["comparison"] == "CRLF-materialization-only" and (paths["etalon-model-io"] / name).read_bytes().replace(b"\r\n",b"\n") != original:
            raise ValueError("IO change exceeds exact reviewed CRLF materialization")
    for entry in receipt["original_files"]:
        if sha(objects["crates/etalon-model/"+entry["path"]]) != entry["pinned_sha256"]:
            raise ValueError("BACKPORT pinned object proof differs")
    for name,digest in approved["model_io"]["license_files"].items():
        if sha(objects[name]) != digest or (paths["etalon-model"] / name).read_bytes() != objects[name]:
            raise ValueError("Pinned overlay license/notice differs")
    result = {name:{"source_directory":str(paths[name]),"source_files":files,
                    "review_sha256":sha(REVIEW_PATH.read_bytes()),"revision":REVISION}
              for name,files in (("etalon-model-io",io_files),("etalon-model",model_files))}
    # Recheck every input after object/manifest inspection, not just before it.
    if inventory(paths["etalon-model-io"],frozenset({"target"})) != io_files or inventory(paths["etalon-model"],frozenset({"target-isolated"})) != model_files:
        raise ValueError("Etalon overlay changed during admission")
    return result,objects

def materialized_io_manifest(original: bytes, workspace: bytes) -> bytes:
    approved = review()["model_io"]
    if sha(original) != approved["files"]["Cargo.toml"]["sha256"] or sha(workspace) != approved["workspace_sha256"]:
        raise ValueError("Unreviewed IO manifest materialization input")
    parsed = tomllib.loads(original.decode("utf-8-sig")); inherited = tomllib.loads(workspace.decode("utf-8"))["workspace"]
    def value(item):
        if isinstance(item,bool): return str(item).lower()
        return json.dumps(item,ensure_ascii=False)
    package = {key:inherited["package"][key] if isinstance(item,dict) and item == {"workspace":True} else item for key,item in parsed["package"].items()}
    lines = ["# Derived from reviewed IO source and immutable Git6ace workspace metadata.","[package]"]
    lines += [f"{key} = {value(item)}" for key,item in package.items()]
    lines += ["", "[workspace]", 'resolver = "3"']
    tables = [("dependencies",parsed["dependencies"]),("target.'cfg(windows)'.dependencies",parsed["target"]["cfg(windows)"]["dependencies"]),("dev-dependencies",parsed["dev-dependencies"])]
    for title,dependencies in tables:
        lines += ["",f"[{title}]"]
        for name,spec in dependencies.items():
            if spec != {"workspace":True} or name not in {"sha2","memmap2","tempfile"}:
                raise ValueError("Unreviewed IO workspace dependency")
            dependency = inherited["dependencies"][name]
            if not isinstance(dependency,str): raise ValueError("IO inherited dependency is not pinned registry metadata")
            lines.append(f"{name} = {value(dependency)}")
    if parsed["lints"] != {"workspace":True}: raise ValueError("Unreviewed IO lint inheritance")
    for kind,rules in inherited["lints"].items():
        lines += ["",f"[lints.{kind}]"]
        lines += [f"{name} = {value(setting)}" for name,setting in rules.items()]
    result = ("\n".join(lines)+"\n").encode()
    tomllib.loads(result.decode())
    return result

def io_materialization(source: Path, objects: dict[str,bytes]) -> dict[str,bytes]:
    approved = review()["model_io"]
    files = {name:(source / name).read_bytes() for name in approved["files"]}
    if {name:sha(data) for name,data in files.items()} != {name:entry["sha256"] for name,entry in approved["files"].items()}:
        raise ValueError("IO source changed before materialization")
    original = files["Cargo.toml"]
    files["Cargo.toml.source"] = original
    files["PINNED-WORKSPACE.toml"] = objects["Cargo.toml"]
    files["Cargo.toml"] = materialized_io_manifest(original,objects["Cargo.toml"])
    for name,digest in approved["license_files"].items():
        if sha(objects[name]) != digest: raise ValueError("IO license materialization differs")
        files[name] = objects[name]
    return files

def stage_io(source: Path, destination: Path, objects: dict[str,bytes], write: bool = True) -> dict:
    files = io_materialization(source,objects)
    expected = {name:sha(data) for name,data in files.items()}
    if destination.exists():
        if inventory(destination,frozenset({"target"})) != expected:
            raise ValueError("Existing IO staging differs; no receipt/source overwrite allowed")
    else:
        if not write: raise ValueError("Reviewed IO staging is missing")
        for name,data in files.items():
            path = destination / name; path.parent.mkdir(parents=True,exist_ok=True); path.write_bytes(data)
    if inventory(destination,frozenset({"target"})) != expected:
        raise ValueError("Staged IO inventory differs")
    return {"directory":str(destination.resolve()),"files":expected,"original_manifest":"Cargo.toml.source",
            "workspace_metadata":"PINNED-WORKSPACE.toml","materialization":"exact source bytes; package/lints/registry dependencies from immutable Git6ace; explicit standalone workspace"}

def profile_overlays(qualia: Path, pin: Path, output: Path, write: bool = True) -> dict:
    entries,objects = verify_live(qualia,pin)
    if not entries: return {}
    source = Path(entries["etalon-model-io"]["source_directory"])
    destination = output / "reviewed-overlays/etalon-model-io"
    reject_linked_ancestry(destination,output)
    entries["etalon-model-io"].update(stage_io(source,destination,objects,write))
    reject_linked_ancestry(destination,output)
    for name,entry in entries.items():
        if name == "etalon-model": entry.update(directory=entry["source_directory"],files=entry["source_files"])
        entry["manifest_path"] = str(Path(entry["directory"]) / "Cargo.toml")
        entry["source_manifest_path"] = str(Path(entry["source_directory"]) / "Cargo.toml")
    # Copy/materialization is covered by a second complete source admission.
    after,_ = verify_live(qualia,pin)
    for name in entries:
        if after[name]["source_files"] != entries[name]["source_files"]:
            raise ValueError("Etalon overlay changed during profile preparation")
    return entries

def verify_captured(root: Path, entries: dict) -> dict[str,Path]:
    if set(entries) != OVERLAY_NAMES: raise ValueError("Captured Etalon overlay set differs")
    approved = review(); paths = {}
    for name,entry in entries.items():
        if entry["review_sha256"] != sha(REVIEW_PATH.read_bytes()) or entry["revision"] != REVISION:
            raise ValueError("Captured Etalon overlay review differs")
        expected = root / ("etalon-model-io" if name == "etalon-model-io" else "qualiaDB/vendor/etalon-model-6ace6cc")
        if entry["directory"] != expected.relative_to(root).as_posix():
            raise ValueError("Captured Etalon overlay path escapes reviewed closure")
        paths[name] = expected / "Cargo.toml"
        if name == "etalon-model":
            _,files = model_receipt(expected)
            original_files = files
        else:
            files = {key:value["sha256"] for key,value in approved["model_io"]["files"].items()}
            original_files = dict(files)
            files["Cargo.toml.source"] = files["Cargo.toml"]
            workspace = (expected / "PINNED-WORKSPACE.toml").read_bytes()
            files["PINNED-WORKSPACE.toml"] = approved["model_io"]["workspace_sha256"]
            files["Cargo.toml"] = sha(materialized_io_manifest((expected / "Cargo.toml.source").read_bytes(),workspace))
            files.update(approved["model_io"]["license_files"])
            if inventory(expected,frozenset({"target"})) != files:
                raise ValueError("Captured IO inventory/hash differs")
        if entry["files"] != files:
            raise ValueError("Captured Etalon overlay file receipt differs")
        if entry["source_files"] != original_files or not Path(entry["source_directory"]).is_absolute():
            raise ValueError("Captured Etalon original source provenance differs")
    return paths
