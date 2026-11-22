#!/usr/bin/env python3
"""Prepare an explicit local native/host/Chora consumer with one Etalon source.

The shipping workspace pins remain unchanged. The generated consumer uses local
working trees and is qualification evidence, not a reproducible release artifact.
Run Cargo against its manifest, then pass Cargo's full metadata to --verify.
"""
# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tomllib
from source_snapshot import AUTHORITY_TRANSPORTS, validate_transport_profile, verify_source_snapshot
from reviewed_etalon_overlays import OVERLAY_NAMES, profile_overlays
from live_integration_sources import (capture_sources, dependency_patches,
                                      verify_sources, verify_metadata, verify_host_test_root)

ROOT = Path(__file__).resolve().parents[1]
REVISION = "6ace6cc6413a87ca10a1da58397ddc7c0b2fc41e"
ETALON_URL = "https://github.com/mediaprophet/etalon.git"
HYPERMEDIA_URL = "https://github.com/mediaprophet/hypermedia-production.git"
ETALON_VERSION = "0.0.2"
SHIM_VERSION = "30.0.0"
BLITZ_PACKAGES = ("blitz-traits", "blitz-dom", "blitz-net")
TEST_ROOTS = {"host": "aether-qualia-host", "core": "qualia-core-db",
              "vibe": "vibe", "abi": "vibescript-abi", "protocol": "aether-chora-bridge-protocol",
              "client": "aether-chora-client"}
STYLO_URL = "https://github.com/servo/stylo"
ADAPTERS = (
    "gpu-device-context", "vector-gpu-render-adapter",
    "vector-cpu-render-adapter", "vector-hybrid-render-adapter",
    "vector-skia-render-adapter", "vector-compute-renderer",
    "vector-compute-shaders", "vector-hybrid-renderer",
    "vector-hybrid-shaders", "pixel-buffer-output", "pixel-window-render-adapter", "gpu-command-profiler",
)
GPU_ADAPTER_REQUIREMENTS = {"gpu-device-context", "vector-compute-renderer",
                            "vector-compute-shaders", "gpu-command-profiler"}


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def quote(value: str | Path) -> str:
    return json.dumps(value.as_posix() if isinstance(value, Path) else value)


def load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8-sig"))


def pinned_source(requested: Path | None) -> Path:
    if requested:
        candidates = [requested.resolve()]
    else:
        homes = [Path.home() / ".cargo"]
        if os.environ.get("CARGO_HOME"):
            homes.insert(0, Path(os.environ["CARGO_HOME"]))
        candidates = [path for home in homes for path in
                      (home / "git/checkouts").glob("etalon-*/6ace6cc*")]
    for candidate in candidates:
        # A Git checkout nested under another workspace can be implicitly
        # captured by that workspace when Cargo loads it as a path patch.
        if candidate.resolve().is_relative_to(ROOT):
            continue
        try:
            actual_revision = git(candidate, "rev-parse", "HEAD")
        except (OSError, subprocess.CalledProcessError):
            continue
        if actual_revision != REVISION:
            continue
        # Cargo's marker is expected; all actual source changes are rejected.
        dirty = git(candidate, "status", "--porcelain", "--untracked-files=all")
        if any(line != "?? .cargo-ok" for line in dirty.splitlines()):
            raise ValueError(f"Etalon source has local modifications: {candidate}")
        return candidate.resolve()
    raise ValueError(f"A clean Etalon {REVISION} checkout is required; use --etalon-source")


def etalon_packages(source: Path) -> dict[str, Path]:
    packages = {}
    for spec in load(source / "Cargo.toml")["workspace"]["dependencies"].values():
        if not isinstance(spec, dict) or "path" not in spec:
            continue
        member = (source / spec["path"]).resolve()
        name = load(member / "Cargo.toml").get("package", {}).get("name", "")
        if (name.startswith("etalon") or name in {"naga", "naga-types"}) and name != "etalon-test":
            packages[name] = member
    # The reviewed native/host graphs use Etalon directly. Do not introduce an
    # unused compatibility-shim patch into their active configuration. The
    # metadata gate still recognizes its exact identity when auditing legacy
    # consumers, and rejects all upstream implementations.
    return packages


