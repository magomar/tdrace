use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::spline::TrackSpline;

/// Racing line visual aid display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RacingLineMode {
    #[default]
    Full,
    LineOnly,
    BrakingOnly,
    Disabled,
}

impl RacingLineMode {
    pub fn next(&self) -> Self {
        match self {
            Self::Full => Self::LineOnly,
            Self::LineOnly => Self::BrakingOnly,
            Self::BrakingOnly => Self::Disabled,
            Self::Disabled => Self::Full,
        }
    }

    pub fn toast_label(&self) -> &'static str {
        match self {
            Self::Full => "FULL",
            Self::LineOnly => "LINE ONLY",
            Self::BrakingOnly => "BRAKING ONLY",
            Self::Disabled => "OFF",
        }
    }

    pub fn to_index(&self) -> usize {
        match self {
            Self::Full => 0,
            Self::LineOnly => 1,
            Self::BrakingOnly => 2,
            Self::Disabled => 3,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Self::LineOnly,
            2 => Self::BrakingOnly,
            3 => Self::Disabled,
            _ => Self::Full,
        }
    }

    pub fn shows_line(&self) -> bool {
        matches!(self, Self::Full | Self::LineOnly)
    }

    pub fn shows_braking(&self) -> bool {
        matches!(self, Self::Full | Self::BrakingOnly)
    }
}

/// Configuration parameters for the minimum-curvature optimal racing line solver.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimalLineConfig {
    /// Safe clearance margin from track boundary edges / walls in meters (default: 1.2m).
    pub boundary_margin_m: f32,
    /// Width of curb considered drivable beyond the asphalt ribbon in meters (default: 0.8m).
    pub curb_extension_m: f32,
    /// Maximum solver iterations (default: 80).
    pub max_iterations: usize,
    /// Successive over-relaxation (SOR) damping parameter (default: 0.55).
    pub sor_damping: f32,
    /// Regularization weight pulling the line toward the track centerline on straights (default: 0.015).
    pub centerline_bias: f32,
    /// Nominal braking deceleration limit in m/s^2 for velocity profiling (default: 9.0).
    pub max_braking_decel_mps2: f32,
    /// Nominal acceleration limit in m/s^2 for corner exit velocity profiling (default: 4.8).
    pub max_accel_mps2: f32,
    /// Baseline tire grip coefficient (default: 0.85).
    pub baseline_grip: f32,
}

impl Default for OptimalLineConfig {
    fn default() -> Self {
        Self {
            boundary_margin_m: 1.2,
            curb_extension_m: 0.8,
            max_iterations: 80,
            sor_damping: 0.55,
            centerline_bias: 0.015,
            max_braking_decel_mps2: 9.0,
            max_accel_mps2: 4.8,
            baseline_grip: 0.85,
        }
    }
}

/// A discrete node along the mathematically ideal racing line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimalLineNode {
    /// 2D world coordinates along the optimal trajectory.
    pub point: Vec2,
    /// Unit tangent vector along the trajectory.
    pub tangent: Vec2,
    /// Lateral displacement from track spline centerline in meters (+ = left, - = right).
    pub lateral_offset: f32,
    /// Cumulative arc-length distance along the optimal racing line (meters).
    pub distance: f32,
    /// Matching distance along the underlying track spline (meters).
    pub spline_distance: f32,
    /// Local trajectory curvature (radians / meter, 1 / radius).
    pub curvature: f32,
    /// Calculated target cornering / transit speed in m/s.
    pub target_speed: f32,
}

/// Dynamic evaluation of upcoming braking requirements for a driver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrakingUrgency {
    /// Dynamic braking urgency [0.0 = full throttle / cruise, 1.0 = hard threshold braking required].
    pub urgency: f32,
    /// Recommended target transit speed at the upcoming critical point (m/s).
    pub target_speed: f32,
    /// Total distance required to decelerate to target speed at current speed (meters).
    pub required_braking_dist: f32,
    /// Track distance to the braking onset point (meters; <= 0 means within or past braking zone).
    pub distance_to_brake_point: f32,
    /// True if the vehicle's current speed exceeds safe cornering speed and requires braking.
    pub must_brake: bool,
}

/// Complete mathematically optimal trajectory for a circuit or layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OptimalRacingLine {
    /// Solved discrete nodes along the circuit.
    pub nodes: Vec<OptimalLineNode>,
    /// Total cumulative arc-length of the optimal trajectory in meters.
    pub total_length: f32,
    /// Total integrated squared curvature across the entire line (sum of kappa^2 * ds).
    pub integrated_curvature: f32,
}

impl OptimalRacingLine {
    /// Solves the constrained minimum-curvature quadratic optimization problem using default configuration.
    pub fn compute_for_spline(spline: &TrackSpline) -> Self {
        Self::compute_for_spline_with_config(spline, &OptimalLineConfig::default())
    }

