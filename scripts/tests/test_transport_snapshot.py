"""Authority transport capture and selected source/profile closure regressions."""
# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from snapshot_qualia_sources import aether_capture_packages
import snapshot_qualia_sources as capture
from source_snapshot import AUTHORITY_TRANSPORTS, authority_transport_closure, validate_transport_profile, verify_source_snapshot


class TransportSnapshotTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="aether-transport-capture-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.aether = self.root / "Aether"
        self.host = self.aether / "packages/qualia-host/Cargo.toml"
        self.package(self.host, "aether-qualia-host", '[dependencies]\naether-core={path="C:/explicit/live-ui"}\naether-chora-bridge-protocol={path="../chora-bridge-protocol"}\naether-chora-client={path="../chora-client",optional=true}\n')
        self.package(self.aether / AUTHORITY_TRANSPORTS["aether-chora-client"] / "Cargo.toml", "aether-chora-client",
                     '[dependencies]\nprotocol={package="aether-chora-bridge-protocol",path="../chora-bridge-protocol"}\n')
        self.package(self.aether / AUTHORITY_TRANSPORTS["aether-chora-bridge-protocol"] / "Cargo.toml", "aether-chora-bridge-protocol")
        self.canonical = ("qualia-core-db", "qualia-cooperative-core", "vibe", "vibescript-abi")
        for name in self.canonical:
            self.package(self.root / "qualiaDB/crates" / name / "Cargo.toml", name)

    def package(self, path, name, dependencies=""):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f'[package]\nname="{name}"\nversion="0.1.0"\n' + dependencies)
        source = path.parent / "src/lib.rs"
        source.parent.mkdir(exist_ok=True)
        source.write_text("// actual owned source bytes\npub fn marker() {}\n")

    def receipt(self):
        packages = {name: "Aether/" + manifest.relative_to(self.aether).as_posix()
                    for name, manifest in aether_capture_packages(self.aether).items()}
        packages.update({name: f"qualiaDB/crates/{name}/Cargo.toml" for name in self.canonical})
        files = []
        for path in self.root.rglob("*"):
            if path.is_file():
                digest = hashlib.sha256(path.read_bytes()).hexdigest()
                relative = path.relative_to(self.root).as_posix()
                files.append({"path": relative, "sha256": digest, "source_sha256": digest,
                              "source": str(self.root / "original-source" / relative)})
        return {"schema": "aether.qualia-source-snapshot.v1", "host_source": "Aether/packages/qualia-host",
                "qualia_source": "qualiaDB", "files": files,
                "packages": [{"name": name, "path": path} for name, path in packages.items()]}

    def verify(self, receipt):
        path = self.root / "source-receipt.json"
        path.write_text(json.dumps(receipt))
        return verify_source_snapshot(path)

    def test_optional_and_aliased_transports_are_captured_with_complete_source_inventory(self):
        packages = aether_capture_packages(self.aether)
        self.assertEqual(set(packages), {"aether-qualia-host", *AUTHORITY_TRANSPORTS})
        receipt = self.receipt()
        for relative in AUTHORITY_TRANSPORTS.values():
            self.assertIn("Aether/" + relative + "/src/lib.rs", {f["path"] for f in receipt["files"]})
        result = self.verify(receipt)
        self.assertEqual(result["packages"]["aether-chora-client"], packages["aether-chora-client"])

    def test_optional_missing_manifest_is_not_omitted_from_capture(self):
        (self.aether / "packages/chora-client/Cargo.toml").unlink()
        with self.assertRaisesRegex(ValueError, "leaves selected source closure"):
            aether_capture_packages(self.aether)

    def test_transport_alias_cannot_select_registry_or_live_source(self):
        for spec in ['{package="aether-chora-client",version="0.1"}',
                     '{package="aether-chora-client",path="C:/live/chora-client"}']:
            self.host.write_text('[package]\nname="aether-qualia-host"\nversion="0.1.0"\n[dependencies]\ntransport=' + spec)
            with self.assertRaisesRegex(ValueError, "explicit captured path|selected source closure"):
                aether_capture_packages(self.aether)

    def test_manifest_hashes_do_not_replace_missing_package_identity(self):
        receipt = self.receipt()
        receipt["packages"] = [p for p in receipt["packages"] if p["name"] != "aether-chora-client"]
        with self.assertRaisesRegex(ValueError, "not captured as a package"):
            self.verify(receipt)

    def test_client_fixture_profile_cannot_claim_uncaptured_worker_helper_inputs(self):
        metadata = {"packages": [{"name": "aether-chora-client", "id": "client"}],
                    "resolve": {"nodes": [{"id": "client", "features": []}]}}
        validate_transport_profile(metadata)
        metadata["resolve"]["nodes"][0]["features"] = ["test-fixture"]
        with self.assertRaisesRegex(ValueError, "uncaptured worker helper"):
            validate_transport_profile(metadata)

    def test_new_local_transport_dependency_requires_explicit_capture_review(self):
        client = self.aether / "packages/chora-client/Cargo.toml"
        client.write_text(client.read_text() + 'ambient={path="../uncaptured-helper"}\n')
        with self.assertRaisesRegex(ValueError, "Unreviewed authority transport path dependency"):
            aether_capture_packages(self.aether)

    def test_capture_requires_resolved_profile_before_copying(self):
        metadata = self.root / "metadata.json"
        metadata.write_text("{}")
        output = self.aether / "target/new-capture"
        argv = ["snapshot_qualia_sources.py", "--aether", str(self.aether), "--qualia", str(self.root / "qualiaDB"),
                "--metadata", str(metadata), "--output", str(output)]
        with patch.object(sys,"argv",argv), self.assertRaisesRegex(ValueError,"Full resolved Cargo metadata"):
            capture.main()
        self.assertFalse(output.exists())

    def test_actual_capture_copies_transport_libraries_and_keeps_ui_external(self):
        self.host.write_text(self.host.read_text().replace('C:/explicit/live-ui', '../core'))
        self.package(self.aether / "packages/core/Cargo.toml", "aether-core")
        (self.root / "qualiaDB/Cargo.toml").write_text('[workspace]\nmembers = [\n' +
            ',\n'.join(json.dumps("crates/" + name) for name in self.canonical) + '\n]\n')
        metadata = self.root / "metadata.json"
        metadata.write_text(json.dumps({"resolve": {"nodes": []}, "packages": [
            {"name": name, "manifest_path": str(self.root / "qualiaDB/crates" / name / "Cargo.toml")} for name in self.canonical]}))
        output = self.aether / "target/new-capture"
        def git(root, *args):
            if args[0] == "rev-parse": return b"fixture-repository-head\n"
            if args[0] == "diff": return b""
            prefixes = args[args.index("--")+1:]
            selected = [p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()
                        and any(p.relative_to(root).as_posix() == prefix or p.relative_to(root).as_posix().startswith(prefix + "/") for prefix in prefixes)]
            return b"\0".join(p.encode() for p in selected)
        argv = ["snapshot_qualia_sources.py", "--aether", str(self.aether), "--qualia", str(self.root / "qualiaDB"),
                "--metadata", str(metadata), "--output", str(output)]
        with patch.object(sys,"argv",argv), patch.object(capture,"git",git):
            capture.main()
        result = verify_source_snapshot(output / "source-receipt.json")
        self.assertTrue(set(AUTHORITY_TRANSPORTS).issubset(result["packages"]))
        receipt = json.loads((output / "source-receipt.json").read_text())
        inventory = {f["path"] for f in receipt["files"]}
        for relative in AUTHORITY_TRANSPORTS.values():
            self.assertIn("Aether/" + relative + "/src/lib.rs", inventory)
        self.assertFalse(any("packages/core/" in path for path in inventory))
        self.assertEqual(receipt["external_source_bindings"][0]["name"], "aether-core")
        self.assertFalse(receipt["authority_transport_profile"]["worker_engine_captured"])


