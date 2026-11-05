//! This example demonstrates how to use `aether::serve` with a custom Axum router.
//!
//! By default, `aether::launch` takes over the main thread and runs the Aether application.
//! However, if you want to integrate Aether into an existing web server or use a custom router,
//! you can use `aether::serve` to create a server that serves your Aether application alongside
//! other routes.
//!
//! `aether::serve` sets up an async runtime, logging, hot-reloading, crash handling, and more.
//! You can then use the `.serve_aether_application` method on your router to serve the Aether app.
//!
//! `aether::serve` is most useful for customizing the server setup, such as adding middleware,
//! custom routes, or integrating with existing axum backend code.
//!
//! Note that `aether::serve` is accepts a Router from `axum`. Aether will use the IP and PORT
//! environment variables to determine where to bind the server. To customize the port, use environment
//! variables or a `.env` file.
//!
//! On other platforms (like desktop or mobile), you'll want to use `aether::launch` instead and then
//! handle async loading of data through hooks like `use_future` or `use_resource` and give the user
//! a loading state while data is being fetched.

use aether::prelude::*;

fn main() {
    // On the client we just launch the app as normal.
    #[cfg(not(feature = "server"))]
    aether::launch(app);

    // On the server, we can use `aether::serve` and `.serve_aether_application` to serve our app with routing.
    // The `aether::server::router` function creates a new axum Router with the necessary routes to serve the Aether app.
    #[cfg(feature = "server")]
    aether::serve(|| async move {
        use aether::server::axum::routing::{get, post};

        Ok(aether::server::router(app)
            .route("/submit", post(|| async { "Form submitted!" }))
            .route("/about", get(|| async { "About us" }))
            .route("/contact", get(|| async { "Contact us" })))
    });
}

fn app() -> Element {
    rsx! {
        div { "Hello from Aether!" }
    }
}
