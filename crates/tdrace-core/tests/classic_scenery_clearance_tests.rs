//! Spec 106 (`specs/106_classic_autocross_launch_chutes_and_tencar_rallycross_chutes.md`): no grandstand or building
//! of a Classic circuit stands on the road or inside its wall line, where cars can hit it.

use tdrace_core::catalog;
use glam::Vec2;

/// Points spread over a footprint given by its 4 corners in order.
fn footprint(corners: [Vec2; 4]) -> Vec<Vec2> {
    let n = 8;
    (0..=n)
        .flat_map(|i| (0..=n).map(move |j| (i as f32 / n as f32, j as f32 / n as f32)))
        .map(|(u, v)| corners[0] + (corners[1] - corners[0]) * u + (corners[3] - corners[0]) * v)
        .collect()
}

/// Scenario: No grandstand inside the walls
///
/// Given the 18 Classic circuits
/// When each grandstand and building footprint is compared with the road and its wall line
/// Then none stands on the road or inside the wall line
#[test]
fn test_no_classic_grandstand_or_building_stands_inside_the_wall_line() {
    let mut problems = Vec::new();
    let circuits: Vec<_> = catalog::module_circuits("classic").collect();
    assert_eq!(circuits.len(), 18);
    for c in circuits {
        let track = c.load().unwrap_or_else(|e| panic!("classic/{}: {e}", c.id));
        let g = &track.geometry;
        let items = g
            .grandstands
            .iter()
            .map(|s| (format!("grandstand {}", s.id), s.corners()))
            .chain(g.buildings.iter().map(|b| (format!("building {}", b.id), b.corners())));
        for (name, corners) in items {
            // How far the footprint reaches past the wall line, beside each road sample (positive = inside it).
            let inside = footprint(corners)
                .into_iter()
                .flat_map(|p| track.spline.samples.iter().map(move |s| (p, s)))
                .filter(|(p, s)| (*p - s.point).dot(s.tangent).abs() <= 1.5)
                .map(|(p, s)| {
                    let lateral = (p - s.point).dot(s.normal);
                    let (wall, gap) = if lateral > 0.0 { (s.left_wall, s.left_wall_distance) } else { (s.right_wall, s.right_wall_distance) };
                    let corridor = s.width * 0.5 + if wall { gap.unwrap_or(0.0) } else { 0.0 };
                    corridor - lateral.abs()
                })
                .fold(f32::MIN, f32::max);
            if inside > 0.0 {
                problems.push(format!("classic/{}: {name} stands {inside:.1} m inside the wall line", c.id));
            }
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}
