use aether_html::{
    AnimationData, BeforeInputData, CancelData, ClipboardData, CompositionData, DragData,
    FocusData, FormData, FormValue, HasCompositionData, HasFileData, HasFocusData, HasFormData,
    HasKeyboardData, HasMouseData, HasPointerData, HasScrollData, HasTouchData, HasTouchPointData,
    HasWheelData, HtmlEventConverter, ImageData, KeyboardData, MediaData, MountedData,
    MountedError, MountedResult, MouseData, PlatformEventData, PointerData, RenderedElementBacking,
    ResizeData, ScrollBehavior, ScrollData, ScrollLogicalPosition, ScrollToOptions, SelectionData,
    ToggleData, TouchData, TouchPoint, TransitionData, VisibleData, WheelData,
    geometry::{
        ClientPoint, ElementPoint, PagePoint, PixelsRect, PixelsSize, PixelsVector2D, ScreenPoint,
        WheelDelta,
        euclid::{Point2D, Size2D, Vector3D},
    },
    input_data::{MouseButton, MouseButtonSet},
    point_interaction::{
        InteractionElementOffset, InteractionLocation, ModifiersInteraction, PointerInteraction,
    },
};
use blitz_dom::{
    BaseDocument, Node, ScrollBehavior as BlitzScrollBehavior,
    ScrollLogicalPosition as BlitzScrollLogicalPosition,
};
use blitz_traits::events::{
    BlitzImeEvent, BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, BlitzScrollEvent,
    BlitzWheelDelta, BlitzWheelEvent, DomEventData, MouseEventButton,
};
use keyboard_types::{Code, Key, Location, Modifiers};
use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    fmt::Display,
    future::Future,
    pin::Pin,
    rc::Rc,
};

use crate::NodeId;

pub struct NativeConverter {}

/// A native event for which the current backend supplies no application payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedNativeEvent {
    event_name: String,
}

impl UnsupportedNativeEvent {
    fn new(event_name: &str) -> Self {
        Self {
            event_name: event_name.to_owned(),
        }
    }

    /// The event that could not be supplied by the native backend.
    pub fn event_name(&self) -> &str {
        &self.event_name
    }
}

impl Display for UnsupportedNativeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "native event {} is unsupported by the current backend",
            self.event_name
        )
    }
}

impl std::error::Error for UnsupportedNativeEvent {}

/// Check the native backend's application event capability before requesting an event.
///
/// This reports payload/routing support, not platform delivery or web compatibility.
/// In particular, clipboard, drag, beforeinput and animation events are not emitted.
pub fn native_event_support(name: &str) -> Result<(), UnsupportedNativeEvent> {
    match name.strip_prefix("on").unwrap_or(name) {
        "pointermove" | "pointerdown" | "pointerup" | "pointercancel" | "pointerenter"
        | "pointerleave" | "pointerover" | "pointerout" | "mousemove" | "mousedown" | "mouseup"
        | "mouseenter" | "mouseleave" | "mouseover" | "mouseout" | "touchstart" | "touchmove"
        | "touchend" | "touchcancel" | "scroll" | "wheel" | "click" | "contextmenu"
        | "dblclick" | "keydown" | "keyup" | "keypress" | "input" | "focus" | "blur"
        | "focusin" | "focusout" | "mounted" | "compositionstart" | "compositionupdate"
        | "compositionend" => Ok(()),
        _ => Err(UnsupportedNativeEvent::new(name)),
    }
}

impl NativeConverter {
    /// This exhaustive boundary pairs each emitted event with its actual native payload.
    /// IME lifecycle events are expanded by `NativeCompositionState` instead.
    pub(crate) fn convert_dom_event(
        event: &DomEventData,
    ) -> Result<PlatformEventData, UnsupportedNativeEvent> {
        let data: Box<dyn Any> = match event {
            DomEventData::PointerMove(event)
            | DomEventData::PointerDown(event)
            | DomEventData::PointerUp(event)
            | DomEventData::PointerCancel(event)
            | DomEventData::PointerLeave(event)
            | DomEventData::PointerEnter(event)
            | DomEventData::PointerOver(event)
            | DomEventData::PointerOut(event)
            | DomEventData::MouseMove(event)
            | DomEventData::MouseDown(event)
            | DomEventData::MouseUp(event)
            | DomEventData::MouseLeave(event)
            | DomEventData::MouseEnter(event)
            | DomEventData::MouseOver(event)
            | DomEventData::MouseOut(event)
            | DomEventData::Click(event)
            | DomEventData::ContextMenu(event)
            | DomEventData::DoubleClick(event) => Box::new(NativePointerData(event.clone())),
            DomEventData::TouchStart(event)
            | DomEventData::TouchMove(event)
            | DomEventData::TouchEnd(event)
            | DomEventData::TouchCancel(event) => Box::new(NativeTouchData(event.clone())),
            DomEventData::Scroll(event) => Box::new(NativeScrollData(event.clone())),
            DomEventData::Wheel(event) => Box::new(NativeWheelData(event.clone())),
            DomEventData::Focus(_)
            | DomEventData::Blur(_)
            | DomEventData::FocusIn(_)
            | DomEventData::FocusOut(_) => Box::new(NativeFocusData),
            DomEventData::KeyDown(event)
            | DomEventData::KeyUp(event)
            | DomEventData::KeyPress(event) => Box::new(BlitzKeyboardData(event.clone())),
            DomEventData::Input(event) => Box::new(NativeFormData {
                value: event.value.clone(),
                values: vec![],
            }),
            DomEventData::Ime(_) | DomEventData::AppleStandardKeybinding(_) => {
                return Err(UnsupportedNativeEvent::new(event.name()));
            }
        };
        native_event_support(event.name())?;
        Ok(PlatformEventData::new(data))
    }
}

