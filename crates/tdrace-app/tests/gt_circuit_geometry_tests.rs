//! GT circuit preset geometry checks: apex kerbs follow real corners and the scale label matches
//! the 0.5x OpenStreetMap import.

use tdrace_app::module::{gt::GtWorldChallengeModule, GameModule};

/// Sine of the deflection angle at waypoint `i` of a closed waypoint loop.
fn deflection_sine(points: &[glam::Vec2], i: usize) -> f32 {
    let n = points.len();
    let v1 = points[i] - points[(i + n - 1) % n];
    let v2 = points[(i + 1) % n] - points[i];
    v1.perp_dot(v2) / (v1.length() * v2.length())
}

#[test]
fn test_gt_kerbs_only_on_real_corners() {
    for def in GtWorldChallengeModule::new().tracks() {
        let track = (def.generator)();
        let points: Vec<glam::Vec2> = track.spline.waypoints.iter().map(|wp| wp.point).collect();

        for (i, wp) in track.spline.waypoints.iter().enumerate() {
            let s = deflection_sine(&points, i);
            // Tolerance covers the 0.1 m rounding of the pasted waypoint coordinates.
            if wp.left_curb {
                assert!(s > 0.34, "{} waypoint {} has a left kerb on a {:.3} deflection", def.id, i, s);
            }
            if wp.right_curb {
                assert!(s < -0.34, "{} waypoint {} has a right kerb on a {:.3} deflection", def.id, i, s);
            }
        }
    }
}

#[test]
fn test_gt_tracks_declare_half_scale() {
    for def in GtWorldChallengeModule::new().tracks() {
        let track = (def.generator)();
        assert_eq!(track.scale(), "0.5x", "{} is imported at 0.5x FIA length", def.id);
    }
}
