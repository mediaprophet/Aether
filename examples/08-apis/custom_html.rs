//! This example shows how to use a custom index.html and custom <HEAD> extensions
//! to add things like stylesheets, scripts, and third-party JS libraries.

use aether::prelude::*;

fn main() {
    aether::LaunchBuilder::new()
        .with_cfg(
            aether::desktop::Config::new().with_custom_index(
                r#"
<!DOCTYPE html>
<html>
  <head>
    <title>Aether app</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <style>body { background-color: olive; }</style>
  </head>
  <body>
    <h1>External HTML</h1>
    <div id="main"></div>
  </body>
</html>
        "#
                .into(),
            ),
        )
        .launch(app);
}

fn app() -> Element {
    rsx! {
        h1 { "Custom HTML!" }
    }
}
