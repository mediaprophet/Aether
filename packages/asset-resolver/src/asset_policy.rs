// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Packaged URL-path policy. The root must remain trusted and stable. Windows
//! serving additionally validates the opened file handle before reading any bytes.
//! Unix opens each canonical path component relative to a retained directory
//! descriptor, refusing symlinks introduced between resolution and opening.
use std::path::{Path, PathBuf};

pub(super) const DEFAULT_MAX_BYTES: usize = 64 * 1024 * 1024;

pub(super) fn decode(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len()
                || !bytes[i + 1].is_ascii_hexdigit()
                || !bytes[i + 2].is_ascii_hexdigit()
            {
                return None;
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    let decoded = percent_encoding::percent_decode_str(path)
        .decode_utf8()
        .ok()?
        .into_owned();
    (!decoded.contains('\0')).then_some(decoded)
}

pub(super) fn relative_path(path: &str) -> Option<PathBuf> {
    let decoded = decode(path)?;
    if decoded.starts_with("//") || decoded.contains(['\\', ':', '\0']) {
        return None;
    }
    let relative = decoded.strip_prefix('/').unwrap_or(&decoded);
    let mut result = PathBuf::new();
    for part in relative.split('/') {
        if part == ".." || (part != "." && (part.ends_with('.') || part.ends_with(' '))) {
            return None;
        }
        if !part.is_empty() && part != "." {
            result.push(part);
        }
    }
    if result.as_os_str().is_empty() || result.is_absolute() {
        None
    } else {
        Some(result)
    }
}

pub(super) fn resolve(root: &Path, path: &str) -> Option<PathBuf> {
    let relative = relative_path(path)?;
    let root = root.canonicalize().ok()?;
    let candidate = root.join(relative).canonicalize().ok()?;
    (candidate.starts_with(&root) && candidate.is_file()).then_some(candidate)
}

/// Read from one file handle; Windows checks that handle's resolved path before IO.
/// This does not defend against replacement of the trusted root itself, hard links
/// deliberately installed by its owner, or concurrent edits to an admitted file.
#[cfg(test)]
pub(super) fn read_packaged(
    root: &Path,
    path: &str,
) -> std::io::Result<Option<(PathBuf, Vec<u8>)>> {
    read_packaged_with_limit(root, path, DEFAULT_MAX_BYTES)
}

pub(super) fn read_packaged_with_limit(
    root: &Path,
    path: &str,
    max_bytes: usize,
) -> std::io::Result<Option<(PathBuf, Vec<u8>)>> {
    let Some(candidate) = resolve(root, path) else {
        return Ok(None);
    };
    let root = root.canonicalize()?;
    let file = open_checked(&root, &candidate)?;
    let body = read_bounded(file, max_bytes)?;
    Ok(Some((candidate, body)))
}

/// Bound reads even if an admitted file grows after its metadata was inspected.
pub(super) fn read_bounded(file: std::fs::File, max_bytes: usize) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    if file.metadata()?.len() > max_bytes as u64 {
        return Err(size_error());
    }
    let mut body = Vec::new();
    file.take((max_bytes as u64).saturating_add(1))
        .read_to_end(&mut body)?;
    if body.len() > max_bytes {
        return Err(size_error());
    }
    Ok(body)
}

pub(super) fn size_error() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "asset exceeds its byte limit",
    )
}

fn open_checked(root: &Path, candidate: &Path) -> std::io::Result<std::fs::File> {
    #[cfg(unix)]
    let file = open_beneath(root, candidate)?;
    #[cfg(windows)]
    let file = std::fs::File::open(candidate)?;
    #[cfg(not(any(unix, windows)))]
    let file: std::fs::File = {
        let _ = (root, candidate);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "checked asset opening is unavailable on this platform",
        ));
    };
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "asset is not a file",
        ));
    }
    #[cfg(windows)]
    if !opened_path(&file)?.starts_with(root) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "opened asset escaped its root",
        ));
    }
    Ok(file)
}

