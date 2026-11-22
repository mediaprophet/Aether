# Native overhaul implementation tracker

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

This is the live queue for the approved Aether/Chora overhaul. Implemented,
compiled, behavior-tested and release-qualified are distinct states. Source work
continues when a missing build input prevents a broader check; no failed check is
silently converted into success. Existing user changes are preserved.

## Continuation planning — 15 November 2026

[Work packets](../implementation/WORK-PACKETS-2026-11-15.md) now specify bounded
agent roles, cost allocation, prerequisites, file/build ownership and acceptance
for all 51 outstanding handoff jobs plus explicit streaming/EMF/QAC integration
checks. The user will report when concurrent QualiaDB, Etalon and Hypermedia fixes
are committed. Dependent execution waits for that notification and
[source/evidence intake](../implementation/UPSTREAM-INTEGRATION-REGISTER.md).
No implementation agent or integration build is dispatched by this plan.

## Active resumption — 14 November 2026

The user authorized resuming [the saved checkpoint](../implementation/PAUSE-CHECKPOINT.md).
The separate-worker implementation and current results are in
[the worker packet](../implementation/resume-worker.md). Protocol/client tests and
the native CPU integration check pass. Current live canonical-host integration
passes 34 tests with the concrete client pump; the actual worker check and full
executable build pass. Actual page/pixel runtime remains unexecuted. Read
[the consolidated handoff](../implementation/HANDOFF-2026-11-14.md) for final
staging evidence and the comprehensive remaining-job inventory. No production
completion is claimed.

## Local recovery — 12 November 2026

Resumed from the saved checkpoint after the previous chat ended in a system error.
Existing uncommitted work and the separate Chora candidate are retained. Three
agents now own dependency/build recovery, canonical host/data integration, and
Chora integration respectively; the supervisor owns native assets, integration
review and this tracker. The sibling Chora checkout is writable in this session.
Earlier statements about read-only access describe the previous session only.

Current work: qualify the canonical host against a verified frozen source set,
complete the actual Chora engine and optional canvas builds, and close remaining
native/browser runtime gates. The new IME surrounding-delete implementation passes
its actual editor and DOM tests. Each packet retains executable evidence and
distinguishes source changes from shipping builds and runtime qualification.

Etalon naming correction is now part of this work. Etalon is the required GPU and
compute implementation, with substantial independent development and work moved
from QualiaDB. Current implementation names are being migrated across Aether,
Chora and the maintained Hypermedia renderers. Historical upstream attribution,
standard WebGPU interfaces and narrowly required compatibility exports remain.
The build and runtime evidence below predates this naming change unless explicitly
marked otherwise; it must not be presented as verification of the changed tree.
Post-naming Windows metadata resolves 1,175 packages with no `wgpu` package
identities. The Etalon family remains at the reviewed `575bab54` revision.
Independent minimal hybrid and Vello consumers retain exactly the same enabled
Etalon features after the direct native dependency was corrected to disable
defaults. Source/snapshot policy tests pass 54/54; all 17 wrapper configurations
compile. Windows Application Control prevented completion of the wrapper runtime
matrix, so compilation does not establish that matrix's runtime results.

The frozen v4 canonical host build is currently blocked before core compilation:
Windows Application Control refused the generated `getrandom` 0.3.4 build script
(error 4551). Two execution approval reviews timed out before dispatch. No frozen
host, canonical dependency, or signed native fixture tests have run successfully;
the earlier 30 passing host tests apply only to their recorded source checkpoint.

Recovery evidence so far:

- Actual root locked Windows native DOM, default native renderer and patched CPU
  renderer checks passed after authorized network retrieval restored dependencies.
  The no-SVG CPU and repaired Vello GPU profiles also pass. Actual CPU single-window,
  late-auxiliary-window and default hybrid renderer probes now run successfully;
  combined default+CPU, public native+refract facade and optional profiler checks
  also pass. The Vello lifecycle executable builds, but Windows Application Control
  blocked launch and execution approval review timed out twice before dispatch.
