pub mod driver;

pub use driver::{DriverCharacter, DriverFavoriteCar, DriverPersonalityOffsets, DriverStats};

use glam::Vec2;
use serde::{Deserialize, Serialize};
use tdrace_core::physics::car::{normalize_angle, Car, CarControls};
use tdrace_core::track::Track;

/// Tactical philosophy and driving personality (6 styles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrivingStyle {
    Smooth,
    Aggressive,
    Tenacious,
    Calculating,
    Bold,
    Balanced,
}

impl DrivingStyle {
    pub const ALL: [Self; 6] = [
        Self::Smooth,
        Self::Aggressive,
        Self::Tenacious,
        Self::Calculating,
        Self::Bold,
        Self::Balanced,
    ];

    pub fn from_str_lossy(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "smooth" => Self::Smooth,
            "aggressive" | "brawler" => Self::Aggressive,
            "tenacious" | "defender" => Self::Tenacious,
            "calculating" | "tactical" | "strategic" | "draft" => Self::Calculating,
            "bold" | "drift" | "renegade" => Self::Bold,
            "balanced" | "club" => Self::Balanced,
            _ => Self::Balanced,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Smooth => "smooth",
            Self::Aggressive => "aggressive",
            Self::Tenacious => "tenacious",
            Self::Calculating => "calculating",
            Self::Bold => "bold",
            Self::Balanced => "balanced",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Smooth => "Smooth",
            Self::Aggressive => "Aggressive",
            Self::Tenacious => "Tenacious",
            Self::Calculating => "Calculating",
            Self::Bold => "Bold",
            Self::Balanced => "Balanced",
        }
    }
}

/// 5-tier experience and performance ladder matching vehicle tiers 1..=5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum DriverTier {
    Rookie = 1,
    Amateur = 2,
    Contender = 3,
    Pro = 4,
    Legend = 5,
}

impl DriverTier {
    pub const fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::Rookie,
            2 => Self::Amateur,
            3 => Self::Contender,
            4 => Self::Pro,
            _ => Self::Legend,
        }
    }

    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "1" | "rookie" | "novice" => Self::Rookie,
            "2" | "amateur" | "club" | "clubman" => Self::Amateur,
            "3" | "contender" | "semipro" | "semi_pro" | "national" => Self::Contender,
            "4" | "pro" | "veteran" => Self::Pro,
            "5" | "legend" | "elite" | "alien" | "champion" => Self::Legend,
            _ => Self::Pro,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Rookie => "Tier 1: Rookie",
            Self::Amateur => "Tier 2: Amateur",
            Self::Contender => "Tier 3: Contender",
            Self::Pro => "Tier 4: Pro",
            Self::Legend => "Tier 5: Legend",
        }
    }

    pub const fn short_name(self) -> &'static str {
        match self {
            Self::Rookie => "Rookie (T1)",
            Self::Amateur => "Amateur (T2)",
            Self::Contender => "Contender (T3)",
            Self::Pro => "Pro (T4)",
            Self::Legend => "Legend (T5)",
        }
    }

    /// Returns the discrete probability distribution [P(T1), P(T2), P(T3), P(T4), P(T5)]
    /// (summing to 100%) for opponent tiers centered on this target difficulty tier.
    pub const fn bell_curve_weights(self) -> [u8; 5] {
        match self {
            Self::Rookie => [55, 35, 10, 0, 0],
            Self::Amateur => [20, 50, 25, 5, 0],
            Self::Contender => [5, 20, 50, 20, 5],
            Self::Pro => [0, 5, 25, 50, 20],
            Self::Legend => [0, 0, 10, 35, 55],
        }
    }

    /// Samples a tier from the discrete bell curve distribution using a pseudo-random value in 0..100.
    pub fn sample_from_bell_curve(self, roll_0_to_99: u8) -> Self {
        let weights = self.bell_curve_weights();
        let mut accum = 0;
        let roll = roll_0_to_99 % 100;
        for (idx, &w) in weights.iter().enumerate() {
            accum += w;
            if roll < accum {
                return Self::from_u8((idx + 1) as u8);
            }
        }
        self
    }

    /// Samples `n` tiers for a grid of opponents given a target tier and deterministic seed.
    pub fn sample_grid_tiers(self, n: usize, seed: u64) -> Vec<Self> {
        let mut tiers = Vec::with_capacity(n);
        let mut s = seed.wrapping_add(1442695040888963407);
        for _ in 0..n {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            let roll = ((s >> 33) % 100) as u8;
            tiers.push(self.sample_from_bell_curve(roll));
        }
        tiers
    }
}

