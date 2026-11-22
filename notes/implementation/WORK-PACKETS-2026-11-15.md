# Aether continuation work packets — 15 November 2026

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

## Mandate and current state

This is an execution plan, not a record that these packets have run. It organizes
the [51-job handoff](HANDOFF-2026-11-14.md) and the 15 November streaming/EMF/QAC
review into bounded assignments with ownership, prerequisites and acceptance.

The user will notify us when the concurrent QualiaDB, Etalon and
hypermedia-production work has been fixed and committed. **Do not update pins,
adapt to moving sibling APIs, run integration builds against them, or repair
their owners' files before that notification.** Planning and independent Aether
preparation may proceed. Notification releases intake for the named repositories;
it does not establish that every format, platform or runtime gate has passed.
Do not require a redundant general approval after the notification: perform the
already-authorized intake and dependent work whose prerequisites are satisfied.

This document does not dispatch implementation agents. Initial packet states are
recorded in the table below. Preserve existing work, immutable captures and all
provenance. No commits, pushes, AI bylines, co-author trailers, branch creation or
renames. Other teams' committed fixes do not authorize this team to commit.

Updates belong in the [upstream integration register](UPSTREAM-INTEGRATION-REGISTER.md).
Use the [assignment and result template](WORK-PACKET-TEMPLATE.md) for each dispatched
slice. Historical evidence stays in the handoff and recovery packets.

## Roles, skills and cost policy

These are capability roles, not required plugins or invented installed skills.
Select the least expensive available agent that can reliably satisfy the role;
record the actual model/reasoning level in its assignment. No model-price estimate
or financial budget is implied.

| Role | Required competence | Cost/effort allocation | Must deliver |
| --- | --- | --- | --- |
| **SUP — architect/supervisor** | Cross-repository Rust architecture, authority/lifecycle boundaries, dependency identity, acceptance design | Strong systems agent; concentrated design/review passes, not continuous broad exploration | Concrete interface decisions, packet acceptance criteria, file claims, build schedule, integrated evidence and unresolved decisions |
| **REC — inventory/evidence agent** | Git read-only inspection, manifests, structured receipts, documentation and log interpretation | Low-cost general agent; no speculative architecture | Exact revisions/hashes, reproducible commands, faithful result summaries; escalate semantic uncertainty |
| **RUST — implementation agent** | Rust ownership, Cargo, bounded async/process code, targeted regression testing | Standard coding agent at moderate effort; raise effort only for a defined failure | Small reviewed implementation, meaningful tests and exact limitations |
| **BOUNDARY — senior systems specialist** | Authority, cryptography integration, parser admission, cancellation/backpressure, safe FFI | Strong coding agent/high reasoning for a narrow packet | Invariants, adversarial cases and implementation without weakened admission |
| **GPU — rendering specialist** | Etalon validation/API, resources, synchronization, surface/device lifecycle, image semantics | Strong specialist for design/debug; standard agent for routine matrix execution | Ownership contract, actual source coherence, hardware-specific results |
| **MEDIA — signal/audio specialist** | Scientific EMF/QAC interpretation, units/timebases, PCM/STFT, callback constraints | Strong specialist for contracts/deadlines; routine evidence collection delegated | Typed interpretation, explicit supported profile, honest display/playback evidence |
| **REVIEW — independent reviewer** | Competence matching the boundary under review; did not author that slice | Short, strong review at decision points; no permanent duplicate implementation team | Actionable findings, rechecked fixes, acceptance recommendation with limits |

Use at most four active agents including SUP. Usually SUP + two implementers +
one short reviewer is enough. A small documentation/inventory job does not need
an architect agent. Escalate when a concrete contract is unresolved, a state-machine
failure remains unexplained, or the same focused attempt fails twice; provide
the evidence to the stronger agent instead of restarting a broad investigation.

SUP defines the contract before implementation of WP03, WP04, WP09, WP11, WP12,
WP14 and WP15. A design pass must produce a reviewable table of inputs/outputs,
ownership, authority, limits, terminal states and refusal behavior. Prose saying
“integrate the SDK” or “support streaming” is not a sufficient assignment.

## Scheduling and ownership

- One file owner at a time. Before dispatch, SUP assigns exact files/new paths,
  base hashes and test commands. Table scopes below are candidate boundaries,
  not permission for overlapping writers.
- SUP alone integrates shared `Cargo.toml`/locks, root source policy and common
  module exports, unless explicitly delegated for one slice. Child agents propose
  shared-file patches to SUP; they do not independently edit them.
