//! The runtime. [`Ui`] owns the root state, all memo/resource values, the
//! retained DOM arena, and every reactive closure. Closures receive a
//! [`Ctx`] — a set of split `&mut` borrows over the runtime — so all access
//! to state is checked statically by the borrow checker. There is no
//! `RefCell` and no runtime borrow counting anywhere.

use std::any::Any;
use std::cell::Cell;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::dom::Dom;
use crate::lens::Lens;
pub use crate::ui_guards::{
    Effect, Memo, ReadGuard, ResourceHandle, ResourceState, WriteGuard,
};
use crate::ui_guards::{
    EffectSlot, Log, MemoSlot, NodeState, Observer, Owned, ResourceSlot, Source, WakeFlag,
};

/// The reactive runtime: owns the root state and everything derived from it.
pub struct Ui<S: 'static> {
    state: S,
    log: Log,
    memos: Vec<MemoSlot<S>>,
    effects: Vec<EffectSlot<S>>,
    resources: Vec<ResourceSlot<S>>,
    dom: Dom<S>,
    /// Effects queued to rerun, by id.
    queue: Vec<usize>,
}

impl<S: 'static> Ui<S> {
    /// Create a runtime owning `state`.
    pub fn new(state: S) -> Self {
        Ui {
            state,
            log: Log::default(),
            memos: Vec::new(),
            effects: Vec::new(),
            resources: Vec::new(),
            dom: Dom::default(),
            queue: Vec::new(),
        }
    }

    fn ctx(&mut self) -> Ctx<'_, S> {
        Ctx {
            state: &mut self.state,
            log: &self.log,
            memos: &mut self.memos,
            effects: &mut self.effects,
            resources: &mut self.resources,
            dom: &mut self.dom,
            queue: &mut self.queue,
        }
    }

    /// Run `f` with a [`Ctx`] and flush all resulting updates.
    pub fn with<R>(&mut self, f: impl FnOnce(&mut Ctx<'_, S>) -> R) -> R {
        let r = f(&mut self.ctx());
        self.flush();
        r
    }

    /// Read through a lens without tracking (there is no observer at the
    /// top level anyway).
    pub fn get<L: Lens<S>>(&self, lens: L) -> &L::Target {
        lens.get(&self.state)
    }

    /// Create an eager effect. It runs immediately and reruns whenever
    /// anything it read changes.
    pub fn effect(&mut self, run: impl FnMut(&mut Ctx<'_, S>) + 'static) -> Effect {
        let e = self.ctx().effect(run);
        self.flush();
        e
    }

    /// Create a lazily-recomputed, `PartialEq`-gated memo.
    pub fn memo<T: PartialEq + 'static>(
        &mut self,
        compute: impl FnMut(&mut Ctx<'_, S>) -> T + 'static,
    ) -> Memo<T> {
        self.ctx().memo(compute)
    }

    /// Create an async resource. `source` runs synchronously with dependency
    /// tracking and returns the future to drive; when any tracked dependency
    /// changes, the in-flight future is dropped (cancelled) and a new one is
    /// started, keeping the previous value visible as
    /// [`ResourceState::Reloading`].
    pub fn resource<T, F>(
        &mut self,
        source: impl FnMut(&mut Ctx<'_, S>) -> F + 'static,
    ) -> ResourceHandle<T>
    where
        T: 'static,
        F: Future<Output = T> + 'static,
    {
        let r = self.ctx().resource(source);
        self.flush();
        r
    }

    /// Read a memo's current value, recomputing it first if needed.
    pub fn read_memo<T: 'static>(&mut self, memo: Memo<T>) -> &T {
        self.flush();
        let mut ctx = self.ctx();
        ctx.ensure_memo(memo.id);
        self.memos[memo.id]
            .value
            .downcast_ref::<Option<T>>()
            .expect("memo type mismatch")
            .as_ref()
            .expect("memo value missing")
    }

    /// Read a resource's current state.
    pub fn read_resource<T: 'static>(&mut self, resource: ResourceHandle<T>) -> &ResourceState<T> {
        self.flush();
        self.resources[resource.id]
            .value
            .downcast_ref::<ResourceState<T>>()
            .expect("resource type mismatch")
    }

    /// Write through a lens from outside any observer. Call [`Ui::flush`]
    /// afterwards (or use [`Ui::with`], which flushes automatically).
    pub fn write<L: Lens<S>>(&mut self, lens: L) -> WriteGuard<'_, L::Target> {
        let path = lens.path();
        WriteGuard {
            value: lens.get_mut(&mut self.state),
            log: &self.log,
            path,
            read_tracked: Cell::new(false),
            wrote: false,
        }
    }

    /// Apply pending writes and rerun invalidated effects until settled.
    pub fn flush(&mut self) {
        self.ctx().flush();
    }

    /// Poll resource futures and flush, repeating until nothing is pending
    /// or progress stops. Returns `true` if all resources are settled.
    pub fn run_until_settled(&mut self) -> bool {
        self.ctx().run_until_settled()
    }

    /// Render a mounted DOM node to an HTML string.
    pub fn render_to_string(&mut self, node: crate::dom::NodeId) -> String {
        self.flush();
        self.dom.render_to_string(node)
    }

    /// Mount a DOM tree, registering effects for its dynamic parts.
    pub fn mount(&mut self, node: impl Into<crate::dom::Node<S>>) -> crate::dom::NodeId {
        let node = node.into();
        let id = self.with(|ctx| crate::dom::mount(ctx, node));
        self.flush();
        id
    }
}

