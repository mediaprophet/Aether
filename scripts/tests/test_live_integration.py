"""Synthetic live-source policy tests; never resolve or build Cargo packages."""
# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
from pathlib import Path
import json
import hashlib
import contextlib
import io
import copy
import subprocess
import stat
import sys
import tempfile
import tomllib
import unittest
from unittest.mock import patch
from types import SimpleNamespace

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import live_integration_sources as live
import prepare_local_integration as prepare


class LiveSourcesTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def package(self, path, name):
        path.mkdir(parents=True, exist_ok=True)
        (path / "Cargo.toml").write_text(f'[package]\nname="{name}"\nversion="0.0.2"\n')
        (path / "lib.rs").write_text("pub fn original() {}\n")
        return path

    def list_files(self, command, **kwargs):
        root = Path(command[command.index("-C") + 1])
        return "\0".join(p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()).encode()

    def states(self):
        roots = {name: self.package(self.root / name, name)
                 for name in ("aether", "etalon", "qualia", "hypermedia")}
        return live.capture_sources(**roots)

    def test_receipt_detects_dirty_bytes_without_head_change(self):
        with patch.object(live, "git", return_value="unchanged"), patch.object(live.subprocess, "check_output", side_effect=self.list_files):
            states = self.states()
            live.verify_sources(states)
            (self.root / "qualia/lib.rs").write_text("pub fn altered() {}\n")
            with self.assertRaisesRegex(ValueError, "qualia"):
                live.verify_sources(states)

    def test_receipt_detects_new_source_and_deleted_source(self):
        with patch.object(live, "git", return_value="unchanged"), patch.object(live.subprocess, "check_output", side_effect=self.list_files):
            states = self.states()
            added = self.root / "hypermedia/new.wgsl"
            added.write_text("shader")
            with self.assertRaisesRegex(ValueError, "hypermedia"):
                live.verify_sources(states)
            added.unlink()
            (self.root / "etalon/lib.rs").unlink()
            with self.assertRaisesRegex(ValueError, "etalon"):
                live.verify_sources(states)

    def test_inventory_rejects_parent_traversal(self):
        names = ["../outside.txt", "/outside.txt"]
        if sys.platform == "win32":
            names.extend(["\\outside\\lib.rs", "C:outside.txt"])
        for name in names:
            with self.subTest(name=name), patch.object(live.subprocess, "check_output", return_value=name.encode() + b"\0"):
                with self.assertRaisesRegex(ValueError, "escapes"):
                    live.inventory(self.root, ["."])

    def test_inventory_rejects_reparse_parent_before_reading(self):
        parent = self.root / "junction"
        parent.mkdir()
        original = Path.lstat

        def lstat(path):
            if path == parent:
                return SimpleNamespace(st_mode=stat.S_IFDIR, st_file_attributes=0x400)
            return original(path)

        with patch.object(live.subprocess, "check_output", return_value=b"junction/lib.rs\0"), \
             patch.object(Path, "lstat", lstat), patch.object(live, "file_digest") as digest:
            with self.assertRaisesRegex(ValueError, "parent is a link"):
                live.inventory(self.root, ["."])
            digest.assert_not_called()

    def test_inventory_rejects_real_symlink_parent_when_supported(self):
        outside = self.root / "outside"
        outside.mkdir()
        (outside / "lib.rs").write_text("external input")
        source = self.root / "source"
        source.mkdir()
        try:
            (source / "linked").symlink_to(outside, target_is_directory=True)
        except (OSError, NotImplementedError) as error:
            self.skipTest(f"OS does not permit creating symlinks: {error}")
        with patch.object(live.subprocess, "check_output", return_value=b"linked/lib.rs\0"):
            with self.assertRaisesRegex(ValueError, "parent is a link"):
                live.inventory(source, ["."])

    def test_dependency_config_patches_are_explicit_and_canonical(self):
        hyper = self.root / "hypermedia"
        (hyper / ".cargo").mkdir(parents=True)
        qualia = self.root / "qualia"
        abi = self.package(qualia / "crates/vibescript-abi", "vibescript-abi")
        gpu = self.package(self.root / "etalon/etalon", "etalon")
        config = hyper / ".cargo/config.toml"
        config.write_text(f'[patch."{live.ETALON_URL}"]\netalon={{path="../etalon/etalon"}}\n'
                          f'[patch."{live.VIBE_URL}"]\nvibescript-abi={{path="../qualia/crates/vibescript-abi"}}\n')
        patches = live.dependency_patches(hyper, qualia, {"etalon": gpu})
        self.assertEqual(patches[live.ETALON_URL]["etalon"], gpu)
        self.assertEqual(patches[live.VIBE_URL]["vibescript-abi"], abi)
        config.write_text(f'[patch."{live.ETALON_URL}"]\netalon={{path="../qualia/crates/vibescript-abi"}}\n')
        with self.assertRaisesRegex(ValueError, "canonical package"):
            live.dependency_patches(hyper, qualia, {"etalon": gpu})

    def test_metadata_rejects_external_gpu_and_duplicate_client(self):
        gpu = self.package(self.root / "etalon/etalon", "etalon")
        client = self.package(self.root / "aether/packages/chora-client", "aether-chora-client")
        states = {name: {"root": str(self.root / name), "files": {}}
                  for name in ("aether", "etalon", "qualia", "hypermedia")}
        states["etalon"]["files"]["etalon/Cargo.toml"] = "hash"
        states["aether"]["files"]["packages/chora-client/Cargo.toml"] = "hash"
        packages = [{"name": "etalon", "manifest_path": str(gpu / "Cargo.toml"), "source": None},
                    {"name": "aether-chora-client", "manifest_path": str(client / "Cargo.toml"), "source": None}]
        live.verify_metadata({"packages": packages}, states, {"etalon": gpu})
        packages.append({"name": "aether-chora-client", "manifest_path": str(self.root / "capture/Cargo.toml"), "source": None})
        with self.assertRaisesRegex(ValueError, "outside live source inventories"):
            live.verify_metadata({"packages": packages}, states, {"etalon": gpu})
        packages.pop()
        packages[0]["source"] = "git+https://github.com/mediaprophet/etalon.git?rev=old"
        with self.assertRaisesRegex(ValueError, "canonical manifest"):
            live.verify_metadata({"packages": packages}, states, {"etalon": gpu})

    def test_live_flags_refuse_frozen_and_engine_profiles_before_writes(self):
        for args in (["--snapshot-receipt", "missing.json"], ["--engine-only"], ["--with-chora"]):
            result = subprocess.run([sys.executable, str(SCRIPTS / "prepare_local_integration.py"), "--live-sources", *args], capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn("cannot combine", result.stderr)

    def test_default_revision_is_preserved(self):
        self.assertEqual(prepare.REVISION, "6ace6cc6413a87ca10a1da58397ddc7c0b2fc41e")

    def test_generator_builds_a_distinct_live_client_profile(self):
        aether = self.root / "aether"
        etalon = self.root / "etalon"
        qualia = self.root / "qualiaDB"
        hyper = self.root / "hypermedia"
        gpu = self.package(etalon / "etalon", "etalon")
        (etalon / "Cargo.toml").write_text('[workspace.dependencies]\netalon={path="etalon"}\n')
        qualia.mkdir()
        revision = "a" * 40
        (qualia / "Cargo.toml").write_text(f'[workspace.dependencies]\netalon={{git="{prepare.ETALON_URL}",rev="{revision}"}}\n')
        for name in prepare.ADAPTERS:
            self.package(hyper / "crates" / name, name)
        for name in prepare.BLITZ_PACKAGES:
            self.package(aether / "vendor/blitz" / name, name)
        for name, relative in prepare.AUTHORITY_TRANSPORTS.items():
            self.package(aether / relative, name)
        output = aether / "target/new-live-profile"
        args = ["prepare", "--live-sources", "--with-host", "--with-chora-client",
                "--etalon-source", str(etalon), "--hypermedia-source", str(hyper), "--output", str(output)]
        with patch.object(prepare, "ROOT", aether), patch.object(prepare, "git", return_value=revision), \
             patch.object(prepare, "capture_sources", return_value={"fixture": True}), \
             patch.object(prepare, "verify_sources"), patch.object(prepare, "profile_overlays", side_effect=AssertionError("old overlay policy invoked")), \
             patch.object(sys, "argv", args):
            prepare.main()
        manifest = tomllib.loads((output / "Cargo.toml").read_text())
        receipt = json.loads((output / "source-receipt.json").read_text())
        self.assertEqual(Path(manifest["patch"][prepare.ETALON_URL]["etalon"]["path"]), gpu.resolve())
        self.assertEqual(manifest["dependencies"]["aether-qualia-host"]["features"], ["chora-client"])
        self.assertIn("chora", manifest["dependencies"]["aether-native"]["features"])
        self.assertEqual(receipt["etalon_revision"], revision)
        self.assertTrue(receipt["live_sources"])
        self.assertEqual(receipt["reviewed_etalon_overlays"], {})
        self.assertEqual(receipt["consumer_manifest_sha256"], hashlib.sha256((output / "Cargo.toml").read_bytes()).hexdigest())
        self.assertEqual(receipt["consumer_library_sha256"], hashlib.sha256((output / "src/lib.rs").read_bytes()).hexdigest())


class LiveHostTestGateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.aether = self.root / "aether"
        self.qualia = self.root / "qualia"
        self.etalon = self.root / "etalon"
        self.output = self.aether / "target/profile"
        (self.output / "src").mkdir(parents=True)
        (self.output / "Cargo.toml").write_text("[workspace]\n")
        (self.output / "src/lib.rs").write_text("pub fn fixture() {}\n")
        self.packages = []
        self.states = {name: {"root": str(path), "files": {}}
                       for name, path in {"aether": self.aether, "qualia": self.qualia,
                                          "etalon": self.etalon, "hypermedia": self.root / "hypermedia"}.items()}
        self.add_package("aether-qualia-host", self.aether / "packages/qualia-host", "aether")
        self.add_package("etalon", self.etalon / "etalon", "etalon")
        for name in ("vibe", "vibescript-abi", "qualia-core-db", "qualia-cooperative-core"):
            self.add_package(name, self.qualia / "crates" / name, "qualia")
        for name, relative in prepare.AUTHORITY_TRANSPORTS.items():
            self.add_package(name, self.aether / relative, "aether")
        (self.etalon / "Cargo.toml").write_text('[workspace.dependencies]\netalon={path="etalon"}\n')
        self.receipt = {
            "live_sources": True, "with_host": True, "with_chora_client": True,
            "etalon_source": str(self.etalon), "etalon_revision": "b" * 40,
            "qualia_source": str(self.qualia), "host_source": str(self.aether / "packages/qualia-host"),
            "hypermedia_source": str(self.root / "hypermedia"), "adapter_manifest_sha256": {},
            "live_source_states": self.states,
            "consumer_manifest_sha256": hashlib.sha256((self.output / "Cargo.toml").read_bytes()).hexdigest(),
            "consumer_library_sha256": hashlib.sha256((self.output / "src/lib.rs").read_bytes()).hexdigest(),
        }
        self.metadata = {
            "packages": self.packages, "workspace_members": ["aether-qualia-host"],
            "workspace_root": str(self.aether / "packages/qualia-host"),
            "resolve": {"root": "aether-qualia-host", "nodes": [
                {"id": package["id"], "features": ["chora-client"] if package["name"] == "aether-qualia-host" else []}
                for package in self.packages]},
        }

    def add_package(self, name, path, root_name):
        path.mkdir(parents=True, exist_ok=True)
        manifest = path / "Cargo.toml"
        manifest.write_text(f'[package]\nname="{name}"\nversion="0.0.2"\n')
        state = self.states[root_name]
        state["files"][manifest.relative_to(state["root"]).as_posix()] = hashlib.sha256(manifest.read_bytes()).hexdigest()
        package = {"id": name, "name": name, "version": "0.0.2", "source": None, "manifest_path": str(manifest)}
        self.packages.append(package)
        return package

    def check(self, metadata=None, test_root="host"):
        metadata_path = self.output / "metadata.json"
        receipt_path = self.output / "source-receipt.json"
        metadata_path.write_text(json.dumps(metadata or self.metadata))
        receipt_path.write_text(json.dumps(self.receipt))
        with patch.object(prepare, "ROOT", self.aether), patch.object(prepare, "git", return_value="b" * 40), \
             patch.object(prepare, "verify_sources") as source_gate, contextlib.redirect_stdout(io.StringIO()):
            prepare.verify(metadata_path, receipt_path, test_root)
            source_gate.assert_called_once_with(self.states)

    def test_accepts_only_actual_live_standalone_host_without_native(self):
        self.check()

    def test_rejects_wrong_manifest_workspace_or_resolver_root(self):
        for mutation in ("manifest", "workspace_members", "workspace_root", "resolver_root"):
            metadata = copy.deepcopy(self.metadata)
            if mutation == "manifest":
                metadata["packages"][0]["manifest_path"] = str(self.aether / "foreign/Cargo.toml")
            elif mutation == "resolver_root":
                metadata["resolve"]["root"] = "etalon"
            else:
                metadata[mutation] = ["etalon", "aether-qualia-host"] if mutation == "workspace_members" else str(self.aether)
            with self.subTest(mutation=mutation), self.assertRaisesRegex(ValueError, "exact standalone"):
                self.check(metadata)

    def test_requires_host_client_feature(self):
        self.metadata["resolve"]["nodes"][0]["features"] = []
        with self.assertRaisesRegex(ValueError, "feature missing"):
            self.check()

    def test_rejects_duplicate_or_foreign_transport(self):
        client = next(package for package in self.packages if package["name"] == "aether-chora-client")
        duplicate = dict(client, id="duplicate-client")
        self.packages.append(duplicate)
        with self.assertRaisesRegex(ValueError, "duplicate authority"):
            self.check()
        self.packages.pop()
        client["source"] = "git+https://example.invalid/foreign"
        with self.assertRaisesRegex(ValueError, "Authority transport"):
            self.check()

    def test_requires_each_sdk_and_etalon(self):
        for name in ("vibe", "vibescript-abi", "qualia-core-db", "qualia-cooperative-core", "etalon"):
            metadata = copy.deepcopy(self.metadata)
            metadata["packages"] = [package for package in metadata["packages"] if package["name"] != name]
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "No Etalon|host packages missing"):
                self.check(metadata)

    def test_rejects_engine_native_and_transport_fixtures(self):
        for name in ("servo", "aether-native", "aether-native-dom"):
            self.add_package(name, self.aether / "packages" / name, "aether")
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "engine/native application"):
                self.check()
            self.packages.pop()
        for name in prepare.AUTHORITY_TRANSPORTS:
            metadata = copy.deepcopy(self.metadata)
            next(node for node in metadata["resolve"]["nodes"] if node["id"] == name)["features"] = ["test-fixture"]
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "test-fixture"):
                self.check(metadata)

    def test_rejects_other_live_roots_or_receipt_without_host(self):
        with self.assertRaisesRegex(ValueError, "only host"):
            self.check(test_root="core")
        self.receipt["with_host"] = False
        with self.assertRaisesRegex(ValueError, "host-enabled"):
            self.check()

    def test_live_host_verification_retains_source_and_artifact_drift_gates(self):
        (self.output / "src/lib.rs").write_text("modified artifact")
        with self.assertRaisesRegex(ValueError, "consumer library changed"):
            self.check()


if __name__ == "__main__":
    unittest.main()
