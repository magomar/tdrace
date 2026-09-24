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
        platform: macroquad::miniquad::conf::Platform {
            linux_wm_class: "tdrace",
            ..Default::default()
        },
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
        } else if clean_arg == "gt" {
            session.switch_to_gt();
        } else if clean_arg == "nascar" {
            session.switch_to_nascar();
        } else if clean_arg == "rally" {
            session.switch_to_rally();
        } else if clean_arg == "kart" {
            session.switch_to_kart();
        } else if clean_arg == "extreme-offroad" || clean_arg == "offroad" || clean_arg == "buggy" {
            session.switch_to_extreme_offroad();
        } else if clean_arg == "classic-kart" {
            session.switch_to_classic();
            session.track_choice = tdrace_app::ui::menu::TrackChoice::KartArena;
            session.car_choice = tdrace_app::ui::menu::CarChoice::Kart;
            session.selected_car_model_id = Some("classic_kart");
        } else if clean_arg == "classic" {
            session.switch_to_classic();
        } else if clean_arg == "nascar-championship" {
            session.start_nascar_championship();
        } else if clean_arg == "offroad-championship" || clean_arg == "extreme-offroad-championship" {
            session.start_extreme_offroad_championship();
        } else if clean_arg == "championship" || clean_arg == "gt-championship" {
            session.start_gt_championship();
        } else if clean_arg == "split" || clean_arg == "splitscreen" || clean_arg == "s" {
            session.game_mode = tdrace_app::ui::menu::GameMode::SplitScreen;
        } else if clean_arg == "garage" {
            let tier = args.iter().position(|a| a == "--tier" || a == "-t")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(2);
            let car_idx = args.iter().position(|a| a == "--car" || a == "-c")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            session.garage_origin = tdrace_app::game::GarageOrigin::ModalitySelect;
            session.garage_tier = tier;
            session.garage_car_idx = car_idx;
            if args.iter().any(|a| a == "--turntable") {
                session.garage_view_mode = tdrace_app::ui::GarageViewMode::TopDownTurntable;
            }
            if args.iter().any(|a| a == "--gallery") {
                session.garage_gallery_mode = true;
            }
            session.state = tdrace_app::game::GameState::Garage(tdrace_app::game::GarageOrigin::ModalitySelect);
        } else if clean_arg == "race" || clean_arg == "quick-race" {
            session.init_race();
            session.state = tdrace_app::game::GameState::StartingGrid;
        } else if clean_arg == "racing" {
            session.init_race();
            session.state = tdrace_app::game::GameState::Racing;
        }
    }

    let screenshot_path = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let max_frames: u32 = args
        .iter()
        .position(|a| a == "--frames")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let mut frame_count: u32 = 0;

    loop {
        // Clear background with active track's pallid backdrop color
        clear_background(session.active_backdrop_color());

        // Update race simulation / input / AI / physics
        session.update();

        // Render world entities & screen HUD
        session.render();

        frame_count += 1;
        if let Some(path) = &screenshot_path {
            if frame_count >= max_frames {
                let img = get_screen_data();
                img.export_png(path);
                println!("Screenshot exported to {}", path);
                break;
            }
        }

        next_frame().await;
    }
}