- Sibling source remains owned by the teams fixing it. A consumer finding returns
  to the register as an upstream action; it is not permission to patch that repo.
  A later explicit reassignment must name repository, files and active owner.
- One heavy Cargo process on this machine by default, and never concurrent Cargo
  writers to one target. Metadata/inventory work is lightweight but must not
  rewrite another owner's lockfile. SUP records target, profile and cache before
  dispatch. Do not create fresh targets or change flags just to retry a failure.
- A test run's owner follows it through completion. Record exit code, log and
  source identity; never infer success from warning counts or an idle process.
- Evidence collection can parallelize implementation in disjoint files. Review
  happens against a settled slice; source drift invalidates only affected claims.
- Use bounded fixtures; stop only owned processes. Do not disable OS protections,
  kill unrelated tools or start background polling of sibling work.
- Reserve interactive windows, GPU benchmarks and audio-device probes for one
  owner at a time. Concurrent hardware workloads invalidate timing comparisons.

## Packet sequence and initial states

“Prepare” allows an inventory, contract or fixture specification, not an unlisted
implementation/build. All execution below is prospective.

| Packet | Outcome | Owner | Prerequisites | Initial state |
| --- | --- | --- | --- | --- |
| WP00 | Architecture, assignments and acceptance ledger | SUP | This plan | Planning only |
| WP01 | Aether preservation, notice and harness preparation | REC; RUST only for bounded corrections | WP00 | Ready for independent preparation |
| WP02 | Accepted upstream commit/evidence intake | REC + SUP | User notification for relevant repos | Intake 1 verified; subsequent Qualia working-tree changes recorded |
| WP03 | Coherent reproducible dependency profiles/captures | RUST + SUP; REVIEW | WP02 | Live source gate and combined check passed before post-run drift; refresh capture for stable qualification; linkage/MSRV remain |
| WP04 | Canonical host/SDK and signed native data lifecycle | BOUNDARY | WP03 | Actual standalone host tests next, with isolated lock and fresh source receipt; signed runtime remains open |
| WP05 | Rechecked streaming contracts and upstream findings | BOUNDARY + REVIEW | WP02; WP03 for execution | Commit/current-diff review done; U01–U06 remain; current integrated runtime tests still needed |
| WP06 | Staged actual Chora worker and baseline runtime | RUST | WP01; accepted worker profile under WP03 | Stage, approved localhost denial/RGBA and supplemental three-case crypto probes passed; see scoped evidence |
| WP07 | Chora view/frame identity and native window integration | BOUNDARY + RUST | WP06; WP04 for signed bridge | Worker baseline available; exact paint identity/native window and signed bridge remain open |
| WP08 | Native input, assets and platform lifecycle | RUST, split by subsystem | WP01; WP03 for dependency-consuming tests | Prepare fixtures; execution held |
| WP09 | Aether canonical streaming consumer | BOUNDARY | WP03, WP04, WP05 | Complete-entry contract may proceed; canonical host regression and source qualification precede consumer execution |
| WP10 | Producer/format compatibility corpus | MEDIA + RUST | WP02, WP03, WP05 | Canonical reader contracts reviewed; create current producer corpus after source refresh; preserve U05 limitation |
| WP11 | Native EMF scientific presentation | MEDIA + GPU | WP09, WP10; relevant WP08 lifecycle | Waiting for consumer/admission |
| WP12 | Native QAC audio/observation path | MEDIA + RUST | WP09, WP10; relevant WP08 lifecycle | Waiting for consumer/admission |
| WP13 | Signed end-to-end native application slice | SUP integrates; RUST | WP04, WP07, WP09, WP11, WP12 | Waiting for constituent evidence |
| WP14 | GPU failure, interop and performance | GPU + REVIEW | WP03; WP11–13 for workload claims | Prepare workload/budget definition |
| WP15 | Browser compatibility and containment | BOUNDARY | WP03, WP06; generic engine owner availability | Trusted worker baseline available; containment, complete web/crypto suite and compatibility matrix remain open |
| WP16 | Distribution, CI and target qualification | RUST + REC | WP01, WP03; required WP08 and WP13–15 results for release acceptance | Prepare artifact/notice matrix |
| WP17 | Independent integrated acceptance and handoff | REVIEW + SUP | Each completed slice; final WP13–16 | Source-policy/client diagnostics reviews done; integrated/release acceptance remains open |

Main dependency chain:

