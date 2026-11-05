<div align="center">
  <h1>📡 Aether Logger 🛰️</h1>
  <p><strong>A logging utility to provide a standard interface whether you're targeting web, desktop, fullstack, and more.</strong></p>
</div>

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/aether-logger">
    <img src="https://img.shields.io/crates/v/aether-logger.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/aether-logger">
    <img src="https://img.shields.io/crates/d/aether-logger.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs -->
  <a href="https://docs.rs/aether-logger">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

-----

`aether-logger` is a basic cross-platform facade for logging in [Aether](https://aetherlabs.com/) that uses the [tracing](https://crates.io/crates/tracing) crate.


```rust
use aether::prelude::*;
use aether_logger::tracing::{Level, info};

fn main() {
  aether_logger::init(Level::INFO).expect("logger failed to init");
  aether::launch(App);
}

#[component]
fn App() -> Element {
  info!("App rendered");
  rsx! {
    p { "hi" }
  }
}
```

## Aether support

As of v0.6, aether_logger is part of aether itself. Aether will call `init` with a default Level, though you can still override the default with `init`.

```rust
use aether::prelude::*;
use aether::logger::tracing::{Level, info};

fn main() {
  aether::logger::init(Level::INFO).expect("logger failed to init");
  aether::launch(App);
}

#[component]
fn App() -> Element {
  info!("App rendered");
  rsx! {
    p { "hi" }
  }
}
```
For non-wasm targets, runtime filtering is based on the `RUST_LOG` environment variable. e.g. for `RUST_LOG=none,crateName=trace` only logs trace and above for `crateName` will be captured. See [here](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives) for syntax. For crates with `-` in the name, these need to be changed to `_` in `RUST_LOG`.

## Platform Support
Aether logger will eventually support every target that Aether does. Currently mobile and TUI are not supported.

## Installation
`aether_logger` is part of Aether v0.6. If you're using Aether v0.6, then no installation is required!


If you're on Aether v0.5 and below, you can add `aether-logger` to your application by adding it to your dependencies.
```toml
[dependencies]
aether-logger = "0.5"
```


## License
This project is licensed under the [MIT license].

[mit license]: ./LICENSE

Every contribution intentionally submitted for inclusion in `aether-logger` by you, shall be licensed as MIT, without any additional terms or conditions.
