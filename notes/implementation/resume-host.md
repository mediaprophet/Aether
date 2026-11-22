# Local host integration resumed — 12 November 2026

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

## Implementation in progress

The host's independent Cargo workspace now selects the actual sibling QualiaDB
sources through explicit paths. A fresh Cargo build with one compilation job is
running against its minimal-feature dependency request; cached historical rlibs
are not being treated as evidence for these changes. The generated host lockfile
records external source selection. Publication still requires a qualified immutable
canonical revision and coherent Etalon revisions across the native renderer graph.

Canonical code under QualiaDB `poet_host/native_authority/graph` adds independently
issuer-signed exact-context grants tied to the adopted admission digest and a real
key-possession session. Native source membership alone grants no graph access.
Owner incarnation, admission generation, subject, expiry and current session are
checked on every read, update and subscription poll. Admission replacement and
session disconnect invalidate derived graph scopes.

This is an authority-owned **project graph**, stored in the existing host-owned
native authority journal. It is deliberately separate from the global daemon graph:
raw daemon mutations neither change these graphs nor confer authority. All graph
writes use one optimistic-revision path with bounded context/row totals and a
journal sync acknowledgement before publication. Failed journal appends latch
reconciliation and suppress publication. Cancellation before journal I/O aborts;
cancellation after append starts cannot roll back an acknowledged commit.

Reads preserve all six NQuin lanes in caller-provided buffers. UI reads use try-lock
and report Busy rather than waiting behind a journal transaction. Subscriptions
retain a scope and last revision, coalesce to a full snapshot, and recheck authority
even if no update occurred. Aether's native view has one bounded snapshot buffer,
reports skipped revisions, clears private buffered data on authority errors, and
closes on component/navigation teardown. No callback executes under authority locks.

The canonical NativeGraphHost implements Vibe's existing Host graph methods with
explicit begin/stage/commit, canonical term conversion, scope-bounded context and
capacity, cancellation, conflict detection and durable receipts. It denies unrelated
privileged methods. Aether native_graph_runtime composes it with the canonical
WebizenSentinelGate; graph authority is separate from extension admission. The
existing scoped_native_runtime profile continues to deny graph access.

Integration uncovered and repaired two canonical scan defects: bounded `take: 0`
returned a row, and QuinRef conversion discarded metadata/parity. Scans now return
zero rows for zero take and preserve all six lanes.

## Verification queue

Added signed issuer/session, forged grant/wrong actor, revocation, exact-context,
optimistic revision, cancellation, snapshot bounds, journal exhaustion, reopen,
nonblocking UI read and coalesced-subscription tests. Added Aether tests for actual
Vibe transaction → canonical journal acknowledgement → UI mailbox → reopen.
Commands and final outcomes will be recorded after the current build completes.

Fresh working-source host verification completed: with the shared production-audit
Cargo cache and `CARGO_BUILD_JOBS=1`,
`cargo test --manifest-path packages/qualia-host/Cargo.toml --lib --offline`
passed **30/30 tests** (0 failures/ignores). The current canonical Vibe and core
library were rebuilt, taking 31m09s; tests ran in 2.96s. This includes signed Vibe
transaction → durable native receipt → owner-thread mailbox → all-lane reopen,
forged scopes/revocation, real VirtualDom task teardown/external cancellation, and
tiny workspace admission/reservation. It is separate from canonical dependency
unit tests: the new core graph/journal and Vibe budget tests still require their
own explicit Cargo invocations. No dependency unit-test pass is inferred here.

The subsequent live-tree `--all-targets` run compiled the headless example and
again passed those 30 tests, but the newly added bytecode regression failed at
parsing its untyped function parameter. The test now uses canonical `q: Quin`;
its corrected execution is pending the frozen qualification run. Do not count
that 31-test invocation as a pass.

## Point-in-time source capture