- The reviewed 62-file Chora series is applied in the actual sibling checkout;
  Cargo regenerated its root lock and the GPU source inventory now passes.
- An explicit local integration consumer resolves the canonical Etalon revision
  with sibling hypermedia adapters. This is source-identity evidence, not GPU
  resource-sharing or release qualification; remote shipping pins are unchanged.
- Actual resolver tests pass with new byte limits: 7 policy tests and 2 public API
  tests, plus 3 document tests. The explicit Windows junction swap test also passes.
  The final actual resolver run passes 8 library tests and 2 API tests, including
  deterministic bundle-root selection. Ubuntu is installed and
  listed, but command approval review timed out before Ubuntu execution; Unix
  descriptor-opening and replacement tests remain unverified locally.
- Review found missing terminal fetch outcomes and CSS imports left pending.
  Attributed local Blitz patches now pass 7 actual network/trait tests and 2 DOM
  completion tests, including render-wait release and failed/malformed imports.
  Integrated native CPU library tests pass 9/9, the actual native DOM suite passes
  21/21 after the IME change. Six surrounding-delete editor regressions and four
  real editor clipboard regressions also pass.
- Canonical signed project-context graph authority and the Vibe/UI binding now
  pass 30 fresh Aether host integration tests against actual current sibling
  sources. Canonical dependency unit tests are a separate pending run. This is
  an authority-owned project graph, not arbitrary daemon-global graph access.

Packet records: [dependency recovery](../implementation/resume-dependencies.md),
[Chora recovery](../implementation/resume-chora.md), [host recovery](../implementation/resume-host.md)
and [native recovery](../implementation/resume-native.md). Historical missing-cache counts below describe the original
checkpoint and are superseded only for commands with new execution evidence.

## Current implementation packets

