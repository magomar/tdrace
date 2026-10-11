//! Native visual smoke test for Spec 096: chassis-conforming ground shadows for sprite cars and every
//! procedural archetype, on the ground and lifted 2.5 m. Saves `vehicle_shadows.png` and
//! `vehicle_shadows_closeup.png` to
//! TDRACE_UI_PREVIEW_DIR (default: current directory) and exits.
use macroquad::prelude::*;
use tdrace_app::catalog::get_all_models;
use tdrace_app::module::VehicleVisualType;
use tdrace_app::render::car::render_car_with_visual_type_model_and_shadows;
use tdrace_app::render::color::CarColorScheme;
use tdrace_core::Car;

fn window_conf() -> Conf {
    Conf { window_title: "Vehicle shadow preview".into(), window_width: 1600, window_height: 900, ..Default::default() }
}

const SPRITE_MODELS: [&str; 3] = ["gt_vandorn_arrowhead_t2", "rally_gallia_200_t1", "kart_blackline_cadet_t1"];

/// One catalog model per procedural archetype, drawn without its sprite.
fn archetype_models() -> Vec<&'static tdrace_app::catalog::RealCarModel> {
    let mut seen: Vec<std::mem::Discriminant<VehicleVisualType>> = Vec::new();
    get_all_models()
        .iter()
        .filter(|m| {
            let d = std::mem::discriminant(&m.visual_type);
            let new = !seen.contains(&d);
            seen.push(d);
            new
        })
        .collect()
}

fn draw_car(model: &tdrace_app::catalog::RealCarModel, x: f32, y: f32, z_lift: f32, sprite: bool, index: usize) {
    let mut car = Car::new(model.to_car_config()).with_pose(tdrace_core::Vec2::new(x, y), -0.6);
    car.state.elevation = z_lift;
    let scheme = CarColorScheme::from_index(index);
    let model_id = if sprite { Some(model.id) } else { None };
    render_car_with_visual_type_model_and_shadows(&car, &scheme, false, model.visual_type, model_id, true, false);
}

#[macroquad::main(window_conf)]
async fn main() {
    let dir = std::env::var("TDRACE_UI_PREVIEW_DIR").unwrap_or_else(|_| ".".into());
    let sprites: Vec<_> = SPRITE_MODELS.iter().filter_map(|id| get_all_models().iter().find(|m| m.id == *id)).collect();
    let archetypes = archetype_models();

    // Full view, then a close-up of the lifted sand rail, kart and open-wheel shadows.
    for (name, view) in [("vehicle_shadows", Rect::new(0.0, 0.0, 40.0, 22.5)), ("vehicle_shadows_closeup", Rect::new(17.0, 14.5, 20.0, 11.25))] {
        for frame in 0..4 {
            clear_background(Color::new(0.55, 0.56, 0.58, 1.0));
            let mut camera = Camera2D::from_display_rect(view);
            camera.zoom.y = -camera.zoom.y; // y down, as in the game
            set_camera(&camera);
            // Row 1: sprite cars on the ground, then lifted 2.5 m (the car rises 4 m on screen).
            for (i, m) in sprites.iter().enumerate() {
                draw_car(m, 3.0 + i as f32 * 5.0, 4.0, 0.0, true, i);
                draw_car(m, 20.0 + i as f32 * 6.0, 6.0, 2.5, true, i);
            }
            // Row 2: procedural archetypes on the ground. Row 3: lifted 2.5 m.
            for (i, m) in archetypes.iter().enumerate() {
                draw_car(m, 3.0 + i as f32 * 5.5, 11.0, 0.0, false, i);
                draw_car(m, 3.0 + i as f32 * 5.5, 20.0, 2.5, false, i);
            }
            set_default_camera();
            if frame == 3 {
                get_screen_data().export_png(&format!("{}/{}.png", dir, name));
            }
            next_frame().await;
        }
    }
}
