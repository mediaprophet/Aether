mod backend;
mod frontend;

use aether::prelude::*;
use frontend::*;

#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[layout(NavBar)]
    #[route("/")]
    DogView,

    #[route("/favorites")]
    Favorites,
}

fn main() {
    // only in production should we set the URL, otherwise let `dx` do the work
    #[cfg(all(not(feature = "server"), feature = "production"))]
    aether::fullstack::set_server_url("https://hot-dog.fly.dev");

    aether::launch(app);
}

fn app() -> Element {
    rsx! {
        Stylesheet { href: asset!("/assets/main.css") }
        Router::<Route> {}
    }
}
