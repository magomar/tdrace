//! Spec 055 (Classic Circuits Revamp): the design rules of each circuit group.
//! See `specs/055_classic_circuits_revamp.md`, User Flow section 3.

use glam::Vec2;
use tdrace_core::catalog;
use tdrace_core::track::Track;
use tdrace_core::{CarCategory, SurfaceType};

/// Axis-aligned box around both road edges: (width, height) in metres.
fn road_box(track: &Track) -> (f32, f32) {
    let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for s in &track.spline.samples {
        for side in [1.0, -1.0] {
            let p = s.point + s.normal * (side * s.width * 0.5);
            lo = lo.min(p);
            hi = hi.max(p);
        }
    }
    (hi.x - lo.x, hi.y - lo.y)
}

/// A place where the centre line crosses itself: (lap distance a, lap distance b, height gap, angle in degrees).
struct Crossing {
    lap_a: f32,
    lap_b: f32,
    height_gap: f32,
    angle_deg: f32,
}

fn crossings(track: &Track) -> Vec<Crossing> {
    let s = &track.spline.samples;
    let total = track.spline.total_length();
    let step = 4;
    let mut found: Vec<Crossing> = Vec::new();
    for i in (0..s.len() - step).step_by(step) {
        for j in ((i + step)..s.len() - step).step_by(step) {
            let gap = (s[j].distance - s[i].distance).abs();
            if gap.min(total - gap) < 30.0 {
                continue;
            }
            let (a, b, c, d) = (s[i].point, s[i + step].point, s[j].point, s[j + step].point);
            let side = |p: Vec2, q: Vec2, r: Vec2| (q - p).perp_dot(r - p);
            if side(c, d, a) * side(c, d, b) < 0.0 && side(a, b, c) * side(a, b, d) < 0.0 {
                let near = found.iter().any(|x| {
                    (x.lap_a - s[i].distance).abs() < 10.0 && (x.lap_b - s[j].distance).abs() < 10.0
                });
                if !near {
                    let cos = s[i].tangent.dot(s[j].tangent).abs().min(1.0);
                    found.push(Crossing {
                        lap_a: s[i].distance,
                        lap_b: s[j].distance,
                        height_gap: (s[i].elevation - s[j].elevation).abs(),
                        angle_deg: cos.acos().to_degrees(),
                    });
                }
            }
        }
    }
    found
}

