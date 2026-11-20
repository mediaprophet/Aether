// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
#[cfg(test)]
mod tests {
    use aether_document::{Document, EvalError, NoOpDocument};
    use futures_util::FutureExt;
    #[test]
    fn noop_eval_stays_unsupported_after_return_and_copy() {
        let mut eval = NoOpDocument.eval("return 1;".to_owned());
        let copy = eval;
        assert!(matches!(eval.send("hello"), Err(EvalError::Unsupported)));
        assert!(matches!(
            eval.recv::<serde_json::Value>().now_or_never(),
            Some(Err(EvalError::Unsupported))
        ));
        assert!(matches!(
            eval.join::<serde_json::Value>().now_or_never(),
            Some(Err(EvalError::Unsupported))
        ));
        assert!(matches!(
            copy.into_future().now_or_never(),
            Some(Err(EvalError::Unsupported))
        ));
    }
}

#[test]
fn document_context_alias_retrieves_the_provided_instance() {
    fn root() -> aether_core::Element {
        let provided: aether_document::DocumentContext =
            std::rc::Rc::new(aether_document::NoOpDocument);
        aether_core::provide_context(provided.clone());
        assert!(std::rc::Rc::ptr_eq(&provided, &aether_document::document()));
        aether_core::VNode::empty()
    }
    aether_core::VirtualDom::new(root).rebuild_in_place();
}

#[test]
fn a_real_evaluator_retains_finished_owner_semantics() {
    use aether_document::{Eval, EvalError, Evaluator};
    use futures_util::FutureExt;
    use std::task::{Context, Poll};
    struct Running;
    impl Evaluator for Running {
        fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            Poll::Ready(Ok(serde_json::json!(42)))
        }
        fn poll_recv(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            Poll::Ready(Ok(serde_json::json!(7)))
        }
        fn send(&self, _: serde_json::Value) -> Result<(), EvalError> {
            Ok(())
        }
    }
    let owner = generational_box::Owner::default();
    let mut eval = Eval::new(owner.insert(Box::new(Running)));
    assert_eq!(eval.recv::<i64>().now_or_never().unwrap().unwrap(), 7);
    assert_eq!(eval.join::<i64>().now_or_never().unwrap().unwrap(), 42);
    drop(owner);
    assert!(matches!(eval.send(1), Err(EvalError::Finished)));
    assert!(matches!(
        eval.recv::<i64>().now_or_never(),
        Some(Err(EvalError::Finished))
    ));
}
