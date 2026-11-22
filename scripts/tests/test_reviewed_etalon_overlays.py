"""Exact reviewed overlay admission, portable capture, and real metadata only."""
# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import reviewed_etalon_overlays as reviewed
import snapshot_qualia_sources as capture
from source_snapshot import verify_source_snapshot
from prepare_local_integration import canonical_patch_sources

ROOT = Path(__file__).resolve().parents[2]
TABLE = {"etalon-model-io":{"path":"../etalon/crates/etalon-model-io"},
         "etalon-model":{"path":"vendor/etalon-model-6ace6cc"}}


class OverlayBoundaryTests(unittest.TestCase):
    def test_no_future_gpu_patch_name_is_admitted(self):
        table = {**TABLE,"etalon-core":{"path":"../etalon/etalon-core"}}
        with self.assertRaisesRegex(ValueError,"exact two"):
            reviewed.overlay_paths(Path("C:/fixture/qualiaDB"),table)

    def test_named_overlay_cannot_escape_expected_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            qualia = Path(temporary) / "qualiaDB"
            for name in TABLE:
                table = {key:dict(value) for key,value in TABLE.items()}
                table[name]["path"] = "../unreviewed"
                with self.assertRaisesRegex(ValueError,"escapes reviewed location"):
                    reviewed.overlay_paths(qualia,table)

    def test_expected_ancestor_link_is_rejected_before_resolve_hides_it(self):
        with tempfile.TemporaryDirectory() as temporary:
            boundary = Path(temporary)
            linked = boundary / "etalon"
            original = Path.is_symlink
            with patch.object(Path,"is_symlink",lambda path: path == linked or original(path)):
                with self.assertRaisesRegex(ValueError,"Linked overlay source ancestry"):
                    reviewed.overlay_paths(boundary / "qualiaDB",TABLE)


class ReviewedSourceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.pin = ROOT.parent / "etalon/target/aether-canonical-6ace6cc-object"
        original_model = ROOT.parent / "qualiaDB/vendor/etalon-model-6ace6cc"
        original_io = ROOT.parent / "etalon/crates/etalon-model-io"
        if not (cls.pin / "Cargo.toml").is_file() or not original_model.is_dir() or not original_io.is_dir():
            raise unittest.SkipTest("Actual reviewed ecosystem sources are required for source/metadata integration")
        cls.temporary = tempfile.TemporaryDirectory(prefix="aether-reviewed-overlays-")
        cls.addClassCleanup(cls.temporary.cleanup)
        cls.root = Path(cls.temporary.name).resolve()
        cls.qualia = cls.root / "qualiaDB"
        cls.model = cls.qualia / "vendor/etalon-model-6ace6cc"
        cls.model_io = cls.root / "etalon/crates/etalon-model-io"
        _,model_files = reviewed.model_receipt(original_model)
        expected_io = {name:entry["sha256"] for name,entry in reviewed.review()["model_io"]["files"].items()}
        if reviewed.inventory(original_io,frozenset({"target"})) != expected_io:
            raise ValueError("Actual IO source differs from reviewed test input")
        for source,destination,files in ((original_model,cls.model,model_files),(original_io,cls.model_io,expected_io)):
            for name in files:
                target = destination / name
                target.parent.mkdir(parents=True,exist_ok=True)
                shutil.copyfile(source / name,target)
        (cls.qualia / "Cargo.toml").write_text('[workspace]\nmembers = [\n]\n[patch."'+reviewed.ETALON_URL+'"]\n' +
            '\n'.join(name+' = { path = '+json.dumps(spec['path'])+' }' for name,spec in TABLE.items())+'\n')
        cls.entries,cls.objects = reviewed.verify_live(cls.qualia,cls.pin)

    @contextlib.contextmanager
    def changed(self,path,data):
        original = path.read_bytes()
        path.write_bytes(data)
        try: yield
        finally: path.write_bytes(original)

    def test_actual_reviewed_sources_bind_all_raw_bytes_and_git_objects(self):
        entries,_ = reviewed.verify_live(self.qualia,self.pin)
        self.assertEqual(len(entries["etalon-model-io"]["source_files"]),18)
        self.assertEqual(len(entries["etalon-model"]["source_files"]),125)
        comparison = [entry["comparison"] for entry in reviewed.review()["model_io"]["files"].values()]
        self.assertEqual(comparison.count("CRLF-materialization-only"),17)
        self.assertEqual(comparison.count("reviewed-platform-change"),1)

    def test_drifting_source_or_self_authored_model_receipt_is_rejected(self):
        platform = self.model_io / "src/lease/platform.rs"
        with self.changed(platform,platform.read_bytes()+b"\n// unreviewed change\n"):
            with self.assertRaisesRegex(ValueError,"IO source inventory/hash drift"):
                reviewed.verify_live(self.qualia,self.pin)
        receipt = self.model / "BACKPORT-SHA256.json"
        data = json.loads(receipt.read_text()); data["validation"] = "new unreviewed claim"
        with self.changed(receipt,json.dumps(data).encode()):
            with self.assertRaisesRegex(ValueError,"external review anchor"):
                reviewed.verify_live(self.qualia,self.pin)

    def test_crlf_description_does_not_allow_different_raw_source_bytes(self):
        source = self.model_io / "src/lib.rs"
        normalized = source.read_bytes().replace(b"\r\n",b"\n")
        self.assertNotEqual(source.read_bytes(),normalized)
        with self.changed(source,normalized):
            with self.assertRaisesRegex(ValueError,"IO source inventory/hash drift"):
                reviewed.verify_live(self.qualia,self.pin)

    def test_extra_inputs_are_rejected_and_only_known_build_directory_is_excluded(self):
        extra = self.model / "unreviewed.rs"
        extra.write_text("pub fn unreviewed() {}")
        try:
            with self.assertRaisesRegex(ValueError,"model source inventory/hash drift"):
                reviewed.verify_live(self.qualia,self.pin)
        finally: extra.unlink()
        build = self.model / "target-isolated/build-output.bin"
        build.parent.mkdir(); build.write_bytes(b"build output")
        try: reviewed.verify_live(self.qualia,self.pin)
        finally: build.unlink(); build.parent.rmdir()

    def test_materialization_uses_pinned_workspace_not_mutable_sibling(self):
        mutable_workspace = self.root / "etalon/Cargo.toml"
        mutable_workspace.write_text('[workspace.package]\nversion="99.0.0"\nlicense="unreviewed"\n')
        try:
            result = reviewed.materialized_io_manifest((self.model_io / "Cargo.toml").read_bytes(),self.objects["Cargo.toml"])
            manifest = tomllib.loads(result.decode())
            self.assertEqual(manifest["package"]["version"],"0.0.2")
            self.assertEqual(manifest["package"]["license"],"CC-BY-NC-ND-4.0")
            self.assertEqual(manifest["dependencies"],{"sha2":"0.10.9"})
            self.assertEqual(manifest["target"]["cfg(windows)"]["dependencies"],{"memmap2":"0.9.12","tempfile":"3.27.0"})
            self.assertEqual(manifest["dev-dependencies"],{"tempfile":"3.27.0"})
            self.assertNotIn("workspace",manifest["lints"])
        finally: mutable_workspace.unlink()

    def test_staging_preserves_original_manifest_licenses_and_rejects_drift(self):
        directory = self.root / "stage-source-test"
        staged = reviewed.stage_io(self.model_io,directory,self.objects)
        self.assertEqual((directory / "Cargo.toml.source").read_bytes(),(self.model_io / "Cargo.toml").read_bytes())
        for name,digest in reviewed.review()["model_io"]["license_files"].items():
            self.assertEqual(reviewed.sha((directory / name).read_bytes()),digest)
        extra = directory / "unexpected-build.rs"; extra.write_text("changed")
        with self.assertRaisesRegex(ValueError,"Existing IO staging differs"):
            reviewed.stage_io(self.model_io,directory,self.objects)

    def test_linked_intermediate_staging_parent_is_rejected_before_write(self):
        output = self.root / "linked-profile"
        linked = output / "reviewed-overlays"
        original = Path.is_symlink
        with patch.object(Path,"is_symlink",lambda path: path == linked or original(path)), \
             patch.object(reviewed,"stage_io",wraps=reviewed.stage_io) as stage:
            with self.assertRaisesRegex(ValueError,"Linked overlay source ancestry"):
                reviewed.profile_overlays(self.qualia,self.pin,output)
            stage.assert_not_called()
        self.assertFalse(output.exists())

    @unittest.skipUnless(shutil.which("cargo"),"Cargo is required for actual metadata resolution")
    def test_actual_offline_metadata_and_lock_keep_kernel_at_clean_pin(self):
        directory = self.root / "metadata-profile"
        entries = reviewed.profile_overlays(self.qualia,self.pin,directory)
        patches = canonical_patch_sources(self.qualia,overlays=entries)
        consumer = directory / "Cargo.toml"
        (directory / "src").mkdir(); (directory / "src/lib.rs").write_text("// metadata fixture only\n")
        consumer.write_text('[package]\nname="reviewed-overlay-metadata"\nversion="0.0.0"\nedition="2024"\n[workspace]\nexclude=["reviewed-overlays/etalon-model-io"]\n[dependencies]\n'+
            'etalon-model-io={path='+json.dumps(str(Path(entries['etalon-model-io']['manifest_path']).parent).replace('\\','/'))+'}\n'+
            'etalon-model={path='+json.dumps(str(self.model).replace('\\','/'))+',features=["gguf-index"]}\n'+
            '[patch."'+reviewed.ETALON_URL+'"]\netalon-inference-kernel={path='+json.dumps((self.pin / 'crates/etalon-inference-kernel').as_posix())+'}\n')
        environment = os.environ.copy()
        environment["CARGO_HOME"] = str(ROOT / "target/production-audit/cargo-home")
        environment["CARGO_TARGET_DIR"] = str(directory / "target")
        command = ["cargo","metadata","--offline","--format-version=1","--manifest-path",str(consumer)]
        result = subprocess.run(command,capture_output=True,text=True,env=environment,timeout=60)
        self.assertEqual(result.returncode,0,result.stderr)
        metadata = json.loads(result.stdout)
        family = {package["name"]:package for package in metadata["packages"] if package["name"].startswith("etalon")}
        self.assertEqual(set(family),{"etalon-model","etalon-model-io","etalon-inference-kernel"})
        self.assertEqual(Path(family["etalon-inference-kernel"]["manifest_path"]),self.pin / "crates/etalon-inference-kernel/Cargo.toml")
        self.assertTrue(all(package["source"] is None and package["version"] == "0.0.2" for package in family.values()))
        locked = subprocess.run(command+["--locked"],capture_output=True,text=True,env=environment,timeout=60)
        self.assertEqual(locked.returncode,0,locked.stderr)
        lock = tomllib.loads((directory / "Cargo.lock").read_text())
        self.assertTrue(all("source" not in package for package in lock["package"] if package["name"].startswith("etalon")))
        self.assertEqual(reviewed.profile_overlays(self.qualia,self.pin,directory,False),entries)

    def test_small_real_overlay_capture_is_portable_and_provenance_bound(self):
        aether = self.root / "Aether"
        def package(name,folder,dependencies=""):
            directory = self.root / folder
            (directory / "src").mkdir(parents=True,exist_ok=True)
            (directory / "src/lib.rs").write_text("// capture fixture only\n")
            (directory / "Cargo.toml").write_text('[package]\nname='+json.dumps(name)+'\nversion="0.0.0"\n'+dependencies)
            return directory / "Cargo.toml"
        package("aether-qualia-host","Aether/packages/qualia-host",'[dependencies]\naether-core={path="../core"}\n')
        package("aether-core","Aether/packages/core")
        canonical = ("qualia-core-db","qualia-cooperative-core","vibe","vibescript-abi")
        manifests = [package(name,"qualiaDB/crates/"+name) for name in canonical]
        original = (self.qualia / "Cargo.toml").read_bytes()
        text = original.decode().replace('members = [\n]','members = [\n'+',\n'.join(json.dumps("crates/"+name) for name in canonical)+'\n]')
        metadata = self.root / "capture-metadata.json"
        metadata.write_text(json.dumps({"resolve":{"nodes":[]},"packages":[{"name":name,"manifest_path":str(manifest)} for name,manifest in zip(canonical,manifests)]}))
        output = aether / "target/reviewed-capture"
        def git(root,*args):
            if args[0] == "rev-parse": return b"fixture-source-head\n"
            if args[0] == "diff": return b""
            prefixes = args[args.index("--")+1:]
            return b"\0".join(path.relative_to(root).as_posix().encode() for path in root.rglob("*") if path.is_file() and any(path.relative_to(root).as_posix() == p or path.relative_to(root).as_posix().startswith(p+"/") for p in prefixes))
        argv = ["snapshot_qualia_sources.py","--aether",str(aether),"--qualia",str(self.qualia),"--metadata",str(metadata),"--output",str(output),"--etalon-source",str(self.pin)]
        with self.changed(self.qualia / "Cargo.toml",text.encode()), patch.object(sys,"argv",argv), patch.object(capture,"git",git),contextlib.redirect_stdout(io.StringIO()):
            capture.main()
        result = verify_source_snapshot(output / "source-receipt.json")
        self.assertEqual(set(result["reviewed_overlays"]),reviewed.OVERLAY_NAMES)
        self.assertEqual(result["packages"]["etalon-model-io"],output / "etalon-model-io/Cargo.toml")
        self.assertTrue((output / "etalon-model-io/Cargo.toml.source").is_file())
        receipt = json.loads((output / "source-receipt.json").read_text())
        item = next(entry for entry in receipt["files"] if entry["path"] == "etalon-model-io/src/lib.rs")
        item["source"] = str(self.root / "incorrect-provenance.rs")
        (output / "source-receipt.json").write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError,"per-file source provenance differs"):
            verify_source_snapshot(output / "source-receipt.json")


if __name__ == "__main__": unittest.main()
