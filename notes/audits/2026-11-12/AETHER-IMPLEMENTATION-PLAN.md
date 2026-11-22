# Aether implementation plan

## Outcome and boundary

Deliver production native applications with a required QualiaDB host, canonical
VibeScript execution and Etalon rendering. Deliver a separate Chora adapter for
traditional web content and mixed sites. Keep historical JavaScript execution in
compatibility content; do not require it for the native human-centric application
path. The browser host owns policy and privileged services, not page scripts or
the renderer.

This plan describes work still required. Packet completion requires the stated
evidence, not just merged code or a renamed dependency.

## A0 — Reproducible baseline and dependency policy

**Scope:** root/workspace manifests, lockfile, toolchains, renderer adapters and CI.
**Findings:** A02, A13, A14. **Dependencies:** none; coordinate with Chora C0/C1.

- Resolve the existing locked graphs in clean writable caches, recording source
  revisions and compiler/target versions. Establish per-profile Rust baselines.
- Select and pin one reviewed Etalon revision across QualiaDB, Aether, Chora and
  hypermedia-production. Update dependent adapters together; do not assume the
  newest local checkout is compatible. Avoid uncontrolled branch updates.
- Add a source-aware dependency gate covering wgpu/core/hal/types, shader tooling,
  adapters and target/optional paths. Reject upstream implementations. An explicitly
  approved Etalon compatibility shim may retain a wgpu package/import name.
- Include lockfiles, Refract and relevant configuration in CI change detection.

**Exit:** default native, explicit GPU, hybrid and CPU profiles resolve and compile;
dependency evidence contains no prohibited upstream implementation. Record expected
feature exclusions. Repeat the graph checks in the final host workspace because
Cargo patches in dependency workspaces do not automatically apply there.

## A1 — Native correctness and trusted asset boundaries

**Scope:** `packages/native`, `native-dom`, `document`, `asset-resolver`.
**Findings:** A03–A11. **Dependency:** A0 for compiled verification.

- Use one consistent renderer selection predicate for imports, option types and
  constructors, or reject unsupported combinations with explicit diagnostics.
- Make event mutation during dispatch safe. Specify whether new handlers run in
  the current event, whether removal suppresses later callbacks, and how nested
  dispatch behaves. Test self-removal and component cleanup during a callback.
- Consolidate initial/additional window preparation and teardown. Ensure contexts,
  subscriptions, event handlers and GPU resources follow each window's lifetime.
- Correct unsupported evaluator lifetime and error semantics. Audit event conversion
  reachability; implement supported input and return documented unsupported behavior
  for the rest. Add IME composition, Unicode, focus, clipboard and accessibility tests.
- Return startup errors and define device/window recovery. Correct asset status and
  cancellation propagation; make WASM/net/tracing guards match supported profiles.
- Separate packaged asset lookup from explicitly authorized user files. Canonicalize
  within allowed roots, handle symlinks/reparse points and race-resistant opens,
  reject malformed paths without panics, and scope origin access. Avoid blanket CORS
  on privileged assets. Preserve legitimate app asset workflows through explicit APIs.

**Exit:** targeted regressions pass; native first/second-window interaction and
repeated create/close tests pass on the first release platform. Traversal, absolute
path, invalid encoding, symlink escape and denied-origin asset tests pass before
connecting any untrusted Chora surface.

## A2 — Required QualiaDB host and canonical authority

**Scope:** Aether/Webizen host composition and adapter; canonical QualiaDB crates.
**Findings:** A01, A12. **Dependencies:** A0, stable A1 lifecycle.

- Map the existing dependency graph before choosing adapter placement; QualiaDB
  already depends on Aether. Reuse narrow contracts and avoid a cyclic dependency
  between top-level applications. Define the required host initialization sequence.
- Bind the existing `vibe` engine and `vibescript-abi` provider SDK. Reuse descriptors,
  borrowed/caller-owned memory, traps, cancellation and lifecycle contracts.
- Implement real admission and post-execution validation/receipts. Do not ship a
  simulated gate as authority. Bind identity, operation, resource, origin/frame when
  relevant, grant lifetime and policy generation to each request.
- Give UI scheduling, script execution and GPU ownership explicit boundaries.
  Bound execution work, queue sizes, output sizes and subscriptions. Cancel on
  component/window destruction and reject stale replies after navigation/reload.
- Distinguish application authority from requested page authority. A running host
  and local database must never imply access for an arbitrary web page.

**Exit:** native integration fixture opens real QualiaDB data, executes an admitted
VibeScript operation, records its receipt and renders the resulting update. Denied,
expired, cancelled and over-budget operations leave no unauthorized effect. Missing
required services produce an actionable startup error, not an optional mock fallback.

## A3 — Native human-centric UI and data semantics

**Scope:** native bindings, reactive updates, input and canonical Vibe UI contracts.
**Dependencies:** A1, A2.

- Reuse existing Vibe UI/form/input/layout and CBOR-LD patch contracts where suitable;
  document supported schema versions and validate all cross-runtime payloads.
- Map canonical objects to native view state with stable identities, bounded batch
  updates and deterministic ordering. Preserve provenance and identity across edits.
