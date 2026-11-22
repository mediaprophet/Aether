# Native input source probe

This independent Cargo workspace compiles the actual repository
`packages/native-dom/src/lib.rs` and its source modules. Its library and Aether
package paths are relative to this directory. It contains no production-source
copy, fake renderer, substituted dependency or mock event implementation.

The fixture uses real Blitz 0.3.0-beta.4, minimal Aether facade features, and disables
default native features. Its lockfile records the independently resolved cached
headless graph. It is deliberately different from the shipping root lockfile and
is not a native window, GPU, default-feature, minimum-Rust or packaging test.

Run on Windows with an MSVC Rust toolchain and the locked dependencies available:

```powershell
./notes/implementation/native-input-probe/verify.ps1
```

An optional writable Cargo home or target directory can reuse existing caches:

```powershell
./notes/implementation/native-input-probe/verify.ps1 -CargoHome ./target/production-audit/cargo-home -TargetDirectory ./target/production-audit/native-input-tests/target
```

The script always uses `--locked --offline --lib` and does not change the shipping
manifest, lockfile or source files. The optional Cargo-home override lasts only for
the script invocation. Missing cached dependencies produce a failure rather than
network fetching or dependency substitution.

Evidence recorded on 12 November 2026 with Rust 1.99.1, Windows MSVC:
`../native-input-tests.log` contains the final successful 20-test run against the
same source/manifest/lock content in its initial scratch location. The IME fixture
resolves layout before input delivery so the native text editor is initialized.
`../native-cpu-check.log` records the separate root-lock-seeded actual native CPU
check blocked by missing `utf8-width` metadata. It is not a success receipt for
native window compilation.
