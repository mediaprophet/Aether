// Regression test for https://github.com/AetherLabs/aether/pull/3480

use aether::prelude::*;

fn main() {
    aether::launch(App);
}

#[component]
fn App() -> Element {
    let mut mounted = use_signal(|| false);

    rsx! {
        div {
            onmounted: move |_| {
                mounted.set(true);
            },
            if mounted() {
                "The mounted event was triggered."
            }
        }
    }
}
