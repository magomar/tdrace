use std::f32::consts::PI;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::spline::SplineSample;

/// Direction of an approaching or active curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveDirection {
    Left,
    Right,
}

impl CurveDirection {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
        }
    }
}

/// A discrete corner or curve along the racing circuit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackCurve {
    /// Zero-based sequential index of this curve along the track.
    pub id: usize,
    /// Turn direction: Left (<) or Right (>).
    pub direction: CurveDirection,
    /// Severity rating from 1 (very gentle kink) to 5 (acute / hairpin).
    pub degree: u8,
    /// Cumulative arc-length distance where the curve begins (meters).
    pub entry_distance: f32,
    /// Cumulative arc-length distance of maximum curvature / apex (meters).
    pub apex_distance: f32,
    /// Cumulative arc-length distance where the curve ends (meters).
    pub exit_distance: f32,
    /// Minimum curve radius at apex in meters.
    pub min_radius: f32,
    /// Peak curvature at apex (radians / meter).
    pub peak_curvature: f32,
    /// Total cumulative angular deflection across the curve (radians).
    pub total_turn_angle: f32,
    /// Physically safe cornering speed at the apex (m/s).
    pub safe_apex_speed_mps: f32,
    /// Cross-slope banking angle at apex in degrees.
    pub bank_angle: f32,
}

/// Real-time evaluation of an upcoming or active curve for the player car.
#[derive(Debug, Clone, PartialEq)]
pub struct CurveApproachStatus {
    /// The upcoming or currently active curve.
    pub curve: TrackCurve,
    /// Distance along track from car to curve entry (meters). Negative if inside curve.
    pub distance_to_entry: f32,
    /// Distance along track from car to curve apex (meters). Negative if past apex.
    pub distance_to_apex: f32,
    /// True if the car's current position is within [entry_distance, exit_distance].
    pub is_inside_curve: bool,
    /// Recommended safe braking distance from current speed down to safe apex speed (meters).
    pub required_braking_distance: f32,
    /// Dynamic urgency factor [0.0 = safe/cruise, 1.0 = critical braking required].
    pub urgency: f32,
    /// True if player must brake to safely make the corner without going off track.
    pub must_brake: bool,
}

/// Normalizes an angle into (-PI, PI].
#[inline]
fn normalize_angle(mut angle: f32) -> f32 {
    while angle > PI {
        angle -= 2.0 * PI;
    }
    while angle <= -PI {
        angle += 2.0 * PI;
    }
    angle
}

/// Computes the signed angle difference between two 2D unit tangent vectors.
/// Positive = counter-clockwise (Left turn), Negative = clockwise (Right turn).
#[inline]
pub fn angle_between_tangents(t0: Vec2, t1: Vec2) -> f32 {
    let cross = t0.x * t1.y - t0.y * t1.x;
    let dot = (t0.dot(t1)).clamp(-1.0, 1.0);
    normalize_angle(cross.atan2(dot))
}

/// Classifies curve severity into degree 1 through 5 based on minimum radius and total deflection angle.
///
/// Degree 1: High-speed gentle kink / sweep (R > 85m). Flat out or minor lift.
/// Degree 2: Mild curve (55m < R <= 85m). Gentle brake check or lift.
/// Degree 3: Medium corner (32m < R <= 55m). Noticeable braking.
/// Degree 4: Sharp turn (18m < R <= 32m). Hard threshold braking.
/// Degree 5: Hairpin / Acute (R <= 18m or R <= 24m with deflection > 85°). Full heavy braking.
pub fn classify_curve_degree(min_radius: f32, total_turn_angle_rad: f32) -> u8 {
    let angle_deg = total_turn_angle_rad.to_degrees().abs();

    if min_radius <= 18.0 || (min_radius <= 24.0 && angle_deg >= 80.0) || angle_deg >= 135.0 {
        5
    } else if min_radius <= 32.0 || (min_radius <= 40.0 && angle_deg >= 70.0) {
        4
    } else if min_radius <= 55.0 || (min_radius <= 65.0 && angle_deg >= 55.0) {
        3
    } else if min_radius <= 85.0 {
        2
    } else {
        1
    }
}

/// Calculates safe apex cornering speed (m/s) based on tire friction, banking, and radius.
pub fn compute_safe_apex_speed(radius: f32, bank_angle_deg: f32) -> f32 {
    let mu = 0.82; // Baseline dry asphalt tire grip coefficient
    let g = 9.81;
    let bank_rad = bank_angle_deg.to_radians().abs();
    let effective_grip = mu + bank_rad.tan().clamp(0.0, 0.65);
    let v_safe = (effective_grip * g * radius.max(1.0)).sqrt();
    v_safe.clamp(7.0, 75.0)
}

