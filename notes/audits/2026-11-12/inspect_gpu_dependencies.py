"""Read Cargo manifests/lockfiles without resolving or downloading dependencies.

Usage: python inspect_gpu_dependencies.py
Outputs a source-level inventory beside this script. Lockfile edges include
optional and target-specific dependencies; they are not an active build graph.
"""

import hashlib
import json
from pathlib import Path
import re
import tomllib


OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[2]
REPOS = {"aether": ROOT, "chora": ROOT.parent / "chora"}
GPU = re.compile(r"^(?:wgpu(?:-.+)?|etalon(?:-.+)?|naga(?:-.+)?)$")


def read_toml(path):
    return tomllib.loads(path.read_text(encoding="utf-8-sig"))


def manifest_paths(root, label):
    yield root / "Cargo.toml"
    dirs = ["packages", "examples", "refract"] if label == "aether" else [
        "components", "ffi", "ports", "tests/unit", "tests/capi"
    ]
    for name in dirs:
        directory = root / name
        if directory.exists():
            for path in directory.rglob("Cargo.toml"):
                if not {"target", "node_modules", ".git"}.intersection(path.parts):
                    yield path


def dep_tables(manifest):
    for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
        yield kind, manifest.get(kind, {})
    for target, config in manifest.get("target", {}).items():
        for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
            yield f"target.{target}.{kind}", config.get(kind, {})
    yield "workspace.dependencies", manifest.get("workspace", {}).get("dependencies", {})


def inspect(root, label):
    workspace = read_toml(root / "Cargo.toml")
    shared = workspace.get("workspace", {}).get("dependencies", {})
    manifests = []
    declarations = []
    errors = []
    for path in sorted(set(manifest_paths(root, label))):
        try:
            data = read_toml(path)
        except (ValueError, OSError) as error:
            errors.append({"path": str(path), "error": str(error)})
            continue
        relative = path.relative_to(root).as_posix()
        manifests.append(relative)
        for table, deps in dep_tables(data):
            for alias, spec in deps.items():
                spec = {"version": spec} if isinstance(spec, str) else spec
                inherited = shared.get(alias, {}) if spec.get("workspace") else {}
                inherited = {"version": inherited} if isinstance(inherited, str) else inherited
                resolved = {**inherited, **spec}
                package = resolved.get("package", alias)
                if GPU.match(alias) or GPU.match(package):
                    declarations.append({"manifest": relative, "table": table,
                                         "alias": alias, "package": package,
                                         "spec": resolved})
    lock_path = root / "Cargo.lock"
    packages = read_toml(lock_path).get("package", [])
    gpu_packages = [p for p in packages if GPU.match(p["name"])]
    upstream = [p for p in gpu_packages if p["name"] in
                {"wgpu", "wgpu-core", "wgpu-hal", "wgpu-types"}
                and p.get("source", "").startswith("registry+")]
    parents = {}
    for target in gpu_packages:
        key = f'{target["name"]}@{target["version"]}'
        found = []
        for package in packages:
            for dep in package.get("dependencies", []):
                parts = dep.split()
                if parts[0] == target["name"] and (len(parts) == 1 or parts[1] == target["version"]):
                    found.append(f'{package["name"]}@{package["version"]}')
        parents[key] = sorted(set(found))
    return {
        "root": str(root), "scope": "source manifests and full lockfile, not active features",
        "manifest_count": len(manifests), "manifests": manifests,
        "lock_sha256": hashlib.sha256(lock_path.read_bytes()).hexdigest(),
        "root_manifest_sha256": hashlib.sha256((root / "Cargo.toml").read_bytes()).hexdigest(),
        "gpu_declarations": declarations, "gpu_lock_packages": gpu_packages,
        "upstream_wgpu_lock_packages": upstream, "reverse_lock_edges": parents,
        "parse_errors": errors,
    }


if __name__ == "__main__":
    report = {label: inspect(root, label) for label, root in REPOS.items()}
    destination = OUT / "gpu-dependency-inventory.json"
    destination.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    for label, repo in report.items():
        print(f'{label}: {repo["manifest_count"]} manifests; '
              f'{len(repo["upstream_wgpu_lock_packages"])} upstream wgpu packages; '
              f'{len(repo["parse_errors"])} parse errors')
        for name, parents in repo["reverse_lock_edges"].items():
            if name.startswith("wgpu@") or name.startswith("wgpu-core@"):
                print(f'  {name} <- {", ".join(parents)}')
    print(destination)
