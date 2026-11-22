"""Copy production TLS bodies and exact Chora regressions into an isolated harness.

This does not compile Chora or exercise engine/resource-thread configuration wiring.
"""
from pathlib import Path
import argparse
import hashlib
import re

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--root", type=Path, default=Path(r"C:/Users/Hp/AppData/Local/Temp/chora-etalon-overhaul"))
parser.add_argument("--fixture", type=Path, default=Path(r"C:/github/Aether/target/production-audit/chora-tls-tests"))
args = parser.parse_args()
ROOT, FIXTURE = args.root, args.fixture
FIXTURE.mkdir(parents=True, exist_ok=True)
source = (ROOT / "components/net/connector.rs").read_text(encoding="utf8")
manager = source[
    source.index("#[derive(Clone, Debug, Default)]\nstruct CertificateErrorOverrideManagerInternal"):
    source.index("#[derive(Clone)]\nstruct TokioExecutor")
]
verifier = source[source.index("/// Global crypto provider cache"):source.index("pub type BoxedBody")]
# Remove instrumentation only. TLS implementation bodies remain unchanged.
body = re.sub(r"#\[servo_tracing::instrument(?:\([^\n]*\))?\]\n", "", manager + verifier)
header = """#![allow(dead_code)]
use std::{collections::HashMap, sync::{Arc, LazyLock}};
use parking_lot::Mutex;
use log::warn;
use rustls::client::danger::ServerCertVerifier;
use rustls::crypto::{CryptoProvider, aws_lc_rs};
use rustls_pki_types::{CertificateDer, ServerName, UnixTime};
use rustls_pki_types::pem::PemObject;
pub use rustls::client::ResolvesClientCert;
pub type TlsConfig = rustls::ClientConfig;
// Fix the default preference in this fixture; explicit policy branches are unchanged.
macro_rules! pref { (network_use_webpki_roots) => { true }; }
"""
(FIXTURE / "connector.rs").write_text(header + body, encoding="utf8", newline="\n")
tests = (ROOT / "components/net/tests/tls_configuration.rs").read_bytes()
(FIXTURE / "tls_configuration.rs").write_bytes(tests)
(FIXTURE / "lib.rs").write_text("extern crate self as net;\npub mod connector;\n#[cfg(test)] mod tls_configuration;\n")
(FIXTURE / "Cargo.toml").write_text('''[package]
name="chora-tls-configuration-verification"
version="0.0.0"
edition="2024"
[lib]
name="net"
path="lib.rs"
[workspace]
[dependencies]
rustls={version="=0.23.46",default-features=false,features=["aws-lc-rs","logging","std","tls12"]}
rustls-pki-types="=1.15.1"
rustls-platform-verifier="=0.7.1"
webpki-roots="=1.0.11"
parking_lot="=0.12.5"
log="0.4"
[dev-dependencies]
rcgen="=0.13.2"
''')
print("Generated", FIXTURE)
print("Production TLS body SHA256", hashlib.sha256(body.encode()).hexdigest())
print("Exact production regression tests SHA256", hashlib.sha256(tests).hexdigest())
