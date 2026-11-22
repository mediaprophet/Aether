# Aether-local artifact and notice preparation — 15 November 2026

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

Status: independent WP01 preparation only; WP06 staging/runtime remains **HELD** pending the user's upstream notification, accepted intake and applicable profile/source review. This receipt does not release WP06 or close W01–W04. All paths below are relative to `C:/github/Aether` unless explicitly absolute.

## Scope and evidence limits

Read `AGENTS.md`, `WORK-PACKETS-2026-11-15.md`, and `HANDOFF-2026-11-14.md`; inspected only Aether-local files. No Cargo, builds, tests, executable launch, stage creation, source/license/lock changes, deletion, commits or sibling inspection were performed. Paths into sibling repositories in existing metadata/receipts were read as strings only. Legacy receipts were not modified. Hash checks below verify existing bytes and stable size/mtime during each read; they do not establish source freshness, authenticity beyond the local receipt, or runtime behavior.

## Notice layout evidence at initial inventory

- Root `LICENSE` exists (1,940 bytes). It attributes Timothy Charles Holborn, identifies CC BY-NC-ND 4.0, includes a literal `...`, and links to the full legal code. It is not a second missing notice or a complete replacement text invented by this preparation.
- Root `LICENSE-QUALIA` is absent. Both `AGENTS.md` and `upstream-licenses/README.md` explicitly reference it alongside `LICENSE`. No license text was fabricated, copied or removed to satisfy staging.
- `upstream-licenses/README.md` preserves Dioxus origin and says inherited material retains Apache-2.0/MIT terms. `LICENSE-APACHE` (9,723 bytes) and `LICENSE-MIT` (1,023 bytes) are present there.
- `packages/chora-worker/stage_runtime.py` explicitly requires root `LICENSE`, root `LICENSE-QUALIA`, and sibling Chora `LICENSE`; copies actual mozangle/ANGLE notices; and emits a dependency-license metadata inventory only after those copies. Its repository-notice loop does not currently copy Aether's upstream-licenses directory. This is a future distribution-completeness review item, not a license interpretation or authorized script correction.
- At initial inventory the intended `LICENSE-QUALIA` layout was unresolved. The user's subsequent clarification and the supervisor's bounded correction below resolve that local filename issue. Upstream notice retention and complete transitive distribution notices remain mandatory. No present development stage qualifies distribution.

### Subsequent user clarification and local correction

The user confirmed the existing shared license remains the temporary current
policy until future `mediaprophet/license` computational requirements can be
supported. The supervisor added root `LICENSE-QUALIA` as a byte-identical copy
of Aether's existing `LICENSE`: 1,940 bytes, SHA-256
`a5ce8e4d8c923a725ea3d1ff2743bc18e1997eac544465acfead9a9b4a9486e6`.
Existing terms/notices were unchanged; no sibling source was read or copied.
Receipt: `target/production-audit/aether-preparation-20261115T065910Z/license-alias-receipt.json`.
The failed stage below was preserved, not repaired or relaunched. Future licensing
requirements are separate, unimplemented work; see [the index](README.md).

## Existing artifacts and receipts

The final recorded build is `target/production-audit/chora-worker-build-final.jsonl`: 802 compiler-artifact messages, exactly one build-finished message with success=true. `resume-chora-worker-build-final.log` records a 54.85-second cached dev build. The earlier verified-native JSONL likewise has 802 artifacts and success=true. Final metadata describes standalone `packages/chora-worker`, 762 packages, root package `aether-chora-worker@0.1.0`.

Both local JSONLs select `C:/github/chora/target/debug/aether-chora-worker.exe` and ANGLE OUT_DIR `C:/github/chora/target/debug/build/mozangle-db27f64dafd38c7c/out`. Those locations were not inspected or rehashed. No assertion is made that current sibling sources or those producer files still match this historical build. Current local manifests/lock are fingerprinted below, but the old build does not bind all source bytes; current-source freshness is therefore **unverified/held**, rather than accepted from timestamps or compilation history.

There is one directory under `target/production-audit/chora-runtime`: `20261114T044633Z-8lhjily2`. Its complete observed file inventory is:

| Relative path within partial stage | Bytes | Evidence |
| --- | ---: | --- |
| aether-chora-worker.exe | 244446208 | SHA-256 below matches handoff |
| libEGL.dll | 474112 | size only; not hashed in this bounded pass |
| libGLESv2.dll | 19616768 | size only; not hashed in this bounded pass |
| native-input-receipt.json | 2166 | preserved historical copy |
| native-input-release.json | 33938 | preserved historical copy |
| notices/aether/LICENSE | 1940 | partial notice copy |
| notices/mozangle/LICENSE | 1655 | partial notice copy |
| notices/mozangle/gfx/angle/checkout/LICENSE | 1643 | partial notice copy |

No `stage-receipt.json`, `dependency-licenses.json`, `notices/aether/LICENSE-QUALIA`, or `notices/chora/LICENSE` is present there. **No complete accepted stage was found in this designated stage root.** The failed stage remains intact and must not be supplied as an accepted worker. This is not an exhaustive search of every unrelated target directory.

