#!/usr/bin/env python3
"""Enforce Aether's reviewed Etalon and Hypermedia sources without resolving dependencies.

Checks every dependency declaration in workspace/package manifests and every
matching package in Cargo.lock. This includes optional and target-specific
entries, but does not determine which entries are active for a feature/target
selection. See scripts/gpu-policy.json for the reviewed pin and shim exception.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys
import tomllib


GPU_FAMILY = re.compile(r"^(?:etalon(?:[-_].*)?|naga(?:[-_].*)?)$", re.IGNORECASE)
PROHIBITED = {"wgpu-core", "wgpu-hal", "wgpu-types"}
MANIFEST_DIRS = ("packages", "examples", "refract", "vendor")
DEP_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")


def source_for(url: str, revision: str) -> str:
    return f"git+{url}?rev={revision}#{revision}"


def manifest_paths(root: Path):
    yield root / "Cargo.toml"
    workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8-sig"))
    for pattern in workspace.get("workspace", {}).get("members", []):
        for member in root.glob(pattern):
            manifest = member / "Cargo.toml"
            if manifest.is_file():
                yield manifest
    for directory_name in MANIFEST_DIRS:
        directory = root / directory_name
        if directory.exists():
            for path in directory.rglob("Cargo.toml"):
                if not {"target", "node_modules", ".git"}.intersection(path.parts):
                    yield path


def dependency_tables(manifest: dict):
    for key in DEP_TABLES:
        yield key, manifest.get(key, {})
    for target, config in manifest.get("target", {}).items():
        for key in DEP_TABLES:
            yield f"target.{target}.{key}", config.get(key, {})
    yield "workspace.dependencies", manifest.get("workspace", {}).get("dependencies", {})


def normalized(name: str) -> str:
    return name.lower().replace("_", "-")


def upstream_component(name: str) -> bool:
    name = normalized(name)
    return name in PROHIBITED or name in {"wgpu-naga-bridge", "wgpu-sync"} or name.startswith(("wgpu-core-", "wgpu-hal-", "wgpu-types-"))


def hypermedia_overrides(root: Path, workspace: dict, rule: dict) -> tuple[dict, list[str]]:
    approved = {}
    errors = []
    declarations = workspace.get("patch", {}).get(rule["git_url"], {})
    for alias, spec in declarations.items():
        name = spec.get("package", alias) if isinstance(spec, dict) else alias
        expected = rule.get("local_overrides", {}).get(name)
        if not expected or not isinstance(spec, dict) or "path" not in spec or "git" in spec:
            errors.append(f"Hypermedia patch {name}: no approved local override")
            continue
        manifest = (root / spec["path"] / "Cargo.toml").resolve()
        expected_manifest = (root / expected["manifest"]).resolve()
        if not expected_manifest.is_relative_to(root) or manifest != expected_manifest:
            errors.append(f"Hypermedia patch {name}: unapproved local manifest path {manifest}")
            continue
        if name in approved:
            errors.append(f"Hypermedia patch {name}: duplicate package override")
            continue
        try:
            package = tomllib.loads(manifest.read_text(encoding="utf-8-sig"))["package"]
        except (OSError, KeyError, tomllib.TOMLDecodeError) as exc:
            errors.append(f"Hypermedia patch {name}: cannot inspect approved manifest: {exc}")
            continue
        if package.get("name") != name or package.get("version") != expected["version"]:
            errors.append(f"Hypermedia patch {name}: approved manifest package identity differs")
            continue
        approved[name] = manifest
    return approved, errors


def validate(root: Path, policy_path: Path) -> list[str]:
    policy = json.loads(policy_path.read_text(encoding="utf-8"))
    url = policy["etalon_git_url"]
    revision = policy["etalon_revision"]
    etalon_version = policy["etalon_version"]
    expected_source = source_for(url, revision)
    shim = policy["compatibility_shim"]
    shim_source = source_for(shim["git_url"], shim["revision"]) if shim else None
    errors: list[str] = []
    hypermedia = policy.get("hypermedia")
    if policy.get("policy_version", 1) >= 2 and not hypermedia:
        return ["GPU policy version 2 requires the reviewed Hypermedia source rule"]

    try:
        workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8-sig"))
        workspace_deps = workspace.get("workspace", {}).get("dependencies", {})
    except (OSError, tomllib.TOMLDecodeError) as exc:
        return [f"Cargo.toml: cannot parse workspace manifest: {exc}"]

    hypermedia_local = {}
    if hypermedia:
        hypermedia_local, local_errors = hypermedia_overrides(root, workspace, hypermedia)
        errors.extend(local_errors)

    manifests = sorted(set(manifest_paths(root)))
    for path in manifests:
        try:
            manifest = tomllib.loads(path.read_text(encoding="utf-8-sig"))
        except (OSError, tomllib.TOMLDecodeError) as exc:
            errors.append(f"{path.relative_to(root).as_posix()}: cannot parse manifest: {exc}")
            continue
        relpath = path.relative_to(root).as_posix()
        for table, dependencies in dependency_tables(manifest):
            for alias, raw_spec in dependencies.items():
                spec = {"version": raw_spec} if isinstance(raw_spec, str) else raw_spec
                resolved = dict(spec)
                if spec.get("workspace"):
                    inherited = workspace_deps.get(alias, {})
                    inherited = {"version": inherited} if isinstance(inherited, str) else inherited
                    resolved = {**inherited, **spec}
                package = resolved.get("package", alias)
                alias_norm = normalized(alias)
                package_norm = normalized(package)
                location = f"{relpath} [{table}] {alias} (package {package})"

                if hypermedia and (package in hypermedia["packages"] or resolved.get("git") == hypermedia["git_url"]):
                    if package not in hypermedia["packages"]:
                        errors.append(f"{location}: package is not in the reviewed Hypermedia inventory")
                    elif "path" in resolved:
                        base = root if spec.get("workspace") else path.parent
                        manifest = (base / resolved["path"] / "Cargo.toml").resolve()
                        if hypermedia_local.get(package) != manifest:
                            errors.append(f"{location}: unapproved Hypermedia local manifest path {manifest}")
                    elif resolved.get("git") != hypermedia["git_url"]:
                        errors.append(f"{location}: Hypermedia package must use its reviewed repository")
                    elif resolved.get("rev") and resolved["rev"] != hypermedia["revision"]:
                        errors.append(f"{location}: Hypermedia revision differs from its reviewed commit")

                if upstream_component(package):
                    errors.append(f"{location}: prohibited upstream wgpu core/hal/types dependency")
                    continue

                if package_norm != "wgpu" and (alias_norm == "wgpu" or alias_norm.startswith("wgpu-")):
                    errors.append(f"{location}: active Etalon dependencies must use Etalon naming, not a historical wgpu alias")

                if package_norm == "wgpu":
                    is_compat_shim = shim is not None and package == shim["package"] and resolved.get("git") == shim["git_url"] and resolved.get("rev") == shim["revision"] and str(resolved.get("version", "")).lstrip("=") == shim["version"]
                    if not is_compat_shim:
                        errors.append(f"{location}: upstream wgpu is forbidden; only an explicitly pinned Etalon compatibility shim is permitted")
                    continue

                # Etalon and its forked Naga components may only resolve from
                # the reviewed Etalon repository and revision. Other Naga
                # packages (for example registry Naga) remain allowed.
                if GPU_FAMILY.match(package) and ("etalon" in package_norm or "etalon" in str(resolved.get("git", ""))):
                    if resolved.get("git") != url or resolved.get("rev") != revision:
                        errors.append(f"{location}: Etalon dependency must use {expected_source}")

    lock_path = root / "Cargo.lock"
    try:
        lock = tomllib.loads(lock_path.read_text(encoding="utf-8-sig"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        return errors + [f"Cargo.lock: cannot parse lockfile: {exc}"]

    etalon_sources = set()
    for package in lock.get("package", []):
        name = package.get("name", "")
        name_norm = normalized(name)
        source = package.get("source", "")
        version = str(package.get("version", ""))
        package_label = f"Cargo.lock {name}@{version}"

        if hypermedia:
            prefix = "git+" + hypermedia["git_url"]
            from_hypermedia = source.startswith(prefix + "?") or source.startswith(prefix + "#")
            if name in hypermedia["packages"] or from_hypermedia:
                if name not in hypermedia["packages"]:
                    errors.append(f"{package_label}: package is not in the reviewed Hypermedia inventory")
                elif not source:
                    expected = hypermedia.get("local_overrides", {}).get(name)
                    if name not in hypermedia_local or not expected or version != expected["version"]:
                        errors.append(f"{package_label}: Hypermedia local source is not an approved override")
                elif not from_hypermedia or source.rsplit("#", 1)[-1] != hypermedia["revision"]:
                    errors.append(f"{package_label}: Hypermedia source does not match reviewed commit {hypermedia['revision']} (source: {source})")

        if upstream_component(name):
            errors.append(f"{package_label}: prohibited upstream wgpu core/hal/types package is locked (source: {source or 'local'})")
            continue

        if name == "wgpu":
            if shim is None or source != shim_source or version != shim["version"]:
                errors.append(f"{package_label}: wgpu package is not the explicitly pinned Etalon compatibility shim (source: {source or 'local'})")
            else:
                etalon_sources.add(source)
            continue

        if name_norm.startswith("etalon-") or name_norm == "etalon":
            etalon_sources.add(source)
            if source != expected_source:
                errors.append(f"{package_label}: Etalon family source {source or 'local'} does not match reviewed source {expected_source}")
            if version != etalon_version:
                errors.append(f"{package_label}: Etalon family version {version} does not match reviewed version {etalon_version}")
        elif (name_norm == "naga" or name_norm.startswith("naga-")) and "etalon" in source:
            etalon_sources.add(source)
            if source != expected_source:
                errors.append(f"{package_label}: Etalon Naga source {source} does not match reviewed source {expected_source}")

    if len(etalon_sources) > 1:
        errors.append("Cargo.lock: mixed Etalon-family source revisions detected: " + ", ".join(sorted(etalon_sources)))
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--policy", type=Path, default=Path(__file__).resolve().with_name("gpu-policy.json"))
    args = parser.parse_args()
    errors = validate(args.root.resolve(), args.policy.resolve())
    if errors:
        for error in errors:
            print(f"::error::{error}")
        return 1
    print("GPU dependency policy passed for inspected GPU declarations and full lockfile.")
    print("Scope: optional and target-specific lock entries included; active feature/target graph not evaluated.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
