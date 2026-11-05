Aether Desktop Renderer

Render the Aether VirtualDom using the platform's native WebView implementation.

# Desktop

One of Aether' flagship features is the ability to quickly build a native desktop app that looks and feels the same across platforms. Apps built with Aether are typically <5mb in size and use existing system resources, so they won't hog extreme amounts of RAM or memory.

Aether Desktop is built off Tauri. Right now there aren't any Aether abstractions over the menubar, handling, etc, so you'll want to leverage Tauri - mostly [Wry](http://github.com/tauri-apps/wry/) and [Tao](http://github.com/tauri-apps/tao) directly. An upcoming release of Aether-Desktop will include components and hooks for notifications, global shortcuts, menubar, etc.

## Getting Set up

Getting Set up with Aether-Desktop is quite easy. Make sure you have Rust and Cargo installed, and then create a new project:

```shell
$ cargo new --bin demo
$ cd app
```

Add Aether and the `desktop` renderer feature:

```shell
$ cargo add aether
$ cargo add aether-desktop
```

Edit your `main.rs`:

```rust, ignore
// main.rs
use aether::prelude::*;

fn main() {
    aether_desktop::launch(app);
}

fn app() -> Element {
    rsx! {
        div {
            "hello world!"
        }
    }
}
```

To configure the webview, menubar, and other important desktop-specific features, checkout out some of the launch configuration in the [API reference](https://docs.rs/aether-desktop/).

## Future Steps

Make sure to read the [Aether Guide](https://aetherlabs.com/learn/0.7/) if you already haven't!
