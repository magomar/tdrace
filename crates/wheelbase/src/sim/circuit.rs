use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::car::{normalize_angle, CarControls};
use crate::config::CarConfig;
use crate::surface::SurfaceType;
use super::harness::SimulationRunner;

const G_ACCEL: f32 = 9.80665;

/// Pre-sampled point along a simulation reference path.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SimPathPoint {
    pub position: Vec2,
    pub tangent: Vec2,
    pub normal: Vec2,
    pub curvature: f32,
    pub distance: f32,
}

/// Parametric reference trajectory for path-following and circuit dynamics simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimPath {
    pub name: String,
    pub points: Vec<SimPathPoint>,
    pub total_length: f32,
    pub is_closed: bool,
}

impl SimPath {
    /// Builds a simulation path from discrete waypoints by linear or arc interpolation.
    pub fn from_waypoints(name: impl Into<String>, waypoints: &[Vec2], is_closed: bool) -> Self {
        assert!(waypoints.len() >= 2, "Path requires at least 2 waypoints");
        let step_size = 1.0f32; // 1 meter sampling resolution
        let mut points = Vec::new();
        let mut cumulative_dist = 0.0f32;

        let n = if is_closed {
            waypoints.len()
        } else {
            waypoints.len() - 1
        };

        for i in 0..n {
            let p0 = waypoints[i];
            let p1 = waypoints[(i + 1) % waypoints.len()];
            let delta = p1 - p0;
            let seg_len = delta.length();
            if seg_len < 1e-4 {
                continue;
            }
            let tangent = delta / seg_len;
            let normal = Vec2::new(-tangent.y, tangent.x);

            let steps = (seg_len / step_size).ceil() as usize;
            let dt_step = seg_len / steps as f32;

            for s in 0..steps {
                let pos = p0 + tangent * (s as f32 * dt_step);
                points.push(SimPathPoint {
                    position: pos,
                    tangent,
                    normal,
                    curvature: 0.0,
                    distance: cumulative_dist + (s as f32 * dt_step),
                });
            }
            cumulative_dist += seg_len;
        }

        // Add final endpoint if open
        if !is_closed {
            let last_wp = *waypoints.last().unwrap();
            let prev_tangent = points.last().map(|p| p.tangent).unwrap_or(Vec2::X);
            let prev_normal = Vec2::new(-prev_tangent.y, prev_tangent.x);
            points.push(SimPathPoint {
                position: last_wp,
                tangent: prev_tangent,
                normal: prev_normal,
                curvature: 0.0,
                distance: cumulative_dist,
            });
        }

        // Compute local curvature from tangent variation: kappa = |d_theta| / ds
        let pt_count = points.len();
        if pt_count >= 3 {
            for i in 0..pt_count {
                let prev_idx = if i == 0 {
                    if is_closed { pt_count - 1 } else { 0 }
                } else {
                    i - 1
                };
                let next_idx = if i == pt_count - 1 {
                    if is_closed { 0 } else { pt_count - 1 }
                } else {
                    i + 1
                };

                let t_prev = points[prev_idx].tangent;
                let t_next = points[next_idx].tangent;
                let a_prev = t_prev.y.atan2(t_prev.x);
                let a_next = t_next.y.atan2(t_next.x);
                let d_theta = normalize_angle(a_next - a_prev).abs();

                let ds = if next_idx >= prev_idx {
                    points[next_idx].distance - points[prev_idx].distance
                } else {
                    (cumulative_dist - points[prev_idx].distance) + points[next_idx].distance
                };

                if ds > 1e-3 {
                    points[i].curvature = d_theta / ds;
                }
            }
        }

        Self {
            name: name.into(),
            points,
            total_length: cumulative_dist,
            is_closed,
        }
    }

