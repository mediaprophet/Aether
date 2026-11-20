//! Native specific utilities for resolving assets in a bundle. This module is intended for use in renderers that
//! need to resolve asset bundles for resources like images, and fonts.

use http::{Response, status::StatusCode};
use std::path::{Path, PathBuf};

use crate::{AssetPathError, NativeAssetResolveError};

#[path = "asset_policy.rs"]
mod asset_policy;

/// Default per-request packaged/trusted asset response limit (64 MiB).
pub const DEFAULT_ASSET_MAX_BYTES: usize = asset_policy::DEFAULT_MAX_BYTES;

/// An error that can occur when serving an asset.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum AssetServeError {
    /// The asset path could not be resolved.
    #[error("Failed to resolve asset: {0}")]
    ResolveError(#[from] NativeAssetResolveError),

    /// An error occurred while constructing the HTTP response.
    #[error("Failed to construct response: {0}")]
    ResponseError(#[from] http::Error),
}

/// Try to resolve the path of an asset from a given URI path.
pub(crate) fn resolve_native_asset_path(path: &str) -> Result<PathBuf, AssetPathError> {
    #[allow(clippy::unnecessary_lazy_evaluations)]
    resolve_trusted_asset_path(path).ok_or_else(|| {
        #[cfg(target_os = "android")]
        {
            // If the asset exists in the Android asset manager, return the can't be represented as a path
            // error instead
            if to_java_load_asset(path).is_some() {
                return AssetPathError::CannotRepresentAsPath;
            }
        }

        AssetPathError::NotFound
    })
}

/// Explicit compatibility path for trusted application-owned URLs, never web input.
fn resolve_trusted_asset_path(path: &str) -> Option<PathBuf> {
    asset_policy::resolve_trusted(get_asset_root().as_deref(), path)
}

struct ResolvedAsset {
    mime_type: &'static str,
    body: Vec<u8>,
}

impl ResolvedAsset {
    fn new(mime_type: &'static str, body: Vec<u8>) -> Self {
        Self { mime_type, body }
    }

    fn into_response(self) -> Result<Response<Vec<u8>>, AssetServeError> {
        Ok(Response::builder()
            .header("Content-Type", self.mime_type)
            .header("X-Content-Type-Options", "nosniff")
            .body(self.body)?)
    }
}

/// Read the bytes for an asset
pub(crate) fn resolve_native_asset(path: &str) -> Result<Vec<u8>, NativeAssetResolveError> {
    // Attempt to serve from the asset dir on android using its loader
    #[cfg(target_os = "android")]
    {
        if let Some(asset) = to_java_load_asset(path) {
            return Ok(asset);
        }
    }

    let Some(uri_path) = resolve_trusted_asset_path(path) else {
        return Err(NativeAssetResolveError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Asset not found",
        )));
    };
    Ok(std::fs::read(uri_path)?)
}

/// Resolve the asset and its mime type
fn resolve_asset(
    path: &str,
    max_bytes: usize,
) -> Result<Option<ResolvedAsset>, NativeAssetResolveError> {
    // Attempt to serve from the asset dir on android using its loader
    #[cfg(target_os = "android")]
    {
        if let Some(asset) = to_java_load_asset_bounded(path, max_bytes)? {
            let extension = path.rsplit_once('.').and_then(|(_, ext)| Some(ext));
            let mime_type = get_mime_from_ext(extension);
            return Ok(Some(ResolvedAsset::new(mime_type, asset)));
        }
    }

    let Some(root) = get_asset_root() else {
        return Ok(None);
    };
    let Some((uri_path, body)) = asset_policy::read_packaged_with_limit(&root, path, max_bytes)?
    else {
        return Ok(None);
    };
    let mime_type = get_mime_from_bytes(&uri_path, &body);
    Ok(Some(ResolvedAsset::new(mime_type, body)))
}

/// Serve a packaged asset contained by the application asset root.
///
/// The root must be trusted and stable during resolution and opening. Windows
/// validates the opened file handle before reading; Unix uses directory-relative
/// opens that refuse symlink replacements. This API does not authorize origins.
/// Use [`serve_trusted_asset`] only for explicit trusted-app compatibility.
///
/// This method properly accesses the asset directory based on the platform and serves the asset
/// wrapped in an HTTP response.
///
/// Platform specifics:
/// - On the web, this returns AssetServerError since there's no filesystem access. Use `fetch` instead.
/// - On Android, it attempts to load assets using the Android AssetManager.
/// - On other platforms, it serves assets from the filesystem.
pub fn serve_asset(path: &str) -> Result<Response<Vec<u8>>, AssetServeError> {
    serve_asset_with_limit(path, asset_policy::DEFAULT_MAX_BYTES)
}

