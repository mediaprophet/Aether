# Aether Hooks

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![Discord chat][discord-badge]][discord-url]

[crates-badge]: https://img.shields.io/crates/v/aether-hooks.svg
[crates-url]: https://crates.io/crates/aether-hooks
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/aetherlabs/aether/blob/main/LICENSE-MIT
[actions-badge]: https://github.com/aetherlabs/aether/actions/workflows/main.yml/badge.svg
[actions-url]: https://github.com/aetherlabs/aether/actions?query=workflow%3ACI+branch%3Amaster
[discord-badge]: https://img.shields.io/discord/899851952891002890.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/XgGxMSkvUM

[Website](https://aetherlabs.com) |
[Guides](https://aetherlabs.com/learn/0.7/) |
[API Docs](https://docs.rs/aether-hooks/latest/aether_hooks) |
[Chat](https://discord.gg/XgGxMSkvUM)

## Overview

`aether-hooks` includes some basic useful hooks for Aether such as:

- use_signal
- use_effect
- use_resource
- use_memo
- use_coroutine

Unlike React, none of these hooks are foundational since they all build off the primitive `use_hook`. You can extend these hooks with [custom hooks](https://aetherlabs.com/learn/0.7/essentials/advanced/custom_hooks) in your own code. If you think they would be useful for the broader community, you can open a PR to add your hook to the [Aether Awesome](https://github.com/AetherLabs/awesome-aether) list.

## State Cheat Sheet

If you aren't sure what hook to use, you can use this cheat sheet to help you decide:

### State Location

Depending on where you need to access the state, you can put your state in one of three places:

| Location                                                                                 | Where can you access the state? | Recommended for Libraries? | Examples                                                                    |
| ---------------------------------------------------------------------------------------- | ------------------------------- | -------------------------- | --------------------------------------------------------------------------- |
| [Hooks](https://docs.rs/aether-hooks/latest/aether_hooks/fn.use_signal.html)             | Any components you pass it to   | ✅                          | `use_signal(\|\| 0)`, `use_memo(\|\| state() * 2)`                          |
| [Context](https://docs.rs/aether-hooks/latest/aether_hooks/fn.use_context_provider.html) | Any child components            | ✅                          | `use_context_provider(\|\| Signal::new(0))`, `use_context::<Signal<i32>>()` |
| [Global](https://docs.rs/aether/latest/aether/prelude/struct.Signal.html#method.global)  | Anything in your app            | ❌                          | `Signal::global(\|\| 0)`                                                    |

### Derived State

If you don't have an initial value for your state, you can derive your state from other states with a closure or asynchronous function:

| Hook                                                                                | Reactive (reruns when dependencies change) | Async | Memorizes Output | Example                                                                             |
| ----------------------------------------------------------------------------------- | ------------------------------------------ | ----- | ---------------- | ----------------------------------------------------------------------------------- |
| [`use_memo`](https://docs.rs/aether/latest/aether/prelude/fn.use_memo.html)         | ✅                                          | ❌     | ✅                | `use_memo(move \|\| count() * 2)`                                                   |
| [`use_resource`](https://docs.rs/aether/latest/aether/prelude/fn.use_resource.html) | ✅                                          | ✅     | ❌                | `use_resource(move \|\| reqwest::get(format!("/users/{user_id}")))`                 |
| [`use_future`](https://docs.rs/aether/latest/aether/prelude/fn.use_future.html)     | ❌                                          | ✅     | ❌                | `use_future(move \|\| println!("{:?}", reqwest::get(format!("/users/{user_id}"))))` |

### Persistent State

The core hooks library doesn't provide hooks for persistent state, but you can extend the core hooks with hooks from [aether-sdk](https://crates.io/crates/aether-sdk) and the [aether-router](https://crates.io/crates/aether-router) to provide persistent state management.

| State                                                                              | Sharable | Example                                                                                           |
| ---------------------------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------- |
| [`use_persistent`](https://github.com/AetherLabs/sdk/tree/master/examples/storage) | ❌        | `use_persistent("unique_key", move \|\| initial_state)`                                           |
| [`Router<Route> {}`](https://aetherlabs.com/learn/0.7/essentials/router/)          | ✅        | `#[derive(Routable, Clone, PartialEq)] enum Route { #[route("/user/:id")] Homepage { id: u32 } }` |

## Contributing

- Report issues on our [issue tracker](https://github.com/aetherlabs/aether/issues).
- Join the discord and ask questions!

## License

This project is licensed under the [MIT license].

[mit license]: https://github.com/aetherlabs/aether/blob/main/LICENSE-MIT

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Aether by you shall be licensed as MIT without any additional
terms or conditions.
