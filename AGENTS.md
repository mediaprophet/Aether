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

Aether is a sovereign cross-platform UI and spatial application framework for Rust, powering Webizen OS-Shell and QualiaDB. It compiles to Web (WASM), Desktop (WebView2/Tao with WebGPU compatibility shims), Mobile (iOS/Android), and Native GPU (Blitz + AnyRender + Etalon).

## Quick Overview

- **Language**: Rust (2024 edition, `1.85.0+`)
- **Primary Crates**: `aether`, `aether-native`, `aether-desktop`, `aether-web`, `refract`
- **UI Model**: Reactive VirtualDOM with components, hooks, copyable signals, plus `refract` zero-heap lens reactivity
- **Syntax**: `rsx!` macro for declaring UI (`use aether::prelude::*;`)
- **Platforms**: Web, Desktop (Windows/macOS/Linux), Mobile, Native GPU, LiveView

## Workspace Structure

```text
packages/
├── aether/           # Primary sovereign crate re-exporting Aether UI + `refract`
├── aether-native/    # Sovereign native GPU renderer crate (Blitz + Etalon)
├── aether-desktop/   # Sovereign Wry/Tao desktop webview renderer crate
├── aether-web/       # Sovereign WASM renderer crate
├── aether-core/      # Sovereign core VirtualDOM crate
├── aether-signals/   # Sovereign reactive signals crate
├── aether-hooks/     # Sovereign built-in hooks crate
├── aether-html/      # Sovereign HTML element definitions crate
├── aether-router/    # Sovereign type-safe routing crate
├── dioxus/           # Internal compatibility bridge crate
├── core/             # VirtualDOM, components, diffing, scheduling implementation
├── rsx/              # RSX macro parsing and code generation
├── rsx-hotreload/    # Template diffing for hot-reload
├── signals/          # Reactive state (Signal, Memo, Store)
├── hooks/            # Built-in hooks (use_signal, use_effect, etc.)
├── router/           # Type-safe routing with #[derive(Routable)]
├── fullstack/        # SSR, hydration, #[server] functions
├── cli/              # Build tool, dev server, bundling
├── web/              # WASM renderer implementation
├── desktop/          # Wry/Tao webview renderer implementation
├── native/           # Blitz/Etalon GPU renderer implementation
├── liveview/         # WebSocket streaming renderer
├── manganis/         # asset!() macro for compile-time assets
├── subsecond/        # Hot-patching system (jump table indirection)
├── devtools/         # Dev server communication protocol
├── interpreter/      # Sledgehammer JS for DOM mutations
└── wasm-split/       # WASM code splitting
refract/              # Zero-heap reactive store and lens runtime (`ArrayLens`)
```

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