```text
WP00 -> WP01 (independent preparation)
user notification -> WP02 -> WP03
WP03 -> WP04 / WP05 / WP06 / WP08
WP04 + WP05 -> WP09; WP05 -> WP10
WP06 -> WP07; WP06 -> WP15
WP09 + WP10 -> WP11 / WP12
WP04 + WP07 + WP09 + WP11 + WP12 -> WP13
WP03 -> WP14 design; WP13 -> WP14 integrated measurements
required WP08 + WP13 + WP14 + WP15 -> WP16 release acceptance -> WP17 final acceptance
WP17 performs narrower independent reviews throughout, not only at the end.
```

WP13 may produce an earlier signed graph/Chora milestone once WP04/WP07 pass;
that is not completion of its media-streaming milestone. Native scientific media
need not travel through the Chora compatibility process.

## Assignment specifications

### WP00 — supervision and architecture decisions

**Own:** this plan, register, shared acceptance records and integrated manifest changes.
**Deliver:** source/profile matrix; exact file claims; queue with target ownership;
decisions on native/captured package identity, stream ownership, scientific
interpretation, media clocks and GPU lifetime. Record alternatives only where they
explain a real tradeoff. Keep QualiaDB/Vibe required and Chora generic.
**Accept:** every dispatched slice has prerequisites, bounded writes, concrete tests
and evidence class. No circular dependency or task requiring another agent's live
file edits. **Review:** an independent specialist checks high-impact contracts.

### WP01 — independent Aether preparation

**Own:** new packet receipts/fixture specifications and existing staging harness
only if a concrete Aether-only correction is assigned; no sibling edits.
**Deliver:** preserved dirty-tree inventory; immutable-v4 and existing worker
artifact references; notice inventory resolving why root `LICENSE-QUALIA` is
absent; exact staging/probe recipes; input/asset/platform fixture matrix.
Do not invent license text. Locate the authoritative notice or record the missing
decision. No new dependency build is needed for these deliverables.
**Accept:** existing changes/artifacts intact, incomplete stage labeled incomplete,
missing prerequisites visible. **Next:** WP02/WP06 can reuse the inventory.

### WP02 — upstream update intake

**Own:** register entries and new receipt files, not upstream source or pins.
**Deliver:** full commit IDs supplied/verified after notification; dirty status;
changed APIs, locks/patches, features, format versions, provenance and owner-run
logs; disposition of every review item U01–U06 in the register. Determine which
profiles depend on which updates so unrelated packets need not wait unnecessarily.
**Accept:** each claim is tied to exact sources and a completed command/result.
Missing evidence remains explicit. A committed fix is not automatically a passing
test; a dirty sibling checkout is not automatically the committed tree.
**Review:** SUP accepts an intake revision and lists the packets it releases.

### WP03 — source alignment and reproducibility

**Own:** SUP-assigned Aether dependency manifests/locks and capture/source-policy
scripts; no overlapping edits with other packets.
**Deliver:** actual resolved GPU/ABI identities for root native, standalone host
and separate worker; coherent reviewed revisions; explicit compatibility overlays;
solution for captured-host/live-native client duplication. Generate a new capture
when necessary, preserving frozen v4. Qualify minimum Rust separately.
**Accept:** metadata/source gates and relevant policy tests pass, then one minimal
build for each intended profile. No mixed live/captured client, hidden upstream
wgpu implementation or silently widened mutable-path allowlist. Model/model-IO
overlay behavior needs its own runtime evidence, not only hash checks.
**Review:** independent source and dependency audit before downstream builds.

### WP04 — canonical host, SDK and durable signed graph

**Own:** assigned `packages/qualia-host` implementation/tests/integration files.
Canonical engine changes remain upstream actions unless separately reassigned.
**Deliver in slices:** (a) current canonical authority/budget/journal dependency
tests and host regressions; (b) signed native fixture; (c) provider lifecycle,
reload, cancellation, durable receipts, enrollment/revocation and restart.
Recheck bytecode transaction parity; preserve fail-closed admission if unsupported.
**Accept:** revision 0 -> authorized Vibe commit -> revision 1 owner-thread delivery
-> redraw -> teardown -> denied late write -> fresh-session reopen comparing all
six lanes. Test expiry, revocation, bounded buffers and failure paths. Production
startup uses real adopted authority, never the disposable test identity.
Source/evidence authoring requires an explicit write contract; snapshot reads do
not authorize it. **Review:** authority/lifecycle specialist.

### WP05 — streaming acceptance and upstream reconciliation