def write_test_patches(manifest: Path, destination: Path) -> None:
    """Extract only existing absolute path patches for actual captured-package tests."""
    tables = load(manifest)["patch"]
    lines = ["# Derived from the explicit native integration consumer; qualification only."]
    for registry, packages in tables.items():
        lines.append("\n[patch." + quote(registry) + "]")
        for name, spec in packages.items():
            if set(spec) != {"path"} or not Path(spec["path"]).is_absolute():
                raise ValueError("Captured-package tests require absolute path-only consumer patches")
            lines.append(quote(name) + " = { path = " + quote(spec["path"]) + " }")
    destination.write_text("\n".join(lines) + "\n", encoding="utf-8")
    if load(destination)["patch"] != tables:
        raise ValueError("Extracted test patches differ from consumer patches")


def canonical_patch_sources(qualia: Path, snapshot: dict | None = None, overlays: dict | None = None) -> dict:
    """Retain canonical workspace patches that dependency manifests cannot propagate."""
    result = {}
    for registry, packages in load(qualia / "Cargo.toml").get("patch", {}).items():
        if registry == ETALON_URL:
            if set(packages) != OVERLAY_NAMES or not overlays or set(overlays) != OVERLAY_NAMES:
                raise ValueError("Canonical Etalon patch table requires the exact reviewed overlay admission")
            for name,spec in packages.items():
                entry = overlays[name]
                declared = (qualia / spec["path"] / "Cargo.toml").resolve() if isinstance(spec,dict) and set(spec) == {"path"} else None
                if declared != Path(entry["source_manifest_path"]):
                    raise ValueError(f"Canonical Etalon overlay path differs: {name}")
                manifest = Path(entry["manifest_path"])
                identity = load(manifest)["package"]
                if identity["name"] != name or identity["version"] != ETALON_VERSION:
                    raise ValueError(f"Canonical Etalon overlay identity differs: {name}")
                result[name] = {"registry":registry,"manifest_path":str(manifest),"version":ETALON_VERSION,
                                "sha256":hashlib.sha256(manifest.read_bytes()).hexdigest(),"reviewed_overlay":True}
            continue
        if registry not in {"crates-io", "https://github.com/mediaprophet/vibe-script.git"}:
            raise ValueError(f"Canonical patch table needs explicit integration review: {registry}")
        for name, spec in packages.items():
            if set(spec) != {"path"}:
                raise ValueError(f"Canonical patch requires explicit source review: {name}")
            manifest = (qualia / spec["path"] / "Cargo.toml").resolve()
            if not manifest.is_relative_to(qualia):
                raise ValueError(f"Canonical patch leaves selected source: {name}")
            identity = load(manifest)["package"]
            if identity["name"] != name or not isinstance(identity["version"], str):
                raise ValueError(f"Canonical patch identity differs: {name}")
            if snapshot and snapshot["packages"].get(name) != manifest:
                raise ValueError(f"Canonical workspace patch was not captured: {name}")
            result[name] = {"registry": registry, "manifest_path": str(manifest),
                            "version": identity["version"],
                            "sha256": hashlib.sha256(manifest.read_bytes()).hexdigest()}
    return result


