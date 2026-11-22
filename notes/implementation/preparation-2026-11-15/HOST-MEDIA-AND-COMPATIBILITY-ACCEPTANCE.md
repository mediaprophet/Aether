# Host, media and compatibility acceptance preparation

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

Status: requirements and fixture specifications only, 15 November 2026. This
document does **not** accept WP04, WP09, WP14 or WP15 implementation. No sibling
source inspection, build, test, device probe or network operation was performed
for this preparation. Existing source observations below are not fresh passes.
Repository licensing and inherited attribution remain applicable.

Read with the [work packets](../WORK-PACKETS-2026-11-15.md),
[upstream register](../UPSTREAM-INTEGRATION-REGISTER.md),
[handoff](../HANDOFF-2026-11-14.md) and
[native release gates](../../architecture/13-NATIVE-PRODUCTION.md).
WP02 must accept named committed inputs and WP03 must qualify coherent profiles
before dependency-consuming execution. U01–U06 need new dispositions; this document
does not presume their historical findings still apply.

## Contract worksheet and ownership

The following is an acceptance vocabulary, not a proposed Rust API, wire schema,
extension ABI or replacement for the canonical SDK. Bind it to accepted canonical
interfaces after intake. Native works require QualiaDB/VibeScript; Chora remains a
generic compatibility engine in its own process/workspace.

| Boundary | Inputs and outputs | Owner / authority | Limits and terminal refusal |
| --- | --- | --- | --- |
| Signed host session | Adopted project authority, authenticated session, signed exact-context grant -> authorized graph operation/receipt | Application worker owns execution; canonical authority owns grant validation and journal; a capability name or source request grants nothing | Scope capacity, canonical budgets, expiry/revocation; malformed, stale, wrong-context or missing authority denies |
| Native delivery | Canonical value/receipt or complete graph snapshot -> component state/redraw | Native owner thread; worker only schedules wakeups; component owns receiver/view/token | Bounded queue and snapshot storage; full/closed/cancelled remain explicit; no worker join or journal I/O in event/render callbacks |
| Streaming admission | Authorized source, authenticated object identity, requested entry/region, typed dependencies -> admitted complete object or explicitly supported partial region | Canonical transport/readers perform verification; Aether owns consumer/view lifecycle, not parser semantics | Declare bytes, decoded storage, queues, in-flight work and retry bounds before run; incomplete, unsupported, corrupt and denied are distinct outcomes |
| Scientific presentation | Admitted typed field/signal plus units, calibration and clocks -> native observation/display; playback only after separate signal admission | Media owner retains measurement meaning; GPU/audio owners retain resource/device lifetime | Unsupported interpretation stays archival or refuses presentation; no empty-valid/playable fallback |
| Browser bridge | Engine-derived caller and current document route plus separately registered host grant -> bounded reply/copied frame | Parent owns private transport and host authorization; engine caller identity is not itself a grant | Wrong route/profile/origin/context, unsupported sandbox/private mode, oversize and stale replies deny; browser crash cannot retain privileged execution |

Current Aether boundaries are useful anchors, not performance budgets:

- [ExecutionLimits](../../../packages/qualia-host/src/runtime.rs) defaults to 64 KiB
  source, 10,000 interpreter steps, canonical `SENTINEL_BYTES` workspace, 64 KiB
  result Debug representation, 4096 extension output bytes and 64 receipts.
  These are cold-path/interpreter limits, not total heap or elapsed-time quotas.
- [The mailbox](../../../packages/qualia-host/src/mailbox.rs) takes caller-selected
  nonzero capacity, invalidates epochs and discards queued values on owner drop.
  [Graph views](../../../packages/qualia-host/src/graph.rs) allocate scope-sized
  scratch and coalesce complete snapshots with skipped-revision counts. They are
  not a delta/event stream. Busy is retryable; terminal authority errors require
  clearing displayed private data.
- [The existing private-pipe protocol](../../../packages/chora-bridge-protocol/src/types.rs)
  caps control messages at 128 KiB, frames at 16 MiB, payloads at 16 KiB,
  replies at 64 KiB and pending requests at 64, with a 30-second maximum deadline.
  These bounds belong to this compatibility transport; they do not set canonical
  media chunk sizes or establish a sandbox or a real-time guarantee.

## WP04: signed host lifecycle scenarios

