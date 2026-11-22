// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Actual native application; compiled by the opt-in local integration consumer.
//! Explicit adopted project inputs and external signer only. No fixture authority.
#[allow(dead_code)]
#[path = "../examples/native_graph.rs"]
mod startup;

use aether::prelude::*;
use aether_qualia_host::{
    ExecutionLimits, NativeGraphBridge, NativeGraphView, UiReceiver, UiSender,
    native_graph_runtime,
    native_ui::{use_qualia_delivery, use_qualia_graph},
    qualia_core_db::{
        NQuin,
        vibe_host::{native_authority::graph::GraphError, vibe_sentinel::WebizenSentinelGate},
    },
    ui_channel,
    vibe::extension_registry::CallExecutionContext,
    vibescript_abi::{EffectSet, ExecutionMode, ResourceEnvelope, q_hash_str},
};
use std::{
    cell::RefCell,
    error::Error,
    fmt::Write,
    num::NonZeroUsize,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

#[derive(Clone)]
struct AppProps {
    inputs: Arc<startup::AuthenticatedInputs>,
    view: Rc<RefCell<Option<NativeGraphView>>>,
    receiver: Rc<RefCell<Option<UiReceiver>>>,
    sender: UiSender,
    cancelled: Arc<AtomicBool>,
    worker: Arc<Mutex<Option<JoinHandle<()>>>>,
    outcome: Arc<Mutex<Option<String>>>,
}

fn app(props: AppProps) -> Element {
    let mut status = use_signal(|| {
        "Signed session ready. Review the supplied source before running it.".to_owned()
    });
    let mut snapshot = use_signal(String::new);
    let mut result = use_signal(String::new);
    let mut started = use_signal(|| false);
    let context = props.inputs.grant.grant.context;
    let view_slot = props.view.clone();
    let outcome = props.outcome.clone();
    let worker = props.worker.clone();
    let mut completion_observed = false;
    let cancellation = props.cancelled.clone();
    use_qualia_graph(
        move || view_slot.borrow_mut().take().expect("one graph owner"),
        props.cancelled.clone(),
        || async {
            futures_timer::Delay::new(Duration::from_millis(100)).await;
            true
        },
        move |update| {
            // The UI never waits behind a worker or a journal write.
            if let Ok(mut pending) = outcome.try_lock() {
                if let Some(message) = pending.take() {
                    completion_observed = true;
                    status.set(message);
                } else if !completion_observed
                    && worker
                        .try_lock()
                        .is_ok_and(|slot| slot.as_ref().is_some_and(JoinHandle::is_finished))
                {
                    // Holding the outcome lock prevents a normal worker from
                    // finishing publication between this check and observation.
                    completion_observed = true;
                    status.set(
                        "Worker ended without an outcome; inspect the retained journal.".into(),
                    );
                }
            }
            match update {
                Ok(Some(view)) => {
                    // Copy only a bounded preview on revision changes, not each tick.
                    let mut text = String::with_capacity(4096);
                    let _ = writeln!(
                        text,
                        "Revision {} — {} quins ({} coalesced revisions)",
                        view.read.revision,
                        view.quins.len(),
                        view.skipped_revisions
                    );
                    for q in view.quins.iter().take(16) {
                        let _ = writeln!(
                            text,
                            "{:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
                            q.subject, q.predicate, q.object, q.context, q.metadata, q.parity
                        );
                    }
                    snapshot.set(text);
                }
                Ok(None) | Err(GraphError::Busy) => {}
                Err(error) => {
                    snapshot.set(String::new());
                    result.set(String::new());
                    status.set(format!("Graph access closed: {error}"));
                    cancellation.store(true, Ordering::Release);
                }
            }
        },
    );
    let receiver_slot = props.receiver.clone();
    use_qualia_delivery(
        move || receiver_slot.borrow_mut().take().expect("one result owner"),
        NonZeroUsize::new(1).unwrap(),
        move |_| result.set("Canonical Vibe result delivered on the native UI owner.".into()),
    );
    rsx! {
        main { style: "background:#172133;color:#f3f6fc;padding:28px;font-family:sans-serif;",
            h1 { "Aether · signed project graph" }
            p { "Context {context}. This view reads the native authority journal." }
            p { "{status}" }
            button {
                disabled: started() || props.cancelled.load(Ordering::Acquire),
                onclick: move |_| {
                    started.set(true);
                    status.set("Executing the supplied Vibe source on its worker…".into());
                    let inputs = props.inputs.clone();
                    let sender = props.sender.clone();
                    let cancelled = props.cancelled.clone();
                    let outcome = props.outcome.clone();
                    match std::thread::Builder::new().name("aether-vibe".into())
                        .spawn(move || execute_worker(inputs, sender, cancelled, outcome)) {
                        Ok(handle) => *props.worker.lock().expect("worker owner") = Some(handle),
                        Err(error) => { started.set(false); status.set(format!("Worker could not start: {error}")); }
                    }
                },
                "Run supplied Vibe source once"
            }
            p { "{result}" }
            pre { style: "font-size:11px;white-space:pre-wrap;", "{snapshot}" }
            p { "Preview: first 16 quins, all six canonical lanes. Closing cancels future work; acknowledged commits remain in the journal." }
        }
    }
}

fn execute_worker(
    inputs: Arc<startup::AuthenticatedInputs>,
    sender: UiSender,
    cancelled: Arc<AtomicBool>,
    outcome: Arc<Mutex<Option<String>>>,
) {
    let mut audit = [NQuin::default(); 2];
    let gate = WebizenSentinelGate::new(
        &[],
        &mut audit,
        1,
        EffectSet::PURE_COMPUTE,
        ResourceEnvelope::hot_zero_heap(100_000, 4096),
    );
    let message = match native_graph_runtime(
        inputs.authority.clone(),
        &inputs.session,
        &inputs.grant,
        gate,
        ExecutionLimits::default(),
        sender,
        cancelled,
    ) {
        Ok(mut runtime) => {
            let result = runtime.prepare(&inputs.source).and_then(|program| {
                runtime.execute(
                    program,
                    CallExecutionContext {
                        actor_did_hash: q_hash_str(&inputs.subject),
                        context_world_hash: inputs.grant.grant.context,
                        lamport_timestamp: 0,
                        mode: ExecutionMode::GovernedApplication,
                    },
                )
            });
            let (host, _) = runtime.into_services();
            let receipt = host.last_receipt().map(|r| r.revision);
            if let Err(error) = &result {
                eprintln!("Vibe execution/delivery failed: {error}");
            }
            // Report acknowledged effects even after the UI receiver has closed.
            format!(
                "Execution {}. Last acknowledged graph revision: {:?}. Inspect the journal before retrying.",
                if result.is_ok() {
                    "completed"
                } else {
                    "failed"
                },
                receipt
            )
        }
        Err(error) => {
            eprintln!("Runtime admission failed: {error}");
            "Runtime admission failed; no script executed.".into()
        }
    };
    println!("{message}");
    if let Ok(mut pending) = outcome.lock() {
        *pending = Some(message);
    }
}

// This owner lives outside the event loop: component teardown only cancels, and
// joining happens after the native launcher returns (or during outer unwinding).
struct SessionOwner {
    inputs: Arc<startup::AuthenticatedInputs>,
    cancelled: Arc<AtomicBool>,
    worker: Arc<Mutex<Option<JoinHandle<()>>>>,
}
impl Drop for SessionOwner {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        if let Ok(mut worker) = self.worker.lock() {
            if let Some(handle) = worker.take() {
                if handle.join().is_err() {
                    eprintln!("Vibe worker panicked; inspect the retained native journal.");
                }
            }
        }
        if let Err(error) = self.inputs.authority.disconnect(&self.inputs.session) {
            eprintln!("Session disconnect requires reconciliation: {error}");
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let Some(inputs) = startup::authenticate_inputs()? else {
        return Ok(());
    };
    let inputs = Arc::new(inputs);
    let cancelled = Arc::new(AtomicBool::new(false));
    let worker = Arc::new(Mutex::new(None));
    let _session_owner = SessionOwner {
        inputs: inputs.clone(),
        cancelled: cancelled.clone(),
        worker: worker.clone(),
    };
    let view =
        NativeGraphBridge::connect(inputs.authority.clone(), &inputs.session, &inputs.grant)?
            .subscribe()?;
    let (sender, receiver) = ui_channel(1, || {})?;
    let props = AppProps {
        inputs,
        view: Rc::new(RefCell::new(Some(view))),
        receiver: Rc::new(RefCell::new(Some(receiver))),
        sender,
        cancelled,
        worker,
        outcome: Arc::new(Mutex::new(None)),
    };
    aether_native::try_launch_cfg_with_props(
        app,
        props,
        vec![],
        vec![Box::new(
            aether_native::Config::new()
                .with_asset_access(aether_native::AssetAccess::Packaged)
                .with_window_attributes(
                    aether_native::WindowAttributes::default()
                        .with_title("Aether · signed project graph"),
                ),
        )],
    )?;
    Ok(())
}
