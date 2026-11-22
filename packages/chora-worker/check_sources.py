# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
"""Validate real worker Cargo metadata, workspace boundaries and reviewed sources.

Generate input with cargo metadata --locked --format-version 1 --manifest-path
packages/chora-worker/Cargo.toml --filter-platform x86_64-pc-windows-msvc.
This gate establishes source identity, not compilation or OS containment.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import tomllib

WORKER = Path(__file__).resolve().parent
CHORA = WORKER.parents[2] / "chora"


def violations(document, manifest, lock, overlay, gpu):
    errors = []
    if Path(document.get("workspace_root", "")).resolve() != WORKER:
        errors.append("worker must have its own workspace root")
    packages = document.get("packages", [])
    resolve = document.get("resolve")
    if not resolve or not resolve.get("nodes"):
        return errors + ["full resolved Cargo metadata is required"]
    by_id = {package["id"]: package for package in packages}
    nodes = {node["id"]: node for node in resolve["nodes"]}
    root = resolve.get("root")
    if root not in by_id or Path(by_id[root]["manifest_path"]).resolve() != WORKER / "Cargo.toml":
        errors.append("metadata root must be the actual worker manifest")
    if document.get("workspace_members") != [root]:
        errors.append("worker must be the only workspace member")
    dependencies = manifest.get("dependencies", {})
    expected = CHORA / "components/servo/Cargo.toml"
    edge = dependencies.get("chora", {})
    if (edge.get("package") != "servo" or edge.get("default-features") is not False
            or edge.get("features") != ["embedded_library"]
            or (WORKER / edge.get("path", "") / "Cargo.toml").resolve() != expected.resolve()):
        errors.append("chora edge must select the actual embedded-library fork")
    actual = [package for package in packages if package["name"] == "servo"]
    if len(actual) != 1 or actual[0].get("source") is not None or Path(actual[0]["manifest_path"]).resolve() != expected.resolve():
        errors.append("resolved engine is not the actual sibling Chora package")
    elif "embedded_library" not in nodes.get(actual[0]["id"], {}).get("features", []):
        errors.append("actual engine embedded_library feature is missing")
    protocol_manifest = WORKER.parent / "chora-bridge-protocol/Cargo.toml"
    protocol_edge = dependencies.get("protocol", {})
    protocol = [package for package in packages if package["name"] == "aether-chora-bridge-protocol"]
    if (protocol_edge.get("package") != "aether-chora-bridge-protocol"
            or (WORKER / protocol_edge.get("path", "") / "Cargo.toml").resolve() != protocol_manifest.resolve()
            or len(protocol) != 1 or protocol[0].get("source") is not None
            or Path(protocol[0]["manifest_path"]).resolve() != protocol_manifest.resolve()):
        errors.append("worker protocol must resolve to the actual sibling protocol manifest")
    forbidden = {"multiprocess", "media-gstreamer", "default_tooling", "devtools"}
    if actual and forbidden.intersection(nodes.get(actual[0]["id"], {}).get("features", [])):
        errors.append("worker enabled unsupported engine features")
    bound = manifest.get("patch", {}).get(gpu.HYPERMEDIA_URL, {})
    if set(bound) != set(overlay):
        errors.append("worker must retain all three reviewed overlay patches")
    for name, entry in bound.items():
        if name not in overlay or (WORKER / entry.get("path", "") / "Cargo.toml").resolve() != overlay[name]["manifest_path"]:
            errors.append(f"worker overlay path differs: {name}")
    errors.extend(gpu.violations(packages, overlay, require_all=False))
    errors.extend(gpu.violations(lock.get("package", []), overlay, require_all=False))
    required = {"etalon-core", "etalon-core-remote", "etalon-core-remote-types", "etalon-types", "etalon-hal", "naga", "naga-types", "aether-chora-bridge-protocol"}
    errors.extend(f"missing worker dependency: {name}" for name in sorted(required - {p["name"] for p in packages}))
    for package in packages:
        name = package["name"]
        if name.startswith(("qualia", "vibe", "webizen", "blitz")) or (name.startswith("aether") and name not in {"aether-chora-worker", "aether-chora-bridge-protocol"}):
            errors.append(f"privileged/native package linked into worker: {name}")
    identities = {(p["name"], p["version"], p.get("source")) for p in lock.get("package", [])}
    for package in packages:
        if (package["name"], package["version"], package.get("source")) not in identities:
            errors.append(f"metadata is not represented in worker lock: {package['name']}")
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("metadata", type=Path)
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location("chora_gpu_policy", CHORA / "support/etalon/check_gpu_sources.py")
    gpu = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(gpu)
    overlay = gpu.overlay_sources(CHORA)
    manifest = tomllib.loads((WORKER / "Cargo.toml").read_text(encoding="utf-8"))
    lock = tomllib.loads((WORKER / "Cargo.lock").read_text(encoding="utf-8"))
    document = json.loads(args.metadata.read_text(encoding="utf-8-sig"))
    errors = violations(document, manifest, lock, overlay, gpu)
    if errors:
        raise SystemExit("Worker source gate FAILED:\n" + "\n".join(f"- {error}" for error in errors))
    print(f"Worker source gate passed: {len(document['packages'])} actual resolved packages; reviewed Etalon {gpu.ETALON_REV}, attributed Hypermedia {gpu.HYPERMEDIA_REV}. Compilation/runtime/containment remain separate.")


if __name__ == "__main__":
    main()
