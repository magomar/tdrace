//! Spec 089: PackedGravel drives like Dirt, DeepGravel slows a car but never traps it.
//! See `specs/089_packed_and_deep_gravel_surfaces_and_asphaltcircuit_gravel_traps.md`.

use tdrace_app::catalog::get_models_for_module_and_tier;
use tdrace_app::module::classic::ClassicGameModule;
use tdrace_core::track::TrackWaypoint;
use tdrace_core::{Car, CarConfig, CarControls, SurfaceType, Vec2};

const DT: f32 = 1.0 / 60.0;

/// Classic GT, kart and rally cars, and the first car of tiers 1 and 5 of the GT, kart and rally modules.
fn reference_cars() -> Vec<(String, CarConfig)> {
    let mut cars = vec![
        ("classic_gt".to_string(), ClassicGameModule::car_classic_gt()),
        ("classic_kart".to_string(), ClassicGameModule::car_classic_kart()),
        ("classic_rally".to_string(), ClassicGameModule::car_classic_rally()),
    ];
    for module in ["gt", "kart", "rally"] {
        for tier in [1u8, 5] {
            let model = get_models_for_module_and_tier(module, tier)
                .into_iter()
                .next()
                .unwrap_or_else(|| panic!("no {} tier {} car", module, tier));
            cars.push((model.id.to_string(), model.to_car_config()));
        }
    }
    cars
}

/// Metres the car covers in 15 s from a stop, with the best of a few fixed throttle settings.
fn escape_distance(cfg: &CarConfig, surface: SurfaceType) -> f32 {
    [0.1f32, 0.15, 0.25, 0.4, 0.6, 0.8, 1.0]
        .iter()
        .map(|&throttle| {
            let mut car = Car::new(cfg.clone());
            let start = car.state.position;
            let controls = CarControls { throttle, ..CarControls::accelerate() };
            for _ in 0..900 {
                car.step(&controls, surface, DT);
            }
            (car.state.position - start).length()
        })
        .fold(0.0, f32::max)
}

/// Metres the car coasts to a stop on `surface` (no throttle, no brake) after reaching 100 km/h on Asphalt.
fn coast_distance(cfg: &CarConfig, surface: SurfaceType) -> f32 {
    let mut car = Car::new(cfg.clone());
    for _ in 0..3000 {
        car.step(&CarControls::accelerate(), SurfaceType::Asphalt, DT);
        if car.speed_kmh() >= 100.0 {
            break;
        }
    }
    let start = car.state.position;
    let coast = CarControls { throttle: 0.0, ..CarControls::accelerate() };
    for _ in 0..3000 {
        if car.speed_kmh() <= 2.0 {
            break;
        }
        car.step(&coast, surface, DT);
    }
    (car.state.position - start).length()
}

/// Scenario: PackedGravel drives like Dirt, a bit more slippery
#[test]
fn test_packed_gravel_grip_is_85_to_95_percent_of_dirt() {
    for (name, cfg) in reference_cars() {
        let car = Car::new(cfg);
        for wheel in &car.state.wheel_assemblies {
            let affinity = &wheel.config.compound.surface_affinity;
            let grip = |s: SurfaceType| s.friction_coefficient() * affinity.get(s);
            let ratio = grip(SurfaceType::PackedGravel) / grip(SurfaceType::Dirt);
            assert!(
                (0.85..=0.95).contains(&ratio),
                "{}: PackedGravel grip is {:.2} of Dirt grip, expected 0.85-0.95",
                name,
                ratio
            );
        }
    }
}

/// Scenario: DeepGravel slows a car that leaves the road
#[test]
fn test_deep_gravel_stops_a_coasting_car_in_60_percent_of_its_asphalt_distance() {
    for (name, cfg) in reference_cars() {
        let asphalt = coast_distance(&cfg, SurfaceType::Asphalt);
        let gravel = coast_distance(&cfg, SurfaceType::DeepGravel);
        assert!(
            gravel <= 0.60 * asphalt,
            "{}: coasts {:.0} m on DeepGravel, {:.0} m on Asphalt (limit 60 %)",
            name,
            gravel,
            asphalt
        );
    }
}

