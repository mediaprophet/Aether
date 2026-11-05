# Aether Fullstack Core

Aether-fullstack-core provides types, traits, hooks, contexts for [`aether-fullstack`](https://crates.io/crates/aether-fullstack). Libraries that need to integrate with aether-fullstack should rely on this crate instead of the full-fledged renderer for quicker build times.

## Usage

To start using this crate, you can run the following command:

```bash
cargo add aether-fullstack-hooks
```

Then you can use hooks like `use_server_future` in your components:

```rust
use aether::prelude::*;


fn App() -> Element {
    let mut article_id = use_signal(|| 0);

    // `use_server_future` will spawn a task that runs on the server and serializes the result to send to the client.
    // The future will rerun any time the
    // Since we bubble up the suspense with `?`, the server will wait for the future to resolve before rendering
    let article = use_server_future(move || fetch_article(article_id()))?;

    rsx! {
        "{article().unwrap()}"
    }
}

async fn fetch_article(id: u32) -> String {
    format!("Article {}", id)
}
```