#[cfg(unix)]
fn open_beneath(root: &Path, candidate: &Path) -> std::io::Result<std::fs::File> {
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::{ffi::OsStrExt, fs::OpenOptionsExt},
        },
        path::Component,
    };
    let denied = || {
        std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "asset escaped its root",
        )
    };
    let relative = candidate.strip_prefix(root).map_err(|_| denied())?;
    let mut parts = relative.components().peekable();
    if parts.peek().is_none() {
        return Err(denied());
    }
    // The application owns the trusted root and its ancestors. Descendants are
    // opened only through directory descriptors, never by reopening a full path.
    let mut current = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(root)?;
    while let Some(part) = parts.next() {
        let Component::Normal(name) = part else {
            return Err(denied());
        };
        let name = CString::new(name.as_bytes()).map_err(|_| denied())?;
        let mut flags = libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK;
        if parts.peek().is_some() {
            flags |= libc::O_DIRECTORY;
        }
        // SAFETY: current is a live owned directory descriptor; name is NUL
        // terminated. No O_CREAT/O_TMPFILE is used, so openat needs no mode.
        let fd = unsafe { libc::openat(current.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: openat returned a new descriptor owned exclusively by this call.
        current = unsafe { std::fs::File::from_raw_fd(fd) };
    }
    Ok(current)
}

#[cfg(windows)]
fn opened_path(file: &std::fs::File) -> std::io::Result<PathBuf> {
    use std::os::windows::{ffi::OsStringExt, io::AsRawHandle};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFinalPathNameByHandleW(
            handle: *mut std::ffi::c_void,
            path: *mut u16,
            size: u32,
            flags: u32,
        ) -> u32;
    }
    // FILE_NAME_NORMALIZED | VOLUME_NAME_DOS = 0, matching std::fs::canonicalize.
    // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew
    let mut buffer = vec![0u16; 512];
    loop {
        // SAFETY: file owns a live handle; buffer is writable for the supplied u16
        // count. The function retains neither pointer, and no bytes are read yet.
        let length = unsafe {
            GetFinalPathNameByHandleW(
                file.as_raw_handle(),
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                0,
            )
        } as usize;
        if length == 0 {
            return Err(std::io::Error::last_os_error());
        }
        if length < buffer.len() {
            return Ok(PathBuf::from(std::ffi::OsString::from_wide(
                &buffer[..length],
            )));
        }
        if length >= 65536 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "opened asset path is too long",
            ));
        }
        buffer.resize(length + 1, 0);
    }
}