The Aether-local SpiderMonkey archive is 90,609,446 bytes and rehashes to its legacy receipt/release-asset digest. Receipt package is `mozjs_sys@153.3.0-3`, features `intl`, `jit`, `libz-sys`, target `x86_64-pc-windows-msvc`; release JSON rehashes to the receipt. No network authenticity or sibling-cache check was repeated. The receipt's old source-cache path is provenance only.

The prepared `target/chora-client/debug/examples/worker_readback.exe` exists (1,254,912 bytes), but no actual-worker execution result is inferred. The copied supplemental crypto fixture rehashes to `0ea8bd68dfb16a15b7835dd5e1d0632bd71b486bbcf20574bc34577065b195f8`; its receipt identifies a page-window adapter, not full WPT/dedicated-worker qualification. No `worker-baseline*.json` or `worker-crypto*.json` was present in the top-level `notes/implementation` JSON inventory. Historical native hybrid/Vello receipts belong to different profiles and do not qualify this worker.

The immutable-v4 source receipt is fingerprinted below only by this artifact-inventory slice. The supervisor separately verified all 4,382 inputs and nine package identities without modifying the capture; see [the preparation index](README.md) and its verification receipt.

## Bounded SHA-256 inventory

| Aether-local path | Bytes | SHA-256 |
| --- | ---: | --- |
| `LICENSE` | 1940 | `a5ce8e4d8c923a725ea3d1ff2743bc18e1997eac544465acfead9a9b4a9486e6` |
| `upstream-licenses/README.md` | 2097 | `e7fec5f2c6a1220f8c6f2e8930f1d07b753000e968a4ce8311732fec9e59f0f3` |
| `upstream-licenses/LICENSE-APACHE` | 9723 | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `upstream-licenses/LICENSE-MIT` | 1023 | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `packages/chora-worker/stage_runtime.py` | 11978 | `859ba2a62022ac6dbeb5320746f808baff806baa9c7025a39b00547354b9d356` |
| `scripts/run_native_probe.ps1` | 5178 | `56fe4b88d654884a26451e15d2925f89b47724b89ccd4200a208c5aae4a6c546` |
| `packages/chora-worker/Cargo.toml` | 1765 | `956d33b303c4acbeab7f68abb80b790d6c6c8755eb587b36e82f149eec74411d` |
| `packages/chora-worker/Cargo.lock` | 234011 | `cc80cc5efb92d8adb47379ca1426e6c3bb97672259e316aa6bc70049d3b72a68` |
| `packages/chora-worker/src/main.rs` | 17091 | `310cadf4b330a356d8d72075999a69654fd1066dd6e768f44416a7fa08a3fb80` |
| `target/production-audit/chora-worker-build-final.jsonl` | 897400 | `ffefe0eb3e4f0d766f7808ae2cda603932ee6283f1aaddf5af86b97cc1a44129` |
| `target/production-audit/chora-worker-build-verified-native.jsonl` | 897548 | `ff269f0f7e619f089628ab664c70a8f234178240d27960bb5ee732d4cf0198f0` |
| `target/production-audit/chora-worker-metadata-final.json` | 3845740 | `447d59d5a151504ada9103d9d681a5de1ef2f853830568634f512e11dd40d76d` |
| `target/production-audit/native-inputs/mozjs-153.3.0-3-dpoz2sa1/source-receipt.json` | 2166 | `2c0080c84476143e637a822e5e64c73d87b6d03eec627fd06c59827adf7ef554` |
| `target/production-audit/native-inputs/mozjs-153.3.0-3-dpoz2sa1/github-release.json` | 33938 | `cee720ac762371daec57edfe16dab4caae073a6a62aba19165159a7de777600e` |
| `target/production-audit/native-inputs/mozjs-153.3.0-3-dpoz2sa1/libmozjs-x86_64-pc-windows-msvc.tar.gz` | 90609446 | `4668c61b92b178b65f555d3eb36652f16681bfeedbd1e2f41e063d4a78caace1` |
| `target/production-audit/chora-runtime/20261114T044633Z-8lhjily2/aether-chora-worker.exe` | 244446208 | `3f5df5a418d901b98abcd5f5e96ef6c75b8642f5f1ac02d31e8656c39029170e` |
| `target/chora-client/debug/examples/worker_readback.exe` | 1254912 | `a3f3e57ec771fbe9e560b44179cfb056ecd1f5a23b3a0496fb14168c5016763b` |
| `target/production-audit/worker-crypto-fixture-20261114/source-receipt.json` | 565 | `8fe5af37e4c7fc074c079bad23f723fbaa26fddb38eddbf711f4b2bba6e7c59e` |
| `target/production-audit/worker-crypto-fixture-20261114/unscoped-host-handles.https.any.js` | 2009 | `0ea8bd68dfb16a15b7835dd5e1d0632bd71b486bbcf20574bc34577065b195f8` |
| `target/qualification-sources/native-host-20261112-v4/source-receipt.json` | 1819326 | `3bcee7dc35c4882ac02053f7efdf5f45de8024d5aea08a3d4cbf581d175c66b7` |
| `notes/implementation/resume-chora-worker-build-final.log` | 1503 | `3e30f50f4498626e3dc81f7c6abfb2a4304fda41c555ea93f48bab33505f501d` |

