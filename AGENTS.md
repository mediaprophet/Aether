# Qualia App Framework — Agent Coordination & Licensing

## Intellectual Property & Licensing Notice

QualiaDB, the Qualia App Framework, and all Qualia-specific architectures, integrations, modifications, and extensions are governed by:

**Copyright (c) 2026 Timothy Charles Holborn**  
**https://www.linkedin.com/in/ubiquitous/ | timothy.holborn@gmail.com**  
**License:** Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)  
Full terms: [`LICENSE-QUALIA`](LICENSE-QUALIA) and [Creative Commons Legal Code](https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode).  
Upstream Dioxus framework code retains its respective Apache-2.0 and MIT licensing.

All participating AI agents and contributors must strictly honor this copyright, moral rights, and non-commercial attribution. Unlawful extraction, stripping of provenance, or unauthorized commercial exploitation is strictly prohibited.

---
# Dioxus Agent Guide

Dioxus is a cross-platform UI framework for Rust, similar to React. It compiles to web (WASM), desktop (webview), mobile (iOS/Android), and native (GPU-rendered).

## Quick Overview

- **Language**: Rust (stable toolchain)
- **UI Model**: React-like with VirtualDOM, components, hooks, signals
- **Syntax**: JSX-like `rsx!` macro for declaring UI
- **Platforms**: Web, Desktop (Windows/macOS/Linux), Mobile, Native, LiveView (server-rendered)

## Workspace Structure

```
packages/
â”œâ”€â”€ dioxus/           # Main re-export crate users depend on
â”œâ”€â”€ core/             # VirtualDOM, components, diffing, scheduling
â”œâ”€â”€ rsx/              # RSX macro parsing and code generation
â”œâ”€â”€ rsx-hotreload/    # Template diffing for hot-reload
â”œâ”€â”€ signals/          # Reactive state (Signal, Memo, Store)
â”œâ”€â”€ hooks/            # Built-in hooks (use_signal, use_effect, etc.)
â”œâ”€â”€ router/           # Type-safe routing with #[derive(Routable)]
â”œâ”€â”€ fullstack/        # SSR, hydration, #[server] functions
â”œâ”€â”€ cli/              # `dx` build tool, dev server, bundling
â”œâ”€â”€ web/              # WASM renderer
â”œâ”€â”€ desktop/          # Wry/Tao webview renderer
â”œâ”€â”€ native/           # Blitz/Vello GPU renderer
â”œâ”€â”€ liveview/         # WebSocket streaming renderer
â”œâ”€â”€ manganis/         # asset!() macro for compile-time assets
â”œâ”€â”€ subsecond/        # Hot-patching system (jump table indirection)
â”œâ”€â”€ devtools/         # Dev server communication protocol
â”œâ”€â”€ interpreter/      # Sledgehammer JS for DOM mutations
â””â”€â”€ wasm-split/       # WASM code splitting
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

## Key Concepts

- **VirtualDOM**: Tree of `VNode` with templates, dynamic nodes, and attributes
- **Signals**: Copy-able reactive primitives via generational-box (generation-based validity)
- **WriteMutations**: Trait that renderers implement to apply DOM changes
- **RSX**: Proc macro that compiles JSX-like syntax to `VNode` construction
- **Server Functions**: `#[server]` macro generates client RPC stubs and server handlers
- **Subsecond**: Hot-patches Rust code via jump table indirection (no memory modification)
- **Manganis**: `asset!("/main.css")` macro for including assets by embedding data via linker symbols

## Common Patterns

**Component definition**:
```rust
#[component]
fn MyComponent(name: String) -> Element {
    let mut count = use_signal(|| 0);
    rsx! {
        button { onclick: move |_| count += 1, "{name}: {count}" }
    }
}
```

## Notes for Agents

1. The `dioxus` crate re-exports from other crates - most implementation is in `packages/core`, `packages/signals`, etc.
2. RSX macro expansion happens in `packages/rsx` - look there for syntax questions
3. Each renderer implements `WriteMutations` differently - see `06-RENDERERS.md`
4. Hot-reload has two systems: RSX template diffing (fast) and Subsecond code patching (full Rust)
5. Assets use link sections and binary patching - the `asset!()` macro creates symbols the CLI processes


