# Native production overhaul

## 22 November application platform source pass

The six work packages add native widget and event lifecycle handling, Etalon
viewports and window roles, route-stamped old-web input, QualiaDB host hooks,
and Aether subscriptions to Refract paths. The source pass includes unit tests.
Cargo verification and runtime qualification remain with the coordinator.

The unpatched renderer adapters still own window-surface initialization. The
optional gpu-cpu-fallback feature enables a native softbuffer retry when that
initialization unwinds. Verify this path, rendered pixels, suspend/resume, and
device-loss behavior on real hardware before release. Mobile-native routing also
requires Android and iOS runtime verification.

The final source review checked the pinned Blitz, AnyRender, Etalon, Chora and
QualiaDB interfaces. The Blitz proxy carries a sendable wake event for opening
auxiliary windows while the VirtualDOM stays on the owner thread. Software
fallback is limited to opaque windows because softbuffer presents RGB. Worker
coordinates are mapped from page space, and cancellation retains receipts from
already-queued Vibe results. These checks add no compilation or runtime evidence.

The old-web worker still carries the existing MPL packaging gate. Verify
input, focus, IME, and resize/navigation ordering across the sibling process.
Canonical authority and reconciliation tests remain standalone. Verify the
HOLO proxy against the renderer before treating the viewport as production ready.


Current implementation status is maintained in
[14-IMPLEMENTATION-TRACKER.md](14-IMPLEMENTATION-TRACKER.md) and the
[implementation evidence](../implementation/README.md). The initial audit below
records the baseline; implemented fixes do not imply that all release gates pass.

The local 12 November recovery has progressed beyond the original dependency
failure: actual Aether native-DOM/default renderer and patched CPU builds pass;
the reviewed Chora changes are applied locally and its locked WebGPU package
build passes. See [native recovery](../implementation/resume-native.md),
[dependency evidence](../implementation/resume-dependencies.md),
[canonical host work](../implementation/resume-host.md) and
[Chora evidence](../implementation/resume-chora.md) for exact scope.

Actual Windows CPU single-window and late-auxiliary-window lifecycle probes now
pass, including per-window contexts, continued redraw dispatch after the first
window closes, and both component owners dropping. The actual patched native DOM
suite passes 21 tests, six surrounding-delete editor regressions pass, and four
real editor clipboard failure regressions pass.
The default hybrid renderer's native lifecycle probe also passes, and attributed
shader/texture-view fixes now pass the actual locked Vello GPU check. These results
do not establish pixel correctness, device-loss behavior or physical hardware
coverage. Canonical application execution and installed qualification remain
separate gates. Minimum compiler versions need separate checks; the retained
Vello shader manifest declares Rust 1.89.

The subsequent Etalon naming migration has passed source review and actual locked
default and Vello native builds. Earlier runtime results above apply to the
recorded pre-migration builds. The post-migration profiler build and frozen
canonical-host build encountered Windows Application Control before their build
scripts could execute; these checks and subsequent runtime qualification remain
open. See the live tracker for the latest profile-specific evidence.

The mixed in-process consumer exposed incompatible Stylo/ICU requirements across
Blitz and SpiderMonkey. Preserving native text behavior and Chora JavaScript
compatibility requires a separate process/ABI composition path unless those
upstream contracts are jointly qualified. GPU source alignment alone does not
resolve this issue. Host authorization, cancellation/crash boundaries, full browser
execution, hardware tests, installed artifacts and cross-platform release evidence
remain mandatory gates.


## Purpose

Aether is a Dioxus fork being overhauled for native QualiaDB and Webizen applications.
The entire ecosystem depends on QualiaDB: it is the required foundation of the
target architecture, not an optional data backend. Aether UI, Etalon rendering and
VibeScript application scripting must integrate through that foundation and its
existing SDKs. VibeScript supplies application scripting for works authored for the
human-centric web. Chora must also support traditional web environments and their
JavaScript applications. Human-centric works should use historical dependencies
only sparingly for explicit compatibility needs. Traditional web compatibility
tests do not establish native ecosystem readiness. The current absence of dependency wiring is an implementation
gap; it does not make QualiaDB optional in the intended product.

This is an initial source inspection, not a completed production certification or
a comprehensive review of the sibling repositories. Baseline: Aether `d5ccce3`,
reviewed 12 November 2026, plus the changes described below.

## Ownership and integration boundaries