Concurrent work outside this packet changed canonical core bundle/stream and
QDNF source inputs after the successful live-tree build. The qualification source
is therefore frozen at
`target/qualification-sources/native-host-20261112-v2/source-receipt.json`.
The strict capture ran from 2026-11-12 10:47:41 to 10:53:04 UTC and contains 4,355
recorded files (65,161,356 bytes): the host and seven actual canonical packages.
All source hashes, file inventories and repository HEADs matched before/after
the successful capture pass. Its receipt SHA-256 is
`f745a72426994010d761edc830e0a86815d7282138b41bb1a5113de49342e3af`.

The capture preserves licensing, workspace metadata, literal compile inputs and
the Windows DXC build inputs. Original manifests are retained beside the derived
manifests; only workspace member selection and the host's actual Aether UI source
path are rewritten. No canonical API or implementation is replaced. Unchanged
source bytes are reused on retries, and partial captures do not receive receipts.
An independent full captured-hash pass succeeded in 22.84 seconds; hashing the
original source set took 25.55 seconds and showed subsequent live-tree changes.
These are point-in-time source records, not claims that a moving working tree
remains identical or that the capture is a clean commit.

The shared qualification target uses `CARGO_BUILD_JOBS=1`,
`CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0`; ordinary test/debug
assertions remain enabled. Canonical core, Vibe and host tests will be selected
explicitly through the generated frozen native+host consumer manifest so source
identities and build artifacts can be reused. Features for dependency packages
come from that verified consumer graph: Cargo rejects `--no-default-features`
when selecting a package outside the consumer workspace.

## Remaining boundaries

This does not authorize access to the daemon-global graph, arbitrary SPARQL, or
external stores. The new grants authorize an entire exact context; there is no
implicit field-level disclosure policy or delegation. Per-grant revocation uses
admission replacement or session revocation; independently mutable grant revocation
lists are not introduced. Native Vibe graph commits are native journal receipts,
not generic extension receipts. Parser/provider process isolation, real enrolled
operator inputs, physical native window qualification, cross-store transactions,
Windows power-loss directory durability, signing, packaging and releases remain
separate work.

The journal retains each last commit's exact signed grant, authenticated session
reference, authority generation and timestamp alongside its all-lane content
digest. Load/append verifies the retained issuer signature and receipt bindings;
this is host-owned evidence, not an external anti-rollback oracle. Corrupt canonical
ECC parity is rejected before storage and on reopen. The headless native_graph
example uses explicit adopted trust and a real one-use native signing challenge;
it contains no fixture identity, private key or auto-grant. The component hook
connects graph view lifetime to the worker's explicit cancellation token.

## Review follow-ups

The v2 capture's first combined consumer metadata check exposed Cargo selecting
the outer Aether workspace for nested canonical packages. The derived immutable
v3 layout binds each of the seven captured canonical package manifests explicitly
to `../../`, retains all seven original manifests and the v2 receipt, and changes
no implementation bytes. It was made only from verified v2 files, without reading
changed live source. Its 4,363-file receipt is
`target/qualification-sources/native-host-20261112-v3/source-receipt.json`, SHA-256
`613fd195b04fb3328365ea707531b02a6b77531a96a52d1fee72196f2e2a7346`.
The receipt records the derivation separately from the original capture interval.
Full file hashes, exact inventory and package identities passed verification.

Canonical host construction now reserves its two fixed quin buffers and 256 KiB
of bounded cold bookkeeping/term scratch using `vibe::Budget`, with fallible
allocation. The Aether helper gives the remaining budget to execution. Terms are
limited to 32 KiB and capability identifiers to 4096 bytes; query filters admit
only canonical leaves or explicit null/variable wildcards. A separate shared
Vibe-router admission charges the conservative query result cell bound before
host dispatch and keeps successful charges on failures. Tests cover tiny runtime
budgets, exact reservation remainder, repeated query exhaustion, host failure and
overflow without invoking the host. This does not claim global allocation
accounting: authority storage/journal and native UI/mailbox owners remain separate.

