use std::rc::Rc;

use aether::prelude::*;
use aether_history::{History, MemoryHistory};
use aether_router::components::HistoryProvider;

// Tests for regressions of <https://github.com/AetherLabs/aether/issues/2468>
#[test]
fn router_without_index_route_parses() {
    let mut vdom = VirtualDom::new_with_props(
        App,
        AppProps {
            path: Route::Test {},
        },
    );
    vdom.rebuild_in_place();
    let as_string = aether_ssr::render(&vdom);
    assert_eq!(as_string, "<div>router with no index route renders</div>")
}

#[derive(Routable, Clone, Copy, PartialEq, Debug)]
enum Route {
    #[route("/test")]
    Test {},
}

#[component]
fn Test() -> Element {
    rsx! {
        div {
            "router with no index route renders"
        }
    }
}

#[component]
fn App(path: Route) -> Element {
    rsx! {
        HistoryProvider {
            history:  move |_| Rc::new(MemoryHistory::with_initial_path(path)) as Rc<dyn History>,
            Router::<Route> {}
        }
    }
}