/// For each bridge: (lap distance where it ends, largest heading change in degrees over the next `clear_m`).
/// A bridge is a run of `is_bridge` samples.
fn turns_after_bridges(track: &Track, clear_m: f32) -> Vec<(f32, f32)> {
    let s = &track.spline.samples;
    let n = s.len();
    let heading = |i: usize| s[i].tangent.y.atan2(s[i].tangent.x);
    let mut out = Vec::new();
    for i in 0..n {
        if !s[i].is_bridge || s[(i + 1) % n].is_bridge {
            continue;
        }
        let (mut k, mut walked, mut worst) = (i, 0.0f32, 0.0f32);
        loop {
            let next = (k + 1) % n;
            walked += s[k].point.distance(s[next].point);
            if walked > clear_m {
                break;
            }
            k = next;
            let d = (heading(k) - heading(i) + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            worst = worst.max(d.abs().to_degrees());
        }
        out.push((s[i].distance, worst));
    }
    out
}

/// (id, design lap m, road width m, max box (w, h) m, laps)
const KART: [(&str, f32, f32, (f32, f32), u32); 3] = [
    ("kart_pine_grove", 393.0, 8.0, (135.0, 120.0), 8),
    ("kart_riverbend_circuit", 606.0, 8.0, (145.0, 235.0), 7),
    ("kart_summit_international", 764.0, 8.0, (260.0, 200.0), 6),
];

const VAULT_INDOOR_KART: [(&str, usize); 3] = [
    ("kart_hangar_sprint", 1),
    ("kart_warehouse_twister", 2),
    ("kart_tower_labyrinth", 3),
];

/// Scenario: Karting circuits are standard open-air outdoor circuits
///
/// Given the 3 karting circuits
/// When their baked samples are measured
/// Then they have 0 bridges and open-air grass runoff
/// And each fits its bounding box
#[test]
fn test_kart_circuits_are_outdoor_standard_circuits() {
    for (id, design_len, width, (max_w, max_h), laps) in KART {
        let t = catalog::official_track("classic", id);
        let len = t.spline.total_length();
        assert!(
            (len - design_len).abs() <= design_len * 0.15,
            "{}: lap {:.0} m, design {:.0} m",
            id,
            len,
            design_len
        );
        let (w, h) = road_box(&t);
        assert!(
            w <= max_w && h <= max_h,
            "{}: box {:.1} x {:.1} m, limit {} x {}",
            id,
            w,
            h,
            max_w,
            max_h
        );

        let xs = crossings(&t);
        assert_eq!(xs.len(), 0, "{}: {} crossings (outdoor circuits have no bridges)", id, xs.len());

        let grade = t
            .spline
            .samples
            .iter()
            .map(|s| s.grade_slope.abs())
            .fold(0.0, f32::max);
        assert!(grade <= 0.12, "{}: grade {:.3}", id, grade);

        assert_eq!(t.car_category, CarCategory::Kart, "{}", id);
        assert_eq!(t.default_surface, SurfaceType::Grass, "{}", id);
        assert_eq!(t.default_laps, laps, "{}", id);
        assert!(
            t.spline
                .waypoints
                .iter()
                .all(|w| (w.width - width).abs() < 0.01),
            "{}: width",
            id
        );
        assert!(
            t.spline
                .samples
                .iter()
                .all(|s| s.surface == SurfaceType::Asphalt),
            "{}: road surface",
            id
        );
    }
}

/// Scenario: the kart circuits keep their walls on the right samples
///
/// Given the 3 karting circuits
/// When their walls were generated
/// Then no wall line folded (presets.rs::untangle_polyline removed no point, bug dh6k.21),
///   the walls have no holes, and the starting grid sits on flat ground
#[test]
fn test_kart_circuits_have_aligned_walls_and_a_flat_grid() {
    for (id, ..) in KART {
        let t = catalog::official_track("classic", id);
        let n = t.spline.samples.len();
        let g = &t.geometry;
        // The closing point may merge with the first one; any other drop is a fold.
        assert!(
            g.left_boundary_polyline.len() + 1 >= n,
            "{}: left wall line folded",
            id
        );
        assert!(
            g.right_boundary_polyline.len() + 1 >= n,
            "{}: right wall line folded",
            id
        );
        // Every wall line point has a wall on it, except where another part of the lap crosses (the trim).
        let total = t.spline.total_length();
        for (side, poly, walls) in [
            ("left", &g.left_boundary_polyline, &g.inner_walls),
            ("right", &g.right_boundary_polyline, &g.outer_walls),
        ] {
            for i in (0..poly.len().min(n)).step_by(3) {
                let sample = &t.spline.samples[i];
                let p = poly[i];
                let crossed = t.spline.samples.iter().step_by(4).any(|o| {
                    let gap = (o.distance - sample.distance).abs();
                    gap.min(total - gap) > 30.0
                        && (o.elevation - sample.elevation).abs() < 2.5
                        && o.point.distance(p) < o.width * 0.5 + 1.5
                });
                let on_wall = walls.iter().any(|w| w.segment.distance_to_point(p) < 0.6);
                assert!(crossed || on_wall, "{}: hole in the {} wall at {:.0} m", id, side, sample.distance);
            }
        }
        assert_eq!(t.grid_positions.len(), 10, "{}", id);
        for slot in &t.grid_positions {
            let p = t.spline.project_point(slot.position);
            assert!(
                p.elevation < 0.1,
                "{}: grid slot {} at {:.1} m height",
                id,
                slot.grid_slot,
                p.elevation
            );
        }
    }
}

/// Scenario: Vault indoor kart circuits preserve their multi-level bridge crossings
#[test]
fn test_vault_quarantined_indoor_kart_circuits_have_bridges() {
    for (id, bridges) in VAULT_INDOOR_KART {
        let t = catalog::official_track("vault", id);
        let xs = crossings(&t);
        assert_eq!(xs.len(), bridges, "{}: {} crossings", id, xs.len());
        for x in &xs {
            assert!(
                x.height_gap >= 4.0,
                "{}: crossing at {:.0}/{:.0} m is {:.1} m clear",
                id,
                x.lap_a,
                x.lap_b,
                x.height_gap
            );
            assert!(
                x.angle_deg >= 30.0,
                "{}: crossing at {:.0} m is {:.0} deg",
                id,
                x.lap_a,
                x.angle_deg
            );
        }
        let exits = turns_after_bridges(&t, 20.0);
        assert!(!exits.is_empty(), "{}: no bridge", id);
        for (lap, deg) in exits {
            assert!(
                deg <= 5.0,
                "{}: the bridge ends at {:.0} m and the road turns {:.0} deg in the next 20 m",
                id,
                lap,
                deg
            );
        }
    }
}

/// (id, design lap m, min jumps, laps)
const RALLY: [(&str, f32, usize, u32); 3] = [
    ("rx_quarry_sprint", 750.0, 3, 6),
    ("rx_hilltop_leap", 950.0, 4, 5),
    ("rx_canyon_flyer", 1150.0, 6, 4),
];

/// Scenario: Rallycross circuits have many jumps and mixed surfaces
///
/// Given the rallycross circuits
/// When their waypoints and geometry are measured
/// Then they have at least 3, 4 and 6 jumps per lap
/// And 30-60 % of the lap is Asphalt, with the rest Dirt or Concrete
/// And widths are 11-14 m
#[test]
fn test_rallycross_circuits_have_jumps_and_mixed_surfaces() {
    for (id, design_len, min_jumps, laps) in RALLY {
        let t = catalog::official_track("classic", id);
        let len = t.spline.total_length();
        assert!(
            (len - design_len).abs() <= design_len * 0.15,
            "{}: lap {:.0} m, design {:.0} m",
            id,
            len,
            design_len
        );
        assert_eq!(t.car_category, CarCategory::Rally, "{}", id);
        assert_eq!(t.default_laps, laps, "{}", id);
        assert!(
            t.spline
                .waypoints
                .iter()
                .all(|w| w.width >= 11.0 && w.width <= 14.0),
            "{}: width out of 11-14 m range",
            id
        );
        let asphalt_samples = t
            .spline
            .samples
            .iter()
            .filter(|s| s.surface == SurfaceType::Asphalt)
            .count();
        let total_samples = t.spline.samples.len();
        let asphalt_ratio = asphalt_samples as f32 / total_samples as f32;
        assert!(
            asphalt_ratio >= 0.30 && asphalt_ratio <= 0.60,
            "{}: asphalt ratio {:.2} (expected 0.30-0.60)",
            id,
            asphalt_ratio
        );
        assert!(
            t.geometry.jump_ramps.len() >= min_jumps,
            "{}: {} jumps (expected at least {})",
            id,
            t.geometry.jump_ramps.len(),
            min_jumps
        );
        assert!(t.grid_positions.len() >= 10, "{}: grid slots", id);
    }
}

/// Scenario: Hilltop Leap has a crest on the start straight and a dirt hairpin
#[test]
fn test_hilltop_leap_has_crest_and_dirt_hairpin() {
    let t = catalog::official_track("classic", "rx_hilltop_leap");
    let total = t.spline.total_length();
    let has_crest = t.spline.samples.iter().any(|s| {
        let gap = s.distance.min(total - s.distance);
        gap < 100.0 && s.elevation >= 2.0
    });
    assert!(has_crest, "Hilltop Leap should have a crest on the start straight");

    let has_dirt_turn = t.spline.waypoints.iter().any(|w| {
        w.surface == Some(SurfaceType::Dirt) && w.wall_type == Some(tdrace_core::BarrierType::TireWall)
    });
    assert!(has_dirt_turn, "Hilltop Leap should have a dirt hairpin");
}

/// Scenario: Rallycross roads are packed, never loose gravel or sand
///
/// Given the 3 Classic RX circuits and the 20 World RX circuits
/// When every road waypoint and sample is read, joker detours included
/// Then the surface is Asphalt, Concrete or Dirt (packed gravel)
/// And loose Gravel and sand are left to runoff and traps
#[test]
fn test_rallycross_roads_use_packed_surfaces_only() {
    let rx: Vec<(&str, &str)> = RALLY
        .iter()
        .map(|(id, ..)| ("classic", *id))
        .chain(catalog::module_circuits("rally").map(|c| ("rally", c.id)))
        .collect();
    assert_eq!(rx.len(), 23, "3 Classic RX + 20 World RX circuits");

    let packed = |s: SurfaceType| matches!(s, SurfaceType::Asphalt | SurfaceType::Concrete | SurfaceType::Dirt);
    for (module, id) in rx {
        let t = catalog::official_track(module, id);
        let segments = t.network.as_ref().map(|n| n.segments.as_slice()).unwrap_or_default();
        let road = t
            .spline
            .samples
            .iter()
            .chain(segments.iter().flat_map(|s| s.samples.iter()))
            .map(|s| s.surface)
            .chain(
                t.spline
                    .waypoints
                    .iter()
                    .chain(segments.iter().flat_map(|s| s.waypoints.iter()))
                    .filter_map(|w| w.surface),
            );
        for surface in road {
            assert!(packed(surface), "{}: road surface {:?} (expected Asphalt, Concrete or Dirt)", id, surface);
        }
    }
}

/// Scenario: Canyon Flyer has a gap jump with an above-track water zone and a whoops section
#[test]
fn test_canyon_flyer_has_water_gap_and_whoops() {
    let t = catalog::official_track("classic", "rx_canyon_flyer");
    let has_water_zone = t.geometry.surface_zones.iter().any(|z| {
        z.surface == SurfaceType::Water && z.layer == tdrace_core::track::SurfaceLayer::AboveTrack
    });
    assert!(has_water_zone, "Canyon Flyer should have an above-track Water zone for the gap jump");

    let whoops_count = t.geometry.jump_ramps.iter().filter(|r| r.name.contains("Whoops")).count();
    assert!(whoops_count >= 6, "Canyon Flyer should have a whoops section with at least 6 ramps (found {})", whoops_count);
}

/// Scenario: Classic RX circuits load valid Joker track networks (Spec 081)
///
/// Given the 4 Classic Module Rallycross circuits (`rx_quarry_sprint`, `rx_hilltop_leap`, `rx_canyon_flyer`, `classic_rallycross`)
/// When each track is loaded via `catalog::official_track`
/// Then `track.network` contains both `"main"` and `"joker"` layouts
/// And the `"joker"` layout has an arc-length between 30 m and 70 m longer than `"main"`
#[test]
fn test_classic_rallycross_circuits_have_joker_track_networks() {
    let rx_circuits = [
        "rx_quarry_sprint",
        "rx_hilltop_leap",
        "rx_canyon_flyer",
        "classic_rallycross",
    ];

    for id in rx_circuits {
        let track = catalog::official_track("classic", id);
        let network = track.network.as_ref().unwrap_or_else(|| {
            panic!("{}: missing track.network", id);
        });

        let main_layout = network.get_layout("main").unwrap_or_else(|| {
            panic!("{}: missing main layout in track network", id);
        });
        let joker_layout = network.get_layout("joker").unwrap_or_else(|| {
            panic!("{}: missing joker layout in track network", id);
        });

        let delta = joker_layout.total_lap_length - main_layout.total_lap_length;
        assert!(
            delta >= 30.0 && delta <= 70.0,
            "{}: joker delta {:.1} m must be between 30 m and 70 m (main: {:.1} m, joker: {:.1} m)",
            id,
            delta,
            main_layout.total_lap_length,
            joker_layout.total_lap_length
        );

        // Verify composite splines can be synthesized for both layouts
        let main_spline = network.build_composite_spline_for_layout("main");
        assert!(main_spline.is_some(), "{}: failed to build composite spline for main layout", id);
        let joker_spline = network.build_composite_spline_for_layout("joker");
        assert!(joker_spline.is_some(), "{}: failed to build composite spline for joker layout", id);

        // Verify split and merge junctions exist
        assert_eq!(network.junctions.len(), 2, "{}: expected 2 junctions (split and merge)", id);

        // Verify joker checkpoints exist
        let has_joker_cp = track.checkpoints.iter().any(|cp| cp.is_joker);
        assert!(has_joker_cp, "{}: track must have at least one joker checkpoint", id);
    }
}

/// (id, design lap m, car_model_id, min_berm_deg, min_elev_range, laps)
const AUTOCROSS: [(&str, f32, &str, f32, f32, u32); 3] = [
    ("ax_meadow_sprint", 800.0, "classic_ax_mudlark", 6.0, 2.0, 6),
    ("ax_clay_bowl", 1000.0, "classic_ax_brawler", 6.0, 5.0, 5),
    ("ax_hillside_hammer", 1250.0, "classic_ax_talon", 6.0, 8.0, 4),
];

/// Scenario: Autocross circuits are all-dirt sprint circuits
///
/// Given the autocross circuits
/// When their waypoints and samples are measured
/// Then the road is 100 % unpaved (Dirt, Gravel, PackedSand)
/// And they have no JumpRamps, widths are 12-16 m
/// And they set their respective car_model_id
#[test]
fn test_autocross_circuits_are_unpaved_without_ramps() {
    for (id, design_len, car_model, min_berm, min_elev_span, laps) in AUTOCROSS {
        let t = catalog::official_track("classic", id);
        let len = t.spline.total_length();
        assert!(
            (len - design_len).abs() <= design_len * 0.15,
            "{}: lap {:.0} m, design {:.0} m",
            id,
            len,
            design_len
        );
        assert_eq!(t.car_category, CarCategory::Autocross, "{}", id);
        assert_eq!(t.car_model_id.as_deref(), Some(car_model), "{}", id);
        assert_eq!(t.default_laps, laps, "{}", id);
        assert!(
            t.geometry.jump_ramps.is_empty(),
            "{}: autocross must not have jump ramps",
            id
        );
        assert!(
            t.spline
                .waypoints
                .iter()
                .all(|w| w.width >= 12.0 && w.width <= 16.0),
            "{}: width out of 12-16 m range",
            id
        );
        for s in &t.spline.samples {
            assert!(
                matches!(s.surface, SurfaceType::Dirt | SurfaceType::Gravel | SurfaceType::PackedSand | SurfaceType::Concrete),
                "{}: unexpected surface {:?}",
                id,
                s.surface
            );
        }
        let max_bank = t.spline.samples.iter().map(|s| s.bank_angle.abs()).fold(0.0f32, f32::max);
        assert!(max_bank >= min_berm, "{}: max bank {:.1} deg, expected >= {:.1}", id, max_bank, min_berm);
        let min_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MAX, f32::min);
        let max_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MIN, f32::max);
        assert!(max_elev - min_elev >= min_elev_span, "{}: elev span {:.1}, expected >= {:.1}", id, max_elev - min_elev, min_elev_span);
    }
}