| Packet | Owner | Status / evidence | Next acceptance work |
| --- | --- | --- | --- |
| A0 dependency recovery | dependency agent | Actual native DOM, default native, CPU/no-SVG, Vello GPU, combined default+CPU, public native+refract and optional profiler checks pass | Frozen canonical qualification and Vello runtime |
| A0 Etalon policy + CI | dependency agent + supervisor | Post-naming exact source gate and 54 source/snapshot tests pass; Windows graph has no wgpu package identities; hybrid/Vello Etalon features unchanged | Complete post-naming builds/runtime and gate final host shipping graphs |
| A0 shared Etalon revision | dependency agent | Explicit local native + canonical host graph resolves at 6ace6cc with source verification | Actual combined build/runtime; reproducible published revision alignment |
| A1 renderer cfg + startup | supervisor + dependency agent | 17 wrapper configurations; fallible launch; actual Windows CPU and default hybrid startup/redraw/teardown pass | Vello runtime, pixels, device failure and other platform runtime checks |
| A1 callbacks + window preparation | supervisor | Registry tests and actual late-auxiliary-window runtime pass; first owner drops while second keeps dispatching, then second drops | Platform suspend/resume, surface loss and installed lifecycle qualification |
| A1 document lifetime/context | supervisor + lifecycle agent | Actual package: 3 public API tests pass | Shipping native profile regression run |
| A1 assets | supervisor | Actual root: 8 library + 2 API tests; separate Windows junction swap passes; native transport regressions pass | Unix opening/replacement runtime and Android loader qualification |
| A1 input/IME | supervisor | Actual changed native DOM 21 tests, six surrounding-delete editor tests and four prior clipboard regressions pass; independent IME source review complete | Platform IME/input/clipboard qualification and remaining unsupported event contracts |
| A2 canonical host/runtime | canonical host agent | Required current QualiaDB/Vibe/ABI and real Sentinel; fresh actual host build and 30 integration tests pass | Canonical authority/budget unit tests, startup and actual native application qualification |
| A2 native UI scheduling | supervisor + canonical host agent | Actual Aether task/batch/teardown coverage in the 30 passing tests; native example and signed disposable window fixture reviewed | Build/run actual native event-loop path with canonical services |
| A3 durable data path | canonical host agent | Signed NativeAuthority session and journal/reopen coverage in the 30 passing tests | Canonical journal tamper unit tests, native application flow and installed restart tests |
| A3 scoped graph subscriptions | canonical host agent | Signed actor/context grants, bounded snapshots, journal receipts, canonical memory reservation and UI adapter pass host integration tests | Actual canonical authority unit tests and signed native window fixture |
| C0 writable Chora work | Chora agent + supervisor | Baseline-verified 62-file series applied to actual sibling checkout | Review resumed changes and full engine build |
| C1 WebGPU + canvas Etalon | Chora agent | Cargo-regenerated root lock passes source gate; actual locked servo-webgpu check passes; mapped-buffer tests pass | Full embedded engine, optional canvas build, CTS/WPT and hardware runtime |
| C2 generic embedding | Chora agent + supervisor | Producer/transport reviewed; ledger5 + actual WebIDL generation pass | Actual protocol build, external consumer and page/runtime tests |
| C3 TLS configuration | Chora agent + supervisor | Actual networking library passes 8 tests including all 6 controller/pool-cutover cases; actual networking TLS integration passes 11 tests | Engine integration, immediate revocation, platform PKI and scoped keystore wiring |
| C4 isolation | Chora agent + supervisor | Required sandbox policy now rejects unsupported configurations before startup; actual policy module passes 6 tests | Engine/parser integration tests, supported OS containment and adversarial runtime evidence |
| C5 API/ABI ownership | Chora agent + supervisor | Builder allocation fix and focused tests | Fallible TLS C-ABI path source-reviewed; external C/Rust runtime consumers remain |
| A4 mixed-site composition | supervisor + disjoint packet owners | Standalone worker, canonical parent read adapter, bounded client and native copied-frame widget implemented; protocol 7 and client 14 tests pass; native CPU integration and the actual Chora worker check pass | Actual worker executable/page/bridge/pixel flow, signed native integration, internal-navigation pixel identity, input and containment |
| A5 GPU/media/performance | supervisor | Queued; no shared GPU claim | Coherent device ownership, canonical EMF/audio, budgets and device loss |
| A6/C6 release | supervisor | Queued | Installed native artifacts, notices, signing, update/recovery and target matrix |
| Independent review | supervisor + cross-reviewing agents | Resumed graph, GPU-map and fetch lifecycle source reviews; discovered issues assigned and corrected | Final test results, post-change review and full runtime qualification |

## Discovered issues fixed during this work

- GPU gate rejected a valid legacy alias and accepted an unverified shim version;
  corrected actual-package/source checks and forked Naga revision coverage.
- Duplicate workspace exclusions temporarily broke TOML parsing; merged them.
- Renderer imports/constructor cfg differed under combined features; aligned them.
- Callback registry borrows survived user callbacks; snapshot/lookup dispatch now
  allows reentrancy/removal, skips an active FnMut, and drops captures after borrows.
- First and additional windows had different preparation; unified their contexts,
  rebuild and resume, retaining pending replacements while surfaces are unavailable.
- NoOp eval depended on a dropped owner; Unsupported is now explicit. DocumentContext
  used Arc while providers used Rc; alias and actual retrieval tests now agree.
- Contained assets broke Manganis development references; explicit trusted authority
  preserves them. Packaged requests no longer delegate file/custom schemes. Windows
  checks the opened handle before reading and MIME does not reopen the path.
- Malformed asset encodings and Android loader errors no longer use these panic paths.
- Host admission mutated issued tickets; unchanged-ticket regression now verifies a
  broader authority envelope. Canonical bounded source ingress precedes parsing;
  unsupported capability argument constraints are rejected instead of discarded.
- Chora Global is not Send/Sync; owner-thread state replaces cross-thread shared
  access. Mapped frames are copied while the guard is alive, then unmapped.
- Chora map cancellation, failed request/response delivery resource leaks and late
  notifications for destroyed documents were found and corrected in the candidate.
- Valid failed WebGPU requests must preserve already-lost-device resolution. The
  interim OperationError regression is being replaced by canonical closed resources
  and a DOM-registration acknowledgement; it is not accepted as compatibility.
