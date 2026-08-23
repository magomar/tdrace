use macroquad::prelude::*;
use tdrace_app::game::RaceSession;
use tdrace_app::render::color::Palette;

fn window_conf() -> Conf {
    Conf {
        window_title: "TDRace - 2D Arcade Racing (GeneRally Style)".to_string(),
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
