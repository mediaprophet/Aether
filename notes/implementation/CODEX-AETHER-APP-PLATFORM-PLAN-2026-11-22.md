# Codex Implementation Plan: Aether Application Platform Completion
**Date:** 2026-11-22  
**Copyright (c) 2026 Timothy Charles Holborn — CC BY-NC-ND 4.0**  
**Target Workspace:** `C:\github\Aether`  
**Execution Role:** External Coding Agent (**Codex**) — **SOURCE-ONLY IMPLEMENTATION (NO CARGO COMMANDS)**  
**Verification & Build Owner:** **Antigravity Coordinator** (runs all `cargo check`, `cargo clippy`, and `cargo test` passes after Codex reports completion)

---

## 0. Mandatory Rules & Constraints for Codex

1. **STRICT NO-CARGO RULE**:
   - **Do NOT run `cargo check`, `cargo test`, `cargo build`, `cargo clippy`, `cargo update`, or `cargo metadata`.**
   - Your job is strictly to write clean, complete, type-correct Rust source files, unit test modules (`#[cfg(test)]`), and targeted dependency wiring as specified in each Work-Package (`WP1`–`WP6`) below.
   - When all 6 Work-Packages are complete, stop and report the modified file list so the Coordinator can review, check, and build.

2. **LICENSING & VOCABULARY INVARIANTS**:
   - All new Aether-owned files must start with:
     ```rust
     // Copyright (c) 2026 Timothy Charles Holborn.
     // SPDX-License-Identifier: CC-BY-NC-ND-4.0
     ```
   - Preserve upstream MIT/Apache-2.0 attribution in `vendor/blitz/*`.
   - **Never** use the words `"sovereign"`, `"sovereignty"`, or `"self-sovereign"` in code, comments, identifiers, or error messages. Use human-rights agreement vocabulary (**Human-Centric**, **Human Rights**, **Human Dignity**, **Human Agency**, **Natural Person**, **Sanctuary**, **Permissive Commons**).
   - **Never** ship `"Aether App"` as a fallback product identity in window titles.
   - Always use **`Etalon`** (`etalon`, `etalon_context`) — never introduce `wgpu` package dependencies or identifiers.

3. **PATCHED VS. UNPATCHED & STANDALONE WORKSPACE BOUNDARIES**:
   - **Writable inside `C:\github\Aether`**:
     - `vendor/blitz/blitz-traits`
     - `vendor/blitz/blitz-dom`
     - `vendor/blitz/blitz-net`
     - `packages/*`
     - `refract/`
   - **Unpatched external crates (DO NOT assume edits to these)**:
     - `blitz-paint =0.3.0-beta.4`
     - `blitz-shell =0.3.0-beta.4`
     - `anyrender =0.14.0`
     - `etalon` (git rev `575bab54accf1aafb24f67e9ba810c0f3f745aea`)
     - `vector-gpu-render-adapter` / `vector-hybrid-render-adapter` / `vector-cpu-render-adapter`
   - **Excluded Standalone Crates (`Cargo.toml` line 127)**:
     - `packages/qualia-host`, `packages/chora-client`, `packages/chora-bridge-protocol`, and `packages/chora-worker` are excluded from the root `[workspace]` and use their own Cargo manifests. Never import `qualia-core-db` or `aether-qualia-host` into root workspace crates (`packages/native`, `packages/native-dom`, `packages/aether`, `refract`).

---

## Work-Package Overview & Disjoint File Ownership

| Work-Package | Title | Owned Files | Depends On |
| --- | --- | --- | --- |
| **WP1** | `vendor/blitz` CustomWidget Lifecycle, Focus & Native DOM Event Emitters | `vendor/blitz/blitz-traits/src/events.rs`, `vendor/blitz/blitz-dom/src/node/{custom_widget.rs, element.rs}`, `vendor/blitz/blitz-dom/src/{document.rs, mutator.rs}`, `vendor/blitz/blitz-dom/src/events/{mod.rs, driver.rs, keyboard.rs, ime.rs}` | None (Foundation) |
| **WP2** | `packages/native-dom` Event Completeness & Zero-`unimplemented!()` Payloads | `packages/native-dom/src/{events.rs, mutation_writer.rs, aether_document.rs, write_once_attr.rs}` | WP1 (`DomEventKind` / `DomEventData`) |
| **WP3** | `packages/native` Etalon Surface Widget, GPU Fallback, Multi-Window Roles & Mobile-Native Routing | `packages/native/src/{viewport.rs, window_role.rs, aether_renderer.rs, aether_application.rs, config.rs, contexts.rs, lib.rs}`, `packages/config-macro/src/lib.rs`, `packages/aether/{Cargo.toml, src/launch.rs, src/lib.rs}` | WP1 (`CustomWidget` lifecycle) |
| **WP4** | Interactive Input/Focus Forwarding & Coalesced Transport for `ChoraFrameWidget` | `packages/chora-bridge-protocol/src/{types.rs, state.rs, tests.rs}`, `packages/chora-client/src/{lib.rs, tests.rs}`, `packages/chora-worker/src/main.rs`, `packages/native/src/chora_frame.rs` | WP1 (`Widget::focus_changed`) |
| **WP5** | `packages/qualia-host` Ergonomic Declarative Hooks & Holographic 10D v2 Viewport Delegate | `packages/qualia-host/{Cargo.toml, src/lib.rs, src/hooks.rs, src/holographic_viewport.rs}` | None (Standalone workspace) |
| **WP6** | `refract` ↔ Aether VirtualDOM Reactive Lens Bridge & Semantic UI RSX Components | `refract/src/{lib.rs, ui.rs, ui_scheduler.rs, dom.rs, semantic_ui.rs}`, `packages/aether/src/refract_bridge.rs` | WP3 (`packages/aether/src/lib.rs` export) |

---

## WP1: `vendor/blitz` CustomWidget Lifecycle, Focus Hook & Native DOM Event Emitters

### Files Owned
- `vendor/blitz/blitz-traits/src/events.rs`
- `vendor/blitz/blitz-dom/src/node/custom_widget.rs`
- `vendor/blitz/blitz-dom/src/node/element.rs`
- `vendor/blitz/blitz-dom/src/document.rs`
- `vendor/blitz/blitz-dom/src/mutator.rs`
- `vendor/blitz/blitz-dom/src/events/mod.rs`
- `vendor/blitz/blitz-dom/src/events/keyboard.rs`
- `vendor/blitz/blitz-dom/src/events/ime.rs`