/// Text supplied by an actual native composition lifecycle event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeCompositionData(pub(crate) String);

impl HasCompositionData for NativeCompositionData {
    fn data(&self) -> String {
        self.0.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub(crate) struct NativeCompositionEvent {
    pub(crate) target: NodeId,
    pub(crate) name: &'static str,
    pub(crate) data: NativeCompositionData,
}

/// IME availability is distinct from an active composition. Keep this state even
/// when no listener exists, so listeners added during composition see correct phases.
#[derive(Default)]
pub(crate) struct NativeCompositionState {
    target: Option<NodeId>,
}

impl NativeCompositionState {
    pub(crate) fn handle(
        &mut self,
        target: NodeId,
        event: &BlitzImeEvent,
    ) -> Result<Vec<NativeCompositionEvent>, UnsupportedNativeEvent> {
        let mut events = Vec::new();
        if let Some(previous) = self.target.filter(|previous| *previous != target) {
            if let Some(event) = self.cancel(previous) {
                events.push(event);
            }
        }
        match event {
            BlitzImeEvent::Enabled => {}
            BlitzImeEvent::Preedit(text, _) => {
                if self.target != Some(target) {
                    if text.is_empty() {
                        return Ok(events);
                    }
                    self.target = Some(target);
                    events.push(NativeCompositionEvent {
                        target,
                        name: "compositionstart",
                        data: NativeCompositionData(String::new()),
                    });
                }
                events.push(NativeCompositionEvent {
                    target,
                    name: "compositionupdate",
                    data: NativeCompositionData(text.clone()),
                });
            }
            BlitzImeEvent::Commit(text) => {
                if self.target.take() == Some(target) {
                    events.push(NativeCompositionEvent {
                        target,
                        name: "compositionend",
                        data: NativeCompositionData(text.clone()),
                    });
                }
            }
            BlitzImeEvent::Disabled => {
                if let Some(event) = self.cancel(target) {
                    events.push(event);
                }
            }
            // The native editor deletes committed surrounding text and emits
            // input. This operation leaves the composition lifecycle intact.
            BlitzImeEvent::DeleteSurrounding { .. } => {}
        }
        Ok(events)
    }

    pub(crate) fn cancel(&mut self, target: NodeId) -> Option<NativeCompositionEvent> {
        if self.target != Some(target) {
            return None;
        }
        self.target = None;
        Some(NativeCompositionEvent {
            target,
            name: "compositionend",
            data: NativeCompositionData(String::new()),
        })
    }

    pub(crate) fn retain_target(&mut self, mut exists: impl FnMut(NodeId) -> bool) {
        if self.target.is_some_and(|target| !exists(target)) {
            self.target = None;
        }
    }
}

// The inherited converter trait is infallible. Unsupported event families cannot
// return absence here without fabricating payloads. `convert_dom_event` and the
// composition state machine are the checked dispatch boundary; callers querying
// capabilities must use `native_event_support` instead of invoking these stubs.
impl HtmlEventConverter for NativeConverter {
    fn convert_cancel_data(&self, _event: &PlatformEventData) -> CancelData {
        unimplemented!("todo: convert_cancel_data in aether-native. requires support in blitz")
    }

    fn convert_form_data(&self, event: &PlatformEventData) -> FormData {
        event.downcast::<NativeFormData>().unwrap().clone().into()
    }

    fn convert_mouse_data(&self, event: &PlatformEventData) -> MouseData {
        event
            .downcast::<NativePointerData>()
            .unwrap()
            .clone()
            .into()
    }

    fn convert_keyboard_data(&self, event: &PlatformEventData) -> KeyboardData {
        event
            .downcast::<BlitzKeyboardData>()
            .unwrap()
            .clone()
            .into()
    }

    fn convert_focus_data(&self, _event: &PlatformEventData) -> FocusData {
        NativeFocusData {}.into()
    }

    fn convert_animation_data(&self, _event: &PlatformEventData) -> AnimationData {
        unimplemented!("todo: convert_animation_data in aether-native. requires support in blitz")
    }

    fn convert_before_input_data(&self, _event: &PlatformEventData) -> BeforeInputData {
        unimplemented!(
            "todo: convert_before_input_data in aether-native. requires support in blitz"
        )
    }

    fn convert_clipboard_data(&self, _event: &PlatformEventData) -> ClipboardData {
        unimplemented!("todo: convert_clipboard_data in aether-native. requires support in blitz")
    }

    fn convert_composition_data(&self, event: &PlatformEventData) -> CompositionData {
        event
            .downcast::<NativeCompositionData>()
            .expect("native composition payload")
            .clone()
            .into()
    }

    fn convert_drag_data(&self, _event: &PlatformEventData) -> DragData {
        unimplemented!("todo: convert_drag_data in aether-native. requires support in blitz")
    }

    fn convert_image_data(&self, _event: &PlatformEventData) -> ImageData {
        unimplemented!("todo: convert_image_data in aether-native. requires support in blitz")
    }

    fn convert_media_data(&self, _event: &PlatformEventData) -> MediaData {
        unimplemented!("todo: convert_media_data in aether-native. requires support in blitz")
    }

    fn convert_mounted_data(&self, event: &PlatformEventData) -> MountedData {
        event.downcast::<NodeHandle>().unwrap().clone().into()
    }

    fn convert_pointer_data(&self, event: &PlatformEventData) -> PointerData {
        event
            .downcast::<NativePointerData>()
            .unwrap()
            .clone()
            .into()
    }

    fn convert_scroll_data(&self, event: &PlatformEventData) -> ScrollData {
        event.downcast::<NativeScrollData>().unwrap().clone().into()
    }

    fn convert_selection_data(&self, _event: &PlatformEventData) -> SelectionData {
        unimplemented!("todo: convert_selection_data in aether-native. requires support in blitz")
    }

    fn convert_toggle_data(&self, _event: &PlatformEventData) -> ToggleData {
        unimplemented!("todo: convert_toggle_data in aether-native. requires support in blitz")
    }

    fn convert_touch_data(&self, event: &PlatformEventData) -> TouchData {
        event.downcast::<NativeTouchData>().unwrap().clone().into()
    }

    fn convert_transition_data(&self, _event: &PlatformEventData) -> TransitionData {
        unimplemented!("todo: convert_transition_data in aether-native. requires support in blitz")
    }

    fn convert_wheel_data(&self, event: &PlatformEventData) -> WheelData {
        event.downcast::<NativeWheelData>().unwrap().clone().into()
    }

    fn convert_resize_data(&self, _event: &PlatformEventData) -> ResizeData {
        unimplemented!("todo: convert_resize_data in aether-native. requires support in blitz")
    }

    fn convert_visible_data(&self, _event: &PlatformEventData) -> VisibleData {
        unimplemented!("todo: convert_visible_data in aether-native. requires support in blitz")
    }
}

#[derive(Clone)]
pub struct NodeHandle {
    pub(crate) doc: Rc<RefCell<BaseDocument>>,
    pub(crate) node_id: NodeId,
}

impl NodeHandle {
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn doc(&self) -> Ref<'_, BaseDocument> {
        self.doc.borrow()
    }

    /// Returns `None` if the document is currently mutably borrowed.
    /// Use this from background tasks to avoid panicking during event handling.
    pub fn try_doc(&self) -> Option<Ref<'_, BaseDocument>> {
        self.doc.try_borrow().ok()
    }

