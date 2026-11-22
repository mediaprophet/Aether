"""Compile the external consumer against an explicitly selected patched Chora checkout."""
from pathlib import Path
import argparse, json, os, subprocess, tempfile, tomllib

parser = argparse.ArgumentParser()
parser.add_argument("--checkout", type=Path, required=True)
parser.add_argument("--cargo-home", type=Path)
args = parser.parse_args()
checkout = args.checkout.resolve()
root = tomllib.loads((checkout / "Cargo.toml").read_text(encoding="utf-8"))
with tempfile.TemporaryDirectory(prefix="chora-c2-consumer-") as tmp:
    target = Path(tmp)
    (target / "src").mkdir()
    (target / "src/main.rs").write_text(Path(__file__).with_name("consumer.rs").read_text(), encoding="utf-8")
    manifest = '[workspace]\n[package]\nname="chora-embedding-consumer"\nversion="0.0.0"\nedition="2024"\n[dependencies]\n'
    manifest += 'chora={package="servo",path=' + json.dumps((checkout / "components/servo").as_posix()) + ',default-features=false,features=["embedded_library"]}\n'
    # Dependency workspace patches are not inherited by an external consumer.
    for registry, patches in root.get("patch", {}).items():
        manifest += "\n[patch." + json.dumps(registry) + "]\n"
        for package, specification in patches.items():
            specification = dict(specification)
            if "path" in specification:
                specification["path"] = (checkout / specification["path"]).resolve().as_posix()
            manifest += json.dumps(package) + " = {" + ",".join(key + "=" + json.dumps(value) for key, value in specification.items()) + "}\n"
    (target / "Cargo.toml").write_text(manifest, encoding="utf-8")
    env = os.environ.copy()
    if args.cargo_home: env["CARGO_HOME"] = str(args.cargo_home.resolve())
    result = subprocess.run(["cargo", "check", "--offline", "--manifest-path", str(target / "Cargo.toml")], env=env, capture_output=True, text=True)
    print(result.stdout, end="")
    print(result.stderr, end="")
    raise SystemExit(result.returncode)
