# Aether Fullstack

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![Discord chat][discord-badge]][discord-url]

[crates-badge]: https://img.shields.io/crates/v/aether-fullstack.svg
[crates-url]: https://crates.io/crates/aether-fullstack
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/aetherlabs/aether/blob/main/LICENSE-MIT
[actions-badge]: https://github.com/aetherlabs/aether/actions/workflows/main.yml/badge.svg
[actions-url]: https://github.com/aetherlabs/aether/actions?query=workflow%3ACI+branch%3Amaster
[discord-badge]: https://img.shields.io/discord/899851952891002890.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/XgGxMSkvUM

[Website](https://aetherlabs.com) |
[Guides](https://aetherlabs.com/learn/0.7/) |
[API Docs](https://docs.rs/aether-fullstack/latest/aether_fullstack/) |
[Chat](https://discord.gg/XgGxMSkvUM)

Fullstack utilities for the [`Aether`](https://aetherlabs.com) framework.

# Features

- Integrates with the [Axum](./examples/axum-hello-world/src/main.rs) server framework with utilities for serving and rendering Aether applications.
- Server functions allow you to call code on the server from the client as if it were a normal function.
- Instant RSX Hot reloading with [`aether-hot-reload`](https://crates.io/crates/aether-hot-reload).
- Passing root props from the server to the client.

# Example
Quickly build fullstack Rust apps with axum and aether.

```rust, no_run
use aether::prelude::*;

fn main() {
    aether::launch(|| {
        let mut meaning = use_action(|| get_meaning("life the universe and everything".into()));

        rsx! {
            h1 { "Meaning of life: {meaning:?}" }
            button { onclick: move |_| meaning.call(), "Run a server function" }
        }
    });
}

#[get("/meaning/?of")]
async fn get_meaning(of: String) -> Result<Option<u32>> {
    Ok(of.contains("life").then(|| 42))
}
```

## Axum Integration

If you have an existing Axum router or you need more control over the server, you can use the [`AetherRouterExt`](https://docs.rs/aether-fullstack/0.6.0-alpha.2/aether_fullstack/prelude/trait.AetherRouterExt.html) trait to integrate with your existing Axum router.

First, make sure your `axum` dependency is optional and enabled by the server feature flag. Axum cannot be compiled to wasm, so if it is enabled by default, it will cause a compile error:

```toml
[dependencies]
aether = { version = "*", features = ["fullstack"] }
axum = { version = "0.8.0", optional = true }
tokio = { version = "1.0", features = ["full"], optional = true }
aether-cli-config = { version = "*", optional = true }

[features]
server = ["aether/server", "dep:axum", "dep:tokio", "aether-cli-config"]
web = ["aether/web"]
```

Then we can set up aether with the axum server:

```rust, no_run
#![allow(non_snake_case)]
use aether::prelude::*;

// The entry point for the server
#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    use aether_server::AetherRouterExt;

    // Get the address the server should run on. If the CLI is running, the CLI proxies fullstack into the main address
    // and we use the generated address the CLI gives us
    let address = aether::cli_config::fullstack_address_or_localhost();

    // Set up the axum router
    let router = axum::Router::new()
        // You can add a aether application to the router with the `serve_aether_application` method
        // This will add a fallback route to the router that will serve your component and server functions
        .serve_aether_application(aether_server::ServeConfig::new(), App);

    // Finally, we can launch the server
    let router = router.into_make_service();
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

// For any other platform, we just launch the app
#[cfg(not(feature = "server"))]
fn main() {
    aether::launch(App);
}

#[component]
fn App() -> Element {
    let mut meaning = use_signal(|| None);

    rsx! {
        h1 { "Meaning of life: {meaning:?}" }
        button {
            onclick: move |_| async move {
                if let Ok(data) = get_meaning("life the universe and everything".into()).await {
                    meaning.set(data);
                }
            },
            "Run a server function"
        }
    }
}

#[server]
async fn get_meaning(of: String) -> ServerFnResult<Option<u32>> {
    Ok(of.contains("life").then(|| 42))
}
```

## Getting Started

To get started with full stack Aether, check out our [getting started guide](https://aetherlabs.com/learn/0.7/getting_started), or the [full stack examples](https://github.com/AetherLabs/aether/tree/master/examples).

## Contributing

- Report issues on our [issue tracker](https://github.com/aetherlabs/aether/issues).
- Join the discord and ask questions!

## License

This project is licensed under the [MIT license].

[mit license]: https://github.com/aetherlabs/aether/blob/main/LICENSE-MIT

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Aether by you shall be licensed as MIT without any additional
terms or conditions.