### Exact Implementation Tasks

#### 1.1 Fix `CustomWidgetData` Lifecycle & GPU `ResourceId` Leak (`custom_widget.rs`, `element.rs`, `document.rs`)
- **Problem**:
  1. `BaseDocument::can_create_surfaces` (`vendor/blitz/blitz-dom/src/node/custom_widget.rs:15-32`) calls `widget.can_create_surfaces(...)` without setting `widget_data.status = CustomWidgetStatus::Active`.
  2. `BaseDocument::destroy_surfaces` (`custom_widget.rs:34-44`) calls `widget.destroy_surfaces()` **without** setting `widget_data.status = CustomWidgetStatus::Suspended` and **without** draining `widget_data.active_resource_ids` into `self.pending_resource_deallocations`. Because unpatched `blitz-paint-0.3.0-beta.4/src/lib.rs:133-136` only invokes `widget.can_create_surfaces` when `widget_data.status == CustomWidgetStatus::Suspended`, failing to reset `status = CustomWidgetStatus::Suspended` breaks surface recovery after suspend/resume.
  3. `ElementData::set_custom_widget` (`vendor/blitz/blitz-dom/src/node/element.rs:794-797`) and `BaseDocument::set_custom_widget` (`document.rs:834-841`) overwrite an existing `SpecialElementData::CustomWidget` without calling `old_widget.widget.disconnected()` or moving `old_widget.take_resource_ids()` into `self.pending_resource_deallocations`, and without calling `new_widget.connected()`.
  4. `BaseDocument::remove_custom_widget` (`document.rs:843-851`) does not call `widget_data.widget.disconnected()` before extracting resource IDs.
- **Required Fix**:
  - In `vendor/blitz/blitz-dom/src/node/custom_widget.rs`:
    - Add a default method on `pub trait Widget`:
      ```rust
      fn focus_changed(&mut self, _focused: bool) {}
      ```
    - In `BaseDocument::can_create_surfaces(&mut self, render_ctx: &mut dyn RenderContext)`:
      - Only invoke `widget_data.widget.can_create_surfaces(&mut proxy_ctx)` when `widget_data.status == CustomWidgetStatus::Suspended`, and set `widget_data.status = CustomWidgetStatus::Active`.
    - In `BaseDocument::destroy_surfaces(&mut self, _render_ctx: &mut dyn RenderContext)`:
      - Call `widget_data.widget.destroy_surfaces()`, set `widget_data.status = CustomWidgetStatus::Suspended`, and collect `widget_data.take_resource_ids()` into `self.pending_resource_deallocations`.
  - In `vendor/blitz/blitz-dom/src/document.rs`:
    - In `BaseDocument::set_custom_widget(&mut self, node_id: usize, mut widget: Box<dyn Widget>)`:
      - If the target element already has `SpecialElementData::CustomWidget(mut old_widget)`, call `old_widget.widget.disconnected()` and `self.pending_resource_deallocations.extend(old_widget.take_resource_ids())`.
      - Call `widget.connected()` before storing `CustomWidgetData::new(widget)`.
    - In `BaseDocument::remove_custom_widget(&mut self, node_id: usize)`:
      - Call `widget_data.widget.disconnected()` before `self.pending_resource_deallocations.extend(widget_data.take_resource_ids())`.

#### 1.2 Notify `CustomWidget` on Focus Gain / Blur (`document.rs`)
- **Problem**: `map_dom_event_to_ui_event` in `vendor/blitz/blitz-dom/src/events/mod.rs:95-98` maps `DomEventData::Focus | Blur | FocusIn | FocusOut => None`, so custom widgets (`ChoraFrameWidget`, `EtalonSurfaceWidget`) never know when focus leaves or enters them.
- **Required Fix**:
  - In `BaseDocument::clear_focus(&mut self)` and `BaseDocument::set_focus_to(&mut self, focus_node_id: usize) -> bool` (`vendor/blitz/blitz-dom/src/document.rs:2092-2142`):
    - Whenever `old_focus_node_id` loses focus, if that node has `SpecialElementData::CustomWidget(widget_data)`, call `widget_data.widget.focus_changed(false)`.
    - Whenever `new_focus_node_id` gains focus, if that node has `SpecialElementData::CustomWidget(widget_data)`, call `widget_data.widget.focus_changed(true)`.

#### 1.3 Expand `DomEventKind` and `DomEventData` in Lockstep (`vendor/blitz/blitz-traits/src/events.rs`)
- **Problem**:
  - `DomEventKind` (`events.rs:116-158`) and `DomEventData` (`events.rs:220-264`) are `#[repr(u8)]` enums whose discriminants MUST match 1:1.
  - `DomEventKind::from_str` (`events.rs:167`) uses `s.trim_start_matches("on")`, which corrupts any event name starting with `"on"`.
- **Required Fix**:
  - Change `s.trim_start_matches("on")` in `DomEventKind::from_str` to `s.strip_prefix("on").unwrap_or(s)`.
  - Add typed event payloads in `vendor/blitz/blitz-traits/src/events.rs`:
    ```rust
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BlitzClipboardEvent {
        pub text: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BlitzBeforeInputEvent {
        pub input_type: String,
        pub data: Option<String>,
        pub is_composing: bool,
        pub value: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BlitzSelectionEvent {
        pub selected_text: String,
        pub start_offset: usize,
        pub end_offset: usize,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BlitzToggleEvent {
        pub old_state: bool,
        pub new_state: bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct BlitzResizeEvent {
        pub border_box_width: f64,
        pub border_box_height: f64,
        pub content_box_width: f64,
        pub content_box_height: f64,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BlitzImageEvent {
        pub load_error: bool,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct BlitzCancelEvent;

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct BlitzFormActionEvent {
        pub value: String,
    }
    ```
  - Append matching variants in identical order to both `DomEventKind` and `DomEventData`:
    - `Copy` / `Copy(BlitzClipboardEvent)` (`"copy"`)
    - `Cut` / `Cut(BlitzClipboardEvent)` (`"cut"`)
    - `Paste` / `Paste(BlitzClipboardEvent)` (`"paste"`)
    - `BeforeInput` / `BeforeInput(BlitzBeforeInputEvent)` (`"beforeinput"`)
    - `Select` / `Select(BlitzSelectionEvent)` (`"select"`)
    - `SelectionChange` / `SelectionChange(BlitzSelectionEvent)` (`"selectionchange"`)
    - `BeforeToggle` / `BeforeToggle(BlitzToggleEvent)` (`"beforetoggle"`)
    - `Toggle` / `Toggle(BlitzToggleEvent)` (`"toggle"`)
    - `Resize` / `Resize(BlitzResizeEvent)` (`"resize"`)
    - `Load` / `Load(BlitzImageEvent)` (`"load"`)
    - `Error` / `Error(BlitzImageEvent)` (`"error"`)
    - `Change` / `Change(BlitzFormActionEvent)` (`"change"`)
    - `Submit` / `Submit(BlitzFormActionEvent)` (`"submit"`)
    - `Reset` / `Reset(BlitzFormActionEvent)` (`"reset"`)
    - `Cancel` / `Cancel(BlitzCancelEvent)` (`"cancel"`)
  - Define `pub const COUNT: usize = 50;` on `DomEventKind` and add a unit test in `vendor/blitz/blitz-traits/src/events.rs` verifying that for every variant, `event_data.kind() as u8 == event_data.discriminant()` and `(event_data.discriminant() as usize) < DomEventKind::COUNT`.