    /// Dynamically constructs a straight test path with sequential turns and chicane.
    ///
    /// Layout:
    /// - 80m launch straight (0° heading)
    /// - 90° right sweeper (R = 35m)
    /// - 50m straight (-90° heading)
    /// - 90° left turn (R = 25m, chicane entry)
    /// - 60m straight (0° heading)
    /// - 90° right bend (R = 35m)
    /// - 70m exit straight
    /// Total length: ~410m.
    pub fn straight_with_turns() -> Self {
        let mut waypoints = Vec::new();
        // 1. Initial straight (80m east)
        waypoints.push(Vec2::new(0.0, 0.0));
        waypoints.push(Vec2::new(40.0, 0.0));
        waypoints.push(Vec2::new(80.0, 0.0));

        // 2. Right arc turn (R = 35m, center = (80.0, -35.0), angle from PI/2 down to 0)
        let r1 = 35.0f32;
        let c1 = Vec2::new(80.0, -r1);
        let arc1_steps = 14;
        for i in 1..=arc1_steps {
            let theta = std::f32::consts::FRAC_PI_2 * (1.0 - i as f32 / arc1_steps as f32);
            waypoints.push(c1 + Vec2::new(r1 * theta.cos(), r1 * theta.sin()));
        }

        // Now at (115.0, -35.0) pointing south
        // 3. Straight south (50m)
        waypoints.push(Vec2::new(115.0, -60.0));
        waypoints.push(Vec2::new(115.0, -85.0));

        // 4. Left arc turn (R = 25m, center = (140.0, -85.0), angle from PI up to 3*PI/2)
        let r2 = 25.0f32;
        let c2 = Vec2::new(140.0, -85.0);
        let arc2_steps = 12;
        for i in 1..=arc2_steps {
            let theta = std::f32::consts::PI + std::f32::consts::FRAC_PI_2 * (i as f32 / arc2_steps as f32);
            waypoints.push(c2 + Vec2::new(r2 * theta.cos(), r2 * theta.sin()));
        }

        // Now at (140.0, -110.0) pointing east
        // 5. Straight east (60m)
        waypoints.push(Vec2::new(170.0, -110.0));
        waypoints.push(Vec2::new(200.0, -110.0));

        // 6. Right bend 90 deg (R = 35m, center = (200.0, -145.0), angle from PI/2 down to 0)
        let r3 = 35.0f32;
        let c3 = Vec2::new(200.0, -145.0);
        let arc3_steps = 14;
        for i in 1..=arc3_steps {
            let theta = std::f32::consts::FRAC_PI_2 * (1.0 - i as f32 / arc3_steps as f32);
            waypoints.push(c3 + Vec2::new(r3 * theta.cos(), r3 * theta.sin()));
        }

        // Now at (235.0, -145.0) pointing south
        // 7. Final straight (70m south)
        waypoints.push(Vec2::new(235.0, -180.0));
        waypoints.push(Vec2::new(235.0, -215.0));

        Self::from_waypoints("Dynamic Straight With Turns", &waypoints, false)
    }

    /// Dynamically constructs a closed hypothetical circuit featuring straights, 90° sweepers, and chicane.
    ///
    /// Continuous closed racetrack loop of ~710m length.
    pub fn hypothetical_circuit() -> Self {
        let mut waypoints = Vec::new();

        // 1. Main Start/Finish Straight (140m east)
        waypoints.push(Vec2::new(0.0, 0.0));
        waypoints.push(Vec2::new(70.0, 0.0));
        waypoints.push(Vec2::new(140.0, 0.0));

        // 2. Turn 1: 90 deg right sweeper (R = 40m, center = (140, -40), PI/2 down to 0)
        let r = 40.0f32;
        let c1 = Vec2::new(140.0, -r);
        for i in 1..=12 {
            let theta = std::f32::consts::FRAC_PI_2 * (1.0 - i as f32 / 12.0);
            waypoints.push(c1 + Vec2::new(r * theta.cos(), r * theta.sin()));
        }

        // Now at (180, -40), heading south
        // 3. East Straight (70m south)
        waypoints.push(Vec2::new(180.0, -75.0));
        waypoints.push(Vec2::new(180.0, -110.0));

        // 4. Turn 2: 90 deg right sweeper (R = 40m, center = (140, -110), 0 down to -PI/2)
        let c2 = Vec2::new(140.0, -110.0);
        for i in 1..=12 {
            let theta = -(std::f32::consts::FRAC_PI_2 * (i as f32 / 12.0));
            waypoints.push(c2 + Vec2::new(r * theta.cos(), r * theta.sin()));
        }

        // Now at (140, -150), heading west
        // 5. Back Straight (160m west)
        waypoints.push(Vec2::new(60.0, -150.0));
        waypoints.push(Vec2::new(-20.0, -150.0));

        // 6. Turn 3: 90 deg right sweeper (R = 40m, center = (-20, -110), -PI/2 down to -PI)
        let c3 = Vec2::new(-20.0, -110.0);
        for i in 1..=12 {
            let theta = -std::f32::consts::FRAC_PI_2 - (std::f32::consts::FRAC_PI_2 * (i as f32 / 12.0));
            waypoints.push(c3 + Vec2::new(r * theta.cos(), r * theta.sin()));
        }

        // Now at (-60, -110), heading north
        // 7. West Straight (70m north)
        waypoints.push(Vec2::new(-60.0, -75.0));
        waypoints.push(Vec2::new(-60.0, -40.0));

        // 8. Turn 4: 90 deg right sweeper (R = 40m, center = (-20, -40), PI down to PI/2)
        let c4 = Vec2::new(-20.0, -40.0);
        for i in 1..=12 {
            let theta = std::f32::consts::PI - (std::f32::consts::FRAC_PI_2 * (i as f32 / 12.0));
            waypoints.push(c4 + Vec2::new(r * theta.cos(), r * theta.sin()));
        }

        // Now at (-20, 0), heading east
        // 9. Approach to finish line (20m east)
        waypoints.push(Vec2::new(-10.0, 0.0));

        Self::from_waypoints("Hypothetical Grand Prix Circuit", &waypoints, true)
    }

