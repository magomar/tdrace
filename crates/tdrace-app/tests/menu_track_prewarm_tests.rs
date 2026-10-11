//! The background prewarm fills the menu circuit cache, so the first visit to a
//! circuit category does not load circuits on the UI thread.
//! Own test binary: other tests clear the shared cache.

use std::time::Instant;
use tdrace_app::ui::menu::{clear_menu_track_cache, prewarm_menu_track_cache, resolve_track_for_menu, TrackChoice};

#[test]
fn prewarm_fills_menu_track_cache() {
    clear_menu_track_cache();
    let choices: Vec<TrackChoice> = TrackChoice::ALL.to_vec();
    let handle = prewarm_menu_track_cache(choices.clone()).expect("prewarm thread must start");
    handle.join().expect("prewarm thread must not panic");

    for choice in &choices {
        let t0 = Instant::now();
        resolve_track_for_menu(choice).expect("standard choice must resolve");
        let took = t0.elapsed();
        assert!(took.as_millis() < 5, "{} must come from the cache, took {:?}", choice.track_id(), took);
    }
}