Use [native_graph_fixture](../../../packages/qualia-host/integration/native_graph_fixture.rs)
and its [disposable support](../../../packages/qualia-host/integration/fixture_support.rs)
as a test anchor. Its source stages one triple inside a Vibe transaction, commits,
then queries eight rows. Do not turn its fixed test seeds into production trust.
[native_graph_window](../../../packages/qualia-host/integration/native_graph_window.rs)
requires real adopted inputs/external signing and is the separate production-startup
path. Current tests are anchors, not evidence of the future accepted profile:
[host regressions](../../../packages/qualia-host/src/authoring_tests.rs),
[runtime/provider regressions](../../../packages/qualia-host/src/tests.rs) and
[mailbox regressions](../../../packages/qualia-host/src/mailbox_tests.rs).

| ID / stimulus | Observable acceptance | Evidence required |
| --- | --- | --- |
| H1: signed empty revision 0 -> authorized source transaction -> revision 1 | Actual native owner observes revision 0, one committed row, result delivery and at least the fixture's three ready redraw callbacks; teardown cancels; late write denies; fresh authenticated session reopens revision 1 | Receipt and journal revision agree; compare subject, predicate, object, context, metadata and parity across journal/read/result, not rendered strings alone; event-loop and owner-drop trace |
| H2: wrong project/context/session, forged or expired grant; revoked enrolled identity | No effect, new receipt claiming success or private publication; displayed retained content clears on terminal authority result; other independent authorized sessions remain correctly scoped | One negative case per authority field; before/after graph/journal and route state; test revocation both with no data updates and with a pending reply |
| H3: reload/navigation while prepared, executing and delivery-pending | Old prepared program cannot execute in new runtime/epoch; old deliveries cannot update new component; fresh component/token may run under new authorized session | Controlled barriers at each stage, owner thread IDs, old/new epoch trace, drops and bounded retained bytes; pending tick cancellation latency measured because token alone does not wake the current graph hook |
| H4: cancellation before effect, during provider/host call, and during journal append | Before admission denies; active call obeys its actual cooperative boundary; no later UI publication. A commit already underway may finish and must be reconciled, retained and visible on reopen | Separate effect outcome from UI outcome; receipt persisted despite closed UI; no unsupported promise of preemption or rollback; bounded completion or explicitly unresolved isolation requirement |
| H5: full receiver, source/result/workspace/step/receipt exhaustion, failed receipt acknowledgement | No success fabricated from truncation; no execution when queue admission refuses; committed effects reconciled on later delivery failure; caller can distinguish capacity, cancellation and execution errors | Boundary and boundary+1 cases for configured limits, peak allocations/queue count, canonical trap and journal observations; simulated provider gate tests labeled simulated |
| H6: provider register/use/reload/drop; denied ticket; provider fails mid-call | Canonical descriptor, borrowing, caller-owned output, resource and lifecycle contracts reused; no use after provider teardown or authority retention across reload | Accepted SDK entry points and lifecycle trace; output bounds/canaries or canonical ownership tests; actual provider evidence separate from mocks/discovery |
| H7: journal append failure, truncated/tampered journal, restart and cross-store grant use | No false durable acknowledgement; canonical recovery/refusal behavior explicit; another store cannot inherit authority; fresh-session reopen agrees with acknowledged effects | Failure injection around append/ack boundaries, error plus persisted bytes/revision, canonical tamper/recovery suite; controlled crash is distinct from actual power-loss qualification |
| H8: source versus bytecode transaction | Supported paths preserve admission/commit/rollback semantics; unsupported bytecode transaction stays fail closed | Recheck `actual_bytecode_cannot_stage_without_native_transaction_admission`; no claim of parity from source-only success |

Source/evidence authoring needs its own persisted write authority; snapshot read
registration does not authorize authoring. Enrollment/revocation deployment and
durable extension receipt acknowledgement need explicit accepted-canonical tests,
not only the disposable graph fixture.

## WP09: streaming states and corpus

For acceptance, distinguish waiting for authority, transferring, retryable
blocked, verified-but-awaiting-dependencies, admitted partial (only when supported),
admitted complete, cancelled, denied, corrupt, unsupported and failed. These labels
do not require new canonical state variants. Map actual upstream outcomes without
collapsing EOF, temporary blocking or a completed leaf into object completion.
Each attempt needs an object identity, exact selection, authority/view epoch and
terminal accounting; exact canonical identifiers are intake-dependent.

