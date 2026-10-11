//! Spec 092: the racing number plate on the car, the nameplate label and the grid and card badges.

use glam::Vec2;
use macroquad::color::Color;
use tdrace_app::catalog::get_all_models;
use tdrace_app::render::car::{
    color_luminance, number_em_size, number_glyph_colors, number_plate, CarNumberItem, NUMBER_DIGIT_ADVANCE_EM,
    NUMBER_DIGIT_HEIGHT_EM,
};
use tdrace_app::render::color::Palette;
use tdrace_app::render::VehicleNameplateItem;
use tdrace_app::ui::menu::CarChoice;
use tdrace_core::physics::config::{CarConfig, ChassisSkeleton};
use tdrace_core::Car;

const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);

fn every_car_config() -> Vec<(String, CarConfig)> {
    let mut cars: Vec<(String, CarConfig)> = get_all_models().iter().map(|m| (m.id.to_string(), m.to_car_config())).collect();
    for choice in CarChoice::ALL {
        cars.push((format!("{choice:?}"), choice.config()));
    }
    cars
}

// ---------------------------------------------------------------------------
// Contrast rule
// ---------------------------------------------------------------------------

#[test]
fn a_dark_body_gets_white_glyphs_with_a_black_outline() {
    for body in [BLACK, Color::new(0.1, 0.1, 0.4, 1.0), Color::new(0.6, 0.0, 0.0, 1.0), Color::new(0.0, 0.3, 0.0, 1.0)] {
        assert!(color_luminance(body) < 0.5, "{body:?}");
        assert_eq!(number_glyph_colors(body), (Palette::WHITE, BLACK), "{body:?}");
    }
}

#[test]
fn a_light_body_gets_black_glyphs_with_a_white_outline() {
    for body in [Palette::WHITE, Color::new(1.0, 0.9, 0.1, 1.0), Color::new(0.9, 0.9, 0.2, 1.0), Color::new(0.5, 0.8, 0.5, 1.0)] {
        assert!(color_luminance(body) >= 0.5, "{body:?}");
        assert_eq!(number_glyph_colors(body), (BLACK, Palette::WHITE), "{body:?}");
    }
}

#[test]
fn the_contrast_switch_is_exactly_at_half_luminance() {
    let grey = |v: f32| Color::new(v, v, v, 1.0);
    assert_eq!(number_glyph_colors(grey(0.499)).0, Palette::WHITE);
    assert_eq!(number_glyph_colors(grey(0.5)).0, BLACK);
    assert!((color_luminance(grey(0.5)) - 0.5).abs() < 1e-4);
    // Green weighs the most, blue the least.
    assert!(color_luminance(Color::new(0.0, 1.0, 0.0, 1.0)) > color_luminance(Color::new(1.0, 0.0, 0.0, 1.0)));
    assert!(color_luminance(Color::new(1.0, 0.0, 0.0, 1.0)) > color_luminance(Color::new(0.0, 0.0, 1.0, 1.0)));
}

#[test]
fn every_livery_in_the_game_gets_a_readable_pair() {
    for (primary, _, _) in Palette::CAR_COLORS.iter() {
        let (glyph, outline) = number_glyph_colors(*primary);
        assert!((color_luminance(glyph) - color_luminance(outline)).abs() > 0.99, "glyph and outline are opposite");
        assert!((color_luminance(glyph) - color_luminance(*primary)).abs() > 0.3, "glyph stands out on {primary:?}");
    }
}

// ---------------------------------------------------------------------------
// Anchor and size
// ---------------------------------------------------------------------------

#[test]
fn the_plate_is_centred_on_the_cabin_of_the_skeleton() {
    // Axles at +1.3 and -1.3 m from the CG, cabin from 0.4 m behind the front axle to 0.3 m ahead of the rear axle.
    let skeleton = ChassisSkeleton::new(0.8, 0.9, 1.8, -0.4, 0.3, 0.65, 0.7, 0.05);
    let plate = number_plate(&skeleton, 2.6, 1.3, 1.3);
    let cabin_centre_from_cg = ((1.3 - 0.4) + (-1.3 + 0.3)) * 0.5; // -0.05
    let body_centre_from_cg = skeleton.geometric_center_offset_from_cg(1.3, 1.3);
    assert!((plate.forward_offset - (cabin_centre_from_cg - body_centre_from_cg)).abs() < 1e-5);
    assert!((plate.half_len - 0.95).abs() < 1e-5, "as long as the cabin (1.9 m)");
    assert!((plate.half_width - 0.63).abs() < 1e-5, "70% of the body width");
}

#[test]
fn the_digits_fit_inside_the_plate() {
    let plate = number_plate(&ChassisSkeleton::default(), 2.6, 1.3, 1.3);
    for digits in [1, 2] {
        let em = number_em_size(&plate, digits);
        assert!(em > 0.0);
        assert!(digits as f32 * NUMBER_DIGIT_ADVANCE_EM * em <= plate.half_len * 2.0 + 1e-4, "{digits} digits are too long");
        assert!(NUMBER_DIGIT_HEIGHT_EM * em <= plate.half_width * 2.0 + 1e-4, "{digits} digits are too tall");
    }
    assert!(number_em_size(&plate, 1) >= number_em_size(&plate, 2), "one digit is never smaller than two");
}

#[test]
fn a_skeleton_with_crossed_cabin_edges_still_gets_a_plate_on_the_body() {
    let skeleton = ChassisSkeleton::new(0.8, 0.9, 1.8, 0.9, -0.9, 0.65, 0.7, 0.05);
    let plate = number_plate(&skeleton, 2.6, 1.3, 1.3);
    assert!(plate.half_len > 0.0);
    assert!(plate.forward_offset.abs() + plate.half_len <= skeleton.half_length(2.6) + 1e-4);
}

