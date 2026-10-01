//! Native visual smoke test for the migrated results, pause, and controls screens.
//! Saves screenshots to TDRACE_UI_PREVIEW_DIR and exits without starting a race.
use macroquad::prelude::*;
use tdrace_app::ui::menu::{render_controls_screen, render_pause_menu, render_results_screen, RaceResultEntry};

fn window_conf() -> Conf {
    Conf { window_title: "UI migration preview".into(), window_width: 1280, window_height: 720, ..Default::default() }
}

#[macroquad::main(window_conf)]
async fn main() {
    let output = std::env::var("TDRACE_UI_PREVIEW_DIR").expect("Set TDRACE_UI_PREVIEW_DIR");
    let fonts = cabinet::ui::Fonts::load_embedded();
    let results: Vec<_> = (0..12).map(|i| RaceResultEntry {
        position: i + 1, car_name: format!("Driver {} / Long vehicle designation", i + 1),
        is_player: i == 6, total_time: 180.0 + i as f32, best_lap: Some(59.125),
        delta_to_leader: i as f32, car_idx: i, points_awarded: 25 - i as u32,
        projected: i > 9,
    }).collect();
    for (name, width, height) in [("results", 1280., 720.), ("results-small", 800., 600.), ("pause", 1280., 720.), ("controls", 1280., 720.)] {
        request_new_screen_size(width, height);
        next_frame().await;
        for frame in 0..3 {
            clear_background(BLACK);
            match name {
                "pause" => render_pause_menu(&fonts, tdrace_core::physics::config::AssistProfile::Arcade, &tdrace_app::audio::AudioSettings::default(), 4),
                "controls" => render_controls_screen(&fonts, tdrace_core::physics::config::AssistProfile::Arcade, false, "", &cabinet::input::InputMap::default(), "Default", &cabinet::input::DigitalInputConfig::default(), None),
                _ => render_results_screen(&fonts, "Preview circuit", &results, false, true, None),
            }
            if frame == 2 {
                get_screen_data().export_png(&format!("{}/{}.png", output, name));
            }
            next_frame().await;
        }
    }
}