Construct deterministic producer-backed fixtures after WP05/WP10 settle supported
formats. Keep a manifest of bytes/digests, dependency relationships, selected
region, decoded reference values and expected outcome. Proposed **test parameters**:
two mixed entries with at least three leaves each, unequal final-leaf lengths,
distinct sentinel values per leaf, one small independent entry, and one shared
mandatory dependency. Increase beyond one window/in-flight capacity as declared
by the accepted scheduler. These counts are test coverage, not production limits.

| ID / injection | Required outcome and measurement |
| --- | --- |
| S1: native/HTTP/WSS complete-entry transfers, each leaf arriving separately/out of order as supported | Every required byte/leaf and dependency verified before whole-object completion; reference digest and decoded values match. Emit no early success after leaf 1. Duplicate delivery does not duplicate effects/completion; unsupported ordering is refused explicitly |
| S2: nonzero selected region crossing a leaf boundary; aligned/unaligned edge and empty/out-of-bounds selection | Exact selected coordinates/bytes or declared unsupported error; never silently serve `[0,total)`. Record requested/served ranges and read/output bytes; no unrelated region exposed |
| S3: missing, malformed, truncated, wrong-family or digest-corrupt mandatory P64/10D/Q42/M64 dependency where supported | Cannot satisfy dependency presence through family tag alone; dependent presentation remains withheld/refused. Optional missing samples differ from a missing mandatory calibration/geometry dependency; record canonical admission error |
| S4: WouldBlock before issue, header, body, acknowledgement and completion; recover then repeat | Retained progress resumes without leaked issued slots/reservations or authority, or explicit terminal abort releases them. Exact bytes and one terminal completion; retry counts/slot occupancy and cleanup recorded. Reconcile U02 rather than invent retries atop ambiguous upstream state |
| S5: slow/paused consumer while two large entries and the small entry compete | Named finite queue/in-flight/byte budgets hold, no UI I/O blocking; supported scheduler fairness evidenced by per-entry progress. No silent data loss; any coalescing requires complete replaceable-snapshot semantics, never arbitrary loss of media chunks |
| S6: cancel/revoke/expire during dependency wait, transfer, admission and queued presentation | No subsequent publication under retired authority/view; release decoded/raw buffers, reservations and resources; late replies dropped. If source transport cannot cancel promptly, consumer still denies delivery and accounts bounded retained work |
| S7: EOF/disconnect after header or penultimate leaf; timeout; corrupt final leaf | Partial state never becomes empty-valid/complete/playable. Defined terminal cleanup/retry policy; fresh authorized restart cannot inherit old completion or unauthenticated cache bytes |
| S8: owner/component/window drop, navigation, restart; independent second view survives | Retired ownership loses publication and resources; second view remains functional. Redraw/event loop stays responsive; memory returns to declared retained-cache baseline with explicit cache authority rules |

For each row, record supported/refused/blocked separately for native, HTTP and WSS,
complete entry and selected region. WSS loopback framing does not establish TLS,
remote network behavior or end-to-end progressive decoding. Whole-entry admission
may be a valid supported restriction if enforced and labeled; it cannot be called
progressive range presentation. No adapter implementation/API is selected here.

## Scientific EMF and QAC acceptance distinctions

WP11/WP12 consume WP09 only after supported contracts and WP10 corpus acceptance.
Structural validity, authenticated transfer, publisher authority, scientific meaning,
displayability and playability are separate assertions.

| Resource / proposed fixture | Required interpretation and observable result |
| --- | --- |
| EMF versioned field: asymmetric known grid, distinct row/column values, valid zero, missing sample and nonzero selected region | Bind frame/axes, units, calibration and measurement/spectral identity from admitted data; orientation/region reference matches actual native pixels; missing is visibly distinct from zero. Record normalization/color map/transfer function separately from source values; no spectral field silently treated as RGB |
| EMF unknown-critical semantics, unsupported version or missing mandatory calibration | Archival-only or explicit refusal according to accepted contract; no plausible scientific display. EMF v2/v3/v4 support needs per-version producer/reader evidence; structural parse alone is insufficient |
| QAC scientific PCM/STFT observation: canonical known impulse/tone/ramp with explicit sample clock, rate, channels, units and STFT parameters | Reference sample/block and spectral comparisons with declared numerical tolerance, channel/time ordering and signed-zero preservation where required; observation is not device playback. Retain spectral identity and window/hop/timebase metadata |
| QAC preservation/archival or four-mode object, opaque semantics, unsupported layout | Do not infer audio from file family or successful decode; retain archival identity or refuse playback. Reconcile versioned alias correction U05 without rewriting historical frozen schema |
| Admitted QAC playable signal -> device callback | Separate milestone: explicitly supported sample format/rate/channel mapping and clock conversion, finite queue, owner/device lifecycle, stop/revoke/device-loss behavior. Compare rendered signal with reference; actual callbacks, per-block deadlines, underruns/overruns and discontinuities measured on named device |

