mod dropped_frame_handling;
mod errors;
mod interleaved_command_buffer;
mod multiple_resolves_per_frame;
mod nested_scopes;

pub fn create_device(
    features: etalon::Features,
) -> Result<(etalon::Backend, etalon::Device, etalon::Queue), etalon::RequestDeviceError> {
    async fn create_default_device_async(
        features: etalon::Features,
    ) -> Result<(etalon::Backend, etalon::Device, etalon::Queue), etalon::RequestDeviceError> {
        let instance =
            etalon::Instance::new(etalon::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&etalon::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&etalon::DeviceDescriptor {
                required_features: features,
                ..Default::default()
            })
            .await?;
        Ok((adapter.get_info().backend, device, queue))
    }

    futures_lite::future::block_on(create_default_device_async(features))
}

#[derive(Debug, Clone, Copy)]
enum Requires {
    Disabled,
    Timestamps,
    TimestampsInEncoders,
    TimestampsInPasses,
}

impl Requires {
    fn expect_time_result(self, features: etalon::Features) -> bool {
        match self {
            Requires::Timestamps => features.contains(etalon::Features::TIMESTAMP_QUERY),
            Requires::TimestampsInEncoders => {
                features.contains(etalon::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
            }
            Requires::TimestampsInPasses => {
                features.contains(etalon::Features::TIMESTAMP_QUERY_INSIDE_PASSES)
            }
            Requires::Disabled => false,
        }
    }
}

#[derive(Debug)]
struct ExpectedScope(String, Requires, Vec<ExpectedScope>);

fn expected_scope(
    label: impl Into<String>,
    requires: Requires,
    children: impl Into<Vec<ExpectedScope>>,
) -> ExpectedScope {
    ExpectedScope(label.into(), requires, children.into())
}

fn validate_results(
    features: etalon::Features,
    results: &[etalon_profiler::GpuTimerQueryResult],
    expected: &[ExpectedScope],
) {
    assert_eq!(
        results.len(),
        expected.len(),
        "results: {results:?}\nexpected: {expected:?}"
    );
    for (result, expected) in results.iter().zip(expected.iter()) {
        assert_eq!(result.label, expected.0);
        assert_eq!(
            result.time.is_some(),
            expected.1.expect_time_result(features),
            "label: {}",
            result.label
        );

        validate_results(features, &result.nested_queries, &expected.2);
    }
}

fn validate_results_unordered(
    features: etalon::Features,
    results: &[etalon_profiler::GpuTimerQueryResult],
    expected: &[ExpectedScope],
) {
    assert_eq!(
        results.len(),
        expected.len(),
        "result: {results:?}\nexpected: {expected:?}"
    );

    let mut expected_by_label =
        std::collections::HashMap::<String, (Requires, &[ExpectedScope])>::from_iter(
            expected
                .iter()
                .map(|expected| (expected.0.clone(), (expected.1, expected.2.as_ref()))),
        );

    for result in results {
        let Some((requires, nested_expectations)) = expected_by_label.remove(&result.label) else {
            panic!("missing result for label: {}", result.label);
        };
        assert_eq!(
            result.time.is_some(),
            requires.expect_time_result(features),
            "label: {}",
            result.label
        );

        validate_results(features, &result.nested_queries, nested_expectations);
    }
}