/// Serve a packaged asset with a per-request byte limit. The default in
/// [`serve_asset`] is 64 MiB. Oversized assets fail before returning partial data.
/// This bounds response bytes, not decoder allocations or aggregate request load.
pub fn serve_asset_with_limit(
    path: &str,
    max_bytes: usize,
) -> Result<Response<Vec<u8>>, AssetServeError> {
    match resolve_asset(path, max_bytes)? {
        Some(asset) => asset.into_response(),
        None => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(String::from("Not Found").into_bytes())?),
    }
}

/// Serve a trusted application's local file URL, including absolute filesystem paths.
///
/// This is intentionally more privileged than [`serve_asset`]. Never connect it
/// directly to remote pages or an untrusted local-service request. No wildcard CORS
/// permission is supplied; the host remains responsible for caller authorization.
pub fn serve_trusted_asset(path: &str) -> Result<Response<Vec<u8>>, AssetServeError> {
    serve_trusted_asset_with_limit(path, asset_policy::DEFAULT_MAX_BYTES)
}

/// The explicit trusted-file compatibility API with a per-request byte limit.
/// The same authority requirements as [`serve_trusted_asset`] apply.
pub fn serve_trusted_asset_with_limit(
    path: &str,
    max_bytes: usize,
) -> Result<Response<Vec<u8>>, AssetServeError> {
    let Some(path) = resolve_trusted_asset_path(path) else {
        return serve_asset_with_limit(path, max_bytes);
    };
    let body = std::fs::File::open(&path)
        .and_then(|file| asset_policy::read_bounded(file, max_bytes))
        .map_err(NativeAssetResolveError::from)?;
    let mime_type = get_mime_from_bytes(&path, &body);
    ResolvedAsset::new(mime_type, body).into_response()
}

/// Get the asset directory, following tauri/cargo-bundles directory discovery approach
///
/// Currently supports:
/// - [x] macOS
/// - [x] iOS
/// - [x] Windows
/// - [x] Linux (appimage)
/// - [x] Linux (rpm)
/// - [x] Linux (deb)
/// - [x] Android (returns current exe.)
#[allow(unreachable_code)]
fn get_asset_root() -> Option<PathBuf> {
    let cur_exe = std::env::current_exe().ok()?;

    // In macOS bundles, the assets are in the `Resources` directory
    if cfg!(target_os = "macos") {
        return Some(cur_exe.parent()?.parent()?.join("Resources"));
    }

    // In linux bundles, the assets are placed in the lib/$product_name directory
    // bin/
    //   main
    // lib/
    //   $product_name/
    //     assets/
    if cfg!(target_os = "linux") {
        if let Some(root) =
            linux_bundle_root(&cur_exe, aether_cli_config::product_name().as_deref())
        {
            return Some(root);
        }
    }

    // For all others, the structure looks like this:
    // app.(exe/appimage)
    //   main.exe
    //   assets/
    Some(cur_exe.parent()?.to_path_buf())
}

fn linux_bundle_root(executable: &Path, product_name: Option<&str>) -> Option<PathBuf> {
    let name = product_name.or_else(|| executable.file_stem()?.to_str())?;
    let mut parts = Path::new(name).components();
    if !matches!(parts.next(), Some(std::path::Component::Normal(_))) || parts.next().is_some() {
        return None;
    }
    let candidate = executable.parent()?.parent()?.join("lib").join(name);
    // Never scan unrelated applications' directories to infer local authority.
    candidate.is_dir().then_some(candidate)
}

#[cfg(test)]
mod root_tests {
    use super::*;

