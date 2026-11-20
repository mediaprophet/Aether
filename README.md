# Aether — Sovereign Application Framework for QualiaDB & Webizen

> High-performance cross-platform application framework with native Etalon/WebGPU acceleration, zero-heap lens reactivity (`refract`), spatial window ergonomics, and unified desktop, web, and GPU-native rendering.

[![CI](https://github.com/mediaprophet/Aether/actions/workflows/main.yml/badge.svg)](https://github.com/mediaprophet/Aether/actions/workflows/main.yml)
[![License: CC BY-NC-ND 4.0](https://img.shields.io/badge/License-CC_BY--NC--ND_4.0-lightgrey.svg)](LICENSE-QUALIA)
[![Ecosystem: QualiaDB](https://img.shields.io/badge/Ecosystem-QualiaDB-blue.svg)](https://github.com/mediaprophet/qualiaDB)
[![Platform: Desktop_|_Web_|_Native](https://img.shields.io/badge/Platforms-Desktop_%7C_Web_%7C_Native-orange.svg)](#architecture--workspace-structure)

**Aether** is a fork of Dioxus being overhauled into the native application framework for [Webizen OS-Shell](https://github.com/WebizenAI/webizen-os-shell) and [QualiaDB](https://github.com/mediaprophet/qualiaDB). Production native applications are the primary development target.

The entire ecosystem depends on **QualiaDB**. Aether's target native application
architecture uses that required foundation for Aether UI, Etalon-backed rendering,
and VibeScript application scripting in place of JavaScript/V8. VibeScript already
has an SDK; its engine and canonical extension ABI belong to QualiaDB. Aether must
reuse those interfaces. This is ongoing work:
the native document evaluator currently reports unsupported execution, and this
repository does not yet provide the complete QualiaDB/VibeScript application bridge.
See the [native production plan](notes/architecture/13-NATIVE-PRODUCTION.md) for
the inspected baseline and release gates.

Chora, the sibling web engine fork, must retain traditional website and JavaScript
application compatibility. Works authored for the QualiaDB/Webizen human-centric
web should use the native ecosystem interfaces and VibeScript, introducing
historical web dependencies only sparingly for an identified compatibility need.
Embedded traditional content connects to host capabilities through explicit,
authorized interfaces.

Engineered for demanding spatial, cognitive, and zero-heap hot-path requirements, Aether unites the ergonomics of modern reactive component programming with the zero-overhead determinism of native Rust systems programming.

---

## Strategic Innovations & Core Pillars

### 1. Built-in WebGPU Acceleration & Limits Compatibility Shim
Embedded webviews and browser runtimes frequently suffer from silent WebGPU failures, missing Vulkan/ANGLE flags, or breaking changes in the W3C WebGPU specification:
- **Out-of-the-Box WebView2 GPU Enablement**: `aether-desktop` automatically configures Windows WebView2 with `--enable-unsafe-webgpu`, `--enable-features=Vulkan,DefaultANGLEVulkan,VulkanFromANGLE`, `--use-angle=vulkan`, and `--ignore-gpu-blocklist`. Hardware-accelerated shaders and compute kernels run without manual registry tweaks or fragile shell wrapper scripts.
- **WebView device-limit compatibility**: When WebGPU is enabled for a desktop WebView, Aether filters unrecognized device limits against the browser adapter before forwarding `requestDevice()`. This supports Etalon applications using the browser WebGPU path.
- **Adaptive Fallback Policy**: Supports multi-tier adapter discovery (Default $\to$ Compatibility $\to$ Low-Power $\to$ Software), ensuring robust execution across discrete GPUs, integrated graphics, and mobile silicon.

### 2. Zero-Heap Reactive Store & Lens Runtime (`refract`)
Standard UI frameworks rely heavily on heap allocations (`Vec`, `String`, `Box`) and runtime borrow counting (`RefCell`), causing allocator contention and cache misses:
- **Store/lens reactivity**: The experimental `refract` runtime is available as `aether::refract` with the optional `refract` feature (Rust 1.88+). It uses split borrows instead of `RefCell` for state access. Its scheduler still uses heap-backed collections; allocation-free hot paths require workload-specific measurement.
- **Fixed-Buffer Array Projections ([`ArrayLens`](refract/src/lens.rs))**: UI components can focus directly on fixed-size stack arrays (`[T; N]`) and 48-byte Super-Quins (`[NQuin; N]`) with element-level invalidation granularity. Updating a single slot wakes only its direct observers without allocating a single byte on the heap.
- **Separate reactive runtime**: `refract` tracks lens dependencies independently of Aether's VirtualDOM signals. Connecting those updates to native rendering and establishing streaming performance remain integration and measurement work.

### 3. Spatial Desktop Shell & Native Window Lifecycle
Designed to serve as the runtime foundation for the Webizen OS-Shell halo chrome:
- **Glassmorphism & Frameless Ergonomics**: Native support for Windows Acrylic, Mica, full alpha transparency, click-through regions, and non-rectangular custom window geometries.
- **System Tray & Auxiliary Window Coordination**: Built-in tray lifecycle management (`HideToTray`, `QuitShell`, restore) and lightweight auxiliary window spawning for companion tools (Poet, Studio, Settings, System Monitor).
- **Direct GPU Viewport Integration**: `aether-native` (Blitz + AnyRender + Etalon) exposes `custom-widget` hooks for renderer-specific Etalon resources. The combined PortalGpu/Forge path, shared resource ownership and performance are qualification work tracked in the [native production plan](notes/architecture/13-NATIVE-PRODUCTION.md).

---

## Architecture & Workspace Structure

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

---

## Quickstart

### 1. Native Application with Etalon-backed Rendering

Add Aether to your `Cargo.toml`:

```toml
[dependencies]
aether = { git = "https://github.com/mediaprophet/Aether.git", branch = "main", features = ["native", "refract"] }
```

Launch through the native renderer. The separate `desktop` feature selects the Wry/WebView backend. The `refract` feature enables the state example below; omit it if unused.

```rust
use aether::prelude::*;

fn main() {
    aether::launch(App);
}

#[component]
fn App() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; background: #030712; color: #f9fafb; font-family: sans-serif;",
            h1 { "Aether Spatial Host" }
            p { "Active Count: {count}" }
            button {
                style: "padding: 8px 16px; border-radius: 8px; background: #4f46e5; color: white; border: none; cursor: pointer;",
                onclick: move |_| count += 1,
                "Increment"
            }
        }
    }
}
```

### 2. Zero-Heap Reactive State (`refract`)

For fixed-size state projections, enable the `refract` feature as shown above.
This example demonstrates granular updates, not a whole-runtime allocation guarantee:

```rust
use aether::refract::{Ui, lens, ArrayLens};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct QuinSlot {
    subject: u64,
    predicate: u64,
    object: u64,
}

struct CoreState {
    slots: [QuinSlot; 8], // Zero-heap fixed-size buffer
}

fn main() {
    let mut ui = Ui::new(CoreState {
        slots: [QuinSlot::default(); 8],
    });

    let slots = lens!(CoreState => 0: slots);
    let slot0 = slots.at(0); // Element-level lens projection (zero heap allocation)

    ui.effect(move |ctx| {
        let s = ctx.get(slot0);
        println!("Slot 0 updated: subject = {}", s.subject);
    });

    ui.with(|ctx| {
        ctx.write(slot0).subject = 42; // Wakes only slot 0 observers
    });
}
```

---

## Building & Verification

### Desktop Build:
```bash
cargo build -p aether --features desktop
```

### Native GPU Renderer Build:
```bash
cargo build -p aether-native
```

### Run `refract` Zero-Heap Suite:
```bash
cargo test -p refract
```

### Web / WASM Verification:
```bash
cargo check -p aether-web --target wasm32-unknown-unknown
```

---

## Upstream & Attribution

Aether is an independent project founded and maintained by Timothy Charles Holborn.
Its starting source was a fork of the Dioxus `0.8.0-alpha.3` snapshot under the MIT and Apache-2.0 licenses. Some foundational VirtualDOM, reactive signal, RSX macro, and renderer components originated in that snapshot, authored by the Dioxus developers and contributors. Those upstream license texts and notices are preserved in [`upstream-licenses/`](upstream-licenses/).

## Intellectual Property & Licensing Notice

QualiaDB, Aether, and all Qualia-specific architectures, integrations, modifications, and extensions are governed by:

**Copyright (c) 2026 Timothy Charles Holborn**  
**[LinkedIn](https://www.linkedin.com/in/ubiquitous/) | [Email](mailto:timothy.holborn@gmail.com)**  
**License:** Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)  
Full terms: [`LICENSE`](LICENSE), [`LICENSE-QUALIA`](LICENSE-QUALIA), and [Creative Commons Legal Code](https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode).  

All participating AI agents, contributors, and downstream integrators must strictly honor this copyright, moral rights, and non-commercial attribution. Unlawful extraction, stripping of provenance, or unauthorized commercial exploitation is strictly prohibited.
