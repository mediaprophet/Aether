# Upstream integration update register

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

Companion to [the continuation packets](WORK-PACKETS-2026-11-15.md).
The user authorized intake and Aether implementation against current sibling
sources, then confirmed the updates were committed on GitHub. Intake 1 records
verified commits and profile-specific acceptance. Earlier results retain their
original source scope; sibling sources remain read-only.

## Repository intake board

| Repository | Last read-only observation, 15 November | Accepted commit/profile | Owner evidence | State / released packets |
| --- | --- | --- | --- | --- |
| QualiaDB | `4ec9ab3dfb2eabf8b19af8c10bcf52c714c5c994`; further postcommit fixes followed the initially clean observation | Live source gate passed before compilation; post-run drift prevents stable-source qualification | Combined native/host cargo check passed; 28 inputs changed during the run | WP03 source policy implemented; refresh receipt before WP04 tests |
| Etalon | `057545fc14bc1202a167b9a76a8f82e246c1738f`; scoped inputs matched the post-build recheck | 25 GPU packages selected from this local source in the live graph | Substantive parent `97568cfa66dfb8eb789f6c00a4ae223c1f40247c`; combined check passed | Resource sharing and alternate renderer/runtime profiles remain open |
| hypermedia-production | `a01384e6b79fa0f80d4de0d0de94da38f184b3d0` | Explicit consumer patch tables select the live source | Combined native/host cargo check passed with hybrid renderer | Native/media runtime and other renderer profiles remain open |
| Chora | `f42e01c686ba24aa6f25d24ad308a6c004077220` plus preserved local implementation | Separate worker Etalon `8e8ab6c` / Hypermedia `af57e821` profile | Current 762-package source gate, build, stage and approved localhost probes passed | WP06 worker caller/denial/pixels and three supplemental page-window crypto cases passed; native window/containment remain open |
| vibe-script / canonical ABI | Canonical engine and `vibescript-abi` supplied by QualiaDB `4ec9ab3` | Reuse canonical embedding/SDK contracts; tooling remains separate | Source reviewed; live host rerun pending | No new scripting ABI or engine introduced |

The working trees may have changed since observation. No clean-tree or successful
current-suite assertion follows from the hashes above. The former 14-entry Qualia
patch table is historical; inspect the current table rather than reusing its count.

## Requested upstream handoff information

Use information already in commits/logs first; ask the user or owner only for
material missing inputs. Do not require them to reconstruct information available
locally. No automatic messages to external owners are authorized by this register.

- Full commit(s), relevant branch, and whether required changes remain uncommitted.
- Changed public Rust/SDK APIs, package moves, feature defaults, MSRV and lockfiles.
- Exact Etalon/Hypermedia/Qualia ABI identity expected together; local patch tables
  versus committed dependency pins, and any compatibility overlays still needed.
- Completed commands with target/profile/toolchain, result, logs, hardware/OS where
  applicable, and explicit warning/error/skip/known-failure disposition.
- Supported EMF/QAC format/profile versions and canonical entry points; complete
  versus partial/range behavior; scientific/archival/presentation distinctions.
- Streaming retry/cancel/backpressure/completion semantics and multi-leaf evidence.
- Notice/resource packaging changes and downstream migration instructions.

## Review items to recheck, not presumed outstanding after fixes

These were source-review observations on a moving tree, without new test execution.
Each must receive a disposition against the accepted commit: fixed + evidence;
intentional restriction + enforced contract; superseded; or unresolved + owner.
Do not reopen an already-fixed issue based only on this table.