| Layer | Owner / source | Aether responsibility |
| --- | --- | --- |
| Required ecosystem foundation | QualiaDB | Build the target runtime on its canonical SDKs and contracts |
| Components, events and UI scheduling | `packages/core`, `packages/signals`, `packages/hooks` | Native UI lifecycle and responsive updates |
| Native DOM and window shell | `packages/native-dom`, `packages/native` | Layout, input, accessibility, window and surface lifecycle |
| GPU and painting | Etalon and hypermedia-production dependencies in root `Cargo.toml` | Compatible device/resource ownership and renderer configuration |
| VibeScript language engine | QualiaDB `crates/vibe` | Integrate the existing engine through explicit host contracts |
| VibeScript provider SDK / ABI | QualiaDB `crates/vibescript-abi`, including `src/sdk.rs` | Reuse provider descriptors, lifecycle, resource and memory contracts |
| VibeScript tooling | Sibling `vibe-script` repository | Reuse tooling; retain language implementation in QualiaDB |
| Data and authority | QualiaDB / Webizen host contracts | Connect authorized operations and subscriptions to UI state |
| Lens reactivity | `refract` | Optional public API; future native scheduler integration |

VibeScript is not assumed to be JavaScript-compatible. An existing API accepting
JavaScript strings must not silently reinterpret those strings as VibeScript.
Native scripting should expose explicit language selection, structured results,
host authority, cancellation, execution budgets and teardown using existing SDK
contracts wherever applicable. Do not introduce a competing ABI or policy engine.

### Existing VibeScript SDK surfaces inspected

The local QualiaDB checkout exposes these existing integration surfaces:

- `crates/vibescript-abi/src/sdk.rs`: compile-time provider/function descriptor
  constructors, ontology `NQuin48` builders, and checked finite slice I/O.
- `crates/vibescript-abi/src/lib.rs`: `VibeNamespaceProvider`, `ProviderDescriptor`,
  `ResourceEnvelope`, `SentinelPolicyGate`, admission tickets, execution receipts,
  borrowed input slices, caller-owned output slices and resource handles.
- `crates/vibe/src/lib.rs`: `Host`, `Engine`, `Budget`, `Env`, `load_program`,
  `eval_function` and `dispatch_hook` for embedding the language runtime.

These are source observations, not new Aether interfaces. Determine the concrete
registration and dispatch path against QualiaDB's implementation before wiring
native UI operations. The sibling VibeScript tooling's descriptor export is
explicitly discovery-only; it does not register providers or grant authority.

## Inspected baseline

### Chora embedding boundary

On 12 November 2026, the Chora fork was found at `C:/github/chora`, alongside
Aether, not inside Aether's workspace or a `vendor/` directory. Its root
`AGENTS.md` requires an independent engine boundary and generic host interfaces;
it prohibits dependencies on `qualia-*`/`webizen-*` crates and copying ecosystem
code into Chora. The QualiaDB-based host supplies that integration externally.

Recommended integration layout: keep the Chora fork as a separate source workspace
and implement the host adapter in a future `packages/aether-chora` crate. A pinned
vendored snapshot may later serve offline release builds; moving or duplicating
the fork is not needed to establish the adapter boundary. No adapter crate,
dependency, vendor copy or submodule has been added by this inspection.

| Observed source | Integration consequence |
| --- | --- |
| `chora/LICENSE` and workspace metadata identify MPL-2.0 | Preserve the fork's notices and third-party licensing inventory separately from Aether's own licence |
| `components/servo/Cargo.toml` names the package and library `servo` | Alias it as `chora` in host code while explicitly selecting the fork path or pinned fork Git revision; a registry alias alone does not select this fork |
| `components/servo/lib.rs` exports `Chora`, `ChoraBuilder`, `ChoraDelegate`, `WebViewBuilder`, `WebViewDelegate` and rendering contexts | Use the actual Rust embedding interfaces for wakeups, input, frame delivery and teardown |
| `embedded_library` includes `js_jit`, and `script` remains a dependency | Preserve JavaScript support for traditional web compatibility; these features do not establish a native VibeScript application path |
| `embedded_library` omits `media-gstreamer` and includes bundled fonts/resources | Verify the resolved build profile and release notices; the feature name alone is not a dependency-licence audit |
| `embedded_library` does not itself enable `multiprocess` | Do not infer process isolation from README descriptions; qualify the selected build and runtime configuration |
| `ffi/capi/Cargo.toml` defines a separate `cdylib` with its own feature choices | C ABI packaging needs separate verification from the Rust embedding profile |

