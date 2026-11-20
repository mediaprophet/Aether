// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
/// Local asset authority selected by the native application host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssetAccess {
    /// Only files contained by the packaged asset root.
    #[default]
    Packaged,
    /// Trusted application sources, including Manganis development absolute paths.
    /// Never use this profile for remote pages or untrusted script asset requests.
    TrustedApplicationFiles,
}

impl AssetAccess {
    /// Restrict transport entry points as well as packaged URL resolution.
    /// HTTP origin permissions and redirects remain the host transport's responsibility.
    pub(crate) fn allows_delegated_scheme(self, scheme: &str) -> bool {
        self == Self::TrustedApplicationFiles || matches!(scheme, "http" | "https" | "data")
    }
}

#[cfg(test)]
mod tests {
    use super::AssetAccess;

    #[test]
    fn packaged_authority_does_not_delegate_files_or_custom_services() {
        for scheme in [
            "file",
            "ftp",
            "smb",
            "qualia",
            "webizen",
            "javascript",
            "unknown",
            "",
        ] {
            assert!(
                !AssetAccess::Packaged.allows_delegated_scheme(scheme),
                "{scheme}"
            );
        }
        for scheme in ["http", "https", "data"] {
            assert!(
                AssetAccess::Packaged.allows_delegated_scheme(scheme),
                "{scheme}"
            );
        }
    }

    #[test]
    fn explicit_trusted_profile_preserves_custom_host_transports() {
        assert_eq!(AssetAccess::default(), AssetAccess::Packaged);
        for scheme in ["file", "custom", "https"] {
            assert!(AssetAccess::TrustedApplicationFiles.allows_delegated_scheme(scheme));
        }
    }
}
