# Current Chora worker intake — 15 November 2026

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

## Result

The current cached worker build and fresh exact-artifact staging pass. The actual-worker baseline fails before readiness; no page/host-denial/pixel acceptance follows. Supplemental crypto execution was skipped because baseline did not pass. No engine, worker, client, dependency or sibling source was edited. The sole Cargo/build slot is released.

## Sources and commands

Evidence directory: `C:/github/Aether/target/production-audit/intake-20261115T073844Z`.

The coordinator supplied source-gated current `worker-metadata.json` (762 packages; worker profile independently selects Etalon 8e8 and Hypermedia af57; no Qualia foundation claim). `worker-source-before.json` records Chora HEAD `f42e01c686ba24aa6f25d24ad308a6c004077220`, its tracked diff hash, untracked file hashes, and Aether worker/protocol hashes. `worker-verification-after.json` finds no drift in those recorded inputs and no staged-file hash errors. Chora tracked diff SHA-256: `0448bcb48ecae5c185e600c17fb427adf177aa343698ab233b4aaffc2d81592a`. This does not inventory newly appearing untracked paths or qualify other Aether profiles.

Build cwd: `C:/github/chora`. Environment:

- `CARGO_HOME=C:/github/Aether/target/production-audit/cargo-home`
- `CARGO_TARGET_DIR=C:/github/chora/target`
- `LIBCLANG_PATH=C:/Program Files/LLVM/bin`
- `MOZJS_ARCHIVE=C:/github/Aether/target/production-audit/native-inputs/mozjs-153.3.0-3-dpoz2sa1/libmozjs-x86_64-pc-windows-msvc.tar.gz`
- `CARGO_BUILD_JOBS=1`

```powershell
cargo build --manifest-path C:/github/Aether/packages/chora-worker/Cargo.toml --locked --offline --message-format=json -j 1
```

`worker-build-outcome.json`, `worker-build.jsonl`, and `worker-build.stderr.log` retain execution. Exit 0; cached dev build 54.40 seconds. Warnings remain: unused gpu-command-profiler/vector-compute-renderer/vector-compute-shaders patches, unmatched tikv-jemalloc-sys dev profile entry, and cached servo-script-bindings uv temporary-file access denied followed by default-Python fallback. No warning suppression or dependency update was used. JSONL SHA-256 `fa4fd7d9721bc4ec4d89c54797f0ad07abf03f021b9ee22a22608a0cd0680a30`; metadata SHA-256 `c9933d867de7eac6aa4643978cad9123bb59fd440b0cc59e8fdb89858e0147f5`.

From Aether:

```powershell
python packages/chora-worker/stage_runtime.py target/production-audit/intake-20261115T073844Z/worker-build.jsonl target/production-audit/intake-20261115T073844Z/worker-metadata.json --target-root C:/github/chora/target --llvm-readobj 'C:/Program Files/LLVM/bin/llvm-readobj.exe' --native-archive-receipt target/production-audit/native-inputs/mozjs-153.3.0-3-dpoz2sa1/source-receipt.json
```

Stage exit 0. New stage `C:/github/Aether/target/production-audit/chora-runtime/20261115T075101Z-ukl5i8a_`; receipt SHA-256 `967559182531494a708a892d6eb63cf412e124949744434456d6098d4c96c3b4`. Old incomplete stage preserved. Script verified native archive, PE imports and installed MSVC identities, copied notices including both Aether root licenses and Chora license, and emitted dependency-license inventory. Complete distribution notices remain a release gate.

| Artifact | SHA-256 |
| --- | --- |
| aether-chora-worker.exe | `3f5df5a418d901b98abcd5f5e96ef6c75b8642f5f1ac02d31e8656c39029170e` |
| libEGL.dll | `7de7768c507068f4e0724ecc5f2b217016194102de0f607397c6b3ba6373c3f0` |
| libGLESv2.dll | `da1496e9f05d1ec25285ed26073315ecc960e70f21645795837cacf8727e3083` |

## Actual-worker probe failure