    /// Projects a 2D world position onto the path, returning the closest point index,
    /// signed cross-track error, and distance along path.
    pub fn project_position(&self, pos: Vec2, hint_idx: usize) -> (usize, f32, f32) {
        let n = self.points.len();
        if n == 0 {
            return (0, 0.0, 0.0);
        }

        let mut best_idx = hint_idx % n;
        let mut best_dist_sq = (self.points[best_idx].position - pos).length_squared();

        // Check forward 100 points (100 meters)
        let search_fwd = 100.min(n);
        for offset in 1..=search_fwd {
            let idx = (hint_idx + offset) % n;
            let d = (self.points[idx].position - pos).length_squared();
            if d < best_dist_sq {
                best_dist_sq = d;
                best_idx = idx;
            }
        }

        // Also check backward 15 points
        let search_bwd = 15.min(n);
        for offset in 1..=search_bwd {
            let idx = (hint_idx + n - offset) % n;
            let d = (self.points[idx].position - pos).length_squared();
            if d < best_dist_sq {
                best_dist_sq = d;
                best_idx = idx;
            }
        }

        // Global fallback if best match is > 10m away
        if best_dist_sq > 100.0 {
            for (i, p) in self.points.iter().enumerate() {
                let d = (p.position - pos).length_squared();
                if d < best_dist_sq {
                    best_dist_sq = d;
                    best_idx = i;
                }
            }
        }

        let pt = &self.points[best_idx];
        let to_car = pos - pt.position;
        let cross_track = to_car.dot(pt.normal);

        (best_idx, cross_track, pt.distance)
    }

    /// Samples a target point along the path at a distance offset from current distance.
    pub fn sample_target(&self, current_dist: f32, lookahead_m: f32) -> SimPathPoint {
        let target_dist = if self.is_closed {
            (current_dist + lookahead_m) % self.total_length
        } else {
            (current_dist + lookahead_m).min(self.total_length)
        };

        // Binary search since points are monotonically sorted by distance
        match self.points.binary_search_by(|p| p.distance.partial_cmp(&target_dist).unwrap_or(std::cmp::Ordering::Equal)) {
            Ok(idx) => self.points[idx],
            Err(idx) => {
                if idx == 0 {
                    self.points[0]
                } else if idx >= self.points.len() {
                    *self.points.last().unwrap()
                } else {
                    let p0 = &self.points[idx - 1];
                    let p1 = &self.points[idx];
                    let seg_len = p1.distance - p0.distance;
                    let frac = if seg_len > 1e-4 {
                        ((target_dist - p0.distance) / seg_len).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    SimPathPoint {
                        position: p0.position.lerp(p1.position, frac),
                        tangent: p0.tangent.lerp(p1.tangent, frac).normalize_or_zero(),
                        normal: p0.normal.lerp(p1.normal, frac).normalize_or_zero(),
                        curvature: p0.curvature * (1.0 - frac) + p1.curvature * frac,
                        distance: target_dist,
                    }
                }
            }
        }
    }

    /// Evaluates the maximum upcoming curvature over the next `window_m` meters.
    pub fn max_upcoming_curvature(&self, current_dist: f32, window_m: f32) -> f32 {
        let num_samples = 10;
        let mut max_curv = 0.0f32;
        for i in 1..=num_samples {
            let offset = window_m * (i as f32 / num_samples as f32);
            let pt = self.sample_target(current_dist, offset);
            if pt.curvature > max_curv {
                max_curv = pt.curvature;
            }
        }
        max_curv
    }
}

/// Simulation outcome status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathSimulationStatus {
    /// Vehicle reached end of path or completed circuit lap.
    Completed,
    /// Vehicle unable to move due to excessive rolling resistance / traction deficit.
    StuckInSand,
    /// Vehicle understeered / broke away off the track corridor (> 22m from centerline).
    OffTrackDeparture,
    /// Vehicle spun out (excessive sideslip / yaw instability).
    SpunOut,
    /// Simulation exceeded timeout.
    TimedOut,
}

