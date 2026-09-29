//! Where the case, the results and the crash logs go, as the environment says.
//!
//! Read through a stand-in for the environment, so the tests never change the
//! process's own variables (which other tests running at the same time read).

use std::ffi::OsString;
use std::path::PathBuf;
use vleo_data::{case_path_from, log_path_from, results_path_from};

fn env(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<OsString> {
    move |k| {
        pairs
            .iter()
            .find(|(name, _)| *name == k)
            .map(|(_, v)| OsString::from(*v))
    }
}

#[test]
fn a_set_variable_is_the_path() {
    let e = env(&[
        ("HOME", "/home/ana"),
        ("VLEO_RESULTS", "/shared/vleo-results"),
    ]);
    assert_eq!(results_path_from(&e), PathBuf::from("/shared/vleo-results"));
}

#[test]
fn unset_means_the_home_folder() {
    let e = env(&[("HOME", "/home/ana")]);
    assert_eq!(
        case_path_from(&e),
        PathBuf::from("/home/ana/.vleo/case/inputs.csv")
    );
    assert_eq!(
        results_path_from(&e),
        PathBuf::from("/home/ana/.vleo/results")
    );
    assert_eq!(log_path_from(&e), PathBuf::from("/home/ana/.vleo/log"));
}

#[test]
fn set_but_empty_means_unset_not_the_current_folder() {
    let e = env(&[
        ("HOME", "/home/ana"),
        ("VLEO_CASE", ""),
        ("VLEO_RESULTS", ""),
        ("VLEO_LOG", ""),
    ]);
    assert_eq!(
        case_path_from(&e),
        PathBuf::from("/home/ana/.vleo/case/inputs.csv")
    );
    assert_eq!(
        results_path_from(&e),
        PathBuf::from("/home/ana/.vleo/results")
    );
    assert_eq!(log_path_from(&e), PathBuf::from("/home/ana/.vleo/log"));
}

#[test]
fn windows_without_home_uses_the_profile_folder() {
    let e = env(&[("USERPROFILE", r"C:\Users\ana")]);
    assert_eq!(
        results_path_from(&e),
        PathBuf::from(r"C:\Users\ana").join(".vleo").join("results")
    );
}