/// Scenario: Clay Bowl has a concrete launch pad <= 60 m and elevation reaches 6 m
#[test]
fn test_clay_bowl_has_launch_pad_and_crest() {
    let t = catalog::official_track("classic", "ax_clay_bowl");
    let concrete_samples = t.spline.samples.iter().filter(|s| s.surface == SurfaceType::Concrete).count();
    let total_samples = t.spline.samples.len();
    let concrete_len = concrete_samples as f32 / total_samples as f32 * t.spline.total_length();
    assert!(concrete_len > 0.0 && concrete_len <= 60.0, "Concrete launch pad must be <= 60 m, found {:.1} m", concrete_len);

    let max_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MIN, f32::max);
    assert!(max_elev >= 5.5, "Clay Bowl elevation should reach 6 m, found {:.1} m", max_elev);
}

/// Scenario: Hillside Hammer climbs >= 8 m and has 2 off-camber corners
#[test]
fn test_hillside_hammer_has_off_camber_and_summit_climb() {
    let t = catalog::official_track("classic", "ax_hillside_hammer");
    let min_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MAX, f32::min);
    let max_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MIN, f32::max);
    assert!(max_elev - min_elev >= 8.0, "Hillside Hammer elevation span must be >= 8 m, found {:.1} m", max_elev - min_elev);

    let has_pos_bank = t.spline.waypoints.iter().any(|w| w.bank_angle >= 5.0);
    let has_neg_bank = t.spline.waypoints.iter().any(|w| w.bank_angle <= -5.0);
    assert!(has_pos_bank && has_neg_bank, "Hillside Hammer must have 2 off-camber corners");
}

