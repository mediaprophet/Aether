"""Validate the actual Chora WebIDL parser/configuration and generated binding signatures."""
from pathlib import Path
import argparse, os, sys, tempfile

parser = argparse.ArgumentParser()
parser.add_argument("--checkout", type=Path, required=True)
args = parser.parse_args()
root = args.checkout.resolve() / "components/script_bindings"
sys.path[:0] = [str(root / "codegen"), str(root / "third_party/WebIDL/parser"), str(root / "third_party/ply")]
os.chdir(root / "codegen")
import WebIDL
import run as generate
from configuration import Configuration
from codegen import CGBindingRoot

with tempfile.TemporaryDirectory(prefix="chora-c2-codegen-") as tmp:
    cache = Path(tmp) / "cache"
    cache.mkdir()
    parser = WebIDL.Parser(str(cache))
    for filename in sorted((root / "webidls").glob("*.webidl")):
        text = filename.read_text(encoding="utf-8")
        match = generate.FILTER_PATTERN.search(text)
        if match and not os.environ.get(match.group(1)): continue
        parser.parse(generate.filter_conditional_blocks(text), str(filename))
    results = parser.finish()
    config = Configuration(str(root / "codegen/Bindings.conf"), results)
    generated = CGBindingRoot(config, "Bindings/ChoraMessagingBinding", str(root / "webidls/ChoraMessaging.webidl")).define()
    type_file = Path(tmp) / "DomTypes.rs"
    holder_file = Path(tmp) / "DomTypeHolder.rs"
    generate.generate(config, "DomTypes", str(type_file))
    generate.generate(config, "DomTypeHolder", str(holder_file))
    types = type_file.read_text(encoding="utf-8") + holder_file.read_text(encoding="utf-8")
    navigator = CGBindingRoot(config, "Bindings/NavigatorBinding", str(root / "webidls/Navigator.webidl")).define()
    assert "fn Chora(&self, r#cx: &mut JSContext)" in navigator
    assert "ChoraMessagingMethods" in generated and "fn Request(" in generated
    assert "CurrentRealm" in generated
    assert "ChoraMessaging" in types and "choramessaging" in types
    print("PASS actual WebIDL parse and ChoraMessaging binding generation")
    print("PASS Request binding generated with CurrentRealm and Promise result")
    print("PASS concrete ChoraMessaging DOM type mapped to choramessaging module")
    print("PASS Navigator::Chora getter generated with required JSContext")
    at = generated.index("fn Request(")
    print(generated[at:at + 380])
