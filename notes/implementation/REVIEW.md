# Implementation review — 2026-11-12

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

This records an independent source review of the shared Aether working tree and
the separate Chora candidate snapshot. It is not production certification or a
review of a merged Chora change. Existing user changes were preserved.

## Findings corrected during implementation

- Merged duplicate Cargo workspace exclusions that prevented TOML parsing.
- Corrected GPU policy rejection of valid legacy Etalon aliases, forked Naga
  revision gaps, nonstandard member enumeration and an unverified shim exception.
- Matched native networking cfg guards to non-WASM dependency declarations.
- Kept unsupported document evaluation independent of a dropped generational owner;
  aligned the public document context alias with actual Rc context lookup.
- Removed mutation of authority-issued admission tickets. The regression authority
  now grants an envelope broader than adapter bounds, so the former clamp would fail.
- Reused canonical bounded source ingress before raw program parsing, and rejected
  capability declaration arguments that the adapter's ID-only grants cannot enforce.
- Restored legitimate Manganis development absolute-source asset handling through
  an explicit trusted profile. Packaged URL resolution does not escalate authority
  when lookup fails; Rust asset helpers remain documented trusted application APIs.
- Corrected a final Chora map-completion edit that referenced an out-of-scope Global.
  The callback now retains the resolved buffer and reads its guarded mapped range,
  rather than resolving a potentially removed resource ID on completion.
- Moved mailbox Waker cloning, replacement and destruction outside its mutex.
  A legitimate last Waker owning the sender could otherwise deadlock on drop.
  Teardown and invalidation also destroy queued values after releasing the lock.
- Released paired Chora device/queue identities on rejected or undelivered requests;
  ordered backend removal before ID reuse and ignored obsolete script epochs.
- Added a device-registration barrier for early loss and replaced a temporary
  OperationError regression with genuine already-closed Etalon device/queue entries.
- Rejected blob/data/inherited-document URLs in Chora messaging even when their
  security origin is HTTP(S), and closed retained-request cancellation during shutdown.
- Prepared TLS verifiers before thread startup rather than reconstructing them on
  active request paths; separated public/private TLS session caches after identifying
  that cloning a rustls ClientConfig shares the template's session storage.

## Final source observations

Aether initial and additional windows use the same context/rebuild preparation.
The exact cached Blitz 0.3.0-beta.4 implementation was inspected: temporarily
retaining pending configurations while Blitz resumes existing views avoids generic
creation without Aether contexts, and each new view receives its own resume.
Pending replacement windows prevent premature last-window exit. Real window and
surface lifecycle behavior still requires native runtime tests.

Callback registry borrows end before user callbacks or captured destructors run.
Nested dispatch skips active mutable callbacks; removals invalidate queued IDs.
The startup result API preserves underlying runtime/event-loop errors and states
its remaining panic, platform and deferred GPU limitations.

Asset decoding rejects malformed percent encoding, invalid UTF-8, NUL, traversal,
drive/ADS/UNC forms, backslashes and Windows trailing-dot/space ambiguity.
Canonical containment rejects stable symlink/junction escapes. Windows packaged
reads now keep one File open, validate GetFinalPathNameByHandleW containment before
reading, and infer MIME from the resulting bytes without reopening. The FFI buffer,
live handle and bounded resize loop were source-reviewed. Stable trusted roots,
hardlinks and concurrent admitted-file content changes remain explicit limits;
non-Windows trees still require immutability during resolution/opening. Explicit trusted
development authority must remain unavailable to untrusted content. Direct binaries
with Manganis placeholder source paths may require explicit trusted configuration.
Blitz's bytes-only NetHandler cannot report typed fetch failure; unsuccessful
responses are withheld rather than published as successful resource bytes.
The Packaged profile now checks transport entry schemes before delegation: only
HTTP, HTTPS and data are delegated, while aether paths use contained resolution.
Other local/custom schemes cannot acquire authority through the inner provider.
Exact pinned blitz-net file handling could not be inspected from available caches;
the outer check therefore supplies the boundary independently. HTTP origins and
redirect behavior remain transport qualification work, not filesystem permission.

The host adapter reuses actual canonical engine, ABI, core database, ScopedEffects
and Sentinel types. Its concrete attached-snapshot check is a configuration guard,
not proof of daemon startup or authenticated admission. Caller-owned RAM receipts
are distinguished from durable journal acknowledgement. SourceSession ingress
also restricts the admitted language profile; it is not unrestricted program
embedding. Native integration, scoped graph access and transactional script effects
remain release gates.

