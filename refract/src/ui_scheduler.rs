//! Internal reactive dependency graph propagation, memo evaluation, effect
//! execution, and async resource polling for [`crate::ui::Ctx`].

use std::sync::atomic::Ordering;
use std::task::{Context as TaskContext, Poll, Waker};

use crate::lens::paths_overlap;
use crate::ui::Ctx;
use crate::ui_guards::{Dep, NodeState, Observer, Owned, Source};

impl<S: 'static> Ctx<'_, S> {
    pub(crate) fn register_owned(&mut self, owned: Owned) {
        if let Some(Observer::Effect(owner)) = self.log.current_observer() {
            self.effects[owner].owned.push(owned);
        }
    }

    pub(crate) fn drop_owned(&mut self, owned: Owned) {
        match owned {
            Owned::Effect(id) => {
                let slot = &mut self.effects[id];
                slot.alive = false;
                slot.run = None;
                slot.deps.clear();
                let children = std::mem::take(&mut slot.owned);
                for child in children {
                    self.drop_owned(child);
                }
            }
            Owned::Memo(id) => {
                let slot = &mut self.memos[id];
                slot.alive = false;
                slot.compute = None;
                slot.deps.clear();
            }
            Owned::Resource(id) => {
                let slot = &mut self.resources[id];
                slot.alive = false;
                slot.source = None;
                slot.driver = None;
                slot.deps.clear();
            }
        }
    }

    pub(crate) fn run_effect(&mut self, id: usize) {
        if !self.effects[id].alive {
            return;
        }
        let owned = std::mem::take(&mut self.effects[id].owned);
        for child in owned {
            self.drop_owned(child);
        }
        self.effects[id].deps.clear();
        let Some(mut run) = self.effects[id].run.take() else {
            return;
        };
        self.log.push_observer(Observer::Effect(id));
        run(&mut self.reborrow());
        self.log.pop_observer();
        let reads = self.log.drain_reads_for(Observer::Effect(id));
        let deps = self.collect_deps(reads);
        let slot = &mut self.effects[id];
        slot.deps = deps;
        slot.state = NodeState::Clean;
        if slot.run.is_none() {
            slot.run = Some(run);
        }
    }

    fn run_effect_if_stale(&mut self, id: usize) {
        if !self.effects[id].alive || self.effects[id].state == NodeState::Clean {
            return;
        }
        if self.effects[id].state == NodeState::Check {
            let deps = self.effects[id].deps.clone();
            let mut changed = false;
            for dep in &deps {
                match &dep.source {
                    Source::State(_) => {}
                    Source::Memo(m) => {
                        self.ensure_memo(*m);
                        if self.memos[*m].version != dep.version {
                            changed = true;
                            break;
                        }
                    }
                    Source::Resource(r) => {
                        if self.resources[*r].version != dep.version {
                            changed = true;
                            break;
                        }
                    }
                }
            }
            if !changed && self.effects[id].state == NodeState::Check {
                self.effects[id].state = NodeState::Clean;
                return;
            }
        }
        self.run_effect(id);
    }

    fn collect_deps(&self, reads: Vec<Source>) -> Vec<Dep> {
        reads
            .into_iter()
            .map(|source| {
                let version = match &source {
                    Source::State(_) => 0,
                    Source::Memo(id) => self.memos[*id].version,
                    Source::Resource(id) => self.resources[*id].version,
                };
                Dep { source, version }
            })
            .collect()
    }

    pub(crate) fn ensure_memo(&mut self, id: usize) {
        if !self.memos[id].alive {
            return;
        }
        if self.memos[id].computing {
            panic!("reactive cycle detected: a memo depends on its own value");
        }
        match self.memos[id].state {
            NodeState::Clean => return,
            NodeState::Check => {
                let deps = self.memos[id].deps.clone();
                let mut changed = false;
                for dep in &deps {
                    match &dep.source {
                        Source::State(_) => {}
                        Source::Memo(m) => {
                            self.ensure_memo(*m);
                            if self.memos[*m].version != dep.version {
                                changed = true;
                                break;
                            }
                        }
                        Source::Resource(r) => {
                            if self.resources[*r].version != dep.version {
                                changed = true;
                                break;
                            }
                        }
                    }
                }
                if !changed {
                    self.memos[id].state = NodeState::Clean;
                    return;
                }
            }
            NodeState::Dirty => {}
        }
        self.recompute_memo(id);
    }

    fn recompute_memo(&mut self, id: usize) {
        let Some(mut compute) = self.memos[id].compute.take() else {
            return;
        };
        let mut value = std::mem::replace(&mut self.memos[id].value, Box::new(()));
        self.memos[id].computing = true;
        self.memos[id].deps.clear();
        self.log.push_observer(Observer::Memo(id));
        let changed = compute(&mut self.reborrow(), value.as_mut());
        self.log.pop_observer();
        let reads = self.log.drain_reads_for(Observer::Memo(id));
        let deps = self.collect_deps(reads);
        let slot = &mut self.memos[id];
        slot.computing = false;
        slot.deps = deps;
        slot.value = value;
        slot.state = NodeState::Clean;
        if slot.compute.is_none() {
            slot.compute = Some(compute);
        }
        if changed {
            self.memos[id].version += 1;
            self.notify_source_changed(Source::Memo(id));
        }
    }

