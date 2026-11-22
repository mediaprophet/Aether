//! Types to represent UI and DOM events

use std::str::FromStr;
use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use bitflags::bitflags;
use keyboard_types::{Code, Key, Location, Modifiers};
use smol_str::SmolStr;

use crate::NodeId;

#[derive(Default)]
pub struct EventState {
    cancelled: bool,
    propagation_stopped: bool,
    redraw_requested: bool,
}
impl EventState {
    #[inline(always)]
    pub fn prevent_default(&mut self) {
        self.cancelled = true;
    }

    #[inline(always)]
    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    #[inline(always)]
    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    #[inline(always)]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    #[inline(always)]
    pub fn propagation_is_stopped(&self) -> bool {
        self.propagation_stopped
    }

    #[inline(always)]
    pub fn redraw_is_requested(&self) -> bool {
        self.redraw_requested
    }

    #[inline]
    pub fn merge(&self, other: &EventState) -> EventState {
        EventState {
            cancelled: self.cancelled | other.cancelled,
            propagation_stopped: self.propagation_stopped | other.propagation_stopped,
            redraw_requested: self.redraw_requested | other.redraw_requested,
        }
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum UiEvent {
    PointerMove(BlitzPointerEvent),
    PointerUp(BlitzPointerEvent),
    PointerDown(BlitzPointerEvent),
    PointerCancel(BlitzPointerEvent),
    /// The pointer has left the document (e.g. it moved out of the window)
    PointerLeave(BlitzPointerEvent),
    Wheel(BlitzWheelEvent),
    KeyUp(BlitzKeyEvent),
    KeyDown(BlitzKeyEvent),
    Ime(BlitzImeEvent),
    AppleStandardKeybinding(SmolStr),
}
impl UiEvent {
    pub fn discriminant(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        // See: https://doc.rust-lang.org/stable/std/mem/fn.discriminant.html#accessing-the-numeric-value-of-the-discriminant
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

#[derive(Debug, Clone)]
pub struct DomEvent {
    pub target: NodeId,
    /// Which is true if the event bubbles up through the DOM tree.
    pub bubbles: bool,
    /// which is true if the event can be canceled.
    pub cancelable: bool,

    /// The initiating control, when an action targets its owning form.
    pub related_target: Option<NodeId>,
    pub data: DomEventData,
    pub request_redraw: bool,
}

impl DomEvent {
    pub fn new(target: NodeId, data: DomEventData) -> Self {
        Self {
            target,
            bubbles: data.bubbles(),
            cancelable: data.cancelable(),
            related_target: None,
            data,
            request_redraw: false,
        }
    }

    /// Returns the name of the event ("click", "mouseover", "keypress", etc)
    pub fn name(&self) -> &'static str {
        self.data.name()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum DomEventKind {
    PointerMove,
    PointerDown,
    PointerUp,
    PointerCancel,
    PointerEnter,
    PointerLeave,
    PointerOver,
    PointerOut,

    MouseMove,
    MouseDown,
    MouseUp,
    MouseEnter,
    MouseLeave,
    MouseOver,
    MouseOut,

    TouchStart,
    TouchMove,
    TouchEnd,
    TouchCancel,

    Scroll,
    Wheel,

    Click,
    ContextMenu,
    DoubleClick,

    KeyPress,
    KeyDown,
    KeyUp,
    Input,
    Ime,

    Focus,
    Blur,
    FocusIn,
    FocusOut,

    AppleStandardKeybinding,

    Copy,
    Cut,
    Paste,
    BeforeInput,
    Select,
    SelectionChange,
    BeforeToggle,
    Toggle,
    Resize,
    Load,
    Error,
    Change,
    Submit,
    Reset,
    Cancel,
}
impl DomEventKind {
    pub const COUNT: usize = Self::Cancel as usize + 1;

    pub fn discriminant(self) -> u8 {
        self as u8
    }
}
impl FromStr for DomEventKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s.strip_prefix("on").unwrap_or(s) {
            "pointermove" => Ok(Self::PointerMove),
            "pointerdown" => Ok(Self::PointerDown),
            "pointerup" => Ok(Self::PointerUp),
            "pointercancel" => Ok(Self::PointerCancel),
            "pointerenter" => Ok(Self::PointerEnter),
            "pointerleave" => Ok(Self::PointerLeave),
            "pointerover" => Ok(Self::PointerOver),
            "pointerout" => Ok(Self::PointerOut),

            "mousemove" => Ok(Self::MouseMove),
            "mousedown" => Ok(Self::MouseDown),
            "mouseup" => Ok(Self::MouseUp),
            "mouseenter" => Ok(Self::MouseEnter),
            "mouseleave" => Ok(Self::MouseLeave),
            "mouseover" => Ok(Self::MouseOver),
            "mouseout" => Ok(Self::MouseOut),

            "touchstart" => Ok(Self::TouchStart),
            "touchmove" => Ok(Self::TouchMove),
            "touchend" => Ok(Self::TouchEnd),
            "touchcancel" => Ok(Self::TouchCancel),

            "scroll" => Ok(Self::Scroll),
            "wheel" => Ok(Self::Wheel),

            "click" => Ok(Self::Click),
            "contextmenu" => Ok(Self::ContextMenu),
            "dblclick" => Ok(Self::DoubleClick),

            "keypress" => Ok(Self::KeyPress),
            "keydown" => Ok(Self::KeyDown),
            "keyup" => Ok(Self::KeyUp),
            "input" => Ok(Self::Input),
            "composition" => Ok(Self::Ime),

            "focus" => Ok(Self::Focus),
            "blur" => Ok(Self::Blur),
            "focusin" => Ok(Self::FocusIn),
            "focusout" => Ok(Self::FocusOut),
            "applekeybinding" => Ok(Self::AppleStandardKeybinding),
            "copy" => Ok(Self::Copy),
            "cut" => Ok(Self::Cut),
            "paste" => Ok(Self::Paste),
            "beforeinput" => Ok(Self::BeforeInput),
            "select" => Ok(Self::Select),
            "selectionchange" => Ok(Self::SelectionChange),
            "beforetoggle" => Ok(Self::BeforeToggle),
            "toggle" => Ok(Self::Toggle),
            "resize" => Ok(Self::Resize),
            "load" => Ok(Self::Load),
            "error" => Ok(Self::Error),
            "change" => Ok(Self::Change),
            "submit" => Ok(Self::Submit),
            "reset" => Ok(Self::Reset),
            "cancel" => Ok(Self::Cancel),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum DomEventData {
    PointerMove(BlitzPointerEvent),
    PointerDown(BlitzPointerEvent),
    PointerUp(BlitzPointerEvent),
    PointerCancel(BlitzPointerEvent),
    PointerEnter(BlitzPointerEvent),
    PointerLeave(BlitzPointerEvent),
    PointerOver(BlitzPointerEvent),
    PointerOut(BlitzPointerEvent),

    MouseMove(BlitzPointerEvent),
    MouseDown(BlitzPointerEvent),
    MouseUp(BlitzPointerEvent),
    MouseEnter(BlitzPointerEvent),
    MouseLeave(BlitzPointerEvent),
    MouseOver(BlitzPointerEvent),
    MouseOut(BlitzPointerEvent),

    TouchStart(BlitzPointerEvent),
    TouchMove(BlitzPointerEvent),
    TouchEnd(BlitzPointerEvent),
    TouchCancel(BlitzPointerEvent),

    Scroll(BlitzScrollEvent),
    Wheel(BlitzWheelEvent),

    Click(BlitzPointerEvent),
    ContextMenu(BlitzPointerEvent),
    DoubleClick(BlitzPointerEvent),

    KeyPress(BlitzKeyEvent),
    KeyDown(BlitzKeyEvent),
    KeyUp(BlitzKeyEvent),
    Input(BlitzInputEvent),
    Ime(BlitzImeEvent),

    Focus(BlitzFocusEvent),
    Blur(BlitzFocusEvent),
    FocusIn(BlitzFocusEvent),
    FocusOut(BlitzFocusEvent),

    AppleStandardKeybinding(SmolStr),

    Copy(BlitzClipboardEvent),
    Cut(BlitzClipboardEvent),
    Paste(BlitzClipboardEvent),
    BeforeInput(BlitzBeforeInputEvent),
    Select(BlitzSelectionEvent),
    SelectionChange(BlitzSelectionEvent),
    BeforeToggle(BlitzToggleEvent),
    Toggle(BlitzToggleEvent),
    Resize(BlitzResizeEvent),
    Load(BlitzImageEvent),
    Error(BlitzImageEvent),
    Change(BlitzFormActionEvent),
    Submit(BlitzFormActionEvent),
    Reset(BlitzFormActionEvent),
    Cancel(BlitzCancelEvent),
}
impl DomEventData {
    pub fn discriminant(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        // See: https://doc.rust-lang.org/stable/std/mem/fn.discriminant.html#accessing-the-numeric-value-of-the-discriminant
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

impl DomEventData {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PointerMove { .. } => "pointermove",
            Self::PointerDown { .. } => "pointerdown",
            Self::PointerUp { .. } => "pointerup",
            Self::PointerCancel { .. } => "pointercancel",
            Self::PointerEnter { .. } => "pointerenter",
            Self::PointerLeave { .. } => "pointerleave",
            Self::PointerOver { .. } => "pointerover",
            Self::PointerOut { .. } => "pointerout",

            Self::MouseMove { .. } => "mousemove",
            Self::MouseDown { .. } => "mousedown",
            Self::MouseUp { .. } => "mouseup",
            Self::MouseEnter { .. } => "mouseenter",
            Self::MouseLeave { .. } => "mouseleave",
            Self::MouseOver { .. } => "mouseover",
            Self::MouseOut { .. } => "mouseout",

            Self::TouchStart { .. } => "touchstart",
            Self::TouchMove { .. } => "touchmove",
            Self::TouchEnd { .. } => "touchend",
            Self::TouchCancel { .. } => "touchcancel",

            Self::Scroll { .. } => "scroll",
            Self::Wheel { .. } => "wheel",

            Self::Click { .. } => "click",
            Self::ContextMenu { .. } => "contextmenu",
            Self::DoubleClick { .. } => "dblclick",

            Self::KeyPress { .. } => "keypress",
            Self::KeyDown { .. } => "keydown",
            Self::KeyUp { .. } => "keyup",
            Self::Input { .. } => "input",
            Self::Ime { .. } => "composition",

            Self::Focus { .. } => "focus",
            Self::Blur { .. } => "blur",
            Self::FocusIn { .. } => "focusin",
            Self::FocusOut { .. } => "focusout",

            Self::AppleStandardKeybinding { .. } => "applekeybinding",
            Self::Copy(..) => "copy",
            Self::Cut(..) => "cut",
            Self::Paste(..) => "paste",
            Self::BeforeInput(..) => "beforeinput",
            Self::Select(..) => "select",
            Self::SelectionChange(..) => "selectionchange",
            Self::BeforeToggle(..) => "beforetoggle",
            Self::Toggle(..) => "toggle",
            Self::Resize(..) => "resize",
            Self::Load(..) => "load",
            Self::Error(..) => "error",
            Self::Change(..) => "change",
            Self::Submit(..) => "submit",
            Self::Reset(..) => "reset",
            Self::Cancel(..) => "cancel",
        }
    }

    pub fn kind(&self) -> DomEventKind {
        match self {
            Self::PointerMove { .. } => DomEventKind::PointerMove,
            Self::PointerDown { .. } => DomEventKind::PointerDown,
            Self::PointerUp { .. } => DomEventKind::PointerUp,
            Self::PointerCancel { .. } => DomEventKind::PointerCancel,
            Self::PointerEnter { .. } => DomEventKind::PointerEnter,
            Self::PointerLeave { .. } => DomEventKind::PointerLeave,
            Self::PointerOver { .. } => DomEventKind::PointerOver,
            Self::PointerOut { .. } => DomEventKind::PointerOut,

            Self::MouseMove { .. } => DomEventKind::MouseMove,
            Self::MouseDown { .. } => DomEventKind::MouseDown,
            Self::MouseUp { .. } => DomEventKind::MouseUp,
            Self::MouseEnter { .. } => DomEventKind::MouseEnter,
            Self::MouseLeave { .. } => DomEventKind::MouseLeave,
            Self::MouseOver { .. } => DomEventKind::MouseOver,
            Self::MouseOut { .. } => DomEventKind::MouseOut,

            Self::TouchStart { .. } => DomEventKind::TouchStart,
            Self::TouchMove { .. } => DomEventKind::TouchMove,
            Self::TouchEnd { .. } => DomEventKind::TouchEnd,
            Self::TouchCancel { .. } => DomEventKind::TouchCancel,

            Self::Scroll { .. } => DomEventKind::Scroll,
            Self::Wheel { .. } => DomEventKind::Wheel,

            Self::Click { .. } => DomEventKind::Click,
            Self::ContextMenu { .. } => DomEventKind::ContextMenu,
            Self::DoubleClick { .. } => DomEventKind::DoubleClick,

            Self::KeyPress { .. } => DomEventKind::KeyPress,
            Self::KeyDown { .. } => DomEventKind::KeyDown,
            Self::KeyUp { .. } => DomEventKind::KeyUp,
            Self::Input { .. } => DomEventKind::Input,
            Self::Ime { .. } => DomEventKind::Ime,

            Self::Focus { .. } => DomEventKind::Focus,
            Self::Blur { .. } => DomEventKind::Blur,
            Self::FocusIn { .. } => DomEventKind::FocusIn,
            Self::FocusOut { .. } => DomEventKind::FocusOut,

            Self::AppleStandardKeybinding { .. } => DomEventKind::AppleStandardKeybinding,
            Self::Copy(..) => DomEventKind::Copy,
            Self::Cut(..) => DomEventKind::Cut,
            Self::Paste(..) => DomEventKind::Paste,
            Self::BeforeInput(..) => DomEventKind::BeforeInput,
            Self::Select(..) => DomEventKind::Select,
            Self::SelectionChange(..) => DomEventKind::SelectionChange,
            Self::BeforeToggle(..) => DomEventKind::BeforeToggle,
            Self::Toggle(..) => DomEventKind::Toggle,
            Self::Resize(..) => DomEventKind::Resize,
            Self::Load(..) => DomEventKind::Load,
            Self::Error(..) => DomEventKind::Error,
            Self::Change(..) => DomEventKind::Change,
            Self::Submit(..) => DomEventKind::Submit,
            Self::Reset(..) => DomEventKind::Reset,
            Self::Cancel(..) => DomEventKind::Cancel,
        }
    }

    pub fn cancelable(&self) -> bool {
        match self {
            Self::PointerMove { .. } => true,
            Self::PointerDown { .. } => true,
            Self::PointerUp { .. } => true,
            Self::PointerCancel { .. } => false,
            Self::PointerEnter { .. } => false,
            Self::PointerLeave { .. } => false,
            Self::PointerOver { .. } => true,
            Self::PointerOut { .. } => true,

            Self::MouseMove { .. } => true,
            Self::MouseDown { .. } => true,
            Self::MouseUp { .. } => true,
            Self::MouseEnter { .. } => false,
            Self::MouseLeave { .. } => false,
            Self::MouseOver { .. } => true,
            Self::MouseOut { .. } => true,

            Self::TouchStart { .. } => true,
            Self::TouchMove { .. } => true,
            Self::TouchEnd { .. } => true,
            Self::TouchCancel { .. } => false,

            Self::Scroll { .. } => false,
            Self::Wheel { .. } => true,

            Self::Click { .. } => true,
            Self::ContextMenu { .. } => true,
            Self::DoubleClick { .. } => true,

            Self::KeyDown { .. } => true,
            Self::KeyUp { .. } => true,
            Self::KeyPress { .. } => true,
            Self::Ime { .. } => true,
            Self::Input { .. } => false,

            Self::Focus { .. } => false,
            Self::Blur { .. } => false,
            Self::FocusIn { .. } => false,
            Self::FocusOut { .. } => false,

            Self::AppleStandardKeybinding { .. } => true,
            Self::Copy(..) => true,
            Self::Cut(..) => true,
            Self::Paste(..) => true,
            Self::BeforeInput(..) => true,
            Self::Select(..) => false,
            Self::SelectionChange(..) => false,
            Self::BeforeToggle(..) => true,
            Self::Toggle(..) => false,
            Self::Resize(..) => false,
            Self::Load(..) => false,
            Self::Error(..) => false,
            Self::Change(..) => false,
            Self::Submit(..) => true,
            Self::Reset(..) => true,
            Self::Cancel(..) => true,
        }
    }

    pub fn bubbles(&self) -> bool {
        match self {
            Self::PointerMove { .. } => true,
            Self::PointerDown { .. } => true,
            Self::PointerUp { .. } => true,
            Self::PointerCancel { .. } => true,
            Self::PointerEnter { .. } => false,
            Self::PointerLeave { .. } => false,
            Self::PointerOver { .. } => true,
            Self::PointerOut { .. } => true,

            Self::MouseMove { .. } => true,
            Self::MouseDown { .. } => true,
            Self::MouseUp { .. } => true,
            Self::MouseEnter { .. } => false,
            Self::MouseLeave { .. } => false,
            Self::MouseOver { .. } => true,
            Self::MouseOut { .. } => true,

            Self::TouchStart { .. } => true,
            Self::TouchMove { .. } => true,
            Self::TouchEnd { .. } => true,
            Self::TouchCancel { .. } => true,

            Self::Scroll { .. } => false,
            Self::Wheel { .. } => true,

            Self::Click { .. } => true,
            Self::ContextMenu { .. } => true,
            Self::DoubleClick { .. } => true,

            Self::KeyDown { .. } => true,
            Self::KeyUp { .. } => true,
            Self::KeyPress { .. } => true,
            Self::Ime { .. } => true,
            Self::Input { .. } => true,

            Self::Focus { .. } => false,
            Self::Blur { .. } => false,
            Self::FocusIn { .. } => true,
            Self::FocusOut { .. } => true,

            Self::AppleStandardKeybinding { .. } => false,
            Self::Copy(..) => true,
            Self::Cut(..) => true,
            Self::Paste(..) => true,
            Self::BeforeInput(..) => true,
            Self::Select(..) => true,
            Self::SelectionChange(..) => true,
            Self::BeforeToggle(..) => false,
            Self::Toggle(..) => false,
            Self::Resize(..) => false,
            Self::Load(..) => false,
            Self::Error(..) => false,
            Self::Change(..) => true,
            Self::Submit(..) => true,
            Self::Reset(..) => true,
            Self::Cancel(..) => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HitResult {
    /// The node_id of the node identified as the hit target
    pub node_id: NodeId,
    /// Whether the hit content is text
    pub is_text: bool,
    /// The x coordinate of the hit within the hit target's border-box
    pub x: f32,
    /// The y coordinate of the hit within the hit target's border-box
    pub y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlitzPointerId {
    Mouse,
    Pen,
    Finger(u64),
}

#[derive(Copy, Clone, Debug)]
pub struct PointerCoords {
    pub page_x: f32,
    pub page_y: f32,
    pub screen_x: f32,
    pub screen_y: f32,
    pub client_x: f32,
    pub client_y: f32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PointerDetails {
    pub pressure: f64, // default 0.5 if buttons pressed else 0.0
    pub tangential_pressure: f32,
    pub tilt_x: i8,
    pub tilt_y: i8,
    pub twist: u16,
    pub altitude: f64,
    pub azimuth: f64,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

#[derive(Clone, Debug)]
pub struct BlitzPointerEvent {
    pub id: BlitzPointerId,
    pub is_primary: bool,
    pub coords: PointerCoords,
    pub button: MouseEventButton,
    pub buttons: MouseEventButtons,
    pub mods: Modifiers,
    pub details: PointerDetails,
    pub element: Point<f32>,
    /// All pointers that are currently active (pressed) on the surface, including
    /// the one that triggered this event.
    ///
    /// This is shared via [`Arc`] so that it can be cheaply cloned into each
    /// dispatched event and cheaply mutated as pointers are pressed, moved and
    /// released. It is primarily used to populate the `touches` list of touch
    /// events (multi-touch support).
    ///
    /// The events stored in this list always have an empty `active_pointers`
    /// list themselves, to avoid a reference cycle.
    pub active_pointers: Arc<AtomicRefCell<Vec<BlitzPointerEvent>>>,
}

impl BlitzPointerEvent {
    #[inline(always)]
    pub fn is_mouse(&self) -> bool {
        matches!(self.id, BlitzPointerId::Mouse)
    }
    #[inline(always)]
    pub fn is_pen(&self) -> bool {
        matches!(self.id, BlitzPointerId::Pen)
    }
    #[inline(always)]
    pub fn is_finger(&self) -> bool {
        matches!(self.id, BlitzPointerId::Finger(_))
    }

    #[inline(always)]
    pub fn page_x(&self) -> f32 {
        self.coords.page_x
    }
    #[inline(always)]
    pub fn page_y(&self) -> f32 {
        self.coords.page_y
    }
    #[inline(always)]
    pub fn client_x(&self) -> f32 {
        self.coords.client_x
    }
    #[inline(always)]
    pub fn client_y(&self) -> f32 {
        self.coords.client_y
    }
    #[inline(always)]
    pub fn screen_x(&self) -> f32 {
        self.coords.screen_x
    }
    #[inline(always)]
    pub fn screen_y(&self) -> f32 {
        self.coords.screen_y
    }

    #[inline(always)]
    pub fn element_x(&self) -> f32 {
        self.element.x
    }
    #[inline(always)]
    pub fn element_y(&self) -> f32 {
        self.element.y
    }
}

#[derive(Clone, Debug)]
pub enum BlitzWheelDelta {
    Lines(f64, f64),
    Pixels(f64, f64),
}

#[derive(Clone, Debug)]
pub struct BlitzWheelEvent {
    pub delta: BlitzWheelDelta,
    pub coords: PointerCoords,
    pub buttons: MouseEventButtons,
    pub mods: Modifiers,
    pub element: Point<f32>,
}

impl BlitzWheelEvent {
    #[inline(always)]
    pub fn page_x(&self) -> f32 {
        self.coords.page_x
    }
    #[inline(always)]
    pub fn page_y(&self) -> f32 {
        self.coords.page_y
    }
    #[inline(always)]
    pub fn client_x(&self) -> f32 {
        self.coords.client_x
    }
    #[inline(always)]
    pub fn client_y(&self) -> f32 {
        self.coords.client_y
    }
    #[inline(always)]
    pub fn screen_x(&self) -> f32 {
        self.coords.screen_x
    }
    #[inline(always)]
    pub fn screen_y(&self) -> f32 {
        self.coords.screen_y
    }
    #[inline(always)]
    pub fn element_x(&self) -> f32 {
        self.element.x
    }
    #[inline(always)]
    pub fn element_y(&self) -> f32 {
        self.element.y
    }
}

#[derive(Clone, Debug)]
pub struct BlitzScrollEvent {
    pub scroll_top: f64,
    pub scroll_left: f64,
    pub scroll_width: i32,
    pub scroll_height: i32,
    pub client_width: i32,
    pub client_height: i32,
}

// struct PointerInputState {
//     id: BlitzPointerId,
//     pointer_down_x: f32,
//     pointer_down_y: f32,
//     pointer_down_time: Option<Instant>,
//     click_count: u16,
// }

// struct PointersInputState {
//     initial_finger_active: bool,
//     mouse: Option<PointerInputState>,
//     fingers: Vec<PointerInputState>,
// }

bitflags! {
    /// The buttons property indicates which buttons are pressed on the mouse
    /// (or other input device) when a mouse event is triggered.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/buttons)
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct MouseEventButtons: u8 {
        /// 0: No button or un-initialized
        const None = 0b0000_0000;
        /// 1: Primary button (usually the left button)
        const Primary = 0b0000_0001;
        /// 2: Secondary button (usually the right button)
        const Secondary = 0b0000_0010;
        /// 4: Auxiliary button (usually the mouse wheel button or middle button)
        const Auxiliary = 0b0000_0100;
        /// 8: 4th button (typically the "Browser Back" button)
        const Fourth = 0b0000_1000;
        /// 16: 5th button (typically the "Browser Forward" button)
        const Fifth = 0b0001_0000;
    }
}

impl Default for MouseEventButtons {
    fn default() -> Self {
        Self::None
    }
}

impl From<MouseEventButton> for MouseEventButtons {
    fn from(value: MouseEventButton) -> Self {
        match value {
            MouseEventButton::Main => Self::Primary,
            MouseEventButton::Auxiliary => Self::Auxiliary,
            MouseEventButton::Secondary => Self::Secondary,
            MouseEventButton::Fourth => Self::Fourth,
            MouseEventButton::Fifth => Self::Fifth,
        }
    }
}

/// The button property indicates which button was pressed
/// on the mouse to trigger the event.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button)
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum MouseEventButton {
    /// Main button pressed, usually the left button or the un-initialized state
    #[default]
    Main = 0,
    /// Auxiliary button pressed, usually the wheel button or the middle button (if present)
    Auxiliary = 1,
    /// Secondary button pressed, usually the right button
    Secondary = 2,
    /// Fourth button, typically the Browser Back button
    Fourth = 3,
    /// Fifth button, typically the Browser Forward button
    Fifth = 4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

impl KeyState {
    pub fn is_pressed(self) -> bool {
        matches!(self, Self::Pressed)
    }
}

#[derive(Clone, Debug)]
pub struct BlitzKeyEvent {
    pub key: Key,
    pub code: Code,
    pub modifiers: Modifiers,
    pub location: Location,
    pub is_auto_repeating: bool,
    pub is_composing: bool,
    pub state: KeyState,
    pub text: Option<SmolStr>,
}

#[derive(Clone, Debug)]
pub struct BlitzInputEvent {
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct BlitzFocusEvent;

/// Copy of Winit IME event to avoid lower-level Blitz crates depending on winit
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlitzImeEvent {
    /// Notifies when the IME was enabled.
    ///
    /// After getting this event you could receive [`Preedit`][Self::Preedit] and
    /// [`Commit`][Self::Commit] events. You should also start performing IME related requests
    /// like `Window::set_ime_cursor_area`.
    Enabled,

    /// Notifies when a new composing text should be set at the cursor position.
    ///
    /// The value represents a pair of the preedit string and the cursor begin position and end
    /// position. When it's `None`, the cursor should be hidden. When `String` is an empty string
    /// this indicates that preedit was cleared.
    ///
    /// The cursor position is byte-wise indexed, assuming UTF-8.
    Preedit(String, Option<(usize, usize)>),

    /// Notifies when text should be inserted into the editor widget.
    ///
    /// Right before this event winit will send empty [`Self::Preedit`] event.
    Commit(String),

    /// Delete text surrounding the cursor or selection.
    ///
    /// This event does not affect either the pre-edit string.
    /// This means that the application must first remove the pre-edit,
    /// then execute the deletion, then insert the removed text back.
    ///
    /// This event assumes text is stored in UTF-8.
    DeleteSurrounding {
        /// Bytes to remove before the selection
        before_bytes: usize,
        /// Bytes to remove after the selection
        after_bytes: usize,
    },

    /// Notifies when the IME was disabled.
    ///
    /// After receiving this event you won't get any more [`Preedit`][Self::Preedit] or
    /// [`Commit`][Self::Commit] events until the next [`Enabled`][Self::Enabled] event. You should
    /// also stop issuing IME related requests like `Window::set_ime_cursor_area` and clear
    /// pending preedit text.
    Disabled,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlitzClipboardEvent {
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlitzBeforeInputEvent {
    pub input_type: String,
    pub data: Option<String>,
    pub is_composing: bool,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlitzSelectionEvent {
    pub selected_text: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlitzToggleEvent {
    pub old_state: bool,
    pub new_state: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlitzResizeEvent {
    pub border_box_width: f64,
    pub border_box_height: f64,
    pub content_box_width: f64,
    pub content_box_height: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlitzImageEvent {
    pub load_error: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlitzCancelEvent;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlitzFormActionEvent {
    pub value: String,
}

#[cfg(test)]
mod platform_event_tests {
    use super::*;

    #[test]
    fn event_kinds_match_payload_tags_and_names() {
        let pointer = BlitzPointerEvent {
            id: BlitzPointerId::Mouse,
            is_primary: true,
            coords: PointerCoords {
                page_x: 0.0,
                page_y: 0.0,
                screen_x: 0.0,
                screen_y: 0.0,
                client_x: 0.0,
                client_y: 0.0,
            },
            button: MouseEventButton::Main,
            buttons: MouseEventButtons::empty(),
            mods: Modifiers::empty(),
            details: PointerDetails::default(),
            element: Point::default(),
            active_pointers: Default::default(),
        };
        let key = BlitzKeyEvent {
            key: Key::Character(String::new()),
            code: Code::Unidentified,
            modifiers: Modifiers::empty(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state: KeyState::Pressed,
            text: None,
        };
        let events = [
            DomEventData::PointerMove(pointer.clone()),
            DomEventData::PointerDown(pointer.clone()),
            DomEventData::PointerUp(pointer.clone()),
            DomEventData::PointerCancel(pointer.clone()),
            DomEventData::PointerEnter(pointer.clone()),
            DomEventData::PointerLeave(pointer.clone()),
            DomEventData::PointerOver(pointer.clone()),
            DomEventData::PointerOut(pointer.clone()),
            DomEventData::MouseMove(pointer.clone()),
            DomEventData::MouseDown(pointer.clone()),
            DomEventData::MouseUp(pointer.clone()),
            DomEventData::MouseEnter(pointer.clone()),
            DomEventData::MouseLeave(pointer.clone()),
            DomEventData::MouseOver(pointer.clone()),
            DomEventData::MouseOut(pointer.clone()),
            DomEventData::TouchStart(pointer.clone()),
            DomEventData::TouchMove(pointer.clone()),
            DomEventData::TouchEnd(pointer.clone()),
            DomEventData::TouchCancel(pointer.clone()),
            DomEventData::Scroll(BlitzScrollEvent {
                scroll_top: 0.0,
                scroll_left: 0.0,
                scroll_width: 0,
                scroll_height: 0,
                client_width: 0,
                client_height: 0,
            }),
            DomEventData::Wheel(BlitzWheelEvent {
                delta: BlitzWheelDelta::Pixels(0.0, 0.0),
                coords: pointer.coords,
                mods: pointer.mods,
                element: pointer.element,
                buttons: pointer.buttons,
            }),
            DomEventData::Click(pointer.clone()),
            DomEventData::ContextMenu(pointer.clone()),
            DomEventData::DoubleClick(pointer.clone()),
            DomEventData::KeyPress(key.clone()),
            DomEventData::KeyDown(key.clone()),
            DomEventData::KeyUp(key.clone()),
            DomEventData::Input(BlitzInputEvent {
                value: String::new(),
            }),
            DomEventData::Ime(BlitzImeEvent::Enabled),
            DomEventData::Focus(BlitzFocusEvent),
            DomEventData::Blur(BlitzFocusEvent),
            DomEventData::FocusIn(BlitzFocusEvent),
            DomEventData::FocusOut(BlitzFocusEvent),
            DomEventData::AppleStandardKeybinding(SmolStr::default()),
            DomEventData::Copy(BlitzClipboardEvent::default()),
            DomEventData::Cut(BlitzClipboardEvent::default()),
            DomEventData::Paste(BlitzClipboardEvent::default()),
            DomEventData::BeforeInput(BlitzBeforeInputEvent::default()),
            DomEventData::Select(BlitzSelectionEvent::default()),
            DomEventData::SelectionChange(BlitzSelectionEvent::default()),
            DomEventData::BeforeToggle(BlitzToggleEvent::default()),
            DomEventData::Toggle(BlitzToggleEvent::default()),
            DomEventData::Resize(BlitzResizeEvent::default()),
            DomEventData::Load(BlitzImageEvent::default()),
            DomEventData::Error(BlitzImageEvent::default()),
            DomEventData::Change(BlitzFormActionEvent::default()),
            DomEventData::Submit(BlitzFormActionEvent::default()),
            DomEventData::Reset(BlitzFormActionEvent::default()),
            DomEventData::Cancel(BlitzCancelEvent::default()),
        ];
        assert_eq!(events.len(), DomEventKind::COUNT);
        for (index, event) in events.into_iter().enumerate() {
            assert_eq!(event.discriminant() as usize, index);
            assert_eq!(event.kind().discriminant(), event.discriminant());
            assert_eq!(event.name().parse(), Ok(event.kind()));
            assert_eq!(format!("on{}", event.name()).parse(), Ok(event.kind()));
        }
        assert!("ononclick".parse::<DomEventKind>().is_err());
    }
}