```powershell
& scripts/run_native_probe.ps1 -Executable C:/github/Aether/target/chora-client/debug/examples/worker_readback.exe -LogPrefix C:/github/Aether/target/production-audit/intake-20261115T073844Z/worker-baseline -ProbeProfile 'actual-chora-worker-loopback-denial-rgba' -TimeoutSeconds 45 -ProgramArguments @('C:/github/Aether/target/production-audit/chora-runtime/20261115T075101Z-ukl5i8a_/aether-chora-worker.exe')
```

Harness SHA-256 `a3f3e57ec771fbe9e560b44179cfb056ecd1f5a23b3a0496fb14168c5016763b`. `worker-baseline.json` reports program/runner exit 1, elapsed 34.833 seconds, timed_out=false, capture_complete=true, process_exited=true. stderr: `Error: "worker failed to become ready"`; no successful fixture assertions. Harness source waits 30 seconds for readiness. Inner worker stderr is explicitly drained to `io::sink()` in `packages/chora-client/src/lib.rs`, so the captured outer stderr cannot identify the worker's startup cause. The observed failure is startup readiness expiration, not the runner's 45-second watchdog.

A separate supervised no-input diagnostic ran the exact staged executable using `run_native_probe.ps1`, profile `startup-diagnostic-no-private-pipe`, timeout 15 seconds, prefix `worker-direct-noinput`. It exited 1 after 3.394 seconds, timed_out=false/capture_complete=true, with `Chora worker stopped: private channel I/O failed`. This reaches worker entry and gives no missing-DLL loader error; absent private pipe is expected for this diagnostic. It does not establish graphics/engine initialization or explain the baseline failure.

Next bounded work: retain a bounded worker diagnostic tail in the client/readback path and report child exit/startup state, then rerun the baseline using the same stage. Any client/harness source change needs its own new build/hash receipt. Engine runtime, caller delivery, expected host denial, exact 64x64 RGBA frame, clean successful teardown, native-window integration and production qualification remain open.

## Final diagnostic implementation and approved localhost evidence

This continuation supersedes the initial runtime failure disposition and its proposed next diagnostic work. The original failures and receipts above are preserved. Subsequent authorized Aether client diagnostics were implemented; worker/protocol and Chora source remain unchanged from the recorded build/source comparison.

Only `packages/chora-client/src/lib.rs`, `packages/chora-client/src/tests.rs`, and `packages/chora-client/examples/worker_readback.rs` changed for diagnostics. The client now retains a fixed 16 KiB raw child-stderr tail, using a 2 KiB reader buffer. Background reading does not forward output to the parent console or block the UI. Snapshot access uses `try_lock` with bounded copying; truncation, EOF completion, read failure and poisoned-lock recovery are explicit. Decoding happens in the diagnostic harness. No host authority or protocol admission behavior changed. Independent review found no actionable defects.

`client-diagnostics-tests-outcome.json` records exit 0. `client-diagnostics-tests.stdout.log` records 13 unit tests and three real process tests passed, zero failed/ignored, plus zero doctests. New tests verify exact tail retention beyond three times the capacity (including non-UTF8 bytes), read-error preservation, poison recovery and immediate Busy under contention. Builds recorded in `client-diagnostics-build-outcome.json`, `client-milestones-build-outcome.json` and `client-http-diagnostics-build-outcome.json` each exit 0 (33.01s, 16.85s, 11.57s). Compiler logs retain hard-link incremental-cache fallback warnings. All commands used the cached standalone client target `C:/github/Aether/target/chora-client`, `--locked --offline -j 1`, and the recorded Aether Cargo home; tests enabled `test-fixture`.

The readback harness now reports bounded milestone timings and fixed HTTP counters and attaches diagnostic tails to failures. Startup still has a 30-second limit. Its separately labeled fixture budget is 30 seconds after readiness, with a 75-second outer runner deadline and bounded shutdown. This changes test diagnostics/budget attribution, not the production startup contract.

### Preserved default-sandbox probes

