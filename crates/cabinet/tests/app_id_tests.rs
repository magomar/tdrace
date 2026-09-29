//! Spec 059 (`specs/059_publishready_shared_crates.md`): a game names its own gamepad profile folder.

use cabinet::input::{set_app_id, GamepadManager};
use std::path::PathBuf;

/// Scenario: A game names its own gamepad profile folder
///
/// Given set_app_id("chariot")
/// When candidate_profile_paths runs
/// Then the list has ../chariot/gamepad_profile.json and ~/.config/chariot/gamepad_profile.json,
/// and no tdrace or asteroids entry. Without set_app_id the list is the same as before this spec.
#[test]
fn profile_paths_follow_the_app_id() {
    let before = GamepadManager::candidate_profile_paths();
    assert_eq!(before, GamepadManager::candidate_profile_paths_for(None));
    assert!(before.contains(&PathBuf::from("../tdrace/gamepad_profile.json")), "default list keeps the tdrace folder");
    assert!(before.contains(&PathBuf::from("../asteroids/gamepad_profile.json")));

    set_app_id("chariot");
    let paths = GamepadManager::candidate_profile_paths();
    assert!(paths.contains(&PathBuf::from("../chariot/gamepad_profile.json")), "{:?}", paths);
    if let Some(home) = std::env::var_os("HOME") {
        let own = PathBuf::from(home).join(".config").join("chariot").join("gamepad_profile.json");
        assert!(paths.contains(&own), "{:?}", paths);
    }
    for p in &paths {
        let s = p.to_string_lossy();
        assert!(!s.contains("tdrace") && !s.contains("asteroids"), "unexpected {}", s);
    }
}
