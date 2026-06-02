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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use gloc_test::{reactor_test, ReactorTester};

    use super::*;

    // ---- happy path ----

    #[test]
    fn toggle_from_light_gives_dark() {
        reactor_test! {
            build: ThemeReactor::new(Theme::Light),
            acts: [|r| r.toggle()],
            expect_states: [Theme::Dark],
        }
    }

    #[test]
    fn toggle_from_dark_gives_light() {
        reactor_test! {
            build: ThemeReactor::new(Theme::Dark),
            acts: [|r| r.toggle()],
            expect_states: [Theme::Light],
        }
    }

    #[test]
    fn double_toggle_returns_to_original() {
        reactor_test! {
            build: ThemeReactor::new(Theme::Light),
            acts: [
                |r| r.toggle(),
                |r| r.toggle(),
            ],
            expect_states: [Theme::Dark, Theme::Light],
        }
    }

    // ---- edge case: transitions capture old and new ----

    #[test]
    fn toggle_transition_records_correct_pair() {
        reactor_test! {
            build: ThemeReactor::new(Theme::Light),
            acts: [|r| r.toggle()],
            expect_transitions: [(Theme::Light, Theme::Dark)],
        }
    }

    // ---- boundary: helper methods reflect the correct theme ----

    #[test]
    fn background_reflects_current_theme() {
        let tester = ReactorTester::new(ThemeReactor::new(Theme::Light));
        assert_eq!(tester.state().background(), "#f0f4f8");

        tester.act(|r| r.toggle());
        assert_eq!(tester.state().background(), "#0f172a");
    }

    #[test]
    fn label_reflects_upcoming_toggle_direction() {
        let tester = ReactorTester::new(ThemeReactor::new(Theme::Light));
        assert_eq!(tester.state().label(), "Switch to Dark Mode");

        tester.act(|r| r.toggle());
        assert_eq!(tester.state().label(), "Switch to Light Mode");
    }
}
