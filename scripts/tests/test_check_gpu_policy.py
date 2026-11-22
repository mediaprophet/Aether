"""Dependency-free regression tests for the Etalon dependency policy."""

import json
from pathlib import Path
import tempfile
import unittest
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_gpu_policy import source_for, validate


REV = "575bab54accf1aafb24f67e9ba810c0f3f745aea"
OTHER_REV = "6ace6cc6413a87ca10a1da58397ddc7c0b2fc41e"
URL = "https://github.com/mediaprophet/etalon.git"
POLICY = {
    "policy_version": 1,
    "etalon_git_url": URL,
    "etalon_revision": REV,
    "etalon_version": "0.0.2",
    "compatibility_shim": {
        "package": "wgpu",
        "git_url": URL,
        "revision": REV,
        "version": "30.0.0",
    },
}


def lock_package(name, version="0.0.2", source=None):
    entry = f'[[package]]\nname = "{name}"\nversion = "{version}"\n'
    if source:
        entry += f'source = "{source}"\n'
    return entry + "\n"


class GpuPolicyTests(unittest.TestCase):
    def fixture(self, manifest="[workspace]\n", lock=""):
        temp = tempfile.TemporaryDirectory()
        root = Path(temp.name)
        (root / "Cargo.toml").write_text(manifest, encoding="utf-8")
        (root / "Cargo.lock").write_text(lock, encoding="utf-8")
        policy_path = root / "policy.json"
        policy_path.write_text(json.dumps(POLICY), encoding="utf-8")
        return temp, root, policy_path

    def check(self, manifest="[workspace]\n", lock=""):
        temp, root, policy = self.fixture(manifest, lock)
        self.addCleanup(temp.cleanup)
        return validate(root, policy)

    def test_rejects_upstream_registry_wgpu(self):
        manifest = '[workspace]\n[workspace.dependencies]\nwgpu = "30.0.1"\n'
        lock = lock_package("wgpu", "30.0.1", "registry+https://github.com/rust-lang/crates.io-index")
        errors = self.check(manifest, lock)
        self.assertTrue(any("upstream wgpu is forbidden" in error for error in errors), errors)
        self.assertTrue(any("not the explicitly pinned Etalon compatibility shim" in error for error in errors), errors)

    def test_rejects_upstream_git_wgpu_even_with_a_pinned_revision(self):
        manifest = '[workspace]\n[workspace.dependencies]\nwgpu = { version = "30.0.1", git = "https://github.com/gfx-rs/wgpu", rev = "abc123" }\n'
        errors = self.check(manifest)
        self.assertTrue(any("upstream wgpu is forbidden" in error for error in errors), errors)

    def test_rejects_prohibited_core_dependency_alias(self):
        manifest = '[workspace]\n[workspace.dependencies]\nrenderer-core = { package = "wgpu-core", version = "30.0.1" }\n'
        errors = self.check(manifest)
        self.assertTrue(any("prohibited upstream wgpu core/hal/types dependency" in error for error in errors), errors)

    def test_rejects_mixed_etalon_lock_revisions(self):
        lock = lock_package("etalon", source=source_for(URL, REV)) + lock_package("etalon-hal", source=source_for(URL, OTHER_REV))
        errors = self.check(lock=lock)
        self.assertTrue(any("does not match reviewed source" in error for error in errors), errors)
        self.assertTrue(any("mixed Etalon-family source revisions" in error for error in errors), errors)

    def test_rejects_optional_platform_component_under_an_alias(self):
        manifest = '[workspace]\n[target."cfg(windows)".dependencies]\nplatform-gpu = { package = "wgpu-core-deps-windows-linux-android", version = "30", optional = true }\n'
        self.assertTrue(any("prohibited upstream" in error for error in self.check(manifest)))

    def test_rejects_upstream_bridge_without_a_top_level_wgpu_package(self):
        errors = self.check(lock=lock_package("wgpu-naga-bridge", "30.0.1", "registry+https://github.com/rust-lang/crates.io-index"))
        self.assertTrue(any("prohibited upstream" in error for error in errors), errors)

    def test_allows_explicit_etalon_compatibility_shim(self):
        manifest = f'''[workspace]\n[workspace.dependencies]\nwgpu = {{ package = "wgpu", version = "30.0.0", git = "{URL}", rev = "{REV}" }}\n'''
        lock = lock_package("wgpu", "30.0.0", source_for(URL, REV))
        self.assertEqual([], self.check(manifest, lock))

    def test_rejects_etalon_core_under_legacy_alias(self):
        manifest = f'[workspace]\n[workspace.dependencies]\nwgpu-core = {{ package = "etalon-core", version = "0.0.2", git = "{URL}", rev = "{REV}" }}\n'
        self.assertTrue(any("must use Etalon naming" in error for error in self.check(manifest, lock_package("etalon-core", source=source_for(URL, REV)))))

    def test_accepts_canonical_etalon_dependency_name(self):
        manifest = f'[workspace]\n[workspace.dependencies]\netalon = {{ version = "0.0.2", git = "{URL}", rev = "{REV}" }}\n'
        self.assertEqual([], self.check(manifest, lock_package("etalon", source=source_for(URL, REV))))

    def test_rejects_mixed_naga_types_source(self):
        errors = self.check(lock=lock_package("etalon", source=source_for(URL, REV)) + lock_package("naga-types", source=source_for(URL, OTHER_REV)))
        self.assertTrue(any("mixed Etalon-family" in e for e in errors), errors)

    def test_checks_workspace_member_outside_conventional_directories(self):
        temp, root, policy = self.fixture('[workspace]\nmembers = ["tools/*"]\n')
        self.addCleanup(temp.cleanup)
        member = root / "tools" / "probe"
        member.mkdir(parents=True)
        (member / "Cargo.toml").write_text('[package]\nname = "probe"\nversion = "0.1.0"\n[dependencies]\nwgpu = "30"\n')
        self.assertTrue(validate(root, policy))

    def test_disabled_shim_rejects_wgpu(self):
        temp, root, policy = self.fixture(lock=lock_package("wgpu", "30.0.0", source_for(URL, REV)))
        self.addCleanup(temp.cleanup)
        config = dict(POLICY, compatibility_shim=None)
        policy.write_text(json.dumps(config))
        self.assertTrue(validate(root, policy))

    def test_allows_local_etalon_texture_example_package(self):
        manifest = '''[workspace]\nmembers = ["examples/etalon-texture"]\n'''
        lock = lock_package("aether-etalon-texture", "0.0.0")
        self.assertEqual([], self.check(manifest, lock))

    def test_checks_target_specific_and_optional_declarations(self):
        manifest = '[workspace]\n[target."cfg(unix)".dependencies]\nwgpu = { version = "30.0.1", optional = true }\n'
        errors = self.check(manifest)
        self.assertTrue(any("upstream wgpu is forbidden" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main()