/// Scans finely sampled spline points and extracts all discrete curve segments along the circuit.
pub fn extract_curves_from_samples(
    samples: &[SplineSample],
    total_length: f32,
    closed: bool,
) -> Vec<TrackCurve> {
    if samples.len() < 4 || total_length < 30.0 {
        return Vec::new();
    }

    // Step 1: Compute local curvature kappa at each sample
    // Using a ~5-meter forward span
    let span_dist = 5.0f32;
    let mut curvatures = Vec::with_capacity(samples.len());

    for i in 0..samples.len() {
        let s0 = &samples[i];

        // Find sample near target forward span distance
        let mut best_idx = (i + 1) % samples.len();
        let mut best_diff = f32::INFINITY;
        let search_range = 16.min(samples.len());
        for step in 1..search_range {
            let idx = (i + step) % samples.len();
            let d_dist = if closed && samples[idx].distance < s0.distance {
                samples[idx].distance + total_length - s0.distance
            } else {
                (samples[idx].distance - s0.distance).abs()
            };
            let diff = (d_dist - span_dist).abs();
            if diff < best_diff {
                best_diff = diff;
                best_idx = idx;
            }
        }

        let s1 = &samples[best_idx];
        let actual_span = if closed && s1.distance < s0.distance {
            s1.distance + total_length - s0.distance
        } else {
            (s1.distance - s0.distance).abs()
        }
        .max(1.0);

        let d_theta = angle_between_tangents(s0.tangent, s1.tangent);
        let kappa = d_theta / actual_span;
        curvatures.push(kappa);
    }

    // Step 2: Identify contiguous regions where |kappa| > threshold (e.g. 0.007 rad/m -> R < 142m)
    let kappa_threshold = 0.007f32;
    let mut raw_curves: Vec<(usize, usize, CurveDirection)> = Vec::new();
    let mut in_curve = false;
    let mut current_dir = CurveDirection::Left;
    let mut start_idx = 0;

    let n = samples.len();
    for i in 0..n {
        let k = curvatures[i];
        let is_turning = k.abs() > kappa_threshold;
        let dir = if k > 0.0 {
            CurveDirection::Left
        } else {
            CurveDirection::Right
        };

        if is_turning {
            if !in_curve {
                in_curve = true;
                current_dir = dir;
                start_idx = i;
            } else if dir != current_dir {
                // Direction flipped (e.g. chicane transition)
                raw_curves.push((start_idx, i - 1, current_dir));
                current_dir = dir;
                start_idx = i;
            }
        } else if in_curve {
            in_curve = false;
            raw_curves.push((start_idx, i - 1, current_dir));
        }
    }

    if in_curve {
        raw_curves.push((start_idx, n - 1, current_dir));
    }

    // Handle wrap-around for closed tracks
    if closed && raw_curves.len() > 1 {
        let first = raw_curves.first().copied();
        let last = raw_curves.last().copied();
        if let (Some((f_start, f_end, f_dir)), Some((l_start, l_end, l_dir))) = (first, last) {
            if f_start == 0 && l_end == n - 1 && f_dir == l_dir {
                raw_curves.pop();
                raw_curves[0] = (l_start, f_end, f_dir);
            }
        }
    }

    // Step 3: Filter micro-noise and merge curves separated by very short straights (< 10m)
    let mut merged_curves: Vec<(usize, usize, CurveDirection)> = Vec::new();
    for (start, end, dir) in raw_curves {
        let length = if end >= start {
            samples[end].distance - samples[start].distance
        } else {
            (total_length - samples[start].distance) + samples[end].distance
        };

        // Discard very brief noise (< 8m)
        if length < 8.0 {
            continue;
        }

        if let Some(prev) = merged_curves.last_mut() {
            let gap = if start >= prev.1 {
                samples[start].distance - samples[prev.1].distance
            } else {
                (total_length - samples[prev.1].distance) + samples[start].distance
            };

            // Merge if same direction and gap is small (< 10m)
            if prev.2 == dir && gap < 10.0 {
                prev.1 = end;
                continue;
            }
        }

        merged_curves.push((start, end, dir));
    }

    // Step 4: Build final TrackCurve structs
    let mut curves = Vec::with_capacity(merged_curves.len());
    for (curve_id, (start_idx, end_idx, direction)) in merged_curves.into_iter().enumerate() {
        let entry_distance = samples[start_idx].distance;
        let exit_distance = samples[end_idx].distance;

        // Find apex (maximum |kappa|) and total angle
        let mut max_kappa = 0.0f32;
        let mut apex_idx = start_idx;
        let mut total_angle = 0.0f32;

        let num_pts = if end_idx >= start_idx {
            end_idx - start_idx + 1
        } else {
            (n - start_idx) + end_idx + 1
        };

        for step in 0..num_pts {
            let idx = (start_idx + step) % n;
            let k = curvatures[idx].abs();
            if k > max_kappa {
                max_kappa = k;
                apex_idx = idx;
            }
            if step > 0 {
                let prev_idx = (start_idx + step - 1) % n;
                let seg_angle = angle_between_tangents(samples[prev_idx].tangent, samples[idx].tangent).abs();
                total_angle += seg_angle;
            }
        }

        let apex_distance = samples[apex_idx].distance;
        let bank_angle = samples[apex_idx].bank_angle;
        let min_radius = if max_kappa > 1e-5 {
            1.0 / max_kappa
        } else {
            150.0
        };

        let degree = classify_curve_degree(min_radius, total_angle);
        let safe_apex_speed_mps = compute_safe_apex_speed(min_radius, bank_angle);

        curves.push(TrackCurve {
            id: curve_id,
            direction,
            degree,
            entry_distance,
            apex_distance,
            exit_distance,
            min_radius,
            peak_curvature: max_kappa,
            total_turn_angle: total_angle,
            safe_apex_speed_mps,
            bank_angle,
        });
    }

    curves
}