#### 1.4 Emit the New DOM Events from Real Blitz Operations (`events/keyboard.rs`, `events/ime.rs`, `events/mod.rs`, `document.rs`)
- In `vendor/blitz/blitz-dom/src/events/keyboard.rs` & `ime.rs`:
  - **Clipboard (`copy`, `cut`, `paste`)**: Before executing text copy, cut, or paste on the target node, dispatch a cancelable `DomEventData::Copy` / `Cut` / `Paste` carrying the selected/clipboard text. If `event_state.is_default_prevented()`, skip the default text mutation / clipboard write.
  - **BeforeInput (`beforeinput`)**: Before applying character insertion, deletion, or paste mutation to `TextInputData`, dispatch a cancelable `DomEventData::BeforeInput(BlitzBeforeInputEvent { input_type, data, is_composing, value })`. If `event_state.is_default_prevented()`, skip the mutation.
  - **Select / Submit (`select`, `submit`, `change`)**: Wire `GeneratedTextInputEvent::Select` to emit `DomEventData::Select(BlitzSelectionEvent { ... })` and `GeneratedTextInputEvent::Submit` to emit `DomEventData::Submit(BlitzFormActionEvent { ... })`.
- In `vendor/blitz/blitz-dom/src/document.rs`:
  - **Toggle (`beforetoggle`, `toggle`)**: In `toggle_details_open(node_id)` (`document.rs:731-752`), dispatch cancelable `DomEventData::BeforeToggle(BlitzToggleEvent { old_state, new_state })` before mutating `open`; if not canceled, apply the toggle and dispatch `DomEventData::Toggle(BlitzToggleEvent { old_state, new_state })`.
  - **Image (`load`, `error`)**: When an `<img>` network response settles (`document.rs:1420-1458`), queue/dispatch `DomEventData::Load(BlitzImageEvent { load_error: false })` on success and `DomEventData::Error(BlitzImageEvent { load_error: true })` on failure.
  - **Resize (`resize`)**: In `set_viewport` (`document.rs:1996`), when viewport dimensions change and a root node exists, dispatch `DomEventData::Resize(BlitzResizeEvent { ... })` on the root element.

---

## WP2: `packages/native-dom` Event Completeness & Zero-`unimplemented!()` Payloads

### Files Owned
- `packages/native-dom/src/events.rs`
- `packages/native-dom/src/mutation_writer.rs`
- `packages/native-dom/src/aether_document.rs`
- `packages/native-dom/src/write_once_attr.rs`

### Exact Implementation Tasks

#### 2.1 Widen `AetherState::event_handler_counts` Safely (`mutation_writer.rs`)
- In `packages/native-dom/src/mutation_writer.rs:274, 424, 431`:
  - Widen `pub(crate) event_handler_counts: [u32; 64]` to `[u32; 96]`.
  - Add a compile-time assertion:
    ```rust
    const _: () = assert!(blitz_traits::events::DomEventKind::COUNT <= 96);
    ```

#### 2.2 Implement All 12 Missing `Native*Data` Structs & Remove Every `unimplemented!()` in `HtmlEventConverter` (`packages/native-dom/src/events.rs`)
- Implement concrete native payload structs for every trait required by `aether_html::HtmlEventConverter`:
  1. `NativeCancelData` (`HasCancelData`)
  2. `NativeBeforeInputData` (`HasBeforeInputData` — mapping `input_type` string to `aether_html::events::InputType`, `data`, `is_composing`, `value`)
  3. `NativeClipboardData` + `NativeDataTransfer` (`HasClipboardData`, `HasDataTransferData`, `HasFileData` — exposing `get_data("text/plain")`, `set_data`, `clear_data`, `effect_allowed`, `drop_effect`, `files`, `items`)
  4. `NativeDragData` (`HasDragData`, `HasMouseData`, `HasFileData`, `HasDataTransferData`)
  5. `NativeImageData` (`HasImageData` — `load_error(&self) -> bool`)
  6. `NativeMediaData` (`HasMediaData`)
  7. `NativeSelectionData` (`HasSelectionData` — `selection(&self) -> Option<TextSelection>`)
  8. `NativeToggleData` (`HasToggleData`)
  9. `NativeTransitionData` (`HasTransitionData` — `property_name`, `pseudo_element`, `elapsed_time`)
  10. `NativeAnimationData` (`HasAnimationData` — `animation_name`, `pseudo_element`, `elapsed_time`)
  11. `NativeResizeData` (`HasResizeData` — `get_border_box_size() -> ResizeResult<PixelsSize>`, `get_content_box_size() -> ResizeResult<PixelsSize>`)
  12. `NativeVisibleData` (`HasVisibleData` — `get_bounding_client_rect`, `get_intersection_ratio`, `get_intersection_rect`, `is_intersecting`, `get_root_bounds`, `get_time`)