Chora's native rendering contexts have not been demonstrated to interoperate with
Aether's Etalon-backed resources. Verify surface ownership, event loops and GPU
interop before choosing shared textures or another composition strategy. Preserve
the existing native renderer while qualifying Chora. VibeScript execution must
use QualiaDB's SDK through the host boundary; the engine's JavaScript evaluator
must not silently become a VibeScript evaluator.

### Traditional web and human-centric works

Both execution paths are required product capabilities. They must coexist without
making traditional JavaScript/DOM execution a prerequisite for every native work.

| Work type | Intended execution | Dependency policy |
| --- | --- | --- |
| Traditional websites and web applications | Chora's web engine, browser APIs and JavaScript runtime | Preserve compatibility and validate representative applications |
| QualiaDB/Webizen human-centric works | QualiaDB foundation, VibeScript SDK and Etalon-backed native integration | Prefer canonical ecosystem interfaces; introduce historical dependencies only for a documented compatibility need |
| Mixed works embedding traditional content | Native host with a bounded Chora web surface | Explicit messaging and host-authorized operations; do not grant web content ambient access to QualiaDB or host capabilities |

The current Chora manifest selects `mozjs` (SpiderMonkey), not V8. Distinguish
ordinary JavaScript/browser compatibility from engine-specific V8 interfaces,
Node.js APIs and native extensions. Determine which of those APIs actual target
applications need before proposing another runtime or claiming they are supported.

The immediate objective is to avoid unnecessary use of historical execution paths
in human-centric works, not to remove JavaScript support from Chora. Separately
assess build-time inclusion, runtime activation and application API dependencies;
an embedded JavaScript engine in a compatibility-capable distribution does not
mean every native work must execute through it. Minimal packaging profiles can be
qualified later without weakening the traditional-web profile.

Validate both paths: traditional web applications must continue to function, while
native works must demonstrate execution through the QualiaDB/VibeScript SDK path.
Test mixed-content bridges for explicit authority and component/window teardown.

This is a source inspection only. Chora compilation, runtime embedding, dependency
licences, dual-path execution and production packaging have not been verified.

### Aether native baseline

| Area | Evidence | Remaining gap |
| --- | --- | --- |
| Native renderer | `packages/native/src/lib.rs`, `aether_renderer.rs` | Build and run the selected renderer on each supported OS; establish GPU failure handling |
| Etalon dependency | Root `Cargo.toml` pins Etalon and selects hypermedia-production adapters | Verify the resolved graph, custom-resource interop and device lifecycle together |
| Script execution | `packages/native/src/contexts.rs` delegates `eval` to `NoOpDocument`; QualiaDB already supplies the VibeScript SDK | Connect Aether to the existing SDK and engine; head script elements are not evidence of execution |
| QualiaDB foundation | No QualiaDB/Vibe dependency found in Aether's inspected manifests | Establish the required runtime dependency through canonical SDKs, including authorized query/subscription/update integration |
| Input coverage | `packages/native-dom/src/events.rs` has `unimplemented!` converters | Establish supported event coverage and remove reachable panic paths |
| Window lifecycle | `packages/native/src/aether_application.rs` initializes the first window separately from `add_window` | Audit context provisioning and teardown across auxiliary windows |
| Renderer features | Renderer selection and GPU-option `cfg` predicates differ in `aether_renderer.rs` | Check combined features as well as individual backends |
| Lens state | `refract/src/ui.rs` uses `Vec` collections alongside fixed-array lenses | Measure named hot paths; connect to native rendering before claiming end-to-end gains |
| Public lens API | This change adds optional `aether::refract` export | Requires Rust 1.88+; Aether's facade still needs a successful dependency-resolved build |
| CI and distribution | `.github/workflows/main.yml` contains workspace tests and desktop bundle jobs | Add explicit native renderer matrix, packaged native launch and runtime evidence |

The optional `refract` export does not replace the VirtualDOM scheduler, provide a
QualiaDB connection, or execute VibeScript.

## Ordered work and release gates

The continued source audit and detailed implementation packets are recorded in
[`../audits/2026-11-12/README.md`](../audits/2026-11-12/README.md), with separate
Aether and Chora findings, plans and verification receipts. These extend the gates
below; they do not mark them complete.

