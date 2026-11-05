use aether::prelude::*;

#[test]
fn static_boolean_attributes() {
    fn app() -> Element {
        rsx! {
            div { hidden: "false" }
            div { hidden: "true" }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild(&mut aether_core::NoOpMutations);

    assert_eq!(
        aether_ssr::render(&dom),
        r#"<div></div><div hidden="true"></div>"#
    );
}

#[test]
fn dynamic_boolean_attributes() {
    fn app() -> Element {
        rsx! {
            div { hidden: false }
            div { hidden: true }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild(&mut aether_core::NoOpMutations);

    assert_eq!(
        aether_ssr::render(&dom),
        r#"<div></div><div hidden=true></div>"#
    );
}
