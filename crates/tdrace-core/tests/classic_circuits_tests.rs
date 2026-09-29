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

/// (id, design lap m, road width m, max box (w, h) m, bridges, laps)
const KART: [(&str, f32, f32, (f32, f32), usize, u32); 3] = [
    ("kart_hangar_sprint", 380.0, 8.0, (90.0, 60.0), 1, 8),
    ("kart_warehouse_twister", 520.0, 7.0, (100.0, 70.0), 2, 7),
    ("kart_tower_labyrinth", 700.0, 6.5, (120.0, 80.0), 3, 6),
];

/// Scenario: Karting circuits are packed indoor circuits with bridges
///
/// Given the 3 karting circuits
/// When their baked samples are measured
/// Then they have 1, 2 and 3 bridges, each at least 4.0 m clear and crossing at 30 degrees or more
/// And each fits its box with a density of at least 0.065 m per m2
#[test]
fn test_kart_circuits_are_packed_with_bridges() {
    for (id, design_len, width, (max_w, max_h), bridges, laps) in KART {
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
        let density = len / (w * h);
        assert!(density >= 0.065, "{}: density {:.3}", id, density);

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
        let grade = t
            .spline
            .samples
            .iter()
            .map(|s| s.grade_slope.abs())
            .fold(0.0, f32::max);
        assert!(grade <= 0.12, "{}: grade {:.3}", id, grade);

        assert_eq!(t.car_category, CarCategory::Kart, "{}", id);
        assert_eq!(t.default_surface, SurfaceType::Concrete, "{}", id);
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

/// Scenario: No turn right after a bridge
///
/// Given the 3 karting circuits
/// When a kart comes off a bridge (the last raised `is_bridge` sample)
/// Then the road runs straight for 20 m or more: its heading changes by 5 degrees or less
#[test]
fn test_kart_circuits_run_straight_for_20_m_after_each_bridge() {
    for (id, ..) in KART {
        let t = catalog::official_track("classic", id);
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
