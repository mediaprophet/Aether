# Aether audit

## Assessment

Aether has a real native rendering path and an existing Etalon-backed dependency
graph. It does not yet implement the required end-to-end QualiaDB/VibeScript host
or the authorized mixed-site Chora integration. Native correctness and release
coverage also have concrete gaps. WebView success would not close these gaps.

Priorities below mean: **P0** blocks exposing local privileges to untrusted browser
content; **P1** blocks the intended native production release; **P2** is a quality,
maintainability or distribution gap. Source findings are not represented as
reproduced runtime failures when compilation was unavailable.

## Findings

| ID | Priority | Evidence | Finding and consequence |
| --- | --- | --- | --- |
| A01 | P1 | Root/package manifests; `packages/native/src/contexts.rs`, `NativeDocument::eval` | Required QualiaDB host and VibeScript integration are absent from the inspected native path. Eval delegates to `NoOpDocument`; this is not VibeScript execution. |
| A02 | P1 | Root `Cargo.toml`, `Cargo.lock`; QualiaDB `Cargo.toml` | Aether already aliases wgpu to Etalon, but its pinned revision differs from QualiaDB. Combining them can duplicate Rust type identities and resource owners even at the same package version. |
| A03 | P0 before browser exposure | `packages/asset-resolver/src/native.rs`, `resolve_asset_path_from_filesystem`, `ResolvedAsset::into_response` | Existing absolute paths are accepted, fallback joins lack canonical root containment, and responses allow wildcard CORS. Invalid UTF-8 uses `expect`; a nonexisting relative path can reach `strip_prefix("/").unwrap()`. Do not expose this trusted-app resolver as an untrusted page's local file service. |
| A04 | P1 | `packages/native/src/aether_renderer.rs`, renderer selection and `RendererOptions` | Renderer priority selects CPU/Skia ahead of default hybrid, while GPU fields/options use `any(vello, vello-hybrid)`. Default hybrid plus CPU/Skia can select a renderer without the imported GPU types subsequently referenced. Source-level cfg inconsistency; compiler reproduction blocked. |
| A05 | P1 | `packages/native/src/event_handlers.rs`, `apply_event`, `add`, `remove`; `hooks.rs` | Dispatch holds a mutable `RefCell` borrow while invoking callbacks. Callback removal/addition can reborrow and panic. Define callback-safe registration/removal and ordering. |
| A06 | P1 | `packages/native/src/aether_application.rs`, `add_window`, `can_create_surfaces` | Initial-window setup injects Aether contexts and performs the initial build. The public additional-window path delegates directly to the inner application. Audit and unify preparation; auxiliary-window parity is not established. |
| A07 | P1 | `packages/document/src/document.rs`, `NoOpDocument::eval`; `eval.rs`; `packages/generational-box/src/lib.rs`, `OwnerInner::drop` | A temporary generational owner is dropped on return, recycling the evaluator slot. Subsequent access can report `Finished` instead of the intended unsupported evaluation result. Repair lifetime/error semantics without treating eval as VibeScript. |
| A08 | P1 | `packages/native-dom/src/events.rs`; `aether_document.rs`, `DomEventData::Ime` | Multiple conversion methods remain `unimplemented!`; IME events are discarded. Reachability differs by event/backend, but general native input and DOM parity cannot be claimed. |
| A09 | P1 | `packages/native/src/lib.rs`, runtime/event-loop/window creation; renderer lifecycle implementation | Several launch paths unwrap failures. Result-based startup, device failure, suspend/resume, teardown and recovery contracts need implementation and platform evidence. |
| A10 | P1 | `packages/native/src/assets.rs`, provider and asset-response handling | Asset errors can be reduced to optional tracing; HTTP status is not carried through the byte callback. A 404 response body can be treated as asset bytes. Define cancellation, status and error propagation. |
| A11 | P1 | `packages/native/Cargo.toml`; `src/lib.rs` WASM setup; `src/assets.rs` | WASM/net guards require feature-target checks; provider construction and feature availability are not consistently guarded. Strong `blitz-html/tracing` and `blitz-net/tracing` feature edges can enable optional dependencies unexpectedly. |
| A12 | P0 before local services | Native navigation handler; `packages/desktop/src/webview.rs`, `edits.rs` | Native links generally open an external browser; the WebView backend is an app renderer with its own transport. Neither is an origin/frame-scoped QualiaDB service broker or a mixed-site browser implementation. The existing desktop transport does use keys; this is not a finding of unauthenticated transport. |
| A13 | P1 | `.github/workflows/main.yml`, path filter, all-features checks and bundle jobs | Path filters omit `Cargo.lock` and `refract/`. All-features builds can hide A04 by selecting Vello first. Desktop packaging jobs do not establish the native renderer's runtime behavior. |
| A14 | P1 | Manifests, toolchains, `refract/Cargo.toml`, GPU dependencies | Declared minimum Rust versions and CI compiler versions differ across the stack. The public facade's nominal baseline cannot stand in for a verified native profile MSRV. |
| A15 | P2 | `refract/src/ui.rs`, existing tests; native DOM tests | Refract includes `Vec` state and is not wired as an end-to-end replacement scheduler. Isolated test success supports its own behavior, not zero-allocation application rendering or GPU performance claims. |
| A16 | P1 release packaging | Root `LICENSE`, AGENTS links, `upstream-licenses/`, `packages/aether/README.md` | Referenced `LICENSE-QUALIA` is missing; root license text is abbreviated. Crate docs include mechanically renamed domains/install references requiring verification. Preserve inherited attribution and ship complete, accurate notices and instructions. |

