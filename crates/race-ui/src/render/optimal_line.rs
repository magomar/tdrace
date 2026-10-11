//! 2D ribbon mesh renderer for the mathematically optimal racing line visual aid.
//!
//! Implements Spec 110:
//! - Constrained minimum-curvature path rendering
//! - Real-time speed-adaptive dynamic coloration (Green -> Yellow -> Red)
//! - Lookahead ribbon slicing (-15m to +125m) with smooth edge fades
//! - Edge antialiasing via feathered lateral alpha gradients
//! - Complete zero-overhead early return when mode is `Disabled`

use glam::Vec2;
use macroquad::color::Color;
use arcade_race_core::track::{
    optimal_line::{OptimalRacingLine, RacingLineMode},
    spline::TrackSpline,
};

/// Evaluates the dynamic speed-adaptive color for a given node along the optimal trajectory.
#[inline]
pub fn compute_racing_line_color(
    mode: RacingLineMode,
    current_speed: f32,
    target_speed: f32,
    base_alpha: f32,
) -> Color {
    let delta_v = current_speed - target_speed;
    const SAFE_SPEED_MARGIN: f32 = 3.5; // m/s (~12.6 km/h)

    match mode {
        RacingLineMode::Disabled => Color::new(0.0, 0.0, 0.0, 0.0),
        RacingLineMode::LineOnly => {
            // Neon cyan / teal uniform line
            Color::new(0.18, 0.90, 0.65, base_alpha)
        }
        RacingLineMode::BrakingOnly => {
            if delta_v <= 0.0 {
                Color::new(0.0, 0.0, 0.0, 0.0)
            } else {
                let urgency = (delta_v / SAFE_SPEED_MARGIN).clamp(0.0, 1.0);
                let hard_brake = ((delta_v - SAFE_SPEED_MARGIN) / 5.0).clamp(0.0, 1.0);

                // Blend from Electric Yellow (0.98, 0.88, 0.12) to Crimson Red (0.96, 0.12, 0.20)
                let r = 0.98 + hard_brake * (0.96 - 0.98);
                let g = 0.88 + hard_brake * (0.12 - 0.88);
                let b = 0.12 + hard_brake * (0.20 - 0.12);
                let a = base_alpha * urgency;

                Color::new(r, g, b, a)
            }
        }
        RacingLineMode::Full => {
            if delta_v <= 0.0 {
                // Vibrant Green (acceleration / full throttle)
                Color::new(0.12, 0.92, 0.35, base_alpha)
            } else if delta_v <= SAFE_SPEED_MARGIN {
                // Lift / Coast / Turn-in: blend Green to Yellow
                let t = (delta_v / SAFE_SPEED_MARGIN).clamp(0.0, 1.0);
                let r = 0.12 + t * (0.98 - 0.12);
                let g = 0.92 + t * (0.88 - 0.92);
                let b = 0.35 + t * (0.12 - 0.35);
                Color::new(r, g, b, base_alpha)
            } else {
                // Hard braking required: blend Yellow to Crimson Red
                let t = ((delta_v - SAFE_SPEED_MARGIN) / 5.0).clamp(0.0, 1.0);
                let r = 0.98 + t * (0.96 - 0.98);
                let g = 0.88 + t * (0.12 - 0.88);
                let b = 0.12 + t * (0.20 - 0.12);
                Color::new(r, g, b, base_alpha)
            }
        }
    }
}