/// Scenario: DeepGravel never traps a car
#[test]
fn test_deep_gravel_never_traps_a_stopped_car() {
    for (name, cfg) in reference_cars() {
        let distance = escape_distance(&cfg, SurfaceType::DeepGravel);
        assert!(distance >= 10.0, "{}: covers only {:.1} m in 15 s from a stop on DeepGravel", name, distance);
    }
}

/// Scenario: Old circuits still load
#[test]
fn test_old_gravel_name_loads_as_packed_gravel_and_saves_with_the_new_name() {
    let waypoint = TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0).with_surface(SurfaceType::PackedGravel);
    let old_json = serde_json::to_string(&waypoint).unwrap().replace("\"PackedGravel\"", "\"Gravel\"");
    assert!(old_json.contains("\"Gravel\""));

    let loaded: TrackWaypoint = serde_json::from_str(&old_json).unwrap();
    assert_eq!(loaded.surface, Some(SurfaceType::PackedGravel));

    let saved = serde_json::to_string(&loaded).unwrap();
    assert!(saved.contains("\"PackedGravel\"") && !saved.contains("\"Gravel\""), "saved: {}", saved);
}

/// Every string value in `value`, with the key it is stored under.
fn surface_strings<'a>(value: &'a serde_json::Value, key: &'a str, out: &mut Vec<(&'a str, &'a str)>) {
    match value {
        serde_json::Value::String(s) => out.push((key, s)),
        serde_json::Value::Array(items) => items.iter().for_each(|v| surface_strings(v, key, out)),
        serde_json::Value::Object(map) => map.iter().for_each(|(k, v)| surface_strings(v, k, out)),
        _ => {}
    }
}

/// Scenario: Official circuits use the new surfaces
#[test]
fn test_official_circuits_use_packed_and_deep_gravel() {
    let tracks = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tracks");
    let mut offroad_deep_sand = 0;
    let mut checked = 0;
    for module in std::fs::read_dir(&tracks).expect("tracks/ submodule is missing") {
        let module = module.unwrap().path();
        if !module.is_dir() {
            continue;
        }
        let module_id = module.file_name().unwrap().to_string_lossy().to_string();
        for file in std::fs::read_dir(&module).unwrap() {
            let path = file.unwrap().path();
            let id = path.file_stem().unwrap().to_string_lossy().to_string();
            if path.extension().map_or(true, |e| e != "json") || id == "MANIFEST" {
                continue;
            }
            let json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let mut strings = Vec::new();
            surface_strings(&json, "", &mut strings);
            let asphalt_circuit = matches!(module_id.as_str(), "gt" | "rally" | "kart")
                || (module_id == "classic" && ["gt_", "rx_", "kart_"].iter().any(|p| id.starts_with(p)));
            for (key, value) in strings {
                assert_ne!(value, "Gravel", "{}/{}: '{}' still uses the old name Gravel", module_id, id, key);
                if asphalt_circuit {
                    assert_ne!(value, "DeepSand", "{}/{}: '{}' uses DeepSand on an asphalt circuit", module_id, id, key);
                    if key.ends_with("_runoff_surface") {
                        assert_ne!(value, "PackedGravel", "{}/{}: gravel run-off must be DeepGravel", module_id, id);
                    }
                } else if value == "DeepSand" && (module_id == "extreme_offroad" || id.starts_with("at_")) {
                    offroad_deep_sand += 1;
                }
            }
            checked += 1;
        }
    }
    assert!(checked > 100, "only {} circuit files found", checked);
    assert!(offroad_deep_sand > 0, "off-road circuits must still use DeepSand");
}
