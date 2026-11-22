//! Public launch-error contracts; these tests do not create windows or a GPU.

use aether_native::{LaunchError, winit::error::EventLoopError};
use std::error::Error;

#[test]
fn runtime_failure_preserves_io_error_details() {
    let error = LaunchError::Runtime(std::io::Error::from_raw_os_error(5));
    let source = error
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .unwrap();
    assert_eq!(source.raw_os_error(), Some(5));
    assert!(error.to_string().contains(&source.to_string()));
}

#[test]
fn event_loop_failures_preserve_stage_and_winit_error() {
    let creation = LaunchError::EventLoopCreation(EventLoopError::RecreationAttempt);
    assert!(matches!(
        creation.source().unwrap().downcast_ref::<EventLoopError>(),
        Some(EventLoopError::RecreationAttempt)
    ));
    let execution = LaunchError::EventLoopRun(EventLoopError::ExitFailure(17));
    assert!(matches!(
        execution.source().unwrap().downcast_ref::<EventLoopError>(),
        Some(EventLoopError::ExitFailure(17))
    ));
    assert_ne!(creation.to_string(), execution.to_string());
}

#[test]
fn legacy_and_fallible_entry_points_have_distinct_return_contracts() {
    let _: fn(fn() -> aether_core::Element) = aether_native::launch;
    let _: fn(fn() -> aether_core::Element) -> Result<(), LaunchError> = aether_native::try_launch;
    // Type-check all configuration paths without starting a platform event loop.
    fn configured(app: fn() -> aether_core::Element) -> Result<(), LaunchError> {
        aether_native::try_launch_cfg(app, vec![], vec![])?;
        aether_native::try_launch_cfg_with_props(app, (), vec![], vec![])
    }
    let _ = configured;
}