The additional NativeAuthoringBridge uses actual opaque NativeAuthority sessions
and canonical source/workflow evidence with disk-backed authoring journal methods.
It pins source revision and admission generation, coalesces one bounded pending
snapshot, and invalidates publication on close. This is source-authoring authority,
not semantic graph authority or a substitute VibeScript extension ABI. Source/tests
were reviewed; canonical snapshot calls can still block after the freshness check,
so count/representation bounds are not a nonblocking UI or real-time guarantee.
The mailbox drains pending deliveries before producer EOF, replaces one async
waiter, and keeps receiver teardown independent of application/scheduler callbacks.
Source recheck confirms the reentrant-Waker fix and queued-value destruction both
release the mutex first.

Native DOM dispatch checks payload types against its supported event profile.
IME observations track target identity, cancel the previous target on a switch,
and clear sessions when focus or nodes disappear. Composition notifications do not
perform a second editor insertion or imply cancellation of Blitz's native edit.
Unsupported infallible HtmlEventConverter methods remain a documented inherited
trait limitation; emitted supported native dispatch is guarded from those stubs.

Chora's candidate creates the non-Send Global on its GPU owner thread and retains
it through Rc. Completion callbacks signal the owner; presentation retains CPU
bytes copied while the mapping guard is live, then unmaps before renderer access.
Final staging-buffer release notifies the owner, which removes the resource before
acknowledging ID reuse. No new unsafe Send/Sync assertion was found in these paths.
Final error scopes route through Etalon, device-loss callbacks ignore removed IDs,
and pending maps are cancelled once on shutdown with late signals ignored.
Map and queue-done callbacks share owner completion ordering. Full request/script
integration, all outstanding promises and GPU teardown require runtime qualification;
narrow API probes do not compile the complete engine.

Final lifecycle review additionally identified request-device reservations and
resources without an owner when request or response delivery fails. Source recheck
confirms local reservation release on failed requests and backend scope/queue/device
cleanup before acknowledgement of undelivered success. Runtime fault injection is
still required. Queued loss/error notifications ignore a removed script document.
Loss is staged until the DOM weak registration sends DeviceReady; unknown or
retired identity epochs are benign. The final Other branch restores already-lost
DOM-device resolution using real destroyed Etalon resources. Its private featureless
Noop instance is separate from browser adapter selection; resources are destroyed
before import. DOM registration precedes loss and request-promise resolution.
Canonical invalid-resource and cross-device validation are exercised in the actual
Etalon fixture, including invalid views and lost asynchronous pipelines. Browser
promise timing and end-to-end CTS/WPT behavior remain unqualified.

Chora's generic messaging candidate derives instance, WebView, pipeline, navigation
epoch, private profile and origin from engine state. It accepts only active top-level
HTTP(S) documents with matching actual immutable origin and no sandbox flags.
The route retains the exact pipeline URL: same-document history/hash changes rotate
the epoch and cancel pending replies. Request/reply sizes, pending count and elapsed
time are bounded; idle waits honor the earliest deadline. Dropped replies deny,
navigation/close/shutdown invalidate requests, and retained host handles receive a
cooperative cancellation token even after view removal. Shutdown refuses queued
new requests and marks tokens again after the shutdown spin. This supplies identity
and reply lifecycle, not automatic Qualia authority or rollback of host work.

Chora TLS preparation explicitly selects platform, WebPKI or embedder-only trust;
malformed supplied roots fail before resource threads start. Public/private HTTP
and WebSocket paths derive configs from the prepared verifier/provider template,
with separate certificate-override managers and fresh TLS resumption storage.
Live root reload and keystore wiring remain absent. ServoBuilder::try_build covers
TLS configuration failure, not every engine initialization failure. The C builder
is reclaimed before fallible startup, returns null on configuration failure, and
catches Rust unwinds before the ABI boundary; abort behavior and full C integration
are outside the narrow ownership tests.

## Evidence and limits

Independently executed by the reviewer:

- 11 GPU policy regression fixtures passed.
- All 17 renderer wrapper contract configurations passed, plus explicit missing
  renderer and optional dependency checks. These use narrow backend contracts.
- Six current Windows asset-policy tests passed, including opened-handle reads.
  A seventh junction-swap fixture failed while creating the junction, before testing
  containment. Ordinary-path retry also failed with Access denied. This test is now
  explicitly ignored locally, selected in Windows native CI, and selectable through
  verify_native_core.py --include-junctions; no junction race result is claimed.
  The Unix symlink test was not executed on this host.
