use macroquad::prelude::*;
use tdrace_app::game::RaceSession;

fn window_conf() -> Conf {
    let cfg = tdrace_app::config::GameConfig::load_or_default();
    let width = cfg.display.window_width.max(640) as i32;
    let height = cfg.display.window_height.max(360) as i32;
    let fullscreen = cfg.display.fullscreen;

    let icon = Some(macroquad::miniquad::conf::Icon {
        small: *include_bytes!("../../../assets/icons/icon_16.rgba"),
        medium: *include_bytes!("../../../assets/icons/icon_32.rgba"),
        big: *include_bytes!("../../../assets/icons/icon_64.rgba"),
    });

    Conf {
        window_title: "TDRace - Modular Arcade Motorsport Platform".to_string(),
        window_width: width,
        window_height: height,
        fullscreen,
        window_resizable: true,
        high_dpi: true,
        sample_count: 4,
        icon,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut session = RaceSession::new();
    session.init_audio().await;

    // Parse CLI arguments for direct motorsport module or championship launch
    let args: Vec<String> = std::env::args().collect();
    for (i, arg) in args.iter().enumerate() {
        let clean_arg = arg.trim_start_matches('-');
        if arg == "--module" || arg == "-m" || clean_arg == "module" {
            if let Some(mod_name) = args.get(i + 1) {
                session.switch_to_module(mod_name.trim_start_matches('-'));
            }
        } else if clean_arg == "gt" || clean_arg == "f1" {
            session.switch_to_gt();
        } else if clean_arg == "nascar" {
            session.switch_to_nascar();
        } else if clean_arg == "rally" {
            session.switch_to_rally();
        } else if clean_arg == "kart" {
            session.switch_to_kart();
        } else if clean_arg == "extreme-offroad" || clean_arg == "offroad" || clean_arg == "buggy" {
            session.switch_to_extreme_offroad();
        } else if clean_arg == "classic" {
            session.switch_to_classic();
        } else if clean_arg == "nascar-championship" {
            session.start_nascar_championship();
        } else if clean_arg == "offroad-championship" || clean_arg == "extreme-offroad-championship" {
            session.start_extreme_offroad_championship();
        } else if clean_arg == "championship" || clean_arg == "gt-championship" || clean_arg == "f1-championship" {
            session.start_gt_championship();
        } else if clean_arg == "split" || clean_arg == "splitscreen" || clean_arg == "s" {
            session.game_mode = tdrace_app::ui::menu::GameMode::SplitScreen;
        }
    }

    loop {
        // Clear background with active track's pallid backdrop color
        clear_background(session.active_backdrop_color());

        // Update race simulation / input / AI / physics
        session.update();

        // Render world entities & screen HUD
        session.render();

        next_frame().await;
    }
}