/// Operationalized performance attributes of a driver quality level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriverQuality {
    pub tier: DriverTier,
    pub pace_limit: f32,
    pub brake_padding: f32,
    pub avoidance_padding: f32,
    pub consistency: f32,
    pub composure: f32,
}

impl DriverQuality {
    pub const fn for_tier(tier: DriverTier) -> Self {
        match tier {
            DriverTier::Rookie => Self {
                tier,
                pace_limit: 0.88,
                brake_padding: 0.22,
                avoidance_padding: 2.5,
                consistency: 0.60,
                composure: 0.50,
            },
            DriverTier::Amateur => Self {
                tier,
                pace_limit: 0.92,
                brake_padding: 0.14,
                avoidance_padding: 1.5,
                consistency: 0.72,
                composure: 0.65,
            },
            DriverTier::Contender => Self {
                tier,
                pace_limit: 0.96,
                brake_padding: 0.07,
                avoidance_padding: 0.8,
                consistency: 0.83,
                composure: 0.78,
            },
            DriverTier::Pro => Self {
                tier,
                pace_limit: 1.00,
                brake_padding: 0.00,
                avoidance_padding: 0.0,
                consistency: 0.93,
                composure: 0.90,
            },
            DriverTier::Legend => Self {
                tier,
                pace_limit: 1.04,
                brake_padding: -0.03,
                avoidance_padding: -0.5,
                consistency: 0.99,
                composure: 0.98,
            },
        }
    }
}


/// Personality and tuning settings for an AI bot driver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BotProfile {
    pub name: &'static str,
    /// Base lookahead horizon in seconds.
    pub lookahead_time: f32,
    /// Overall pace and corner entry speed aggressiveness (0.85 = cautious, 1.05 = expert).
    pub speed_factor: f32,
    /// Proportional gain for steering towards target waypoint.
    pub steering_kp: f32,
    /// Derivative gain for damping steering oscillations and oversteer.
    pub steering_kd: f32,
    /// Minimum braking distance multiplier.
    pub brake_margin: f32,
    /// Overtaking lateral aggression [0.0 = passive, 1.0 = divebomb].
    pub aggression: f32,
    /// Proximity avoidance safety radius in meters.
    pub avoidance_distance: f32,
}

impl Default for BotProfile {
    fn default() -> Self {
        Self::pro()
    }
}

impl BotProfile {
    pub const fn pro() -> Self {
        Self {
            name: "Pro Bot",
            lookahead_time: 0.38,
            speed_factor: 1.00,
            steering_kp: 2.2,
            steering_kd: 0.06,
            brake_margin: 1.05,
            aggression: 0.80,
            avoidance_distance: 7.0,
        }
    }

    pub const fn rookie() -> Self {
        Self {
            name: "Rookie Bot",
            lookahead_time: 0.48,
            speed_factor: 0.88,
            steering_kp: 1.7,
            steering_kd: 0.08,
            brake_margin: 1.30,
            aggression: 0.35,
            avoidance_distance: 9.5,
        }
    }

    pub const fn smooth() -> Self {
        Self {
            name: "Smooth Archetype",
            lookahead_time: 0.40,
            speed_factor: 1.01,
            steering_kp: 2.30,
            steering_kd: 0.08,
            brake_margin: 1.02,
            aggression: 0.70,
            avoidance_distance: 6.5,
        }
    }

    pub const fn aggressive() -> Self {
        Self {
            name: "Aggressive Archetype",
            lookahead_time: 0.32,
            speed_factor: 1.02,
            steering_kp: 2.50,
            steering_kd: 0.05,
            brake_margin: 0.90,
            aggression: 0.95,
            avoidance_distance: 5.0,
        }
    }