The native window example uses the real fallible native launcher and shared
signed-input startup. It binds UI graph/result delivery, a single worker and a real
100 ms timer. The main owner joins after event-loop return, while the component
only cancels. Its source is ready for the generated local integration profile;
compilation and actual signed-operator window execution are not yet claimed.

Transaction qualification is specifically for the canonical `vibe::Engine` AST
evaluator used by this adapter. It calls `graph_begin` and always aborts remaining
staging when leaving a transaction block; explicit `graph.commit` acknowledges a
write. The current canonical bytecode compiler emits a transaction as a plain
block without those lifecycle calls. NativeGraphHost rejects staging without an
active transaction, so bytecode transaction parity is a separate canonical engine
implementation/review packet. Shared query-budget router coverage does not imply
transaction parity or qualify bytecode as this application's execution path.

The separately named `native_graph_fixture` example is test-only. It creates a
disposable software-agent project using canonical admission/signatures/sessions
and a temporary native journal; it never supplies a fallback to production
startup. Its required order is native revision 0 → actual authorized Vibe commit
→ revision 1 snapshot plus result delivery → three ready redraw callbacks →
component teardown/cancellation → denied post-close write → fresh-session reopen
with all six lanes compared. A 30-second real-window timer bounds the event loop.
The main thread joins outside the event loop. Source review is complete; actual
compilation/run and GPU/pixel qualification are still separate evidence gates.

## Frozen test-workspace qualification setup

Cargo rejected testing the host as a dependency of the combined consumer because
it has dev dependencies and is not a workspace member. That command stopped
before compilation. Tests therefore use the actual captured host and canonical
workspace manifests, with the same reviewed dependency patch tables supplied
through Cargo configuration and one shared target directory.

The immutable v4 derivative retains v3, both original workspace lockfiles, and
the resolved test-root lockfiles. Its receipt is
`target/qualification-sources/native-host-20261112-v4/source-receipt.json`, SHA-256
`3bcee7dc35c4882ac02053f7efdf5f45de8024d5aea08a3d4cbf581d175c66b7`.
Strict verification passed for 4,382 files and nine package identities: the host,
seven canonical packages, and the canonical workspace's vendored `imap-proto`.
The latter is not selected in the combined 859-package consumer graph, but its
patch declaration and source must remain available when resolving the real
canonical test workspace. Its 16 files (77,963 bytes) were separately captured
with stable pre/post hashes on 2026-11-12 at 11:41:54.986–11:41:55.605 UTC, from
the same recorded QualiaDB HEAD. Existing captured implementation bytes were not
refreshed from the moving live tree.

Both actual test-root metadata commands passed offline under
`target/production-audit/frozen-v4-test-patches.toml`: 536 packages for the host
workspace and 558 for the seven-member canonical workspace, with
`qualia-core-db` features empty in both. Core test dependencies add no missing
canonical path packages; Vibe has no dev dependencies. Metadata and command
records are retained under `target/production-audit/frozen-v4-*`. These are
resolution/source checks, not executed unit tests. The next invocations use
`--locked`, debug symbols disabled, debug assertions retained, and jobs=1.

The reusable source capture now preserves canonical workspace path patches and
explicitly binds captured canonical workspace members, so these layout and patch
closure omissions do not recur. The requested Etalon naming inventory found no
legacy GPU alias references in the owned host package or the capture script.

Both additional actual test-root policy checks passed: the host is a real
captured workspace member with eight resolved captured packages; core is a real
member of the seven-package canonical workspace with seven resolved captured
packages. Both use 28 Etalon-family packages from the clean exact `6ace6cc`
checkout. The extracted consumer patch tables equal the test configuration.
All 33 owned live-source entries still match their captured source hashes, as
recorded separately in `resume-host-source-hashes.json`.