/// Renders the mathematically optimal racing line 2D ribbon mesh with speed-adaptive dynamic coloration.
pub fn render_optimal_racing_line(
    spline: &TrackSpline,
    optimal_line: &OptimalRacingLine,
    player_pos: Vec2,
    player_speed_mps: f32,
    mode: RacingLineMode,
    opacity: f32,
    view_bounds: Option<(Vec2, Vec2)>,
) {
    if mode == RacingLineMode::Disabled || opacity <= 0.001 || optimal_line.nodes.len() < 2 {
        return;
    }

    let n = optimal_line.nodes.len();
    let total_len = optimal_line.total_length;

    // 1. Locate player along the trajectory
    let mut closest_idx = 0;
    let mut min_dist_sq = f32::MAX;
    for (i, node) in optimal_line.nodes.iter().enumerate() {
        let d_sq = (node.point - player_pos).length_squared();
        if d_sq < min_dist_sq {
            min_dist_sq = d_sq;
            closest_idx = i;
        }
    }

    let player_dist = optimal_line.nodes[closest_idx].distance;
    let speed = player_speed_mps.abs();

    const BEHIND_WINDOW_M: f32 = 15.0;
    const AHEAD_WINDOW_M: f32 = 125.0;
    const HALF_WIDTH: f32 = 0.375; // 0.75m total width

    let mut builder = crate::render::track::BatchMeshBuilder::new(None);

    let num_segments = if spline.closed { n } else { n - 1 };

    for i in 0..num_segments {
        let ip1 = (i + 1) % n;
        let node0 = &optimal_line.nodes[i];
        let node1 = &optimal_line.nodes[ip1];

        // Jump / discontinuity check
        if (node1.point - node0.point).length_squared() > 625.0 {
            continue;
        }

        // Relative distance from player for both nodes
        let rel_dist0 = if spline.closed && total_len > 0.0 {
            let mut d = (node0.distance - player_dist) % total_len;
            if d > total_len * 0.5 {
                d -= total_len;
            } else if d < -total_len * 0.5 {
                d += total_len;
            }
            d
        } else {
            node0.distance - player_dist
        };

        let rel_dist1 = if spline.closed && total_len > 0.0 {
            let mut d = (node1.distance - player_dist) % total_len;
            if d > total_len * 0.5 {
                d -= total_len;
            } else if d < -total_len * 0.5 {
                d += total_len;
            }
            d
        } else {
            node1.distance - player_dist
        };

        // Window culling: at least one node must be inside [-15m, 125m]
        let in_window = (rel_dist0 >= -BEHIND_WINDOW_M && rel_dist0 <= AHEAD_WINDOW_M)
            || (rel_dist1 >= -BEHIND_WINDOW_M && rel_dist1 <= AHEAD_WINDOW_M);

        if !in_window {
            continue;
        }

        // Viewport culling
        if let Some((min, max)) = view_bounds {
            let seg_min_x = node0.point.x.min(node1.point.x) - 4.0;
            let seg_max_x = node0.point.x.max(node1.point.x) + 4.0;
            let seg_min_y = node0.point.y.min(node1.point.y) - 4.0;
            let seg_max_y = node0.point.y.max(node1.point.y) + 4.0;

            if seg_max_x < min.x || seg_min_x > max.x || seg_max_y < min.y || seg_min_y > max.y {
                continue;
            }
        }

        // Window fade calculation
        let fade0 = if rel_dist0 < -BEHIND_WINDOW_M || rel_dist0 > AHEAD_WINDOW_M {
            0.0
        } else if rel_dist0 < -5.0 {
            ((rel_dist0 + BEHIND_WINDOW_M) / (BEHIND_WINDOW_M - 5.0)).clamp(0.0, 1.0)
        } else if rel_dist0 > 95.0 {
            ((AHEAD_WINDOW_M - rel_dist0) / (AHEAD_WINDOW_M - 95.0)).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let fade1 = if rel_dist1 < -BEHIND_WINDOW_M || rel_dist1 > AHEAD_WINDOW_M {
            0.0
        } else if rel_dist1 < -5.0 {
            ((rel_dist1 + BEHIND_WINDOW_M) / (BEHIND_WINDOW_M - 5.0)).clamp(0.0, 1.0)
        } else if rel_dist1 > 95.0 {
            ((AHEAD_WINDOW_M - rel_dist1) / (AHEAD_WINDOW_M - 95.0)).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let alpha0 = opacity * fade0;
        let alpha1 = opacity * fade1;

        if alpha0 <= 0.001 && alpha1 <= 0.001 {
            continue;
        }

        let col0 = compute_racing_line_color(mode, speed, node0.target_speed, alpha0);
        let col1 = compute_racing_line_color(mode, speed, node1.target_speed, alpha1);

        if col0.a <= 0.001 && col1.a <= 0.001 {
            continue;
        }

        // Compute 2D lateral normals (perpendicular to tangent)
        let norm0 = Vec2::new(-node0.tangent.y, node0.tangent.x);
        let norm1 = Vec2::new(-node1.tangent.y, node1.tangent.x);

        let p_l0 = node0.point + norm0 * HALF_WIDTH;
        let p_c0 = node0.point;
        let p_r0 = node0.point - norm0 * HALF_WIDTH;

        let p_l1 = node1.point + norm1 * HALF_WIDTH;
        let p_c1 = node1.point;
        let p_r1 = node1.point - norm1 * HALF_WIDTH;

        // Soft edge antialiasing: outer edge alpha is attenuated
        const EDGE_ALPHA_FACTOR: f32 = 0.15;
        let c_edge0 = Color::new(col0.r, col0.g, col0.b, col0.a * EDGE_ALPHA_FACTOR);
        let c_edge1 = Color::new(col1.r, col1.g, col1.b, col1.a * EDGE_ALPHA_FACTOR);

        let zero_uv = macroquad::prelude::Vec2::ZERO;

        // Quad 1: Left ribbon half (Outer Left -> Center)
        builder.push_quad(
            p_l0, zero_uv, c_edge0,
            p_c0, zero_uv, col0,
            p_c1, zero_uv, col1,
            p_l1, zero_uv, c_edge1,
        );

        // Quad 2: Right ribbon half (Center -> Outer Right)
        builder.push_quad(
            p_c0, zero_uv, col0,
            p_r0, zero_uv, c_edge0,
            p_r1, zero_uv, c_edge1,
            p_c1, zero_uv, col1,
        );
    }

    builder.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use arcade_race_core::track::optimal_line::OptimalLineNode;

    #[test]
    fn test_compute_racing_line_color_modes() {
        // Disabled mode always returns alpha 0
        let c_dis = compute_racing_line_color(RacingLineMode::Disabled, 50.0, 20.0, 0.8);
        assert_eq!(c_dis.a, 0.0);

        // LineOnly mode returns uniform color
        let c_line = compute_racing_line_color(RacingLineMode::LineOnly, 50.0, 20.0, 0.8);
        assert!((c_line.a - 0.8).abs() < 1e-4);
        assert!((c_line.r - 0.18).abs() < 1e-2);
        assert!((c_line.g - 0.90).abs() < 1e-2);

        // BrakingOnly mode returns alpha 0 when below target speed
        let c_brk_safe = compute_racing_line_color(RacingLineMode::BrakingOnly, 15.0, 20.0, 0.8);
        assert_eq!(c_brk_safe.a, 0.0);

        // BrakingOnly mode returns red when well above target speed
        let c_brk_hard = compute_racing_line_color(RacingLineMode::BrakingOnly, 50.0, 20.0, 0.8);
        assert!(c_brk_hard.a > 0.7);
        assert!(c_brk_hard.r > 0.9);
        assert!(c_brk_hard.g < 0.25);

        // Full mode returns green when below target speed
        let c_full_green = compute_racing_line_color(RacingLineMode::Full, 15.0, 20.0, 0.8);
        assert!(c_full_green.g > 0.85);
        assert!(c_full_green.r < 0.2);

        // Full mode returns yellow at threshold
        let c_full_yellow = compute_racing_line_color(RacingLineMode::Full, 23.5, 20.0, 0.8);
        assert!(c_full_yellow.r > 0.8);
        assert!(c_full_yellow.g > 0.8);

        // Full mode returns crimson red when fast
        let c_full_red = compute_racing_line_color(RacingLineMode::Full, 50.0, 20.0, 0.8);
        assert!(c_full_red.r > 0.9);
        assert!(c_full_red.g < 0.25);
    }

    #[test]
    fn test_render_optimal_racing_line_headless_execution() {
        let spline = TrackSpline::default();
        let mut nodes = Vec::new();
        for i in 0..10 {
            let s = i as f32 * 10.0;
            nodes.push(OptimalLineNode {
                point: Vec2::new(s, 0.0),
                tangent: Vec2::X,
                lateral_offset: 0.0,
                distance: s,
                spline_distance: s,
                curvature: 0.0,
                target_speed: 25.0,
            });
        }
        let optimal_line = OptimalRacingLine {
            nodes,
            total_length: 90.0,
            integrated_curvature: 0.0,
        };

        // Render in Disabled mode
        render_optimal_racing_line(
            &spline,
            &optimal_line,
            Vec2::ZERO,
            20.0,
            RacingLineMode::Disabled,
            0.7,
            None,
        );

        // Render in Full mode
        render_optimal_racing_line(
            &spline,
            &optimal_line,
            Vec2::new(30.0, 0.0),
            40.0,
            RacingLineMode::Full,
            0.7,
            Some((Vec2::new(-50.0, -50.0), Vec2::new(100.0, 100.0))),
        );
    }
}
