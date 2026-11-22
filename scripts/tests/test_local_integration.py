"""Reject mixed GPU implementations in the opt-in local consumer's real graph gate."""
# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
import contextlib
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from prepare_local_integration import canonical_patch_sources, verify, write_test_patches


class LocalIntegrationGateTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(prefix="aether-local-gpu-policy-")
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.source = self.root / "etalon"
        self.receipt = self.root / "source-receipt.json"
        self.receipt.write_text(json.dumps({
            "etalon_source": str(self.source),
            "hypermedia_source": str(self.root / "hypermedia"),
            "adapter_manifest_sha256": {},
        }), encoding="utf-8")
        self.metadata = self.root / "metadata.json"
        self.etalon = self.package("etalon", self.source / "etalon")

    def package(self, name, directory, source=None, version="0.0.2"):
        return {"name": name, "manifest_path": str(directory / "Cargo.toml"), "source": source, "version": version}

    def check(self, packages, resolved=True):
        self.metadata.write_text(json.dumps({
            "packages": packages, "resolve": {"nodes": []} if resolved else None,
        }), encoding="utf-8")
        with patch("prepare_local_integration.pinned_source", return_value=self.source), contextlib.redirect_stdout(io.StringIO()):
            verify(self.metadata, self.receipt)

    def test_accepts_one_canonical_gpu_and_independent_registry_shader_tool(self):
        self.check([
            self.etalon,
            self.package("etalon-core", self.source / "etalon-core"),
            self.package("naga", self.root / "registry/naga", "registry+https://github.com/rust-lang/crates.io-index"),
        ])

    def test_rejects_same_name_gpu_from_another_revision(self):
        with self.assertRaisesRegex(ValueError, "GPU source differs"):
            self.check([self.etalon, self.package("etalon-core", self.root / "old/etalon-core", "git+https://github.com/mediaprophet/etalon.git?rev=old#old")])

    def test_reviewed_non_gpu_overlays_do_not_exempt_gpu_or_kernel_sources(self):
        receipt = json.loads(self.receipt.read_text())
        overlays = {name:{"manifest_path":str(self.root / "reviewed" / name / "Cargo.toml")}
                    for name in ("etalon-model","etalon-model-io")}
        receipt.update(reviewed_etalon_overlays=overlays,qualia_source=str(self.root / "qualia"))
        self.receipt.write_text(json.dumps(receipt))
        packages = [self.etalon,*[self.package(name,Path(entry["manifest_path"]).parent) for name,entry in overlays.items()]]
        with patch("prepare_local_integration.profile_overlays",return_value=overlays):
            self.check(packages)
            for name in ("etalon-core","etalon-inference-kernel","etalon-agent"):
                with self.assertRaisesRegex(ValueError,"GPU source differs"):
                    self.check([*packages,self.package(name,self.root / "mutable" / name)])

    def test_rejects_upstream_gpu_transitive_package(self):
        with self.assertRaisesRegex(ValueError, "upstream GPU implementation"):
            self.check([self.etalon, self.package("wgpu-core-deps-windows-linux-android", self.root / "registry/wgpu", "registry+https://github.com/rust-lang/crates.io-index")])

    def test_accepts_only_canonical_compatibility_shim(self):
        self.check([self.etalon, self.package("wgpu", self.source / "compat/wgpu", version="30.0.0")])
        with self.assertRaisesRegex(ValueError, "upstream GPU implementation"):
            self.check([self.etalon, self.package("wgpu", self.root / "shim")])
        with self.assertRaisesRegex(ValueError, "upstream GPU implementation"):
            self.check([self.etalon, self.package("wgpu", self.source / "compat/wgpu", version="30.0.1")])
        with self.assertRaisesRegex(ValueError, "upstream GPU implementation"):
            self.check([self.etalon, self.package("wgpu", self.source / "compat/unreviewed", version="30.0.0")])

    def test_rejects_remote_adapter_even_when_gpu_is_canonical(self):
        with self.assertRaisesRegex(ValueError, "adapter is outside"):
            self.check([self.etalon, self.package("gpu-device-context", self.root / "git/adapter", "git+https://github.com/mediaprophet/hypermedia-production.git")])

    def test_rejects_adapter_inside_wrong_subdirectory_of_selected_tree(self):
        with self.assertRaisesRegex(ValueError, "adapter is outside"):
            self.check([self.etalon, self.package("gpu-device-context", self.root / "hypermedia/unreviewed")])

    def test_canonical_workspace_patch_cannot_resolve_from_registry(self):
        manifest = self.root / "qualia/vendor/imap-proto/Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('[package]\nname="imap-proto"\nversion="0.16.6"\n')
        receipt = json.loads(self.receipt.read_text())
        receipt["canonical_patches"] = {"imap-proto": {
            "manifest_path": str(manifest), "version": "0.16.6",
            "sha256": hashlib.sha256(manifest.read_bytes()).hexdigest(), "registry": "crates-io"}}
        self.receipt.write_text(json.dumps(receipt))
        self.check([self.etalon, self.package("imap-proto", manifest.parent, version="0.16.6")])
        with self.assertRaisesRegex(ValueError, "Canonical workspace patch source differs"):
            self.check([self.etalon, self.package("imap-proto", self.root / "registry/imap", "registry+https://github.com/rust-lang/crates.io-index", "0.16.6")])

    def test_requires_full_resolved_graph(self):
        with self.assertRaisesRegex(ValueError, "Full resolved Cargo metadata"):
            self.check([self.etalon], resolved=False)
        with self.assertRaisesRegex(ValueError, "No Etalon implementation"):
            self.check([])

    def test_rejects_registry_blitz_in_a_local_contract_profile(self):
        receipt = json.loads(self.receipt.read_text())
        receipt["blitz_source"] = str(self.root / "blitz")
        self.receipt.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, "reviewed local network contract"):
            self.check([self.etalon, self.package("blitz-traits", self.root / "registry/blitz-traits", "registry+https://github.com/rust-lang/crates.io-index")])

    def test_requires_the_actual_canonical_host_packages(self):
        receipt = json.loads(self.receipt.read_text())
        receipt.update(with_host=True, qualia_source=str(self.root / "qualia"))
        self.receipt.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, "Required canonical host packages missing"):
            self.check([self.etalon])
        packages = [self.package(name, self.root / "qualia/crates" / name)
                    for name in ("vibe", "vibescript-abi", "qualia-core-db", "qualia-cooperative-core")]
        self.check([self.etalon, *packages])
        with self.assertRaisesRegex(ValueError, "host SDK is outside"):
            self.check([self.etalon, *packages, self.package("vibescript-abi", self.root / "qualia/duplicate")])

    def test_rejects_mixed_style_sources_even_with_equal_versions(self):
        receipt = json.loads(self.receipt.read_text())
        receipt["stylo"] = {"revision": "reviewed", "packages": ["stylo", "selectors"]}
        self.receipt.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, "Mixed Stylo source identity"):
            self.check([self.etalon, self.package("selectors", self.root / "registry/selectors", "registry+https://github.com/rust-lang/crates.io-index", version="0.41.0")])

    def test_engine_only_graph_cannot_pull_host_authority_or_language_engine(self):
        receipt = json.loads(self.receipt.read_text())
        receipt["engine_only"] = True
        self.receipt.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, "host-specific package"):
            self.check([self.etalon, self.package("vibe", self.root / "qualia/crates/vibe")])

    def test_generic_engine_uses_core_without_requiring_the_high_level_wrapper(self):
        receipt = json.loads(self.receipt.read_text())
        receipt["engine_only"] = True
        self.receipt.write_text(json.dumps(receipt))
        self.check([self.package("etalon-core", self.source / "etalon-core")])
        with self.assertRaisesRegex(ValueError, "No Etalon implementation"):
            self.check([self.package("etalon-types", self.source / "etalon-types")])

    def test_adapter_qualification_rejects_application_core_and_requires_real_adapters(self):
        receipt = json.loads(self.receipt.read_text())
        receipt["gpu_adapters_only"] = True
        self.receipt.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, "application/engine dependency"):
            self.check([self.etalon, self.package("qualia-core-db", self.root / "qualia/crates/qualia-core-db")])
        with self.assertRaisesRegex(ValueError, "Required GPU adapters missing"):
            self.check([self.etalon])


class CanonicalPatchCaptureTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(prefix="aether-canonical-patches-")
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name).resolve()
        self.manifest = self.root / "vendor/imap-proto/Cargo.toml"
        self.manifest.parent.mkdir(parents=True)
        self.manifest.write_text('[package]\nname="imap-proto"\nversion="0.16.6"\n')
        (self.root / "Cargo.toml").write_text('[workspace]\n[patch.crates-io]\nimap-proto={path="vendor/imap-proto"}\n')

    def test_retains_canonical_patch_identity_and_manifest_hash(self):
        entries = canonical_patch_sources(self.root, {"packages": {"imap-proto": self.manifest}})
        self.assertEqual(entries["imap-proto"]["manifest_path"], str(self.manifest))
        self.assertEqual(entries["imap-proto"]["sha256"], hashlib.sha256(self.manifest.read_bytes()).hexdigest())

    def test_rejects_uncaptured_canonical_patch(self):
        with self.assertRaisesRegex(ValueError, "was not captured"):
            canonical_patch_sources(self.root, {"packages": {}})

    def test_unknown_patch_source_requires_explicit_review(self):
        (self.root / "Cargo.toml").write_text('[workspace]\n[patch."https://example.test/other"]\nother={path="vendor/imap-proto"}\n')
        with self.assertRaisesRegex(ValueError, "explicit integration review"):
            canonical_patch_sources(self.root)


class CapturedTestRootGateTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(prefix="aether-captured-test-policy-")
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name).resolve()
        self.source = self.root / "etalon"
        self.snapshot_file = self.root / "snapshot.json"
        self.snapshot_file.write_text("{}")
        self.core = self.root / "snapshot/qualiaDB/crates/qualia-core-db/Cargo.toml"
        self.host = self.root / "snapshot/Aether/packages/qualia-host/Cargo.toml"
        self.snapshot = {"root": self.root / "snapshot",
                         "qualia_source": self.root / "snapshot/qualiaDB",
                         "host_source": self.host.parent,
                         "packages": {"qualia-core-db": self.core, "aether-qualia-host": self.host}}
        self.receipt = self.root / "receipt.json"
        self.receipt.write_text(json.dumps({
            "etalon_source": str(self.source), "hypermedia_source": str(self.root / "hypermedia"),
            "adapter_manifest_sha256": {}, "with_host": True,
            "snapshot_receipt": str(self.snapshot_file),
            "snapshot_receipt_sha256": hashlib.sha256(self.snapshot_file.read_bytes()).hexdigest(),
            "qualia_source": str(self.snapshot["qualia_source"]), "host_source": str(self.host.parent),
        }))

    def check(self, packages, members, test_root="core", nodes=None):
        metadata = self.root / "metadata.json"
        metadata.write_text(json.dumps({"packages": packages, "workspace_members": members,
                                        "resolve": {"nodes": nodes or []}}))
        with patch("prepare_local_integration.pinned_source", return_value=self.source), \
             patch("prepare_local_integration.verify_source_snapshot", return_value=self.snapshot), \
             contextlib.redirect_stdout(io.StringIO()):
            verify(metadata, self.receipt, test_root)

    def package(self, name="qualia-core-db", path=None):
        return {"id": name, "name": name, "manifest_path": str(path or self.core),
                "source": None, "version": "0.1.0"}

    def test_actual_cpu_test_root_accepts_captured_subset_without_native_gpu(self):
        self.check([self.package()], ["qualia-core-db"])

    def test_dependency_presence_does_not_substitute_for_actual_workspace_tests(self):
        with self.assertRaisesRegex(ValueError, "not the captured workspace package"):
            self.check([self.package()], ["some-consumer"])

    def test_rejects_same_name_test_package_outside_snapshot(self):
        with self.assertRaisesRegex(ValueError, "not the captured workspace package"):
            self.check([self.package(path=self.root / "live/Cargo.toml")], ["qualia-core-db"])

    def test_subset_gate_still_rejects_unrecorded_canonical_dependencies(self):
        with self.assertRaisesRegex(ValueError, "outside the captured dependency closure"):
            self.check([self.package(), self.package("qualia-unknown", self.root / "live/Cargo.toml")], ["qualia-core-db"])

    def test_captured_test_gate_rejects_live_transport_even_with_valid_canonical_core(self):
        for name in ("aether-chora-client", "aether-chora-bridge-protocol"):
            with self.assertRaisesRegex(ValueError, "outside the captured dependency closure"):
                self.check([self.package(), self.package(name, self.root / "live" / name / "Cargo.toml")], ["qualia-core-db"])

    def test_captured_client_fixture_requires_additional_worker_input_capture(self):
        name = "aether-chora-client"
        manifest = self.root / "snapshot/Aether/packages/chora-client/Cargo.toml"
        self.snapshot["packages"][name] = manifest
        with self.assertRaisesRegex(ValueError, "uncaptured worker helper"):
            self.check([self.package(), self.package(name,manifest)], ["qualia-core-db"], nodes=[{"id": name, "features": ["test-fixture"]}])

    def test_full_consumer_preserves_but_does_not_activate_unused_canonical_patch(self):
        packages = [self.package(), self.package("aether-qualia-host", self.host)]
        for name in ("vibe", "vibescript-abi", "qualia-cooperative-core"):
            manifest = self.snapshot["qualia_source"] / "crates" / name / "Cargo.toml"
            self.snapshot["packages"][name] = manifest
            packages.append(self.package(name, manifest))
        unused = self.snapshot["qualia_source"] / "vendor/imap-proto/Cargo.toml"
        unused.parent.mkdir(parents=True)
        unused.write_text('[package]\nname="imap-proto"\nversion="0.16.6"\n')
        self.snapshot["packages"]["imap-proto"] = unused
        # Capturing an optional client is necessary for reproducible standalone
        # pump tests, but the minimal native/host profile must not activate it.
        self.snapshot["packages"]["aether-chora-client"] = self.root / "snapshot/Aether/packages/chora-client/Cargo.toml"
        receipt = json.loads(self.receipt.read_text())
        receipt["canonical_patches"] = {"imap-proto": {"manifest_path": str(unused),
            "version": "0.16.6", "sha256": hashlib.sha256(unused.read_bytes()).hexdigest()}}
        self.receipt.write_text(json.dumps(receipt))
        gpu = self.package("etalon", self.source / "etalon/Cargo.toml")
        gpu["version"] = "0.0.2"
        self.check([*packages, gpu], [], test_root=None)

    def test_patch_extraction_preserves_tables_and_rejects_nonlocal_specs(self):
        manifest = self.root / "Cargo.toml"
        output = self.root / "patches.toml"
        manifest.write_text('[package]\nname="consumer"\n[patch."https://example.test/repo"]\ncore={path=' + json.dumps(self.core.parent.as_posix()) + '}\n')
        write_test_patches(manifest, output)
        self.assertIn(self.core.parent.as_posix(), output.read_text())
        manifest.write_text('[patch.crates-io]\ncore={version="1"}\n')
        with self.assertRaisesRegex(ValueError, "absolute path-only"):
            write_test_patches(manifest, output)


if __name__ == "__main__":
    unittest.main()