    pub fn doc_mut(&self) -> RefMut<'_, BaseDocument> {
        self.doc.borrow_mut()
    }

    pub fn node(&self) -> Ref<'_, Node> {
        Ref::map(self.doc.borrow(), |doc| {
            doc.get_node(self.node_id)
                .expect("Node does not exist in the Document")
        })
    }

    pub fn node_mut(&self) -> RefMut<'_, Node> {
        RefMut::map(self.doc.borrow_mut(), |doc| {
            doc.get_node_mut(self.node_id)
                .expect("Node does not exist in the Document")
        })
    }

    fn node_not_exist_err<T>(&self) -> Pin<Box<dyn Future<Output = MountedResult<T>>>> {
        let node_id = self.node_id;
        let err = MountedError::OperationFailed(Box::new(NodeNotExistErr(node_id)));
        Box::pin(async move { Err(err) })
    }
}

#[derive(Debug)]
struct NodeNotExistErr(NodeId);
impl Display for NodeNotExistErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The node {} does not exist", self.0)
    }
}
impl std::error::Error for NodeNotExistErr {}

impl RenderedElementBacking for NodeHandle {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_scroll_offset(&self) -> Pin<Box<dyn Future<Output = MountedResult<PixelsVector2D>>>> {
        let Some(scroll_offset) = self
            .doc()
            .get_node(self.node_id)
            .map(|node| *node.scroll_offset())
        else {
            return self.node_not_exist_err();
        };
        Box::pin(async move { Ok(PixelsVector2D::new(scroll_offset.x, scroll_offset.y)) })
    }