The locked frozen host test build then stopped during dependency compilation,
before canonical core compilation or any unit tests. Windows refused to execute
`target/qualification-build/native-host-20261112-v2/debug/build/getrandom-4802c0a7f34b0d0f/build-script-build`
for `getrandom 0.3.4`: **Application Control, OS error 4551**, "An Application
Control policy has blocked this file." Cargo exited 101; the helper was never
executed. The complete build log is
`target/production-audit/frozen-v4-host-tests.log`.

The exact locked command was submitted through the approved escalated execution
channel, then retried once as permitted. Automatic approval review timed out on
both attempts; neither escalated command executed. No elevated execution log is
claimed. No security setting was changed, and no alternative-launch workaround
was attempted. The corrected 31-test host run, canonical graph/journal and scoped
scan tests, Vibe budget unit tests, headless startup invocation, and native signed
fixture execution therefore remain pending. The earlier actual live-tree 30/30
host result remains valid only for its recorded inputs; it is not a substitute
for these unexecuted frozen-source gates.

Independent read-only Etalon naming review compared all 55 recovered vendor Rust
files with `a46b4e2` sources and inspected normalized sibling renderer/context
diffs. It found a missing crate-root compatibility re-export of the context
blocking helper, now corrected by the supervisor, plus two provenance wording
corrections. Vendor copyright/SPDX/license notice lines are unchanged. Two
undeclared sibling tracing feature gates also exist in the baseline and are not
naming regressions. No snapshot bytes were changed during this review.


## User pause checkpoint — 2026-11-13

The user requested a pause to charge the laptop. The pending initial protocol
file write settled; no further implementation was started. Observed files:

- `Cargo.toml`: present
- `.gitignore`: present
- `src/lib.rs`: present
- `src/types.rs`: present
- `src/codec.rs`: absent
- `src/state.rs`: absent
- `src/tests.rs`: absent

This standalone protocol draft is **NOT implemented or buildable**. Its lib.rs
references unwritten modules: codec, state, tests. No protocol tests were
attempted. No parent authorization/cancellation adapter or protocol dependency
has been added to qualia-host. The immutable v4 qualification capture is unchanged.

The saved types define the approved RouteStamp (private session, view, parent
generation, worker document counter, viewport counter), engine-derived CallerContext,
ControlEnvelope/Control, ReplyError/WorkerFailure and binary Frame data shapes.
Control JSON is capped at 128 KiB; binary RGBA pixels at 16 MiB. checked_frame_layout
admits dimensions and conservatively reserves four pixel-buffer sizes for readback
scratch, separately from engine/GPU memory. These DTOs are data, not authority.
PeerGuard/VerifiedMessage, Sequence, FrameMailbox and read_message/write_message
are planned exports only; their implementation is absent.

Resume ownership: this agent implements the neutral protocol codec, session/sequence
and direction guard, one-capture/one-frame mailbox, tests, then the parent-only
read-only canonical graph adapter. Root owns actual Chora worker/event-loop/native
frame code; dependencies owns worker profile/source gates. Agreed stamp behavior:
Navigate advances parent generation; actual URL change advances worker document;
Resize advances viewport and cancels old page requests and capture. Replies must
match the retained full stamp and request ID. Initial service is only the explicitly
registered qualia.graph.snapshot route; default denial, no scripts or mutations.
Canonical authority remains a real NativeGraphBridge in the parent. Required
sandbox admission must reject unsupported hosts; a separate process is not containment.

No owned process remains running. Frozen test process 87321 exited 101 before core;
review process 60546 completed; both escalated attempts never executed; pending
protocol write cell g17:230 settled. All frozen execution gates listed above remain
unexecuted. Continue only after the user resumes; see central PAUSE-CHECKPOINT.md.

## Resumed protocol and parent bridge — 2026-11-14

The standalone protocol is now implemented: versioned bounded private-pipe
control/binary-frame codec, local-role direction/session/sequence guard, terminal
guard failures, and one-outstanding-capture/one-latest-frame mailbox. JSON escaping
is bounded during serialization before writing any pipe bytes. Pixel admission
retains the 16 MiB limit and conservative four-buffer readback reservation.
Transport correlation still grants no canonical authority or OS containment.

