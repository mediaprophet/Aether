# Verification evidence, 2026-11-12

The final adapter, concrete foundation, durable authoring bridge and Aether owner
scheduler binding compiled, linked and passed **25 tests**
on Windows with `rustc 1.99.1 (58cf8f9c3 2026-10-13)`. The final tests linked existing
canonical QualiaDB build artifacts. This is actual runtime/API evidence, not a
fresh build or verification of the pinned Cargo dependency graph. The cached
core-db artifact includes more features than this crate's `default-features=false`
dependency requests; that requested minimal profile remains unqualified.

## Exact final inputs and command

Source under test: all current `packages/qualia-host/src` files, including the
mandatory concrete `foundation.rs` module. Existing canonical artifacts:

- `C:/github/qualiaDB/target/debug/deps/libqualia_core_db.rlib`
- `C:/github/qualiaDB/target/debug/deps/libvibe-9b52f50e04df6110.rlib`
- `C:/github/qualiaDB/target/debug/deps/libvibescript_abi-8979b075383fe0c0.rlib`
- `C:/github/qualiaDB/target/debug/deps/libqualia_cooperative_core-ea47681ddc82fb93.rlib`
- `C:/github/qualiaDB/target/debug/deps/libtempfile-342229f48ccfdf07.rlib` (tests only)
- `C:/github/Aether/target/native-core-contracts/document/target/debug/deps/libaether_core-bb04a1a9c6ff06e4.rlib`
- Their actual dependency artifacts from the same canonical target directory.

The local artifacts' complete build source/feature provenance is not independently
attested. They must not be substituted for release dependency-source validation.

```powershell
rustc --edition 2024 --test packages/qualia-host/src/lib.rs `
  --crate-name aether_qualia_host `
  --extern vibe=C:/github/qualiaDB/target/debug/deps/libvibe-9b52f50e04df6110.rlib `
  --extern vibescript_abi=C:/github/qualiaDB/target/debug/deps/libvibescript_abi-8979b075383fe0c0.rlib `
  --extern qualia_core_db=C:/github/qualiaDB/target/debug/deps/libqualia_core_db.rlib `
  --extern qualia_cooperative_core=C:/github/qualiaDB/target/debug/deps/libqualia_cooperative_core-ea47681ddc82fb93.rlib `
  --extern tempfile=C:/github/qualiaDB/target/debug/deps/libtempfile-342229f48ccfdf07.rlib `
  --extern aether_core=C:/github/Aether/target/native-core-contracts/document/target/debug/deps/libaether_core-bb04a1a9c6ff06e4.rlib `
  -L dependency=C:/github/qualiaDB/target/debug/deps `
  -L dependency=C:/github/Aether/target/native-core-contracts/document/target/debug/deps `
  -L native=C:/Users/Hp/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/windows_x86_64_msvc-0.48.5/lib `
  -L native=C:/Users/Hp/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/windows_x86_64_msvc-0.52.6/lib `
  -o C:/github/Aether/target/qualia-host-contract/host-authoring-tests.exe
& C:/github/Aether/target/qualia-host-contract/host-authoring-tests.exe --test-threads=1 --nocapture
```

Result: `25 passed; 0 failed; 0 ignored`, 2.40 seconds after the final queued-value
teardown hardening. Covered:

- Canonical interpreter computation and instruction exhaustion.
- Canonical source-session ingress rejecting 20,000 nested groups before parsing.
- Rejection of capability arguments that ID-only grants cannot prove.
- Original authority-ticket identity even with broader approved envelopes.
- Canonical provider admission denial, expiry, actor mismatch, flagged simulation,
  and receipt commit failure.
- Runtime ownership, navigation epoch, owner teardown, bounded pending deliveries,
  source bounds, result bounds and native-thread delivery.
- Concrete foundation rejecting detached snapshots.
- Actual Webizen Sentinel deontic admission of a mechanics operation, canonical
  engine execution, two RAM audit NQuins, one receipt, and UI-owner result delivery.
- Actual signed native software-instrument admission, key-possession session,
  source-review request creation and canonical file-synced journal acknowledgement;
  reopen restores the request but rejects the old session and requires fresh proof.
- Native authoring view revocation/navigation invalidation, actual journal entry
  exhaustion and reconciliation, snapshot representation bounds, and safe retry
  after canonical authority lock contention. These use real temporary journals,
  not an in-memory authority substitute.
- Async mailbox task wake, waiter replacement, producer EOF after pending results,
  and retained-waker release on receiver teardown. Reentrant teardown/replacement
  regressions make the final stored waker own the sender: its release can reacquire
  the mailbox mutex without deadlocking.
- Actual Aether `VirtualDom`/component tasks: one receiver factory call across
  rerender, bounded per-poll owner-thread delivery, and DOM teardown cancelling
  subsequent runtime publication.

The durable fixture persists a source-review request record. It does not prove
persisted source-byte editing, graph mutation, or durable Vibe extension receipts.
The VirtualDom tests do not create a platform window or exercise GPU rendering.

An earlier 12-test adapter probe also passed after freshly compiling the current
canonical `crates/vibe/src/lib.rs`, using cached ABI/Etalon/bytemuck/unicode artifacts.
That probe predates the concrete foundation addition and is supplementary only.

## Cargo qualification blockers

The intended command is:

```powershell
cargo test --manifest-path packages/qualia-host/Cargo.toml --offline
```

With the writable audit cache this failed before compilation because pinned
QualiaDB revision `afbe431195167fde0fadc233ed79fd2fe85f73f0` was not available in its
Git checkout cache. A disposable local-source harness using unchanged canonical
QualiaDB files and Etalon `6ace6cc` then reached an unavailable VibeScript Git
checkout (`c26acd2aacceffc0778b5b96b1a935e251fd5e96`) during offline resolution,
despite the canonical ABI patch. A directory-source fallback was rejected because
the existing registry source cache lacks expected checksum files. No checksums,
canonical source, ABI implementations or policy implementations were fabricated.

Complete the ordinary locked Cargo build with canonical source provenance and
minimal feature qualification before release. Platform window contexts/teardown,
durable receipts, scoped graph access, restart/offline behavior, complete native
application rendering and Etalon device ownership remain separate required gates.