- Replace all 12 `unimplemented!()` methods in `impl HtmlEventConverter for NativeConverter` (`packages/native-dom/src/events.rs:243-349`) with `event.downcast::<Native*Data>().unwrap().clone().into()` (matching `convert_form_data` and `convert_mouse_data`).
- Update `native_event_support(name: &str)` (`events.rs:76-87`) and `NativeConverter::convert_dom_event` (`events.rs:92-137`):
  - Enable support for the newly wired real Blitz events: `"copy"`, `"cut"`, `"paste"`, `"beforeinput"`, `"select"`, `"selectionchange"`, `"beforetoggle"`, `"toggle"`, `"resize"`, `"load"`, `"error"`, `"change"`, `"submit"`, `"reset"`, `"cancel"`.
  - Keep `native_event_support` returning `Err(UnsupportedNativeEvent)` for events whose OS/engine source does not yet exist in unpatched `blitz-shell` (`"drag"`, `"dragstart"`, `"dragend"`, `"dragenter"`, `"dragover"`, `"dragleave"`, `"drop"`, `"animationstart"`, `"animationend"`, `"animationiteration"`, `"transitionend"`, `"visible"`, `"play"`, `"pause"`, etc.) so `native_event_support` never fabricates fake capabilities.
  - Update the unit test `unsupported_native_events_report_capability_errors_without_fake_data` (`events.rs:1082-1112`) and add unit tests verifying `convert_dom_event` and `HtmlEventConverter` round-trip for `Copy`, `Cut`, `Paste`, `BeforeInput`, `Select`, `SelectionChange`, `BeforeToggle`, `Toggle`, `Resize`, `Load`, `Error`, `Change`, `Submit`, `Reset`, and `Cancel`.

#### 2.3 Stable `CustomWidgetHandle` Helper (`write_once_attr.rs`)
- In `packages/native-dom/src/write_once_attr.rs`:
  - Add a documentation note and helper constructor `CustomWidgetAttr::with_stable_id<T: Widget + 'static>(id: usize, value: T) -> Self` so RSX components that re-render with a stable widget ID do not force `PartialEq` inequality on `<object data: ...>` every frame.

---

## WP3: `packages/native` Etalon Surface Widget, GPU Fallback, Multi-Window Roles & Mobile-Native Routing

### Files Owned
- `packages/native/src/viewport.rs` (NEW)
- `packages/native/src/window_role.rs` (NEW)
- `packages/native/src/aether_renderer.rs`
- `packages/native/src/aether_application.rs`
- `packages/native/src/config.rs`
- `packages/native/src/contexts.rs`
- `packages/native/src/lib.rs`
- `packages/config-macro/src/lib.rs`
- `packages/aether/Cargo.toml`
- `packages/aether/src/launch.rs`
- `packages/aether/src/lib.rs`

### Exact Implementation Tasks

#### 3.1 `EtalonSurfaceWidget` & `EtalonSurfaceDelegate` (`packages/native/src/viewport.rs`)
- Create `packages/native/src/viewport.rs` (available across all native renderer profiles, with GPU delegate execution gated on `#[cfg(any(feature = "vello", feature = "vello-hybrid"))]`).
- Define:
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub enum EtalonSurfaceError {
      NoGpuContextAvailable,
      TextureAllocationFailed(String),
      ResourceRegistrationFailed(String),
      DeviceLost(String),
      RenderPassFailed(String),
  }

  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum RenderOutcome {
      /// Scene is static until the next input or resize.
      Static,
      /// Request another animation frame redraw.
      ContinuousRedraw,
  }

  #[derive(Clone, Debug, PartialEq, Eq)]
  pub enum EtalonViewportState {
      Suspended,
      Active,
      CpuFallbackNotice(String),
      Faulted(EtalonSurfaceError),
  }
  ```
- When `#[cfg(any(feature = "vello", feature = "vello-hybrid"))]` is enabled, define:
  ```rust
  pub struct EtalonFrameContext<'a> {
      pub device: &'a etalon::Device,
      pub queue: &'a etalon::Queue,
      pub target_view: &'a etalon::TextureView,
      pub width: u32,
      pub height: u32,
      pub scale: f64,
      pub frame_index: u64,
  }

  pub trait EtalonSurfaceDelegate: 'static {
      fn prepare(&mut self, device: &etalon::Device, queue: &etalon::Queue) -> Result<(), EtalonSurfaceError>;
      fn render_frame(&mut self, ctx: &EtalonFrameContext<'_>) -> Result<RenderOutcome, EtalonSurfaceError>;
      fn handle_event(&mut self, _event: &blitz_traits::events::UiEvent, _scale: f64) {}
      fn focus_changed(&mut self, _focused: bool) {}
      fn intrinsic_sizes(&self) -> blitz_dom::layout::replaced::IntrinsicSizes {
          Default::default()
      }
      fn destroy(&mut self) {}
  }
  ```
- Implement `pub struct EtalonSurfaceWidget` (`impl blitz_dom::Widget for EtalonSurfaceWidget`):
  - In `can_create_surfaces`: safely inspect `render_ctx.renderer_specific_context()` and downcast to `etalon_context::DeviceHandle`. If unavailable (e.g., running on CPU fallback), transition cleanly to `EtalonViewportState::CpuFallbackNotice("CPU raster backend active".into())` instead of panicking.
  - In `paint`:
    - Never call `.unwrap()` on `render_ctx.try_register_custom_resource(...)`; on `Err(err)`, transition to `EtalonViewportState::Faulted(EtalonSurfaceError::ResourceRegistrationFailed(err.to_string()))` and return an empty or fallback `Scene`.
    - Double-buffer `etalon::Texture` (`TextureFormat::Rgba8Unorm`, `RENDER_ATTACHMENT | TEXTURE_BINDING`), unregistering previous textures via `render_ctx.unregister_resource(old_id)` on resize, `destroy_surfaces`, and `disconnected`.
    - Forward `handle_event` (passing cached `last_scale: f64`) and `focus_changed` to the delegate.

