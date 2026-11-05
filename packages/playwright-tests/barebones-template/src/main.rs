use aether::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    aether::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Hero {}

    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://aetherlabs.com/learn/0.7/", "📚 Learn Aether" }
                a { href: "https://aetherlabs.com/awesome", "🚀 Awesome Aether" }
                a { href: "https://github.com/aether-community/", "📡 Community Libraries" }
                a { href: "https://github.com/AetherLabs/sdk", "⚙️ Aether Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=AetherLabs.aether", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}