/// Evaluates upcoming curve proximity, required braking distance, and dynamic urgency for the car.
pub fn evaluate_curve_approach(
    curves: &[TrackCurve],
    current_dist: f32,
    total_length: f32,
    closed: bool,
    car_speed_mps: f32,
    max_lookahead: f32,
) -> Option<CurveApproachStatus> {
    if curves.is_empty() || total_length <= 0.0 {
        return None;
    }

    let a_brake = 7.2f32; // Nominal safe deceleration rate (m/s²)
    let warning_buffer = 35.0f32; // Buffer distance for smooth green -> yellow -> red transition (meters)

    let mut best_status: Option<CurveApproachStatus> = None;
    let mut best_forward_dist = f32::INFINITY;

    for curve in curves {
        // Compute forward track distance from current_dist to curve entry and exit
        let dist_to_entry = if closed {
            let mut d = curve.entry_distance - current_dist;
            if d < 0.0 {
                d += total_length;
            }
            d
        } else {
            curve.entry_distance - current_dist
        };

        let dist_to_exit = if closed {
            let mut d = curve.exit_distance - current_dist;
            if d < 0.0 {
                d += total_length;
            }
            d
        } else {
            curve.exit_distance - current_dist
        };

        let dist_to_apex = if closed {
            let mut d = curve.apex_distance - current_dist;
            if d < 0.0 {
                d += total_length;
            }
            d
        } else {
            curve.apex_distance - current_dist
        };

        // Determine if currently inside this curve
        let curve_span = if curve.exit_distance >= curve.entry_distance {
            curve.exit_distance - curve.entry_distance
        } else {
            (total_length - curve.entry_distance) + curve.exit_distance
        };

        let is_inside = dist_to_exit <= curve_span && dist_to_entry > (total_length - curve_span);

        let forward_check_dist = if is_inside { 0.0 } else { dist_to_entry };

        if forward_check_dist > max_lookahead && !is_inside {
            continue;
        }

        if forward_check_dist < best_forward_dist {
            best_forward_dist = forward_check_dist;

            // Compute braking distance required
            let required_braking_distance = if car_speed_mps > curve.safe_apex_speed_mps {
                (car_speed_mps * car_speed_mps - curve.safe_apex_speed_mps * curve.safe_apex_speed_mps)
                    / (2.0 * a_brake)
            } else {
                0.0
            };

            // Compute dynamic urgency factor [0.0 to 1.0]
            let (urgency, must_brake) = if is_inside {
                // If inside before apex and still over-speeding: critical
                if dist_to_apex < (total_length * 0.5) && car_speed_mps > curve.safe_apex_speed_mps {
                    (1.0, true)
                } else {
                    (0.0, false)
                }
            } else if required_braking_distance <= 0.0 {
                // Already traveling at or below safe cornering speed!
                (0.0, false)
            } else if dist_to_entry <= required_braking_distance {
                // Inside the hard braking zone!
                (1.0, true)
            } else if dist_to_entry <= required_braking_distance + warning_buffer {
                // Approaching braking zone: smooth ramp 0.0 -> 1.0
                let t = (required_braking_distance + warning_buffer - dist_to_entry) / warning_buffer;
                (t.clamp(0.0, 1.0), t >= 0.85)
            } else {
                // Plenty of runway ahead
                (0.0, false)
            };

            let signed_entry_dist = if is_inside {
                -(curve_span - dist_to_exit)
            } else {
                dist_to_entry
            };

            let signed_apex_dist = if dist_to_apex > total_length * 0.5 {
                dist_to_apex - total_length
            } else {
                dist_to_apex
            };

            best_status = Some(CurveApproachStatus {
                curve: curve.clone(),
                distance_to_entry: signed_entry_dist,
                distance_to_apex: signed_apex_dist,
                is_inside_curve: is_inside,
                required_braking_distance,
                urgency,
                must_brake,
            });
        }
    }

    best_status
}