class CargoTransportCompositionTests(unittest.TestCase):
    @unittest.skipUnless(shutil.which("cargo"), "Cargo is required for actual path-identity resolution")
    def test_inactive_native_transport_does_not_change_active_frozen_host_sources(self):
        with tempfile.TemporaryDirectory(prefix="aether-transport-resolution-") as temporary:
            root = Path(temporary)
            def crate(folder, name, extra=""):
                path = root / folder
                (path / "src").mkdir(parents=True)
                (path / "src/lib.rs").write_text("pub fn marker() {}")
                (path / "Cargo.toml").write_text(f'[package]\nname="{name}"\nversion="0.1.0"\n[workspace]\n' + extra)
                return path
            for tree in ("live", "captured"):
                crate(tree+"/protocol", "transport-protocol")
                crate(tree+"/client", "transport-client", '[dependencies]\ntransport-protocol={path="../protocol"}\n')
            crate("live/native", "native", '[features]\nchora=["dep:transport-client"]\n[dependencies]\ntransport-client={path="../client",optional=true}\n')
            crate("captured/host", "host", '[features]\nchora-client=["dep:transport-client"]\n[dependencies]\ntransport-protocol={path="../protocol"}\ntransport-client={path="../client",optional=true}\n')
            consumer = crate("consumer", "consumer", '[dependencies]\nnative={path="../live/native"}\nhost={path="../captured/host"}\n')
            environment = os.environ.copy()
            environment["CARGO_HOME"] = str(root / "cargo-home")
            environment["CARGO_TARGET_DIR"] = str(root / "target")
            run = subprocess.run(["cargo", "metadata", "--offline", "--format-version=1", "--manifest-path", str(consumer / "Cargo.toml")],
                                 capture_output=True, text=True, timeout=60, env=environment)
            self.assertEqual(run.returncode, 0, run.stderr)
            metadata = json.loads(run.stdout)
            active = {node["id"] for node in metadata["resolve"]["nodes"]}
            protocols = [p for p in metadata["packages"] if p["name"] == "transport-protocol"]
            self.assertEqual(len(protocols), 1)
            self.assertIn(protocols[0]["id"], active)
            self.assertFalse(any(p["name"] == "transport-client" for p in metadata["packages"]))
            self.assertEqual(Path(protocols[0]["manifest_path"]), root / "captured/protocol/Cargo.toml")


if __name__ == "__main__":
    unittest.main()
