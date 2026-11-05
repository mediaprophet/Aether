# Aether Architecture Overview

This directory contains comprehensive architecture documentation for the Aether framework, designed to help future Claude agents (and human contributors) understand the codebase deeply.

## Document Index

| File | Description |
|------|-------------|
| `01-CORE.md` | VirtualDOM, rendering pipeline, component system, diffing algorithm |
| `02-CLI.md` | Build system, dev server, bundling, terminal UI, platform support |
| `03-RSX.md` | RSX macro parsing, AST types, code generation, autofmt |
| `04-SIGNALS.md` | Signals, generational-box, hooks, stores, state management |
| `05-FULLSTACK.md` | Server functions, SSR, hydration, client-server communication |
| `06-RENDERERS.md` | Web, desktop, native, liveview renderer implementations |
| `07-HOTRELOAD.md` | Subsecond hot-patching, RSX hot-reload, devtools protocol |
| `08-ASSETS.md` | Manganis asset system, const serialization, build-time processing |
| `09-ROUTER.md` | Routing system, Routable trait, navigation, history |
| `10-WASM-SPLIT.md` | WASM code splitting, lazy loading, chunk management |

## Crate Dependency Overview

```
aether (umbrella crate)
├── aether-core (VirtualDOM, runtime, scheduler)
│   └── generational-box (Copy semantics for references)
├── aether-rsx (macro parsing)
│   └── aether-autofmt (code formatting)
├── aether-signals (reactive state)
│   ├── generational-box
│   └── aether-stores (nested reactivity)
├── aether-hooks (use_signal, use_effect, etc.)
├── aether-html (HTML elements, events)
├── aether-router (client-side routing)
│
├── Renderers:
│   ├── aether-web (WASM/browser)
│   ├── aether-desktop (wry/tao webview)
│   ├── aether-native (blitz/vello GPU)
│   ├── aether-liveview (WebSocket streaming)
│   └── aether-ssr (server-side rendering)
│
├── Fullstack:
│   ├── aether-fullstack (client/server coordination)
│   ├── aether-fullstack-core (transport, types)
│   ├── aether-fullstack-macro (#[server] macro)
│   └── aether-server (Axum integration)
│
├── CLI (dx command):
│   ├── aether-cli (main binary)
│   ├── aether-cli-config (Aether.toml parsing)
│   └── aether-cli-opt (optimization passes)
│
├── Hot Reload:
│   ├── subsecond (hot-patching runtime)
│   ├── aether-rsx-hotreload (template diffing)
│   └── aether-devtools (WebSocket communication)
│
├── Assets:
│   ├── manganis (asset!() macro)
│   ├── manganis-core (asset types)
│   ├── manganis-macro (proc-macro)
│   └── const-serialize (compile-time CBOR)
│
└── Code Splitting:
    ├── wasm-splitter (CLI tool)
    ├── wasm-split-macro (#[wasm_split])
    └── wasm-split-cli (binary processing)
```

## Key Architectural Patterns

### 1. Copy Semantics for Reactive State
Aether uses `generational-box` to provide `Copy` semantics for references. This enables signals and other reactive primitives to be freely passed around without explicit cloning.

### 2. WriteMutations Trait
All renderers implement `WriteMutations` which defines how VirtualDOM changes translate to real DOM operations. This abstraction enables the same component code to run on web, desktop, mobile, and server.

### 3. Template-Based Rendering
RSX macros generate static `Template` definitions at compile time. Only dynamic parts are diffed at runtime, making updates extremely efficient.

### 4. Hot-Reload Without Full Rebuild
Two complementary systems:
- **RSX Hot-Reload**: Changes to template literals are diffed and sent via WebSocket
- **Subsecond Hot-Patching**: Full Rust function replacement via jump table indirection

### 5. Compile-Time Asset Processing
The `asset!()` macro embeds asset metadata in binary link sections. The CLI extracts this at build time, processes assets, and patches the binary with final URLs.

### 6. Server Functions as RPC
The `#[server]` macro generates dual code paths - client serialization and server handler - enabling seamless RPC-style communication.

## Important Design Decisions

1. **Single VirtualDOM**: Even multi-window desktop apps share one VirtualDOM for simpler state management
2. **Scope-Based Cleanup**: All state (signals, tasks, contexts) is tied to component scope lifetime
3. **Platform Agnostic Events**: Events are converted through `HtmlEventConverter` trait for cross-platform support
4. **Binary Protocols**: Desktop and liveview use Sledgehammer binary format for compact mutation encoding
5. **Generational GC**: References are invalidated by generation counters, preventing use-after-free without runtime overhead per access

## Future Development Areas

These docs are designed to help with planned features:
- **Custom Bundlers**: Replacing tauri-bundler with native implementation
- **CLI Tunnels**: Remote device hot-reloading via SSH/SCP
- **Workspace Hot-Patching**: Patching library crates, not just the tip crate
- **Universal Android Builds**: Single APK for all architectures
- **Fullstack Extensions**: First-party database, auth, IAAC, secrets