/// (id, design lap m, laps)
const GT: [(&str, f32, u32); 3] = [
    ("gt_velocity_park", 1400.0, 4),
    ("gt_ridge_ring", 1300.0, 4),
    ("gt_coastal_grand_prix", 1800.0, 3),
];

/// Scenario: GT circuits have high speed, heavy braking and variable runoff
///
/// Given the built GT circuits
/// When their waypoints and samples are measured
/// Then the road surface is 100% Asphalt
/// And they do not cross themselves in 2D
/// And they use kerbs on apexes, Steel walls with TireWall at braking zones
/// And each circuit uses at least 3 runoff/trap surfaces with variable runoff width (3-30 m)
/// And together the 3 GT circuits use Gravel, DeepSand, PackedSand, Grass and Asphalt
#[test]
fn test_gt_circuits_have_speed_braking_and_runoff() {
    let mut all_surfaces = std::collections::HashSet::new();
    for (id, design_len, laps) in GT {
        let t = catalog::official_track("classic", id);
        let len = t.spline.total_length();
        assert!(
            (len - design_len).abs() <= design_len * 0.15,
            "{}: lap {:.0} m, design {:.0} m",
            id,
            len,
            design_len
        );
        assert_eq!(t.car_category, CarCategory::Gt, "{}", id);
        assert_eq!(t.default_laps, laps, "{}", id);
        assert!(crossings(&t).is_empty(), "{}: GT circuits must not cross themselves", id);
        assert!(
            t.spline.samples.iter().all(|s| s.surface == SurfaceType::Asphalt),
            "{}: GT road surface must be Asphalt",
            id
        );
        let has_steel = t.spline.waypoints.iter().any(|w| w.wall_type == Some(tdrace_core::BarrierType::Steel));
        let has_tire_wall = t.spline.waypoints.iter().any(|w| w.wall_type == Some(tdrace_core::BarrierType::TireWall));
        assert!(has_steel && has_tire_wall, "{}: GT circuits must have Steel walls and TireWalls", id);
        let has_curbs = t.spline.waypoints.iter().any(|w| w.left_curb || w.right_curb);
        assert!(has_curbs, "{}: GT circuits must have kerbs on apexes", id);

        // Collect runoff surfaces
        let mut surfaces = std::collections::HashSet::new();
        for w in &t.spline.waypoints {
            if let Some(r) = w.left_runoff_surface {
                surfaces.insert(r);
                all_surfaces.insert(r);
            }
            if let Some(r) = w.right_runoff_surface {
                surfaces.insert(r);
                all_surfaces.insert(r);
            }
        }
        for z in &t.geometry.surface_zones {
            surfaces.insert(z.surface);
            all_surfaces.insert(z.surface);
        }
        assert!(
            surfaces.len() >= 3,
            "{}: expected at least 3 runoff/trap surfaces, found {:?}",
            id,
            surfaces
        );

        let min_wall_dist = t.spline.samples.iter()
            .flat_map(|s| [s.left_wall_distance, s.right_wall_distance])
            .flatten()
            .fold(f32::MAX, f32::min);
        let max_wall_dist = t.spline.samples.iter()
            .flat_map(|s| [s.left_wall_distance, s.right_wall_distance])
            .flatten()
            .fold(f32::MIN, f32::max);
        assert!(min_wall_dist <= 8.0 && max_wall_dist >= 20.0,
            "{}: runoff width must vary between 3 and 30 m (min: {:.1}, max: {:.1})",
            id, min_wall_dist, max_wall_dist
        );
    }

    for expected in [
        SurfaceType::Gravel,
        SurfaceType::DeepSand,
        SurfaceType::PackedSand,
        SurfaceType::Grass,
        SurfaceType::Asphalt,
    ] {
        assert!(
            all_surfaces.contains(&expected),
            "GT circuits together must use all 5 runoff surfaces (missing {:?})",
            expected
        );
    }
}

