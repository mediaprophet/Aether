# aether-core-macro

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![Discord chat][discord-badge]][discord-url]

[crates-badge]: https://img.shields.io/crates/v/aether-core-macro.svg
[crates-url]: https://crates.io/crates/aether-core-macro
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/aetherlabs/aether/blob/main/LICENSE-MIT
[actions-badge]: https://github.com/aetherlabs/aether/actions/workflows/main.yml/badge.svg
[actions-url]: https://github.com/aetherlabs/aether/actions?query=workflow%3ACI+branch%3Amaster
[discord-badge]: https://img.shields.io/discord/899851952891002890.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/XgGxMSkvUM

[Website](https://aetherlabs.com) |
[Guides](https://aetherlabs.com/learn/0.7/) |
[API Docs](https://docs.rs/aether-core-macro/latest/aether_core_macro) |
[Chat](https://discord.gg/XgGxMSkvUM)

## Overview

`aether-core-macro` provides a handful of helpful macros used by the `aether` crate. These include:

- The `rsx!` macro that underpins templates and node creation.
- The `component` attribute macro denotes a function as a Aether component. Currently, this:
  - Transforms function arguments into an auto-derived struct.
  - Ensures that your component name uses PascalCase.
  - Probably more stuff in the future. This macro allows us to have a way of distinguishing functions and components, which can be quite handy.
- The `format_args_f` macro which allows f-string formatting with support for expressions.

## Contributing

- Report issues on our [issue tracker](https://github.com/aetherlabs/aether/issues).
- Join the discord and ask questions!

## License

This project is licensed under the [MIT license].

[mit license]: https://github.com/aetherlabs/aether/blob/main/LICENSE-MIT

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Aether by you shall be licensed as MIT without any additional
terms or conditions.
