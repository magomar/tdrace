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
            session.garage_car_idx = 3;
        } else if clean_arg == "classic" {
            session.switch_to_classic();
        } else if clean_arg == "nascar-championship" || clean_arg == "stockcar-championship" {
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
                .and_then(|s| s.parse::<usize>().ok());
            session.garage_origin = tdrace_app::game::GarageOrigin::ModalitySelect;
            session.garage_tier = tier;
            if let Some(idx) = car_idx {
                session.garage_car_idx = idx;
            }
            if args.iter().any(|a| a == "--turntable") {
                session.garage_view_mode = tdrace_app::ui::GarageViewMode::TopDownTurntable;
            }
            if args.iter().any(|a| a == "--gallery") {
                session.garage_gallery_mode = true;
            }
        } else if clean_arg == "track" || clean_arg == "circuit" {
            if let Some(target) = args.get(i + 1) {
                let track_opt = if target.ends_with(".json") || target.contains('/') {
                    tdrace_core::track::Track::load_from_file(target).ok()
                } else if let Ok(t) = tdrace_core::track::Track::load_from_file(format!("tracks/gt/{}.json", target)) {
                    Some(t)
                } else {
                    session.track_manager.load_track_by_slug(target).ok()
                };
                if let Some(track) = track_opt {
                    let id = target.trim_end_matches(".json").split('/').next_back().unwrap_or(target).to_string();
                    let name = track.name.clone();
                    let path = format!("tracks/gt/{}.json", id);
                    session.track_choice = tdrace_app::ui::menu::TrackChoice::Custom {
                        id,
                        title: name,
                        description: String::new(),
                        path,
                    };
                    session.track = track;
                }
            }
        } else if clean_arg == "editor" {
            let track = session.track.clone();
            session.enter_track_editor(track);
        } else if clean_arg == "in-pit" || clean_arg == "pit-stop" {
            session.init_race();
            if let Some(lane) = &session.track.pit_lane {
                if let Some(b0) = lane.pit_boxes.first() {
                    if let Some(v0) = session.world.vehicles.first_mut() {
                        let fwd = if b0.direction.length_squared() > 1e-4 {
                            b0.direction.normalize()
                        } else {
                            tdrace_core::Vec2::X
                        };
                        let angle = fwd.y.atan2(fwd.x);
                        v0.state.position = b0.position;
                        v0.state.velocity = tdrace_core::Vec2::ZERO;
                        v0.state.angle = angle;
                    }
                    if let Some(pit_state) = session.world.pit_states.first_mut() {
                        *pit_state = race_kit::PitServiceState::StationaryInBox {
                            timer: 1.25,
                            target_duration: 2.5,
                        };
                    }
                }
            }
            session.state = tdrace_app::game::GameState::Racing;
        } else if clean_arg == "race" || clean_arg == "quick-race" {
            session.init_race();
            session.state = tdrace_app::game::GameState::StartingGrid;
        } else if clean_arg == "racing" {
            session.init_race();
            session.state = tdrace_app::game::GameState::Racing;
        } else if clean_arg == "lan" || clean_arg == "lan-hub" {
            session.state = tdrace_app::game::GameState::LanHub { selected_idx: 0 };
        } else if clean_arg == "lan-host" {
            // Optional value opens a lobby sub-screen: `--lan-host circuit` or `--lan-host car`
            session.select_lan_hub_option(0);
            match args.get(i + 1).map(|s| s.as_str()) {
                Some("circuit") => session.open_lan_circuit_selector(),
                Some("car") => session.open_lan_garage(),
                _ => {}
            }
        }
    }

    // Dev-only: `editor --editor-select <kind>` opens the inspector on a fixed selection (spec 086).
    if let Some(kind) = args.iter().position(|a| a == "--editor-select").and_then(|i| args.get(i + 1)) {
        if let Some(state) = session.editor_state.as_mut() {
            let selection = tdrace_app::editor::state::dev_selection(kind, &state.track);
            state.select(selection);
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
    let focus_pit = args.iter().any(|a| a == "--focus-pit");
    let mut frame_count: u32 = 0;

    loop {
        if focus_pit && frame_count <= 2 {
            if let Some(lane) = &session.track.pit_lane {
                let mut min = tdrace_core::Vec2::splat(f32::MAX);
                let mut max = tdrace_core::Vec2::splat(f32::MIN);
                for wp in &lane.spline.waypoints {
                    min = min.min(wp.point);
                    max = max.max(wp.point);
                }
                min -= tdrace_core::Vec2::splat(30.0);
                max += tdrace_core::Vec2::splat(30.0);
                let sw = screen_width();
                let sh = screen_height();
                session.editor_camera.focus_bounds(min, max, sw, sh);
                session.editor_camera.center = session.editor_camera.target_center;
                session.editor_camera.zoom = session.editor_camera.target_zoom;
            }
        }

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

