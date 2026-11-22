#!/usr/bin/env python3
"""Apply a Chora patch series in a fresh temporary directory, never in its source.

Records exact input hashes, checks sequential git-apply applicability and final source-text
agreement with the reviewed candidate for every file named by the patches. This
is patch integration evidence, not an engine compilation or runtime test.
"""
# Copyright (c) 2026 Timothy Charles Holborn. Aether repository licensing applies.
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import tempfile


def sha(data):
    return hashlib.sha256(data).hexdigest()


def patch_paths(patch):
    result = set()
    for line in patch.read_text(encoding="utf-8").splitlines():
        if line.startswith(("--- a/", "+++ b/")):
            value = line[6:].split("\t", 1)[0]
            path = PurePosixPath(value)
            if path.is_absolute() or ".." in path.parts or "\\" in value or ":" in value or path.parts[0] == ".git":
                raise ValueError(f"unsafe patch path {value!r}")
            result.add(value)
    if not result:
        raise ValueError(f"no source files in {patch}")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--patch", type=Path, action="append", required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    source = args.source.resolve(strict=True)
    candidate = args.candidate.resolve(strict=True)
    patches = [patch.resolve(strict=True) for patch in args.patch]
    paths = sorted(set().union(*(patch_paths(patch) for patch in patches)))
    staging = Path(tempfile.mkdtemp(prefix="chora-patch-verification-"))
    report = {"source": str(source), "candidate": str(candidate), "staging": str(staging),
              "scope": "All patch-listed files; no full source-distribution or engine-build claim", "patches": [], "files": {}}
    for relative in paths:
        original = source / relative
        if original.exists():
            if not original.resolve().is_relative_to(source):
                raise ValueError(f"source symlink leaves checkout: {relative}")
            dest = staging / relative
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(original, dest)
            report["files"][relative] = {"baseline_sha256": sha(original.read_bytes())}
        else:
            report["files"][relative] = {"baseline_sha256": None}
    ok = True
    for patch in patches:
        entry = {"path": str(patch), "sha256": sha(patch.read_bytes())}
        check = subprocess.run(["git", "-c", "core.autocrlf=false", "apply", "--check", str(patch)], cwd=staging, capture_output=True, text=True)
        entry.update(check_exit=check.returncode, check_output=check.stdout + check.stderr)
        if not check.returncode:
            apply = subprocess.run(["git", "-c", "core.autocrlf=false", "apply", str(patch)], cwd=staging, capture_output=True, text=True)
            entry.update(apply_exit=apply.returncode, apply_output=apply.stdout + apply.stderr)
            ok = apply.returncode == 0
        else:
            ok = False
        report["patches"].append(entry)
        if not ok:
            break
    if ok:
        for relative, entry in report["files"].items():
            actual, expected = staging / relative, candidate / relative
            a = actual.read_bytes() if actual.exists() else None
            e = expected.read_bytes() if expected.exists() else None
            same = a == e
            equivalent = same or (a is not None and e is not None and a.replace(b"\r\n", b"\n") == e.replace(b"\r\n", b"\n"))
            entry.update(applied_sha256=sha(a) if a is not None else None,
                         candidate_sha256=sha(e) if e is not None else None,
                         exact_bytes=same, source_text_matches=equivalent)
            ok = ok and equivalent
    report["passed"] = ok
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"Patch series {'passed' if ok else 'FAILED'}; report: {args.report}; staged files: {len(paths)}")
    return 0 if ok else 1

if __name__ == "__main__":
    raise SystemExit(main())