/// Scenario: Number plate is legible and anchored to the car (every car in the game)
#[test]
fn the_plate_is_inside_the_body_outline_of_every_car() {
    let cars = every_car_config();
    assert!(cars.len() > 50, "catalog and presets are covered ({})", cars.len());
    for (name, cfg) in cars {
        let plate = number_plate(&cfg.chassis, cfg.wheelbase, cfg.cg_to_front, cfg.cg_to_rear);
        let body_half_len = cfg.chassis.half_length(cfg.wheelbase);
        let body_half_w = cfg.chassis.half_width();
        assert!(plate.half_len > 0.0 && plate.half_width > 0.0, "{name}: empty plate {plate:?}");
        assert!(plate.forward_offset.abs() + plate.half_len <= body_half_len + 1e-4, "{name}: plate past the end of the body {plate:?}");
        assert!(plate.half_width <= body_half_w + 1e-4, "{name}: plate wider than the body");
        for digits in [1, 2] {
            let em = number_em_size(&plate, digits);
            assert!(em > 0.0 && em.is_finite(), "{name}: bad size {em}");
            let half_text_len = digits as f32 * NUMBER_DIGIT_ADVANCE_EM * em * 0.5;
            let half_text_h = NUMBER_DIGIT_HEIGHT_EM * em * 0.5;
            assert!(plate.forward_offset.abs() + half_text_len <= body_half_len, "{name}: digits past the end of the body");
            assert!(half_text_h <= body_half_w, "{name}: digits wider than the body");
        }
    }
}

#[test]
fn karts_get_digits_big_enough_to_read() {
    let kart = CarChoice::Kart.config();
    let plate = number_plate(&kart.chassis, kart.wheelbase, kart.cg_to_front, kart.cg_to_rear);
    let em = number_em_size(&plate, 2);
    assert!(em * NUMBER_DIGIT_HEIGHT_EM >= 0.25, "two digits on a kart are {} m high", em * NUMBER_DIGIT_HEIGHT_EM);
}

// ---------------------------------------------------------------------------
// On the car
// ---------------------------------------------------------------------------

#[test]
fn only_valid_numbers_get_a_plate() {
    let car = Car::new(CarChoice::Kart.config());
    for bad in [0u8, 100, 150, 255] {
        assert!(CarNumberItem::for_car(&car, bad, Palette::WHITE).is_none(), "{bad}");
    }
    for good in [1u8, 7, 10, 99] {
        assert!(CarNumberItem::for_car(&car, good, Palette::WHITE).is_some(), "{good}");
    }
}

#[test]
fn the_plate_turns_with_the_car_and_stays_on_its_centre_line() {
    let cfg = CarChoice::SportsCar.config();
    let at_origin = |angle: f32| CarNumberItem::for_car(&Car::new(cfg).with_pose(Vec2::ZERO, angle), 17, BLACK).unwrap();
    let east = at_origin(0.0);
    let north = at_origin(std::f32::consts::FRAC_PI_2);
    assert!((east.center.length() - north.center.length()).abs() < 1e-4, "same distance from the car centre");
    assert!(east.center.y.abs() < 1e-4, "heading east: the plate is on the x axis");
    assert!(north.center.x.abs() < 1e-3, "heading north: the plate is on the y axis");
    assert_eq!(east.angle, 0.0);
    assert!((north.angle - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    assert_eq!(east.em, north.em);
}

#[test]
fn a_car_in_the_air_carries_its_number_up_with_it() {
    let cfg = CarChoice::SportsCar.config();
    let ground = Car::new(cfg).with_pose(Vec2::ZERO, 0.0);
    let mut air = Car::new(cfg).with_pose(Vec2::ZERO, 0.0);
    air.state.elevation = 2.0;
    let g = CarNumberItem::for_car(&ground, 17, BLACK).unwrap();
    let a = CarNumberItem::for_car(&air, 17, BLACK).unwrap();
    assert!((a.center.y - g.center.y + 2.0 * 1.6).abs() < 0.1, "lifted as far as the body: {} vs {}", a.center.y, g.center.y);
    assert!(a.em > g.em, "the lifted body is drawn a little bigger");
}

// ---------------------------------------------------------------------------
// Nameplate
// ---------------------------------------------------------------------------

fn nameplate(name: &str, race_number: u8) -> VehicleNameplateItem<'_> {
    VehicleNameplateItem {
        car_idx: 1,
        name,
        tier_label: None,
        accent_color: Palette::WHITE,
        position: Vec2::ZERO,
        elevation: 0.0,
        distance_to_player: 10.0,
        race_number,
    }
}

#[test]
fn the_nameplate_shows_the_number_before_the_name() {
    assert_eq!(nameplate("Vortex", 17).label(), "#17 Vortex");
    assert_eq!(nameplate("Vortex", 7).label(), "#7 Vortex");
    assert_eq!(nameplate("Vortex", 99).label(), "#99 Vortex");
}

#[test]
fn a_nameplate_without_a_valid_number_shows_only_the_name() {
    for bad in [0u8, 100, 200] {
        assert_eq!(nameplate("Vortex", bad).label(), "Vortex", "{bad}");
    }
}

#[test]
fn the_badge_label_is_empty_without_a_number() {
    assert_eq!(tdrace_app::ui::race_number_label(17), "#17");
    assert_eq!(tdrace_app::ui::race_number_label(0), "");
    assert_eq!(tdrace_app::ui::race_number_label(100), "");
}