| ID | Observed concern / source location | Required resolution evidence | Affected packets | Disposition |
| --- | --- | --- | --- | --- |
| U01 | Native `bundle/stream/mixed/transfer.rs` admitted TenD/Q42/M64 IDs to `present` without inner-format checks; HTTP mixed admitted TenD/M64 similarly | Malformed family-tagged bytes cannot satisfy mandatory geometry/calibration dependencies; canonical admission used | WP05, WP09, WP10 | Unresolved at `4ec9ab3`; Aether must perform canonical inner admission |
| U02 | `plan/schedule.rs` marks issued before `service/mux.rs` transport; errors leave unit issued/occupied without retained retry progress | WouldBlock at each protocol step; resumable state or explicit terminal abort/cleanup; authority/reservation ownership preserved | WP05, WP09 | Unresolved at `4ec9ab3`; do not claim resumability after this driver errors |
| U03 | Mixed native join schedules one leaf per entry without whole-entry continuation | Multi-leaf entry exact completion or explicit enforced single-leaf restriction; distinguish leaf report from object completion | WP05, WP09, WP13 | Unresolved at `4ec9ab3`; leaf count is not object completion |
| U04 | HTTP mixed reads scheduled leaf but serves `[0,total)` | Nonzero region selection honored or explicitly refused; bounded output/fairness/cancellation behavior | WP05, WP09 | Unresolved at `4ec9ab3`; exclude this driver from selected-range consumption |
| U05 | Frozen `emf-qac-machine-schemas-v1.json` advertises `Q42AC064` alias; canonical reader/adversarial test rejects it | Versioned normative contract agrees with producer/reader; preserve historical frozen artifact and identify successor | WP10 | Documentation mismatch at `4ec9ab3`; canonical reader enforces padded qac magic, cutover supersedes old alias obligation |
| U06 | `emf_qac_perf_realtime_gate.rs` calls a synchronous ten-block test realtime while allowing five seconds for about 107 ms of audio | Clearly labeled coarse gate; actual per-block deadline/underrun/device evidence for playback claims | WP12, WP14 | Qualification gap at `4ec9ab3`; count only as coarse synchronous regression |

Additional acceptance constraints: current top-level EMF/QAC stream facades were
complete-entry paths; lower-level EMF range readers did not establish end-to-end
progressive consumption. QAC observations were not playback. WSS loopback framing
was not a TLS handshake or physical-network test. Re-evaluate these constraints
against the delivered implementation, rather than assuming they remain or vanished.

## Evidence carried forward

- Aether's earlier worker build, host34/client14/protocol7 and policy86 results
  remain scoped to [their recorded profiles](HANDOFF-2026-11-14.md).
- Qualia streaming retained an earlier 148-test integrated pass. The latest
  `swarm-wave3-cargo-source-complete-2026-11-15.log` inspected during review ended
  at core compilation without a final result; this is not a claim about its
  state after the observation.
- Qualia EMF/QAC helper logs retained 81 producer/helper and 58 portal-QAC passes.
  They were not native Aether streaming display or real audio-device qualification.
- Hypermedia canonical envelope writer/reader use was source-confirmed. Current
  dirty sibling build/runtime acceptance was not established by that review.

## Intake revision template — append a new section for each update

### Intake <number> — <date / user notification reference>

**State:** received / verified / accepted-for-profile / rejected-with-reason.

| Repository/profile | Supplied commit | Verified checkout + relevant dirty hashes | Lock/patch/feature identity | Completed evidence | Unresolved limits |
| --- | --- | --- | --- | --- | --- |
| Fill at intake | Not supplied | Not inspected | Not inspected | Not inspected | Not assessed |

**Public contract changes:** package/API/ABI/format/profile/clock/ownership changes.

**U01–U06 dispositions:** link exact source and completed regression result, or
document an intentional supported-profile limit and its enforcement test.

**Source acceptance decision:** selected immutable revision/capture, remaining
overlays and source policy changes; why it is coherent with each consumer.

**Packet impact:** released IDs; amended contracts; blocked IDs and missing input;
old results requiring rerun. State whether this is metadata, compilation,
component/process, real-device or installed-artifact evidence.

**Build allocation:** owner, exact target/cache/profile, command, start/finish state.

**Supervisor / independent review:** reviewer role, findings and disposition.
No AI byline or co-author attribution is needed; use functional role names only.

**Next action:** one concrete bounded assignment, not a general claim of readiness.


## Independent preparation / license clarification — 15 November 2026