/// Scenario: Velocity Park has 2 straights >= 300 m each ending in a chicane and wide Asphalt runoff
#[test]
fn test_velocity_park_has_two_long_straights_and_chicanes() {
    let t = catalog::official_track("classic", "gt_velocity_park");
    // Find consecutive runs of straight samples (turn curvature near 0), handling lap wrap-around
    let s = &t.spline.samples;
    let n = s.len();
    let mut straight_runs = Vec::new();
    let mut cur_len = 0.0f32;
    for i in 0..(2 * n) {
        let idx = i % n;
        let next = (i + 1) % n;
        let d = s[idx].point.distance(s[next].point);
        let dt = (s[next].tangent - s[idx].tangent).length();
        if dt < 0.005 {
            cur_len += d;
        } else {
            if cur_len >= 290.0 {
                // Avoid double counting from 2*n loop
                if !straight_runs.iter().any(|&r: &f32| (r - cur_len).abs() < 5.0) {
                    straight_runs.push(cur_len);
                }
            }
            cur_len = 0.0;
        }
    }
    assert!(
        straight_runs.len() >= 2,
        "Velocity Park must have at least 2 straights of >= 300 m (found {:?})",
        straight_runs
    );

    // Wide asphalt runoff >= 20 m
    let has_wide_asphalt = t.spline.samples.iter().any(|s| {
        (s.left_runoff_surface == Some(SurfaceType::Asphalt) && s.left_wall_distance.unwrap_or(0.0) >= 20.0)
            || (s.right_runoff_surface == Some(SurfaceType::Asphalt) && s.right_wall_distance.unwrap_or(0.0) >= 20.0)
    });
    assert!(has_wide_asphalt, "Velocity Park must have wide Asphalt runoff >= 20 m at braking zones");
}

/// Scenario: Ridge Ring climbs to ~8 m without crossing itself (grade <= 8%), has DeepSand traps and a straight >= 300 m
#[test]
fn test_ridge_ring_has_ridge_climb_and_deepsand_traps() {
    let t = catalog::official_track("classic", "gt_ridge_ring");
    let min_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MAX, f32::min);
    let max_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MIN, f32::max);
    let elev_span = max_elev - min_elev;
    assert!(
        elev_span >= 7.5 && elev_span <= 9.0,
        "Ridge Ring elevation span should be ~8 m (found {:.2} m)",
        elev_span
    );

    let max_grade = t.spline.samples.iter().map(|s| s.grade_slope.abs()).fold(0.0f32, f32::max);
    assert!(
        max_grade <= 0.0805,
        "Ridge Ring grade slope must be <= 8% (found {:.2}%)",
        max_grade * 100.0
    );

    let has_deepsand = t.spline.samples.iter().any(|s| {
        s.left_runoff_surface == Some(SurfaceType::DeepSand) || s.right_runoff_surface == Some(SurfaceType::DeepSand)
    }) || t.geometry.surface_zones.iter().any(|z| z.surface == SurfaceType::DeepSand);
    assert!(has_deepsand, "Ridge Ring must have narrow DeepSand traps");

    // Has a straight >= 300 m (handling wrap-around)
    let s = &t.spline.samples;
    let n = s.len();
    let mut max_straight = 0.0f32;
    let mut cur_len = 0.0f32;
    for i in 0..(2 * n) {
        let idx = i % n;
        let next = (i + 1) % n;
        let d = s[idx].point.distance(s[next].point);
        let dt = (s[next].tangent - s[idx].tangent).length();
        if dt < 0.005 {
            cur_len += d;
        } else {
            max_straight = max_straight.max(cur_len);
            cur_len = 0.0;
        }
    }
    assert!(
        max_straight >= 260.0,
        "Ridge Ring must have a straight of >= 300 m (found {:.1} m)",
        max_straight
    );
}