- Support persistence/restart, offline operation, subscriptions, transactional errors
  and recovery. Propagate errors as typed application state, not silent empty views.
- Keep canonical EMF/material and immersive audio data intact wherever these enter
  the native rendering pipeline. Perform display/audio-device conversion at output
  boundaries; do not collapse stored/computational data into RGB or audible bands.
- Benchmark Refract separately before selecting it for a specific reactive hot path;
  its optional facade export alone does not establish integration or allocation gains.

**Exit:** a packaged native example reads, edits and observes persisted data through
VibeScript with no JavaScript evaluation dependency for those operations. Input and
accessibility flows work; restart/offline/failure tests preserve data semantics.

## A4 — Chora adapter and mixed-site composition

**Scope:** host-side adapter, browser shell and service broker.
**Dependencies:** A2/A3 and Chora C1/C2/C4/C5 readiness for the selected platform.

- Depend explicitly on the pinned Chora fork despite its current `servo` package
  name. Keep ecosystem code outside Chora. Define process/thread/ABI ownership.
- Implement mixed content within a page/site, not just two unrelated application
  windows. Define layout, focus, input, accessibility, navigation and surface
  composition boundaries for traditional and human-centric components.
- Use canonical ecosystem descriptors/formats for component discovery and execution.
  Specify unsupported versions and content policy; do not invent a competing ABI.
- Route generic Chora embedding requests through the host broker. Authenticate the
  engine-supplied caller context; page-supplied origin strings are not authority.
- Scope grants to operation/resource/context. Cover redirects, nested frames,
  opaque origins, navigation epochs, workers, back/forward cache and profile/session
  isolation. Add revocation, user-visible permission state and auditable results.
- Maintain a requirements list for actual Node/V8-specific applications. Standard
  web JavaScript uses Chora's SpiderMonkey; do not promise Node/V8 interfaces on
  that basis or add V8 indiscriminately.

**Exit:** one HTTPS test site contains traditional JS and human-centric components;
authorized local QualiaDB/Etalon operations work while a cross-origin iframe and
revoked/stale requests are denied. Navigation cancels pending effects. Traditional
web behavior remains intact. Engine crashes cannot grant additional host authority.

## A5 — GPU, media and performance qualification

**Scope:** Etalon host context, native adapters, Chora surfaces and instrumentation.
**Dependencies:** A0–A4 for mixed workloads; native-only work can start after A3.

- Define device/queue ownership and threading; validate shared textures, fences,
  surface lifetime, resize, scale, suspend/resume and device loss. If processes own
  separate devices, specify supported external-memory transfers or copy paths.
- Preserve native EMF and immersive media semantics through computation. Test
  explicit output conversions rather than claiming WebGPU/CSS has native support
  for all ecosystem data models.
- Set measured budgets for startup, memory, input-to-frame latency, steady-state
  allocations, database change bursts and scripting work on representative hardware.
- Exercise missing adapters, driver faults and supported CPU fallback. Establish
  visual/reference tests and real hardware checks for each advertised backend.

**Exit:** published workload definitions and repeatable measurements meet agreed
release budgets; no leaked GPU resources or stale access under lifecycle stress.

## A6 — Distribution and release gates

**Scope:** CLI, bundles, CI, release docs, notices and operational diagnostics.
**Dependencies:** preceding packets for capabilities included in a release.

- Package and launch the native renderer explicitly on Windows, macOS and Linux;
  verify installed assets/fonts, clean-user startup and offline first launch.
  Qualify Android/iOS separately before advertising mobile readiness.
- Correct license files and links, preserve Dioxus/Chora/upstream attribution,
  inventory actual distribution dependencies and scope any Etalon license-policy
  changes. Verify installation URLs and examples from a fresh external consumer.
- Establish signing, update/rollback, migration recovery, crash reporting with
  privacy controls, security update ownership and reproducible source manifests.
- Add required native runtime, mixed-site authorization and negative dependency
  gates. Retain WebView checks as separate compatibility coverage.

**Exit:** release artifacts pass an installed-app matrix and every applicable P0/P1
finding has linked closure evidence or the affected capability is explicitly
excluded from the release. Do not describe excluded capabilities as delivered.

## Minimum validation matrix

| Axis | Required coverage |
| --- | --- |
| Renderer features | Default hybrid; explicit Vello; CPU softbuffer; supported Skia; default plus CPU/Skia; documented invalid combinations |
| Hosts | Native QualiaDB application; mixed Chora browser; separate WebView compatibility |
| Targets | Windows/macOS/Linux shipping targets; WASM feature checks; mobile qualification before claims |
| Authority | Allow, deny, expiry, revocation, stale reply, cross-origin frame, worker and profile separation |
| Lifecycle | First/second window, nested callbacks, reload, navigation, suspend, device loss, crash, shutdown |
| Data/media | Persistent data, subscriptions, provenance, canonical EMF preservation, display/audio conversion |
| Distribution | Clean install, no source checkout, required resources, notices, upgrade/rollback |

Start with A0 and A1 alongside Chora C0/C1. Keep the implementation tracker in
[13-NATIVE-PRODUCTION.md](../../architecture/13-NATIVE-PRODUCTION.md) tied to actual
test artifacts and resolved dependency graphs.
