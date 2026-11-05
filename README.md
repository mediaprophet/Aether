# Aether — Sovereign Application Framework for QualiaDB & Webizen

> High-performance cross-platform application framework with native Etalon/WebGPU acceleration, zero-heap lens reactivity (`refract`), spatial window ergonomics, and unified desktop, web, and GPU-native rendering.

[![CI](https://github.com/mediaprophet/Aether/actions/workflows/main.yml/badge.svg)](https://github.com/mediaprophet/Aether/actions/workflows/main.yml)
[![License: CC BY-NC-ND 4.0](https://img.shields.io/badge/License-CC_BY--NC--ND_4.0-lightgrey.svg)](LICENSE-QUALIA)
[![Ecosystem: QualiaDB](https://img.shields.io/badge/Ecosystem-QualiaDB-blue.svg)](https://github.com/mediaprophet/qualiaDB)
[![Platform: Desktop_|_Web_|_Native](https://img.shields.io/badge/Platforms-Desktop_%7C_Web_%7C_Native-orange.svg)](#architecture--workspace-structure)

**Aether** is the sovereign application engine powering the next-generation spatial computing environment of [Webizen OS-Shell](https://github.com/WebizenAI/webizen-os-shell) and the [QualiaDB](https://github.com/mediaprophet/qualiaDB) multi-agent cognitive ecosystem.

Engineered for demanding spatial, cognitive, and zero-heap hot-path requirements, Aether unites the ergonomics of modern reactive component programming with the zero-overhead determinism of native Rust systems programming.

---

## Strategic Innovations & Core Pillars

### 1. Built-in WebGPU Acceleration & Limits Compatibility Shim
Embedded webviews and browser runtimes frequently suffer from silent WebGPU failures, missing Vulkan/ANGLE flags, or breaking changes in the W3C WebGPU specification:
- **Out-of-the-Box WebView2 GPU Enablement**: `aether-desktop` automatically configures Windows WebView2 with `--enable-unsafe-webgpu`, `--enable-features=Vulkan,DefaultANGLEVulkan,VulkanFromANGLE`, `--use-angle=vulkan`, and `--ignore-gpu-blocklist`. Hardware-accelerated shaders and compute kernels run without manual registry tweaks or fragile shell wrapper scripts.
- **W3C Device Limits Compatibility Shim**: Modern Chromium/Edge builds throw hard JavaScript exceptions when unrecognized limits are passed to `requestDevice()`. In particular, `maxInterStageShaderComponents` was removed by W3C, causing downstream `wgpu` / `etalon` applications to fail on initialization. Aether injects Qualia's authoritative compatibility shim into every window, automatically filtering deprecated or unsupported limits against `adapter.limits` before `requestDevice()` executes.
- **Adaptive Fallback Policy**: Supports multi-tier adapter discovery (Default $\to$ Compatibility $\to$ Low-Power $\to$ Software), ensuring robust execution across discrete GPUs, integrated graphics, and mobile silicon.

### 2. Zero-Heap Reactive Store & Lens Runtime (`refract`)
Standard UI frameworks rely heavily on heap allocations (`Vec`, `String`, `Box`) and runtime borrow counting (`RefCell`), causing allocator contention and cache misses:
- **QualiaDB Rule 0 Compliance**: Built specifically to adhere to the immovable *Zero heap in hot paths* constraint, `refract` (re-exported via `aether::refract`) delivers a borrow-checked reactive UI system with **zero `RefCell` overhead** and zero runtime borrow counting.
- **Fixed-Buffer Array Projections ([`ArrayLens`](refract/src/lens.rs))**: UI components can focus directly on fixed-size stack arrays (`[T; N]`) and 48-byte Super-Quins (`[NQuin; N]`) with element-level invalidation granularity. Updating a single slot wakes only its direct observers without allocating a single byte on the heap.
- **Autoregressive Streaming at 60+ Tokens/Sec**: Evaluates and updates high-frequency LLM token streams and cognitive traces directly at leaf nodes without triggering virtual DOM tree reconciliation passes.

### 3. Spatial Desktop Shell & Native Window Lifecycle
Designed to serve as the runtime foundation for the Webizen OS-Shell halo chrome:
- **Glassmorphism & Frameless Ergonomics**: Native support for Windows Acrylic, Mica, full alpha transparency, click-through regions, and non-rectangular custom window geometries.
- **System Tray & Auxiliary Window Coordination**: Built-in tray lifecycle management (`HideToTray`, `QuitShell`, restore) and lightweight auxiliary window spawning for companion tools (Poet, Studio, Settings, System Monitor).
- **Direct GPU Viewport Integration**: `aether-native` (Blitz + AnyRender + Etalon) provides `custom-widget` hooks to mount raw `etalon` (`wgpu`) surfaces directly inside the window tree. Qualia's `PortalGpu` 10D manifold projections and WGSL Forge compute shaders render alongside UI elements with zero IPC serialization overhead.

---

## Architecture & Workspace Structure

```text
aether/
├── packages/
│   ├── aether/           # Primary sovereign crate re-exporting Aether UI + `refract`
│   ├── aether-native/    # Native GPU renderer (Blitz + AnyRender + Etalon custom-widget support)
│   ├── aether-desktop/   # Wry/Tao desktop webview runner with WebGPU & limits shim
│   ├── aether-web/       # Direct WASM browser renderer
│   ├── aether-core/      # VirtualDOM, dynamic templates, diffing, and component scheduler
│   ├── aether-signals/   # Copyable reactive signals & stores (generational-box)
│   ├── aether-hooks/     # Ergonomic hooks (use_signal, use_effect, use_resource, etc.)
│   ├── aether-html/      # HTML element vocabulary and event bindings
│   ├── aether-router/    # Type-safe client & desktop routing
│   ├── fullstack/        # SSR, progressive hydration, and server functions
│   └── cli/              # Development server, hot-reloading, and multiplatform bundler
├── refract/              # Zero-heap store & lens reactive runtime (QualiaDB optimized)
├── .github/workflows/    # Clean CI workflows configured for standard GitHub Actions
└── upstream-licenses/    # Upstream MIT and Apache-2.0 license texts
```

---

## Quickstart

### 1. Standard Desktop Application with WebGPU

Add Aether to your `Cargo.toml`:

```toml
[dependencies]
aether = { git = "https://github.com/mediaprophet/Aether.git", branch = "main", features = ["desktop"] }
```

Launch a hardware-accelerated desktop window:

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

For high-frequency or memory-constrained evaluators where heap allocation is prohibited:

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