| Receipt prefix in intake directory | Exit / duration | Observed scope |
| --- | --- | --- |
| `worker-baseline` | 1 / 34.833s | Original startup readiness failure; original harness discarded child stderr. |
| `worker-baseline-diagnostics` | 1 / 38.547s | Readiness reached; shared startup/fixture budget ended at bridge/pixel deadline. |
| `worker-baseline-milestones` | 1 / 38.395s | Ready at 879ms; first white 64x64 frame at 1099ms; 1869 captures, no host denial, empty retained stderr. |
| `worker-baseline-http-diagnostics` | 1 / 34.209s | Ready at 899ms; first white 64x64 frame at 1163ms; 1853 captures, no host denial; HTTP accepted=0/root=0; empty retained stderr. |

All four receipts report timed_out=false, capture_complete=true, process_exited=true. No failure log was overwritten. Zero HTTP acceptance in the restricted run did not establish a code defect. The coordinator reran the identical bounded localhost fixture through approved execution, preserving the restricted failures rather than weakening checks or altering source/security policy.

### Approved execution results

The final harness SHA-256 is `c500ec7767b44867a89c0b5df660c0ab1a6732a75c742a859a5111eedc674004`. Both approved probes use that harness and the exact new staged worker above. Crypto fixture `C:/github/Aether/target/production-audit/worker-crypto-fixture-20261114/unscoped-host-handles.https.any.js` has SHA-256 `0ea8bd68dfb16a15b7835dd5e1d0632bd71b486bbcf20574bc34577065b195f8`.

The coordinator executed from `C:/github/Aether`:

```powershell
& C:/github/Aether/scripts/run_native_probe.ps1 -Executable C:/github/Aether/target/chora-client/debug/examples/worker_readback.exe -LogPrefix C:/github/Aether/target/production-audit/intake-20261115T073844Z/worker-baseline-approved-localhost -ProbeProfile 'actual-chora-worker-loopback-denial-rgba-approved-localhost' -TimeoutSeconds 75 -ProgramArguments @('C:/github/Aether/target/production-audit/chora-runtime/20261115T075101Z-ukl5i8a_/aether-chora-worker.exe')

& C:/github/Aether/scripts/run_native_probe.ps1 -Executable C:/github/Aether/target/chora-client/debug/examples/worker_readback.exe -LogPrefix C:/github/Aether/target/production-audit/intake-20261115T073844Z/worker-crypto-approved-localhost -ProbeProfile 'actual-chora-worker-three-webcrypto-cases-page-window-adapter' -TimeoutSeconds 75 -ProgramArguments @('C:/github/Aether/target/production-audit/chora-runtime/20261115T075101Z-ukl5i8a_/aether-chora-worker.exe', 'C:/github/Aether/target/production-audit/worker-crypto-fixture-20261114/unscoped-host-handles.https.any.js')
```

`worker-baseline-approved-localhost.json`: program/runner exit 0, elapsed 3.125s, timed_out=false, capture_complete=true, process_exited=true. stdout records ready at 1030ms, page request and host-denial enqueue at 1435ms, one accepted HTTP root request, zero malformed requests/write failures, and successful exact opaque 64x64 RGBA assertion after JS promise rejection. stderr is empty.

`worker-crypto-approved-localhost.json`: program/runner exit 0, elapsed 2.734s, timed_out=false, capture_complete=true, process_exited=true. stdout records ready at 724ms, page request at 987ms, host-denial enqueue at 988ms, three HTTP connections/one root, zero malformed requests/write failures, and successful exactly-three secure-context WebCrypto page-window cases with the same opaque 64x64 pixel assertion. stderr is empty.

The successful assertions check the actual engine-derived caller/request, expected host denial, every pixel `[68,85,102,255]` in a 64x64 frame, and successful bounded child reap. The supplemental result is three page-window crypto cases using the copied fixture, not full WPT or dedicated-worker qualification. These are actual separate-worker component/process results. Native-window display, OS containment, private browsing, authorized QualiaDB/VibeScript application behavior, installed distribution and production qualification remain separate open gates. No native readiness claim follows from the helper test/build or these offscreen worker probes alone.

Documentation settled after reading the final approved receipts. No further Cargo, worker launch or source changes were performed for this final evidence append. Build/runtime slots remain released.
