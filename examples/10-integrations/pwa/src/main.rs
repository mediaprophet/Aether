use aether::prelude::*;

fn main() {
    aether::launch(app);
}

fn app() -> Element {
    rsx! (
        div { style: "text-align: center;",
            h1 { "🌗 Aether 🚀" }
            h3 { "Frontend that scales." }
            p { "Build web, desktop, and mobile apps with Aether" }
        }
    )
}
