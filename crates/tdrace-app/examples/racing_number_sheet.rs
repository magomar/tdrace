//! Contact sheet for Spec 111: every catalog car of one kind with its racing number, as drawn in a race (sprite
//! body, number in its zone). One sheet per visual type: `racing_number_sheet_<kind>.png` in TDRACE_UI_PREVIEW_DIR
//! (default: current directory).
use macroquad::prelude::*;
use tdrace_app::camera::RaceCamera;
use tdrace_app::catalog::get_all_models;
use tdrace_app::module::VehicleVisualType;
use tdrace_app::render::car::{render_car_numbers, render_car_with_visual_type_model_and_shadows, CarNumberItem};
use tdrace_app::render::color::CarColorScheme;
use tdrace_app::ui::font::Fonts;
use tdrace_core::Car;

fn window_conf() -> Conf {
    Conf { window_title: "Racing number sheet".into(), window_width: 1600, window_height: 900, ..Default::default() }
}

fn kind(v: VehicleVisualType) -> &'static str {
    match v {
        VehicleVisualType::GoKart { .. } => "kart",
        VehicleVisualType::SandRail { .. } => "buggy",
        VehicleVisualType::StockCar { .. } => "stock",
        VehicleVisualType::RallyHatch { .. } => "rally",
        VehicleVisualType::TouringGT { .. } => "gt",
        VehicleVisualType::OpenWheel { .. } => "open_wheel",
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let dir = std::env::var("TDRACE_UI_PREVIEW_DIR").unwrap_or_else(|_| ".".into());
    let fonts = Fonts::load_embedded();
    for sheet in ["buggy", "stock", "rally", "gt", "open_wheel"] {
        let models: Vec<_> = get_all_models().iter().filter(|m| kind(m.visual_type) == sheet).collect();
        let cols = 8;
        let mut camera = RaceCamera::new();
        camera.current_zoom = 26.0 * screen_height() / 900.0;
        let rows = models.len().div_ceil(cols);
        camera.current_pos = tdrace_core::Vec2::new((cols as f32 - 1.0) * 3.5, -((rows as f32 - 1.0) * 3.5));
        for frame in 0..4 {
            clear_background(Color::new(0.30, 0.31, 0.33, 1.0));
            camera.apply();
            let mut items = Vec::new();
            for (i, m) in models.iter().enumerate() {
                let pos = tdrace_core::Vec2::new((i % cols) as f32 * 7.0, -((i / cols) as f32) * 7.0);
                // Heading up the screen, as on most grids.
                let car = Car::new(m.to_car_config()).with_pose(pos, std::f32::consts::FRAC_PI_2);
                let scheme = CarColorScheme::from_index(i);
                render_car_with_visual_type_model_and_shadows(&car, &scheme, false, m.visual_type, Some(m.id), true, false);
                items.extend(CarNumberItem::for_car(&car, 88, scheme.primary, m.visual_type));
            }
            camera.reset_to_screen();
            render_car_numbers(&fonts, &camera, None, &items);
            if frame == 3 {
                get_screen_data().export_png(&format!("{}/racing_number_sheet_{}.png", dir, sheet));
            }
            next_frame().await;
        }
    }
}