**Mandatory GPU gate:** Etalon is the GPU and compute implementation, with
substantial refactoring and additional capabilities including work moved from
QualiaDB. Current Aether, Chora and maintained renderer implementation names are
being migrated to Etalon. Their resolved GPU implementations already use Etalon;
upstream wgpu packages remain prohibited by the source gates. Align Etalon
revisions across QualiaDB, Aether, Chora and hypermedia-production, verify each
shipping graph, and preserve traditional JavaScript/WebGPU compatibility.
Disabling compatibility features does not complete their integration.

**Current release blockers:** required QualiaDB/VibeScript host wiring, native
feature/lifecycle/input defects, scoped assets and local-service authorization,
Chora embedding/crypto configuration wiring, platform isolation and native packaged
runtime evidence. Track closure with links to executable evidence; do not infer
readiness from metadata enumeration or the isolated Refract tests.

1. **Reproducible native baseline.** Resolve the locked dependency graph in a clean
   writable environment. Check native DOM, the default hybrid renderer, GPU and CPU
   alternatives, combined feature selections, and the public facade. Record compiler,
   target, dependency revisions and failures. Verify minimum Rust versions separately.
2. **Native lifecycle correctness.** Exercise first and auxiliary windows, input,
   focus, IME, clipboard, accessibility, resizing, scale changes, suspend/resume and
   shutdown. Test resource cleanup and GPU initialization/device-loss failures.
   Unsupported capabilities need explicit behavior rather than reachable panics.
3. **QualiaDB foundation and VibeScript SDK integration.** Wire the required
   QualiaDB runtime through its existing engine/host and provider SDK interfaces.
   Implement the Aether-specific bindings for native execution and reuse canonical
   authority, resource and lifecycle contracts. Verify permitted and rejected
   effects, bounded work, cancellation, diagnostics and script reload without
   rebuilding the application. Keep language implementation in QualiaDB.
4. **QualiaDB-to-UI integration.** On that required foundation, deliver a native example that reads actual data,
   subscribes to changes, invokes an authorized VibeScript operation, and updates the
   UI. Verify error propagation, bounded update queues and subscription cancellation
   when a component or window closes.
5. **Etalon integration and performance.** Exercise a custom GPU resource alongside
   UI rendering. Measure frame latency, memory, allocations in named hot paths,
   streaming updates and resource recovery on representative devices. Define budgets
   for actual workloads before making performance claims.
6. **Native distribution.** Build, package and launch native applications on Windows,
   macOS and Linux with required assets and fonts. Add platform-specific mobile
   evidence before declaring mobile readiness. Establish signing, upgrades,
   diagnostics and preserved license/provenance notices for release artifacts.

## Initial verification commands

Run from the repository root with writable Cargo caches and available dependencies:

```sh
cargo check --locked -p aether-native-dom
cargo check --locked -p aether-native
cargo check --locked -p aether-native --no-default-features --features vello,svg
cargo check --locked -p aether-native --no-default-features --features vello-cpu-softbuffer,svg
cargo check --locked -p aether --features native,refract
cargo test --locked -p refract
```

These are starting checks, not the complete release matrix. Compilation does not
verify window interaction, visual output, VibeScript execution or packaging.

The initial `cargo check -p aether-native --locked --offline` stopped during
dependency checkout because the default Cargo cache was not writable. No native
compile or launch success was established by that attempt.
A second attempt to check `aether --features native,refract` with a writable,
isolated Cargo home also stopped before compilation: the locked
hypermedia-production checkout was not available in that offline cache.

Verification of this change: Cargo metadata with `--no-deps --offline` confirmed
that the `refract` feature enables the optional local dependency. An unchanged copy
of the dependency-free `refract` sources and existing tests was built in an isolated
workspace under `target/refract-verification`: all 24 integration tests and one
doctest passed; one existing doctest was ignored. This verifies the lens runtime,
not compilation of the Aether facade or the native rendering stack.

## 14 November resumption evidence