    fn notify_source_changed(&mut self, source: Source) {
        let mut dirty_memos = Vec::new();
        for (i, m) in self.memos.iter().enumerate() {
            if m.alive && m.deps.iter().any(|d| d.source == source) {
                dirty_memos.push(i);
            }
        }
        for i in &dirty_memos {
            if self.memos[*i].state == NodeState::Clean {
                self.memos[*i].state = NodeState::Dirty;
                self.propagate_check(*i);
            } else {
                self.memos[*i].state = NodeState::Dirty;
            }
        }
        let mut effects = Vec::new();
        for (i, e) in self.effects.iter().enumerate() {
            if e.alive && e.deps.iter().any(|d| d.source == source) {
                effects.push(i);
            }
        }
        for i in effects {
            self.effects[i].state = NodeState::Dirty;
            if !self.queue.contains(&i) {
                self.queue.push(i);
            }
        }
        let mut resources = Vec::new();
        for (i, r) in self.resources.iter().enumerate() {
            if r.alive && r.deps.iter().any(|d| d.source == source) {
                resources.push(i);
            }
        }
        for i in resources {
            self.restart_resource(i);
        }
    }

    fn propagate_check(&mut self, id: usize) {
        let mut stack = vec![Source::Memo(id)];
        while let Some(source) = stack.pop() {
            let mut found = Vec::new();
            for (i, m) in self.memos.iter().enumerate() {
                if m.alive
                    && m.state == NodeState::Clean
                    && m.deps.iter().any(|d| d.source == source)
                {
                    found.push(i);
                }
            }
            for i in found {
                self.memos[i].state = NodeState::Check;
                stack.push(Source::Memo(i));
            }
            let mut effects = Vec::new();
            for (i, e) in self.effects.iter().enumerate() {
                if e.alive && e.deps.iter().any(|d| d.source == source) {
                    effects.push(i);
                }
            }
            for i in effects {
                if self.effects[i].state == NodeState::Clean {
                    self.effects[i].state = NodeState::Check;
                }
                if !self.queue.contains(&i) {
                    self.queue.push(i);
                }
            }
        }
    }

    pub(crate) fn start_resource(&mut self, id: usize) {
        if !self.resources[id].alive {
            return;
        }
        let Some(mut source) = self.resources[id].source.take() else {
            return;
        };
        self.resources[id].deps.clear();
        self.log.push_observer(Observer::Resource(id));
        let driver = source(&mut self.reborrow());
        self.log.pop_observer();
        let reads = self.log.drain_reads_for(Observer::Resource(id));
        let deps = self.collect_deps(reads);
        let slot = &mut self.resources[id];
        slot.deps = deps;
        slot.driver = Some(driver);
        slot.woken.0.store(true, Ordering::Relaxed);
        if slot.source.is_none() {
            slot.source = Some(source);
        }
    }

    fn restart_resource(&mut self, id: usize) {
        if !self.resources[id].alive {
            return;
        }
        self.resources[id].driver = None;
        let reload = self.resources[id].reload;
        reload(self.resources[id].value.as_mut());
        self.start_resource(id);
        self.notify_source_changed(Source::Resource(id));
    }

    fn poll_resources(&mut self) -> bool {
        let mut polled = false;
        for id in 0..self.resources.len() {
            if !self.resources[id].woken.0.swap(false, Ordering::Relaxed) {
                continue;
            }
            let Some(mut driver) = self.resources[id].driver.take() else {
                continue;
            };
            polled = true;
            let waker = Waker::from(self.resources[id].woken.clone());
            let mut cx = TaskContext::from_waker(&waker);
            match driver.as_mut().poll(&mut cx) {
                Poll::Ready(out) => {
                    let complete = self.resources[id].complete;
                    complete(self.resources[id].value.as_mut(), out);
                    self.resources[id].version += 1;
                    self.notify_source_changed(Source::Resource(id));
                }
                Poll::Pending => {
                    self.resources[id].driver = Some(driver);
                }
            }
        }
        polled
    }

    pub(crate) fn flush(&mut self) {
        loop {
            let writes = self.log.writes.take();
            if writes.is_empty() && self.queue.is_empty() {
                break;
            }
            for path in writes {
                self.apply_write(&path);
            }
            if !self.queue.is_empty() {
                let mut best = 0;
                for (i, id) in self.queue.iter().enumerate() {
                    if self.effects[*id].depth < self.effects[self.queue[best]].depth {
                        best = i;
                    }
                }
                let id = self.queue.remove(best);
                self.run_effect_if_stale(id);
            }
        }
    }

    fn apply_write(&mut self, path: &[u32]) {
        let mut dirty = Vec::new();
        for (i, m) in self.memos.iter().enumerate() {
            if m.alive
                && m.deps
                    .iter()
                    .any(|d| matches!(&d.source, Source::State(p) if paths_overlap(p, path)))
            {
                dirty.push(i);
            }
        }
        for i in dirty {
            self.memos[i].state = NodeState::Dirty;
            self.propagate_check(i);
        }
        let mut effects = Vec::new();
        for (i, e) in self.effects.iter().enumerate() {
            if e.alive
                && e.deps
                    .iter()
                    .any(|d| matches!(&d.source, Source::State(p) if paths_overlap(p, path)))
            {
                effects.push(i);
            }
        }
        for i in effects {
            self.effects[i].state = NodeState::Dirty;
            if !self.queue.contains(&i) {
                self.queue.push(i);
            }
        }
        let mut resources = Vec::new();
        for (i, r) in self.resources.iter().enumerate() {
            if r.alive
                && r.deps
                    .iter()
                    .any(|d| matches!(&d.source, Source::State(p) if paths_overlap(p, path)))
            {
                resources.push(i);
            }
        }
        for i in resources {
            self.restart_resource(i);
        }
    }

    pub(crate) fn run_until_settled(&mut self) -> bool {
        loop {
            self.flush();
            let progressed = self.poll_resources();
            self.flush();
            if !progressed {
                return self.resources.iter().all(|r| r.driver.is_none());
            }
        }
    }
}