/// Application-owned file references, including Manganis's unbundled source URLs.
pub(super) fn resolve_trusted(root: Option<&Path>, path: &str) -> Option<PathBuf> {
    let decoded = decode(path)?;
    if !decoded.starts_with("/assets/") {
        let candidate = PathBuf::from(&decoded);
        if candidate.is_absolute() && candidate.is_file() {
            return candidate.canonicalize().ok();
        }
        #[cfg(windows)]
        if let Some(stripped) = decoded.strip_prefix('/') {
            let candidate = PathBuf::from(stripped);
            if candidate.is_absolute() && candidate.is_file() {
                return candidate.canonicalize().ok();
            }
        }
    }
    resolve(root?, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "aether-asset-policy-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("bundle")).unwrap();
        fs::write(root.join("bundle/hello world.txt"), b"asset").unwrap();
        fs::write(root.join("outside.txt"), b"private").unwrap();
        root
    }
    #[test]
    fn rejects_traversal_and_platform_path_escapes() {
        for path in [
            "../private",
            "/../private",
            "/%2e%2e/private",
            "/assets/%2E%2E/private",
            "/assets/..%5cprivate",
            "/C:/private",
            "C:\\private",
            "//server/share",
            "/file:stream",
            "/file%00.png",
            "/dir/.. /private",
        ] {
            assert!(relative_path(path).is_none(), "accepted {path:?}");
        }
    }
    #[test]
    fn malformed_encoding_never_panics() {
        for path in ["%", "%0", "%zz", "%ff", "/a%00b"] {
            assert!(relative_path(path).is_none(), "accepted {path:?}");
        }
    }
    #[test]
    fn resolves_only_files_inside_the_bundle() {
        let fixture = fixture();
        let root = fixture.join("bundle");
        assert_eq!(
            resolve(&root, "/hello%20world.txt"),
            Some(root.join("hello world.txt").canonicalize().unwrap())
        );
        assert_eq!(
            resolve(&root, "hello%20world.txt"),
            resolve(&root, "/hello%20world.txt")
        );
        assert!(resolve(&root, "/missing").is_none());
        assert!(resolve(&root, "../outside.txt").is_none());
        assert!(resolve(&root, "/").is_none());
    }
    #[test]
    fn manganis_absolute_development_urls_require_trusted_access() {
        let fixture = fixture();
        let root = fixture.join("bundle");
        let source = fixture.join("outside.txt");
        let url = source.to_string_lossy().replace('\\', "/");
        #[cfg(windows)]
        let url = format!("/{url}");
        assert!(resolve(&root, &url).is_none());
        assert_eq!(
            resolve_trusted(Some(&root), &url),
            Some(source.canonicalize().unwrap())
        );
    }

    #[test]
    fn packaged_read_uses_the_resolved_file() {
        let fixture = fixture();
        let (path, bytes) = read_packaged(&fixture.join("bundle"), "/hello%20world.txt")
            .unwrap()
            .unwrap();
        assert_eq!(bytes, b"asset");
        assert_eq!(
            path,
            fixture
                .join("bundle/hello world.txt")
                .canonicalize()
                .unwrap()
        );
        assert!(
            read_packaged(&fixture.join("bundle"), "../outside.txt")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn per_request_limit_rejects_large_assets_and_accepts_exact_boundary() {
        let fixture = fixture();
        let root = fixture.join("bundle");
        assert_eq!(
            read_packaged_with_limit(&root, "hello%20world.txt", 4)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidData
        );
        assert_eq!(
            read_packaged_with_limit(&root, "hello%20world.txt", 5)
                .unwrap()
                .unwrap()
                .1,
            b"asset"
        );
        fs::write(root.join("empty"), b"").unwrap();
        assert!(
            read_packaged_with_limit(&root, "empty", 0)
                .unwrap()
                .unwrap()
                .1
                .is_empty()
        );
        assert!(read_packaged_with_limit(&root, "hello%20world.txt", 0).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_replacement_after_path_resolution() {
        let fixture = fixture();
        let root = fixture.join("bundle").canonicalize().unwrap();
        let file = root.join("hello world.txt");
        let checked = resolve(&root, "hello%20world.txt").unwrap();
        fs::remove_file(&file).unwrap();
        std::os::unix::fs::symlink(fixture.join("outside.txt"), &file).unwrap();
        assert!(open_checked(&root, &checked).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_directory_replacement_after_path_resolution() {
        let fixture = fixture();
        let root = fixture.join("bundle").canonicalize().unwrap();
        fs::create_dir(root.join("entry")).unwrap();
        fs::write(root.join("entry/outside.txt"), b"public").unwrap();
        let checked = resolve(&root, "entry/outside.txt").unwrap();
        fs::rename(root.join("entry"), root.join("retained")).unwrap();
        std::os::unix::fs::symlink(&fixture, root.join("entry")).unwrap();
        assert!(open_checked(&root, &checked).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn permits_internal_symlinks_resolved_before_opening() {
        let fixture = fixture();
        let root = fixture.join("bundle");
        std::os::unix::fs::symlink("hello world.txt", root.join("alias")).unwrap();
        assert_eq!(read_packaged(&root, "alias").unwrap().unwrap().1, b"asset");
    }

    #[cfg(windows)]
    #[test]
    fn handle_check_rejects_a_file_outside_the_captured_root() {
        let fixture = fixture();
        let root = fixture.join("bundle").canonicalize().unwrap();
        // Simulate the resulting target of a redirected path between resolution
        // and opening. Admission is based on the opened handle, not its old name.
        let error = open_checked(&root, &fixture.join("outside.txt")).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        let file = open_checked(&root, &root.join("hello world.txt")).unwrap();
        assert_eq!(opened_path(&file).unwrap(), root.join("hello world.txt"));
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "requires directory-junction permission; run explicitly in native qualification"]
    fn rejects_junction_replacement_between_resolution_and_open() {
        let fixture = fixture();
        let root = fixture.join("bundle").canonicalize().unwrap();
        let entry = root.join("entry");
        let retained = root.join("entry-original");
        let outside = fixture.join("private-directory");
        fs::create_dir(&entry).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(entry.join("payload.txt"), b"packaged").unwrap();
        fs::write(outside.join("payload.txt"), b"private").unwrap();
        let checked = resolve(&root, "entry/payload.txt").unwrap();
        // Only move our fixture directory within its verified absolute test root.
        assert!(entry.starts_with(&root) && retained.starts_with(&root));
        fs::rename(&entry, &retained).unwrap();
        // A directory junction requires no symlink privilege. Paths are passed as
        // environment data, not interpolated into shell code.
        let output = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command",
                "$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path $env:AETHER_TEST_LINK -Target $env:AETHER_TEST_TARGET | Out-Null"])
            .env("AETHER_TEST_LINK", fixture.join("bundle/entry"))
            .env("AETHER_TEST_TARGET", &outside)
            .output().expect("Windows junction fixture requires PowerShell");
        assert!(
            output.status.success(),
            "junction creation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        // The pathname was admitted before the swap. Opening it now follows the
        // junction, but its actual handle must be rejected before reading bytes.
        let error = open_checked(&root, &checked).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(read_packaged(&root, "entry/payload.txt").unwrap().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() {
        let fixture = fixture();
        std::os::unix::fs::symlink(fixture.join("outside.txt"), fixture.join("bundle/link"))
            .unwrap();
        assert!(resolve(&fixture.join("bundle"), "link").is_none());
    }
}
