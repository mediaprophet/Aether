#![allow(non_snake_case)]
use aether::prelude::*;
use nested_suspense::app;

fn main() {
    aether::logger::init(aether::logger::tracing::Level::TRACE).expect("logger failed to init");

    aether::LaunchBuilder::new()
        .with_cfg(server_only! {
            ServeConfig::builder()
                .incremental(
                    aether::server::IncrementalRendererConfig::new()
                        .static_dir(
                            std::env::current_exe()
                                .unwrap()
                                .parent()
                                .unwrap()
                                .join("public")
                        )
                        .clear_cache(false)
                )
                .enable_out_of_order_streaming()
        })
        .launch(app);
}

#[server(endpoint = "static_routes")]
async fn static_routes() -> ServerFnResult<Vec<String>> {
    Ok(vec!["/".to_string()])
}
