use tdrace_core::catalog;
use tdrace_core::surface::SurfaceType;
use tdrace_core::track::{validate_track, ValidationSeverity};

#[test]
fn test_seventeen_autocross_circuits_embedded_in_catalog() {
    let circuits: Vec<_> = catalog::module_circuits("autocross").collect();
    assert_eq!(
        circuits.len(),
        17,
        "Expected exactly 17 circuits embedded for 'autocross' module"
    );

    let expected_slugs = [
        "nova_paka_ax",
        "prerov_ax",
        "humpolec_ax",
        "matschenberg_ax",
        "seelow_ax",
        "schluechtern_ax",
        "uelzen_ax",
        "st_georges_ax",
        "faleyras_ax",
        "st_junien_ax",
        "bazaigues_ax",
        "maggiora_ax",
        "musa_ax",
        "vilkyciai_ax",
        "arteixo_ax",
        "carballo_ax",
        "castelo_branco_ax",
    ];

    for (c, expected_id) in circuits.iter().zip(expected_slugs) {
        assert_eq!(c.id, expected_id);
        assert_eq!(c.module, "autocross");
        assert_eq!(c.category_label, "FIA Autocross");
        assert!(!c.country_code.is_empty());
        assert!((4..=6).contains(&c.default_laps));

        let track = c.load().expect("Embedded circuit must deserialize cleanly");
        assert_eq!(track.car_category, tdrace_core::CarCategory::Autocross);
        assert_eq!(track.module_id.as_deref(), Some("autocross"));
        assert_eq!(track.scale, "1:1");
        assert_eq!(track.grid_positions.len(), 8, "Autocross launch chutes seat 8 vehicles (spec 103)");
        assert!(track.checkpoints.len() >= 4, "Must have valid checkpoints");

        // Verify dominant surface is Dirt and runoff is DeepGravel
        let total_samples = track.spline.samples.len();
        let loose_samples = track
            .spline
            .samples
            .iter()
            .filter(|s| s.surface == SurfaceType::Dirt)
            .count();
        assert!(
            loose_samples as f32 / total_samples as f32 >= 0.90,
            "Circuit {} dominant surface must be pure Dirt (got {}/{} samples)",
            c.id,
            loose_samples,
            total_samples
        );
        for (idx, wp) in track.spline.waypoints.iter().enumerate() {
            assert_eq!(wp.left_runoff_surface, Some(SurfaceType::DeepGravel), "Circuit {} wp {} left runoff", c.id, idx);
            assert_eq!(wp.right_runoff_surface, Some(SurfaceType::DeepGravel), "Circuit {} wp {} right runoff", c.id, idx);
        }
        for (idx, s) in track.spline.samples.iter().enumerate() {
            assert_eq!(s.left_runoff_surface, Some(SurfaceType::DeepGravel), "Circuit {} sample {} left runoff", c.id, idx);
            assert_eq!(s.right_runoff_surface, Some(SurfaceType::DeepGravel), "Circuit {} sample {} right runoff", c.id, idx);
        }
        if let Some(net) = &track.network {
            for seg in &net.segments {
                for (idx, wp) in seg.waypoints.iter().enumerate() {
                    assert_eq!(wp.left_runoff_surface, Some(SurfaceType::DeepGravel), "Circuit {} seg {} wp {} left runoff", c.id, seg.id.0, idx);
                    assert_eq!(wp.right_runoff_surface, Some(SurfaceType::DeepGravel), "Circuit {} seg {} wp {} right runoff", c.id, seg.id.0, idx);
                }
                for (idx, s) in seg.samples.iter().enumerate() {
                    assert_eq!(s.left_runoff_surface, Some(SurfaceType::DeepGravel), "Circuit {} seg {} sample {} left runoff", c.id, seg.id.0, idx);
                    assert_eq!(s.right_runoff_surface, Some(SurfaceType::DeepGravel), "Circuit {} seg {} sample {} right runoff", c.id, seg.id.0, idx);
                }
            }
        }

        // Verify OSM provenance URL
        assert!(
            track.osm_url.as_ref().is_some_and(|u| u.starts_with("https://www.openstreetmap.org/way/")),
            "Circuit {} missing valid OSM URL: {:?}",
            c.id,
            track.osm_url
        );

        // Verify zero validation errors
        let errors: Vec<_> = validate_track(&track)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .collect();
        assert!(errors.is_empty(), "Circuit {} has validation errors: {:?}", c.id, errors);
    }
}

#[test]
fn test_autocross_circuits_width_jump_and_whoop_moguls() {
    use tdrace_core::collision::wall::resolve_all_wall_collisions;
    use tdrace_core::physics::car::{Car, CarControls};
    use tdrace_core::physics::config::CarConfig;
    use tdrace_core::track::geometry::{JumpRampCarExt, RampKind};

    for c in catalog::module_circuits("autocross") {
        let track = c.load().expect("Embedded circuit must deserialize cleanly");

        // 1. Minimum spline width >= 10.0m
        for (idx, wp) in track.spline.waypoints.iter().enumerate() {
            assert!(
                wp.width >= 10.0,
                "Circuit {} waypoint {} width {:.1}m < 10.0m",
                c.id,
                idx,
                wp.width
            );
        }

        // 2. Exactly 1 Jump (RampKind::Ramp) and 1 Whoop Moguls (RampKind::Whoops)
        assert_eq!(
            track.geometry.jump_ramps.len(),
            2,
            "Circuit {} must have 1 jump and 1 whoop moguls segment",
            c.id
        );
        let jumps: Vec<_> = track
            .geometry
            .jump_ramps
            .iter()
            .filter(|r| r.kind == RampKind::Ramp)
            .collect();
        let whoops: Vec<_> = track
            .geometry
            .jump_ramps
            .iter()
            .filter(|r| r.kind == RampKind::Whoops)
            .collect();
        assert_eq!(jumps.len(), 1, "Circuit {} must have 1 RampKind::Ramp", c.id);
        assert_eq!(whoops.len(), 1, "Circuit {} must have 1 RampKind::Whoops", c.id);

        let jump = jumps[0];
        let whoop = whoops[0];
        assert_eq!(jump.surface, SurfaceType::Dirt);
        assert_eq!(whoop.surface, SurfaceType::Dirt);
        assert!(whoop.whoop_count() >= 4, "Circuit {} whoops should have >= 4 moguls", c.id);

        // 3. Verify jump is on a straight segment and passes physical landing containment
        for &approach_speed in &[30.0f32, 42.0f32] {
            let mut car = Car::new(CarConfig::rally_car()).with_pose(jump.shape.center(), jump.angle());
            car.state.velocity = jump.direction * approach_speed;

            assert!(car.try_trigger_jump_ramp(jump));
            let ctrl = CarControls::accelerate();
            for _ in 0..120 {
                car.step(&ctrl, SurfaceType::Dirt, 1.0 / 60.0);
                if car.state.just_landed {
                    break;
                }
            }
            assert!(car.state.just_landed, "Car at {:.1} m/s did not land on {}", approach_speed, c.id);
            let hit_inner = resolve_all_wall_collisions(&mut car, &track.geometry.inner_walls, &[]);
            let hit_outer = resolve_all_wall_collisions(&mut car, &track.geometry.outer_walls, &[]);
            assert!(
                hit_inner.is_empty() && hit_outer.is_empty(),
                "Circuit {} jump landed into walls at {:.1} m/s",
                c.id,
                approach_speed
            );
        }
    }
}