- Retained async wakers were dropped/replaced under the mailbox mutex, allowing
  a reentrant producer destructor to deadlock. Move/drop outside the lock and two
  regressions now pass as part of the final 25 host tests.
- TLS tests initially used identical issuer/subject names, which made the Windows
  chain fixture ambiguous; distinct test names fixed it without weakening validation.
- The native text editor emitted input for copy and could destroy selected text
  after failed clipboard access. Copy is now nonmutating; failed/empty paste and
  failed cut preserve selection, with four actual editor regressions passing.
- The actual optional GPU build exposed a boxed Etalon Naga validation-error
  mismatch and three missing identity-swizzle fields in pinned adapters. Attributed
  fixes now pass the actual locked Vello GPU check. Default hybrid lifecycle also
  runs successfully with exact Etalon575/hypermediaa46 sources. Pixel/device-loss
  and physical-adapter qualification are not inferred from redraw callbacks.
- Windows required sandboxing previously launched a child that later reached an
  unsupported sandbox panic. Shared typed policy now rejects this before launch;
  this is admission-policy enforcement, not a Windows OS sandbox implementation.

## Explicit follow-up tasks

1. **Dependency qualification:** original cache/DNS failures have been overcome for
   the resumed native-DOM/default/CPU builds. Complete the remaining actual profiles
   and native-plus-host build, retaining exact sources and lockfiles. The historical
   archive inventory is not evidence that every optional target is now qualified.
2. **GPU source coherence:** Aether 575bab54, QualiaDB 6ace6cc, candidate Chora and
   hypermedia 8e8ab6c differ. Qualify the canonical adapters together, regenerate
   actual lockfiles and inspect all optional/target paths before sharing resources.
   The actual locked root CPU check now passes. Local native-plus-host metadata
   verifies canonical Etalon coherence; runtime and reproducible release pins remain.
3. **Scoped graph authority:** canonical NativeAuthority now has signed project
   graph grants and journal-backed context updates; fresh host integration tests
   pass. Review identified uncharged host buffers; those are now reserved through
   the canonical budget before constructing a runtime, with a small-budget
   denial regression. Canonical unit and actual-window tests remain. This project graph does not
   authorize arbitrary daemon-global graph access; source/evidence authoring
   remains a separate persisted data path.
4. **Assets:** verify Unix descriptor-relative opening and replacement/size tests.
   The Windows junction fixture now passes locally and remains explicit in CI.
   Roots must remain trusted/stable. Typed terminal fetch/cancellation is implemented
   in attributed Blitz sources; finish DOM/native integration and Android runtime
   checks. Larger admitted assets have an explicit host-selected byte limit.
5. **Native behavior:** finish IME suite, then clipboard/drag/beforeinput, pointer
   capture, accessibility and platform editing fixtures. Infallible upstream event
   conversion traits cannot manufacture unsupported native payloads.
6. **Execution:** real source/parser/provider isolation and cancellation during
   host calls, minimal host feature graph, durable extension receipt acknowledgement,
   identity enrolment and policy revocation against actual deployed authority.
7. **Chora qualification:** exact Stylo and other dependencies were retrieved;
   reviewed patches and regenerated lock are applied locally and WebGPU builds.
   Finish full embedded engine, optional canvas, conformance and teardown checks.
   Preserve SpiderMonkey ICU and native text support: incompatible in-process ICU
   constraints require separate-process composition, not feature removal.
8. **Embedding/crypto/isolation:** complete engine-derived caller transport, test
   origin spoofing/navigation/profile separation, add scoped keystore handles only
   after that authority exists, and qualify untrusted process isolation.
   Live TLS root reload now uses a non-serializable host capability and prepares
   both profiles before swapping connection snapshots. Existing operations and
   host exceptions retain their explicit authority; immediate revocation returns
   Unsupported before mutation. The atomicity, profile-isolation and live
   controller/pool-cutover tests now pass in the actual networking library. The
   first loopback request was blocked by Windows socket policy; approved execution
   then passed the exact loopback test and the complete eight-test library suite.
   The actual networking TLS integration target also passes all 11 selected cases.
   Engine runtime, platform PKI and scoped keystore authority remain separate gates.