For playback, compute each block's available period from actual frames/sample
rate and measure callback execution/jitter against that period, plus end-to-end
queue latency and clock drift. Define which callback operations forbid allocation,
locks or I/O and instrument them. A generous aggregate ten-block wall-clock gate
cannot establish real-time output (U06). Seek/cancel/revoke must remove queued old
audio at the documented device boundary; already emitted sound cannot be erased.
Do not assert zero latency revocation, inaudibility or physical capture without
device/loopback evidence and a declared clock/error model.

## WP14: GPU failure and measurement plan

Select and record default hybrid, Vello GPU, CPU alternative and intended combined
features/facade separately. Identify resolved Etalon packages/sources, Hypermedia
adapters, toolchain, OS, GPU/adapter, driver, power/display/scale configuration,
binary hash and accepted source receipt for each run. An unavailable profile is
open, not passed. Use one hardware workload owner to avoid contention.

| Workload / failure | Measurement and correctness oracle |
| --- | --- |
| Static known native UI; changing signed graph; copied Chora frame | Frame time and event-to-visible latency distributions/tails; reference pixels, alpha/orientation and owner-thread responsiveness; compare like-for-like profiles, not WebView or Refract microbenchmarks |
| EMF selected-region updates plus mixed multi-leaf transfer; admitted QAC observation/playback alongside redraw | Bytes decoded/uploaded, CPU/GPU memory peaks, queue depth, frames missed, per-stage latency and named hot-path allocations; exact field/signal oracles still pass while slow-consumer injection is active |
| First/auxiliary window, resize/scale, close ordering, suspend/resume | Resource ownership and counts across surface replacement, no stale frame from retired generation, bounded recovery and retained content clearing on terminal failure; reuse native lifecycle examples as anchors |
| GPU initialization failure, surface loss, device loss, allocation/upload/map/submit failure where injectable | Reproducible injection point and error; bounded refusal/recovery according to selected renderer, no stale resource handle or cross-device use, UI failure diagnostic; explicitly measure time/resources after recovery |
| Custom resource with UI renderer; future shared resource design | First establish correctness with independent/copied resources. Shared ownership separately requires accepted compatible source/API identity, device owner, fences/synchronization, release and device-loss rules; copied frames cannot prove zero-copy interoperability |

Existing anchors:
[native_lifecycle_smoke](../../../packages/native/examples/native_lifecycle_smoke.rs),
[native_multiwindow_smoke](../../../packages/native/examples/native_multiwindow_smoke.rs)
and the signed fixture above. Their timers/redraw counts are watchdog/lifecycle
parameters, not frame budgets or pixel evidence.

Before executing, fill workload dimensions/rates/duration, warm-up and repeat
policy, acceptable latency quantiles, total/retained memory and allocation limits,
recovery bound and rationale for target hardware. Separate startup from steady
state; identify instrumentation overhead and GPU timing availability. Record raw
samples and failures/skips. Do not pick universal production targets here or
declare passing because a measured value exists; budgets require an agreed
workload and acceptance decision, and later workload changes require remeasurement.

## WP15: browser compatibility and containment matrix

Run against the actual staged Chora engine, not only transport peers. Generic
engine work remains with its owner. Preserve traditional JavaScript/browser
behavior and keep privileged ecosystem operations behind parent authorization.