    /// Solves the constrained minimum-curvature trajectory with custom parameters.
    pub fn compute_for_spline_with_config(spline: &TrackSpline, config: &OptimalLineConfig) -> Self {
        let n = spline.samples.len();
        if n < 4 {
            return Self::default();
        }

        let is_closed = spline.closed;

        // Step 1: Compute lateral displacement bounds for each sample
        let mut min_offsets = Vec::with_capacity(n);
        let mut max_offsets = Vec::with_capacity(n);

        for s in &spline.samples {
            let left_curb_extra = if s.left_curb { config.curb_extension_m } else { 0.0 };
            let right_curb_extra = if s.right_curb { config.curb_extension_m } else { 0.0 };

            let max_left = (s.width * 0.5 + left_curb_extra - config.boundary_margin_m).max(0.0);
            let max_right = (s.width * 0.5 + right_curb_extra - config.boundary_margin_m).max(0.0);

            min_offsets.push(-max_right);
            max_offsets.push(max_left);
        }

        // Step 2: Initialize lateral displacements to 0.0 (centerline)
        let mut offsets = vec![0.0f32; n];

        // Step 3: Iterative Projected Successive Over-Relaxation (Biharmonic Curvature Minimization)
        let omega = config.sor_damping;
        let lambda = config.centerline_bias;

        for _iter in 0..config.max_iterations {
            for i in 0..n {
                // Neighbors with periodic or clamped boundary handling
                let (im2, im1, ip1, ip2) = if is_closed {
                    (
                        (i + n - 2) % n,
                        (i + n - 1) % n,
                        (i + 1) % n,
                        (i + 2) % n,
                    )
                } else {
                    (
                        i.saturating_sub(2),
                        i.saturating_sub(1),
                        (i + 1).min(n - 1),
                        (i + 2).min(n - 1),
                    )
                };

                let p_im2 = spline.samples[im2].point + spline.samples[im2].normal * offsets[im2];
                let p_im1 = spline.samples[im1].point + spline.samples[im1].normal * offsets[im1];
                let p_ip1 = spline.samples[ip1].point + spline.samples[ip1].normal * offsets[ip1];
                let p_ip2 = spline.samples[ip2].point + spline.samples[ip2].normal * offsets[ip2];

                // Biharmonic stencil target: 6 r_i = 4(r_{i-1} + r_{i+1}) - (r_{i-2} + r_{i+2})
                let p_target = (4.0 * (p_im1 + p_ip1) - (p_im2 + p_ip2)) / 6.0;

                // Project target point onto sample normal vector relative to centerline
                let normal = spline.samples[i].normal;
                let c_i = spline.samples[i].point;
                let unconstrained_offset = (p_target - c_i).dot(normal);

                // Apply mild centerline bias
                let biased_offset = unconstrained_offset / (1.0 + lambda);

                // SOR update and box constraint projection
                let new_offset = (1.0 - omega) * offsets[i] + omega * biased_offset;
                offsets[i] = new_offset.clamp(min_offsets[i], max_offsets[i]);
            }
        }

        // Step 4: Construct trajectory points and initial tangents
        let mut raw_points = Vec::with_capacity(n);
        for i in 0..n {
            let pt = spline.samples[i].point + spline.samples[i].normal * offsets[i];
            raw_points.push(pt);
        }

        // Step 5: Compute Menger curvature at each node
        let mut curvatures = Vec::with_capacity(n);
        for i in 0..n {
            let im1 = if is_closed { (i + n - 1) % n } else { i.saturating_sub(1) };
            let ip1 = if is_closed { (i + 1) % n } else { (i + 1).min(n - 1) };

            let p0 = raw_points[im1];
            let p1 = raw_points[i];
            let p2 = raw_points[ip1];

            let a = (p1 - p0).length();
            let b = (p2 - p1).length();
            let c = (p2 - p0).length();

            let cross = (p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x);
            let denom = a * b * c;
            let kappa = if denom > 1e-4 {
                (2.0 * cross.abs() / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            curvatures.push(kappa);
        }

        // Step 6: Kinematic cornering speed envelope
        let g = 9.81f32;
        let mut target_speeds = Vec::with_capacity(n);
        for i in 0..n {
            let s = &spline.samples[i];
            let mu = s.surface.friction_coefficient().max(0.40) * config.baseline_grip;
            let bank_rad = s.bank_angle.to_radians().abs();
            let effective_mu = mu + bank_rad.tan().clamp(0.0, 0.50);

            let kappa = curvatures[i];
            let v_corner = if kappa > 1e-4 {
                let radius = 1.0 / kappa;
                (effective_mu * g * radius).sqrt().clamp(7.0, 75.0)
            } else {
                75.0
            };
            target_speeds.push(v_corner);
        }

        // Step 7: Two-pass velocity profiling (backward braking + forward acceleration)
        // Repeat 2 full passes for closed circuits to guarantee boundary equilibrium
        let passes = if is_closed { 2 } else { 1 };
        let a_brake = config.max_braking_decel_mps2;
        let a_accel = config.max_accel_mps2;

        for _ in 0..passes {
            // Backward pass (deceleration)
            for idx in 0..n {
                let i = (n - 1 + n - idx) % n;
                let ip1 = (i + 1) % n;
                if !is_closed && i == n - 1 {
                    continue;
                }
                let ds = (raw_points[ip1] - raw_points[i]).length().max(0.1);
                let v_next = target_speeds[ip1];
                let max_v = (v_next * v_next + 2.0 * a_brake * ds).sqrt();
                if max_v < target_speeds[i] {
                    target_speeds[i] = max_v;
                }
            }

            // Forward pass (traction acceleration)
            for idx in 0..n {
                let i = idx;
                let im1 = if i == 0 { if is_closed { n - 1 } else { 0 } } else { i - 1 };
                if !is_closed && i == 0 {
                    continue;
                }
                let ds = (raw_points[i] - raw_points[im1]).length().max(0.1);
                let v_prev = target_speeds[im1];
                let max_v = (v_prev * v_prev + 2.0 * a_accel * ds).sqrt();
                if max_v < target_speeds[i] {
                    target_speeds[i] = max_v;
                }
            }
        }

        // Step 8: Build nodes with tangents and arc-length distances
        let mut nodes = Vec::with_capacity(n);
        let mut total_dist = 0.0f32;
        let mut integrated_curvature = 0.0f32;

        for i in 0..n {
            let pt = raw_points[i];
            let ip1 = if is_closed { (i + 1) % n } else { (i + 1).min(n - 1) };
            let tangent = if ip1 != i {
                (raw_points[ip1] - pt).normalize_or_zero()
            } else {
                spline.samples[i].tangent
            };

            let ds = (raw_points[ip1] - pt).length();
            integrated_curvature += curvatures[i] * curvatures[i] * ds;

            nodes.push(OptimalLineNode {
                point: pt,
                tangent,
                lateral_offset: offsets[i],
                distance: total_dist,
                spline_distance: spline.samples[i].distance,
                curvature: curvatures[i],
                target_speed: target_speeds[i],
            });

            total_dist += ds;
        }

        Self {
            nodes,
            total_length: total_dist,
            integrated_curvature,
        }
    }

    /// Evaluates upcoming dynamic braking requirements given a distance along the spline and vehicle speed.
    pub fn evaluate_braking(&self, spline_dist: f32, current_speed_mps: f32) -> BrakingUrgency {
        if self.nodes.is_empty() {
            return BrakingUrgency {
                urgency: 0.0,
                target_speed: current_speed_mps,
                required_braking_dist: 0.0,
                distance_to_brake_point: f32::INFINITY,
                must_brake: false,
            };
        }

        let total_spline_len = self.nodes.last().map(|n| n.spline_distance).unwrap_or(1.0);
        let s_norm = spline_dist % total_spline_len;

        // Search forward up to 130 meters ahead
        let max_lookahead_m = 130.0f32;
        let a_brake = 9.0f32;

        let mut max_urgency = 0.0f32;
        let mut critical_target_speed = current_speed_mps;
        let mut critical_brake_dist = 0.0f32;
        let mut min_dist_to_brake_point = f32::INFINITY;
        let mut must_brake = false;

        let n = self.nodes.len();
        for i in 0..n {
            let node = &self.nodes[i];
            let dist_ahead = if node.spline_distance >= s_norm {
                node.spline_distance - s_norm
            } else {
                node.spline_distance + total_spline_len - s_norm
            };

            if dist_ahead > max_lookahead_m {
                continue;
            }

            if node.target_speed < current_speed_mps - 1.0 {
                let speed_diff_sq = current_speed_mps * current_speed_mps - node.target_speed * node.target_speed;
                let req_dist = speed_diff_sq / (2.0 * a_brake);
                let dist_to_brake = dist_ahead - req_dist;

                let urgency = if req_dist > 0.0 {
                    ((req_dist + 5.0) / dist_ahead.max(1.0)).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                if urgency > max_urgency {
                    max_urgency = urgency;
                    critical_target_speed = node.target_speed;
                    critical_brake_dist = req_dist;
                    min_dist_to_brake_point = dist_to_brake;
                    must_brake = dist_to_brake <= 4.0;
                }
            }
        }

        BrakingUrgency {
            urgency: max_urgency,
            target_speed: critical_target_speed,
            required_braking_dist: critical_brake_dist,
            distance_to_brake_point: min_dist_to_brake_point,
            must_brake,
        }
    }
}
