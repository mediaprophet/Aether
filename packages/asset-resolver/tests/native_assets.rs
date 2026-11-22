// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
#![cfg(feature = "native")]

use aether_asset_resolver::{
    asset_path,
    native::{serve_asset, serve_trusted_asset, serve_trusted_asset_with_limit},
};
#[test]
fn trusted_application_sources_remain_explicit() {
    let root = std::env::temp_dir().join(format!(
        "aether-asset-api-{}",
        std::process::id().to_string()
            + "-"
            + &std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string()
    ));
    std::fs::create_dir(&root).unwrap();
    let file = root.join("source.txt");
    std::fs::write(&file, b"application asset").unwrap();
    let url = file.to_string_lossy().replace('\\', "/");
    #[cfg(windows)]
    let url = format!("/{url}");
    assert_eq!(asset_path(&url).unwrap(), file.canonicalize().unwrap());
    let denied = serve_asset(&url).unwrap();
    assert_eq!(denied.status().as_u16(), 404);
    let allowed = serve_trusted_asset(&url).unwrap();
    assert_eq!(allowed.status().as_u16(), 200);
    assert_eq!(allowed.body(), b"application asset");
    assert!(serve_trusted_asset_with_limit(&url, 16).is_err());
    assert_eq!(
        serve_trusted_asset_with_limit(&url, 17).unwrap().body(),
        b"application asset"
    );
    assert!(
        !allowed
            .headers()
            .contains_key("access-control-allow-origin")
    );
    assert_eq!(allowed.headers()["x-content-type-options"], "nosniff");
    assert_eq!(
        aether_asset_resolver::native::get_mime_from_ext(Some("unknown")),
        "application/octet-stream"
    );
}
#[test]
fn malformed_asset_url_is_not_a_panic() {
    for path in ["%ff", "%", "/../secret", "/C:/secret"] {
        assert_eq!(serve_asset(path).unwrap().status().as_u16(), 404);
    }
}
