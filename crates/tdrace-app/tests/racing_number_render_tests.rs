//! Spec 092: the racing number plate on the car, the nameplate label and the grid and card badges.
//! Spec 110: where each kind of car carries the number, and its size.

use glam::Vec2;
use macroquad::color::Color;
use tdrace_app::catalog::get_all_models;
use tdrace_app::module::VehicleVisualType;
use tdrace_app::render::car::{
    color_luminance, number_em_size, number_glyph_colors, number_plate, number_plate_zone, CarNumberItem, PlateZone,
    NUMBER_DIGIT_ADVANCE_EM, NUMBER_DIGIT_HEIGHT_EM, NUMBER_MAX_HEIGHT_FRACTION,
};
use tdrace_app::render::color::Palette;
use tdrace_app::render::VehicleNameplateItem;
use tdrace_app::ui::menu::CarChoice;
use tdrace_core::physics::config::{CarConfig, ChassisSkeleton};
use tdrace_core::Car;

const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);

fn every_car() -> Vec<(String, CarConfig, VehicleVisualType)> {
    let mut cars: Vec<(String, CarConfig, VehicleVisualType)> =
        get_all_models().iter().map(|m| (m.id.to_string(), m.to_car_config(), m.visual_type)).collect();
    for choice in CarChoice::ALL {
        cars.push((format!("{choice:?}"), choice.config(), choice.visual_type()));
    }
    cars
}

const RALLY: VehicleVisualType = VehicleVisualType::RallyHatch { roof_scoop: true, mudflaps: true, large_wing: true };
const KART: VehicleVisualType = VehicleVisualType::GoKart { exposed_driver: true, side_bumpers: true };

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
// Placement by car type (spec 110)
// ---------------------------------------------------------------------------

#[test]
fn each_kind_of_car_has_its_zone() {
    assert_eq!(number_plate_zone(KART), None, "karts show no number on the car");
    let sand_rail = VehicleVisualType::SandRail { lightbar: true, whip_antenna: true, paddle_tires: true };
    let gt = VehicleVisualType::TouringGT { widebody: true, gt_wing: true, diffuser: true };
    let open_wheel = VehicleVisualType::OpenWheel { front_wing_span: 1.0, rear_wing_height: 1.0, halo: true };
    let stock = VehicleVisualType::StockCar { tall_wing: true, roof_fins: true, window_net: true };
    assert_eq!(number_plate_zone(sand_rail), Some(PlateZone::Front), "buggies");
    assert_eq!(number_plate_zone(gt), Some(PlateZone::Front), "GT: bonnet");
    assert_eq!(number_plate_zone(open_wheel), Some(PlateZone::Front), "open-wheel: nose");
    assert_eq!(number_plate_zone(stock), Some(PlateZone::Roof), "stock cars");
    assert_eq!(number_plate_zone(RALLY), Some(PlateZone::Roof), "rally cars");
}

// Axles at +1.3 and -1.3 m from the CG, cabin from 0.4 m behind the front axle to 0.3 m ahead of the rear axle,
// 0.8 m front overhang, 1.8 m wide.
fn skeleton() -> ChassisSkeleton {
    ChassisSkeleton::new(0.8, 0.9, 1.8, -0.4, 0.3, 0.65, 0.7, 0.05)
}

#[test]
fn the_roof_plate_is_centred_on_the_cabin() {
    let s = skeleton();
    let plate = number_plate(&s, 2.6, 1.3, 1.3, PlateZone::Roof);
    let cabin_centre_from_cg = ((1.3 - 0.4) + (-1.3 + 0.3)) * 0.5; // -0.05
    let body_centre_from_cg = s.geometric_center_offset_from_cg(1.3, 1.3);
    assert!((plate.forward_offset - (cabin_centre_from_cg - body_centre_from_cg)).abs() < 1e-5);
    assert!((plate.half_len - 0.95).abs() < 1e-5, "as long as the cabin (1.9 m)");
}

#[test]
fn the_front_plate_is_between_the_cabin_and_the_nose() {
    let s = skeleton();
    let plate = number_plate(&s, 2.6, 1.3, 1.3, PlateZone::Front);
    let body_centre_from_cg = s.geometric_center_offset_from_cg(1.3, 1.3);
    let zone_centre_from_cg = ((1.3 - 0.4) + (1.3 + 0.8)) * 0.5; // 1.5
    assert!((plate.forward_offset - (zone_centre_from_cg - body_centre_from_cg)).abs() < 1e-5);
    assert!((plate.half_len - 0.6).abs() < 1e-5, "as long as the zone (1.2 m)");
}

#[test]
fn the_digits_are_at_most_35_percent_of_the_body_width_high() {
    let s = skeleton();
    for zone in [PlateZone::Front, PlateZone::Roof] {
        let plate = number_plate(&s, 2.6, 1.3, 1.3, zone);
        for digits in [1, 2] {
            let height = NUMBER_DIGIT_HEIGHT_EM * number_em_size(&plate, digits);
            assert!(height <= NUMBER_MAX_HEIGHT_FRACTION * s.body_width + 1e-4, "{zone:?} {digits}: {height} m");
            assert!(digits as f32 * NUMBER_DIGIT_ADVANCE_EM * number_em_size(&plate, digits) <= plate.half_len * 2.0 + 1e-4);
        }
    }
    assert_eq!(NUMBER_MAX_HEIGHT_FRACTION, 0.35);
}

