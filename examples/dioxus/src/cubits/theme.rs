//! `ThemeReactor` — manages the app's visual theme (Light / Dark).
//!
//! Demonstrates a simple enum state with GLoC. The cubit holds no extra
//! fields — it only owns the current theme and exposes a single `toggle()`.

use gloc::reactor;
use gloc::Reactor;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// The two visual themes the app supports.
#[derive(Clone, PartialEq, Debug)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    /// Returns the CSS page background colour for this theme.
    pub fn background(&self) -> &'static str {
        match self {
            Theme::Light => "#f0f4f8",
            Theme::Dark => "#0f172a",
        }
    }

    /// Returns the CSS card/surface background colour for this theme.
    pub fn card_background(&self) -> &'static str {
        match self {
            Theme::Light => "#ffffff",
            Theme::Dark => "#1e293b",
        }
    }

    /// Returns the CSS text colour for this theme.
    pub fn text_color(&self) -> &'static str {
        match self {
            Theme::Light => "#1a202c",
            Theme::Dark => "#f1f5f9",
        }
    }

    /// Returns the label shown on the toggle button.
    pub fn label(&self) -> &'static str {
        match self {
            Theme::Light => "Switch to Dark Mode",
            Theme::Dark => "Switch to Light Mode",
        }
    }
}

// ---------------------------------------------------------------------------
// Cubit
// ---------------------------------------------------------------------------

/// Manages the app's visual theme.
///
/// The macro generates `new()` and `impl Reactor`.
#[reactor(state = Theme)]
pub struct ThemeReactor {}

impl ThemeReactor {
    /// Toggles between Light and Dark themes.
    pub fn toggle(&mut self) {
        let next = match self.state() {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
        self.emit(next);
    }
}