impl PathSimulationStatus {
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Completed)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "Completed",
            Self::StuckInSand => "Stuck In Sand",
            Self::OffTrackDeparture => "Off-Track Departure",
            Self::SpunOut => "Spun Out",
            Self::TimedOut => "Timed Out",
        }
    }
}

/// Complete empirical telemetry and metrics result from a circuit/path simulation run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathSimulationResult {
    pub surface: SurfaceType,
    pub path_name: String,
    pub path_length_m: f32,
    pub distance_traveled_m: f32,
    pub completion_pct: f32,
    pub elapsed_time_s: f32,
    pub avg_speed_kmh: f32,
    pub peak_speed_kmh: f32,
    pub max_cross_track_error_m: f32,
    pub rms_cross_track_error_m: f32,
    pub peak_lateral_accel_g: f32,
    pub avg_lateral_accel_g: f32,
    pub peak_slip_angle_deg: f32,
    pub avg_front_slip_angle_deg: f32,
    pub avg_rear_slip_angle_deg: f32,
    pub peak_rolling_resistance_n: f32,
    pub avg_rolling_resistance_n: f32,
    pub peak_traction_force_n: f32,
    pub avg_traction_force_n: f32,
    pub status: PathSimulationStatus,
    pub failure_reason: Option<String>,
}