| Slice | Positive and adversarial scenario | Acceptance evidence / claim limit |
| --- | --- | --- |
| Traditional pages, navigation and mixed native surface | JS/DOM and representative required app operations; top-level/subframe navigation with delayed old paint/reply; resize/focus/keyboard/pointer/scroll/IME as supported | Actual document-to-paint identity, dimensions, alpha/orientation, stale suppression and native interaction. Unsupported input explicit; SpiderMonkey web support does not promise V8/Node/native extensions |
| Parent signed bridge | Actual engine caller authorized read; spoofed origin/profile/pipeline/view/context, revoked/expired session, wrong route/capture and page-supplied marker strings | Real adopted grant plus signed graph result; all spoof/stale cases deny without effect; no page-originated authority minting. Navigation retires requests/frames and crash clears retained image |
| WebCrypto / scoped provider | Actual unit regressions and official WPT in window and worker globals; unscoped/scoped handle misuse, provider denial/revocation and profile separation | Completed harness result per target; supplemental page wrapper and source policy remain narrower evidence; opaque handles must not become serializable authority |
| WebGPU / Etalon | Affected official CTS; mapped-range failure/unmap, cancellation, device loss and delivery failure; optional canvas/profiler paths | Traditional API outcomes and resource cleanup, actual hardware/runtime receipt; compile/source gate is not CTS or full application qualification |
| Rust/C embedding | Actual caller identity; fallible startup/TLS setup, allocation/free ownership, invalid parameters, partial startup and repeated teardown | Real external consumers and ABI ownership traces; no double free/use after teardown or implicit process-wide authority |
| TLS and network profiles | Roots/host exceptions, root reload for future HTTP/WSS connections, mTLS identity retention, session resumption and profile-specific pools | Actual handshake/network traces and rejection controls; document existing-connection/pool cutover separately. Immediate revocation stays Unsupported until an implemented mechanism is demonstrated |
| Private browsing | If unsupported, prelaunch refusal; if implemented, profile-isolated storage, cache/network/keystore and teardown | No private-to-normal or cross-profile residue/access in actual engine. A flag, child process or private pipe alone supplies no evidence |
| Required sandbox | Unsupported target/profile rejects before launch; supported profile attempts filesystem/network/process/resource escape and crash/restart | Per-OS actual restrictions and adversarial results, builder/CLI/C startup refusal tests; watchdog/timeout is not containment. TrustedEmbedding cannot silently satisfy RequireSandbox |
| Pressure and termination | Oversize frame/reply, pending-request flood, stderr flood, startup failure, EOF, hard crash and timeout | Finite retained diagnostics/queues, no secrets in logs, owner process reaped and native UI responsive; independent views remain functional |

Repeat containment/platform claims on every claimed OS; Windows evidence cannot
qualify Linux/macOS. [Client state tests](../../../packages/chora-client/src/tests.rs),
[process helpers](../../../packages/chora-client/tests/process.rs) and
[protocol tests](../../../packages/chora-bridge-protocol/src/tests.rs) are useful
boundary anchors. Fake-peer process passes cannot establish actual engine
semantics, sandboxing or private storage. Complete staging/notices under WP06
is a prerequisite, not optional cleanup after a successful browser probe.

## Intake-dependent unknowns and evidence receipt

Fill these only after accepted upstream commits; do not freeze moving sibling APIs:

1. Canonical engine/provider/authority entry points, transaction parity, durable
   extension acknowledgement, enrollment/revocation and cooperative-call bounds.
2. Exact format versions/profiles, corrected QAC schema alias, scientific metadata,
   typed dependency admission and producer/reference bytes (U01/U05).
3. Native/HTTP/WSS complete/range/progressive support, multi-leaf continuation,
   WouldBlock ownership/retry and cancellation/completion semantics (U02–U04).
4. Consumer storage/in-flight/decoded-resource ownership, memory accounting and
   cancellation clocks; audio device bridge, permitted callback operations and
   supported signal conversions (U06).
5. Coherent Etalon identity/API and intended renderer/device matrix; supported
   fault injection and resource recovery; external resource-sharing contract.
6. Actual Chora stage, engine/document paint identity, TLS cutover/revocation
   limitations, private/sandbox platform support, WPT/CTS harness availability and
   any concrete product requirement for V8/Node APIs.

For every executed scenario retain: scenario ID; accepted sources/dirty-input
hashes and lock/features; fixture manifest/digests and expected payload; authority
scope without secrets; configured limits/workload; exact command/environment and
completed exit/timeout; raw logs/measurements; observed values/resources/terminal
states; pass/fail/blocked/unsupported with reason; evidence class (unit, fake-peer,
actual engine, native pixels, device, installed); reviewer finding/disposition.
No skipped/terminated run passes. Failed injection runs still need cleanup
evidence. Independent review and release-ledger updates belong to their assigned
owners; this preparation edits neither acceptance status nor shared records.
