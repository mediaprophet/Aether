# Verification record

## Completed checks

| Check | Result | What it establishes |
| --- | --- | --- |
| `python notes/audits/2026-11-12/inspect_gpu_dependencies.py` | 122 Aether manifests; 79 Chora manifests; zero parse errors | Source declaration and lockfile inventory, including optional paths |
| `cargo metadata --no-deps --offline --format-version 1` in Aether with writable Cargo home | 118 workspace packages | Workspace manifest enumeration; not dependency resolution or compilation |
| `cargo metadata --manifest-path C:/github/chora/Cargo.toml --no-deps --offline --format-version 1` | 78 workspace packages | Chora workspace manifest enumeration; not compilation |
| Existing Refract tests copied unchanged to an isolated workspace | 24 integration tests and one doctest passed; one doctest ignored | Isolated Refract behavior only; not the Aether facade or GPU integration |

The inspection includes example/manifests beyond the package count returned by
workspace metadata, so those counts need not match. The GPU inventory records
source manifest and lockfile SHA-256 hashes. Its reverse edges are derived from
the complete lockfile, not an active feature resolution.

## Native build attempts

Environment reported Rust/Cargo 1.99.1. Attempts used a writable Cargo home and
target directory under Aether's `target/production-audit`. Earlier attempts were
also blocked by default cache permissions or missing offline Git checkouts.

### Aether

```text
cargo check -p aether-native --locked --offline --config target/production-audit/offline-cache.toml
```

The temporary config attempted to use the existing unpacked registry source as a
directory source. Cargo stopped because `ab_glyph-0.2.32/.cargo-checksum.json` was
absent. An unpacked cache is not automatically a valid Cargo vendor directory.
No native compilation result follows from this attempt.
See [aether-native-check.log](aether-native-check.log).

### Chora

```text
cargo check --manifest-path C:/github/chora/Cargo.toml -p servo --no-default-features --features embedded_library --locked --offline
```

Cargo stopped before compilation because the offline cache lacked Stylo revision
`a05fadffd43bc173b2d6e1e2caf44a36b45354af`, required through `servo_arc`.
See [chora-check.log](chora-check.log).

## Not established

No successful full native build, native launch, WebGPU/Vello runtime test, WPT/CTS
run, TLS/keystore handshake fixture, sandbox test, packaged application launch or
combined QualiaDB/VibeScript/Chora service operation was obtained in this audit.
No current advisory-database scan was performed. Source-derived defects are
identified as such in the reports; compiler and behavioral reproduction are the
first acceptance work in the implementation plans.

No runtime migration was made by this audit. Existing source changes in the working
tree were preserved. The new audit documents and inventory do not establish that
their planned features have been implemented.

## Reproduction after dependency access is restored

Use normal verified Cargo registry/Git caches or a properly produced vendor tree;
do not fabricate checksums or modify package names to force resolution. Restore
the pinned revisions, run locked builds, then capture `cargo tree`/resolved metadata
for every shipping feature/target combination and the final host workspace.
Apply the regression, native interaction, browser authorization and distribution
matrices in the two implementation plans. Compilation alone closes none of the
runtime, security or packaging gates.