Actual command, using the recorded cache and a dedicated protocol build directory:

```powershell
$env:CARGO_HOME='C:/github/Aether/target/production-audit/cargo-home'
$env:CARGO_TARGET_DIR='C:/github/Aether/target/chora-protocol'
$env:CARGO_BUILD_JOBS='1'
cargo test --manifest-path packages/chora-bridge-protocol/Cargo.toml --locked --offline
```

The final protocol run passed **7/7 tests**, zero failures/ignores, plus zero
doctests. Compiler: rustc 1.99.1 (58cf8f9c3 2026-10-13), Cargo 1.99.1, Windows MSVC.
An earlier six-test revision passed; the first seven-test relink hit LNK1104
opening the existing test executable. Repeating the same command, target and
executable succeeded normally. Incremental hard-link warnings fell back to file
copies. No OS policy, alternate launcher or security setting was changed.

`ChoraGraphRoutes` now requires explicit full-stamp/caller/profile registration
backed by a genuine `NativeGraphBridge`. The canonical provider SDK and Vibe
embedding exports were inspected before integration. No engine, SDK or extension
ABI is duplicated. The parent invokes canonical nonblocking `read_graph`, which
revalidates the signed exact-context scope and authenticated session on each read.
Only `qualia.graph.snapshot` with exactly `{}` as payload is admitted. Script,
mutation and unregistered routes deny. All six canonical lanes, context and
revision are encoded as decimal strings so web JavaScript cannot round u64 data.
Replies stop at 64 KiB; scratch is cleared after success or authority errors.

At most 64 real requests are retained, each with a 30-second deadline. Individual
cancellation removes one exact request. Navigation, resize and actual document
invalidation remove the old session/view route and pending work; replacement
stamps do not automatically register a new grant. Closed adapters reject work.

The optional `chora-client` feature supplies `pump_client` for the concrete owned
worker client. It handles at most 64 events per call on a host scheduling thread,
honors explicit route registration, and forwards cancellation/invalidation. Busy
canonical reads receive Busy replies. A full/failed outbound transport shuts down
the dedicated worker and clears its parent routes instead of silently dropping a
real request. This is overload cancellation, not process containment.

Stale reply delivery caused by a concurrent parent/worker lifecycle transition
discards only that old exact request and preserves the new owner. Worker lifecycle
acknowledgements remove previous stamps while retaining explicitly registered
routes and pending requests already matching the acknowledged full stamp. Public
`invalidate` remains the explicit pre-navigation/resize cancellation action.

Two real signed-authority fixture tests cover all-lane snapshots, forged origin,
private and full-stamp mismatch denial, mutation/script denial, session revocation,
64-request bounds, duplicates, cancellation, deadline expiry, viewport invalidation
and closure. A third test bounds serialized snapshot output. Their actual execution
is recorded below. The live host Cargo build began with
the recorded cache, jobs=1 and the root target, then was deliberately interrupted
to release the shared target for the supervisor's native Chora feature check.
It had compiled dependencies only; canonical core and host tests had not run.

The resumed root-target attempt also ended before tests. Inspection confirmed the
earlier actual live host cache is the standalone package's own target directory;
the 2026-11-12 test executable and its source/fingerprint records remain there.
The current full library invocation therefore uses that confirmed cache:

```powershell
$env:CARGO_HOME='C:/github/Aether/target/production-audit/cargo-home'
$env:CARGO_TARGET_DIR='C:/github/Aether/packages/qualia-host/target'
$env:CARGO_BUILD_JOBS='1'
cargo test --manifest-path packages/qualia-host/Cargo.toml --features chora-client --lib --offline -- --test-threads=1
```

