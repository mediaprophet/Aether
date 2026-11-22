// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
use std::{fs::File, io};

// Preserve the parent-created pipe, then move all library/CRT stdout diagnostics
// to stderr before creating any engine or reader threads. Only this File carries
// protocol bytes; inherited diagnostics cannot masquerade as wire messages.
pub fn private_output() -> io::Result<File> {
    #[cfg(windows)]
    let file = {
        use std::os::windows::io::AsHandle;
        let file = File::from(io::stdout().as_handle().try_clone_to_owned()?);
        file
    };
    #[cfg(unix)]
    let file = {
        use std::os::fd::AsFd;
        File::from(io::stdout().as_fd().try_clone_to_owned()?)
    };
    // SAFETY: duplicates the live stderr descriptor onto stdout, before threads.
    if unsafe { libc::dup2(2, 1) } < 0 {
        return Err(io::Error::last_os_error());
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::{
            GetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle,
        };
        // SAFETY: process standard handles are live; this runs before threads.
        if unsafe { SetStdHandle(STD_OUTPUT_HANDLE, GetStdHandle(STD_ERROR_HANDLE)) } == 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(file)
}
