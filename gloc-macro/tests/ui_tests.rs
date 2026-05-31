//! Trybuild UI tests — compile-pass and compile-fail scenarios for `#[reactor]`.
//!
//! Run with: `cargo test -p gloc-macro --test ui_tests`
//!
//! To regenerate expected `.stderr` files after intentional error message changes:
//! `TRYBUILD=overwrite cargo test -p gloc-macro --test ui_tests`

#[test]
fn ui() {
    let t = trybuild::TestCases::new();

    // --- Pass cases: these must compile and run without errors ---
    t.pass("tests/ui/pass/mode_a_basic.rs");
    t.pass("tests/ui/pass/mode_b_basic.rs");
    t.pass("tests/ui/pass/mode_a_no_new.rs");
    t.pass("tests/ui/pass/mode_a_no_observers.rs");
    t.pass("tests/ui/pass/mode_b_extra_fields.rs");
    t.pass("tests/ui/pass/events_mode_a.rs");

    // --- Fail cases: these must fail with the expected compiler error ---
    t.compile_fail("tests/ui/fail/no_state_arg_no_annotation.rs");
    t.compile_fail("tests/ui/fail/both_modes.rs");
    t.compile_fail("tests/ui/fail/applied_to_tuple_struct.rs");
    t.compile_fail("tests/ui/fail/mode_b_no_state_fields.rs");
    t.compile_fail("tests/ui/fail/events_missing_on_event.rs");
}
