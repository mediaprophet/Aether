#!/usr/bin/env python3
"""Run actual document/asset public API tests in two small, locked graphs.

This does not verify the root shipping graph, native renderer, or window runtime.
Set CARGO_HOME to a writable verified cache; --offline forbids fetching dependencies.
The checked-in fixture locks describe only these targeted library dependency graphs.
"""
# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
import argparse
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def quoted(path):
    return json.dumps(path.as_posix())

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--only", choices=("document", "assets"), help="Run one library fixture")
    parser.add_argument("--output-dir", type=Path, default=ROOT / "target/native-core-contracts",
                        help="Separate scratch directory beneath Aether/target, e.g. for WSL")
    parser.add_argument("--include-junctions", action="store_true", help="Run Windows junction fixture; requires reparse-point creation permission")
    args = parser.parse_args()
    output_dir = args.output_dir.resolve()
    if not output_dir.is_relative_to(ROOT / "target"):
        parser.error("--output-dir must remain beneath Aether/target")
    cases = {
        "document": ("aether-document-lifecycle-verification", "native_lifecycle", "document", {
            "aether-document": '{ path = ' + quoted(ROOT / "packages/document") + ' }',
            "aether-core": '{ path = ' + quoted(ROOT / "packages/core") + ' }',
            "generational-box": '{ path = ' + quoted(ROOT / "packages/generational-box") + ' }',
            "futures-util": '"0.3.31"', "serde_json": '"1"',
        }),
        "assets": ("aether-asset-api-verification", "native_assets", "asset-resolver", {
            "aether-asset-resolver": '{ path = ' + quoted(ROOT / "packages/asset-resolver") + ', features = ["native"] }',
        }),
    }
    for key, (package, test, source_package, dependencies) in cases.items():
        if args.only and args.only != key:
            continue
        target = output_dir / key
        target.mkdir(parents=True, exist_ok=True)
        manifest = ('[package]\nname=' + json.dumps(package) + '\nversion="0.0.0"\nedition="2024"\nautotests=false\nautolib=false\n[workspace]\n[dependencies]\n')
        manifest += ''.join(name + '=' + spec + '\n' for name, spec in dependencies.items())
        if key == "assets":
            manifest += '[features]\ndefault=["native"]\nnative=[]\n'
        manifest += '[[test]]\nname=' + json.dumps(test) + '\npath=' + quoted(ROOT / "packages" / source_package / "tests" / (test + ".rs")) + '\n'
        if key == "assets":
            # Compile the actual private path policy as a module, retaining its tests.
            policy = ROOT / "packages/asset-resolver/src/asset_policy.rs"
            (target / "asset_policy.rs").write_text('#[path=' + quoted(policy) + ']\nmod asset_policy;\n', encoding="utf-8")
            manifest += '[[test]]\nname="asset_path_policy"\npath="asset_policy.rs"\n'
            # The policy module uses the same direct dependency as the real crate.
            manifest = manifest.replace('[dependencies]\n', '[dependencies]\npercent-encoding="2"\n')
            manifest += '\n[target.\'cfg(unix)\'.dependencies]\nlibc="0.2.186"\n'
        (target / "Cargo.toml").write_text(manifest, encoding="utf-8")
        (target / "Cargo.lock").write_bytes((ROOT / "scripts/contract-locks" / (key + ".lock")).read_bytes())
        command = ["cargo", "test", "--locked", "--manifest-path", str(target / "Cargo.toml")]
        if args.offline:
            command.append("--offline")
        if key == "assets" and args.include_junctions:
            command.extend(["--", "--include-ignored"])
        # Separate targets preserve Cargo's usual isolation from the shipping workspace.
        result = subprocess.run(command, cwd=ROOT, env=os.environ, text=True, capture_output=True)
        output = result.stdout + result.stderr
        (target / "verification.log").write_text(output, encoding="utf-8")
        print(output, end="", flush=True)
        if result.returncode:
            return result.returncode
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
