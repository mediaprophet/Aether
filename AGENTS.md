# Aether — Agent Coordination & Licensing Guide

## Intellectual Property & Licensing Notice

QualiaDB, Aether, and all Qualia-specific architectures, integrations, modifications, and extensions are governed by:

**Copyright (c) 2026 Timothy Charles Holborn**  
**https://www.linkedin.com/in/ubiquitous/ | timothy.holborn@gmail.com**  
**License:** Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)  
Full terms: [`LICENSE`](LICENSE), [`LICENSE-QUALIA`](LICENSE-QUALIA), and [Creative Commons Legal Code](https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode).  
Upstream historical code retains its respective Apache-2.0 and MIT licensing as documented in [`upstream-licenses/`](upstream-licenses/).

All participating AI agents and contributors must strictly honor this copyright, moral rights, and non-commercial attribution. Unlawful extraction, stripping of provenance, or unauthorized commercial exploitation is strictly prohibited.

---

# Aether Agent Guide

## Project purpose and priorities

Aether is a fork of Dioxus being overhauled for QualiaDB and Webizen. Prioritize
production native applications through `packages/native` and `packages/native-dom`,
with Etalon and the hypermedia-production rendering stack. The Wry/WebView desktop
backend remains a separate compatibility path; a successful WebView build does not
establish native readiness.

Etalon is the required GPU implementation. Migrate upstream wgpu dependencies,
including transitive and optional renderer paths, to the reviewed Etalon stack.
Check resolved package sources as well as names. Aether-owned dependencies and
imports use `etalon`; retain old names only at documented compatibility or
historical-provenance boundaries. Etalon is the substantive GPU and compute
implementation, including architecture and capabilities moved from QualiaDB.
Align revisions with QualiaDB and hypermedia-production
before sharing devices or resources. The old-web (Servo-derived) engine's WebGPU API must retain traditional
web behavior while its underlying implementation migrates to Etalon.

The entire ecosystem depends on QualiaDB. Treat QualiaDB as the required foundation
of the target application architecture, not an optional database integration.
Missing dependency wiring in this fork is an overhaul gap, not an architectural
choice to make QualiaDB optional. Reuse its canonical SDKs and contracts across
the UI, scripting and rendering integration.

Webizen uses VibeScript for application scripting in place of JavaScript/V8.
This applies to works authored for the QualiaDB/Webizen human-centric web.
The old-web engine must also retain traditional web compatibility, including JavaScript web
applications. Do not remove that compatibility globally to achieve the native
application profile. Human-centric works should use canonical QualiaDB/VibeScript
and Etalon interfaces, introducing historical web dependencies only sparingly for
an identified compatibility need. Keep compatibility execution and privileged
ecosystem operations distinct, with explicit host-authorized bridges.
The canonical language engine is QualiaDB's `crates/vibe`; the sibling `vibe-script`
repository supplies tooling. Integrate the existing engine and host contracts;
do not duplicate the engine or assume JavaScript evaluation is a VibeScript API.
VibeScript already has an SDK: inspect QualiaDB's `crates/vibescript-abi`, including
`src/sdk.rs`, and the `vibe` engine's public embedding interfaces before designing
integration code. Reuse provider descriptors, lifecycle and memory contracts;
do not invent a competing extension ABI or infer execution from tooling discovery.
Keep script execution, host authority, UI scheduling, and GPU resource ownership
explicit. Preserve the Dioxus attribution of inherited code.

The sibling `C:/github/chora` repository is **webizen-WWW-engine**
(Servo-derived, MPL-2.0). Keep its source and workspace separate and follow its
local `AGENTS.md` when working there. **Chora** in QualiaDB is the
spatio-temporal commons canvas (Ring B). Do not redefine that product as Servo
updates. GitHub is still `WebCivics/chora` until renamed to
`webizen-www-engine`. Ring B's future anti-slavery licence does not relicense
this engine; composition is a sibling worker built **from the engine tree**.
If that licence cannot sit beside MPL as a separate program, drop the engine —
do not mix files. **Known leak:** `packages/chora-worker` path-depends on
`servo` while declaring CC-BY-NC-ND; E0 closes it. `aether-chora-client` and
`aether-chora-bridge-protocol` already stay on the Ring B side of the wire.

The engine's generic embedding interfaces are the boundary: QualiaDB/VibeScript host
integration belongs on the Aether/Webizen side, without adding ecosystem-specific
dependencies or copying ecosystem code into the engine. The required QualiaDB foundation
is supplied by the host application. Preserve Servo/MPL attribution on Covered Software.
The engine's Cargo package name remains `servo`, so select the actual tree explicitly
rather than assuming a dependency alias selects it.
The engine currently uses SpiderMonkey. Standard JavaScript web compatibility and
support for V8-specific or Node.js interfaces are separate requirements; identify
the actual API requirements before adding a runtime or promising compatibility.