**Own:** Aether-side compatibility tests/specifications and register dispositions;
read upstream implementation without competing with its owners.
**Deliver:** recheck U01–U04 on committed sources; transport capability matrix for
native/HTTP/WSS, complete entry versus range, retry, cancellation and completion;
actual mixed multi-leaf regression evidence. Confirm whether a named restriction
is intentional and safely enforced or still an implementation gap.
**Accept:** malformed mandatory dependencies refuse; selected regions are honored
or explicitly unsupported; multi-leaf objects complete correctly; temporary
blocking and terminal failure have defined state cleanup; bounded buffering and
fairness survive a slow consumer. WSS framing-only results cannot count as TLS
or physical-network qualification. **Review:** independent state-machine audit.

### WP06 — actual worker staging and runtime

**Own:** assigned `packages/chora-worker` staging tools and
`packages/chora-client` probe/process-test files. Shared protocol edits require SUP.
**Deliver:** complete license/runtime stage selected from successful Cargo JSON,
exact executable/DLL hashes, and bounded real-engine loopback readback results.
Reuse the verified SpiderMonkey input only if it still matches the accepted profile.
**Accept:** real caller delivery, expected denial, exact 64x64 RGBA pixels and
bounded child teardown; supplemental crypto page result labeled separately from
unit/WPT coverage. Exercise crash, startup failure, stderr pressure and EOF cleanup.
No partial stage, fake transport peer or compile result substitutes for runtime.
**Review:** process/lifecycle reviewer; next WP07/WP15.

### WP07 — Chora frame identity and actual native composition

**Own:** assigned `chora-client`, host Chora adapter and native frame-widget files;
split sequentially if WP04 owns the adapter. Generic engine changes belong to a
separately coordinated Chora owner.
**Deliver:** exact document-to-paint identity for internal navigation, stale-reply
invalidation, resize/scale/crash behavior and a real native frame window. Add
explicit signed graph route registration after WP04, bounded diagnostics, and
define supported input forwarding. Preserve copied-pixel process separation.
**Accept:** delayed old paints/replies cannot acquire new document authority;
correct alpha/orientation; terminal image disappears; UI never joins child work;
authorized read succeeds while wrong origin/profile/context/revoked reads deny.
**Review:** independent lifecycle/authority review before integrated acceptance.

### WP08 — native platform behavior

**Own in separate slices:** (a) assets/resolver, (b) DOM/editor/input, (c) native
window/event lifecycle. Assign exact files because these touch shared native code.
**Deliver:** Unix descriptor/replacement and Android loader evidence; terminal
fetch/CSS completion; real IME, clipboard, pointer/drag/beforeinput/accessibility
behavior; first/auxiliary windows and suspend/resume/scale tests.
**Accept:** supported paths have bounded failure and no reachable unsupported
conversion panic; real OS behavior is distinguished from editor unit tests.
Run only available platforms and leave unavailable ones explicitly open.
**Review:** standard reviewer; senior specialist for handle/FFI containment changes.

### WP09 — Aether streaming consumer architecture and implementation

**Own:** a new Aether-side adapter area selected by SUP after intake; no copied
Qualia parser, transport engine or alternative Vibe ABI.
**Deliver first:** contract from canonical authorized source -> authenticated
record/range -> typed admission -> bounded owner-thread delivery. Specify memory
and queue limits, cancellation/revocation epochs, partial/complete/error states,
clock metadata and teardown. Existing graph snapshots remain snapshots, not deltas.
**Deliver next:** smallest actual adapter with slow-consumer, corruption, revocation,
cancel/restart and multi-leaf tests using canonical interfaces.
**Accept:** incomplete/opaque/invalid content never appears as a valid empty or
playable resource; publication cannot outlive its authority/view; no UI-blocking
I/O; measured high-water buffers stay within the declared budget.
**Review:** BOUNDARY reviewer before WP11/WP12 consume it.

### WP10 — producer and format interoperability corpus

**Own:** Aether-side corpus manifests/tests; producer or codec fixes go upstream.
**Deliver:** exact producer bytes/receipts from accepted Hypermedia and admission
through accepted Qualia readers; EMF v2/v3/v4 support matrix; current QAC scientific
and preservation profiles; P64/10D/Q42/M64 dependency fixtures as selected by WP05.
Reconcile U05 with a versioned correction; preserve historical frozen contracts.
**Accept:** valid round trips, malformed/truncated/digest-corrupt refusals,
unknown-critical semantics kept archival or rejected, exact units/timebases and
signed-zero preservation where required. Structural parsing is not scientific
validation or publisher authority. **Review:** format/media specialist.