DEV_DEBUG and TEST_DEBUG overrides are unset: this is the ordinary default debug
profile, matching the earlier live cache, not frozen qualification's debug=0
profile. The existing standalone and current default-debug serde fingerprints
share profile hash `11926785003450549200`. Actual Cargo reused dependencies and
rebuilt current Vibe, protocol and canonical core source. The new execution log is
`target/production-audit/resume-chora-live-host-tests-standalone.log`; the stopped
root-target log is `resume-chora-live-host-tests.log`.

The standalone command **exited 0 and passed all 34/34 host library tests** with
zero failures/ignores, on 2026-11-14 (final log timestamp 11:46:57 Australia/Sydney).
Cargo reported a 34m43s actual current-source build and 8.68s test execution,
ordinary unoptimized+debuginfo test profile, rustc 1.99.1 and Cargo 1.99.1, jobs=1.
This includes both signed Chora graph tests, same-stamp lifecycle acknowledgement
preservation, bounded output, and the corrected actual bytecode test proving that
staging without native transaction admission fails closed. It does not establish
bytecode transaction parity. Current canonical Vibe/core were rebuilt; their
existing warnings were retained. No security setting or OS execution workaround
was changed. The concrete-client pump was compiled through the selected feature.

This closes the current **live host library integration test** gate only. Canonical
dependency unit tests, immutable v4 qualification, real worker/authorized bridge
runtime, frozen native/client composition and production release gates remain
separate. No unchanged host test rerun is needed for the following Python source
generator changes.

The immutable v4 capture, recorded frozen failures and outstanding native/worker
runtime and release gates are unchanged. This implementation and protocol unit
execution do not establish native production readiness.

## Source capture and client-profile closure — 2026-11-14

The future capture now follows both declared authority transport paths, including
optional and aliased dependencies, into `packages/chora-bridge-protocol` and
`packages/chora-client`. It records their complete selected source inventories and
Cargo package identities. The verifier requires those manifests inside the captured
Aether tree, rejects registry/live-path substitutions and new unreviewed transport
path dependencies, and preserves the established explicit live `aether-core` UI
boundary. The capture tool also records its imported verifier/helper hash. Immutable
older receipts remain supported when their host declares no transport dependencies.

`prepare_local_integration.py --with-chora-client` requires `--with-host`, selects
host `chora-client` and native `chora` features, and remains distinct from the
existing `--with-chora` in-process diagnostic. Metadata verification checks actual
consumer/transport sources and selected features, and rejects linking package
`servo` into this client profile. Minimal `--with-host` behavior is unchanged.
Captured protocol/client standalone test-root selectors are available.

Capture claims are bounded to the client library/pump profile. The client's
`test-fixture` binary imports worker private-pipe/input helpers that this capture
does not contain; metadata selecting that feature is rejected. No worker engine
manifest/source closure is copied into the native capture.

Frozen native+client composition is explicitly **unsupported and rejected before
profile output**: live native UI selects its live client path while the captured
host selects captured client/protocol identities. There is no fallback to live
authority transport. A reviewed derived native manifest or frame-source decoupling
is a separate follow-up. An actual offline Cargo metadata fixture verified that
inactive optional native/client paths do not appear in the minimal frozen host's
resolved package graph, preserving the existing default composition.

Final policy command `python -m unittest discover -s scripts/tests -v` passed
**73/73 tests**, zero failures/errors/skips, in 22.662s. Log:
`target/production-audit/resume-chora-all-source-policy-tests.log`. This includes
real capture/copy/hash/receipt verification in a disposable test fixture, source
escape and package-identity rejection, feature/profile separation, and actual
offline Cargo path-identity resolution. The temporary fixture is not a new real
qualification capture. The updated verifier also completed a full read-only check
of immutable v4: all 4,382 recorded inputs and nine package identities remain valid.
Its receipt SHA256 remains
`3bcee7dc35c4882ac02053f7efdf5f45de8024d5aea08a3d4cbf581d175c66b7`.

An actual live profile generation attempt used:

```powershell
python scripts/prepare_local_integration.py --with-host --with-chora-client --etalon-source C:/github/etalon/target/aether-canonical-6ace6cc-object --output target/local-integration-host-client-20261114-v1
```

It failed **before writing any output** at the existing canonical patch review
gate: `Canonical patch table needs explicit integration review:
https://github.com/mediaprophet/etalon.git`. Current QualiaDB workspace overlays are
`etalon-model-io -> C:/github/etalon/crates/etalon-model-io` and
`etalon-model -> C:/github/qualiaDB/vendor/etalon-model-6ace6cc`.
The explicit clean profile instead selects both packages from
`C:/github/etalon/target/aether-canonical-6ace6cc-object/crates/` at revision
`6ace6cc6413a87ca10a1da58397ddc7c0b2fc41e`.

The live `etalon-model-io` Cargo manifest (inherits workspace version) matches the
selected manifest hash
`0c96ecaa77d0ec7e3b8c81f7c9450dca22c4e045bf02ab31b1c6b01c112fdecc`;
that does not establish source equality. The vendor `etalon-model` is version
0.0.2 with manifest hash
`20da06de29ef02285cbfa704cbb8f29f212293c9309ac5c9b05527caf91b0323`,
versus selected clean manifest hash
`2f2e6414a5d04372cedfffae56e0e66f6a81a7101b8926ac92f44918a7c6d351`.
The observed live Etalon repository HEAD was
`b0349cf35bcab6a9cdd0cb2156b0b1d9c55049a3`. No overlay, source identity or
workspace was rebased, replaced or silently admitted. The source capturer likewise
requires review of external canonical workspace patch paths; transport closure
fixes do not qualify those moving GPU overlays. This actual live-profile blocker
is separate from the passing current host tests and profile/policy regressions.
The independent host workspace uses its own Cargo lock/patch graph and does not
inherit QualiaDB's root workspace patch tables. Its successful current Vibe/core
rebuild does not qualify those newly declared model overlays.

Owned edits are protocol codec/state/mailbox/tests, the parent-only graph route
adapter and concrete-client pump, its canonical fixture tests and manifest/lock,
the three capture/profile scripts and relevant tests, and this packet. No owned
Cargo or test job remains active. Canonical dependency unit tests, frozen host
execution, real worker/authorized bridge runtime, OS containment and production
release gates remain outstanding.

## Exact reviewed model overlays — 2026-11-14

The later explicit source-review authorization is limited to two non-GPU packages:
`etalon-model-io` at sibling `etalon/crates/etalon-model-io` and `etalon-model` at
`qualiaDB/vendor/etalon-model-6ace6cc`. It does not admit the mutable Etalon
workspace or a different GPU/kernel revision. The new fixed policy file
`scripts/reviewed_etalon_overlays.json` has SHA256
`2a38fc5225d26d9078423aa8c8b169046ed2308a174f8ccc94bdab3b3a04d35c`.
Both capture and profile receipts record the helper/JSON/tool hashes.

The helper binds all 18 current raw model-IO files, including platform SHA256
`b088b1149dec0564f786da5cf55f5d245e1c7dc989b0694c73be3028fb20cf64`.
Seventeen other files have exactly reviewed CRLF materialization relative to raw
Git6ace objects; that comparison explains semantic scope and does **not** permit
alternative whitespace or line-ending bytes. The platform file is the one reviewed
semantic change. Independent source/hash review checked all 18 current/raw-Git
anchors, pinned workspace metadata and five license/notice anchors.

The vendor model is bound to the externally approved BACKPORT receipt SHA256
`a0a97e042cb1def73e701c02168c1ab31297fc4ffa3f1b6e1584b2bf85f8ea5b`.
All 113 original hashes are checked against immutable Git6ace objects; all 124
original/added current hashes plus that receipt form an exact 125-file inventory.
Only its known root `target-isolated` build directory is omitted. A future
self-authored receipt, extra input, changed file, wrong patch name/path or linked
source ancestor is rejected. The historical validation prose inside the vendor
receipt/provenance is retained as history, not re-executed or newly qualified here.