## Required foundation and SDK reuse

The canonical language engine is `C:/github/qualiaDB/crates/vibe`; the sibling
vibe-script repository is tooling. The SDK already exists in
`crates/vibescript-abi/src/sdk.rs`. It supplies provider/function descriptors and
checked data construction. `descriptor.rs` defines `VibeNamespaceProvider::call`
with borrowed input slices, caller-owned output and `VibeTrap`. `lifecycle.rs`
defines admission, post-execution validation, receipts, actor/context information,
policy generations and resource budgets.

The engine exports `Host`, `Engine`, `Budget`, `Env` and program evaluation APIs.
`extension_registry` has prepare/dispatch APIs and execution context. Its
`SimulatedPolicyGate` is not production authorization. `web_runtime.rs` already
contains UI/form/input/layout support and CBOR-LD UI patch operations. Reuse these
interfaces and test their actual semantics before inventing adapters or protocols.

QualiaDB's root manifest already references Aether. Simply adding a dependency on
the entire QualiaDB application from Aether risks a cycle. Put required host
composition above the reusable UI framework and depend on canonical narrow runtime,
ABI and data contracts. This preserves the mandatory QualiaDB application
foundation without forcing every low-level component to depend on the top-level app.

## GPU interpretation

Aether's dependency key `wgpu` resolves to `package = "etalon"` version 0.0.2,
revision `575bab54accf1aafb24f67e9ba810c0f3f745aea`. The lockfile's Etalon family
and its Naga dependencies share that source revision. Renderer adapters resolve
from hypermedia-production, locked to a commit beginning `a46b4e2b`.
QualiaDB's inspected Etalon and related declarations instead use
`6ace6cc6413a87ca10a1da58397ddc7c0b2fc41e`.

Renaming the import is insufficient and unnecessary for proving migration. Select
a coherent reviewed source revision, check every active target/feature graph, and
verify whether devices, queues and textures can actually be shared. One source
identity is necessary for direct Rust sharing but does not establish correct GPU
ownership or synchronization. Etalon's compatibility package named `wgpu` is not
the same as an upstream registry implementation; any allowed shim needs explicit
source and revision validation.

## Existing foundations and limits

Native DOM tests already cover removed nodes, suspended rendering, stress,
document access from tasks and mounted-node removal. The CLI contains a native
renderer build harness (`packages/cli/src/test_harnesses.rs`); native support is
not entirely absent from development tooling. Existing renderer methods forward
lifecycle operations to the selected backend. Build on these paths rather than
replacing them indiscriminately.

The isolated, unchanged Refract test copy passed 24 integration tests and one
doctest, with one doctest ignored. Native builds, GPU execution, accessibility and
packaged launch were not validated. See [verification](VERIFICATION.md) for the
dependency failures and [implementation plan](AETHER-IMPLEMENTATION-PLAN.md) for
acceptance criteria. No remote exploit was demonstrated for the asset finding;
its P0 classification concerns the planned untrusted-content boundary.