- Three Chora owner-queue tests passed at the review checkpoint, including cancellation followed
  by a late success signal. A transient execution-service failure was retried.

Other agents report 25 host tests against actual cached canonical artifacts and
actual Aether core, including signed authority/journal reopen, revocation, busy-owner
retry, native task lifecycle, and two reentrant-Waker regressions. Native DOM's
actual headless source suite reports 20 passing tests, including end-to-end IME
commit-once behavior; this is not the locked shipping feature graph.

The reviewer inspected successful Etalon API/runtime logs. The core probe compiles
actual shared types/protocol/encoder and error modules; the final runtime fixture
passes 11 tests using production closed-device, invalid-resource, asynchronous
pipeline, readiness and owner-queue modules against pinned actual Etalon. It does
not compile the complete owner request handler or script/DOM integration. CAPI's
two ownership tests and the five source-policy fixtures also pass as reported.
Final source/export hashes must include the late lifecycle fixes.

Chora messaging reports five executed ledger tests, 12-file Rust parse/format
checks and actual WebIDL generation. Protocol-module validation tests are present
but unexecuted because their actual dependency graph could not be built. TLS reports nine
source-copy tests using actual rustls and real TLS/mTLS handshakes. Its resumption
isolation regression also failed as expected when only the fresh-cache reset was
removed from the generated fixture. These suites were not independently rerun by
this reviewer; see their verification records for exact inputs and restrictions.

The supervisor also reports successful compilation of the actual asset-resolver
under an isolated 73-package graph. Its API fixture verification is recorded
separately from the independently executed lexical policy tests above.
The final native-core verification reports document 3/3, asset policy 6/6 with one
junction test ignored, and asset public API 2/2. The junction creation failure log
is retained as asset-junction-attempt.log.
`scripts/verify_native_core.py` was source-reviewed: it builds actual path crates
and their public document/asset test files with checked-in targeted graph locks;
it does not replace implementations or certify the shipping workspace graph.

The full Aether CPU build remains blocked by unavailable locked utf8-width; other
requested graphs also lack dependency metadata/archives.
The requested minimal host feature graph and fresh pinned sources are unqualified.
The Chora root is blocked by unavailable Stylo, and its canvas probe build by the
missing svg_fmt archive despite graph resolution. Browser CTS/WPT, packaged native
launches, device loss, mixed-site authority, application-level durable transactions,
non-Windows race-resistant asset opening and cross-repository Etalon/device coherence
are not established. Source review permits exporting unmerged review packets; it
does not approve merging, release qualification or production native readiness.

## Resumed independent review — 12 November 2026

The paragraphs above preserve the original checkpoint. The actual Chora sibling
is now writable with the verified patch series applied. Dependency retrieval has
restored actual native DOM/default renderer/CPU builds and the Chora WebGPU build;
the historical missing-cache and failed junction-creation statements above are
superseded by the specific new evidence records linked from the live tracker.

The supervisor reviewed the resumed canonical graph scope/session and journal
binding, including historical signed receipts, failed-commit behavior, freshness,
nonblocking UI reads and teardown. The review required explicit canonical runtime
memory reservation, bounded host inputs, fallible buffers and accounting for
retained query results. Those fixes now pass the fresh 30-test host integration
suite against actual current canonical sources. Dependency unit tests are being
executed separately; the host suite does not count those tests as passed.

Cross-review of the typed network completion changes found missing cancellation
in async/callback entrypoints and CSS imports that remained pending on error.
Both were corrected. Actual results now pass 5 trait tests, 2 network tests,
2 DOM completion tests and 9 native CPU library tests. Late success is converted
to cancellation, application wakers are invoked/dropped outside the mutex, and
the document thread owns the stylesheet terminal transition.

The supervisor also found destructive clipboard failure behavior in the actual
Blitz editor. The fix preserves text/selection on failed cut/paste, makes copy
nonmutating, supports uppercase shortcuts and makes empty paste a no-op. The
Chora agent independently reviewed the code and four real editor regressions,
with no further actionable findings. All four actual editor regressions now pass;
the fixture's key-state type mismatch was corrected before that successful run.

The supervisor reviewed Chora's mapped-buffer completion guard: failed response
construction/delivery unmaps, successful delivery transfers ownership, and claimed
ranges are checked against the actual buffer before copy. The actual WebGPU build
and 13 pinned-Etalon lifecycle tests pass. This does not prove full engine,
browser promise timing, shared native GPU ownership or hardware conformance.

