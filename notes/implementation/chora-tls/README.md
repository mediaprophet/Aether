# Chora C3 TLS configuration wiring

This records the original C3 checkpoint and its preserved isolated tests. The
complete series has since been applied to the local sibling checkout and Cargo
regenerated its root lock. See [current local evidence](../resume-chora.md); full
engine/TLS runtime qualification is separate from the original source-copy tests.

The patch prepares the configured root store, trust policy, client-certificate
resolver and host verifier before starting network resource threads. Both public
and private HTTP clients use the prepared configuration. WebSockets clone the
same prepared TLS template, preserving their own profile's certificate override
manager. Active connection setup does not reconstruct a fallible platform
verifier after preflight.
Each generated config also receives independent TLS resumption/session storage;
cloning rustls's template alone would share its Arc-backed session cache.

`EmbedderRootsOnly`, `EmbedderAndWebPki` and `EmbedderAndPlatform` remain distinct.
Explicit policies ignore the default-root preference. Every supplied PEM/DER
anchor is checked; empty, malformed and unreadable bundles fail instead of being
dropped or silently falling back. Explicit platform roots return an error on
Android, where this embedding profile lacks platform-verifier setup.

The builder gains `root_trust_policy`, `client_cert_resolver`,
`embedder_cert_verifier` and `try_build`. The latter reports TLS configuration and
root-file errors; it is not a general engine-startup recovery API. The legacy
`build` signature remains and explicitly panics on those errors. Existing global
options/preferences are initialized before TLS validation; retrying engine startup
after an error is not qualified here. The separate C API patch maps `try_build`
failure into its C failure path.

At the original checkpoint, live root reload was unsupported and
`NetworkManager::add_custom_root_pem` returned `ErrorKind::Unsupported` without
changing roots or active/pool connections. The resumed implementation adds a
separate host-only controller and explicit future-connection cutover; its tests
and current qualification status are recorded in [current evidence](../resume-chora.md).
Immediate revocation remains unsupported. Typed keystore registration remains
untouched pending authenticated caller authority.

## Patch and provenance

- `chora-c3-tls.patch`: nine scoped Chora files, including tests and one test-only
  `rcgen` dependency. All Chora source retains MPL headers. No ecosystem dependency
  or implementation was added.
- `patch-manifest.json`: exact original-byte and C3-patched-byte SHA256 values.
  The C3-only versions of `servo.rs` and `lib.rs` exclude the concurrent C2 producer
  changes; their patched hashes describe this patch rather than the combined tree.
- `baseline-sha256.json`: original file hashes; all eight existing files matched
  the original read-only Chora checkout. The new test file has no baseline hash.

The supervisor verified the combined series in this order: C1 GPU migration, C2
transport, C3 TLS, then C10 C API startup. Use a checkout matching the recorded
baselines. See ../chora-patch-verification.json for the combined postimages. C3 includes
only the TLS exports in `servo/lib.rs`; it contains no C2 exports or producer
handler changes. At that checkpoint, root manifests and the root lockfile were
untouched. Local resumption has now regenerated the root lock with Cargo, including
the `rcgen = "0.13.2"` test dependency.

`git apply --check` passed against an external directory containing exact baseline
files. At the original checkpoint, running git in the protected original checkout was
denied and that checkout was not modified. The resumed session subsequently
verified and applied the series to the now-writable local checkout. Full baseline source copies and editing scripts are
outside Aether and are not deliverables.

## Original checkpoint verification

`isolated-tests.log` records **9 passed, 0 failed** on Windows with Rust 1.96.1.
The fixture uses actual rustls 0.23.46, rustls-pki-types 1.15.1,
rustls-platform-verifier 0.7.1, the AWS-LC provider and rcgen 0.13.2. Its cached
WebPKI roots 1.0.11 satisfy Chora's declared `^1.0.10` requirement.

The tests exercise real in-memory rustls client/server TLS handshakes, including
custom-CA trust and unrelated-chain rejection; all three explicit root modes on
Windows; mTLS success with the configured identity and failure without one; host
verifier invocation, defer and denial; and separate profile certificate
overrides. A public/private/public sequence proves the public profile retains its
own resumable session after private browsing. Direct rustls verification also checks wrong-host, expired and
unrelated-chain failures. Root parser tests cover valid PEM/DER, malformed and
empty input, malformed trailing certificates, invalid anchors and empty
embedder-only trust.

`resumption-negative-control.log` is an intentionally failing regression check:
removing only the fresh-cache assignment from the generated fixture changes the
last public handshake from `Resumed` to `Full`. The assertion fails as expected,
showing that the test detects shared-cache interference. The production source
was untouched and the generated fixture was restored afterward.

The initial platform fixture used identical default subject names for CA and
leaf. Windows rejected that ambiguous chain. Distinct CA and leaf subjects fixed
the fixture; no production validation rule was weakened.

To reproduce after applying the patch to a Chora checkout:

```powershell
python notes/implementation/chora-tls/prepare_verification.py --root <patched-chora>
$env:RUSTUP_TOOLCHAIN='1.96.1'
$env:CARGO_HOME='C:/github/Aether/target/production-audit/cargo-home'
cargo test --offline --manifest-path target/production-audit/chora-tls-tests/Cargo.toml
```

The generator copies the production connector's TLS implementation bodies and
the exact production test file into an isolated crate. It removes instrumentation
attributes and fixes the default-root preference to WebPKI. Explicit trust-policy
branches and TLS implementation bodies remain unchanged. It prints SHA256 values
for the copied implementation and tests.

This is source-copy isolation, not a successful Chora build or a browser test.
The resumed actual embedded-library graph is tracked separately in
[resume-chora.md](../resume-chora.md). These original tests do not exercise
engine builder/resource-thread startup, sockets, DNS, HTTP/2, proxies, actual
browser WebSockets, root reload, or external embedder integration. Windows
extra-root validation does not establish every platform trust-store behavior;
Android, other operating systems, public/system-root inventories and platform
revocation policies need qualification. No PQ negotiation or ML-DSA handshake was
tested. Host `Allow` decisions deliberately override standard chain validation
and remain the host's responsibility; TLS handshake signatures still use the
configured rustls provider. Client-certificate resolver configuration is
engine-wide, not authenticated per-page authority.
