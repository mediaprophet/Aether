//! Read and write guards, reactive handles (`Memo`, `Effect`, `ResourceHandle`),
//! and cell-based append-only read/write logging for [`crate::Ui`].

use std::any::Any;
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::Wake;

use crate::lens::Path;
use crate::ui::Ctx;

/// What a reactive observer read: a structural path into the root state, a
/// memo, or a resource.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Source {
    State(Path),
    Memo(usize),
    Resource(usize),
}

/// A dependency edge, remembering the version of the source at read time
/// (versions are only meaningful for memo/resource sources).
#[derive(Clone, Debug)]
pub(crate) struct Dep {
    pub(crate) source: Source,
    pub(crate) version: u64,
}

/// The currently-running reactive observer.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Observer {
    Effect(usize),
    Memo(usize),
    Resource(usize),
}

/// Cell-based append-only read/write log. Guards only need `&Log`, so they
/// can coexist with the `&T`/`&mut T` they carry. `Cell<Vec<_>>` take/push/
/// put-back can never fail, unlike `RefCell` borrows.
#[derive(Default)]
pub(crate) struct Log {
    pub(crate) reads: Cell<Vec<(Observer, Source)>>,
    pub(crate) writes: Cell<Vec<Path>>,
    pub(crate) observers: Cell<Vec<Observer>>,
}

impl Log {
    pub(crate) fn current_observer(&self) -> Option<Observer> {
        let stack = self.observers.take();
        let top = stack.last().copied();
        self.observers.set(stack);
        top
    }

    pub(crate) fn push_observer(&self, o: Observer) {
        let mut stack = self.observers.take();
        stack.push(o);
        self.observers.set(stack);
    }

    pub(crate) fn pop_observer(&self) {
        let mut stack = self.observers.take();
        stack.pop();
        self.observers.set(stack);
    }

    pub(crate) fn record_read(&self, source: Source) {
        if let Some(observer) = self.current_observer() {
            let mut reads = self.reads.take();
            reads.push((observer, source));
            self.reads.set(reads);
        }
    }

    pub(crate) fn record_write(&self, path: Path) {
        let mut writes = self.writes.take();
        writes.push(path);
        self.writes.set(writes);
    }

    /// Remove and return the reads recorded for `observer`, leaving reads
    /// belonging to other (outer) observers in place.
    pub(crate) fn drain_reads_for(&self, observer: Observer) -> Vec<Source> {
        let reads = self.reads.take();
        let (mine, rest): (Vec<_>, Vec<_>) = reads.into_iter().partition(|(o, _)| *o == observer);
        self.reads.set(rest);
        mine.into_iter().map(|(_, s)| s).collect()
    }
}

/// A zero-copy read guard: derefs to plain `&T`. The read is recorded (once)
/// on first `Deref`, so merely creating a guard subscribes to nothing.
pub struct ReadGuard<'a, T: ?Sized> {
    pub(crate) value: &'a T,
    pub(crate) log: &'a Log,
    pub(crate) source: Option<Source>,
    pub(crate) tracked: Cell<bool>,
}

impl<T: ?Sized> std::ops::Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        if !self.tracked.replace(true)
            && let Some(source) = &self.source
        {
            self.log.record_read(source.clone());
        }
        self.value
    }
}

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for ReadGuard<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + std::fmt::Display> std::fmt::Display for ReadGuard<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

/// A zero-copy write guard: derefs to plain `&mut T`. `Deref` records a read;
/// `DerefMut` flags a write. Subscribers are only notified (on guard drop) if
/// `DerefMut` was actually used, so a write guard used read-only is free.
pub struct WriteGuard<'a, T: ?Sized> {
    pub(crate) value: &'a mut T,
    pub(crate) log: &'a Log,
    pub(crate) path: Path,
    pub(crate) read_tracked: Cell<bool>,
    pub(crate) wrote: bool,
}

impl<T: ?Sized> std::ops::Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        if !self.read_tracked.replace(true) {
            self.log.record_read(Source::State(self.path));
        }
        self.value
    }
}

