# B1a — native lifecycle (human-centric chrome)

**Date:** 2026-11-20  
**Status:** Verified on this box (uncommitted dirty tree). `cargo check` B1a features 6m 53s; `native_lifecycle_smoke` opened a window, three redraws, owner dropped, process exited. Aether is a **toolkit**, not the app.

## Identity

Aether is a QualiaDB-native GUI toolkit (Dioxus / Tauri analogue). Host applications — `webizen-os-shell`, Synth operator UI, later Qaps — consume it. Do not ship Aether window titles, tray items, or default icons as product identity.

Two webs:

- **Human-centric web** — QualiaDB + VibeScript + Aether native RSX. **Chora** is the spatial commons canvas in Qualia, not this window crate. B1a is the toolkit window.
- **Old web** — Webizen Web Browser Qap. Engine is the **Servo-derived MPL tree** (repo currently misnamed `chora`). Sibling worker + copied-frame. Not WebView, not Wry.

B1a leaves the old-web engine unlinked so the toolkit window can paint. That is sequencing. Do not redefine Chora as Servo updates.

QualiaDB is the required application foundation. B1a does **not** link Qualia crates; missing host wiring is C0/C2.

## Slice

| In B1a | Out (later) |
|---|---|
| `packages/native` window / lifecycle / input | `chora_frame.rs` (B1b; leave untracked) |
| `packages/native-dom` | `packages/chora-client` (workspace excluded) |
| `try_launch` / `try_launch_cfg` / `LaunchError` | `qualia-host` (workspace excluded) |
| `examples/native_lifecycle_smoke.rs` | Frozen v4, pixels, device-loss, Vibe eval |
| `asset_access.rs` (required by `assets.rs`) | |

`chora` is an **optional** native feature (copied-frame client). Default B1a checks leave it off so the engine is not linked. B1b turns it on once `chora-client` and `chora_frame.rs` are the published worker path.

## Acceptance

```text
cargo run -p aether-native --example native_lifecycle_smoke --features prelude,vello-cpu-softbuffer
```

A Windows window opens, at least three redraws, owner drops, process exits. This is **lifecycle**, not pixel certification.

Os-shell C1 path-pins:

```toml
aether = { path = "../../Aether/packages/aether", default-features = false, features = ["macro","html","signals","hooks","launch","native"] }
aether-native = { path = "../../Aether/packages/native", default-features = false, features = ["vello-cpu-softbuffer","system-fonts","clipboard","accessibility","html"] }
```

`packages/aether` takes `aether-native` with `default-features = false` so the host renderer pin is the one that compiles (CPU softbuffer for C1). Enabling `aether/native` alone does not pull vello-hybrid/Etalon GPU.

Os-shell must also `[patch.crates-io]` `blitz-traits` / `blitz-dom` / `blitz-net` to `../../Aether/vendor/blitz/...`. Aether workspace patches do not apply to a sibling package; crates.io Blitz 0.3.0-beta.4 lacks `CompletionHandler` / `NetError`.

Synth stays on git `d5ccce3` desktop (G4) until it opts into a published B1a rev.

## Check command (this machine)

```text
Set-Location C:\github\Aether
$env:CARGO_TARGET_DIR = "C:\github\Aether\target-b1a"
cargo check -p aether-native --offline --no-default-features --features "prelude,vello-cpu-softbuffer,system-fonts,clipboard,accessibility,html,net"
```
