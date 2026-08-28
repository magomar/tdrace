use macroquad::prelude::*;
use tdrace_app::game::RaceSession;
use tdrace_app::render::color::Palette;

fn window_conf() -> Conf {
    Conf {
        window_title: "TDRace - Modular Arcade Motorsport Platform".to_string(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        sample_count: 4,
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
        } else if clean_arg == "f1" {
            session.switch_to_f1();
        } else if clean_arg == "rally" {
            session.switch_to_rally();
        } else if clean_arg == "kart" {
            session.switch_to_kart();
        } else if clean_arg == "classic" {
            session.switch_to_classic();
        } else if clean_arg == "championship" || clean_arg == "f1-championship" {
            session.start_f1_championship();
        }
    }

    loop {
        // Clear background with rich GeneRally grass color
        clear_background(Palette::GRASS);

        // Update race simulation / input / AI / physics
        session.update();

        // Render world entities & screen HUD
        session.render();

        next_frame().await;
    }
}

