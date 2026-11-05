//! XSS Safety
//!
//! This example proves that Aether is safe from XSS attacks.

use aether::prelude::*;

fn main() {
    aether::launch(app);
}

fn app() -> Element {
    let mut contents = use_signal(|| String::from("<script>alert(\"hello world\")</script>"));

    rsx! {
        div {
            h1 {"Aether is XSS-Safe"}
            h3 { "{contents}" }
            input {
                value: "{contents}",
                r#type: "text",
                oninput: move |e| contents.set(e.value()),
            }
        }
    }
}
