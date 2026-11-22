use std::{cell::RefCell, rc::Rc, sync::atomic::AtomicUsize};

use aether_core::{AttributeValue, IntoAttributeValue};
use blitz_dom::{Document, Widget};

#[derive(Clone, PartialEq)]
pub struct SubDocumentAttr(WriteOnceAttr<Box<dyn Document>>);

impl SubDocumentAttr {
    /// Accepts any [`Document`] implementation, e.g. a plain [`BaseDocument`](blitz_dom::BaseDocument).
    pub fn new(doc: impl Document) -> Self {
        let id = doc.id();
        Self(WriteOnceAttr::new(id, Box::new(doc) as Box<dyn Document>))
    }
}

impl IntoAttributeValue for SubDocumentAttr {
    fn into_value(self) -> AttributeValue {
        AttributeValue::Any(Rc::new(self.0))
    }
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, PartialEq)]
pub struct CustomWidgetAttr(WriteOnceAttr<Box<dyn Widget>>);

impl CustomWidgetAttr {
    pub fn new<T: Widget + 'static>(widget: T) -> Self {
        let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(WriteOnceAttr::generated(id, Box::new(widget)))
    }

    /// Keep the mounted widget across RSX renders by supplying the same identity.
    /// Change the ID when the underlying widget should be replaced.
    pub fn with_stable_id<T: Widget + 'static>(id: usize, value: T) -> Self {
        Self(WriteOnceAttr::new(id, Box::new(value)))
    }
}

impl IntoAttributeValue for CustomWidgetAttr {
    fn into_value(self) -> AttributeValue {
        AttributeValue::Any(Rc::new(self.0))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WriteOnceId {
    Stable(usize),
    Generated(usize),
}

pub(crate) struct WriteOnceAttr<T> {
    id: WriteOnceId,
    value: Rc<RefCell<Option<T>>>,
}

impl<T> WriteOnceAttr<T> {
    pub(crate) fn new(id: usize, value: T) -> Self {
        Self::with_id(WriteOnceId::Stable(id), value)
    }
    fn generated(id: usize, value: T) -> Self {
        Self::with_id(WriteOnceId::Generated(id), value)
    }
    fn with_id(id: WriteOnceId, value: T) -> Self {
        let value = Rc::new(RefCell::new(Some(value)));
        Self { id, value }
    }
    pub(crate) fn take(&self) -> Option<T> {
        self.value.borrow_mut().take()
    }
}

impl<T> Clone for WriteOnceAttr<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Rc::clone(&self.value),
        }
    }
}

impl<T> PartialEq for WriteOnceAttr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