Total hashed: **344,095,567 bytes**, below the 350,000,000-byte cap. DLLs and other unlisted large artifacts were not rehashed.


## Future recipes — HELD, not executed

Release prerequisites: explicit upstream completion notification; accepted WP02 intake and applicable WP03 source/profile alignment; sole target/runtime owner; authoritative notice-layout resolution; successful current-profile Cargo JSON and Windows metadata source gate; verified producer executable/ANGLE DLLs; matching SpiderMonkey package/features/archive receipt; installed LLVM PE reader and required MSVC runtime. The final historical inputs below are candidates only if accepted as source-current. Otherwise the responsible owner must supply replacement successful records; do not relabel stale records. Stage creation follows sibling artifact/notice paths, so it remains dependent work even though its output is inside Aether.

After those prerequisites, the exact recorded staging invocation is:

```powershell
Set-Location C:/github/Aether
python packages/chora-worker/stage_runtime.py `
  target/production-audit/chora-worker-build-final.jsonl `
  target/production-audit/chora-worker-metadata-final.json `
  --target-root C:/github/chora/target `
  --llvm-readobj 'C:/Program Files/LLVM/bin/llvm-readobj.exe' `
  --native-archive-receipt target/production-audit/native-inputs/mozjs-153.3.0-3-dpoz2sa1/source-receipt.json
```

Record exit code and stdout, which identifies a new unique stage, its executable, receipt path and receipt hash. Verify that receipt, all copied file hashes, Cargo JSON/metadata/lock hashes, PE imports, notices and installed-MSVC identities. Reject a missing receipt, hash mismatch, redirected path, source drift or incomplete notices. The staging script checks success records and archive identity, but its invocation does not independently establish that the supplied old metadata passed a current source gate. That remains an intake prerequisite.

Using PowerShell 7, set the following only from the **new accepted stage receipt**; preserve the old partial directory. The placeholder deliberately refuses execution until replaced.

```powershell
$stagedWorker = '<absolute executable from new accepted stage receipt>'
if (-not [IO.Path]::IsPathFullyQualified($stagedWorker)) { throw 'Select the accepted stage executable first.' }
# Check its SHA-256 against that receipt before invocation.
& C:/github/Aether/scripts/run_native_probe.ps1 `
  -Executable C:/github/Aether/target/chora-client/debug/examples/worker_readback.exe `
  -LogPrefix C:/github/Aether/notes/implementation/worker-baseline-NEXT `
  -ProbeProfile 'actual-chora-worker-loopback-denial-rgba' `
  -TimeoutSeconds 45 -ProgramArguments @($stagedWorker)
```

Choose a fresh `NEXT` suffix before each run; the runner writes `.json`, `.stdout.log` and `.stderr.log` and can overwrite an existing prefix. Preserve all old receipts. Acceptance requires actual caller delivery, expected denial, exact 64x64 RGBA pixels and bounded child teardown. Check runner/program exit, timeout and capture completeness, plus application assertions. The runner hashes the harness only; separately bind the new stage receipt/worker hash to the result. It is a process watchdog, not a sandbox.

After baseline acceptance, the separate supplemental recipe is:

```powershell
$cryptoFixture = 'C:/github/Aether/target/production-audit/worker-crypto-fixture-20261114/unscoped-host-handles.https.any.js'
& C:/github/Aether/scripts/run_native_probe.ps1 `
  -Executable C:/github/Aether/target/chora-client/debug/examples/worker_readback.exe `
  -LogPrefix C:/github/Aether/notes/implementation/worker-crypto-NEXT `
  -ProbeProfile 'actual-chora-worker-supplemental-page-webcrypto' `
  -TimeoutSeconds 45 -ProgramArguments @($stagedWorker, $cryptoFixture)
```

Recheck the fixture hash first; require all three cases and expected pixels. This remains supplemental page execution, not WPT, dedicated-worker, WebCrypto unit, WebGPU CTS, signed graph authorization, actual native-window display or production qualification. Crash/startup/stderr/EOF and installed-platform results remain separate WP06/WP15/WP16 gates.

## Exact checks completed and missing inputs

Completed: Python read-only parsing of both named JSONLs and final metadata; Aether-local directory/file inventory of the designated stage, native-input and crypto-fixture roots; inspection of the existing stage/probe script interfaces and local documentation; size/mtime-guarded SHA-256 reads for the table above. No tests were necessary or run for this documentation-only preparation. The hash pass was bounded before every read to 350,000,000 bytes and consumed 344,095,567 bytes.

Remaining: accepted current upstream commit/API/source evidence; current producer binary/DLL identity and complete stage receipt; current source-gate/source-build correspondence; new real-worker baseline/supplemental receipts; native frame and signed-host execution evidence; full transitive/installed/target distribution evidence. The root notice filename has been resolved as above. The full immutable-v4 inventory was revalidated separately by the supervisor, as recorded in the index. Existing artifacts are historical evidence and remain preserved.
