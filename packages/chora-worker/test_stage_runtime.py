# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
"""Artifact-selection fixtures; these do not qualify any worker executable."""
import copy
from pathlib import Path
import os
import tempfile
import unittest

import stage_runtime as staging


class ArtifactSelectionTests(unittest.TestCase):
    def setUp(self):
        self.target = staging.REPOSITORY.parent / "chora/target"
        self.metadata = {"workspace_root": str(staging.WORKER), "resolve": {"root": "worker"}, "packages": [
            {"id": "worker", "name": "aether-chora-worker", "manifest_path": str(staging.WORKER / "Cargo.toml")},
            {"id": "angle", "name": "mozangle", "manifest_path": "unused"},
        ]}
        self.messages = [
            {"reason": "compiler-artifact", "package_id": "worker", "target": {"name": "aether-chora-worker", "kind": ["bin"]}, "executable": str(self.target / "debug/aether-chora-worker.exe")},
            {"reason": "build-script-executed", "package_id": "angle", "out_dir": str(self.target / "debug/build/mozangle-exact/out")},
            {"reason": "build-finished", "success": True},
        ]

    def select(self):
        return staging.select_artifacts(self.messages, self.metadata, self.target)

    def test_exact_successful_artifacts_selected(self):
        executable, out_dir, angle = self.select()
        self.assertEqual(executable.name, "aether-chora-worker.exe")
        self.assertEqual(out_dir, (self.target / "debug/build/mozangle-exact/out").resolve())
        self.assertEqual(angle["id"], "angle")

    def test_failed_build_is_rejected(self):
        self.messages[-1]["success"] = False
        with self.assertRaises(ValueError):
            self.select()

    def test_check_without_executable_is_rejected(self):
        self.messages[0]["executable"] = None
        with self.assertRaises(ValueError):
            self.select()

    def test_other_package_with_same_target_name_is_rejected(self):
        self.messages[0]["package_id"] = "other-worker"
        with self.assertRaises(ValueError):
            self.select()

    def test_different_angle_out_dirs_are_rejected(self):
        other = copy.deepcopy(self.messages[1])
        other["out_dir"] = str(self.target / "debug/build/mozangle-other/out")
        self.messages.insert(2, other)
        with self.assertRaises(ValueError):
            self.select()

    def test_missing_exact_angle_record_is_rejected(self):
        self.messages[1]["package_id"] = "other-angle"
        with self.assertRaises(ValueError):
            self.select()

    def test_executable_outside_target_is_rejected(self):
        self.messages[0]["executable"] = str(staging.WORKER / "aether-chora-worker.exe")
        with self.assertRaises(ValueError):
            self.select()

    def test_out_dir_outside_target_is_rejected(self):
        self.messages[1]["out_dir"] = str(staging.WORKER / "out")
        with self.assertRaises(ValueError):
            self.select()

    def test_parent_workspace_metadata_is_rejected(self):
        self.metadata["workspace_root"] = str(staging.REPOSITORY)
        with self.assertRaises(ValueError):
            self.select()


class DestinationAncestryTests(unittest.TestCase):
    def setUp(self):
        self.base = staging.checked_output_directory(staging.REPOSITORY, staging.REPOSITORY / "target/production-audit")
        self.temporary = tempfile.TemporaryDirectory(prefix="worker-stage-test-", dir=self.base)
        self.directory = Path(self.temporary.name)
        self.root = self.directory / "repository"
        self.root.mkdir()
        self.addCleanup(self.cleanup)

    def cleanup(self):
        # Only this fixture's verified unique directory is recursively cleaned.
        if not self.directory.resolve().is_relative_to(self.base.resolve()):
            raise AssertionError("Test cleanup directory escaped its explicit base")
        self.temporary.cleanup()

    def test_missing_normal_suffix_is_admitted_without_creating_it(self):
        destination = self.root / "target/production-audit/chora-runtime"
        self.assertEqual(staging.checked_output_directory(self.root, destination), destination)
        self.assertFalse(destination.exists())

    def test_outside_destination_is_rejected(self):
        with self.assertRaises(ValueError):
            staging.checked_output_directory(self.root, self.directory / "outside")

    def test_non_directory_ancestor_is_rejected(self):
        (self.root / "target").write_text("fixture", encoding="utf-8")
        with self.assertRaises(ValueError):
            staging.checked_output_directory(self.root, self.root / "target/chora-runtime")

    def test_actual_redirected_ancestor_is_rejected_before_writes(self):
        outside = self.directory / "outside"
        outside.mkdir()
        link = self.root / "target"
        if os.name == "nt":
            import _winapi
            _winapi.CreateJunction(str(outside), str(link))
        else:
            link.symlink_to(outside, target_is_directory=True)
        try:
            self.assertEqual(link.resolve(), outside.resolve())
            with self.assertRaises(ValueError):
                staging.checked_output_directory(self.root, link / "production-audit/chora-runtime")
            self.assertEqual(list(outside.iterdir()), [])
        finally:
            # Remove only the fixture redirect itself before recursive cleanup.
            if os.name == "nt":
                os.rmdir(link)
            else:
                link.unlink()


if __name__ == "__main__":
    unittest.main()