    fn get_scroll_size(&self) -> Pin<Box<dyn Future<Output = MountedResult<PixelsSize>>>> {
        let doc = self.doc();
        let Some(node) = doc.get_node(self.node_id) else {
            drop(doc);
            return self.node_not_exist_err();
        };
        let scroll_width = node.final_layout().scroll_width() as f64;
        let scroll_height = node.final_layout().scroll_height() as f64;
        Box::pin(async move { Ok(PixelsSize::new(scroll_width, scroll_height)) })
    }

    fn get_client_rect(&self) -> Pin<Box<dyn Future<Output = MountedResult<PixelsRect>>>> {
        let Some(bounding_rect) = self.doc_mut().get_client_bounding_rect(self.node_id) else {
            return self.node_not_exist_err();
        };
        let pixels_rect = PixelsRect::new(
            Point2D::new(bounding_rect.x, bounding_rect.y),
            Size2D::new(bounding_rect.width, bounding_rect.height),
        );
        Box::pin(async move { Ok(pixels_rect) })
    }

    /// Scroll the element into view (the backing for both `MountedData::scroll_to` and
    /// `MountedData::scroll_to_with_options`).
    fn scroll_to(
        &self,
        options: ScrollToOptions,
    ) -> Pin<Box<dyn Future<Output = MountedResult<()>>>> {
        let node_id = self.node_id;
        let mut doc = self.doc_mut();
        if doc.get_node(node_id).is_none() {
            drop(doc);
            return self.node_not_exist_err();
        }

        let behavior = match options.behavior {
            ScrollBehavior::Smooth => BlitzScrollBehavior::Smooth,
            ScrollBehavior::Instant => BlitzScrollBehavior::Instant,
        };
        let vertical = match options.vertical {
            ScrollLogicalPosition::Start => BlitzScrollLogicalPosition::Start,
            ScrollLogicalPosition::Center => BlitzScrollLogicalPosition::Center,
            ScrollLogicalPosition::End => BlitzScrollLogicalPosition::End,
            ScrollLogicalPosition::Nearest => BlitzScrollLogicalPosition::Nearest,
        };
        let horizontal = match options.horizontal {
            ScrollLogicalPosition::Start => BlitzScrollLogicalPosition::Start,
            ScrollLogicalPosition::Center => BlitzScrollLogicalPosition::Center,
            ScrollLogicalPosition::End => BlitzScrollLogicalPosition::End,
            ScrollLogicalPosition::Nearest => BlitzScrollLogicalPosition::Nearest,
        };
        doc.scroll_into_view(node_id, behavior, vertical, horizontal);

        Box::pin(async { Ok(()) })
    }

    /// Scroll the element's own content to the given offset (the backing for
    /// `MountedData::scroll`). The offset is clamped to the element's scrollable range.
    fn scroll(
        &self,
        coordinates: PixelsVector2D,
        behavior: ScrollBehavior,
    ) -> Pin<Box<dyn Future<Output = MountedResult<()>>>> {
        let node_id = self.node_id;
        let mut doc = self.doc_mut();
        if doc.get_node(node_id).is_none() {
            drop(doc);
            return self.node_not_exist_err();
        }

        doc.scroll_to(
            node_id,
            coordinates.x,
            coordinates.y,
            match behavior {
                ScrollBehavior::Smooth => BlitzScrollBehavior::Smooth,
                ScrollBehavior::Instant => BlitzScrollBehavior::Instant,
            },
        );

        Box::pin(async { Ok(()) })
    }

    fn set_focus(&self, focus: bool) -> Pin<Box<dyn Future<Output = MountedResult<()>>>> {
        let mut doc = self.doc_mut();
        if focus {
            // TODO: queue focus events somehow
            doc.set_focus_to(self.node_id);
        } else if doc.get_focussed_node_id() == Some(self.node_id) {
            // Q: Should this only clear focus if the node is focussed?
            // TODO: queue blur events somehow
            doc.clear_focus();
        }

        Box::pin(async { Ok(()) })
    }
}

#[derive(Clone, Debug)]
pub struct NativeFormData {
    pub value: String,
    pub values: Vec<(String, FormValue)>,
}

