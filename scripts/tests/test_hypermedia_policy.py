"""Regressions for immutable adapter sources and attributed local overrides."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_gpu_policy import validate
from test_check_gpu_policy import POLICY, REV, URL, lock_package, source_for

HYPERMEDIA = "https://github.com/mediaprophet/hypermedia-production.git"
REVIEWED = "a46b4e2ba5a9d57fc1a68e20ae87ce4bc2d076cf"
NEWER = "af57e82120b30b47fca238edb21a839d2cdb4278"
VENDOR = "vendor/hypermedia/vector-compute-shaders"


class HypermediaPolicyTests(unittest.TestCase):
    def fixture(self, manifest="[workspace]\n", locked=""):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        (root / "Cargo.toml").write_text(manifest)
        (root / "Cargo.lock").write_text(lock_package("etalon", source=source_for(URL, REV)) + locked)
        rule = {
            "git_url": HYPERMEDIA, "revision": REVIEWED,
            "packages": ["vector-gpu-render-adapter", "vector-compute-shaders"],
            "local_overrides": {"vector-compute-shaders": {
                "version": "0.11.0", "manifest": VENDOR + "/Cargo.toml",
            }},
        }
        policy = root / "policy.json"
        policy.write_text(json.dumps(dict(POLICY, policy_version=2, hypermedia=rule)))
        return root, policy

    def write_vendor(self, root, folder=VENDOR, name="vector-compute-shaders", version="0.11.0"):
        path = root / folder
        path.mkdir(parents=True)
        (path / "Cargo.toml").write_text(f'[package]\nname="{name}"\nversion="{version}"\n')

    def patch(self, folder=VENDOR):
        return f'[workspace]\n[patch."{HYPERMEDIA}"]\nshader-fix={{package="vector-compute-shaders",path="{folder}"}}\n'

    def test_accepts_actual_reviewed_commit_independent_of_branch_label(self):
        root, policy = self.fixture(locked=lock_package("vector-gpu-render-adapter", "0.15.0", f"git+{HYPERMEDIA}?branch=any-label#{REVIEWED}"))
        self.assertEqual([], validate(root, policy))

    def test_rejects_adapter_drift_even_when_etalon_did_not_change(self):
        root, policy = self.fixture(locked=lock_package("vector-gpu-render-adapter", "0.15.0", f"git+{HYPERMEDIA}?branch=stable#{NEWER}"))
        errors = validate(root, policy)
        self.assertTrue(any("Hypermedia source does not match reviewed commit" in error for error in errors), errors)

    def test_accepts_registered_vendor_manifest_identity_under_a_patch_alias(self):
        root, policy = self.fixture(self.patch(), lock_package("vector-compute-shaders", "0.11.0"))
        self.write_vendor(root)
        self.assertEqual([], validate(root, policy))

    def test_rejects_unapproved_local_path_with_the_same_package_identity(self):
        root, policy = self.fixture(self.patch("scratch/shaders"), lock_package("vector-compute-shaders", "0.11.0"))
        self.write_vendor(root, "scratch/shaders")
        self.assertTrue(any("unapproved local manifest path" in error for error in validate(root, policy)))

    def test_rejects_package_identity_changes_at_the_registered_path(self):
        root, policy = self.fixture(self.patch(), lock_package("vector-compute-shaders", "0.11.0"))
        self.write_vendor(root, name="unreviewed-shaders")
        self.assertTrue(any("package identity differs" in error for error in validate(root, policy)))

    def test_unregistered_local_adapter_is_not_covered_by_another_vendor_exception(self):
        manifest = f'[workspace]\n[patch."{HYPERMEDIA}"]\nrenderer={{package="vector-gpu-render-adapter",path="vendor/renderer"}}\n'
        root, policy = self.fixture(manifest, lock_package("vector-gpu-render-adapter", "0.15.0"))
        self.write_vendor(root, "vendor/renderer", "vector-gpu-render-adapter", "0.15.0")
        self.assertTrue(any("no approved local override" in error for error in validate(root, policy)))

    def test_checks_actual_dependency_package_even_under_an_alias(self):
        manifest = '[workspace]\n[workspace.dependencies]\nrenderer={package="vector-gpu-render-adapter",version="0.15"}\n'
        root, policy = self.fixture(manifest)
        self.assertTrue(any("reviewed repository" in error for error in validate(root, policy)))


if __name__ == "__main__":
    unittest.main()