9. **Distribution:** installed multi-platform native/browser artifacts, resource
   inventory and MPL/upstream notices, failure diagnostics, signing/update/recovery,
   and measured canonical EMF/media/GPU behavior on supported hardware.

## Records

- [Initial audit/plans](../audits/2026-11-12/README.md)
- [Implementation evidence](../implementation/README.md)
- [Independent review](../implementation/REVIEW.md)
- [Canonical host verification](../../packages/qualia-host/VERIFICATION.md)
- [Native input evidence](../implementation/native-input.md)
- [Combined Chora patch series](../implementation/chora-patches.md)
- [Chora Etalon candidate](../implementation/chora-etalon/README.md)

The separate editable Chora snapshot is
`C:/Users/Hp/AppData/Local/Temp/chora-etalon-overhaul`. Baseline-hashed patches are
exported under notes/implementation. The previous session was read-only for the
sibling; this resumed session applied the verified patches to `C:/github/chora`.
The temp copy is historical evidence, not the active source. No release or full
browser runtime qualification is claimed by applying those changes.

## Resumed 14 November 2026

The user explicitly authorized resumption. A4 now has the bounded protocol,
separate actual Chora worker, parent process owner, canonical read bridge/client
pump, and native copied-frame widget. Current evidence and limitations are in
[the worker packet](../implementation/resume-worker.md); this supersedes the
paused status above without relabeling historical tests.

Protocol7/7 and parent-client6/6 plus two actual private-pipe process tests pass.
The native CPU+prelude+Chora-frame feature compiles. The exact saved post-naming
hybrid executable also passed its bounded lifecycle run, with its recorded hash.
Chora's full embedded engine and explicit canvas/profiler checks pass; its actual
production-source identity fixture passes15 tests. Worker engine execution,
current canonical-host tests, precise internal-navigation pixel ownership and
remaining release gates require their separate recorded results.


## Independent preparation accepted — 15 November 2026

[The preparation packet](../implementation/preparation-2026-11-15/README.md)
contains WP01 preservation/artifacts, WP06 held recipes, WP08/WP16 platform
matrices and WP04/WP09/WP14/WP15 requirements. Acceptance is for preparation
only; the 51 implementation/qualification jobs remain open. The supervisor added
LICENSE-QUALIA with exactly the existing LICENSE bytes following the user's
current-policy clarification. No new stage, Cargo run, worker/device probe or
sibling inspection occurred. WP02 and dependent execution still wait for the
user's confirmation of committed sibling fixes.

## Intake execution — 15 November 2026

The committed-fix notification has arrived. [Intake 1](../implementation/UPSTREAM-INTEGRATION-REGISTER.md)
records verified GitHub/local commits and current U01–U06 dispositions. WP03 is
implementing an opt-in, content-bound live dependency profile; WP05 completed its
commit-bound contract review. WP06 built and staged the actual isolated worker,
then added bounded diagnostics after a startup-readiness failure. Client tests:
13 unit and 3 process tests pass; approved localhost worker caller/denial/64x64 RGBA and
three supplemental page-window crypto cases now pass, with restricted failures
preserved. WP04's live host checks follow actual source resolution. These tasks
use disjoint implementation agents and one Cargo owner; historical pauses above
are superseded, while immutable captures and uncommitted work remain preserved.

The live combined native/host source gate passed (673 packages, 25 GPU packages)
and the actual Cargo library check exited zero after 37m09s. Post-run verification
then detected 28 changed QualiaDB inputs at the same HEAD. Keep this as a successful
moving-tree check; stable source qualification, linked native/host artifacts and
current standalone host tests remain open. The intake register records the next
host test-root validation and isolated Cargo lockfile procedure. No commits,
pushes or branch changes were made; shipping pins and frozen-v4 bytes are retained.