#### 3.2 Renderer Health Inspection & Fallible GPU Preflight (`packages/native/src/aether_renderer.rs`)
- Add `RendererFallbackPolicy` and `RendererHealth`:
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
  pub enum RendererFallbackPolicy {
      #[default]
      CpuOnFailure,
      FailClosed,
  }

  #[derive(Clone, Debug)]
  pub struct RendererHealth {
      pub backend_name: &'static str,
      pub gpu_active: bool,
      pub faulted: bool,
      pub last_error: Option<String>,
  }
  ```
- Add `pub fallback_policy: RendererFallbackPolicy` to `RendererOptions`.
- Expose `pub fn health(&self) -> RendererHealth` and `pub fn record_fault(&self, reason: impl Into<String>)` on `AetherNativeWindowRenderer`.

#### 3.3 Window Roles, No `"Aether App"` Identity Leak & `NativeAppHandle` Multi-Window Controller (`config.rs`, `window_role.rs`, `aether_application.rs`)
- In `packages/native/src/config.rs`:
  - Fix line 22: replace `.unwrap_or_else(|| "Aether App".to_string())` with `.unwrap_or_default()` so Aether never injects `"Aether App"` as product identity.
- Create `packages/native/src/window_role.rs`:
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
  pub enum WindowRole {
      #[default]
      StandardAppWindow,
      /// Frameless, transparent, always-on-top radial 8-Sector Command Orb (◉) overlay.
      OrbOverlay,
      /// Always-on-top urgent modal (e.g., WellFair remote carer alert).
      UrgentAlertModal,
      /// Floating auxiliary palette or inspector window.
      AuxiliaryToolPalette,
  }

  #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
  pub enum WindowClosePolicy {
      #[default]
      CloseAndDrop,
      /// Hide window on close request instead of destroying its VirtualDom (used for OrbOverlay).
      HideOnClose,
  }
  ```
  - Implement `WindowRole::apply_to_attributes(self, attrs: WindowAttributes) -> (WindowAttributes, Option<CompositeAlphaMode>, Option<Color>)`:
    - `OrbOverlay`: `with_decorations(false)`, `with_transparent(true)`, `with_window_level(WindowLevel::AlwaysOnTop)`, `with_resizable(false)`, `CompositeAlphaMode::PreMultiplied` (or `Transparent`), `Color::TRANSPARENT`.
    - `UrgentAlertModal`: `with_window_level(WindowLevel::AlwaysOnTop)`, `with_active(true)`.
    - `AuxiliaryToolPalette`: `with_window_level(WindowLevel::AlwaysOnTop)`, `with_resizable(true)`.
  - Add `.with_window_role(self, role: WindowRole) -> Self` and `.with_close_policy(self, policy: WindowClosePolicy) -> Self` on `Config`.
- In `packages/native/src/aether_application.rs`:
  - Add `AetherNativeEvent::OpenAuxiliaryWindow(Box<WindowConfig<AetherNativeWindowRenderer>>)` and `AetherNativeEvent::CloseWindow(WindowId)` and `AetherNativeEvent::SetWindowVisible { window: WindowId, visible: bool }`.
  - Create and inject into every window's `ScopeId::ROOT` a `#[derive(Clone)] pub struct NativeAppHandle` (accessible via `pub fn use_native_app() -> NativeAppHandle`):
    - `pub fn open_window(&self, vdom: VirtualDom, config: Config)`
    - `pub fn open_component_window(&self, app: fn() -> Element, config: Config)`
    - `pub fn request_close(&self, window_id: WindowId)`
    - `pub fn set_window_visible(&self, window_id: WindowId, visible: bool)`
  - Honor `WindowClosePolicy::HideOnClose` in `AetherNativeApplication::window_event` when `WindowEvent::CloseRequested` fires on a registered `HideOnClose` window (`window.window.set_visible(false)` instead of removing the window).
- In `packages/native/src/lib.rs`:
  - Add `pub fn try_launch_cfg_once<P: 'static>(app: impl FnOnce(P) -> Element + 'static, props: P, contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>, configs: Vec<Box<dyn Any>>) -> Result<(), LaunchError>` so callers with non-`Clone` root startup props can pass them cleanly via a root context provider without requiring `P: Clone`.

#### 3.4 Fix `client!`, Deprecation Guard & `mobile-native` Routing (`packages/config-macro/src/lib.rs`, `packages/aether/Cargo.toml`, `packages/aether/src/launch.rs`, `packages/aether/src/lib.rs`)
- In `packages/config-macro/src/lib.rs:25`:
  - Change:
    ```rust
    define_config_macro!(client if any(feature = "desktop", feature = "web", feature = "mobile", feature = "native"));
    ```
- In `packages/aether/src/launch.rs:88-98`:
  - Include `feature = "native"` inside the `not(any(...))` deprecation guard so enabling only `features = ["native"]` never triggers the `"No renderer is enabled"` deprecation warning.
- In `packages/aether/Cargo.toml`:
  - Add feature:
    ```toml
    mobile-native = ["native", "aether-config-macro/mobile"]
    ```
- In `packages/aether/src/lib.rs`:
  - Under `#[cfg(feature = "native")]` in `pub mod prelude`, re-export `aether_native::{Config as NativeConfig, NativeAppHandle, WindowClosePolicy, WindowRole, use_native_app, use_window}`.

---

## WP4: Interactive Input/Focus Forwarding & Coalesced Transport for `ChoraFrameWidget`

### Files Owned
- `packages/chora-bridge-protocol/src/types.rs`
- `packages/chora-bridge-protocol/src/state.rs`
- `packages/chora-bridge-protocol/src/tests.rs`
- `packages/chora-client/src/lib.rs`
- `packages/chora-client/src/tests.rs`
- `packages/chora-worker/src/main.rs`
- `packages/native/src/chora_frame.rs`

### Exact Implementation Tasks

#### 4.1 Wire Input & Focus Protocol Types (`packages/chora-bridge-protocol/src/types.rs` & `state.rs`)
- Keep `aether-chora-bridge-protocol` 100% pure Ring B wire types (zero Servo/MPL dependencies).
- Add bounded `#[serde(deny_unknown_fields)]` input types in `types.rs`:
  ```rust
  pub const MAX_KEY_BYTES: usize = 64;

  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub enum PointerAction {
      Move,
      Down,
      Up,
      Leave,
      Cancel,
  }

  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub enum PointerButton {
      Main,
      Auxiliary,
      Secondary,
      Fourth,
      Fifth,
  }

  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(deny_unknown_fields)]
  pub struct InputModifiers {
      pub shift: bool,
      pub ctrl: bool,
      pub alt: bool,
      pub meta: bool,
  }

  #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
  #[serde(deny_unknown_fields)]
  pub struct WirePointerEvent {
      pub action: PointerAction,
      pub x_px: f32,
      pub y_px: f32,
      pub button: Option<PointerButton>,
      pub buttons: u8,
      pub modifiers: InputModifiers,
  }

  #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
  #[serde(deny_unknown_fields)]
  pub struct WireWheelEvent {
      pub delta_x: f64,
      pub delta_y: f64,
      pub delta_is_lines: bool,
      pub x_px: f32,
      pub y_px: f32,
  }

  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub enum WireKeyState {
      Down,
      Up,
  }

  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(deny_unknown_fields)]
  pub struct WireKeyEvent {
      pub state: WireKeyState,
      pub key: String,
      pub code: String,
      pub location: u8,
      pub modifiers: InputModifiers,
      pub repeat: bool,
      pub is_composing: bool,
  }

  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub enum WireImeEvent {
      Preedit {
          text: String,
          cursor_start: Option<u32>,
          cursor_end: Option<u32>,
      },
      Commit {
          text: String,
      },
      Dismiss,
  }
  ```