### WP11 — native EMF presentation

**Own:** SUP-assigned Aether scientific presentation adapter and fixtures; generic
Etalon/Hypermedia shader fixes remain upstream work unless assigned explicitly.
**Deliver:** declared supported scientific profile, typed field/calibration/frame
binding and selected-region presentation with transparent missing samples.
Preserve spectral/measurement identity; color mapping is an explicit display
operation, not a replacement for field data.
**Accept:** actual native pixels compared to controlled admitted values; correct
region, orientation, scale and units; corrupt/missing/opaque data visibly distinct;
revocation/cancellation clears retained content. GPU budgets recorded.
**Review:** MEDIA semantic review and GPU resource-lifetime review.

### WP12 — QAC observations and real audio

**Own:** Aether consumer/audio bridge and fixtures; canonical QAC signal admission
and generic device-output code reused rather than duplicated.
**Deliver in separate milestones:** retained scientific PCM/STFT observations;
then actual playback only for an admitted supported signal. Define device ownership,
bounded queue, timebase, rate/channel handling, underrun/overrun, stop/revoke and
device-loss behavior. Do not equate archival or four-mode resources with audio.
**Accept:** exact known-signal comparison plus real device callback evidence;
per-block deadlines and underruns measured on named hardware, not a generous
aggregate wall-clock assertion. No blocking allocation/I/O in callback paths
where the contract prohibits it. Recheck U06. **Review:** audio systems specialist.

### WP13 — signed integrated native application

**Own:** a disposable integration example/fixture and integration receipts; no
concurrent edits to constituent libraries without returning work to their owner.
**Deliver:** producer -> authorized source -> verified transfer -> typed admission
-> native EMF display/QAC observation or admitted playback, alongside actual
Qualia graph/Vibe UI behavior. Chora compatibility runs as its own process.
**Accept:** meaningful multi-leaf mixed workload; slow network/consumer, authority
revocation, resize, restart and teardown; exact pixel/signal/data checks and bounded
resource peaks. Record graph/Chora-only and full-media milestones separately.
**Review:** independent integrated acceptance, not author self-certification.

### WP14 — renderer, GPU lifetime and workload performance

**Own:** assigned native renderer tests/integration code; no uncoordinated Etalon edits.
**Deliver:** intended renderer/feature matrix; device initialization/loss, surface
loss and recovery; named workload latency/memory/allocation measurements. Shared
GPU resources are a separately reviewed design after source/API alignment, not
a prerequisite for initial copied-frame correctness.
**Accept:** actual adapter/driver identified, failures reproducible, no stale GPU
resource ownership; declared budgets met. Refract microbenchmarks and canvas
compile checks do not qualify complete applications. **Review:** GPU specialist.

### WP15 — browser compatibility, networking and containment

**Own:** first the acceptance matrix; implementation files only after agreement
with the Chora owner. No ecosystem-specific dependencies in the generic engine.
**Deliver in bounded slices:** actual crypto units/WPT and affected WebGPU CTS;
external C/Rust embedding ownership; TLS roots/mTLS/resumption/profile/revocation;
private browsing; supported OS containment/adversarial tests. Source gate and
application integration are not substitutes for CTS.
**Accept:** unsupported sandbox/private profiles continue to reject before launch;
traditional web semantics preserved; containment proved on each claimed platform.
V8/Node-specific support is conditional on an identified product requirement,
recorded as supported or explicitly out of scope, not silently promised.
**Review:** security/compatibility specialist independent of implementation.

### WP16 — packaging, CI and installed target matrix

**Own:** assigned Aether workflow/packaging/probe scripts and distribution inventory.
**Deliver:** complete transitive notices, native libraries/assets/fonts, reproducible
build and source receipts, installed launch/update/recovery, signing workflow and
Windows/macOS/Linux target evidence. Add Android/iOS qualification before any
mobile readiness claim. Do not publish or sign with unavailable user credentials.
**Accept:** installed native application exercises required foundation and selected
media profile; clean-machine dependencies known; update failure preserves data;
CI runs intended checks without false success on skipped/blocked runtime.
Required native input/assets/lifecycle results from WP08 must pass for each
claimed target; an integrated demo does not replace that platform matrix.
Actual publication or other new external action remains outside this packet.

### WP17 — independent acceptance and final handoff

