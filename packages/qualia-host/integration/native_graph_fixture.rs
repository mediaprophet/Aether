// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! TEST ONLY: real signed disposable authority, Vibe, journal and native window.
//! No user project, enrolled human, production key or fallback Host is used.
mod fixture_support;
use aether::prelude::*;
use aether_native::winit::event::WindowEvent;
use aether_qualia_host::{
    ExecutionLimits, NativeGraphBridge, NativeGraphView, UiReceiver, UiSender,
    native_graph_runtime,
    native_ui::{use_qualia_delivery, use_qualia_graph},
    qualia_core_db::{
        NQuin,
        vibe_host::{
            native_authority::{
                NativeAuthority, SessionHandle,
                graph::{GraphError, GraphReceipt, SignedGraphGrant},
            },
            vibe_sentinel::WebizenSentinelGate,
        },
    },
    ui_channel,
    vibe::{Value, extension_registry::CallExecutionContext},
    vibescript_abi::{EffectSet, ExecutionMode, ResourceEnvelope, q_hash_str},
};
use fixture_support::{CONTEXT, Fixture, SOURCE, SUBJECT, authenticate};
use std::{
    cell::RefCell,
    error::Error,
    num::NonZeroUsize,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

#[derive(Default)]
struct Observed {
    zero: AtomicBool,
    one: AtomicBool,
    delivered: AtomicBool,
    dropped: AtomicBool,
    timed_out: AtomicBool,
    ready_redraws: AtomicUsize,
    snapshot: Mutex<Option<NQuin>>,
    result: Mutex<Option<[u64; 6]>>,
}
#[derive(Clone)]
struct Props {
    authority: Arc<NativeAuthority>,
    session: SessionHandle,
    grant: SignedGraphGrant,
    view: Rc<RefCell<Option<NativeGraphView>>>,
    receiver: Rc<RefCell<Option<UiReceiver>>>,
    sender: UiSender,
    cancelled: Arc<AtomicBool>,
    observed: Arc<Observed>,
    worker: Arc<Mutex<Option<JoinHandle<GraphReceipt>>>>,
}
fn app(props: Props) -> Element {
    let mut label = use_signal(|| "Waiting for authorized revision 0".to_owned());
    let window = aether_native::use_window();
    let started = use_hook(Instant::now);
    let events = props.observed.clone();
    aether_native::use_window_event(move |event, event_loop| {
        if matches!(event, WindowEvent::RedrawRequested) {
            if events.one.load(Ordering::Acquire)
                && events.delivered.load(Ordering::Acquire)
                && events.ready_redraws.fetch_add(1, Ordering::AcqRel) >= 2
            {
                event_loop.exit();
            } else if started.elapsed() > Duration::from_secs(30) {
                events.timed_out.store(true, Ordering::Release);
                event_loop.exit();
            }
        }
    });
    let dropped = props.observed.clone();
    use_drop(move || dropped.dropped.store(true, Ordering::Release));
    let receiver = props.receiver.clone();
    let delivered = props.observed.clone();
    use_qualia_delivery(
        move || receiver.borrow_mut().take().unwrap(),
        NonZeroUsize::new(1).unwrap(),
        move |delivery| {
            let Value::List(rows) = delivery.value else {
                panic!("expected canonical query rows");
            };
            assert_eq!(rows.len(), 1);
            let Value::QuinRef(row) = &rows[0] else {
                panic!("expected canonical QuinRef");
            };
            *delivered.result.lock().unwrap() = Some(row.raw_fields());
            delivered.delivered.store(true, Ordering::Release);
        },
    );
    let view = props.view.clone();
    let owner = props.clone();
    use_qualia_graph(
        move || view.borrow_mut().take().unwrap(),
        props.cancelled.clone(),
        move || {
            let window = window.clone();
            async move {
                futures_timer::Delay::new(Duration::from_millis(100)).await;
                window.request_redraw(); // Real events enforce the bounded exit deadline.
                true
            }
        },
        move |update| match update {
            Ok(Some(snapshot)) if snapshot.read.revision == 0 => {
                assert!(snapshot.quins.is_empty());
                assert!(!owner.observed.zero.swap(true, Ordering::AcqRel));
                label.set("Revision 0 observed; executing authorized Vibe mutation".into());
                let worker_owner = owner.clone();
                // Only Send canonical services cross into the worker, never the UI view.
                let authority = worker_owner.authority.clone();
                let session = worker_owner.session.clone();
                let grant = worker_owner.grant.clone();
                let sender = worker_owner.sender.clone();
                let cancelled = worker_owner.cancelled.clone();
                let handle = std::thread::spawn(move || {
                    execute(authority, session, grant, sender, cancelled)
                });
                *owner.worker.lock().unwrap() = Some(handle);
            }
            Ok(Some(snapshot)) => {
                assert!(owner.observed.zero.load(Ordering::Acquire));
                assert_eq!(snapshot.read.revision, 1);
                assert_eq!(snapshot.quins.len(), 1);
                *owner.observed.snapshot.lock().unwrap() = Some(snapshot.quins[0]);
                owner.observed.one.store(true, Ordering::Release);
                label.set("Revision 1 delivered to the native UI; checking teardown".into());
            }
            Ok(None) | Err(GraphError::Busy) | Err(GraphError::Cancelled) => {}
            Err(error) => panic!("unexpected native graph failure: {error}"),
        },
    );
    rsx! { main { style: "padding:32px;background:#172133;color:white;font-family:sans-serif;",
        h1 { "Disposable signed native graph fixture — TEST ONLY" }
        p { "{label}" }
        p { "Actual canonical signatures, scoped graph journal, Vibe engine and native UI owner." }
    }}
}
fn execute(
    authority: Arc<NativeAuthority>,
    session: SessionHandle,
    grant: SignedGraphGrant,
    sender: UiSender,
    cancelled: Arc<AtomicBool>,
) -> GraphReceipt {
    let mut audit = [NQuin::default(); 2];
    let gate = WebizenSentinelGate::new(
        &[],
        &mut audit,
        1,
        EffectSet::PURE_COMPUTE,
        ResourceEnvelope::hot_zero_heap(100_000, 4096),
    );
    let mut runtime = native_graph_runtime(
        authority,
        &session,
        &grant,
        gate,
        ExecutionLimits::default(),
        sender,
        cancelled,
    )
    .unwrap();
    let program = runtime.prepare(SOURCE).unwrap();
    let outcome = runtime.execute(
        program,
        CallExecutionContext {
            actor_did_hash: q_hash_str(SUBJECT),
            context_world_hash: CONTEXT,
            lamport_timestamp: 1,
            mode: ExecutionMode::GovernedApplication,
        },
    );
    let (host, _) = runtime.into_services();
    let receipt = host.last_receipt().cloned();
    assert!(
        outcome.is_ok(),
        "execution failed: {outcome:?}; acknowledged receipt: {receipt:?}"
    );
    receipt.expect("durable native acknowledgement")
}
fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "TEST ONLY: creating a disposable signed software-agent project; no production authority."
    );
    let fixture = Fixture::new();
    let authority = fixture.open();
    let session = authenticate(&authority);
    let grant = fixture.grant();
    let view = NativeGraphBridge::connect(authority.clone(), &session, &grant)?.subscribe()?;
    let (sender, receiver) = ui_channel(1, || {})?;
    let cancelled = Arc::new(AtomicBool::new(false));
    let observed = Arc::new(Observed::default());
    let worker = Arc::new(Mutex::new(None));
    let props = Props {
        authority: authority.clone(),
        session: session.clone(),
        grant: grant.clone(),
        view: Rc::new(RefCell::new(Some(view))),
        receiver: Rc::new(RefCell::new(Some(receiver))),
        sender,
        cancelled: cancelled.clone(),
        observed: observed.clone(),
        worker: worker.clone(),
    };
    let launch = aether_native::try_launch_cfg_with_props(
        app,
        props,
        vec![],
        vec![Box::new(
            aether_native::Config::new()
                .with_asset_access(aether_native::AssetAccess::Packaged)
                .with_window_attributes(
                    aether_native::WindowAttributes::default()
                        .with_title("Aether disposable signed graph fixture — TEST ONLY"),
                ),
        )],
    );
    // Join only outside the event loop; no worker ever waits for a UI acknowledgement.
    let was_cancelled = cancelled.swap(true, Ordering::AcqRel);
    let receipt = worker
        .lock()
        .unwrap()
        .take()
        .map(|h| h.join().expect("fixture worker panicked"));
    launch?;
    assert!(
        !observed.timed_out.load(Ordering::Acquire),
        "native fixture exceeded 30-second window deadline"
    );
    assert!(observed.zero.load(Ordering::Acquire) && observed.one.load(Ordering::Acquire));
    assert!(observed.delivered.load(Ordering::Acquire) && observed.dropped.load(Ordering::Acquire));
    assert!(
        was_cancelled,
        "actual component teardown must cancel the host token"
    );
    assert!(observed.ready_redraws.load(Ordering::Acquire) >= 3);
    assert_eq!(receipt.unwrap().revision, 1);
    let bridge = NativeGraphBridge::connect(authority.clone(), &session, &grant)?;
    assert_eq!(
        bridge.replace(1, &[], &cancelled),
        Err(GraphError::Cancelled)
    );
    drop(bridge);
    authority.disconnect(&session)?;
    drop(authority);
    let reopened = fixture.open();
    let fresh_session = authenticate(&reopened);
    let scope = reopened.bind_graph_scope(&fresh_session, &grant)?;
    let mut rows = [NQuin::default(); 8];
    let read = reopened.read_graph(&scope, &mut rows)?;
    assert_eq!((read.revision, read.written), (1, 1));
    assert_eq!(Some(rows[0]), *observed.snapshot.lock().unwrap());
    assert_eq!(
        Some([
            rows[0].subject,
            rows[0].predicate,
            rows[0].object,
            rows[0].context,
            rows[0].metadata,
            rows[0].parity
        ]),
        *observed.result.lock().unwrap()
    );
    reopened.disconnect(&fresh_session)?;
    println!(
        "Disposable signed native fixture passed: revision 0 → Vibe commit → revision 1 UI → teardown → fresh-session reopen, all six lanes preserved."
    );
    Ok(())
}