The separate Chora process transport, parent canonical read adapter and native
copied-frame widget are now implemented locally; see
[the worker packet](../implementation/resume-worker.md). Seven protocol tests,
eleven parent state tests and three real private-pipe process tests pass. The actual
native CPU+prelude+Chora feature compiles. These tests use an explicit transport
peer and do not prove Chora worker runtime. The exact saved post-naming hybrid
lifecycle executable now runs successfully, including three redraws and owner
teardown; its source/executable receipt remains distinct from the new feature.
Full Chora embedded and optional canvas/profiler compilation also now passes.
Current live host tests pass 34/34 and source-policy tests pass 73/73. Full frozen-v4
source verification passes, while its execution remains a separate gate. Current
live profile generation rejects the changed canonical Etalon overlays; no source
identity substitution is accepted to bypass that gate.
Containment, actual worker/native pixels, input and installed production evidence
remain mandatory. Existing historical baseline sections are retained above.

The resumed source-policy suite now passes86/86, with exact two-overlay admission,
source/receipt drift rejection and staging ancestry checks. A separate current
Qualia workspace change redirects14 Etalon packages including GPU packages; the
actual live profile is still rejected before output. The isolated overlay Cargo
metadata check does not qualify that broader live graph. No frozen bytes changed.

The actual separate worker subsequently completed its full Windows development
build (96m42), using the digest-verified supported SpiderMonkey archive input.
The consolidated [14 November handoff](../implementation/HANDOFF-2026-11-14.md)
records exact artifacts, qualification limits and 51 ordered remaining jobs.
Real worker page/pixel execution, native frame display and release qualification
remain open; the successful build does not close those gates.
Final exact development staging failed because the required root `LICENSE-QUALIA`
file is absent. Its partial artifacts are preserved without a completed receipt;
resolve the authoritative notice layout before accepting a stage or running the
prepared worker runtime probes.

## 15 November continuation plan

The [work packets](../implementation/WORK-PACKETS-2026-11-15.md) allocate the
remaining native/host/browser/media/release gates and add explicit streaming
acceptance for format readiness, transfer retry/completion, selected ranges,
scientific interpretation and actual audio deadlines. They are prospective work,
not additional qualification evidence. Sibling-dependent execution waits for the
user's committed-fix notification and the
[upstream integration register](../implementation/UPSTREAM-INTEGRATION-REGISTER.md).


## Independent preparation evidence — 15 November 2026

[Preparation results](../implementation/preparation-2026-11-15/README.md) record
Aether-only inventories/specifications and read-only verification of all 4,382
frozen-v4 inputs/nine package identities with unchanged receipt. They do not
qualify implementation or runtime. The user's current shared-license policy is
recorded; missing LICENSE-QUALIA was added byte-identical to existing LICENSE.
The old partial stage remains incomplete. Sibling-dependent implementation,
builds and runtime/device tests remain held until user notification and intake.

## Committed sibling intake — 15 November 2026

The user has now authorized implementation and confirmed committed sibling fixes.
[Intake 1](../implementation/UPSTREAM-INTEGRATION-REGISTER.md) verifies QualiaDB
`4ec9ab3`, Etalon `057545f` and Hypermedia `a01384e` locally and on GitHub. A new
explicit live consumer profile is being qualified; earlier root, frozen-v4 and
separate-worker evidence retains its original source identities. Canonical
complete-entry EMF/QAC readers are available, with the six recorded mixed-driver,
schema and audio-qualification constraints. No progressive/playback claim follows.

The current isolated worker passed its 762-package source gate, cached build and
exact development staging. Bounded client stderr retention now passes 13 unit and
three process tests. Initial restricted probes failed; the same actual staged
worker subsequently passed approved localhost caller/host-denial/64x64 RGBA and
three supplemental page-window WebCrypto cases. These do not qualify native-window
composition, an authorized Qualia bridge, full WPT or OS containment. See
[worker intake evidence](../implementation/INTAKE-WORKER-2026-11-15.md).
This releases concrete continuation work; it does not close native production,
containment, hardware, installed-artifact or release gates.

The live combined native/Qualia host profile subsequently passed full resolved
metadata/source verification (673 Windows packages, 25 canonical GPU packages)
and `cargo check --lib --locked --offline` in 37m09s. This is type checking, not
linkage, host unit tests or an application runtime pass. Its post-run source gate
failed on 28 concurrently changed QualiaDB inputs; no single immutable source
qualification follows. Preserve that evidence and refresh the receipt for the
next execution. [Intake 1](../implementation/UPSTREAM-INTEGRATION-REGISTER.md)
records exact artifacts, source drift and the isolated-lock standalone host test
procedure. Root shipping pins and frozen v4 remain unchanged.
