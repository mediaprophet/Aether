# Aether — Sovereign Application Framework for QualiaDB & Webizen

> High-performance cross-platform application framework with native WebGPU acceleration, zero-heap lens reactivity (`refract`), spatial window ergonomics, and unified desktop, web, and spatial rendering.

[![CI](https://github.com/mediaprophet/rust-app-framework/actions/workflows/main.yml/badge.svg)](https://github.com/mediaprophet/rust-app-framework/actions/workflows/main.yml)
[![License: CC BY-NC-ND 4.0](https://img.shields.io/badge/License-CC_BY--NC--ND_4.0-lightgrey.svg)](LICENSE-QUALIA)
[![Ecosystem: QualiaDB](https://img.shields.io/badge/Ecosystem-QualiaDB-blue.svg)](https://github.com/mediaprophet/qualiaDB)
[![Platform: Desktop_|_Web_|_Mobile](https://img.shields.io/badge/Platforms-Desktop_%7C_Web_%7C_Mobile-orange.svg)](#architecture--workspace-structure)

**Aether** is the sovereign application engine powering the next-generation spatial computing environment of [Webizen OS-Shell](https://github.com/WebizenAI/webizen-os-shell) and the [QualiaDB](https://github.com/mediaprophet/qualiaDB) multi-agent cognitive ecosystem. 

Forked from Dioxus and re-engineered for demanding spatial, cognitive, and zero-heap hot-path requirements, Aether unites the ergonomics of modern reactive web programming with the zero-overhead determinism of native Rust systems programming.

---

## Strategic Innovations & Core Pillars

### 1. Built-in WebGPU Acceleration & Limits Compatibility Shim
Embedded webviews and browser runtimes frequently suffer from silent WebGPU failures, missing Vulkan/ANGLE flags, or breaking changes in the W3C WebGPU specification:
- **Out-of-the-Box WebView2 GPU Enablement**: Aether Desktop automatically configures Windows WebView2 with `--enable-unsafe-webgpu`, `--enable-features=Vulkan,DefaultANGLEVulkan,VulkanFromANGLE`, `--use-angle=vulkan`, and `--ignore-gpu-blocklist`. Hardware-accelerated shaders and compute kernels run without manual registry tweaks or fragile shell wrapper scripts.
- **W3C Device Limits Compatibility Shim**: Modern Chromium/Edge builds throw hard JavaScript exceptions when unrecognized limits are passed to `requestDevice()`. In particular, `maxInterStageShaderComponents` was removed by W3C, causing downstream `wgpu` applications to fail on initialization. Aether injects Qualia's authoritative compatibility shim into every window, automatically filtering deprecated or unsupported limits against `adapter.limits` before `requestDevice()` executes.
- **Adaptive Fallback Policy**: Supports multi-tier adapter discovery (Default $\to$ Compatibility $\to$ Low-Power $\to$ Software), ensuring robust execution across discrete GPUs, integrated graphics, and mobile silicon.

### 2. Zero-Heap Reactive Store & Lens Runtime (`refract`)
Standard UI frameworks rely heavily on heap allocations (`Vec`, `String`, `Box`) and runtime borrow counting (`RefCell`), causing allocator contention and cache misses:
- **QualiaDB Rule 0 Compliance**: Built specifically to adhere to the immovable *Zero heap in hot paths* constraint, `refract` delivers a borrow-checked reactive UI system with **zero `RefCell` overhead** and zero runtime borrow counting.
- **Fixed-Buffer Array Projections ([`ArrayLens`](refract/src/lens.rs))**: UI components can focus directly on fixed-size stack arrays (`[T; N]`) and 48-byte Super-Quins (`[NQuin; N]`) with element-level invalidation granularity. Updating a single slot wakes only its direct observers without allocating a single byte on the heap.
- **Autoregressive Streaming at 60+ Tokens/Sec**: Evaluates and updates high-frequency LLM token streams and cognitive traces directly at leaf nodes without triggering virtual DOM tree reconciliation passes.

### 3. Spatial Desktop Shell & Native Window Lifecycle
Designed to serve as the runtime foundation for the Webizen OS-Shell halo chrome:
- **Glassmorphism & Frameless Ergonomics**: Native support for Windows Acrylic, Mica, full alpha transparency, click-through regions, and non-rectangular custom window geometries.
- **System Tray & Auxiliary Window Coordination**: Built-in tray lifecycle management (`HideToTray`, `QuitShell`, restore) and lightweight auxiliary window spawning for companion tools (Poet, Studio, Settings, System Monitor).
- **Direct GPU Viewport Integration**: Extends `packages/native` (Blitz + Vello) with `custom-widget` hooks to mount raw `wgpu` surfaces directly inside the window tree. Qualia's `PortalGpu` 10D manifold projections and WGSL Forge compute shaders render alongside UI elements with zero IPC serialization overhead.

---

## Architecture & Workspace Structure

```text
rust-app-framework/
├── packages/
│   ├── desktop/          # Wry/Tao desktop webview runner with WebGPU & limits shim
│   ├── native/           # Blitz + AnyRender + Vello native GPU renderer (custom-widget support)
│   ├── web/              # Direct WASM browser renderer
│   ├── core/             # VirtualDOM, dynamic templates, diffing, and component scheduler
│   ├── signals/          # Copyable reactive signals & stores (generational-box)
│   ├── hooks/            # Ergonomic hooks (use_signal, use_effect, use_resource, etc.)
│   ├── router/           # Type-safe client & desktop routing
│   ├── fullstack/        # SSR, progressive hydration, and server functions
│   └── cli/              # Development server, hot-reloading, and multiplatform bundler
├── refract/              # Zero-heap store & lens reactive runtime (QualiaDB optimized)
├── .github/workflows/    # Clean CI workflows configured for standard GitHub Actions
└── upstream-licenses/    # Upstream Dioxus MIT and Apache-2.0 license texts
```

---

## Quickstart

### 1. Standard Desktop Application with WebGPU

Add Aether to your `Cargo.toml`:

```toml
[dependencies]
dioxus = { git = "https://github.com/mediaprophet/rust-app-framework.git", branch = "qualia-main", features = ["desktop"] }
```

Launch a hardware-accelerated desktop window:

```rust
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
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
use refract::{Ui, lens, ArrayLens};

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
cargo build -p dioxus-desktop
```

### Run `refract` Zero-Heap Suite:
```bash
cargo test -p refract
```

### Web / WASM Verification:
```bash
cargo check -p dioxus-web --target wasm32-unknown-unknown
```

---

## Intellectual Property & Licensing Notice

QualiaDB, Aether, the Qualia App Framework, and all Qualia-specific architectures, integrations, modifications, and extensions are governed by:

**Copyright (c) 2026 Timothy Charles Holborn**  
**[LinkedIn](https://www.linkedin.com/in/ubiquitous/) | [Email](mailto:timothy.holborn@gmail.com)**  
**License:** Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)  
Full terms: [`LICENSE-QUALIA`](LICENSE-QUALIA) and [Creative Commons Legal Code](https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode).  

Upstream Dioxus framework code retains its respective Apache-2.0 and MIT licensing as documented in [`upstream-licenses/`](upstream-licenses/).

All participating AI agents, contributors, and downstream integrators must strictly honor this copyright, moral rights, and non-commercial attribution. Unlawful extraction, stripping of provenance, or unauthorized commercial exploitation is strictly prohibited.
