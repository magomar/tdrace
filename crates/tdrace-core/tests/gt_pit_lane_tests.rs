//! Spec 077: Procedural GT Circuit Pit Lanes from OpenStreetMap Survey Data.
//! Automated test suite verifying physical pit lane geometries, gates, stalls, and validation rules.

use std::fs;
use std::path::Path;
use tdrace_core::track::Track;
use wheelbase::SurfaceType;

const ALL_GT_CIRCUITS: &[&str] = &[
    "monza",
    "spa",
    "catalunya",
    "silverstone",
    "red_bull_ring",
    "nurburgring_gp",
    "interlagos",
    "le_mans_sarthe",
    "bathurst",
    "portimao_gp",
    "madring",
    "suzuka",
    "cota",
    "montreal",
    "marina_bay",
    "monaco",
    "bahrain",
    "zandvoort",
];

fn load_track_from_tracks_dir(slug: &str) -> Track {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir)
        .join("../../tracks/gt")
        .join(format!("{}.json", slug));
    let content = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("Failed to read {}: {}", path.display(), e);
    });
    Track::from_json(&content).unwrap_or_else(|e| {
        panic!("Failed to parse track JSON {}: {}", path.display(), e);
    })
}

#[test]
fn test_all_18_gt_circuits_have_valid_baked_pit_lanes() {
    for &slug in ALL_GT_CIRCUITS {
        let track = load_track_from_tracks_dir(slug);
        let lane = track.pit_lane.as_ref().unwrap_or_else(|| {
            panic!("Circuit '{}' must declare a physical pit lane", slug);
        });

        // 1. Minimum length and samples
        let pit_length = lane.spline.total_length();
        assert!(
            pit_length >= 80.0,
            "Circuit '{}': pit lane length ({:.1}m) must be >= 80.0m",
            slug,
            pit_length
        );
        assert!(
            !lane.spline.samples.is_empty(),
            "Circuit '{}': pit lane spline must have baked samples",
            slug
        );

        // 2. Team pit stalls
        assert!(
            lane.pit_boxes.len() >= 4 && lane.pit_boxes.len() <= 10,
            "Circuit '{}': team stalls count ({}) must be between 4 and 10",
            slug,
            lane.pit_boxes.len()
        );
        for (i, box_slot) in lane.pit_boxes.iter().enumerate() {
            assert!(
                box_slot.stop_radius >= 2.0 && box_slot.stop_radius <= 5.0,
                "Circuit '{}' stall {}: stop radius ({:.1}m) must be valid",
                slug,
                i,
                box_slot.stop_radius
            );
            assert!(
                box_slot.direction.length() > 0.9,
                "Circuit '{}' stall {}: stall direction must be unit vector",
                slug,
                i
            );
        }

        // 3. Road width & speed governor
        assert!(
            lane.road_width >= 4.0 && lane.road_width <= 8.0,
            "Circuit '{}': road width ({:.1}m) must be realistic (4-8m)",
            slug,
            lane.road_width
        );
        assert!(
            (lane.speed_limit - 16.67).abs() < 0.1,
            "Circuit '{}': speed limit ({:.2} m/s) must be 16.67 m/s (60 km/h)",
            slug,
            lane.speed_limit
        );

        // 4. Timing / Detection gates
        assert!(
            (lane.entry_gate.start - lane.entry_gate.end).length() >= 4.0,
            "Circuit '{}': entry gate must span pit road width",
            slug
        );
        assert!(
            (lane.exit_gate.start - lane.exit_gate.end).length() >= 4.0,
            "Circuit '{}': exit gate must span pit road width",
            slug
        );

        // 5. Entry & exit divergence angles
        let entry_sample = lane.spline.sample_at_distance(0.0);
        let main_proj_entry = track.spline.project_point(entry_sample.point);
        let dot_entry = entry_sample.tangent.dot(main_proj_entry.tangent).clamp(-1.0, 1.0);
        let angle_entry_deg = dot_entry.acos().to_degrees();
        assert!(
            angle_entry_deg <= 60.0,
            "Circuit '{}': entry divergence angle ({:.1}°) must be <= 60.0°",
            slug,
            angle_entry_deg
        );

        let exit_sample = lane.spline.sample_at_distance(lane.spline.total_length());
        let main_proj_exit = track.spline.project_point(exit_sample.point);
        let dot_exit = exit_sample.tangent.dot(main_proj_exit.tangent).clamp(-1.0, 1.0);
        let angle_exit_deg = dot_exit.acos().to_degrees();
        assert!(
            angle_exit_deg <= 60.0,
            "Circuit '{}': exit merge angle ({:.1}°) must be <= 60.0°",
            slug,
            angle_exit_deg
        );

        // 6. Track validation passes cleanly
        let diagnostics = track.validate();
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.severity == tdrace_core::track::ValidationSeverity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "Circuit '{}' failed validation: {:?}",
            slug,
            errors
        );

        // 7. Grid launch clearance: starting grid launch path must not intersect entry_gate or exit_gate
        if let Some(grid_0) = track.grid_positions.first() {
            let fwd = glam::Vec2::new(grid_0.angle.cos(), grid_0.angle.sin());
            let launch_path = tdrace_core::track::geometry::LineSegment::new(
                grid_0.position,
                grid_0.position + fwd * 40.0,
            );
            assert!(
                launch_path.intersect_segment(&lane.entry_gate).is_none(),
                "Circuit '{}': starting grid slot 0 launch path intersects entry_gate!",
                slug
            );
            assert!(
                launch_path.intersect_segment(&lane.exit_gate).is_none(),
                "Circuit '{}': starting grid slot 0 launch path intersects exit_gate!",
                slug
            );
        }
    }
}

#[test]
fn test_gt_monza_surface_sampling_throughput_bench() {
    let track = load_track_from_tracks_dir("monza");
    let lane = track.pit_lane.as_ref().expect("Monza has pit lane");

    // 1. Point on track
    let p_on_track = track.spline.samples[0].point;
    assert_eq!(track.sample_surface(p_on_track), SurfaceType::Asphalt);

    // 2. Point on pit lane
    let p_pit = lane.spline.samples[lane.spline.samples.len() / 2].point;
    assert_eq!(track.sample_surface(p_pit), SurfaceType::Asphalt);

    // 3. Point in runoff/off-track far away from pit lane
    let p_far_offtrack = track.spline.samples[track.spline.samples.len() / 2].point + glam::Vec2::new(50.0, 50.0);
    assert_eq!(track.sample_surface(p_far_offtrack), track.default_surface);

    // 4. Benchmark throughput: 50,000 samples must execute smoothly
    let start = std::time::Instant::now();
    let n = 50_000;
    let hint = track.spline.total_length() * 0.5;
    for i in 0..n {
        let frac = i as f32 / n as f32;
        let pt = p_far_offtrack + glam::Vec2::new(frac * 100.0, frac * -100.0);
        let _ = track.sample_surface_near(pt, hint);
    }
    let elapsed = start.elapsed();
    #[cfg(debug_assertions)]
    let max_millis = 3000;
    #[cfg(not(debug_assertions))]
    let max_millis = 250;
    assert!(elapsed.as_millis() < max_millis, "50k samples took too long: {:?}", elapsed);
}
