// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Bounded real-window startup/redraw/teardown probe. Run on an interactive host:
//! cargo run --locked -p aether-native --example native_lifecycle_smoke --features prelude
//! This exercises native lifecycle; it does not certify pixel output, device loss,
//! multiple windows, platform input, or the required QualiaDB application profile.
use aether_native::{prelude::*, winit::event::WindowEvent};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static REDRAWS: AtomicUsize = AtomicUsize::new(0);
static DROPPED: AtomicBool = AtomicBool::new(false);

fn app() -> Element {
    let window = aether_native::use_window();
    use_drop(|| DROPPED.store(true, Ordering::SeqCst));
    aether_native::use_window_event(move |event, event_loop| {
        if matches!(event, WindowEvent::RedrawRequested) {
            let frames = REDRAWS.fetch_add(1, Ordering::SeqCst) + 1;
            if frames >= 3 {
                event_loop.exit();
            } else {
                window.request_redraw();
            }
        }
    });
    rsx! {
        main { style: "background: #182132; color: #f4f7ff; padding: 32px; font-family: sans-serif;",
            h1 { "Aether native lifecycle check" }
            p { "Native window, renderer, UI owner and teardown." }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    aether_native::try_launch_cfg(
        app,
        vec![],
        vec![Box::new(
            aether_native::Config::new()
                .with_asset_access(aether_native::AssetAccess::Packaged)
                .with_window_attributes(
                    aether_native::WindowAttributes::default()
                        .with_title("Aether native lifecycle check"),
                ),
        )],
    )?;
    assert!(
        REDRAWS.load(Ordering::SeqCst) >= 3,
        "native redraws did not complete"
    );
    assert!(
        DROPPED.load(Ordering::SeqCst),
        "component owner survived window teardown"
    );
    println!("Native window initialized; three redraw callbacks ran; component owner was dropped.");
    Ok(())
}