[Preparation evidence](preparation-2026-11-15/README.md) is available. This is not
an upstream intake: no sibling commit was accepted and no dependent packet was
released. The user confirmed that the existing shared license remains the current
temporary policy pending future computational support for mediaprophet/license
requirements. Aether now has LICENSE-QUALIA byte-identical to its existing LICENSE.
Future license migration is a separate requirements/update item; its repository
requirements were not inspected or implemented here. Preserve current and upstream
notices while planning that future transition.

## Intake 1 — 15 November 2026, committed-fix notification

**State:** verified and accepted as inputs for new live qualification. This
supersedes the waiting state above; it does not qualify a release or replace
frozen-v4 or separate-worker source identities.

GitHub commit objects and local HEAD agree for all three repositories:
[QualiaDB 4ec9ab3](https://github.com/mediaprophet/qualiaDB/commit/4ec9ab3dfb2eabf8b19af8c10bcf52c714c5c994),
[Etalon 057545f](https://github.com/mediaprophet/etalon/commit/057545fc14bc1202a167b9a76a8f82e246c1738f),
and [Hypermedia a01384e](https://github.com/mediaprophet/hypermedia-production/commit/a01384e6b79fa0f80d4de0d0de94da38f184b3d0).
Settled `git status --short -uno` was empty in each. Untracked build outputs,
including Etalon's `target-container-standards`, were preserved. Commit/tree/parent
metadata and initial working-tree observations are retained under
`target/production-audit/intake-20261115T073844Z/`; remote existence is not a test result.

**Source/API decision.** QualiaDB now declares Etalon `057545f` throughout its
canonical workspace dependencies. Its previous model/model-IO vendor redirections
are no longer the current patch contract. Hypermedia commits the new Qualia/Etalon
pins, but its dependency-local `.cargo/config.toml` still supplies canonical local
paths. Consumers must reproduce reviewed patches explicitly. The new opt-in
`--live-sources` profile binds current bytes, HEADs, dirty state and generated
consumer files, while keeping the old `6ace6cc` default and immutable frozen-v4
profile unchanged. All resolved GPU identities, canonical SDK paths and shared
client/protocol identities must pass the actual metadata gate before build results
are attributed to this profile. Root shipping pins remain separately qualified.

**Committed streaming review.** The U01–U06 implementation, canonical reader,
adversarial-test and decision files were rechecked against QualiaDB `4ec9ab3`:
`git diff --quiet HEAD -- <reviewed paths>` exited zero. Dispositions in the table
are current source findings, without a new upstream test execution. These constrain
the affected drivers; they do not block unrelated Aether preparation or builds.
Upstream fixes remain owned upstream; Aether owns enforcing its supported profile.

`EmfStreamView::admit(AuthenticatedRange, Limits)` requires a complete selected
entry, supports v4/v2/v3, and excludes opaque archival interpretation from plan
readiness. Its inner-region selector runs after complete admission. Canonical
`QacCompleteView` / `QacStreamView` likewise require complete entries; scientific
signal interpretation does not imply playback, and absent timing stays absent.
WP09 may implement this bounded complete-entry profile after the live host source
gate; progressive delivery, multi-leaf completion and physical playback stay open.

**Owner evidence reviewed.** Qualia R01 records seven standalone core gates with
30 passes, Etalon signal62/sim51/sense-core185, sensing-bridge12, Hypermedia
anatomy-bridge10 and Synth presentation2. Its accepted-source fingerprint and
resolved manifest are absent, so these do not qualify the newly selected Aether
graph. Hypermedia's final helper log records84 passes, and the filtered production
suite25 passes/200 filtered. Portal QAC records58 passes, including bounded
zero-allocation reader operations. The inspected Wave-3 streaming log ends during
compilation; no completed current integrated streaming result follows. These are
owner-log observations, not commands rerun by this intake.

**Packet allocation and review.** WP02 intake is recorded. WP03 source-profile
implementation and WP06 actual worker qualification are assigned to separate
Sol agents at medium reasoning. A Sol agent at high reasoning checks committed
streaming contracts and independently reviews WP03; the supervisor integrates
results and owns acceptance. WP04 waits only for its concrete source/build gates,
not another commit notification. WP09 uses the complete-entry restriction above.
WP07/08 and WP10–17 retain their individual runtime, format and release gates.

**Worker evidence.** The independently selected Chora worker passed current locked
offline Windows metadata/source verification (762 packages) and a fresh cached
build (54.40 seconds). A new exact development stage completed. The baseline
page/readback probe initially failed its startup wait. The client now retains a
bounded 16KiB diagnostic tail; 13 unit and three process tests pass. A restricted
rerun reached readiness and white-frame readback but received zero localhost HTTP
connections. Approved execution of the identical staged worker/harness passed the
actual caller, host denial and expected 64x64 RGBA fixture; the supplemental
three-case WebCrypto page-window adapter also passed. Restricted failures remain
in the evidence, and no full WPT, authorized Qualia bridge, native-window display
or OS containment claim follows. See
[the current worker intake](INTAKE-WORKER-2026-11-15.md) for artifact hashes and logs.

**Build allocation.** One Cargo owner at a time: worker build has finished;
client diagnostic tests/rebuild precede live consumer metadata and native/host
checks. No frozen source recapture, sibling modifications, commits, pushes or
branch changes are part of this intake.

### Compilation and subsequent source drift

The generated `target/live-integration-20261115-intake1-v2` profile resolved 673
Windows packages. Its actual source gate passed for 25 GPU packages at Etalon
`057545f`, canonical QualiaDB/VibeScript paths and one shared authority transport.
The generated receipt SHA256 is
`fb450b6324cf1ebd5560de98823836ca9d625a437decdd8a072d63803d33309d`.
Metadata/lock digests and input inventories are recorded in
`target/production-audit/intake-20261115T073844Z/live-source-gate.json`.
The initially missing Etalon commit was imported from the verified local checkout
into the isolated Cargo Git cache; sibling Git state and shipping locks were not
changed. No source identity or checksum was substituted to pass resolution.

Actual command, with the existing qualification cache:

```powershell
$env:CARGO_HOME='C:/github/Aether/target/production-audit/cargo-home'
$env:CARGO_TARGET_DIR='C:/github/Aether/target/qualification-build/native-host-20261112-v2'
$env:CARGO_BUILD_JOBS='1'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
cargo check --lib --locked --offline -j 1 --manifest-path C:/github/Aether/target/live-integration-20261115-intake1-v2/Cargo.toml
```

**Exit 0, Cargo elapsed 37m09s; outer receipt 2231.95 seconds.** This checks Aether
native/facade, canonical host/core/Vibe/ABI, current Hypermedia hybrid rendering
and the selected Etalon stack together. It is a type-check result, not executable
linkage, host test execution, GPU sharing or a signed native application run.
The log contains 14 unused-patch warnings for inactive optional packages and no
Rust compiler warnings. This is limited to this selected profile. The earlier
restricted attempt was interrupted and restarted with identical flags/cache and
approved direct execution; its incomplete logs remain preserved.

Evidence is in `live-check-direct-outcome.json` and
`live-check-direct.{stdout,stderr}.log` under the same audit directory. Fresh Cargo
version output was `cargo 1.99.1 (5f94df478 2026-08-27)`; minimum-Rust qualification
remains separate.

**Post-run gate: failed closed on QualiaDB source drift.** Aether and Etalon matched
the starting receipt at that check, but 28 QualiaDB inputs changed while the check
ran; HEAD remained `4ec9ab3`. Exact paths and before/after inventory digests are in
`post-check-qualia-drift.json`; the later state is saved separately. A separate
Hypermedia post-check inventory matched its starting state exactly. Source also
continued changing between the failed gate and the diagnostic inventory. Do not
promote the successful compile to qualification of a single immutable source
state or the latest checkout. Keep the starting receipt and create a fresh output
profile for the next qualification; do not overwrite it or disable drift checks.
The following tracking-document edits are also intentional receipt drift.

A bounded review of the additional streaming/authority diffs found target-specific
cfg/import repairs, a read-only workspace getter and equivalent session-buffer
borrowing. They do not resolve U01–U06. This was not a full security, API or WASM
review, and no upstream regression pass is inferred from the source edits.

### Next canonical host execution

The actual standalone host workspace must be tested; selecting its nonmember
package from the combined consumer is unsuitable because it has dev dependencies.
Use the current consumer's extracted patch tables and an isolated lockfile,
preserving `packages/qualia-host/Cargo.lock`:

1. Generate a fresh live profile after capturing the next settled source state.
   Preserve all sibling working-tree changes; no additional user approval or
   commit notification is required to continue within this authorization.
2. Extract `[patch.*]` tables with `--write-test-patches`. Seed an unoccupied
   audit directory's `Cargo.lock` from the new consumer lock and set
   `CARGO_RESOLVER_LOCKFILE_PATH` to that absolute filename.
3. Resolve full Windows metadata against the actual
   `packages/qualia-host/Cargo.toml`, with `--features chora-client`, `--offline`
   and `--config <generated-test-patches.toml>`. This initial isolated lock may
   acquire the host's dev dependencies; it must not replace the source lock.
4. Verify the actual live host test root, canonical SDK/GPU sources, required
   feature and shared client/protocol identities before running tests. Host-only
   metadata must not be relabeled as combined native-consumer metadata.
5. Run that same actual manifest/config with `cargo test --lib --locked --offline
   -j 1 --features chora-client -- --test-threads=1`, then record source drift and
   results. Signed native fixture, provider lifecycle and native frame-window
   qualification remain subsequent gates.

The installed Cargo documentation at
`C:/Users/Hp/.rustup/toolchains/stable-x86_64-pc-windows-msvc/share/doc/rust/html/cargo/reference/config.html#resolverlockfile-path`
documents `CARGO_RESOLVER_LOCKFILE_PATH`, a filename ending in `Cargo.lock`, and
stabilization in Rust 1.97. This avoids nightly flags, copied engines or a new ABI.
The live host test-root verifier is implemented and independently reviewed. It
admits only the actual standalone host workspace, requires the canonical Etalon
implementation and all four host SDK packages, and preserves the source, artifact,
patch and transport gates. Selected client features must be present; duplicate or
foreign transports, fixture features, native/engine packages and other live test
roots are rejected. Combined, old-default and frozen modes retain their policies.

```powershell
python -B scripts/prepare_local_integration.py --output target/<fresh-profile> --verify target/<fresh-profile>/host-metadata.json --verify-test-root host
```

Final focused policy suite: **43 passed, one skipped**, exit zero. The skipped
real-symlink test requires a Windows privilege unavailable to that test process;
deterministic reparse-parent and traversal tests passed. Independent review found
no remaining actionable findings. This follow-up changes the preparation tools
after the earlier receipt, another reason to generate a fresh profile. Actual
standalone host metadata, host test execution and linked native artifacts remain
open; no host test pass follows from these synthetic policy tests.

Intake implementation used two Sol agents at medium reasoning for disjoint source
policy and worker/client work, with a Sol agent at high reasoning for contract and
independent review. The supervisor retained shared records, build allocation and
acceptance. No sibling source was edited by this intake.

Final preservation check: 526 pre-existing files retained identical bytes; the
10 changed pre-existing files are the assigned source-policy/client changes and
six tracking documents. No pre-existing file is missing and no unexpected
pre-existing edit was found. New deliverables are the live-source helper, its
test module and the worker intake note. Only this run's new Python bytecode cache
was removed. Aether remains on `main` at `d5ccce3`; the frozen-v4 receipt retains
SHA256 `3bcee7dc35c4882ac02053f7efdf5f45de8024d5aea08a3d4cbf581d175c66b7`.
Details are in `preservation-final.json` in the intake audit directory. All owned
builds/probes have completed; unrelated processes and sibling work were retained.
