//! Regression tests for yofi's rendering pipeline.
//!
//! These tests render headlessly to a buffer and compare against stored reference snapshots.
//! With YOFI_BLESS=1 env, snapshot images are overwritten.
//! Snapshots are stored as PNG files in `tests/fixtures/` and can be viewed directly.
//!
//! On mismatch, `<name>.new.png` and `<name>.diff.png` are saved next to the fixture for
//! inspection. The diff highlights changed pixels in red.

mod snap;
use snap::{
    run_regression, run_regression_with_padding, test_entries, test_entries_with_long_name, Action,
};

#[test]
fn initial() {
    run_regression("initial", test_entries(), &[]);
}

#[test]
fn search() {
    run_regression("search", test_entries(), &[Action::Type("fire")]);
}

#[test]
fn nav_down() {
    run_regression(
        "nav_down",
        test_entries(),
        &[Action::NextItem, Action::NextItem],
    );
}

#[test]
fn search_then_nav() {
    run_regression(
        "search_then_nav",
        test_entries(),
        &[Action::Type("te"), Action::NextItem],
    );
}

#[test]
fn long_name() {
    run_regression_with_padding("long_name", test_entries_with_long_name(), &[], (20, 10));
}

#[test]
fn long_input() {
    run_regression_with_padding(
        "long_input",
        test_entries(),
        &[Action::Type(
            "very long input string that goes well past the input width",
        )],
        (20, 10),
    );
}