Model IO is staged/captured as a separate 25-file standalone package: exact raw
source, preserved `Cargo.toml.source`, immutable `PINNED-WORKSPACE.toml`, five
original notices/licenses, and a derived manifest. Package fields, registry
dependency versions and lint rules come only from immutable Git6ace workspace
metadata. The live mutable workspace is never inherited. Generated consumers
exclude that nested standalone staging root from automatic workspace membership.
The profile's only exceptions to clean6ace Etalon sources are these exact two
separately receipted packages; inference-kernel and every GPU-family package keep
the existing clean-pin source gate. Source and staging ancestry are checked before
writes and after staging. Capture keeps per-file original source mappings and
pre/post hashes, relocates only the reviewed model-IO patch into the captured
closure, and never updates an existing completed receipt.

Final command `python -m unittest discover -s scripts/tests -v` passed **86/86**,
zero failures/errors/skips, in 64.759s. Log:
`target/production-audit/reviewed-etalon-final-policy-tests.log`. This preserves the
73-test baseline and adds 13 focused policy regressions. Coverage includes exact
raw/EOL bounds, external receipt anchoring, extra input denial, source/staging
ancestor escapes before writes, unchanged notices/original manifests, GPU/kernel
exceptions remaining closed, and a small disposable capture with real reviewed
overlay bytes and minimal test-only SDK sources. No new large real qualification
capture was produced. The updated verifier also passed the full unchanged v4
inventory/hash check: 4,382 recorded inputs, nine package identities, no overlays;
receipt SHA256 remains
`3bcee7dc35c4882ac02053f7efdf5f45de8024d5aea08a3d4cbf581d175c66b7`.

Actual offline Cargo metadata and a locked repeat were run only for an isolated
two-patch test consumer. Its Etalon family resolved to model, standalone model IO,
and the single inference-kernel from the clean6ace checkout; the generated lock
retained those exact selections. No GPU packages were selected by that isolated
consumer. This is not acceptance of the actual full live SDK/native profile and
does not compile or execute either model implementation, numerical kernel or OS
lease behavior. No Rust, worker or native build was started in this follow-up.
The earlier full live host test result remains scoped to its own recorded graph.

The actual live preparation attempt at **2026-11-14T02:37:45.791632Z** exited 1
before output creation: `Only the exact two reviewed Etalon overlay names are
admitted`. The current canonical table had expanded concurrently to these 14
paths, which were preserved without rebasing or widening admission:

| Package | Path relative to QualiaDB |
| --- | --- |
| naga | ../etalon/naga |
| etalon | ../etalon/etalon |
| etalon-diagnostics | ../etalon/crates/etalon-diagnostics |
| etalon-agent | ../etalon/crates/etalon-agent |
| etalon-wgsl-forge | ../etalon/crates/etalon-wgsl-forge |
| etalon-nquin | ../etalon/crates/etalon-nquin |
| etalon-tensor | ../etalon/crates/etalon-tensor |
| etalon-render | ../etalon/crates/etalon-render |
| etalon-sim | ../etalon/crates/etalon-sim |
| etalon-signal | ../etalon/crates/etalon-signal |
| etalon-wasm | ../etalon/crates/etalon-wasm |
| etalon-resources | ../etalon/crates/etalon-resources |
| etalon-model-io | ../etalon/crates/etalon-model-io |
| etalon-model | vendor/etalon-model-6ace6cc |

The attempt log is
`target/production-audit/reviewed-etalon-live-profile-attempt.log`; destination
`target/local-integration-reviewed-overlay-20261114-v1` was absent afterwards.
This broader moving source graph remains a review/alignment blocker. Mixed frozen
native/client identities and captured client `test-fixture` remain rejected as
previously documented. These changes provide bounded policy/receipt checks and
isolated metadata evidence, not full source-graph, Rust integration or runtime
qualification. No owned build or test process remains active.