def verify(metadata_path: Path, receipt_path: Path, test_root: str | None = None) -> None:
    receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    live = receipt.get("live_sources", False)
    live_host_test = live and test_root == "host"
    if live:
        if receipt.get("snapshot_receipt") or receipt.get("reviewed_etalon_overlays"):
            raise ValueError("Live sources cannot be combined with frozen snapshot/overlay verification")
        if test_root and (not live_host_test or not receipt.get("with_host")):
            raise ValueError("Live test-root verification supports only host with a host-enabled receipt")
        verify_sources(receipt["live_source_states"])
        source = Path(receipt["etalon_source"]).resolve()
        if git(source, "rev-parse", "HEAD") != receipt["etalon_revision"]:
            raise ValueError("Live Etalon revision differs from the profile")
    else:
        source = pinned_source(Path(receipt["etalon_source"]))
    metadata = json.loads(metadata_path.read_text(encoding="utf-8-sig"))
    if not metadata.get("resolve"):
        raise ValueError("Full resolved Cargo metadata is required (omit --no-deps)")
    if live:
        if live_host_test:
            verify_host_test_root(metadata, Path(receipt["host_source"]))
        manifest = receipt_path.parent / "Cargo.toml"
        verify_metadata(metadata, receipt["live_source_states"], etalon_packages(source), manifest)
        if hashlib.sha256(manifest.read_bytes()).hexdigest() != receipt["consumer_manifest_sha256"]:
            raise ValueError("Live consumer manifest changed since profile preparation")
        if hashlib.sha256((receipt_path.parent / "src/lib.rs").read_bytes()).hexdigest() != receipt["consumer_library_sha256"]:
            raise ValueError("Live consumer library changed since profile preparation")
    errors = []
    snapshot = None
    if receipt.get("snapshot_receipt"):
        snapshot_path = Path(receipt["snapshot_receipt"])
        if hashlib.sha256(snapshot_path.read_bytes()).hexdigest() != receipt["snapshot_receipt_sha256"]:
            raise ValueError("Canonical snapshot receipt changed since profile generation")
        snapshot = verify_source_snapshot(snapshot_path)
        validate_transport_profile(metadata)
        if receipt.get("with_chora_client"):
            raise ValueError("Frozen native Chora client composition is unsupported: live Aether UI selects live transport paths; use captured host standalone tests or the explicit live client profile")
        if str(snapshot["qualia_source"]) != str(Path(receipt["qualia_source"])) or str(snapshot["host_source"]) != str(Path(receipt["host_source"])):
            raise ValueError("Consumer source roots differ from the captured snapshot")
    if test_root and not live_host_test:
        if not snapshot or test_root not in TEST_ROOTS:
            raise ValueError("Test-root verification requires a captured canonical package")
        expected_name = TEST_ROOTS[test_root]
        if expected_name not in snapshot["packages"]:
            raise ValueError(f"Requested test root was not captured: {expected_name}")
        expected_manifest = snapshot["packages"][expected_name]
        members = set(metadata.get("workspace_members", []))
        if not any(p["name"] == expected_name and p.get("id") in members
                   and Path(p["manifest_path"]).resolve() == expected_manifest
                   and p.get("source") is None for p in metadata["packages"]):
            raise ValueError("Selected test root is not the captured workspace package")
    found = set()
    host_found = set()
    blitz_found = set()
    host_packages = {"vibe", "vibescript-abi", "qualia-core-db", "qualia-cooperative-core"}
    chora_found = False
    transport_found = set()
    transport_counts = {}
    client_consumer_found = set()
    resolved_features = {node["id"]: set(node.get("features", [])) for node in metadata["resolve"].get("nodes", [])}
    snapshot_found = set()
    adapter_found = set()
    stylo = receipt.get("stylo")
    canonical_patches = receipt.get("canonical_patches", {})
    overlays = receipt.get("reviewed_etalon_overlays", {})
    expected_overlays = snapshot.get("reviewed_overlays", {}) if snapshot else profile_overlays(Path(receipt["qualia_source"]),source,receipt_path.parent,False) if overlays else {}
    if overlays != expected_overlays:
        raise ValueError("Reviewed Etalon overlay source/inventory changed since profile preparation")
    for name, entry in canonical_patches.items():
        if hashlib.sha256(Path(entry["manifest_path"]).read_bytes()).hexdigest() != entry["sha256"]:
            errors.append(f"Canonical patch manifest changed: {name}")
    for name, digest in receipt["adapter_manifest_sha256"].items():
        manifest = Path(receipt["hypermedia_source"]) / "crates" / name / "Cargo.toml"
        if hashlib.sha256(manifest.read_bytes()).hexdigest() != digest:
            errors.append(f"Adapter manifest changed since this profile was prepared: {name}")
    for package in metadata["packages"]:
        name = package["name"]
        path = Path(package["manifest_path"]).resolve()
        if live_host_test and name in {"servo", "aether-native", "aether-native-dom", "aether-local-integration"}:
            errors.append(f"Live standalone host test graph contains an engine/native application package: {name}")
        if receipt.get("gpu_adapters_only") and name in {"qualia-core-db", "vibe", "aether-native", "aether-qualia-host"}:
            errors.append(f"GPU adapter qualification contains an application/engine dependency: {name}")
        if name in canonical_patches:
            entry = canonical_patches[name]
            if package.get("source") is not None or path != Path(entry["manifest_path"]) or package["version"] != entry["version"]:
                errors.append(f"Canonical workspace patch source differs: {name} at {path}")
        if snapshot and name in snapshot["packages"]:
            snapshot_found.add(name)
            if package.get("source") is not None or path != snapshot["packages"][name]:
                errors.append(f"Package differs from captured canonical source: {name} at {path}")
        elif snapshot and (path.is_relative_to(snapshot["root"])
                           or path.is_relative_to(ROOT.parent / "qualiaDB")
                           or name.startswith(("qualia-", "webizen-"))
                           or name in {"vibe", "vibescript-abi", "aether-qualia-host"}
                           or name in AUTHORITY_TRANSPORTS):
            errors.append(f"Canonical package is outside the captured dependency closure: {name} at {path}")
        if name in AUTHORITY_TRANSPORTS:
            transport_found.add(name)
            transport_counts[name] = transport_counts.get(name, 0) + 1
            if live_host_test and transport_counts[name] != 1:
                errors.append(f"Live host test graph contains duplicate authority transport: {name}")
            expected = (snapshot["packages"].get(name) if snapshot else
                        ROOT / AUTHORITY_TRANSPORTS[name] / "Cargo.toml")
            if package.get("source") is not None or path != expected:
                errors.append(f"Authority transport is outside selected source closure: {name} at {path}")
            if snapshot and name == "aether-chora-client" and "test-fixture" in resolved_features.get(package.get("id"), set()):
                errors.append("Captured client test-fixture requires uncaptured worker helper inputs; only library/pump profile is admitted")
            if live_host_test and "test-fixture" in resolved_features.get(package.get("id"), set()):
                errors.append(f"Live host test graph excludes transport test-fixture features: {name}")
        if receipt.get("with_chora_client"):
            if name == "servo":
                errors.append("Separate Chora client profile must not link the Chora engine")
            required_feature = {"aether-qualia-host": "chora-client", "aether-native": "chora"}.get(name)
            if required_feature:
                client_consumer_found.add(name)
                expected_consumer = Path(receipt["host_source"]) / "Cargo.toml" if name == "aether-qualia-host" else ROOT / "packages/native/Cargo.toml"
                if package.get("source") is not None or path != expected_consumer:
                    errors.append(f"Chora client consumer source differs: {name} at {path}")
            if required_feature and required_feature not in resolved_features.get(package.get("id"), set()):
                errors.append(f"Chora client profile feature missing: {name}/{required_feature}")
            if name == "aether-chora-client" and "test-fixture" in resolved_features.get(package.get("id"), set()):
                errors.append("Chora client library/pump profile excludes worker-backed test-fixture features")
        if receipt.get("engine_only") and (name.startswith(("qualia-", "webizen-")) or name in {"vibe", "vibescript-abi", "aether-native", "aether-qualia-host"}):
            errors.append(f"The generic engine graph contains host-specific package {name}")
        if name == "wgpu" or name.startswith(("wgpu-", "wgpu_")):
            if not (name == "wgpu" and path == source / "compat/wgpu/Cargo.toml"
                    and package.get("version") == SHIM_VERSION and package.get("source") is None):
                errors.append(f"upstream GPU implementation: {name} at {path}")
        # Registry Naga used as an independent shader-build tool is allowed by
        # the shipping gate too. Etalon's own Naga must share its exact source.
        forked_naga = name in {"naga", "naga-types"} and not str(package.get("source", "")).startswith("registry+")
        if name.startswith("etalon") or name == "wgpu" or forked_naga:
            reviewed_overlay = name in OVERLAY_NAMES and name in overlays and path == Path(overlays[name]["manifest_path"])
            if package.get("source") is not None or (not path.is_relative_to(source) and not reviewed_overlay):
                errors.append(f"GPU source differs from the canonical local checkout: {name} at {path}")
            found.add(name)
            expected_version = SHIM_VERSION if name == "wgpu" else ETALON_VERSION
            if package.get("version") != expected_version:
                errors.append(f"Unexpected canonical GPU package version: {name}@{package.get('version')}")
        if name in ADAPTERS:
            adapter_found.add(name)
            if name not in receipt["adapter_manifest_sha256"]:
                errors.append(f"Resolved adapter has no captured manifest hash: {name}")
            expected_manifest = Path(receipt["hypermedia_source"]) / "crates" / name / "Cargo.toml"
            if package.get("source") is not None or path != expected_manifest:
                errors.append(f"adapter is outside the selected local source: {name} at {path}")
            elif name in receipt["adapter_manifest_sha256"] and package["version"] != load(expected_manifest)["package"]["version"]:
                errors.append(f"Resolved adapter version differs from its captured manifest: {name}")
        if receipt.get("blitz_source") and name in BLITZ_PACKAGES:
            blitz_found.add(name)
            if package.get("source") is not None or path != Path(receipt["blitz_source"]) / name / "Cargo.toml":
                errors.append(f"Blitz must use the reviewed local network contract: {name} at {path}")
        if receipt.get("with_host") and name in host_packages:
            if package.get("source") is not None or path != Path(receipt["qualia_source"]) / "crates" / name / "Cargo.toml":
                errors.append(f"host SDK is outside the canonical local source: {name} at {path}")
            host_found.add(name)
        if receipt.get("with_chora") and name == "servo":
            chora_found = True
            if package.get("source") is not None or path != Path(receipt["chora_source"]) / "components/servo/Cargo.toml":
                errors.append(f"Chora must select the actual local fork: {path}")
        if stylo and name in stylo["packages"]:
            expected_source = f'git+{STYLO_URL}?rev={stylo["revision"]}#{stylo["revision"]}'
            if package.get("source") != expected_source:
                errors.append(f"Mixed Stylo source identity: {name} at {path}")
    implementation = "etalon-core" if receipt.get("engine_only") else "etalon"
    if (not test_root or live_host_test) and implementation not in found:
        errors.append("No Etalon implementation resolved; this graph cannot establish GPU coherence")
    if receipt.get("gpu_adapters_only") and not GPU_ADAPTER_REQUIREMENTS.issubset(adapter_found):
        errors.append("Required GPU adapters missing: " + ", ".join(sorted(GPU_ADAPTER_REQUIREMENTS - adapter_found)))
    if (not test_root or live_host_test) and receipt.get("with_host") and host_found != host_packages:
        errors.append("Required canonical host packages missing: " + ", ".join(sorted(host_packages - host_found)))
    # Canonical workspace patches are preserved even when an optional or
    # target-specific dependency does not activate them in this consumer.
    required_snapshot_packages = set(snapshot["packages"]) - set(canonical_patches) if snapshot else set()
    if not receipt.get("with_chora_client"):
        required_snapshot_packages.discard("aether-chora-client")
    if snapshot and not test_root and not required_snapshot_packages.issubset(snapshot_found):
        errors.append("Captured canonical packages missing from consumer graph: " + ", ".join(sorted(required_snapshot_packages - snapshot_found)))
    if receipt.get("with_chora") and not chora_found:
        errors.append("The requested Chora fork did not resolve")
    if receipt.get("with_chora_client") and not set(AUTHORITY_TRANSPORTS).issubset(transport_found):
        errors.append("Required authority transport packages missing: " + ", ".join(sorted(set(AUTHORITY_TRANSPORTS) - transport_found)))
    expected_consumers = {"aether-qualia-host"} if live_host_test else {"aether-qualia-host", "aether-native"}
    if receipt.get("with_chora_client") and client_consumer_found != expected_consumers:
        errors.append("Chora client profile requires the actual standalone host consumer" if live_host_test
                      else "Chora client profile requires both actual host and native consumers")
    if not test_root and receipt.get("blitz_source") and blitz_found != set(BLITZ_PACKAGES):
        errors.append("Required local Blitz packages missing: " + ", ".join(sorted(set(BLITZ_PACKAGES) - blitz_found)))
    if errors:
        raise ValueError("\n".join(errors))
    if snapshot:
        print(f"Captured canonical source gate passed: {len(snapshot_found)} resolved packages of {len(snapshot['packages'])} captured; receipt SHA256 {receipt['snapshot_receipt_sha256']}")
    if test_root:
        print(f"Actual {'live' if live_host_test else 'captured'} test workspace verified: {TEST_ROOTS[test_root]}")
    print(f"Resolved GPU source gate passed: {len(found)} packages at Etalon {receipt.get('etalon_revision', REVISION)}")
    print("This verifies source identity only; native execution/device sharing still require runtime tests.")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--etalon-source", type=Path)
    parser.add_argument("--live-sources", action="store_true", help="Explicitly bind current sibling source bytes, including local modifications, in a new qualification receipt")
    parser.add_argument("--hypermedia-source", type=Path, default=ROOT.parent / "hypermedia-production")
    parser.add_argument("--with-host", action="store_true")
    parser.add_argument("--snapshot-receipt", type=Path)
    parser.add_argument("--host-source", type=Path)
    parser.add_argument("--qualia-source", type=Path)
    engine_mode = parser.add_mutually_exclusive_group()
    engine_mode.add_argument("--with-chora", action="store_true", help="Experimental in-process diagnostic; current SpiderMonkey/Blitz ICU versions conflict. Chora's production process boundary is separate.")
    engine_mode.add_argument("--with-chora-client", action="store_true", help="Select the separate-process client and canonical parent pump; requires --with-host and never links the engine")
    engine_mode.add_argument("--engine-only", action="store_true", help="Generate only the separate generic Chora engine consumer with canonical Etalon")
    engine_mode.add_argument("--gpu-adapters-only", action="store_true", help="Qualify sibling context/renderer/profiler libraries without native UI or the application database core")
    parser.add_argument("--output", type=Path, default=ROOT / "target/local-integration")
    parser.add_argument("--verify", type=Path, help="Verify full Cargo metadata against this profile's receipt")
    parser.add_argument("--verify-test-root", choices=TEST_ROOTS, help="Verify actual captured package tests, or only the standalone host for a live host-enabled receipt")
    parser.add_argument("--write-test-patches", action="store_true", help="Extract this generated consumer's absolute patch tables without changing its graph")
    args = parser.parse_args()
    if args.live_sources and (args.snapshot_receipt or args.with_chora or args.engine_only):
        parser.error("--live-sources cannot combine with frozen snapshots, captured test roots or an in-process Chora engine")
    if args.live_sources and args.verify_test_root not in {None, "host"}:
        parser.error("--live-sources supports only --verify-test-root host")
    if args.with_chora_client and not args.with_host:
        parser.error("--with-chora-client requires --with-host")
    if (args.engine_only or args.gpu_adapters_only) and args.with_host:
        parser.error("The generic engine consumer and the QualiaDB host consumer require separate profiles")
    with_native = not (args.engine_only or args.gpu_adapters_only)
    with_chora = args.with_chora or args.engine_only
    output = args.output.resolve()
    if not output.is_relative_to(ROOT / "target"):
        parser.error("Generated consumer must be beneath Aether/target")
    receipt_path = output / "source-receipt.json"
    if args.write_test_patches:
        write_test_patches(output / "Cargo.toml", output / "test-patches.toml")
        print(output / "test-patches.toml")
        return
    if args.verify_test_root and not args.verify:
        parser.error("--verify-test-root requires --verify metadata")
    if args.verify:
        verify(args.verify, receipt_path, args.verify_test_root)
        return
    if receipt_path.exists():
        raise ValueError("Existing profile receipt is immutable; choose a new output directory")
    host_source = ROOT / "packages/qualia-host"
    qualia_source = ROOT.parent / "qualiaDB"
    snapshot_receipt = None
    snapshot = None
    if args.snapshot_receipt:
        if not args.with_host:
            parser.error("Canonical source snapshots require --with-host")
        snapshot_receipt = args.snapshot_receipt.resolve()
        snapshot = verify_source_snapshot(snapshot_receipt)
        if args.with_chora_client:
            parser.error("Frozen native Chora client composition is unsupported: live Aether UI selects live transport paths; use captured host standalone tests or the explicit live client profile")
        host_source = snapshot["host_source"]
        qualia_source = snapshot["qualia_source"]
    for requested, actual in ((args.host_source, host_source), (args.qualia_source, qualia_source)):
        if requested is not None and requested.resolve() != actual:
            parser.error("Source overrides must match the canonical snapshot receipt")
    source = (args.etalon_source or ROOT.parent / "etalon").resolve() if args.live_sources else pinned_source(args.etalon_source)
    if args.live_sources and source.is_relative_to(ROOT):
        raise ValueError("Live Etalon must be an independent sibling workspace")
    revision = git(source, "rev-parse", "HEAD") if args.live_sources else REVISION
    hypermedia = args.hypermedia_source.resolve()
    live_states = capture_sources(ROOT, source, qualia_source, hypermedia) if args.live_sources else None
    overlays = {} if args.live_sources else (snapshot.get("reviewed_overlays", {}) if snapshot else profile_overlays(qualia_source,source,output)) if args.with_host else {}
    canonical_patches = canonical_patch_sources(qualia_source, snapshot, overlays) if args.with_host else {}
    patches = etalon_packages(source)
    live_patches = dependency_patches(hypermedia, qualia_source, patches) if args.live_sources else {}
    if args.live_sources:
        for alias, spec in load(qualia_source / "Cargo.toml")["workspace"]["dependencies"].items():
            if isinstance(spec, dict) and spec.get("git") == ETALON_URL and spec.get("rev") != revision:
                raise ValueError(f"Canonical dependency does not declare selected live Etalon revision: {alias}")
    for name,entry in overlays.items():
        patches[name] = Path(entry["manifest_path"]).parent
    workspace_exclusion = 'exclude = ["reviewed-overlays/etalon-model-io"]\n' if overlays and not snapshot else ''
    text = '[package]\nname="aether-local-integration"\nversion="0.0.0"\nedition="2024"\npublish=false\n[workspace]\n' + workspace_exclusion + '[dependencies]\n'
    exports = ''
    if args.gpu_adapters_only:
        text += 'etalon-context = { package = "gpu-device-context", path = ' + quote(hypermedia / "crates/gpu-device-context") + ' }\n'
        text += 'renderer = { package = "vector-compute-renderer", path = ' + quote(hypermedia / "crates/vector-compute-renderer") + ', features = ["etalon-profiler"] }\n'
        exports += 'pub use etalon_context;\npub use renderer;\n'
    if with_native:
        native_features = ', features = ["prelude", "chora"]' if args.with_chora_client else ', features = ["prelude"]' if args.with_host else ''
        text += 'aether-native = { path = ' + quote(ROOT / "packages/native") + native_features + ' }\n'
        exports += 'pub use aether_native as native;\n'
    if args.with_host:
        host_features = ', features = ["chora-client"]' if args.with_chora_client else ''
        text += 'aether-qualia-host = { path = ' + quote(host_source) + host_features + ' }\n'
        text += 'aether = { path = ' + quote(ROOT / "packages/aether") + ', features = ["native"] }\n'
        text += 'serde_json = "1"\nfutures-timer = "3"\ntempfile = "3"\n'
        exports += 'pub use aether_qualia_host as host;\n'
    if with_chora:
        text += 'chora = { package = "servo", path = ' + quote(ROOT.parent / "chora/components/servo") + ', default-features = false, features = ["embedded_library"] }\n'
        exports += 'pub use chora;\n'
    if args.with_host:
        text += '\n[[example]]\nname = "native_graph_window"\npath = ' + quote(host_source / "integration/native_graph_window.rs") + '\n'
        text += '\n[[example]]\nname = "native_graph_fixture"\npath = ' + quote(host_source / "integration/native_graph_fixture.rs") + '\n'
    text += '\n[patch.' + quote(ETALON_URL) + ']\n'
    for name, path in sorted(patches.items()):
        text += quote(name) + ' = { path = ' + quote(path) + ' }\n'
    text += '\n[patch.' + quote(HYPERMEDIA_URL) + ']\n'
    manifest_hashes = {}
    for name in ADAPTERS:
        path = hypermedia / "crates" / name
        manifest = path / "Cargo.toml"
        if load(manifest)["package"]["name"] != name:
            raise ValueError(f"Unexpected adapter identity in {manifest}")
        text += quote(name) + ' = { path = ' + quote(path) + ' }\n'
        manifest_hashes[name] = hashlib.sha256(manifest.read_bytes()).hexdigest()
    text += '\n[patch.crates-io]\n'
    blitz_source = ROOT / "vendor/blitz"
    for name in BLITZ_PACKAGES if with_native else ():
        path = blitz_source / name
        if load(path / "Cargo.toml")["package"]["name"] != name:
            raise ValueError(f"Unexpected Blitz identity in {path}")
        text += quote(name) + ' = { path = ' + quote(path) + ' }\n'
    for name, entry in canonical_patches.items():
        if entry["registry"] == "crates-io":
            if name in BLITZ_PACKAGES:
                raise ValueError(f"Canonical patch conflicts with reviewed native source: {name}")
            text += quote(name) + ' = { path = ' + quote(Path(entry["manifest_path"]).parent) + ' }\n'
    stylo = None
    if with_chora:
        # Both engines link servo_style_crate. Cargo cannot combine registry
        # Stylo with Chora's Git copy, even when their semantic versions match.
        # Select the engine's exact source only in this experimental consumer.
        style_deps = {}
        for alias, spec in load(ROOT.parent / "chora/Cargo.toml")["workspace"]["dependencies"].items():
            if isinstance(spec, dict) and spec.get("git") == STYLO_URL:
                style_deps[spec.get("package", alias)] = spec["rev"]
        revisions = set(style_deps.values())
        if len(revisions) != 1 or "stylo" not in style_deps:
            raise ValueError("Chora must select one explicit Stylo revision")
        if with_native:
            for name, revision in sorted(style_deps.items()):
                text += quote(name) + ' = { git = ' + quote(STYLO_URL) + ', rev = ' + quote(revision) + ' }\n'
        stylo = {"revision": next(iter(revisions)), "packages": sorted(style_deps)}
    if with_native or args.gpu_adapters_only:
        text += '\n[patch."https://github.com/mediaprophet/vibe-script.git"]\nvibescript-abi = { path = ' + quote(qualia_source / "crates/vibescript-abi") + ' }\n'
    for name, entry in canonical_patches.items():
        if entry["registry"] == "https://github.com/mediaprophet/vibe-script.git":
            if name == "vibescript-abi":
                if Path(entry["manifest_path"]) != qualia_source / "crates/vibescript-abi/Cargo.toml":
                    raise ValueError("Canonical ABI patch differs from selected host ABI")
            else:
                text += quote(name) + ' = { path = ' + quote(Path(entry["manifest_path"]).parent) + ' }\n'
    if args.live_sources:
        # Dependency-local .cargo configuration is not inherited by consumers.
        # Etalon is already patched above; canonical ABI shares the host source.
        existing = tomllib.loads(text).get("patch", {})
        for registry, entries in live_patches.items():
            missing = {}
            for name, path in entries.items():
                current = existing.get(registry, {}).get(name)
                if current is not None:
                    if current != {"path": path.as_posix()}:
                        raise ValueError(f"Live dependency patch conflicts with consumer: {name}")
                else:
                    missing[name] = path
            if missing:
                if registry in existing:
                    raise ValueError(f"Live patch table must be composed explicitly: {registry}")
                text += '\n[patch.' + quote(registry) + ']\n'
                for name, path in sorted(missing.items()):
                    text += quote(name) + ' = { path = ' + quote(path) + ' }\n'
        # Bind the exact Aether authority transport selected by both consumers.
        if args.with_chora_client:
            for name, relative in AUTHORITY_TRANSPORTS.items():
                if load(ROOT / relative / "Cargo.toml")["package"]["name"] != name:
                    raise ValueError(f"Unexpected live authority transport identity: {name}")
    if snapshot_receipt:
        text += '\n[profile.dev]\ndebug = 0\n[profile.test]\ndebug = 0\n'
    output.mkdir(parents=True, exist_ok=True)
    (output / "src").mkdir(exist_ok=True)
    (output / "Cargo.toml").write_text(text, encoding="utf-8")
    (output / "src/lib.rs").write_text(exports, encoding="utf-8")
    receipt = {
        "qualification_only": True, "etalon_revision": revision,
        "live_sources": args.live_sources,
        "live_source_states": live_states,
        "consumer_manifest_sha256": hashlib.sha256((output / "Cargo.toml").read_bytes()).hexdigest() if args.live_sources else None,
        "consumer_library_sha256": hashlib.sha256((output / "src/lib.rs").read_bytes()).hexdigest() if args.live_sources else None,
        "etalon_source": source.as_posix(), "hypermedia_source": hypermedia.as_posix(),
        "hypermedia_head": git(hypermedia, "rev-parse", "HEAD"),
        "hypermedia_status": git(hypermedia, "status", "--porcelain"),
        "adapter_manifest_sha256": manifest_hashes,
        "canonical_patches": canonical_patches,
        "reviewed_etalon_overlays": overlays,
        "preparation_tool": {"path":str(Path(__file__).resolve()),"sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                             "dependencies":[{"path":str(Path(__file__).with_name(name).resolve()),"sha256":hashlib.sha256(Path(__file__).with_name(name).read_bytes()).hexdigest()}
                                             for name in ("source_snapshot.py","reviewed_etalon_overlays.py","reviewed_etalon_overlays.json","live_integration_sources.py")]},
        "with_host": args.with_host, "with_chora": with_chora, "with_chora_client": args.with_chora_client,
        "with_native": with_native, "engine_only": args.engine_only,
        "gpu_adapters_only": args.gpu_adapters_only,
        "qualia_source": qualia_source.as_posix(), "host_source": host_source.as_posix(),
        "snapshot_receipt": snapshot_receipt.as_posix() if snapshot_receipt else None,
        "snapshot_receipt_sha256": hashlib.sha256(snapshot_receipt.read_bytes()).hexdigest() if snapshot_receipt else None,
        "build_environment": {"CARGO_BUILD_JOBS": "1", "CARGO_PROFILE_DEV_DEBUG": "0", "CARGO_PROFILE_TEST_DEBUG": "0"} if snapshot_receipt else None,
        "chora_source": (ROOT.parent / "chora").as_posix(),
        "blitz_source": blitz_source.as_posix() if with_native else None,
        "stylo": stylo,
    }
    if args.live_sources:
        verify_sources(live_states)
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(output / "Cargo.toml")
    print("Explicit local integration consumer prepared; shipping manifests and lockfile were not changed.")


if __name__ == "__main__":
    main()