New single-window and late-auxiliary-window probes exercise actual native startup,
per-window contexts, redraw callbacks and teardown. Both actual Windows CPU
examples now build and run successfully. The revised auxiliary probe closes the
first window only after the second starts, avoiding reliance on redraws to an
occluded first window. Both owners drop, and the second keeps dispatching after
the first closes. This is lifecycle evidence, not pixel or GPU qualification.
WSL Ubuntu is installed/listed, but command approval review timed out before
Ubuntu execution; the Unix asset tests remain unverified locally. Release,
installed artifacts and cross-platform/hardware qualification remain open.
## Resumed sandbox admission review

The supervisor independently reviewed Chora's shared sandbox-request validator,
typed initialization errors, parent launch checks, child bootstrap and shell
parser. Unsupported requests are rejected before engine globals or parent IPC
creation; the defensive child check precedes options and script initialization.
Explicit trusted unsandboxed defaults remain available. No additional source
finding arose in this review. Six actual policy-module tests pass; engine/parser
integration execution remains pending. Windows previously reached an unsupported
child sandbox panic: these changes enforce admission policy and do not implement
or qualify Windows OS isolation.

## Resumed source and IME review

The supervisor reviewed the independent Hypermedia commit policy, alias handling
and exact approved local manifest identities. No new issue was found in those
checks. Workflow review did find an incorrectly indented minimal-CPU matrix entry;
the dependency agent corrected it and verified the complete YAML with PyYAML 6.0.3,
including all five peer profile entries and the new editor regression step.

Independent Chora-agent review of the surrounding-text deletion implementation
confirmed that protected preedit bytes never rely on grapheme-snapped temporary
selections, both UTF-8 endpoints are validated before mutation, and original
selection endpoints are remapped independently. Six actual locked editor tests
pass, and all 21 changed native DOM tests pass, including real input routing.
The fallible temporary
buffer reservation is not a general layout-memory guarantee.

## Etalon implementation naming review

Independent review normalized all 55 Aether Hypermedia Rust files against the
recorded `a46b4e2` source. The remaining changes are the intended Etalon names,
three texture-view swizzle defaults, boxed shader-error handling and documented
compatibility exports. Original copyright, SPDX and license lines are preserved.
Review found that a legacy helper alias in a private context module was not
exported at the crate root; this is corrected. Two inherited comments now retain
their historical provenance without inventing an Etalon version or relabelling
the original code. Standard WebGPU and WebGL API names are unchanged.

The minimal hybrid and Vello consumers retain identical resolved Etalon feature
sets after disabling defaults on the direct native type dependency. The actual
post-naming default and Vello native builds pass. Optional-profiler verification
stopped at Windows Application Control before its shader build script executed;
this is an uncompleted check, not a passing result.


## Separate-worker review — 14 November 2026

Independent review checked the actual worker APIs and protocol/client/native
ownership boundaries. Explicit parent navigation initially reused a delegate
while `load()` was only an asynchronous request; old callbacks could thereby
receive a new generation stamp. The worker now retires the old view/delegate
before creating a new one, and ignores all retired callbacks. New readback waits
for the new view's frame-ready notification and paints that view before reading
its retained rendering context. Close/crash also cancels pending ownership.
Internal same-view navigation still lacks a public document-to-paint commit
identity and remains an explicit runtime/architecture gate.

Review also corrected a same-stamp resize acknowledgement cancelling a later
capture, repeated captures of static pages waiting indefinitely, startup lock
contention, expected stale parent replies unnecessarily shutting down a new view,
and escaped reply sizes failing asynchronously in the writer. Same-stamp
acknowledgements preserve already-current ownership; stale replies discard only
the old request; the real bounded codec now preflights reply serialization.

The protocol and parent pipe tests use actual source. The explicit process peer
imports the worker's exact stdio-isolation helper and verifies that stdout
diagnostics cannot corrupt protocol bytes. Those tests do not run Chora. Actual
Chora embedded-engine, identity fixture and optional canvas/profiler checks have
separate successful receipts. Worker executable and signed canonical integration
results must be taken from their packet logs, not inferred from this review.

Windows GUI launch: actual installed Rust std propagates a NULL inherited stderr
handle when the GUI parent lacks one. Client now supplies a private stderr pipe,
drains to sink with bounded memory, and retains no diagnostic bytes. Independent
review found no lifecycle/backpressure issue; all 11 client tests pass, including
a 256 KiB synchronous diagnostic burst before protocol acknowledgement. Evidence:
`resume-chora-client-stderr-tests.log`. No Windows security settings changed.