/// Runs a closed-loop automotive simulation along `path` across `surface`.
pub fn run_path_simulation(
    config: &CarConfig,
    surface: SurfaceType,
    path: &SimPath,
    timeout_s: f32,
    dt: f32,
) -> PathSimulationResult {
    // Initial vehicle pose aligned with start of path
    let start_pt = path.points[0];
    let start_angle = start_pt.tangent.y.atan2(start_pt.tangent.x);
    let mut runner = SimulationRunner::new(config.clone(), dt)
        .with_state(start_pt.position, start_angle, Vec2::ZERO);

    let mut ctrl_idx = 0usize;
    let mut eval_idx = 0usize;
    let mut prev_heading_error = 0.0f32;
    let mut has_prev_heading = false;

    let mut peak_speed = 0.0f32;
    let mut speed_sum = 0.0f32;
    let mut sample_count = 0usize;

    let mut max_cross_track = 0.0f32;
    let mut sum_cross_track_sq = 0.0f32;

    let mut peak_lat_accel = 0.0f32;
    let mut lat_accel_sum = 0.0f32;

    let mut peak_slip_deg = 0.0f32;
    let mut front_slip_sum = 0.0f32;
    let mut rear_slip_sum = 0.0f32;

    let mut peak_rr = 0.0f32;
    let mut rr_sum = 0.0f32;
    let mut peak_traction = 0.0f32;
    let mut traction_sum = 0.0f32;

    let mut stuck_timer = 0.0f32;
    let mut spin_timer = 0.0f32;
    let mut departure_timer = 0.0f32;

    let mut status = PathSimulationStatus::TimedOut;
    let mut failure_reason = None;
    let mut max_dist_achieved = 0.0f32;

    runner.run_until(
        timeout_s,
        surface,
        |_t, car| {
            let pos = car.state().position;
            let speed = car.state().speed;
            let car_angle = car.state().angle;

            let (idx, _cross_err, cur_dist) = path.project_position(pos, ctrl_idx);
            ctrl_idx = idx;

            // 1. Dynamic lookahead based on speed: Ld = 7m to 25m
            let lookahead_m = (7.0 + speed * 0.40).clamp(7.0, 25.0);
            let target_sample = path.sample_target(cur_dist, lookahead_m);

            // 2. Heading error to target point
            let to_target = target_sample.position - pos;
            let target_heading = to_target.y.atan2(to_target.x);
            let heading_error = normalize_angle(target_heading - car_angle);

            // Derivative of heading error
            let d_heading = if has_prev_heading && dt > 1e-4 {
                normalize_angle(heading_error - prev_heading_error) / dt
            } else {
                has_prev_heading = true;
                0.0
            };
            prev_heading_error = heading_error;

            // 3. Cross-track error to closest point
            let closest_pt = &path.points[idx];
            let cross_err = (pos - closest_pt.position).dot(closest_pt.normal);

            // 4. Steering command: PD heading regulation + Stanley cross-track correction
            let kp_steer = 2.40;
            let kd_steer = 0.08;
            let k_lat = 0.22;
            let steer_demand = -(heading_error * kp_steer + d_heading * kd_steer + (cross_err * k_lat) / (1.0 + 0.06 * speed));
            let steer_cmd = steer_demand.clamp(-1.0, 1.0);

            // 5. Analytical Racing Braking Envelope: v_allowable = sqrt(v_apex^2 + 2 * a_brake * d)
            let mu = surface.friction_coefficient();
            let mut target_speed = config.top_speed_mps;
            let a_brake = (mu * G_ACCEL * 0.70).clamp(1.5, 7.5);
            let max_lookahead = (25.0 + (speed * speed) / 6.0).clamp(30.0, 120.0);
            let num_scans = 15;

            for s in 1..=num_scans {
                let dist_ahead = max_lookahead * (s as f32 / num_scans as f32);
                let pt = path.sample_target(cur_dist, dist_ahead);
                if pt.curvature > 0.003 {
                    let radius = 1.0 / pt.curvature;
                    let v_apex = (mu * G_ACCEL * radius).sqrt() * 0.78;
                    let v_allowable = (v_apex * v_apex + 2.0 * a_brake * dist_ahead).sqrt();
                    if v_allowable < target_speed {
                        target_speed = v_allowable;
                    }
                }
            }

            // 6. Longitudinal throttle / brake commands
            let speed_err = target_speed - speed;
            let (throttle, brake) = if speed_err > 0.3 {
                let th = (speed_err / 4.0).clamp(0.20, 1.0);
                (th, 0.0)
            } else if speed_err < -0.8 {
                let brk = ((-speed_err) / 5.0).clamp(0.35, 1.0);
                (0.0, brk)
            } else {
                (0.05, 0.0)
            };

            CarControls {
                throttle,
                steer: steer_cmd,
                brake,
                handbrake: false,
                reverse: false,
            }
        },
        |t, car| {
            let pos = car.state().position;
            let speed = car.state().speed;
            let lat_accel_g = car.state().acceleration_local.y.abs() / G_ACCEL;
            let sideslip_deg = car.state().sideslip_angle.abs().to_degrees();

            let (idx, cross_err, cur_dist) = path.project_position(pos, eval_idx);
            eval_idx = idx;

            if cur_dist > max_dist_achieved {
                max_dist_achieved = cur_dist;
            }

            // Accumulate metrics
            sample_count += 1;
            speed_sum += speed;
            if speed > peak_speed {
                peak_speed = speed;
            }

            let abs_cross = cross_err.abs();
            if abs_cross > max_cross_track {
                max_cross_track = abs_cross;
            }
            sum_cross_track_sq += cross_err * cross_err;

            lat_accel_sum += lat_accel_g;
            if lat_accel_g > peak_lat_accel {
                peak_lat_accel = lat_accel_g;
            }

            // Wheel slip and force analysis
            let w_front_slip = (car.state().wheels[0].slip_angle.abs() + car.state().wheels[1].slip_angle.abs()) * 0.5;
            let w_rear_slip = (car.state().wheels[2].slip_angle.abs() + car.state().wheels[3].slip_angle.abs()) * 0.5;
            let max_wheel_slip = w_front_slip.max(w_rear_slip).to_degrees();
            if max_wheel_slip > peak_slip_deg {
                peak_slip_deg = max_wheel_slip;
            }
            front_slip_sum += w_front_slip.to_degrees();
            rear_slip_sum += w_rear_slip.to_degrees();

            // Total rolling resistance vs tire forward thrust
            let normal_load_total: f32 = car.state().wheels.iter().map(|w| w.normal_load).sum();
            let rr_coeff = config.rolling_resistance_coefficient * surface.rolling_resistance_multiplier();
            let current_rr_force = rr_coeff * normal_load_total;
            if current_rr_force > peak_rr {
                peak_rr = current_rr_force;
            }
            rr_sum += current_rr_force;

            let current_traction: f32 = car.state().wheels.iter().map(|w| w.longitudinal_force.max(0.0)).sum();
            if current_traction > peak_traction {
                peak_traction = current_traction;
            }
            traction_sum += current_traction;

            // Termination check: Completion
            if path.is_closed {
                if max_dist_achieved >= path.total_length * 0.95 && cur_dist < 15.0 && t > 5.0 {
                    status = PathSimulationStatus::Completed;
                    return true;
                }
            } else {
                if cur_dist >= path.total_length - 8.0 {
                    status = PathSimulationStatus::Completed;
                    return true;
                }
            }

            // Termination check: Stuck in sand (speed < 0.6 m/s despite throttle > 0.5)
            if speed < 0.6 && t > 2.0 {
                stuck_timer += dt;
                if stuck_timer > 3.0 {
                    status = PathSimulationStatus::StuckInSand;
                    failure_reason = Some(format!(
                        "Vehicle bogged down at {:.1} m/s (RR force {:.0} N exceeded forward tire thrust {:.0} N)",
                        speed, current_rr_force, current_traction
                    ));
                    return true;
                }
            } else {
                stuck_timer = (stuck_timer - dt * 2.0).max(0.0);
            }

            // Termination check: Off-track departure (understeered off corridor)
            if abs_cross > 22.0 {
                departure_timer += dt;
                if departure_timer > 1.5 {
                    status = PathSimulationStatus::OffTrackDeparture;
                    failure_reason = Some(format!(
                        "Terminal understeer: departed corridor by {:.1}m at progress {:.1}m",
                        abs_cross, cur_dist
                    ));
                    return true;
                }
            } else {
                departure_timer = 0.0;
            }

            // Termination check: Spun out
            if sideslip_deg > 55.0 {
                spin_timer += dt;
                if spin_timer > 1.0 {
                    status = PathSimulationStatus::SpunOut;
                    failure_reason = Some(format!(
                        "Severe yaw instability: sideslip angle reached {:.1}°",
                        sideslip_deg
                    ));
                    return true;
                }
            } else {
                spin_timer = 0.0;
            }

            false
        },
    );

    let elapsed = runner.time;
    let avg_speed_kmh = if sample_count > 0 {
        (speed_sum / sample_count as f32) * 3.6
    } else {
        0.0
    };
    let rms_cross = if sample_count > 0 {
        (sum_cross_track_sq / sample_count as f32).sqrt()
    } else {
        0.0
    };

    let avg_lat_accel = if sample_count > 0 {
        lat_accel_sum / sample_count as f32
    } else {
        0.0
    };
    let avg_front_slip = if sample_count > 0 {
        front_slip_sum / sample_count as f32
    } else {
        0.0
    };
    let avg_rear_slip = if sample_count > 0 {
        rear_slip_sum / sample_count as f32
    } else {
        0.0
    };
    let avg_rr = if sample_count > 0 {
        rr_sum / sample_count as f32
    } else {
        0.0
    };
    let avg_trac = if sample_count > 0 {
        traction_sum / sample_count as f32
    } else {
        0.0
    };

    let completion_pct = (max_dist_achieved / path.total_length * 100.0).clamp(0.0, 100.0);

    PathSimulationResult {
        surface,
        path_name: path.name.clone(),
        path_length_m: path.total_length,
        distance_traveled_m: max_dist_achieved,
        completion_pct,
        elapsed_time_s: elapsed,
        avg_speed_kmh,
        peak_speed_kmh: peak_speed * 3.6,
        max_cross_track_error_m: max_cross_track,
        rms_cross_track_error_m: rms_cross,
        peak_lateral_accel_g: peak_lat_accel,
        avg_lateral_accel_g: avg_lat_accel,
        peak_slip_angle_deg: peak_slip_deg,
        avg_front_slip_angle_deg: avg_front_slip,
        avg_rear_slip_angle_deg: avg_rear_slip,
        peak_rolling_resistance_n: peak_rr,
        avg_rolling_resistance_n: avg_rr,
        peak_traction_force_n: peak_traction,
        avg_traction_force_n: avg_trac,
        status,
        failure_reason,
    }
}