    pub const fn tenacious() -> Self {
        Self {
            name: "Tenacious Archetype",
            lookahead_time: 0.42,
            speed_factor: 0.99,
            steering_kp: 2.20,
            steering_kd: 0.08,
            brake_margin: 1.05,
            aggression: 0.82,
            avoidance_distance: 6.0,
        }
    }

    pub const fn calculating() -> Self {
        Self {
            name: "Calculating Archetype",
            lookahead_time: 0.39,
            speed_factor: 1.00,
            steering_kp: 2.40,
            steering_kd: 0.07,
            brake_margin: 1.00,
            aggression: 0.75,
            avoidance_distance: 6.5,
        }
    }

    pub const fn bold() -> Self {
        Self {
            name: "Bold Archetype",
            lookahead_time: 0.31,
            speed_factor: 1.01,
            steering_kp: 2.70,
            steering_kd: 0.04,
            brake_margin: 0.88,
            aggression: 0.92,
            avoidance_distance: 5.2,
        }
    }

    pub const fn balanced() -> Self {
        Self {
            name: "Balanced Archetype",
            lookahead_time: 0.38,
            speed_factor: 0.98,
            steering_kp: 2.10,
            steering_kd: 0.07,
            brake_margin: 1.05,
            aggression: 0.65,
            avoidance_distance: 7.0,
        }
    }

    pub fn from_style(style: DrivingStyle) -> Self {
        match style {
            DrivingStyle::Smooth => Self::smooth(),
            DrivingStyle::Aggressive => Self::aggressive(),
            DrivingStyle::Tenacious => Self::tenacious(),
            DrivingStyle::Calculating => Self::calculating(),
            DrivingStyle::Bold => Self::bold(),
            DrivingStyle::Balanced => Self::balanced(),
        }
    }

    pub fn from_style_and_quality(style: DrivingStyle, quality: &DriverQuality) -> Self {
        let (base_lookahead, base_kp, base_kd, style_brake, style_aggression, style_avoidance, style_speed_mult) = match style {
            DrivingStyle::Smooth => (0.40, 2.30, 0.08, 1.02, 0.70, 6.5, 1.01),
            DrivingStyle::Aggressive => (0.32, 2.50, 0.05, 0.90, 0.95, 5.0, 1.02),
            DrivingStyle::Tenacious => (0.42, 2.20, 0.08, 1.05, 0.82, 6.0, 0.99),
            DrivingStyle::Calculating => (0.39, 2.40, 0.07, 1.00, 0.75, 6.5, 1.00),
            DrivingStyle::Bold => (0.31, 2.70, 0.04, 0.88, 0.92, 5.2, 1.01),
            DrivingStyle::Balanced => (0.38, 2.10, 0.07, 1.05, 0.65, 7.0, 0.98),
        };

        Self {
            name: "Composite Bot",
            lookahead_time: (base_lookahead * (0.80 + 0.20 * quality.consistency)).clamp(0.20, 0.55),
            speed_factor: (quality.pace_limit * style_speed_mult).clamp(0.80, 1.15),
            steering_kp: base_kp,
            steering_kd: base_kd * (0.75 + 0.25 * quality.consistency),
            brake_margin: (style_brake + quality.brake_padding).clamp(0.80, 1.45),
            aggression: style_aggression,
            avoidance_distance: (style_avoidance + quality.avoidance_padding).clamp(3.5, 12.0),
        }
    }

    pub fn from_archetype(name: &str) -> Self {
        let norm = name.to_ascii_lowercase();
        if let Some((s_str, t_str)) = norm.split_once('_') {
            let style = DrivingStyle::from_str_lossy(s_str);
            let tier = DriverTier::from_str_lossy(t_str);
            return Self::from_style_and_quality(style, &DriverQuality::for_tier(tier));
        }
        match norm.as_str() {
            "rookie" | "cautious" => Self::from_style_and_quality(DrivingStyle::Balanced, &DriverQuality::for_tier(DriverTier::Rookie)),
            "fast" | "pro" | "hotlap" => Self::from_style_and_quality(DrivingStyle::Smooth, &DriverQuality::for_tier(DriverTier::Legend)),
            "strategic" | "draft" => Self::from_style_and_quality(DrivingStyle::Calculating, &DriverQuality::for_tier(DriverTier::Pro)),
            "brawler" => Self::from_style_and_quality(DrivingStyle::Aggressive, &DriverQuality::for_tier(DriverTier::Pro)),
            "defender" => Self::from_style_and_quality(DrivingStyle::Tenacious, &DriverQuality::for_tier(DriverTier::Pro)),
            "drift" => Self::from_style_and_quality(DrivingStyle::Bold, &DriverQuality::for_tier(DriverTier::Pro)),
            "club" => Self::from_style_and_quality(DrivingStyle::Balanced, &DriverQuality::for_tier(DriverTier::Contender)),
            _ => Self::from_style(DrivingStyle::from_str_lossy(&norm)),
        }
    }