impl HasFormData for NativeFormData {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn value(&self) -> String {
        self.value.clone()
    }

    fn values(&self) -> Vec<(String, FormValue)> {
        self.values.clone()
    }
    fn valid(&self) -> bool {
        // todo: actually implement validation here.
        true
    }
}

impl HasFileData for NativeFormData {
    fn files(&self) -> Vec<aether_html::FileData> {
        vec![]
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BlitzKeyboardData(pub(crate) BlitzKeyEvent);

impl ModifiersInteraction for BlitzKeyboardData {
    fn modifiers(&self) -> Modifiers {
        self.0.modifiers
    }
}

impl HasKeyboardData for BlitzKeyboardData {
    fn key(&self) -> Key {
        self.0.key.clone()
    }

    fn code(&self) -> Code {
        self.0.code
    }

    fn location(&self) -> Location {
        self.0.location
    }

    fn is_auto_repeating(&self) -> bool {
        self.0.is_auto_repeating
    }

    fn is_composing(&self) -> bool {
        self.0.is_composing
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

#[derive(Clone)]
pub struct NativePointerData(pub(crate) BlitzPointerEvent);

impl InteractionLocation for NativePointerData {
    fn client_coordinates(&self) -> ClientPoint {
        ClientPoint::new(self.0.client_x() as f64, self.0.client_y() as f64)
    }

    fn screen_coordinates(&self) -> ScreenPoint {
        ScreenPoint::new(self.0.screen_x() as f64, self.0.screen_y() as f64)
    }

    fn page_coordinates(&self) -> PagePoint {
        PagePoint::new(self.0.page_x() as f64, self.0.page_y() as f64)
    }
}

impl InteractionElementOffset for NativePointerData {
    fn element_coordinates(&self) -> ElementPoint {
        ElementPoint::new(self.0.element_x() as f64, self.0.element_y() as f64)
    }
}

impl ModifiersInteraction for NativePointerData {
    fn modifiers(&self) -> Modifiers {
        self.0.mods
    }
}

impl PointerInteraction for NativePointerData {
    fn trigger_button(&self) -> Option<MouseButton> {
        Some(match self.0.button {
            MouseEventButton::Main => MouseButton::Primary,
            MouseEventButton::Auxiliary => MouseButton::Auxiliary,
            MouseEventButton::Secondary => MouseButton::Secondary,
            MouseEventButton::Fourth => MouseButton::Fourth,
            MouseEventButton::Fifth => MouseButton::Fifth,
        })
    }

    fn held_buttons(&self) -> MouseButtonSet {
        aether_html::input_data::decode_mouse_button_set(self.0.buttons.bits() as u16)
    }
}
impl HasMouseData for NativePointerData {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

impl HasPointerData for NativePointerData {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn is_primary(&self) -> bool {
        self.0.is_primary
    }

    fn pointer_id(&self) -> i32 {
        match self.0.id {
            BlitzPointerId::Mouse => 0,
            BlitzPointerId::Pen => 0,
            BlitzPointerId::Finger(id) => id as i32,
        }
    }

    fn pointer_type(&self) -> String {
        match self.0.id {
            BlitzPointerId::Mouse => String::from("mouse"),
            BlitzPointerId::Pen => String::from("pen"),
            BlitzPointerId::Finger(_) => String::from("touch"),
        }
    }

    fn pressure(&self) -> f32 {
        self.0.details.pressure as f32
    }
    fn tangential_pressure(&self) -> f32 {
        self.0.details.tangential_pressure
    }
    fn tilt_x(&self) -> i32 {
        self.0.details.tilt_x as i32
    }
    fn tilt_y(&self) -> i32 {
        self.0.details.tilt_y as i32
    }
    fn twist(&self) -> i32 {
        self.0.details.twist as i32
    }

    // TODO: implement these fields with real values
    fn width(&self) -> f64 {
        1.0
    }
    fn height(&self) -> f64 {
        1.0
    }
}

/// Touch event data exposed to Aether Native application code.
///
/// Blitz tracks input via pointer events, so a touch event is generated from the
/// pointer event of the finger that triggered it (`touches_changed`). The full
/// list of concurrent touches is carried on the triggering event's
/// [`BlitzPointerEvent::active_pointers`] list and reported via `touches`.
#[derive(Clone)]
pub struct NativeTouchData(pub(crate) BlitzPointerEvent);

impl ModifiersInteraction for NativeTouchData {
    fn modifiers(&self) -> Modifiers {
        self.0.mods
    }
}

impl HasTouchData for NativeTouchData {
    fn touches(&self) -> Vec<TouchPoint> {
        // All pointers currently active on the surface (multi-touch).
        self.0
            .active_pointers
            .borrow()
            .iter()
            .map(|event| TouchPoint::new(NativeTouchPointData(event.clone())))
            .collect()
    }

    fn touches_changed(&self) -> Vec<TouchPoint> {
        // Just the touch that triggered this event.
        vec![TouchPoint::new(NativeTouchPointData(self.0.clone()))]
    }

    fn target_touches(&self) -> Vec<TouchPoint> {
        // We don't track a per-touch target, so approximate `targetTouches`
        // (touches that started on the event target) with all active touches.
        self.touches()
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

#[derive(Clone)]
pub struct NativeTouchPointData(BlitzPointerEvent);

impl InteractionLocation for NativeTouchPointData {
    fn client_coordinates(&self) -> ClientPoint {
        ClientPoint::new(self.0.client_x() as f64, self.0.client_y() as f64)
    }

    fn screen_coordinates(&self) -> ScreenPoint {
        ScreenPoint::new(self.0.screen_x() as f64, self.0.screen_y() as f64)
    }

    fn page_coordinates(&self) -> PagePoint {
        PagePoint::new(self.0.page_x() as f64, self.0.page_y() as f64)
    }
}

impl HasTouchPointData for NativeTouchPointData {
    fn identifier(&self) -> i32 {
        match self.0.id {
            BlitzPointerId::Finger(id) => id as i32,
            BlitzPointerId::Mouse | BlitzPointerId::Pen => 0,
        }
    }

    fn force(&self) -> f64 {
        self.0.details.pressure
    }

    fn radius(&self) -> ScreenPoint {
        // TODO: expose real touch radius once blitz tracks it
        ScreenPoint::new(1.0, 1.0)
    }

    fn rotation(&self) -> f64 {
        // TODO: expose real touch rotation once blitz tracks it
        0.0
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

#[derive(Clone)]
pub struct NativeFocusData;
impl HasFocusData for NativeFocusData {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

#[derive(Clone)]
pub struct NativeScrollData(pub(crate) BlitzScrollEvent);
impl HasScrollData for NativeScrollData {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn scroll_top(&self) -> f64 {
        self.0.scroll_top
    }

    fn scroll_left(&self) -> f64 {
        self.0.scroll_left
    }

    fn scroll_width(&self) -> i32 {
        self.0.scroll_width
    }

    fn scroll_height(&self) -> i32 {
        self.0.scroll_height
    }

    fn client_width(&self) -> i32 {
        self.0.client_width
    }

    fn client_height(&self) -> i32 {
        self.0.client_height
    }
}

#[derive(Clone)]
pub struct NativeWheelData(pub(crate) BlitzWheelEvent);
impl HasWheelData for NativeWheelData {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn delta(&self) -> WheelDelta {
        match self.0.delta {
            BlitzWheelDelta::Lines(x, y) => {
                aether_html::geometry::WheelDelta::Lines(Vector3D::new(x, y, 0.0))
            }
            BlitzWheelDelta::Pixels(x, y) => {
                aether_html::geometry::WheelDelta::Pixels(Vector3D::new(x, y, 0.0))
            }
        }
    }
}

impl HasMouseData for NativeWheelData {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

impl PointerInteraction for NativeWheelData {
    fn trigger_button(&self) -> Option<MouseButton> {
        None
    }

    fn held_buttons(&self) -> MouseButtonSet {
        aether_html::input_data::decode_mouse_button_set(self.0.buttons.bits() as u16)
    }
}

impl ModifiersInteraction for NativeWheelData {
    fn modifiers(&self) -> Modifiers {
        self.0.mods
    }
}

impl InteractionElementOffset for NativeWheelData {
    fn element_coordinates(&self) -> ElementPoint {
        ElementPoint::new(self.0.element_x() as f64, self.0.element_y() as f64)
    }
}

impl InteractionLocation for NativeWheelData {
    fn client_coordinates(&self) -> ClientPoint {
        ClientPoint::new(self.0.client_x() as f64, self.0.client_y() as f64)
    }

    fn screen_coordinates(&self) -> ScreenPoint {
        ScreenPoint::new(self.0.screen_x() as f64, self.0.screen_y() as f64)
    }

    fn page_coordinates(&self) -> PagePoint {
        PagePoint::new(self.0.page_x() as f64, self.0.page_y() as f64)
    }
}

pub fn synthetic_click_event(node: &Node, modifiers: Modifiers) -> Box<dyn Any> {
    Box::new(NativePointerData(
        node.synthetic_click_event_data(modifiers),
    ))
}

pub fn synthetic_form_event(
    value: impl Into<String>,
    values: impl Into<Vec<(String, FormValue)>>,
) -> Box<dyn Any> {
    Box::new(NativeFormData {
        value: value.into(),
        values: values.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use blitz_traits::events::{
        BlitzPointerId, MouseEventButton, MouseEventButtons, Point, PointerCoords, PointerDetails,
    };

    // State-only test identifiers are never used for DOM lookup.
    fn target_id(raw: u64) -> NodeId {
        NodeId::from_u64(raw)
    }

    fn finger_event(id: u64, x: f32, y: f32) -> BlitzPointerEvent {
        BlitzPointerEvent {
            id: BlitzPointerId::Finger(id),
            is_primary: id == 0,
            coords: PointerCoords {
                page_x: x,
                page_y: y,
                screen_x: x,
                screen_y: y,
                client_x: x,
                client_y: y,
            },
            button: MouseEventButton::Main,
            buttons: MouseEventButtons::from(MouseEventButton::Main),
            mods: Default::default(),
            details: PointerDetails::default(),
            element: Point::default(),
            active_pointers: Default::default(),
        }
    }

    fn phases(events: Vec<NativeCompositionEvent>) -> Vec<(&'static str, String)> {
        events
            .into_iter()
            .map(|event| (event.name, event.data.0))
            .collect()
    }

    #[test]
    fn ime_preedit_clear_and_commit_preserve_text_and_phase_order() {
        let mut state = NativeCompositionState::default();
        assert!(
            state
                .handle(target_id(1), &BlitzImeEvent::Enabled)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            phases(
                state
                    .handle(
                        target_id(1),
                        &BlitzImeEvent::Preedit("\u{306b}".into(), Some((3, 3)))
                    )
                    .unwrap()
            ),
            [
                ("compositionstart", String::new()),
                ("compositionupdate", "\u{306b}".into())
            ]
        );
        assert_eq!(
            phases(
                state
                    .handle(
                        target_id(1),
                        &BlitzImeEvent::Preedit("\u{65e5}\u{672c}".into(), None)
                    )
                    .unwrap()
            ),
            [("compositionupdate", "\u{65e5}\u{672c}".into())]
        );
        // Winit clears preedit immediately before commit. This is an update, not
        // an end followed by a second, invented composition session.
        assert_eq!(
            phases(
                state
                    .handle(target_id(1), &BlitzImeEvent::Preedit(String::new(), None))
                    .unwrap()
            ),
            [("compositionupdate", String::new())]
        );
        assert_eq!(
            phases(
                state
                    .handle(
                        target_id(1),
                        &BlitzImeEvent::Commit("\u{65e5}\u{672c}".into())
                    )
                    .unwrap()
            ),
            [("compositionend", "\u{65e5}\u{672c}".into())]
        );
        assert!(
            state
                .handle(target_id(1), &BlitzImeEvent::Disabled)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn ime_availability_empty_preedit_and_direct_commit_do_not_invent_composition() {
        let mut state = NativeCompositionState::default();
        for event in [
            BlitzImeEvent::Enabled,
            BlitzImeEvent::Preedit(String::new(), None),
            BlitzImeEvent::Commit("\u{e9}".into()),
            BlitzImeEvent::Disabled,
            BlitzImeEvent::Disabled,
        ] {
            assert!(state.handle(target_id(1), &event).unwrap().is_empty());
        }
    }

    #[test]
    fn ime_disable_and_focus_loss_cancel_once_without_leaking_between_targets() {
        let mut state = NativeCompositionState::default();
        state
            .handle(target_id(1), &BlitzImeEvent::Preedit("draft".into(), None))
            .unwrap();
        assert_eq!(
            phases(
                state
                    .handle(target_id(1), &BlitzImeEvent::Disabled)
                    .unwrap()
            ),
            [("compositionend", String::new())]
        );
        assert!(
            state
                .handle(target_id(1), &BlitzImeEvent::Disabled)
                .unwrap()
                .is_empty()
        );
        state
            .handle(target_id(1), &BlitzImeEvent::Preedit("draft".into(), None))
            .unwrap();
        assert!(state.cancel(target_id(2)).is_none());
        assert_eq!(state.cancel(target_id(1)).unwrap().name, "compositionend");
        assert!(state.cancel(target_id(1)).is_none());
        assert!(
            state
                .handle(target_id(2), &BlitzImeEvent::Commit("other".into()))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn ime_target_switch_ends_previous_target_and_teardown_clears_stale_state() {
        let mut state = NativeCompositionState::default();
        state
            .handle(target_id(1), &BlitzImeEvent::Preedit("old".into(), None))
            .unwrap();
        let switched = state
            .handle(target_id(2), &BlitzImeEvent::Preedit("new".into(), None))
            .unwrap();
        assert_eq!(
            switched
                .iter()
                .map(|event| event.target)
                .collect::<Vec<_>>(),
            [target_id(1), target_id(2), target_id(2)]
        );
        assert_eq!(
            phases(switched),
            [
                ("compositionend", String::new()),
                ("compositionstart", String::new()),
                ("compositionupdate", "new".into()),
            ]
        );
        state.retain_target(|_| false);
        assert!(
            state
                .handle(target_id(2), &BlitzImeEvent::Commit("new".into()))
                .unwrap()
                .is_empty()
        );
        let restarted = state
            .handle(target_id(2), &BlitzImeEvent::Preedit("reused".into(), None))
            .unwrap();
        assert_eq!(restarted[0].name, "compositionstart");
        let ended = state
            .handle(target_id(3), &BlitzImeEvent::Commit("direct".into()))
            .unwrap();
        assert_eq!(ended[0].target, target_id(2));
        assert_eq!(phases(ended), [("compositionend", String::new())]);
    }

    #[test]
    fn unsupported_native_events_report_capability_errors_without_fake_data() {
        for name in [
            "beforeinput",
            "copy",
            "paste",
            "drag",
            "animationstart",
            "resize",
            "visible",
            "selectionchange",
            "toggle",
            "composition",
            "not-an-event",
        ] {
            assert_eq!(native_event_support(name).unwrap_err().event_name(), name);
        }
        for name in [
            "oninput",
            "pointerdown",
            "touchcancel",
            "compositionstart",
            "oncompositionupdate",
            "compositionend",
            "mounted",
        ] {
            assert!(native_event_support(name).is_ok());
        }
        assert!(
            NativeConverter::convert_dom_event(&DomEventData::Ime(BlitzImeEvent::Enabled)).is_err()
        );
    }

    #[test]
    fn surrounding_delete_does_not_end_or_duplicate_composition() {
        let mut state = NativeCompositionState::default();
        state
            .handle(target_id(1), &BlitzImeEvent::Preedit("仮".into(), None))
            .unwrap();
        assert!(
            state
                .handle(
                    target_id(1),
                    &BlitzImeEvent::DeleteSurrounding {
                        before_bytes: 3,
                        after_bytes: 0,
                    }
                )
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            phases(
                state
                    .handle(target_id(1), &BlitzImeEvent::Commit("確".into()))
                    .unwrap()
            ),
            [("compositionend", "確".into())]
        );
    }

    #[test]
    fn typed_composition_and_pointer_converters_preserve_real_payloads() {
        let converter = NativeConverter {};
        let event = PlatformEventData::new(Box::new(NativeCompositionData(
            "\u{65e5}\u{672c}\u{8a9e}".into(),
        )));
        assert_eq!(
            converter.convert_composition_data(&event).data(),
            "\u{65e5}\u{672c}\u{8a9e}"
        );
        let raw = DomEventData::PointerDown(finger_event(42, 10.0, 20.0));
        let event = NativeConverter::convert_dom_event(&raw).unwrap();
        let pointer = converter.convert_pointer_data(&event);
        assert_eq!(pointer.pointer_id(), 42);
        assert_eq!(pointer.pointer_type(), "touch");
        assert_eq!(pointer.client_coordinates(), ClientPoint::new(10.0, 20.0));
        assert_eq!(
            converter.convert_mouse_data(&event).client_coordinates(),
            pointer.client_coordinates()
        );
    }

    #[test]
    fn touches_reports_all_active_pointers() {
        let f0 = finger_event(0, 10.0, 20.0);
        let f1 = finger_event(1, 30.0, 40.0);

        // The triggering event (second finger down) carries the list of all
        // currently-active pointers.
        let trigger = f1.clone();
        {
            let mut list = trigger.active_pointers.borrow_mut();
            list.push(f0.clone());
            list.push(f1.clone());
        }

        let data = NativeTouchData(trigger);

        // `touches` reports every active pointer ...
        let touches = data.touches();
        assert_eq!(touches.len(), 2);
        let coords: Vec<(f64, f64)> = touches
            .iter()
            .map(|t| {
                let c = t.client_coordinates();
                (c.x, c.y)
            })
            .collect();
        assert!(coords.contains(&(10.0, 20.0)));
        assert!(coords.contains(&(30.0, 40.0)));

        // ... while `changed_touches` reports only the triggering touch.
        assert_eq!(data.touches_changed().len(), 1);
        let changed = data.touches_changed();
        let changed_coords = changed[0].client_coordinates();
        assert_eq!((changed_coords.x, changed_coords.y), (30.0, 40.0));
    }

    #[test]
    fn touches_is_empty_when_no_pointers_are_active() {
        // e.g. a `touchend` for the last finger: it has been removed from the
        // active list before dispatch, so `touches` is empty but
        // `changed_touches` still reports the finger that ended.
        let data = NativeTouchData(finger_event(0, 1.0, 2.0));
        assert!(data.touches().is_empty());
        assert_eq!(data.touches_changed().len(), 1);
    }
}
