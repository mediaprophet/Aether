# aether-core

`aether-core` provides a fast and featureful VirtualDom implementation for Rust.

```rust, no_run
# tokio::runtime::Runtime::new().unwrap().block_on(async {
use aether_core::{VirtualDom, Event, Element, Mutations, VNode, ElementId};

let mut vdom = VirtualDom::new(app);
let real_dom = SomeRenderer::new();

loop {
    tokio::select! {
        evt = real_dom.event() => {
            let evt = Event::new(evt, true);
            vdom.runtime().handle_event("onclick", evt, ElementId::from_raw(0))
        },
        _ = vdom.wait_for_work() => {}
    }
    vdom.render_immediate(&mut real_dom.apply())
}

# fn app() -> Element { VNode::empty() }
# struct SomeRenderer; impl SomeRenderer { fn new() -> SomeRenderer { SomeRenderer } async fn event(&self) -> std::rc::Rc<dyn std::any::Any> { unimplemented!() } fn apply(&self) -> Mutations { Mutations::default() } }
# });
```

## Features

A virtualdom is an efficient and flexible tree data structure that allows you to manage state for a graphical user interface. The Aether VirtualDom is perhaps the most fully-featured virtualdom implementation in Rust and powers renderers running across Web, Desktop, Mobile, SSR, TUI, LiveView, and more. When you use the Aether VirtualDom, you immediately enable users of your renderer to leverage the wide ecosystem of Aether components, hooks, and associated tooling.

Some features of `aether-core` include:

- UI components are just functions
- State is provided by hooks
- Deep integration with async
- Strong focus on performance
- Integrated hotreloading support
- Extensible system for UI elements and their attributes

If you are just starting, check out the Guides first.

## Understanding the implementation

`aether-core` is designed to be a lightweight crate that. It exposes a number of flexible primitives without being deeply concerned about the intracices of state management itself. We provide a number of useful abstractions built on these primitives in the `aether-hooks` crate as well as the `aether-signals` crate.

The important abstractions to understand are:

- The [`VirtualDom`]
- The [`Component`] and its [`Properties`]
- Handling events
- Working with async
- Suspense

## Usage

The `aether` crate exports the `rsx` macro which transforms a helpful, simpler syntax of Rust.

First, start with your app:

```rust
# use aether::aether_core::{Mutations, VirtualDom};
use aether::prelude::*;

// First, declare a root component
fn app() -> Element {
    rsx! {
        div { "hello world" }
    }
}

fn main() {
    // Next, create a new VirtualDom using this app as the root component.
    let mut dom = VirtualDom::new(app);

    // The initial render of the dom will generate a stream of edits for the real dom to apply
    let mutations = dom.rebuild_to_vec();
}
```

## Contributing

- Check out the website [section on contributing](https://aetherlabs.com/learn/0.7/beyond/contributing).
- Report issues on our [issue tracker](https://github.com/aetherlabs/aether/issues).
- [Join](https://discord.gg/XgGxMSkvUM) the discord and ask questions!

<a href="https://github.com/aetherlabs/aether/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=aetherlabs/aether&max=30&columns=10" />
</a>

## License

This project is licensed under the [MIT license].

[mit license]: https://github.com/AetherLabs/aether/blob/master/LICENSE-MIT

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Aether by you, shall be licensed as MIT, without any additional
terms or conditions.