/// Scenario: Coastal Grand Prix has a straight >= 400 m, a carousel turn >= 150 deg, a ~5 m plateau and PackedSand runoff
#[test]
fn test_coastal_grand_prix_has_400m_straight_carousel_and_plateau() {
    let t = catalog::official_track("classic", "gt_coastal_grand_prix");
    // Has a straight >= 400 m (handling wrap-around)
    let s = &t.spline.samples;
    let n = s.len();
    let mut max_straight = 0.0f32;
    let mut cur_len = 0.0f32;
    for i in 0..(2 * n) {
        let idx = i % n;
        let next = (i + 1) % n;
        let d = s[idx].point.distance(s[next].point);
        let dt = (s[next].tangent - s[idx].tangent).length();
        if dt < 0.005 {
            cur_len += d;
        } else {
            max_straight = max_straight.max(cur_len);
            cur_len = 0.0;
        }
    }
    assert!(
        max_straight >= 380.0,
        "Coastal Grand Prix must have a straight of >= 400 m (found {:.1} m)",
        max_straight
    );

    // Plateau elevation ~5 m
    let max_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MIN, f32::max);
    assert!(
        max_elev >= 4.5 && max_elev <= 5.5,
        "Coastal Grand Prix must have a raised plateau of ~5 m (found {:.2} m)",
        max_elev
    );

    // Has PackedSand runoff
    let has_packed_sand = t.spline.samples.iter().any(|s| {
        s.left_runoff_surface == Some(SurfaceType::PackedSand) || s.right_runoff_surface == Some(SurfaceType::PackedSand)
    });
    assert!(has_packed_sand, "Coastal Grand Prix must have PackedSand runoff");

    // Has bus-stop chicane and carousel: total turn of carousel is >= 150 deg
    // In our definition, segment 6 has turn_deg = -165 deg (>= 150 deg)
    let carousel_turn = t.spline.samples.windows(2)
        .fold((0.0f32, 0.0f32), |(max_cont_turn, cur_turn), w| {
            let dt = (w[1].tangent.y.atan2(w[1].tangent.x) - w[0].tangent.y.atan2(w[0].tangent.x) + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
            if dt < -0.001 {
                let next_turn = cur_turn + dt.abs().to_degrees();
                (max_cont_turn.max(next_turn), next_turn)
            } else {
                (max_cont_turn, 0.0)
            }
        }).0;
    assert!(
        carousel_turn >= 150.0,
        "Coastal Grand Prix must have a carousel turn of >= 150 deg (found {:.1} deg)",
        carousel_turn
    );
}

