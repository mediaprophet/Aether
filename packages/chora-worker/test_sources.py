# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
"""Adversarial policy tests using fixtures; not build/source qualification evidence."""
import copy
import importlib.util
import unittest
from pathlib import Path
import tomllib

import check_sources as policy


class SourcePolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location("gpu", policy.CHORA / "support/etalon/check_gpu_sources.py")
        cls.gpu = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.gpu)
        cls.overlay = cls.gpu.overlay_sources(policy.CHORA)

    def setUp(self):
        self.manifest = tomllib.loads((policy.WORKER / "Cargo.toml").read_text())
        names = ["aether-chora-worker", "aether-chora-bridge-protocol", "servo", "etalon-core", "etalon-core-remote", "etalon-core-remote-types", "etalon-types", "etalon-hal", "naga", "naga-types"]
        packages = [{"id": name, "name": name, "version": "0.0.2", "source": None, "manifest_path": str(policy.WORKER / "Cargo.toml")} for name in names]
        packages[2]["manifest_path"] = str(policy.CHORA / "components/servo/Cargo.toml")
        packages[1]["manifest_path"] = str(policy.WORKER.parent / "chora-bridge-protocol/Cargo.toml")
        for package in packages[3:]:
            package["source"] = f"git+{self.gpu.ETALON_URL}?rev={self.gpu.ETALON_REV}#{self.gpu.ETALON_REV}"
        self.document = {"workspace_root": str(policy.WORKER), "workspace_members": [names[0]], "packages": packages, "resolve": {"root": names[0], "nodes": [{"id": name, "features": ["embedded_library"] if name == "servo" else []} for name in names]}}
        self.lock = {"package": copy.deepcopy(packages)}

    def errors(self):
        return policy.violations(self.document, self.manifest, self.lock, self.overlay, self.gpu)

    def test_fixture_is_admitted(self):
        self.assertEqual(self.errors(), [])

    def test_no_deps_metadata_rejected(self):
        self.document["resolve"] = None
        self.assertTrue(self.errors())

    def test_parent_workspace_rejected(self):
        self.document["workspace_root"] = str(policy.WORKER.parents[1])
        self.assertTrue(self.errors())

    def test_engine_alias_cannot_select_registry(self):
        self.document["packages"][2]["source"] = "registry+https://github.com/rust-lang/crates.io-index"
        self.assertTrue(self.errors())

    def test_wrong_fork_path_rejected(self):
        self.document["packages"][2]["manifest_path"] = str(Path("C:/other/components/servo/Cargo.toml"))
        self.assertTrue(self.errors())

    def test_protocol_substitution_rejected(self):
        self.document["packages"][1]["manifest_path"] = str(policy.WORKER.parent / "other-protocol/Cargo.toml")
        self.assertTrue(self.errors())

    def test_forbidden_features_rejected(self):
        for feature in ["multiprocess", "media-gstreamer", "devtools"]:
            with self.subTest(feature=feature):
                self.document["resolve"]["nodes"][2]["features"] = ["embedded_library", feature]
                self.assertTrue(self.errors())

    def test_mixed_etalon_revision_rejected(self):
        self.document["packages"][3]["source"] = self.document["packages"][3]["source"].replace(self.gpu.ETALON_REV, "0" * 40)
        self.assertTrue(self.errors())

    def test_privileged_dependency_rejected(self):
        self.document["packages"].append({"id": "qualia", "name": "qualia-core-db", "version": "0.1", "source": None})
        self.assertTrue(self.errors())

    def test_overlay_substitution_rejected(self):
        self.manifest["patch"][self.gpu.HYPERMEDIA_URL]["vector-compute-renderer"]["path"] = "../other"
        self.assertTrue(self.errors())

    def test_missing_lock_identity_rejected(self):
        self.lock["package"].pop()
        self.assertTrue(self.errors())


if __name__ == "__main__":
    unittest.main()