- Add variants to `pub enum Control`:
  ```rust
  InputPointer(WirePointerEvent),
  InputWheel(WireWheelEvent),
  InputKey(WireKeyEvent),
  InputIme(WireImeEvent),
  SetFocus { focused: bool },
  ```
- In `ControlEnvelope::validate(&self)`:
  - Validate `InputPointer`: `x_px.is_finite() && y_px.is_finite()`.
  - Validate `InputWheel`: `delta_x.is_finite() && delta_y.is_finite() && x_px.is_finite() && y_px.is_finite()`.
  - Validate `InputKey`: `key.len() <= MAX_KEY_BYTES && code.len() <= MAX_KEY_BYTES && location <= 3`.
  - Validate `InputIme`: `text.len() <= MAX_PAYLOAD_BYTES`, and for `Preedit`, if `cursor_start`/`cursor_end` are present, `start <= end && (end as usize) <= text.len()`.
- In `PeerGuard::check` (`packages/chora-bridge-protocol/src/state.rs:72-95`):
  - Allow `Control::InputPointer(_) | Control::InputWheel(_) | Control::InputKey(_) | Control::InputIme(_) | Control::SetFocus { .. }` when `from_parent == true` (and reject if `from_parent == false`).
- Add unit tests in `packages/chora-bridge-protocol/src/tests.rs` verifying validation and `PeerGuard` direction enforcement for all 5 new `Control` messages.

#### 4.2 Non-Blocking Route-Stamped Input Forwarding in `ChoraClient` (`packages/chora-client/src/lib.rs` & `tests.rs`)
- Add non-blocking input sender methods on `ChoraClient`:
  - `pub fn send_pointer(&self, expected_stamp: RouteStamp, event: WirePointerEvent) -> Result<(), ProtocolError>`
  - `pub fn send_wheel(&self, expected_stamp: RouteStamp, event: WireWheelEvent) -> Result<(), ProtocolError>`
  - `pub fn send_key(&self, expected_stamp: RouteStamp, event: WireKeyEvent) -> Result<(), ProtocolError>`
  - `pub fn send_ime(&self, expected_stamp: RouteStamp, event: WireImeEvent) -> Result<(), ProtocolError>`
  - `pub fn set_focus(&self, expected_stamp: RouteStamp, focused: bool) -> Result<(), ProtocolError>`
- **Invariants**:
  - Check `state.ready && !state.closed && !state.exited`.
  - Verify `state.stamp == expected_stamp` (return `Err(ProtocolError::StaleFrame)` if the route/viewport stamp has transitioned, preventing stale clicks from hitting a new navigation origin).
  - For `PointerAction::Move`, if the bounded `output` channel is nearly full, drop or coalesce redundant move packets rather than poisoning or closing the session.
- Add unit tests in `packages/chora-client/src/tests.rs` testing stamp mismatch rejection (`ProtocolError::StaleFrame`) and input delivery.

#### 4.3 Worker Input Dispatch (`packages/chora-worker/src/main.rs`)
- In `packages/chora-worker/src/main.rs` (`Control` match arm):
  - Handle `Control::InputPointer`, `Control::InputWheel`, `Control::InputKey`, `Control::InputIme`, and `Control::SetFocus { focused }` **only when `next == current`** (discarding stale-stamp inputs before a resize or navigation settles):
    - `SetFocus { focused }` -> `view.set_focused(focused)`
    - `InputPointer(ev)` -> translate to `chora::InputEvent::MouseMove` / `MouseButton` / `MouseLeftViewport` at `WebViewPoint::Device(DevicePoint::new(ev.x_px, ev.y_px))` and call `view.notify_input_event(...)`
    - `InputWheel(ev)` -> translate to `chora::InputEvent::Wheel(...)` at `WebViewPoint::Device(DevicePoint::new(ev.x_px, ev.y_px))` and call `view.notify_input_event(...)`
    - `InputKey(ev)` -> translate to `chora::InputEvent::Keyboard(...)` and call `view.notify_input_event(...)`
    - `InputIme(ev)` -> translate to `chora::InputEvent::Ime(...)` and call `view.notify_input_event(...)`

#### 4.4 `ChoraFrameWidget` Hit-Test Input & Focus Translation (`packages/native/src/chora_frame.rs`)
- Update `ChoraFrameWidget` in `packages/native/src/chora_frame.rs`:
  - Add fields:
    ```rust
    last_scale: f64,
    last_stamp: Option<RouteStamp>,
    focused: bool,
    last_move_px: Option<(i32, i32)>,
    ```
  - In `Widget::paint`: record `self.last_scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };` and `self.last_stamp = Some(stamp);`.
  - Implement `Widget::focus_changed(&mut self, focused: bool)`:
    - Update `self.focused = focused;` and if `let Some(stamp) = self.last_stamp`, call `let _ = self.client.set_focus(stamp, focused);`.
  - Implement `Widget::handle_event(&mut self, event: &blitz_traits::events::UiEvent)`:
    - Retrieve `let Some(stamp) = self.last_stamp else { return; };`.
    - Convert logical CSS coordinates to device physical pixels (`x_px = (coords.x as f64 * self.last_scale) as f32`, `y_px = (coords.y as f64 * self.last_scale) as f32`).
    - Coalesce sub-pixel identical `PointerMove` coordinates (`(x_px as i32, y_px as i32) == last_move_px`).
    - Forward `UiEvent::PointerMove/Down/Up/Cancel/Leave`, `UiEvent::Wheel`, `UiEvent::KeyDown/KeyUp`, and `UiEvent::Ime` to `self.client.send_pointer / send_wheel / send_key / send_ime` with `stamp`.

---

## WP5: `packages/qualia-host` Ergonomic Declarative Hooks & Holographic 10D v2 Viewport Delegate