impl<T: ?Sized> std::ops::DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.wrote = true;
        self.value
    }
}

impl<T: ?Sized> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        if self.wrote {
            self.log.record_write(std::mem::take(&mut self.path));
        }
    }
}

/// Memo/resource dirtiness, propagated qk-style but lazily.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum NodeState {
    Clean,
    Check,
    Dirty,
}

/// A handle to a lazily-recomputed, equality-gated derived value.
pub struct Memo<T> {
    pub(crate) id: usize,
    pub(crate) _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Memo<T> {}

/// A handle to an eager reactive computation.
#[derive(Clone, Copy)]
pub struct Effect {
    pub(crate) id: usize,
}

impl Effect {
    /// The effect's slot index in its [`crate::Ui`].
    pub fn id(&self) -> usize {
        self.id
    }
}

/// A handle to an async resource.
pub struct ResourceHandle<T> {
    pub(crate) id: usize,
    pub(crate) _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T> Clone for ResourceHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for ResourceHandle<T> {}

/// The observable state of a [`ResourceHandle`]. Stale data is kept visible
/// while a reload is in flight.
#[derive(Clone, PartialEq, Debug)]
pub enum ResourceState<T> {
    /// No value has ever been produced.
    Pending,
    /// The latest run completed with this value.
    Ready(T),
    /// A previous value exists, but dependencies changed and a new run is in flight.
    Reloading(T),
}

impl<T> ResourceState<T> {
    /// The current value, if any (including stale data during a reload).
    pub fn value(&self) -> Option<&T> {
        match self {
            ResourceState::Pending => None,
            ResourceState::Ready(v) | ResourceState::Reloading(v) => Some(v),
        }
    }
}

/// Something owned by an effect scope, torn down when the effect reruns.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Owned {
    Effect(usize),
    Memo(usize),
    Resource(usize),
}

pub(crate) type MemoCompute<S> = Box<dyn FnMut(&mut Ctx<'_, S>, &mut dyn Any) -> bool>;

pub(crate) struct MemoSlot<S: 'static> {
    pub(crate) value: Box<dyn Any>,
    pub(crate) compute: Option<MemoCompute<S>>,
    pub(crate) state: NodeState,
    pub(crate) computing: bool,
    pub(crate) version: u64,
    pub(crate) deps: Vec<Dep>,
    pub(crate) alive: bool,
}

pub(crate) type EffectRun<S> = Box<dyn FnMut(&mut Ctx<'_, S>)>;

pub(crate) struct EffectSlot<S: 'static> {
    pub(crate) run: Option<EffectRun<S>>,
    pub(crate) deps: Vec<Dep>,
    pub(crate) owned: Vec<Owned>,
    pub(crate) depth: u32,
    pub(crate) state: NodeState,
    pub(crate) alive: bool,
}

pub(crate) type ResourceDriver = Pin<Box<dyn Future<Output = Box<dyn Any>>>>;
pub(crate) type ResourceSource<S> = Box<dyn FnMut(&mut Ctx<'_, S>) -> ResourceDriver>;

pub(crate) struct ResourceSlot<S: 'static> {
    pub(crate) value: Box<dyn Any>,
    pub(crate) reload: fn(&mut dyn Any),
    pub(crate) source: Option<ResourceSource<S>>,
    pub(crate) driver: Option<ResourceDriver>,
    pub(crate) complete: fn(&mut dyn Any, Box<dyn Any>),
    pub(crate) woken: Arc<WakeFlag>,
    pub(crate) version: u64,
    pub(crate) deps: Vec<Dep>,
    pub(crate) alive: bool,
}

/// A single-task waker: `wake` just flags the resource for re-polling.
pub(crate) struct WakeFlag(pub(crate) AtomicBool);

impl Wake for WakeFlag {
    fn wake(self: Arc<Self>) {
        self.0.store(true, Ordering::Relaxed);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.store(true, Ordering::Relaxed);
    }
}
