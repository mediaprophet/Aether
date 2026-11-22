import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from source_snapshot import verify_source_snapshot


class CanonicalSnapshotTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.path = self.root / "receipt.json"
        self.receipt = {
            "schema": "aether.qualia-source-snapshot.v1",
            "host_source": "Aether/packages/qualia-host", "qualia_source": "qualiaDB",
            "files": [], "packages": [],
        }
        names = ("aether-qualia-host", "qualia-core-db", "qualia-cooperative-core", "vibe", "vibescript-abi")
        for name in names:
            folder = self.receipt["host_source"] if name == "aether-qualia-host" else "qualiaDB/crates/" + name
            relative = folder + "/Cargo.toml"
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(f'[package]\nname="{name}"\nversion="0.0.0"\n')
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            self.receipt["files"].append({"path": relative, "sha256": digest, "source_sha256": digest, "source": str(self.root / "original" / relative)})
            self.receipt["packages"].append({"name": name, "path": relative})

    def verify(self):
        self.path.write_text(json.dumps(self.receipt))
        return verify_source_snapshot(self.path)

    def test_accepts_captured_sources_without_requiring_originals_to_stay_frozen(self):
        result = self.verify()
        self.assertEqual(result["host_source"], self.root / "Aether/packages/qualia-host")

    def test_rejects_changed_source_bytes(self):
        (self.root / self.receipt["files"][1]["path"]).write_text("changed")
        with self.assertRaisesRegex(ValueError, "Snapshot input changed"):
            self.verify()

    def test_rejects_unrecorded_source_additions(self):
        (self.root / "unrecorded.rs").write_text("fn injected() {}")
        with self.assertRaisesRegex(ValueError, "Unrecorded snapshot inputs"):
            self.verify()

    def test_allows_generated_cargo_target_artifacts(self):
        target = self.root / "Aether/packages/qualia-host/target"
        target.mkdir()
        (target / "build-output").write_bytes(b"generated")
        self.verify()

    def test_rejects_escape_from_snapshot_root(self):
        self.receipt["files"][0]["path"] = "../outside.rs"
        with self.assertRaisesRegex(ValueError, "escapes"):
            self.verify()

    def test_rejects_roots_that_differ_from_captured_packages(self):
        self.receipt["qualia_source"] = "unrecorded-canonical"
        with self.assertRaisesRegex(ValueError, "canonical source does not match"):
            self.verify()

    def test_requires_package_manifests_to_be_hashed(self):
        self.receipt["packages"][0]["path"] = "unrecorded/Cargo.toml"
        with self.assertRaisesRegex(ValueError, "manifest was not captured"):
            self.verify()

    def test_requires_manifest_transformations_to_be_explicit(self):
        entry = self.receipt["files"][0]
        path = self.root / entry["path"]
        path.write_text(path.read_text() + "# local path rewrite\n")
        entry["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
        with self.assertRaisesRegex(ValueError, "transformation is not recorded"):
            self.verify()
        entry["transform"] = "explicit local dependency paths"
        self.verify()

    def test_rejects_ambiguous_provenance_source(self):
        self.receipt["files"][0]["source"] = "relative/source.rs"
        with self.assertRaisesRegex(ValueError, "absolute source"):
            self.verify()

    def test_linked_ancestry_is_checked_again_on_each_verification(self):
        self.verify()
        original = Path.is_symlink
        linked = self.root / "qualiaDB/crates"
        with patch.object(Path, "is_symlink", lambda path: path == linked or original(path)):
            with self.assertRaisesRegex(ValueError, "Linked snapshot input"):
                self.verify()

    def test_input_leaf_link_cannot_reuse_an_ancestor_cache_entry(self):
        original = Path.is_symlink
        linked = self.root / self.receipt["files"][1]["path"]
        with patch.object(Path, "is_symlink", lambda path: path == linked or original(path)):
            with self.assertRaisesRegex(ValueError, "Linked snapshot input"):
                self.verify()


if __name__ == "__main__":
    unittest.main()
