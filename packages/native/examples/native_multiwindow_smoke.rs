// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Real late-window/context/close regression. Run on an interactive host.
//! This does not qualify platform suspend/resume, pixels, or GPU device recovery.
use aether_native::{
    AetherDocument, AetherNativeApplication, AetherNativeWindowRenderer, DocumentConfig,
    WindowAttributes,
    prelude::*,
    winit::{
        application::ApplicationHandler,
        event::{StartCause, WindowEvent},
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::WindowId,
    },
};
use blitz_shell::{BlitzShellEvent, BlitzShellProxy, WindowConfig};
use blitz_traits::shell::ShellProvider;
use std::{
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

static REDRAWS: [AtomicUsize; 2] = [const { AtomicUsize::new(0) }; 2];
static DROPPED: [AtomicBool; 2] = [const { AtomicBool::new(false) }; 2];

fn probe(index: usize) -> Element {
    let window = aether_native::use_window();
    // Each late window must receive its own document/history and shell contexts
    // before the component is first built.
    let document: Rc<dyn aether_document::Document> = consume_context();
    let _: Rc<dyn aether_history::History> = consume_context();
    let shell: Arc<dyn ShellProvider> = consume_context();
    use_hook(move || {
        document.set_title(format!("Aether lifecycle window {}", index + 1));
        println!("Window {index}: component initialized with native contexts");
    });
    use_drop(move || {
        DROPPED[index].store(true, Ordering::SeqCst);
        println!("Window {index}: component owner dropped");
    });
    aether_native::use_window_event(move |event, _| {
        if matches!(event, WindowEvent::RedrawRequested) {
            let frames = REDRAWS[index].fetch_add(1, Ordering::SeqCst) + 1;
            if frames <= 3 {
                println!("Window {index}: redraw {frames}");
            }
            if index == 1 && frames >= 3 && DROPPED[0].load(Ordering::SeqCst) {
                // The first owner has gone; this window must still dispatch and close.
                shell.request_window_close();
            } else {
                window.request_redraw();
            }
        }
    });
    rsx! {
        main { style: "background: #182132; color: #f4f7ff; padding: 32px;",
            h1 { "Aether native window {index}" }
            p { "Independent document, history, callbacks and teardown." }
        }
    }
}

fn window(index: usize) -> WindowConfig<AetherNativeWindowRenderer> {
    let document = AetherDocument::new(
        VirtualDom::new_with_props(probe, index),
        DocumentConfig::default(),
    );
    WindowConfig::with_attributes(
        Box::new(document),
        AetherNativeWindowRenderer::new(),
        WindowAttributes::default().with_title("Aether native multiwindow check"),
    )
}

struct Scenario {
    inner: AetherNativeApplication,
    proxy: BlitzShellProxy,
    first: Option<WindowId>,
    added_second: bool,
    closed_first: bool,
}

impl ApplicationHandler for Scenario {
    #[cfg(target_os = "macos")]
    fn macos_handler(
        &mut self,
    ) -> Option<&mut dyn aether_native::winit::platform::macos::ApplicationHandlerExtMacOS> {
        self.inner.macos_handler()
    }

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.resumed(event_loop);
    }
    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.suspended(event_loop);
    }
    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.destroy_surfaces(event_loop);
    }
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.can_create_surfaces(event_loop);
    }
    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.about_to_wait(event_loop);
    }
    fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        self.inner.new_events(event_loop, cause);
    }
    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.proxy_wake_up(event_loop);
    }
    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let redraw = matches!(event, WindowEvent::RedrawRequested);
        let first = *self.first.get_or_insert(id);
        self.inner.window_event(event_loop, id, event);
        if redraw && !self.added_second {
            self.added_second = true;
            self.inner.add_window(window(1));
        }
        if redraw && id != first && !self.closed_first {
            // Do not wait for redraws from the first window: the new window may
            // occlude it and the OS may legitimately suppress those redraws.
            self.closed_first = true;
            self.proxy
                .send_event(BlitzShellEvent::CloseWindow { window_id: first });
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let (proxy, queue) = BlitzShellProxy::new(event_loop.create_proxy());
    let inner = AetherNativeApplication::new(proxy.clone(), queue, window(0));
    event_loop.run_app(Scenario {
        inner,
        proxy,
        first: None,
        added_second: false,
        closed_first: false,
    })?;
    for index in 0..2 {
        assert!(
            REDRAWS[index].load(Ordering::SeqCst) >= if index == 0 { 1 } else { 3 },
            "window {index} did not redraw"
        );
        assert!(
            DROPPED[index].load(Ordering::SeqCst),
            "window {index} retained its component owner"
        );
    }
    println!(
        "Both native windows initialized; the late window remained live after the first closed; both owners were dropped."
    );
    Ok(())
}