    pub fn archetype_for_index(idx: usize) -> Self {
        Self::from_style(DrivingStyle::ALL[idx % DrivingStyle::ALL.len()])
    }
}

/// Multi-car Bot Racing AI Controller.
#[derive(Debug, Clone)]
pub struct BotAiDriver {
    pub profile: BotProfile,
    pub prev_heading_error: f32,
    pub has_prev_heading: bool,
    pub current_target_dist: f32,
    pub avoidance_lateral_bias: f32,
    pub stuck_timer: f32,
    pub reverse_recovery_timer: f32,
    pub last_pos: Option<Vec2>,
}

impl BotAiDriver {
    pub fn new(profile: BotProfile) -> Self {
        Self {
            profile,
            prev_heading_error: 0.0,
            has_prev_heading: false,
            current_target_dist: 0.0,
            avoidance_lateral_bias: 0.0,
            stuck_timer: 0.0,
            reverse_recovery_timer: 0.0,
            last_pos: None,
        }
    }

    /// Computes deterministic driving controls (throttle, steer, brake, handbrake)
    /// for the given bot car navigating the track amidst other cars.
    pub fn compute_controls(
        &mut self,
        car: &Car,
        track: &Track,
        other_cars: &[&Car],
        dt: f32,
    ) -> CarControls {
        let spline = &track.spline;
        if spline.samples.is_empty() {
            return CarControls::default();
        }

        let car_pos = car.state.position;
        let car_speed = car.state.speed;
        let car_fwd = car.forward_vector();
        let car_right = car.right_vector();

        // 1. Project onto spline to find current track distance with continuity constraint
        let proj = if self.last_pos.is_some() {
            spline.project_point_continuity(car_pos, self.current_target_dist, 50.0)
        } else {
            spline.project_point(car_pos)
        };
        let curr_dist = proj.progress_distance;

        // 2. Dynamic lookahead based on speed and profile
        let lookahead_dist = (10.0 + car_speed * (self.profile.lookahead_time + 0.10)).clamp(9.0, 45.0);
        let target_dist = (curr_dist + lookahead_dist) % spline.total_length();
        self.current_target_dist = target_dist;

        let target_sample = spline.sample_at_distance(target_dist);
        let mut target_point = target_sample.point;

        // Obstacle clearance: shift target point away from track obstacles (apex tire stacks)
        for obs in &track.geometry.obstacles {
            let obs_center = obs.center();
            let to_obs = obs_center - car_pos;
            let dist = to_obs.length();
            if dist < 10.0 {
                let obs_lat = to_obs.dot(car_right);
                if obs_lat.abs() < 3.5 {
                    let push_dir: f32 = if obs_lat > 0.0 { 1.0 } else { -1.0 };
                    let urgency: f32 = (1.0f32 - (dist / 10.0)).clamp(0.0f32, 1.0f32);
                    target_point += target_sample.normal * (push_dir * urgency * 3.5);
                }
            }
        }

        // 3. Check heading error to target
        let to_target = target_point - car_pos;
        let desired_heading = to_target.y.atan2(to_target.x);
        let heading_error = normalize_angle(desired_heading - car.state.angle);

        // Stuck / Wall-pin detection & Reverse recovery state machine
        if self.reverse_recovery_timer > 0.0 {
            self.reverse_recovery_timer -= dt;
            let steer_rev = -heading_error.signum();
            return CarControls {
                throttle: 0.85,
                steer: steer_rev,
                brake: 0.0,
                handbrake: false,
                reverse: true,
            };
        }

        let moved_dist = if let Some(last_p) = self.last_pos {
            (car_pos - last_p).length()
        } else {
            1.0
        };
        self.last_pos = Some(car_pos);

        let car_alignment = car_fwd.dot(proj.tangent);
        let is_stuck_situation = (!proj.is_on_track && car_speed < 1.2) || (car_alignment < -0.35 && car_speed < 1.5);
        if moved_dist < (1.2 * dt) && is_stuck_situation {
            self.stuck_timer += dt;
            if self.stuck_timer > 0.8 {
                self.stuck_timer = 0.0;
                self.reverse_recovery_timer = 1.0;
            }
        } else {
            self.stuck_timer = (self.stuck_timer - dt * 2.0).max(0.0);
        }

        // 3. Physically Exact Autonomous Racing Braking Envelope: v_allowable = sqrt(v_apex^2 + 2*a_brake*d)
        let max_braking_lookahead = (lookahead_dist + (car_speed * car_speed) / 7.5).clamp(35.0, 220.0);
        let a_brake = 6.0 * self.profile.brake_margin; // safe braking deceleration m/s²
        let mu = 0.78;
        let g = 9.81;

        let mut target_speed = car.config.top_speed_mps;
        let num_scan_samples = 20;

        for s_idx in 1..=num_scan_samples {
            let dist_ahead = max_braking_lookahead * (s_idx as f32 / num_scan_samples as f32);
            let scan_dist = (curr_dist + dist_ahead) % spline.total_length();

            // Measure local track curvature over a 10-meter span at the scan point
            let span = 10.0f32;
            let s0 = spline.sample_at_distance(scan_dist);
            let s1 = spline.sample_at_distance((scan_dist + span) % spline.total_length());

            let a0 = s0.tangent.y.atan2(s0.tangent.x);
            let a1 = s1.tangent.y.atan2(s1.tangent.x);
            let d_theta = normalize_angle(a1 - a0).abs();
            let local_curvature = d_theta / span;

            if local_curvature > 1e-4 {
                let local_radius = 1.0 / local_curvature;
                let bank_rad = s0.bank_angle.to_radians().abs();
                let effective_grip = mu + bank_rad.tan().clamp(0.0, 0.75);
                let v_apex = (effective_grip * g * local_radius).sqrt() * self.profile.speed_factor;
                // Maximum entry speed from distance d: v = sqrt(v_apex^2 + 2 * a * d)
                let v_allowable = (v_apex * v_apex + 2.0 * a_brake * dist_ahead).sqrt();
                if v_allowable < target_speed {
                    target_speed = v_allowable;
                }
            }
        }

        target_speed = target_speed.clamp(7.0, car.config.top_speed_mps);

        // 4. Multi-car Collision Avoidance & Overtaking
        let mut throttle_limit = 1.0f32;
        let mut extra_brake = 0.0f32;
        let mut avoidance_steer = 0.0f32;

        // Gradually decay previous lateral bias
        self.avoidance_lateral_bias *= 0.95;

        // 4b. Opponent cars avoidance
        for &opp in other_cars {
            let to_opp = opp.state.position - car_pos;
            let dist = to_opp.length();

            if dist < self.profile.avoidance_distance && dist > 0.05 {
                let opp_fwd_proj = to_opp.dot(car_fwd);
                let opp_lat_proj = to_opp.dot(car_right);

                // Opponent is in front of us
                if opp_fwd_proj > 0.5 {
                    let rel_speed = car_speed - opp.state.speed;

                    // Slipstream Drafting Behavior (Pack Racing):
                    // At high speeds, cars tucked in the slipstream cone follow the wake
                    // to gain aerodynamic tow instead of steering out prematurely.
                    let in_slipstream_zone = opp_fwd_proj > 3.5 && opp_fwd_proj < 30.0 && car_speed > 30.0;
                    if in_slipstream_zone && opp_lat_proj.abs() < 3.2 && self.profile.aggression > 0.5 {
                        // Align toward leader's wake (tuck in)
                        let align_wake = -opp_lat_proj * 0.12;
                        avoidance_steer += align_wake;
                    }

                    // If we are rapidly closing in on car ahead
                    if rel_speed > 1.2 && opp_fwd_proj < 10.0 {
                        // In high-speed pack drafting on straights, allow close tucking / bump-drafting
                        let is_bump_drafting = in_slipstream_zone && rel_speed < 3.0 && opp_fwd_proj > 2.5;
                        if !is_bump_drafting {
                            // Slow down to match speed or avoid rear-end crash
                            let urgency = (1.0 - (opp_fwd_proj / 10.0)).clamp(0.0, 1.0);
                            throttle_limit = (1.0 - urgency * 0.7).min(throttle_limit);
                            if rel_speed > 3.0 && opp_fwd_proj < 5.0 {
                                extra_brake = extra_brake.max(urgency * 0.8);
                            }
                        }
                    }

                    // Attempt lateral slingshot overtaking maneuver around car in front
                    if opp_fwd_proj < 15.0 && opp_lat_proj.abs() < 2.8 {
                        // Pick the side with more track clearance
                        let evade_dir = if opp_lat_proj >= 0.0 { 1.0 } else { -1.0 };
                        let evade_strength = (1.0 - (opp_fwd_proj / 15.0)) * self.profile.aggression;
                        avoidance_steer += evade_dir * evade_strength * 0.50;
                    }
                }
                // Opponent is side-by-side (prevent rubbing/interlocking wheels)
                else if opp_fwd_proj.abs() <= 2.5 && opp_lat_proj.abs() < 2.0 {
                    let push_away = if opp_lat_proj > 0.0 { 1.0 } else { -1.0 };
                    let side_urgency = (1.0 - (opp_lat_proj.abs() / 2.0)).clamp(0.0, 1.0);
                    avoidance_steer += push_away * side_urgency * 0.40;
                }
            }
        }

        // Apply avoidance lateral offset to target point
        if avoidance_steer.abs() > 0.05 {
            target_point += target_sample.normal * (avoidance_steer * 2.5);
        }

        // 5. Steering PID Controller with Error Derivative
        let to_target = target_point - car_pos;
        let desired_heading = to_target.y.atan2(to_target.x);
        let heading_error = normalize_angle(desired_heading - car.state.angle);

        let d_error = if self.has_prev_heading && dt > 1e-4 {
            normalize_angle(heading_error - self.prev_heading_error) / dt
        } else {
            self.has_prev_heading = true;
            0.0
        };
        self.prev_heading_error = heading_error;

        let steer = -(heading_error * self.profile.steering_kp + d_error * self.profile.steering_kd);
        let steer_cmd = steer.clamp(-1.0, 1.0);

        // Heading alignment and corner steering throttle limit (prevents spinning when loaded laterally)
        if car_speed > 4.0 {
            if heading_error.abs() > 0.45 {
                let align_factor = (1.0f32 - (heading_error.abs() - 0.45) / 0.85).clamp(0.2, 1.0);
                throttle_limit *= align_factor;
            }
            let steer_traction_limit = (1.0f32 - steer_cmd.abs() * 0.55).clamp(0.3, 1.0);
            throttle_limit *= steer_traction_limit;
        }
        if heading_error.abs() > 1.15 && car_speed > 6.0 {
            extra_brake = extra_brake.max(0.6);
        }

        // 6. Longitudinal Throttle & Brake Management
        let speed_err = target_speed - car_speed;
        let (throttle_cmd, brake_cmd) = if extra_brake > 0.2 {
            (0.0, extra_brake)
        } else if speed_err > 0.5 {
            // Accelerate
            let th = ((speed_err / 6.0) * throttle_limit).clamp(0.1, 1.0);
            (th, 0.0)
        } else if speed_err < -0.8 {
            // Decelerate / Brake for upcoming corner
            let brk = ((-speed_err / 6.0) * self.profile.brake_margin).clamp(0.35, 1.0);
            (0.0, brk)
        } else {
            // Coasting in balance zone
            (0.0, 0.0)
        };

        let handbrake_cmd = false;

        CarControls {
            throttle: throttle_cmd,
            steer: steer_cmd,
            brake: brake_cmd,
            handbrake: handbrake_cmd,
            reverse: false,
        }
    }
}