#[test]
fn a_skeleton_with_crossed_cabin_edges_still_gets_plates_on_the_body() {
    let s = ChassisSkeleton::new(0.8, 0.9, 1.8, 0.9, -0.9, 0.65, 0.7, 0.05);
    for zone in [PlateZone::Front, PlateZone::Roof] {
        let plate = number_plate(&s, 2.6, 1.3, 1.3, zone);
        assert!(plate.half_len > 0.0, "{zone:?}");
        assert!(plate.forward_offset.abs() + plate.half_len <= s.half_length(2.6) + 1e-4, "{zone:?}");
    }
    let front = number_plate(&s, 2.6, 1.3, 1.3, PlateZone::Front);
    assert!(front.forward_offset > 0.0, "the fallback front plate is still at the front");
}

/// Scenarios: buggies, GT cars and open-wheelers at the front; stock cars and rally cars on the roof; smaller numbers
#[test]
fn every_car_in_the_game_gets_its_plate_inside_the_body() {
    let cars = every_car();
    assert!(cars.len() > 50, "catalog and presets are covered ({})", cars.len());
    let mut fronts = 0;
    let mut roofs = 0;
    for (name, cfg, visual) in cars {
        let Some(zone) = number_plate_zone(visual) else {
            continue;
        };
        let (lf, lr, s) = (cfg.cg_to_front, cfg.cg_to_rear, cfg.chassis);
        let plate = number_plate(&s, cfg.wheelbase, lf, lr, zone);
        let geom = s.geometric_center_offset_from_cg(lf, lr);
        let centre_from_cg = plate.forward_offset + geom;
        let cabin_front = lf + s.cabin_start_offset;
        let cabin_rear = -lr + s.cabin_end_offset;
        let body_half_len = s.half_length(cfg.wheelbase);
        assert!(plate.half_len > 0.0 && plate.half_width > 0.0, "{name}: empty plate {plate:?}");
        assert!(plate.forward_offset.abs() + plate.half_len <= body_half_len + 1e-4, "{name}: plate past the end of the body");
        match zone {
            PlateZone::Front => {
                fronts += 1;
                assert!(centre_from_cg > (cabin_front + cabin_rear) * 0.5, "{name}: front plate not ahead of the cabin centre");
                assert!(centre_from_cg <= lf + s.front_overhang + 1e-4, "{name}: front plate past the nose");
                if cabin_front - cabin_rear > 0.1 && lf + s.front_overhang - cabin_front > 0.1 {
                    assert!(centre_from_cg >= cabin_front - 1e-4, "{name}: front plate behind the cabin front");
                }
            }
            PlateZone::Roof => {
                roofs += 1;
                assert!((centre_from_cg - (cabin_front + cabin_rear) * 0.5).abs() < 1e-4, "{name}: roof plate not on the cabin");
            }
        }
        for digits in [1, 2] {
            let em = number_em_size(&plate, digits);
            assert!(em > 0.0 && em.is_finite(), "{name}: bad size {em}");
            let half_text_len = digits as f32 * NUMBER_DIGIT_ADVANCE_EM * em * 0.5;
            assert!(plate.forward_offset.abs() + half_text_len <= body_half_len, "{name}: digits past the end of the body");
            assert!(NUMBER_DIGIT_HEIGHT_EM * em <= NUMBER_MAX_HEIGHT_FRACTION * s.body_width + 1e-4, "{name}: digits too tall");
        }
    }
    assert!(fronts > 10 && roofs > 10, "both zones are used ({fronts} front, {roofs} roof)");
}

// ---------------------------------------------------------------------------
// On the car
// ---------------------------------------------------------------------------

/// Scenario: Karts show no number on the car
#[test]
fn a_kart_gets_no_plate() {
    let kart = Car::new(CarChoice::Kart.config());
    for number in [1u8, 7, 17, 99] {
        assert!(CarNumberItem::for_car(&kart, number, Palette::WHITE, KART).is_none(), "{number}");
    }
}

#[test]
fn only_valid_numbers_get_a_plate() {
    let car = Car::new(CarChoice::RallyCar.config());
    for bad in [0u8, 100, 150, 255] {
        assert!(CarNumberItem::for_car(&car, bad, Palette::WHITE, RALLY).is_none(), "{bad}");
    }
    for good in [1u8, 7, 10, 99] {
        assert!(CarNumberItem::for_car(&car, good, Palette::WHITE, RALLY).is_some(), "{good}");
    }
}

#[test]
fn the_plate_turns_with_the_car_and_stays_on_its_centre_line() {
    let cfg = CarChoice::SportsCar.config();
    let visual = CarChoice::SportsCar.visual_type();
    let at_origin = |angle: f32| CarNumberItem::for_car(&Car::new(cfg).with_pose(Vec2::ZERO, angle), 17, BLACK, visual).unwrap();
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
    let cfg = CarChoice::RallyCar.config();
    let ground = Car::new(cfg).with_pose(Vec2::ZERO, 0.0);
    let mut air = Car::new(cfg).with_pose(Vec2::ZERO, 0.0);
    air.state.elevation = 2.0;
    let g = CarNumberItem::for_car(&ground, 17, BLACK, RALLY).unwrap();
    let a = CarNumberItem::for_car(&air, 17, BLACK, RALLY).unwrap();
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
