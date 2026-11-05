//! Regression test for <https://github.com/AetherLabs/aether/issues/4646>

use aether::prelude::*;

fn main() {
    // we split these two to ensure `aether::serve` works properly.
    #[cfg(feature = "server")]
    aether::serve(|| async move { Ok(aether::server::router(app)) });

    #[cfg(not(feature = "server"))]
    aether::launch(app);
}

fn app() -> Element {
    rsx! {
        Comp {}
        Comp {}
        Button {}
    }
}

#[component]
fn Button() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        button {
            id: "counter",
            onclick: move |_| {
                count += 1;
            },
            "Count: {count}"
        }
    }
}

#[component]
fn Comp(#[props(extends = GlobalAttributes)] attributes: Vec<Attribute>) -> Element {
    rsx! {
        div {
            width: 100,
            div {
                ..attributes,
            }
        }
    }
}
