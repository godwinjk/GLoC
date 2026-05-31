//! Integration tests for the `#[reactor_state]` attribute macro.
//!
//! Tests verify that:
//! - Required derives (Clone, PartialEq, Debug) are always injected
//! - Extra derives passed via `derive(...)` are appended correctly
//! - Works on both structs and enums
//! - Non-`#[reactor_state]` attributes on the item are preserved

use gloc_macro::reactor_state;

// ---------------------------------------------------------------------------
// Struct tests
// ---------------------------------------------------------------------------

/// Basic struct — no extras, required derives only.
#[reactor_state]
pub struct CounterState {
    pub count: i32,
}

#[test]
fn struct_is_cloneable() {
    let s = CounterState { count: 5 };
    let c = s.clone();
    assert_eq!(c.count, 5);
}

#[test]
fn struct_is_comparable() {
    let a = CounterState { count: 5 };
    let b = CounterState { count: 5 };
    let c = CounterState { count: 9 };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn struct_is_debuggable() {
    let s = CounterState { count: 42 };
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("42"));
}

// ---------------------------------------------------------------------------
// Struct with extra derives
// ---------------------------------------------------------------------------

/// Struct with an extra derive — Hash appended after the required three.
#[reactor_state(derive(Eq, Hash))]
pub struct TaggedState {
    pub tag: u32,
}

#[test]
fn extra_derive_hash_works() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(TaggedState { tag: 1 });
    set.insert(TaggedState { tag: 2 });
    set.insert(TaggedState { tag: 1 }); // duplicate — not inserted
    assert_eq!(set.len(), 2);
}

#[test]
fn extra_derive_struct_still_has_required_derives() {
    let s = TaggedState { tag: 7 };
    let c = s.clone();
    assert_eq!(s, c);
    let _ = format!("{:?}", s);
}

// ---------------------------------------------------------------------------
// Enum tests
// ---------------------------------------------------------------------------

/// Enum state — required derives injected automatically.
#[reactor_state]
pub enum LoadingState {
    Idle,
    Loading,
    Done,
    Error(String),
}

#[test]
fn enum_is_cloneable() {
    let s = LoadingState::Error("oops".into());
    let c = s.clone();
    assert_eq!(c, LoadingState::Error("oops".into()));
}

#[test]
fn enum_is_comparable() {
    assert_eq!(LoadingState::Idle, LoadingState::Idle);
    assert_ne!(LoadingState::Idle, LoadingState::Loading);
}

#[test]
fn enum_is_debuggable() {
    let s = LoadingState::Loading;
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("Loading"));
}

// ---------------------------------------------------------------------------
// Enum with extra derives
// ---------------------------------------------------------------------------

#[reactor_state(derive(Eq, Hash))]
pub enum StatusState {
    Active,
    Inactive,
}

#[test]
fn enum_extra_derive_hash_works() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(StatusState::Active);
    set.insert(StatusState::Inactive);
    set.insert(StatusState::Active); // duplicate
    assert_eq!(set.len(), 2);
}

// ---------------------------------------------------------------------------
// Multi-field struct
// ---------------------------------------------------------------------------

/// Struct with multiple fields of different types.
#[reactor_state]
pub struct UserState {
    pub name: String,
    pub age: u32,
    pub active: bool,
}

#[test]
fn multi_field_struct_derives_work() {
    let s = UserState {
        name: "Alice".into(),
        age: 30,
        active: true,
    };
    let c = s.clone();
    assert_eq!(s, c);
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("Alice"));
    assert!(dbg.contains("30"));
}

// ---------------------------------------------------------------------------
// Preserved existing attributes
// ---------------------------------------------------------------------------

/// Other attributes on the struct must survive the macro.
#[reactor_state]
#[allow(dead_code)]
pub struct AnnotatedState {
    value: i32,
}

#[test]
fn other_attributes_are_preserved() {
    let s = AnnotatedState { value: 1 };
    let c = s.clone();
    assert_eq!(s, c);
}

// ---------------------------------------------------------------------------
// Works alongside #[reactor]
// ---------------------------------------------------------------------------

use gloc::Reactor;
use gloc_macro::reactor;

#[reactor_state]
pub struct ScoreState {
    pub score: u32,
}

#[reactor(state = ScoreState)]
pub struct ScoreReactor {}

impl ScoreReactor {
    pub fn add(&mut self, pts: u32) {
        self.emit(ScoreState {
            score: self.state().score + pts,
        });
    }
}

#[test]
fn reactor_state_and_reactor_work_together() {
    let mut r = ScoreReactor::new(ScoreState { score: 0 });
    r.add(10);
    r.add(5);
    assert_eq!(r.state().score, 15);
}

#[test]
fn reactor_state_change_detection_works() {
    let mut r = ScoreReactor::new(ScoreState { score: 10 });
    r.emit(ScoreState { score: 10 }); // no-op
    assert_eq!(r.state().score, 10);
}