### Files Owned
- `packages/qualia-host/Cargo.toml`
- `packages/qualia-host/src/lib.rs`
- `packages/qualia-host/src/hooks.rs` (NEW)
- `packages/qualia-host/src/holographic_viewport.rs` (NEW)

### Exact Implementation Tasks

#### 5.1 Dependency Update (`packages/qualia-host/Cargo.toml`)
- Add `aether-signals = { path = "../signals", version = "=0.8.0-alpha.3" }` to `[dependencies]` in `packages/qualia-host/Cargo.toml` (keeping `packages/qualia-host` in its standalone workspace).

#### 5.2 Ergonomic Declarative Hooks Preserving All Authority Invariants (`packages/qualia-host/src/hooks.rs`)
- Implement `packages/qualia-host/src/hooks.rs` and export via `pub mod hooks;` and `pub mod prelude;` in `packages/qualia-host/src/lib.rs`:
  1. **`QualiaHostContext` & `use_qualia_context()` / `try_use_qualia_context()`**:
     - Holds `authority: Arc<NativeAuthority>`, optional `authoring: Option<NativeAuthoringBridge>`, and optional `action_dispatcher: Option<Arc<dyn VibeActionDispatcher>>`.
     - **Security invariant**: No `Default` implementation, no mock authority constructor; requires an adopted `Arc<NativeAuthority>`.
  2. **`VibeActionDispatcher` & `use_vibe_action(create_receiver, max_per_turn)`**:
     - Defines:
       ```rust
       pub trait VibeActionDispatcher: Send + Sync + 'static {
           fn dispatch_action(
               &self,
               program_id: &str,
               entry_fn: &str,
               args: Vec<vibe::Value>,
               cancel_token: Arc<AtomicBool>,
           ) -> Result<u64, HostError>;
       }

       #[derive(Clone, Debug)]
       pub enum VibeActionState {
           Idle,
           Running { epoch: u64 },
           Delivered(UiDelivery),
           Failed(String),
           /// Surfaces committed receipts when an error occurs after engine entry.
           NeedsReconcile {
               cause: String,
               committed_receipts: Vec<vibescript_abi::ExecutionReceiptHeader>,
           },
           Cancelled,
       }
       ```
     - Wraps `use_qualia_delivery` with reactive `aether_signals::Signal<VibeActionState>`, explicit `cancel()` (setting `AtomicBool` and invalidating the receiver epoch), and explicit `record_host_error(&mut self, err: HostError)` that routes `HostError::Reconcile { cause, committed_receipts }` into `VibeActionState::NeedsReconcile` so committed receipts are never lost.
  3. **`use_qualia_subscription<T, F>(create_view, cancelled, next_tick, project)`**:
     - Wraps `use_qualia_graph` with reactive `Signal<Option<ProjectedGraphSnapshot<T>>>` and `Signal<GraphSubscriptionStatus>`.
     - `project: impl for<'a> FnMut(&GraphSnapshot<'a>) -> T + 'static` projects bounded domain data out of `GraphSnapshot<'a>` on the UI owner thread.
     - **Revocation / Stale Clearing Invariant**: On `Err(GraphError::Busy)`, sets `status = GraphSubscriptionStatus::BusyPreserved` and keeps the last projected snapshot. On **any** other `GraphError` (`Authentication`, `Stale`, `Cancelled`, `Closed`, etc.), **immediately sets the snapshot `Signal` to `None`** so revoked private graph data is erased from UI state.
  4. **`use_authoring_bridge<F>(create_view, next_tick)`**:
     - Wraps `NativeAuthoringView` in a component-owned handle with `evidence: Signal<Option<WorkflowEvidence>>`, `last_review: Signal<Option<SourceReviewRequest>>`, and `status: Signal<AuthoringBridgeStatus>`.
     - Provides `request_source_review(&self, id: &str, evidence_ref: &str) -> Result<SourceReviewRequest, AuthoringBridgeError>` on the UI thread, clearing `evidence` immediately on `AuthoringBridgeError::{Closed, StaleScope, PublicationUnavailable}` and closing the view on component drop.
- Add unit tests in `packages/qualia-host/src/hooks.rs` verifying:
  - Revocation/authentication error immediately clears `use_qualia_subscription`'s projected snapshot signal while `GraphError::Busy` preserves it.
  - `HostError::Reconcile` transitions `use_vibe_action` to `VibeActionState::NeedsReconcile` with all `committed_receipts` preserved.
  - Component unmount/drop cancels `use_qualia_subscription` and closes `NativeAuthoringView`.

#### 5.3 Holographic 10D v2 Viewport Spec & Validator (`packages/qualia-host/src/holographic_viewport.rs`)
- Create `packages/qualia-host/src/holographic_viewport.rs` binding `qualia_core_db::container_10d::holographic_phase_section::{FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE, HOLOGRAPHIC_MAX_RAYMARCH_STEPS, HolographicPhaseHeader, M64HoloRecord, EulerianWavefieldRecord}` for native viewport rendering:
  - Validate that every bound `.10d` payload carries `FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE = 0x0306` (`HOLO`), a 64-byte cache-line-aligned `HolographicPhaseHeader`, and enforces `raymarch_steps <= HOLOGRAPHIC_MAX_RAYMARCH_STEPS` (`32`).
  - Provide `Holographic10dViewportState` (`HeaderVerified`, `MissingHoloSection`, `RaymarchBudgetExceeded { requested: u32, max: u32 }`) and a 12-triangle bounding-box proxy lens synthesizer for Etalon's `< 32`-step raymarching pipeline, plus unit tests.

---

## WP6: `refract` ↔ Aether VirtualDOM Reactive Lens Bridge & Semantic UI RSX Components

### Files Owned
- `refract/src/lib.rs`
- `refract/src/ui.rs`
- `refract/src/ui_scheduler.rs`
- `refract/src/dom.rs`
- `refract/src/semantic_ui.rs`
- `packages/aether/src/refract_bridge.rs` (NEW)

### Exact Implementation Tasks

