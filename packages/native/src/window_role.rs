// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use anyrender::CompositeAlphaMode;
use peniko::Color;
use winit::window::{WindowAttributes, WindowLevel};

/// Presentation role selected by the host application.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowRole {
    #[default]
    StandardAppWindow,
    /// Frameless, transparent command overlay.
    OrbOverlay,
    /// Active, always-on-top alert window.
    UrgentAlertModal,
    /// Always-on-top tool palette or inspector.
    AuxiliaryToolPalette,
}

/// Action taken when a native window receives a close request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowClosePolicy {
    #[default]
    CloseAndDrop,
    /// Hide the window while retaining its VirtualDOM and state.
    HideOnClose,
}

impl WindowRole {
    /// Apply the role and return its optional compositor defaults.
    pub fn apply_to_attributes(
        self,
        attrs: WindowAttributes,
    ) -> (WindowAttributes, Option<CompositeAlphaMode>, Option<Color>) {
        match self {
            Self::StandardAppWindow => (attrs, None, None),
            Self::OrbOverlay => (
                attrs
                    .with_decorations(false)
                    .with_transparent(true)
                    .with_window_level(WindowLevel::AlwaysOnTop)
                    .with_resizable(false),
                Some(CompositeAlphaMode::Transparent),
                Some(Color::TRANSPARENT),
            ),
            Self::UrgentAlertModal => (
                attrs
                    .with_window_level(WindowLevel::AlwaysOnTop)
                    .with_active(true),
                None,
                None,
            ),
            Self::AuxiliaryToolPalette => (
                attrs
                    .with_window_level(WindowLevel::AlwaysOnTop)
                    .with_resizable(true),
                None,
                None,
            ),
        }
    }
}
