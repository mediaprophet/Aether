use aether::LaunchBuilder;
use aether::prelude::*;

fn main() {
    LaunchBuilder::new()
        .with_cfg(
            aether::desktop::Config::new()
                .with_windows_browser_args("--remote-debugging-port=8787"),
        )
        .launch(app);
}

fn app() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        h1 { "High-five counter: {count}" }
        button {
            id: "increment-button",
            onclick: move |_| count += 1, "Up high!"
        }
    }
}