#### 6.1 Dirty-Path Flush Tracking & Read-Only DOM Visitor in `refract` (`refract/src/{ui.rs, ui_scheduler.rs, dom.rs, semantic_ui.rs, lib.rs}`)
- Keep `refract` 100% dependency-free (`#![warn(missing_docs)]` is enabled in `refract/src/lib.rs` — **every public item, struct, enum, variant, field, and method in `refract` MUST have a doc comment!**).
- In `refract/src/ui.rs` & `ui_scheduler.rs`:
  - Add `flushed_dirty_paths: Vec<InlinePath>` to `Ui<S>`.
  - Whenever `notify_state_written(path)` runs during `Ui::with` / `Ui::flush`, push `path` into `flushed_dirty_paths` (if not already present).
  - Expose:
    ```rust
    /// Drain the list of state [`InlinePath`] projections mutated since the last drain.
    pub fn take_dirty_paths(&mut self) -> Vec<InlinePath>
    ```
- In `refract/src/quin_lens.rs` (or `lens.rs`):
  - Add a non-panicking `InlinePath::try_push(&mut self, index: usize) -> bool` helper and document the `MAX_INLINE_PATH_DEPTH = 8` depth rule (`QuinWindow` + `QuinSlotLens` + `QuinFieldLens` uses 3 segments, leaving up to 5 parent `Field` nesting levels).
- Export all `semantic_ui` types (`InstrumentReadiness`, `WellFairDimension`, `DimensionOutcome`, `FairnessReviewView`, `AssertionKind`, `ChallengeState`, `ContextualAssertionView`, `ContributionStage`, `ContributionStageView`) and their helper methods (`status_code()`, `human_label()`, `sanitize_hold_reason()`, etc.) cleanly.

#### 6.2 `packages/aether/src/refract_bridge.rs` (`#[cfg(feature = "refract")]`)
- Create `packages/aether/src/refract_bridge.rs` and export it in `packages/aether/src/lib.rs` under `#[cfg(feature = "refract")] pub mod refract_bridge;` (and in `aether::prelude` when `feature = "refract"` is enabled):
  1. **`RefractStore<S: 'static>`**:
     - Wraps `Rc<RefCell<RefractStoreInner<S>>>` containing `ui: refract::Ui<S>` and `subscribers: Vec<(refract::InlinePath, aether_signals::ReactiveContext)>`.
     - **`pub fn use_refract_store<S: 'static>(init: impl FnOnce() -> S) -> RefractStore<S>`**: initializes and provides `RefractStore<S>` in the current component scope.
     - **`pub fn read<L: refract::Lens<S>, R>(&self, lens: L, f: impl FnOnce(&L::Target) -> R) -> R`**:
       - If `ReactiveContext::current()` is `Some(rc)`, registers `(lens.path(), rc)` in `subscribers`.
       - Borrows `ui.get(lens)` and returns `f(&*guard)`.
     - **`pub fn get_cloned<L: refract::Lens<S>>(&self, lens: L) -> L::Target where L::Target: Clone`**:
       - Convenience wrapper around `self.read(lens, Clone::clone)`.
     - **`pub fn write<L: refract::Lens<S>>(&self, lens: L, f: impl FnOnce(&mut L::Target))`**:
       - Mutates via `ui.with(|ctx| f(&mut *ctx.write(lens)))`, drains `let dirty_paths = inner.ui.take_dirty_paths();`, collects all `ReactiveContext`s in `subscribers` where `dirty_paths.iter().any(|dp| refract::paths_overlap(&sub_path, dp))`, **drops the `RefMut` borrow first** (preventing any `RefCell` reentrancy panic!), and then calls `rc.mark_dirty()` on each matched subscriber.
  2. **Zero-Heap 6-Column `NQuin` Lens Helpers**:
     - `pub fn read_quin_slot<S: 'static, L, const N: usize>(&self, window_lens: L, slot: usize) -> refract::QuinSlot where L: refract::Lens<S, Target = refract::QuinWindow<N>>`
     - `pub fn read_quin_field<S: 'static, L, const N: usize>(&self, window_lens: L, slot: usize, field: u8) -> u64 where L: refract::Lens<S, Target = refract::QuinWindow<N>>`
     - `pub fn write_quin_field<S: 'static, L, const N: usize>(&self, window_lens: L, slot: usize, field: u8, value: u64) where L: refract::Lens<S, Target = refract::QuinWindow<N>>`
  3. **First-Class Aether RSX Semantic UI Components (`#[cfg(all(feature = "refract", feature = "html", feature = "macro"))]`)**:
     - Render `refract::semantic_ui` views directly as native Aether `Element` RSX trees preserving every epistemic and governance invariant:
       - `pub fn render_instrument_readiness_card(instrument_id: &str, readiness: InstrumentReadiness) -> aether_core::Element` (enforcing `collect != activate`, `data-inspect-before-fetch="true"`, and sanitized `Held { why }` copy that never renders `"broken"`).
       - `pub fn render_fairness_review_panel(review: &FairnessReviewView) -> aether_core::Element` (rendering all 6 `WellFairDimension` outcomes and remedies with `data-collapsed-score="forbidden"`).
       - `pub fn render_contextual_assertion_badge(assertion: &ContextualAssertionView) -> aether_core::Element` (enforcing epistemic separation across `Supported`, `Attested`, `Contextual`, `Heuristic`, `Simulated` and `ChallengeState`).
       - `pub fn render_contribution_lifecycle_bar(view: &ContributionStageView) -> aether_core::Element` (enforcing `data-pool-equals-paid="false"` and distinct `pool_funded_minor` vs. `contributor_paid_minor`).
- Add unit tests in `packages/aether/src/refract_bridge.rs` verifying:
  - Writing to slot `0` of a `QuinWindow<8>` marks dirty only components reading slot `0`, leaving components reading slot `1` clean (`paths_overlap` granularity).
  - All 4 RSX semantic UI renderers preserve their domain attributes and invariants.

---

## 7. Coordinator Post-Codex Verification Matrix (For Antigravity Only — NOT for Codex)

Once Codex reports completion of `WP1`–`WP6`, the Antigravity Coordinator will review the diffs and execute:

1. `cargo test --locked -p refract`
2. `cargo test --locked -p aether-native-dom`
3. `cargo check --locked -p aether-native`
4. `cargo check --locked -p aether-native --no-default-features --features vello,svg`
5. `cargo check --locked -p aether-native --no-default-features --features vello-cpu-softbuffer,svg`
6. `cargo test --locked -p aether --features native,refract`
7. `cargo test --manifest-path packages/chora-bridge-protocol/Cargo.toml`
8. `cargo test --manifest-path packages/chora-client/Cargo.toml`
9. `cargo check --manifest-path packages/chora-worker/Cargo.toml`
10. `cargo test --manifest-path packages/qualia-host/Cargo.toml`
