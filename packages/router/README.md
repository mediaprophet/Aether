# Aether Router

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![Discord chat][discord-badge]][discord-url]

[crates-badge]: https://img.shields.io/crates/v/aether-router.svg
[crates-url]: https://crates.io/crates/aether-router
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/aetherlabs/aether/blob/main/LICENSE-MIT
[actions-badge]: https://github.com/aetherlabs/aether/actions/workflows/main.yml/badge.svg
[actions-url]: https://github.com/aetherlabs/aether/actions?query=workflow%3ACI+branch%3Amaster
[discord-badge]: https://img.shields.io/discord/899851952891002890.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/XgGxMSkvUM

[Website](https://aetherlabs.com) |
[Guides](https://aetherlabs.com/learn/0.7/essentials/router/) |
[API Docs](https://docs.rs/aether-router/latest/aether_router) |
[Chat](https://discord.gg/XgGxMSkvUM)

## Overview

Aether Router is a first-party Router for all your Aether Apps. It provides an
interface similar to React Router, but takes advantage of types for more
expressiveness.

```rust, no_run
use aether::prelude::*;
use std::str::FromStr;

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, Routable)]
enum Route {
    #[nest("/blog")]
        #[layout(Blog)]
            #[route("/")]
            BlogList {},

            #[route("/:blog_id")]
            BlogPost { blog_id: usize },
        #[end_layout]
    #[end_nest]
    #[route("/")]
    Index {},
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> { }
    }
}

#[component]
fn Index() -> Element {
    rsx! {
        h1 { "Index" }
        Link {
            to: Route::BlogList {},
            "Go to the blog"
        }
    }
}

#[component]
fn Blog() -> Element {
    rsx! {
        h1 { "Blog" }
        Outlet::<Route> { }
    }
}

#[component]
fn BlogList() -> Element {
    rsx! {
        h2 { "List of blog posts" }
        Link {
            to: Route::BlogPost { blog_id: 0 },
            "Blog post 1"
        }
        Link {
            to: Route::BlogPost { blog_id: 1 },
            "Blog post 2"
        }
    }
}

#[component]
fn BlogPost(blog_id: usize) -> Element {
    rsx! {
        h2 { "Blog Post" }
    }
}
```

You need to enable the right features for the platform you're targeting since these are not determined automatically!

## Bundle Splitting

The Aether Router supports automatic bundle splitting along route variants. To enable this, you need to manually turn on the `wasm-split` feature explicitly on the aether-router crate:

```toml
[dependencies]
aether = { version = "*", features = ["router", "wasm-split"] }
aether-router = { version = "*", features = ["wasm-split"] }
```

Note that `wasm-split` must also be turned on in aether since the macro uses the re-exported `wasm-split` from the aether prelude.

Enabling splitting disconnects the call graph, meaning if you try to run your app with a normal `dx serve`, it won't work. When running with router splitting, you need to pass `--experimental-wasm-split`.

```sh
dx serve --experimental-wasm-split
```

In practice, we recommend passing `aether-router?/wasm-split` as a feature only when bundling:

```sh
dx bundle --features "aether-router?/wasm-split"  --experimental-wasm-split
```

Note that the router will call `.suspend()` so you should add a SuspenseBoundary above the Outlet to prevent suspending the entire page.

## Contributing

- Report issues on our [issue tracker](https://github.com/aetherlabs/aether/issues).
- Join the discord and ask questions!

## License

This project is licensed under the [MIT license].

[mit license]: https://github.com/aetherlabs/aether/blob/main/LICENSE-MIT

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Aether by you shall be licensed as MIT without any additional
terms or conditions.