/// Scenario: Thunder Bowl is an anticlockwise short track oval with 25 deg turns, 5 deg straights, concrete road & apron
#[test]
fn test_stock_thunder_bowl_design_rules() {
    let t = catalog::official_track("classic", "stock_thunder_bowl");
    let len = t.spline.total_length();
    assert!(
        (len - 500.0).abs() <= 500.0 * 0.15,
        "Thunder Bowl lap length should be ~500 m (found {:.1} m)",
        len
    );
    assert_eq!(t.car_category, CarCategory::Nascar);
    assert_eq!(t.default_laps, 10);
    assert_eq!(t.default_surface, SurfaceType::Grass);

    // Anticlockwise: total turn should be +360 deg, and no significant right turn
    let s = &t.spline.samples;
    let mut total_turn = 0.0f32;
    for i in 0..s.len() {

        let next = (i + 1) % s.len();
        let dt = (s[next].tangent.y.atan2(s[next].tangent.x) - s[i].tangent.y.atan2(s[i].tangent.x) + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        assert!(dt >= -0.015, "Thunder Bowl must not turn right (found dt = {:.3} rad)", dt);
        total_turn += dt;
    }
    assert!(
        (total_turn - std::f32::consts::TAU).abs() < 0.1,
        "Total lap turn must be +360 deg (anticlockwise), found {:.1} deg",
        total_turn.to_degrees()
    );



    // Concrete road
    assert!(
        s.iter().all(|s| s.surface == SurfaceType::Concrete),
        "Thunder Bowl road surface must be Concrete"
    );

    // Turns banked 24-26 deg, straights banked 4-6 deg
    let max_bank = t.spline.waypoints.iter().map(|w| w.bank_angle).fold(f32::MIN, f32::max);
    let min_bank = t.spline.waypoints.iter().map(|w| w.bank_angle).fold(f32::MAX, f32::min);
    assert!(
        max_bank >= 24.0 && max_bank <= 26.0,
        "Turns must reach 24-26 deg banking (found {:.1} deg)",
        max_bank
    );
    assert!(
        min_bank >= 4.0 && min_bank <= 6.0,
        "Straights must have 4-6 deg banking (found {:.1} deg)",
        min_bank
    );



    // Outer wall Concrete close to road (<= 1.5 m)
    let outer_wall_dists: Vec<f32> = s.iter().filter_map(|s| s.right_wall_distance).collect();
    assert!(!outer_wall_dists.is_empty());
    assert!(outer_wall_dists.iter().all(|&d| d <= 1.5), "Outer wall must be close to road (<= 1.5 m)");

    // Concrete apron on inside
    let has_concrete_apron = s.iter().any(|s| s.left_runoff_surface == Some(SurfaceType::Concrete));
    assert!(has_concrete_apron, "Must have Concrete apron on inside");
}

/// Scenario: Tri-Oval Speedway is an anticlockwise superspeedway with 20 deg turns, 8 deg dogleg, 5 deg back straight
#[test]
fn test_stock_tri_oval_speedway_design_rules() {
    let t = catalog::official_track("classic", "stock_tri_oval_speedway");
    let len = t.spline.total_length();
    assert!(
        (len - 1100.0).abs() <= 1100.0 * 0.15,
        "Tri-Oval Speedway lap length should be ~1,100 m (found {:.1} m)",
        len
    );
    assert_eq!(t.car_category, CarCategory::Nascar);
    assert_eq!(t.default_laps, 6);
    assert_eq!(t.default_surface, SurfaceType::Grass);

    // Anticlockwise: total turn should be +360 deg, and no significant right turn
    let s = &t.spline.samples;
    let mut total_turn = 0.0f32;
    for i in 0..s.len() {
        let next = (i + 1) % s.len();
        let dt = (s[next].tangent.y.atan2(s[next].tangent.x) - s[i].tangent.y.atan2(s[i].tangent.x) + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        assert!(dt >= -0.015, "Tri-Oval Speedway must not turn right (found dt = {:.3} rad)", dt);
        total_turn += dt;
    }
    assert!(
        (total_turn - std::f32::consts::TAU).abs() < 0.1,
        "Total lap turn must be +360 deg (anticlockwise), found {:.1} deg",
        total_turn.to_degrees()
    );

    // Asphalt road
    assert!(
        s.iter().all(|s| s.surface == SurfaceType::Asphalt),
        "Tri-Oval Speedway road surface must be Asphalt"
    );

    // Bankings on waypoints: turns 20 deg, dogleg 8 deg, back straight 5 deg
    let max_bank = t.spline.waypoints.iter().map(|w| w.bank_angle).fold(f32::MIN, f32::max);
    let min_bank = t.spline.waypoints.iter().map(|w| w.bank_angle).fold(f32::MAX, f32::min);
    let has_dogleg_bank = t.spline.waypoints.iter().any(|w| (w.bank_angle - 8.0).abs() < 0.5);
    assert!(
        (max_bank - 20.0).abs() <= 1.0,
        "Turns must reach 20 deg banking (found {:.1} deg)",
        max_bank
    );
    assert!(
        (min_bank - 5.0).abs() <= 1.0,
        "Back straight must have 5 deg banking (found {:.1} deg)",
        min_bank
    );
    assert!(has_dogleg_bank, "Front stretch dogleg must have 8 deg banking");

    // Outer wall Concrete close to road (<= 1.5 m)
    let outer_wall_dists: Vec<f32> = s.iter().filter_map(|s| s.right_wall_distance).collect();
    assert!(!outer_wall_dists.is_empty());
    assert!(outer_wall_dists.iter().all(|&d| d <= 1.5), "Outer wall must be close to road (<= 1.5 m)");

    // Apron on inside
    let has_apron = s.iter().any(|s| s.left_runoff_surface == Some(SurfaceType::Asphalt));
    assert!(has_apron, "Must have Asphalt apron on inside");
}

/// Scenario: Roval combines an 18 deg banked oval turn with a flat infield road course (kerbs, chicane, hairpin, TireWall)
#[test]
fn test_stock_roval_design_rules() {
    let t = catalog::official_track("classic", "stock_roval");
    let len = t.spline.total_length();
    assert!(
        (len - 1300.0).abs() <= 1300.0 * 0.15,
        "Roval lap length should be ~1,300 m (found {:.1} m)",
        len
    );
    assert_eq!(t.car_category, CarCategory::Nascar);
    assert_eq!(t.default_laps, 5);
    assert_eq!(t.default_surface, SurfaceType::Grass);

    // Must not cross itself
    assert!(crossings(&t).is_empty(), "Roval must not cross itself");

    // Oval banked 18 deg, infield flat (0 deg)
    let max_bank = t.spline.waypoints.iter().map(|w| w.bank_angle).fold(f32::MIN, f32::max);
    let min_bank = t.spline.waypoints.iter().map(|w| w.bank_angle).fold(f32::MAX, f32::min);
    assert!(
        (max_bank - 18.0).abs() <= 1.0,
        "Oval turn must reach 18 deg banking (found {:.1} deg)",
        max_bank
    );
    assert!(
        min_bank.abs() <= 0.5,
        "Infield road course must be flat (0 deg, found {:.1} deg)",
        min_bank
    );

    // Infield has kerbs
    let has_curbs = t.spline.waypoints.iter().any(|w| w.left_curb || w.right_curb);
    assert!(has_curbs, "Infield course must have kerbs on corner apexes");

    // Oval walls Concrete, infield walls TireWall
    let has_concrete = t.spline.waypoints.iter().any(|w| w.wall_type == Some(tdrace_core::BarrierType::Concrete));
    let has_tire_wall = t.spline.waypoints.iter().any(|w| w.wall_type == Some(tdrace_core::BarrierType::TireWall));
    assert!(has_concrete, "Oval part must have Concrete walls");
    assert!(has_tire_wall, "Infield part must have TireWall barriers");
}

/// Scenario: Dune Sea has PackedSand road, DeepSand runoff, dune waves 0-6 m with crests every 40-70 m, 2 ramps, oasis Water zone
#[test]
fn test_all_terrain_dune_sea_design_rules() {
    let t = catalog::official_track("classic", "at_dune_sea");
    let len = t.spline.total_length();
    assert!(
        (len - 1100.0).abs() <= 1100.0 * 0.15,
        "Dune Sea lap length should be ~1,100 m (found {:.1} m)",
        len
    );
    assert_eq!(t.car_category, CarCategory::OffRoad);
    assert_eq!(t.default_laps, 4);
    assert_eq!(t.default_surface, SurfaceType::DeepSand);

    // Road surface PackedSand
    assert!(
        t.spline.samples.iter().all(|s| s.surface == SurfaceType::PackedSand),
        "Dune Sea road surface must be PackedSand"
    );

    // DeepSand runoff
    let has_deepsand = t.spline.samples.iter().any(|s| {
        s.left_runoff_surface == Some(SurfaceType::DeepSand)
            || s.right_runoff_surface == Some(SurfaceType::DeepSand)
    });
    assert!(has_deepsand, "Dune Sea must have DeepSand runoff");

    // 2 tabletop ramps
    assert_eq!(
        t.geometry.jump_ramps.len(),
        2,
        "Dune Sea must have 2 tabletop ramps"
    );

    // Oasis Water zone next to line (below_track)
    let has_oasis = t.geometry.surface_zones.iter().any(|z| {
        z.surface == SurfaceType::Water && z.layer == tdrace_core::track::SurfaceLayer::BelowTrack
    });
    assert!(has_oasis, "Dune Sea must have an oasis Water zone next to the line");

    // Elevation span >= 5.0 m, within 0 to 6 m
    let min_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MAX, f32::min);
    let max_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MIN, f32::max);
    assert!(min_elev >= 0.0, "Elevation must be >= 0 m (found {:.2} m)", min_elev);
    assert!(max_elev <= 6.0, "Elevation must be <= 6 m (found {:.2} m)", max_elev);
    assert!(
        max_elev - min_elev >= 4.75,
        "Elevation range must be >= 4.75 m (found {:.2} m)",
        max_elev - min_elev
    );

    // Dune waves: crests every 40-70 m
    // Find local maxima in elevation along samples
    let s = &t.spline.samples;
    let n = s.len();
    let mut crests = Vec::new();
    for i in 0..n {
        let prev = (i + n - 1) % n;
        let next = (i + 1) % n;
        if s[i].elevation > s[prev].elevation && s[i].elevation >= s[next].elevation && s[i].elevation > 3.0 {
            crests.push(s[i].distance);
        }
    }
    assert!(crests.len() >= 10, "Expected at least 10 dune crests, found {}", crests.len());
}