    #[test]
    fn linux_bundle_discovery_never_selects_an_unrelated_application() {
        let root = std::env::temp_dir().join(format!(
            "aether-bundle-root-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir_all(root.join("lib/unrelated/assets")).unwrap();
        let executable = root.join("bin/aether-demo");
        assert!(linux_bundle_root(&executable, None).is_none());
        std::fs::create_dir_all(root.join("lib/aether-demo/assets")).unwrap();
        assert_eq!(
            linux_bundle_root(&executable, None),
            Some(root.join("lib/aether-demo"))
        );
        assert_eq!(
            linux_bundle_root(&executable, Some("unrelated")),
            Some(root.join("lib/unrelated"))
        );
        assert!(linux_bundle_root(&executable, Some("../unrelated")).is_none());
        assert!(linux_bundle_root(&executable, Some("")).is_none());
    }
}

// Infer from exactly the bytes being returned; do not reopen a checked path.
fn get_mime_from_bytes(path: &Path, body: &[u8]) -> &'static str {
    if path.extension().is_some_and(|ext| ext == "svg") {
        return "image/svg+xml";
    }
    match infer::get(body).map(|f| f.mime_type()) {
        Some(mime) if mime != "text/plain" => mime,
        _ => get_mime_by_ext(path),
    }
}

/// Get the mime type from a URI using its extension
fn get_mime_by_ext(trimmed: &Path) -> &'static str {
    let ext = trimmed.extension().as_ref().and_then(|ext| ext.to_str());
    get_mime_from_ext(ext)
}

/// Get the mime type from a URI using its extension
pub fn get_mime_from_ext(ext: Option<&str>) -> &'static str {
    match ext {
        // The common assets are all utf-8 encoded
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",

        // the rest... idk? probably not
        Some("mjs") => "text/javascript; charset=utf-8",
        Some("bin") => "application/octet-stream",
        Some("csv") => "text/csv",
        Some("ico") => "image/vnd.microsoft.icon",
        Some("jsonld") => "application/ld+json",
        Some("rtf") => "application/rtf",
        Some("mp4") => "video/mp4",
        // Unknown extensions are downloads/data, not implicitly executable HTML.
        Some(_) => "application/octet-stream",
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
        // using octet stream according to this:
        None => "application/octet-stream",
    }
}

#[cfg(target_os = "android")]
pub(crate) fn to_java_load_asset(filepath: &str) -> Option<Vec<u8>> {
    to_java_load_asset_bounded(filepath, asset_policy::DEFAULT_MAX_BYTES)
        .ok()
        .flatten()
}

#[cfg(target_os = "android")]
fn to_java_load_asset_bounded(
    filepath: &str,
    max_bytes: usize,
) -> std::io::Result<Option<Vec<u8>>> {
    // The Option closure represents absent/unavailable assets; explicit read and
    // size failures remain Result errors instead of becoming an HTTP 404.
    let load = || -> Option<std::io::Result<Vec<u8>>> {
        let relative = asset_policy::relative_path(filepath)?;
        let normalized = relative
            .to_str()?
            .strip_prefix("assets/")
            .unwrap_or(relative.to_str()?);

        // in debug mode, the asset might be under `/data/local/tmp/dx/` - attempt to read it from there if it exists
        #[cfg(debug_assertions)]
        {
            match asset_policy::read_packaged_with_limit(
                &aether_cli_config::android_session_cache_dir(),
                normalized,
                max_bytes,
            ) {
                Ok(Some((_, body))) => return Some(Ok(body)),
                Err(error) => return Some(Err(error)),
                Ok(None) => {}
            }
        }

        use std::ptr::NonNull;

        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.ok()?;
        let mut env = vm.attach_current_thread().ok()?;

        // Query the Asset Manager
        let asset_manager_ptr = env
            .call_method(
                unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) },
                "getAssets",
                "()Landroid/content/res/AssetManager;",
                &[],
            )
            .ok()?
            .l()
            .ok()?;

        unsafe {
            let asset_manager =
                ndk_sys::AAssetManager_fromJava(env.get_native_interface(), *asset_manager_ptr);

            let asset_manager = ndk::asset::AssetManager::from_ptr(NonNull::new(asset_manager)?);

            let cstr = std::ffi::CString::new(normalized).ok()?;

            let mut asset = asset_manager.open(&cstr)?;
            let buffer = asset.buffer().ok()?;
            if buffer.len() > max_bytes {
                return Some(Err(asset_policy::size_error()));
            }
            Some(Ok(buffer.to_vec()))
        }
    };
    load().transpose()
}
