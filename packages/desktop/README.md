# Aether Desktop (webview)

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![Discord chat][discord-badge]][discord-url]

[crates-badge]: https://img.shields.io/crates/v/aether-desktop.svg
[crates-url]: https://crates.io/crates/aether-desktop
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/aetherlabs/aether/blob/main/LICENSE-MIT
[actions-badge]: https://github.com/aetherlabs/aether/actions/workflows/main.yml/badge.svg
[actions-url]: https://github.com/aetherlabs/aether/actions?query=workflow%3ACI+branch%3Amaster
[discord-badge]: https://img.shields.io/discord/899851952891002890.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/XgGxMSkvUM

[Website](https://aetherlabs.com) |
[Guides](https://aetherlabs.com/learn/0.7/) |
[API Docs](https://docs.rs/aether-desktop/latest/aether_desktop) |
[Chat](https://discord.gg/XgGxMSkvUM)

## Overview

`aether-desktop` provides a webview-based desktop renderer for the Aether VirtualDom.

This requires that webview is installed on the target system. WebView is installed by default on macOS and iOS devices, but might not come preinstalled on Windows or Linux devices. To fix these issues, follow the [instructions in the guide](guide-url).

[guide-url]: https://aetherlabs.com/learn/0.7/getting_started

## Features

- Simple, one-line launch for desktop apps
- Aether VirtualDom running on a native thread
- Full HTML/CSS support via `wry` and `tao`
- Exposed `window` and `Proxy` types from tao for direct window manipulation
- Helpful hooks for accessing the window, WebView, and running javascript.

## Contributing

- Report issues on our [issue tracker](https://github.com/aetherlabs/aether/issues).
- Join the discord and ask questions!

## License

This project is licensed under the [MIT license].

[mit license]: https://github.com/aetherlabs/aether/blob/main/LICENSE-MIT

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Aether by you shall be licensed as MIT without any additional
terms or conditions.