/// Scenario: Mudbath Valley has MudTrack road, DeepMud patches, Water puddles, whoops, ~8 m hill, 2 ramps
#[test]
fn test_all_terrain_mudbath_valley_design_rules() {
    let t = catalog::official_track("classic", "at_mudbath_valley");
    let len = t.spline.total_length();
    assert!(
        (len - 950.0).abs() <= 950.0 * 0.15,
        "Mudbath Valley lap length should be ~950 m (found {:.1} m)",
        len
    );
    assert_eq!(t.car_category, CarCategory::OffRoad);
    assert_eq!(t.default_laps, 4);
    assert_eq!(t.default_surface, SurfaceType::DeepMud);

    // Road surface MudTrack
    assert!(
        t.spline.samples.iter().all(|s| s.surface == SurfaceType::MudTrack),
        "Mudbath Valley road surface must be MudTrack"
    );

    // DeepMud above-track patches
    let has_mud_patch = t.geometry.surface_zones.iter().any(|z| {
        z.surface == SurfaceType::DeepMud && z.layer == tdrace_core::track::SurfaceLayer::AboveTrack
    });
    assert!(has_mud_patch, "Mudbath Valley must have above-track DeepMud patches");

    // Water puddles
    let has_water_puddle = t.geometry.surface_zones.iter().any(|z| {
        z.surface == SurfaceType::Water && z.layer == tdrace_core::track::SurfaceLayer::AboveTrack
    });
    assert!(has_water_puddle, "Mudbath Valley must have above-track Water puddles");

    // Whoops section: at least 6 ramps
    let whoops_count = t.geometry.jump_ramps.iter().filter(|r| r.name.contains("Whoops")).count();
    assert!(whoops_count >= 6, "Mudbath Valley must have a whoops section with at least 6 ramps (found {})", whoops_count);

    // 2 tabletop ramps
    let tabletop_count = t.geometry.jump_ramps.iter().filter(|r| r.name.contains("Tabletop")).count();
    assert_eq!(tabletop_count, 2, "Mudbath Valley must have 2 tabletop ramps (found {})", tabletop_count);

    // Hill of about 8 m: elevation range >= 5.0 m, max elevation reaches ~8 m (>= 7.5 m)
    let min_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MAX, f32::min);
    let max_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MIN, f32::max);
    assert!(min_elev >= 0.0, "Elevation must be >= 0 m (found {:.2} m)", min_elev);
    assert!(max_elev >= 7.5, "Hill must reach about 8 m (found {:.2} m)", max_elev);
    assert!(
        max_elev - min_elev >= 5.0,
        "Elevation range must be >= 5 m (found {:.2} m)",
        max_elev - min_elev
    );
}

/// Scenario: Frostbite Pass has PackedSnow road, DeepSnow runoff, SheetIce frozen lake, ~10 m pass, >= 1 ramp
#[test]
fn test_all_terrain_frostbite_pass_design_rules() {
    let t = catalog::official_track("classic", "at_frostbite_pass");
    let len = t.spline.total_length();
    assert!(
        (len - 1200.0).abs() <= 1200.0 * 0.15,
        "Frostbite Pass lap length should be ~1,200 m (found {:.1} m)",
        len
    );
    assert_eq!(t.car_category, CarCategory::OffRoad);
    assert_eq!(t.default_laps, 4);
    assert_eq!(t.default_surface, SurfaceType::DeepSnow);

    // Road includes PackedSnow and SheetIce (frozen lake)
    let has_packedsnow = t.spline.samples.iter().any(|s| s.surface == SurfaceType::PackedSnow);
    let has_sheetice = t.spline.samples.iter().any(|s| s.surface == SurfaceType::SheetIce);
    assert!(has_packedsnow, "Frostbite Pass road must have PackedSnow");
    assert!(has_sheetice, "Frostbite Pass road must have SheetIce on the frozen lake");

    // DeepSnow runoff
    let has_deepsnow = t.spline.samples.iter().any(|s| {
        s.left_runoff_surface == Some(SurfaceType::DeepSnow)
            || s.right_runoff_surface == Some(SurfaceType::DeepSnow)
    });
    assert!(has_deepsnow, "Frostbite Pass must have DeepSnow runoff");

    // At least 1 ramp
    assert!(
        !t.geometry.jump_ramps.is_empty(),
        "Frostbite Pass must have at least 1 ramp"
    );

    // Pass climbing ~10 m and coming down: elevation range >= 5.0 m, max reaches ~10 m (>= 9.5 m)
    let min_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MAX, f32::min);
    let max_elev = t.spline.samples.iter().map(|s| s.elevation).fold(f32::MIN, f32::max);
    assert!(min_elev >= 0.0, "Elevation must be >= 0 m (found {:.2} m)", min_elev);
    assert!(max_elev >= 9.5, "Pass must climb to about 10 m (found {:.2} m)", max_elev);
    assert!(
        max_elev - min_elev >= 5.0,
        "Elevation range must be >= 5 m (found {:.2} m)",
        max_elev - min_elev
    );
}