/// Split `&mut` borrows over a [`Ui`], handed to reactive closures. All state
/// access flows through this, so aliasing is impossible by construction.
pub struct Ctx<'a, S: 'static> {
    pub(crate) state: &'a mut S,
    pub(crate) log: &'a Log,
    pub(crate) memos: &'a mut Vec<MemoSlot<S>>,
    pub(crate) effects: &'a mut Vec<EffectSlot<S>>,
    pub(crate) resources: &'a mut Vec<ResourceSlot<S>>,
    pub(crate) dom: &'a mut Dom<S>,
    pub(crate) queue: &'a mut Vec<usize>,
}

impl<S: 'static> Ctx<'_, S> {
    pub(crate) fn reborrow(&mut self) -> Ctx<'_, S> {
        Ctx {
            state: self.state,
            log: self.log,
            memos: self.memos,
            effects: self.effects,
            resources: self.resources,
            dom: self.dom,
            queue: self.queue,
        }
    }

    /// Read through a lens. Tracked: the current observer subscribes to the
    /// lens' path. Multiple read guards may be alive at once; the borrow
    /// checker prevents overlap with any write guard.
    pub fn get<L: Lens<S>>(&self, lens: L) -> ReadGuard<'_, L::Target> {
        ReadGuard {
            value: lens.get(&*self.state),
            log: self.log,
            source: Some(Source::State(lens.path())),
            tracked: Cell::new(false),
        }
    }

    /// Read through a lens without subscribing.
    pub fn peek<L: Lens<S>>(&self, lens: L) -> &L::Target {
        lens.get(&*self.state)
    }

    /// Write through a lens. The write is only published (on guard drop) if
    /// `DerefMut` was used.
    pub fn write<L: Lens<S>>(&mut self, lens: L) -> WriteGuard<'_, L::Target> {
        let path = lens.path();
        WriteGuard {
            value: lens.get_mut(self.state),
            log: self.log,
            path,
            read_tracked: Cell::new(false),
            wrote: false,
        }
    }

    /// Run `f` without dependency tracking.
    pub fn untracked<R>(&mut self, f: impl FnOnce(&mut Ctx<'_, S>) -> R) -> R {
        let saved = self.log.observers.take();
        let r = f(&mut self.reborrow());
        self.log.observers.set(saved);
        r
    }

    /// Create a nested effect, owned by the current observer (torn down when
    /// the owner reruns).
    pub fn effect(&mut self, run: impl FnMut(&mut Ctx<'_, S>) + 'static) -> Effect {
        let depth = match self.log.current_observer() {
            Some(Observer::Effect(id)) => self.effects[id].depth + 1,
            _ => 0,
        };
        let id = self.effects.len();
        self.effects.push(EffectSlot {
            run: Some(Box::new(run)),
            deps: Vec::new(),
            owned: Vec::new(),
            depth,
            state: NodeState::Clean,
            alive: true,
        });
        self.register_owned(Owned::Effect(id));
        self.run_effect(id);
        Effect { id }
    }

    /// Create a memo, owned by the current observer if any.
    pub fn memo<T: PartialEq + 'static>(
        &mut self,
        mut compute: impl FnMut(&mut Ctx<'_, S>) -> T + 'static,
    ) -> Memo<T> {
        let id = self.memos.len();
        self.memos.push(MemoSlot {
            value: Box::new(None::<T>),
            compute: Some(Box::new(move |ctx, cell| {
                let new = compute(ctx);
                let cell = cell
                    .downcast_mut::<Option<T>>()
                    .expect("memo type mismatch");
                let changed = cell.as_ref() != Some(&new);
                *cell = Some(new);
                changed
            })),
            state: NodeState::Dirty,
            computing: false,
            version: 0,
            deps: Vec::new(),
            alive: true,
        });
        self.register_owned(Owned::Memo(id));
        Memo {
            id,
            _marker: std::marker::PhantomData,
        }
    }

    /// Read a memo, recomputing it first if stale. Tracked.
    pub fn read_memo<T: 'static>(&mut self, memo: Memo<T>) -> &T {
        self.ensure_memo(memo.id);
        self.log.record_read(Source::Memo(memo.id));
        self.memos[memo.id]
            .value
            .downcast_ref::<Option<T>>()
            .expect("memo type mismatch")
            .as_ref()
            .expect("memo value missing")
    }

    /// Create a resource, owned by the current observer if any.
    pub fn resource<T, F>(
        &mut self,
        mut source: impl FnMut(&mut Ctx<'_, S>) -> F + 'static,
    ) -> ResourceHandle<T>
    where
        T: 'static,
        F: Future<Output = T> + 'static,
    {
        let id = self.resources.len();
        self.resources.push(ResourceSlot {
            value: Box::new(ResourceState::<T>::Pending),
            reload: |cell| {
                let cell = cell
                    .downcast_mut::<ResourceState<T>>()
                    .expect("resource type mismatch");
                if let ResourceState::Ready(v) =
                    std::mem::replace(cell, ResourceState::<T>::Pending)
                {
                    *cell = ResourceState::Reloading(v);
                }
            },
            source: Some(Box::new(move |ctx| {
                let fut = source(ctx);
                Box::pin(async move { Box::new(fut.await) as Box<dyn Any> })
            })),
            driver: None,
            complete: |cell, out| {
                let cell = cell
                    .downcast_mut::<ResourceState<T>>()
                    .expect("resource type mismatch");
                let value = *out.downcast::<T>().expect("resource output mismatch");
                *cell = ResourceState::Ready(value);
            },
            woken: Arc::new(WakeFlag(AtomicBool::new(true))),
            version: 0,
            deps: Vec::new(),
            alive: true,
        });
        self.register_owned(Owned::Resource(id));
        self.start_resource(id);
        ResourceHandle {
            id,
            _marker: std::marker::PhantomData,
        }
    }

    /// Read a resource's state. Tracked.
    pub fn read_resource<T: 'static>(&mut self, resource: ResourceHandle<T>) -> &ResourceState<T> {
        self.log.record_read(Source::Resource(resource.id));
        self.resources[resource.id]
            .value
            .downcast_ref::<ResourceState<T>>()
            .expect("resource type mismatch")
    }
}