**Own:** review records only; corrections returned to the owning implementer.
**Deliver:** prioritized findings per completed slice, source-linked test evidence,
recheck of fixes and residual release matrix. SUP reconciles integrated results.
**Accept:** all declared deliverables accounted for as passed/failed/blocked or
explicitly deferred with reason; no canceled run counted as passed, stale-source
result reused, or process/component test promoted to hardware/production evidence.
Keep release incomplete until mandatory gates actually pass.

## Coverage of the previous 51 jobs

This maps responsibility, not completion. The detailed original acceptance text
remains in the handoff; a packet closing does not automatically close all mapped jobs.
All 51 original jobs initially remain **open for their outstanding acceptance**;
their earlier implementation/test evidence remains valid only for its recorded
profile. On dispatch, copy each covered ID into the packet receipt and record its
individual state/evidence. SUP alone updates the architecture release evidence.

| Handoff jobs | Owning packets |
| --- | --- |
| W01–W04 | WP01, WP06 |
| W05–W08, W10 | WP07; signed authority also WP04/WP13 |
| W09 | WP07/WP08 |
| S01–S04 | WP02/WP03 |
| S05–S07 | WP03/WP04; model/IO runtime under accepted upstream evidence |
| H01–H08 | WP04/WP13; bytecode or canonical engine changes returned upstream |
| N01–N03, N08 | WP08/WP14 |
| N04–N07 | WP08 |
| N09 | WP13 |
| C01–C09 | WP15; frame/caller identity also WP07 |
| R01, R03 | WP14 |
| R02 | WP09–WP13 |
| R04–R07 | WP16 |
| R08 | WP16/WP17 |

## Restart procedure after the user's update

1. Read the notification and applicable `AGENTS.md`; inspect current status without
   resetting or stashing work. Do not assume yesterday's hashes or findings remain.
2. Fill a new register intake revision. Verify the named commits, actual checkout,
   remaining dirt, owner evidence and changed public contracts.
3. SUP amends only affected packet contracts and records why. Recheck U01–U06;
   accept fixes, enforce documented limits or return unresolved work to its owner.
4. Release WP03 and only profiles with complete prerequisite intake. Agree on one
   Cargo owner/cache/profile; obtain a concrete coherent build before wider work.
5. Dispatch small slices using the template, then review and integrate evidence.
   If upstream changes again, mark dependent results stale and repeat the affected
   intake; do not erase previously valid historical results.


## Independent preparation completed — 15 November 2026

The user authorized independent preparation while sibling-dependent work stays
held. [Preparation results](preparation-2026-11-15/README.md) cover WP01, WP06
artifact recipes, WP08/WP16 fixture matrices and WP04/WP09/WP14/WP15 acceptance
requirements. Three gpt-6.1-sol agents at medium reasoning had disjoint document
ownership; the supervisor performed preservation/source checks and final review.
No Cargo, dependency integration, runtime or device execution occurred. No
upstream intake or implementation/release job is accepted by this preparation.

The user confirmed the shared current license is a temporary policy pending
future mediaprophet/license computational support. The supervisor added the
missing root LICENSE-QUALIA as an exact copy of existing LICENSE, preserving
terms and attribution. Stage execution remains held; the old partial stage
remains incomplete. WP02 still waits for the user's committed-fix notification.

## Intake 1 execution — 15 November 2026

The notification arrived; [the intake register](UPSTREAM-INTEGRATION-REGISTER.md)
records verified committed inputs and releases WP03 implementation and WP05
contract review. WP06's separate worker build/stage, approved localhost denial
and 64x64 RGBA probe, and three supplemental page-window crypto cases now pass.
Bounded client diagnostics pass 13 unit and 3 process tests. Native composition,
authorized host operation and release qualification remain separate jobs.
WP04 proceeds after the actual live-source metadata gate. The broad wait above
is historical; follow the register's specific current constraints and evidence.

The combined native/host `cargo check --lib --locked --offline` subsequently
passed (37m09s). Its post-run gate detected 28 changed QualiaDB inputs, so this is
a successful moving-tree compile, not stable-source or release qualification.
Refresh the source receipt before the next gated execution. The register contains
the actual standalone-host test route using Cargo's isolated lockfile support;
source manifests/locks, frozen v4 and the independent worker profile are preserved.

The live standalone host test-root source gate is now implemented and reviewed:
43 focused policy tests passed, one Windows symlink-privilege test skipped. This
prepares WP04's actual test execution without changing its source lockfile. It
does not count as execution of the canonical host tests. Fresh receipt capture is
required after ongoing Qualia fixes and these Aether policy/document updates.