Track evidence and outstanding release gates in
[`notes/architecture/13-NATIVE-PRODUCTION.md`](notes/architecture/13-NATIVE-PRODUCTION.md).
Do not present planned integration, compilation alone, or WebView tests as proof
that the native QualiaDB/VibeScript application path is ready for production.

Aether is a QualiaDB-native GUI toolkit (the Dioxus / Tauri analogue for this stack). It is not Webizen, not QualiaDB, and not a shipped application. Host apps — `webizen-os-shell`, Synth operator UI, later Qaps — consume Aether. Do not ship Aether defaults (window title “Aether App”, fallback tray, demo icons) as product identity.

It compiles to Web (WASM), Desktop (WebView2/Tao with WebGPU compatibility shims), Mobile (iOS/Android), and Native GPU (Blitz + AnyRender + Etalon).

Two webs, two products, two engines:

- **Human-centric web** — QualiaDB, VibeScript, Aether native RSX, Qaps. **Chora** in this ecosystem is the spatio-temporal commons canvas (`qualia://chora/universe`, Qualia `chora/` module). It is not an HTML engine and is not MPL.
- **Old web** — HTML/CSS/JS. Product is the **Webizen Web Browser** Qap. Intended engine is the **Servo-derived MPL tree** (independent repo, currently named `WebCivics/chora` — that name is a defect). Employ as a sibling worker. Not WebView, not Wry, as the intended engine.

Aether is the toolkit: native RSX for human-centric chrome, copied-frame worker for the old-web engine. Blitz and Servo/Stylo cannot share a process. Never copy Qualia types into the MPL engine. Do not redefine **Chora** as “Servo updates”.

B1a is native window lifecycle (engine feature off) so human-centric chrome can paint. B1b + worker artefact employs the old-web engine for the Web Browser. QualiaDB remains the required application foundation.

## Quick Overview

- **Language**: Rust (2024 edition, `1.85.0+`)
- **Primary Crates**: `aether`, `aether-native`, `aether-desktop`, `aether-web`, `refract`
- **UI Model**: Reactive VirtualDOM with components, hooks, copyable signals, plus `refract` zero-heap lens reactivity
- **Syntax**: `rsx!` macro for declaring UI (`use aether::prelude::*;`)
- **Platforms**: Web, Desktop (Windows/macOS/Linux), Mobile, Native GPU, LiveView

## Workspace Structure

| Path | Role |
| --- | --- |
| `packages/aether` | Public facade, including optional `refract` export |
| `packages/native`, `packages/native-dom` | Native window renderer and headless DOM |
| `packages/desktop` | Wry/Tao WebView backend |
| `packages/web` | Browser/WASM renderer |
| `packages/core`, `packages/rsx` | VirtualDOM and RSX syntax |
| `packages/signals`, `packages/hooks`, `packages/stores` | Reactive state |
| `packages/html`, `packages/router` | Element vocabulary and routing |
| `packages/fullstack`, `packages/ssr`, `packages/liveview` | Server rendering and streaming |
| `packages/cli`, `packages/devtools`, `packages/subsecond` | Builds, development tools and hot patching |
| `packages/manganis`, `packages/asset-resolver` | Assets |
| `packages/interpreter`, `packages/wasm-split` | JavaScript DOM interpreter and WASM splitting |
| `refract` | Experimental store/lens runtime |
| `.github/workflows`, `upstream-licenses` | CI and preserved upstream licensing |

## Architecture Documentation

For deeper understanding, see `notes/architecture/`:

| When working on...                         | Read...            |
| ------------------------------------------ | ------------------ |
| VirtualDOM, components, diffing, events    | `01-CORE.md`       |
| CLI, build system, bundling, dev server    | `02-CLI.md`        |
| RSX macro, parsing, formatting             | `03-RSX.md`        |
| Signals, state management, reactivity      | `04-SIGNALS.md`    |
| Server functions, SSR, hydration           | `05-FULLSTACK.md`  |
| Web/desktop/native/liveview renderers      | `06-RENDERERS.md`  |
| Hot-reload, hot-patching, devtools         | `07-HOTRELOAD.md`  |
| Asset macro, manganis, const serialization | `08-ASSETS.md`     |
| Router, navigation, nested routes          | `09-ROUTER.md`     |
| WASM code splitting                        | `10-WASM-SPLIT.md` |

## Common Patterns

**Component definition**:
```rust
use aether::prelude::*;

#[component]
fn MyComponent(name: String) -> Element {
    let mut count = use_signal(|| 0);
    rsx! {
        button { onclick: move |_| count += 1, "{name}: {count}" }
    }
}
```
