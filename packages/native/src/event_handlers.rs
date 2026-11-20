use slotmap::{DefaultKey, Key, KeyData, SlotMap};
use std::cell::RefCell;
use std::rc::Rc;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

/// The unique identifier of a window event handler. This can be used to later remove the handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WinitEventHandlerId(pub(crate) u64);

impl WinitEventHandlerId {
    /// Unregister this event handler from the window
    pub fn remove(&self) {
        let handlers: Rc<WindowEventHandlers> = aether_core::consume_context();
        handlers.remove(*self);
    }
}

struct WinitWindowEventHandlerInner {
    window_id: WindowId,

    #[allow(clippy::type_complexity)]
    handler: Box<dyn FnMut(&WindowEvent, &dyn ActiveEventLoop) + 'static>,
}

#[derive(Default)]
pub(crate) struct WindowEventHandlers {
    handlers: CallbackRegistry<WinitWindowEventHandlerInner>,
}

/// Snapshot registration order, but look up each callback immediately before calling it.
/// No registry borrow survives into user code. An active FnMut is skipped by nested dispatch.
struct CallbackRegistry<T> {
    handlers: RefCell<SlotMap<DefaultKey, Rc<RefCell<T>>>>,
    order: RefCell<Vec<DefaultKey>>,
}

impl<T> Default for CallbackRegistry<T> {
    fn default() -> Self {
        Self {
            handlers: RefCell::new(SlotMap::with_key()),
            order: RefCell::new(Vec::new()),
        }
    }
}

impl<T> CallbackRegistry<T> {
    fn add(&self, handler: T) -> DefaultKey {
        let key = self
            .handlers
            .borrow_mut()
            .insert(Rc::new(RefCell::new(handler)));
        self.order.borrow_mut().push(key);
        key
    }

    fn remove(&self, key: DefaultKey) {
        let removed = self.handlers.borrow_mut().remove(key);
        self.order.borrow_mut().retain(|entry| *entry != key);
        // Captures may themselves unregister callbacks when dropped.
        drop(removed);
    }

    fn dispatch(&self, mut call: impl FnMut(&mut T)) {
        let order = self.order.borrow().clone();
        for key in order {
            let handler = self.handlers.borrow().get(key).cloned();
            if let Some(handler) = handler {
                // Reentering the same FnMut would alias its mutable state. Other handlers
                // may still receive the nested event, in their registration order.
                if let Ok(mut handler) = handler.try_borrow_mut() {
                    call(&mut handler);
                }
            }
        }
    }
}

impl WindowEventHandlers {
    pub(crate) fn add(
        &self,
        window_id: WindowId,
        handler: impl FnMut(&WindowEvent, &dyn ActiveEventLoop) + 'static,
    ) -> WinitEventHandlerId {
        let key = self.handlers.add(WinitWindowEventHandlerInner {
            window_id,
            handler: Box::new(handler),
        });
        WinitEventHandlerId(key.data().as_ffi())
    }

    pub(crate) fn remove(&self, id: WinitEventHandlerId) {
        let key = DefaultKey::from(KeyData::from_ffi(id.0));
        self.handlers.remove(key);
    }

    pub fn apply_event(
        &self,
        window_id: WindowId,
        event: &WindowEvent,
        target: &dyn ActiveEventLoop,
    ) {
        self.handlers.dispatch(|handler| {
            if handler.window_id == window_id {
                (handler.handler)(event, target)
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    type Handler = Box<dyn FnMut()>;

    #[test]
    fn removal_and_registration_during_dispatch_take_effect_at_defined_times() {
        let handlers = Rc::new(CallbackRegistry::<Handler>::default());
        let calls = Rc::new(RefCell::new(Vec::new()));
        let removed = Rc::new(Cell::new(None));
        let own_id = Rc::new(Cell::new(None));
        let weak = Rc::downgrade(&handlers);
        handlers.add(Box::new({
            let calls = calls.clone();
            let removed = removed.clone();
            let own_id = own_id.clone();
            move || {
                calls.borrow_mut().push("first");
                let handlers = weak.upgrade().unwrap();
                handlers.remove(removed.get().unwrap());
                handlers.remove(own_id.get().unwrap());
                handlers.add(Box::new({
                    let calls = calls.clone();
                    move || calls.borrow_mut().push("added")
                }));
            }
        }));
        own_id.set(handlers.order.borrow().first().copied());
        removed.set(Some(handlers.add(Box::new({
            let calls = calls.clone();
            move || calls.borrow_mut().push("removed")
        }))));
        handlers.add(Box::new({
            let calls = calls.clone();
            move || calls.borrow_mut().push("last")
        }));

        handlers.dispatch(|handler| handler());
        assert_eq!(*calls.borrow(), ["first", "last"]);
        handlers.dispatch(|handler| handler());
        assert_eq!(*calls.borrow(), ["first", "last", "last", "added"]);
    }

    #[test]
    fn nested_dispatch_skips_active_handler_and_preserves_order() {
        let handlers = Rc::new(CallbackRegistry::<Handler>::default());
        let calls = Rc::new(RefCell::new(Vec::new()));
        let weak = Rc::downgrade(&handlers);
        handlers.add(Box::new({
            let calls = calls.clone();
            move || {
                calls.borrow_mut().push("outer");
                weak.upgrade().unwrap().dispatch(|handler| handler());
            }
        }));
        handlers.add(Box::new({
            let calls = calls.clone();
            move || calls.borrow_mut().push("second")
        }));
        handlers.dispatch(|handler| handler());
        assert_eq!(*calls.borrow(), ["outer", "second", "second"]);
    }

    #[test]
    fn stale_ids_cannot_remove_reused_slots() {
        let handlers = CallbackRegistry::<Handler>::default();
        let stale = handlers.add(Box::new(|| {}));
        handlers.remove(stale);
        let calls = Rc::new(Cell::new(0));
        handlers.add(Box::new({
            let calls = calls.clone();
            move || calls.set(calls.get() + 1)
        }));
        handlers.remove(stale);
        handlers.dispatch(|handler| handler());
        assert_eq!(calls.get(), 1);
    }
}
