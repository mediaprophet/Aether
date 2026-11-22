"""Keep parent/client qualification separate from engine and frozen UI identity."""
# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import tomllib
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import prepare_local_integration as profile
from source_snapshot import AUTHORITY_TRANSPORTS


class TransportProfileTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="aether-client-profile-")
        self.addCleanup(self.temp.cleanup)
        base = Path(self.temp.name).resolve()
        self.root, self.hypermedia, self.etalon = base / "Aether", base / "hypermedia", base / "etalon"
        self.output = self.root / "target/client-profile"
        self.hypermedia.mkdir()
        (base / "qualiaDB").mkdir()
        (base / "qualiaDB/Cargo.toml").write_text("[workspace]\n")
        for name in profile.ADAPTERS:
            folder = self.hypermedia / "crates" / name
            folder.mkdir(parents=True)
            (folder / "Cargo.toml").write_text(f'[package]\nname="{name}"\nversion="0.1.0"\n')
        for name in profile.BLITZ_PACKAGES:
            folder = self.root / "vendor/blitz" / name
            folder.mkdir(parents=True)
            (folder / "Cargo.toml").write_text(f'[package]\nname="{name}"\nversion="0.1.0"\n')

    def generate(self, *flags, snapshot=None):
        argv = ["prepare_local_integration.py", "--output", str(self.output), "--hypermedia-source", str(self.hypermedia), *flags]
        with patch.object(profile, "ROOT", self.root), patch.object(profile, "pinned_source", return_value=self.etalon), \
             patch.object(profile, "etalon_packages", return_value={"etalon": self.etalon / "etalon"}), \
             patch.object(profile, "git", return_value="test-working-tree"), patch.object(sys, "argv", argv), \
             patch.object(profile, "verify_source_snapshot", return_value=snapshot), \
             contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            profile.main()
        return tomllib.loads((self.output / "Cargo.toml").read_text())

    def test_explicit_client_profile_enables_parent_pump_and_native_frame_without_engine(self):
        manifest = self.generate("--with-host", "--with-chora-client")
        self.assertEqual(manifest["dependencies"]["aether-qualia-host"]["features"], ["chora-client"])
        self.assertEqual(manifest["dependencies"]["aether-native"]["features"], ["prelude", "chora"])
        self.assertNotIn("chora", manifest["dependencies"])
        receipt = json.loads((self.output / "source-receipt.json").read_text())
        self.assertTrue(receipt["with_chora_client"])
        self.assertFalse(receipt["with_chora"])

    def test_minimal_host_profile_does_not_activate_client(self):
        manifest = self.generate("--with-host")
        self.assertNotIn("features", manifest["dependencies"]["aether-qualia-host"])
        self.assertEqual(manifest["dependencies"]["aether-native"]["features"], ["prelude"])

    def test_client_requires_host_and_is_distinct_from_in_process_diagnostic(self):
        for flags in [("--with-chora-client",), ("--with-host", "--with-chora-client", "--with-chora")]:
            with self.assertRaises(SystemExit):
                self.generate(*flags)
        self.assertFalse(self.output.exists())

    def test_frozen_native_client_mixed_path_identity_is_rejected_before_writing(self):
        receipt = self.root / "frozen.json"
        snapshot = {"host_source": self.root / "captured/Aether/packages/qualia-host", "qualia_source": self.root / "captured/qualiaDB"}
        with self.assertRaises(SystemExit):
            self.generate("--with-host", "--with-chora-client", "--snapshot-receipt", str(receipt), snapshot=snapshot)
        self.assertFalse(self.output.exists())

    def metadata(self):
        manifest = self.generate("--with-host", "--with-chora-client")
        paths = {"etalon": self.etalon / "etalon/Cargo.toml", "aether-native": self.root / "packages/native/Cargo.toml",
                 "aether-qualia-host": self.root / "packages/qualia-host/Cargo.toml"}
        paths.update({name: self.root / relative / "Cargo.toml" for name, relative in AUTHORITY_TRANSPORTS.items()})
        paths.update({name: self.root.parent / "qualiaDB/crates" / name / "Cargo.toml"
                      for name in ("vibe", "vibescript-abi", "qualia-core-db", "qualia-cooperative-core")})
        paths.update({name: self.root / "vendor/blitz" / name / "Cargo.toml" for name in profile.BLITZ_PACKAGES})
        packages = [{"id": name, "name": name, "manifest_path": str(path), "source": None,
                     "version": "0.0.2" if name == "etalon" else "0.1.0"} for name, path in paths.items()]
        features = {"aether-native": ["chora"], "aether-qualia-host": ["chora-client"]}
        return {"packages": packages, "resolve": {"nodes": [{"id": p["id"], "features": features.get(p["id"], [])} for p in packages]}}

    def verify(self, metadata):
        path = self.output / "metadata.json"
        path.write_text(json.dumps(metadata))
        with patch.object(profile, "ROOT", self.root), patch.object(profile, "pinned_source", return_value=self.etalon), \
             contextlib.redirect_stdout(io.StringIO()):
            profile.verify(path, self.output / "source-receipt.json")

    def test_source_gate_accepts_real_client_profile_and_rejects_live_alias_replacement(self):
        metadata = self.metadata()
        self.verify(metadata)
        client = next(p for p in metadata["packages"] if p["name"] == "aether-chora-client")
        client["manifest_path"] = str(self.root.parent / "unreviewed/client/Cargo.toml")
        with self.assertRaisesRegex(ValueError, "Authority transport is outside"):
            self.verify(metadata)

    def test_profile_rejects_engine_link_and_unselected_pump_feature(self):
        metadata = self.metadata()
        metadata["packages"].append({"name": "servo", "manifest_path": str(self.root.parent / "chora/components/servo/Cargo.toml"), "source": None})
        with self.assertRaisesRegex(ValueError, "must not link the Chora engine"):
            self.verify(metadata)
        metadata["packages"].pop()
        next(n for n in metadata["resolve"]["nodes"] if n["id"] == "aether-qualia-host")["features"] = []
        with self.assertRaisesRegex(ValueError, "profile feature missing"):
            self.verify(metadata)

    def test_shipping_pump_profile_excludes_worker_backed_test_fixture(self):
        metadata = self.metadata()
        next(n for n in metadata["resolve"]["nodes"] if n["id"] == "aether-chora-client")["features"] = ["test-fixture"]
        with self.assertRaisesRegex(ValueError, "excludes worker-backed"):
            self.verify(metadata)


if __name__ == "__main__":
    unittest.main()
