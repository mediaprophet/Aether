//! A simple example on how to use assets loading from the filesystem.
//!
//! Aether provides the asset!() macro which is a convenient way to load assets from the filesystem.
//! This ensures the asset makes it into the bundle through dependencies and is accessible in environments
//! like web and android where assets are lazily loaded using platform-specific APIs.

use aether::prelude::*;

static ASSET_PATH: Asset = asset!("/examples/assets/logo.png");

fn main() {
    aether::launch(app);
}

fn app() -> Element {
    rsx! {
        div {
            h1 { "This should show an image:" }
            img { src: ASSET_PATH }
        }
    }
}
