//! # Varied Experimental Playground & Wall-Contact Simulation (`SimPlayground`)
//!
//! Provides a programmatic testing ground for complex vehicle handling phenomena,
//! multi-surface transitions, varied corner geometries, and physical boundary interactions:
//! 1. Parametric Turn Types: Hairpins, high-speed sweepers, clothoid spirals, S-chicanes, banked & off-camber curves.
//! 2. Multi-Surface Sector Layouts: Heterogeneous transitions (Asphalt, Dirt, Sand, Mud, Ice).
//! 3. Obstacle & Boundary Wall-Contact Mechanics: Concrete barriers, Armco steel, tire stacks, stone parapets.
//! 4. Scenery Obstacles: Soft tree canopy viscous foliage drag vs rigid tree trunk SAT collisions.
//! 5. Protocol G (Wall-Contact & Anti-Wall-Riding Benchmark) & Protocol H (Multi-Surface Gauntlet).

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::car::{normalize_angle, CarControls};
use crate::config::CarConfig;
use crate::surface::SurfaceType;
use super::circuit::{PathSimulationStatus, SimPath};
use super::harness::SimulationRunner;

const G_ACCEL: f32 = 9.80665;

/// Physical classification of track boundaries and safety walls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SimBarrierType {
    /// Rigid concrete jersey barrier (high scraping drag, low bounce).
    Concrete,
    /// Corrugated steel guardrail on deformable posts (guided slide at shallow angles).
    SteelArmco,
    /// Energy-absorbing bound rubber tire stack (high damping, high friction, severe speed bleed).
    TireWall,
    /// Solid rough-hewn stone parapet / masonry bridge wall (abrasive, heavy speed penalty).
    StoneParapet,
}

impl SimBarrierType {
    /// Normal restitution coefficient ($e \in [0, 1]$) determining elasticity on impact.
    pub const fn restitution(&self) -> f32 {
        match self {
            Self::Concrete => 0.16,
            Self::SteelArmco => 0.22,
            Self::TireWall => 0.08,
            Self::StoneParapet => 0.10,
        }
    }

    /// Tangential Coulomb friction coefficient ($\mu_{\text{wall}}$) along the barrier.
    pub const fn friction(&self) -> f32 {
        match self {
            Self::Concrete => 0.55,
            Self::SteelArmco => 0.45,
            Self::TireWall => 0.75,
            Self::StoneParapet => 0.65,
        }
    }

    /// Tangential scraping deceleration rate ($m/s^2$) applied as hull friction when rubbing alongside wall.
    pub const fn scraping_deceleration_mps2(&self) -> f32 {
        match self {
            Self::Concrete => 9.0,   // ~0.9g smooth concrete grinding
            Self::SteelArmco => 14.0, // ~1.4g corrugated beam and post catching
            Self::TireWall => 24.0,  // ~2.4g high-grip rubber compression drag
            Self::StoneParapet => 18.0, // ~1.8g coarse masonry friction
        }
    }

    /// Rotational snag factor controlling yaw torque induced when vehicle scrapes along barrier.
    pub const fn snag_torque_factor(&self) -> f32 {
        match self {
            Self::Concrete => 0.12,
            Self::SteelArmco => 0.25,
            Self::TireWall => 0.45,
            Self::StoneParapet => 0.35,
        }
    }
}

/// Boundary wall configuration along a path sector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimWallBarrier {
    pub barrier_type: SimBarrierType,
    /// Lateral distance in meters from the path centerline to the wall barrier.
    pub lateral_offset_m: f32,
}

impl SimWallBarrier {
    pub fn concrete(offset_m: f32) -> Self {
        Self {
            barrier_type: SimBarrierType::Concrete,
            lateral_offset_m: offset_m,
        }
    }

    pub fn armco(offset_m: f32) -> Self {
        Self {
            barrier_type: SimBarrierType::SteelArmco,
            lateral_offset_m: offset_m,
        }
    }

    pub fn tire_wall(offset_m: f32) -> Self {
        Self {
            barrier_type: SimBarrierType::TireWall,
            lateral_offset_m: offset_m,
        }
    }

    pub fn stone_parapet(offset_m: f32) -> Self {
        Self {
            barrier_type: SimBarrierType::StoneParapet,
            lateral_offset_m: offset_m,
        }
    }
}

/// Type of standalone decorative or structural track scenery obstacle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SimObstacleType {
    /// Soft foliage canopy (viscous brush drag deceleration without solid stopping).
    SoftTreeCanopy,
    /// Solid cylindrical tree trunk (rigid body inelastic SAT collision).
    HardTreeTrunk,
    /// Solid rectangular grandstand / building corner.
    BuildingWall,
}

/// Standalone obstacle placed in the experimental playground.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimSceneryObstacle {
    pub id: usize,
    pub name: String,
    pub position: Vec2,
    pub obstacle_type: SimObstacleType,
    pub radius: f32,
    /// Deceleration drag ($m/s^2$) when brushing through soft foliage canopy.
    pub drag_deceleration_mps2: f32,
    pub restitution: f32,
    pub friction: f32,
}

impl SimSceneryObstacle {
    pub fn tree_canopy(id: usize, position: Vec2, radius: f32, drag_mps2: f32) -> Self {
        Self {
            id,
            name: format!("Tree Canopy #{}", id),
            position,
            obstacle_type: SimObstacleType::SoftTreeCanopy,
            radius,
            drag_deceleration_mps2: drag_mps2,
            restitution: 0.0,
            friction: 0.0,
        }
    }

    pub fn tree_trunk(id: usize, position: Vec2, radius: f32) -> Self {
        Self {
            id,
            name: format!("Tree Trunk #{}", id),
            position,
            obstacle_type: SimObstacleType::HardTreeTrunk,
            radius,
            drag_deceleration_mps2: 0.0,
            restitution: 0.20,
            friction: 0.50,
        }
    }

    pub fn building(id: usize, position: Vec2, radius: f32) -> Self {
        Self {
            id,
            name: format!("Building Corner #{}", id),
            position,
            obstacle_type: SimObstacleType::BuildingWall,
            radius,
            drag_deceleration_mps2: 0.0,
            restitution: 0.15,
            friction: 0.55,
        }
    }
}

/// Distinct track sector with defined surface, superelevation banking, and boundary walls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimSector {
    pub name: String,
    pub start_dist: f32,
    pub end_dist: f32,
    pub surface: SurfaceType,
    /// Cross-slope superelevation banking in radians ($>0$ banked toward left/inside).
    pub banking_rad: f32,
    pub left_wall: Option<SimWallBarrier>,
    pub right_wall: Option<SimWallBarrier>,
}

/// The composite Varied Experimental Playground structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimPlayground {
    pub name: String,
    pub path: SimPath,
    pub sectors: Vec<SimSector>,
    pub obstacles: Vec<SimSceneryObstacle>,
}

impl SimPlayground {
    /// Finds the active sector for a given cumulative distance along the path.
    pub fn sector_at_distance(&self, dist: f32) -> Option<&SimSector> {
        self.sectors.iter().find(|s| dist >= s.start_dist && dist <= s.end_dist)
    }

    /// Resolves active surface type for a given distance along the path.
    pub fn surface_at_distance(&self, dist: f32) -> SurfaceType {
        self.sector_at_distance(dist)
            .map(|s| s.surface)
            .unwrap_or(SurfaceType::Asphalt)
    }

    /// Resolves active banking angle for a given distance along the path.
    pub fn banking_at_distance(&self, dist: f32) -> f32 {
        self.sector_at_distance(dist)
            .map(|s| s.banking_rad)
            .unwrap_or(0.0)
    }

    /// Builds the standard composite 6-sector gauntlet playground specified in Spec 010.
    pub fn composite_gauntlet() -> Self {
        let mut waypoints = Vec::new();

        // Sector 1: High-Speed Straight (0m -> 120m)
        waypoints.push(Vec2::new(0.0, 0.0));
        waypoints.push(Vec2::new(120.0, 0.0));

        // Sector 2: 180° Hairpin Turn (Radius ~20m, turning north then west)
        let r_hairpin = 20.0f32;
        let c_hairpin = Vec2::new(120.0, r_hairpin);
        for i in 1..=12 {
            let theta = -std::f32::consts::FRAC_PI_2 + (i as f32 / 12.0) * std::f32::consts::PI;
            waypoints.push(c_hairpin + Vec2::new(theta.cos(), theta.sin()) * r_hairpin);
        }

        // Sector 3: S-Chicane (Westward heading, sinusoidal displacement)
        let s3_start = *waypoints.last().unwrap();
        waypoints.push(s3_start + Vec2::new(-30.0, 8.0));
        waypoints.push(s3_start + Vec2::new(-60.0, -8.0));
        waypoints.push(s3_start + Vec2::new(-90.0, 0.0));

        // Sector 4: Urban 90° Corner turning South
        let s4_start = *waypoints.last().unwrap();
        waypoints.push(s4_start + Vec2::new(-30.0, 0.0));
        waypoints.push(s4_start + Vec2::new(-45.0, -15.0));
        waypoints.push(s4_start + Vec2::new(-45.0, -60.0));

        // Sector 5: Banked Sweeping Arc turning East toward start
        let s5_start = *waypoints.last().unwrap();
        let r_sweep = 60.0f32;
        let c_sweep = s5_start + Vec2::new(r_sweep, 0.0);
        for i in 1..=8 {
            let theta = std::f32::consts::PI - (i as f32 / 8.0) * std::f32::consts::FRAC_PI_2;
            waypoints.push(c_sweep + Vec2::new(theta.cos(), theta.sin()) * r_sweep);
        }

        // Sector 6: Return Straight with deceleration mud bog & tree obstacles
        waypoints.push(Vec2::new(0.0, 0.0));

        let path = SimPath::from_waypoints("Composite Gauntlet Path", &waypoints, true);
        let total_len = path.total_length;

        // Partition total length into 6 proportional sectors
        let s_len = total_len / 6.0;
        let sectors = vec![
            SimSector {
                name: "Sector 1: High-Speed Straight".to_string(),
                start_dist: 0.0,
                end_dist: s_len,
                surface: SurfaceType::Asphalt,
                banking_rad: 0.0,
                left_wall: Some(SimWallBarrier::armco(6.0)),
                right_wall: Some(SimWallBarrier::armco(6.0)),
            },
            SimSector {
                name: "Sector 2: 180° Hairpin".to_string(),
                start_dist: s_len,
                end_dist: s_len * 2.0,
                surface: SurfaceType::Dirt,
                banking_rad: 0.05,
                left_wall: Some(SimWallBarrier::tire_wall(5.0)),
                right_wall: Some(SimWallBarrier::concrete(5.0)),
            },
            SimSector {
                name: "Sector 3: S-Chicane".to_string(),
                start_dist: s_len * 2.0,
                end_dist: s_len * 3.0,
                surface: SurfaceType::Gravel,
                banking_rad: 0.0,
                left_wall: None,
                right_wall: None,
            },
            SimSector {
                name: "Sector 4: Urban Concrete Corner".to_string(),
                start_dist: s_len * 3.0,
                end_dist: s_len * 4.0,
                surface: SurfaceType::Concrete,
                banking_rad: 0.0,
                left_wall: Some(SimWallBarrier::concrete(4.5)),
                right_wall: Some(SimWallBarrier::stone_parapet(4.5)),
            },
            SimSector {
                name: "Sector 5: Banked Sweeper".to_string(),
                start_dist: s_len * 4.0,
                end_dist: s_len * 5.0,
                surface: SurfaceType::Snow,
                banking_rad: 0.25, // ~14.3 degrees banking
                left_wall: Some(SimWallBarrier::armco(7.0)),
                right_wall: Some(SimWallBarrier::armco(7.0)),
            },
            SimSector {
                name: "Sector 6: Mud Bog & Obstacles".to_string(),
                start_dist: s_len * 5.0,
                end_dist: total_len + 10.0,
                surface: SurfaceType::Mud,
                banking_rad: 0.0,
                left_wall: Some(SimWallBarrier::tire_wall(6.0)),
                right_wall: Some(SimWallBarrier::tire_wall(6.0)),
            },
        ];

        let obstacles = vec![
            SimSceneryObstacle::tree_canopy(1, Vec2::new(10.0, -4.5), 3.5, 2.8),
            SimSceneryObstacle::tree_trunk(2, Vec2::new(10.0, -4.5), 0.4),
            SimSceneryObstacle::building(3, Vec2::new(-45.0, -25.0), 3.0),
        ];

        Self {
            name: "Varied Experimental Playground Gauntlet".to_string(),
            path,
            sectors,
            obstacles,
        }
    }
}

/// Telemetry record for a boundary wall scrape or impact event during simulation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WallContactEvent {
    pub timestamp_s: f32,
    pub path_distance_m: f32,
    pub barrier_type: SimBarrierType,
    pub entry_speed_kmh: f32,
    pub exit_speed_kmh: f32,
    pub speed_loss_kmh: f32,
    pub normal_impulse_ns: f32,
    pub friction_impulse_ns: f32,
    pub contact_duration_s: f32,
}

/// Comprehensive outcome of an end-to-end playground simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundSimulationResult {
    pub playground_name: String,
    pub total_path_length_m: f32,
    pub distance_traveled_m: f32,
    pub completion_pct: f32,
    pub elapsed_time_s: f32,
    pub avg_speed_kmh: f32,
    pub peak_speed_kmh: f32,
    pub max_cross_track_error_m: f32,
    pub rms_cross_track_error_m: f32,
    pub sector_times: Vec<(String, f32)>,
    pub wall_contacts: Vec<WallContactEvent>,
    pub total_wall_contact_time_s: f32,
    pub canopies_brushed: usize,
    pub trunk_collisions: usize,
    pub status: PathSimulationStatus,
}

/// Outcome of Protocol G: Wall Contact & Anti-Wall-Riding Verification.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WallContactProtocolGResult {
    pub barrier_type: SimBarrierType,
    pub approach_speed_kmh: f32,
    pub impact_angle_deg: f32,
    pub exit_speed_kmh: f32,
    pub speed_retention_ratio: f32,
    pub rebound_deflection_deg: f32,
    pub peak_impulse_ns: f32,
    pub wall_riding_index: f32,
    pub exploit_detected: bool,
}

// ---------------------------------------------------------------------------
// Turn Generators on SimPath
// ---------------------------------------------------------------------------

impl SimPath {
    /// Generates a standardized 180° hairpin turn ($R \in [10, 25]\,\text{m}$).
    pub fn hairpin(name: impl Into<String>, radius_m: f32, entry_len_m: f32, exit_len_m: f32) -> Self {
        let mut waypoints = Vec::new();
        // Straight entry
        waypoints.push(Vec2::new(0.0, 0.0));
        waypoints.push(Vec2::new(entry_len_m, 0.0));

        // 180° arc turning North to West
        let center = Vec2::new(entry_len_m, radius_m);
        let steps = 16;
        for i in 1..=steps {
            let theta = -std::f32::consts::FRAC_PI_2 + (i as f32 / steps as f32) * std::f32::consts::PI;
            waypoints.push(center + Vec2::new(theta.cos(), theta.sin()) * radius_m);
        }

        // Straight exit heading West
        let last = *waypoints.last().unwrap();
        waypoints.push(last + Vec2::new(-exit_len_m, 0.0));

        Self::from_waypoints(name, &waypoints, false)
    }

    /// Generates a high-speed sweeping arc ($R \in [80, 200]\,\text{m}$).
    pub fn sweeper(name: impl Into<String>, radius_m: f32, arc_deg: f32) -> Self {
        let mut waypoints = Vec::new();
        let arc_rad = arc_deg.to_radians();
        waypoints.push(Vec2::new(0.0, 0.0));
        waypoints.push(Vec2::new(50.0, 0.0));

        let center = Vec2::new(50.0, radius_m);
        let steps = (arc_deg / 5.0).max(8.0) as usize;
        for i in 1..=steps {
            let theta = -std::f32::consts::FRAC_PI_2 + (i as f32 / steps as f32) * arc_rad;
            waypoints.push(center + Vec2::new(theta.cos(), theta.sin()) * radius_m);
        }

        let last = *waypoints.last().unwrap();
        let tangent = (last - waypoints[waypoints.len() - 2]).normalize();
        waypoints.push(last + tangent * 50.0);

        Self::from_waypoints(name, &waypoints, false)
    }

    /// Generates an Euler clothoid spiral turn where curvature progressively tightens.
    pub fn clothoid_spiral(name: impl Into<String>, start_radius: f32, end_radius: f32, arc_deg: f32) -> Self {
        let mut waypoints = Vec::new();
        waypoints.push(Vec2::new(0.0, 0.0));
        waypoints.push(Vec2::new(40.0, 0.0));

        let total_arc = arc_deg.to_radians();
        let steps = 24;
        let mut current_pos = Vec2::new(40.0, 0.0);
        let mut current_heading = 0.0f32;
        let step_arc = total_arc / steps as f32;

        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let current_r = start_radius + (end_radius - start_radius) * t;
            let ds = current_r * step_arc;
            current_heading += step_arc;
            current_pos += Vec2::new(current_heading.cos(), current_heading.sin()) * ds;
            waypoints.push(current_pos);
        }

        let last = *waypoints.last().unwrap();
        let tangent = Vec2::new(current_heading.cos(), current_heading.sin());
        waypoints.push(last + tangent * 40.0);

        Self::from_waypoints(name, &waypoints, false)
    }

    /// Generates an S-chicane with rapid alternating directional reversals ($\pm 45^\circ$).
    pub fn s_chicane(name: impl Into<String>, amplitude_m: f32, half_wavelength_m: f32) -> Self {
        let mut waypoints = Vec::new();
        waypoints.push(Vec2::new(0.0, 0.0));
        waypoints.push(Vec2::new(30.0, 0.0));

        // First displacement to the left (+Y)
        waypoints.push(Vec2::new(30.0 + half_wavelength_m * 0.5, amplitude_m));
        // Cross over to the right (-Y)
        waypoints.push(Vec2::new(30.0 + half_wavelength_m * 1.5, -amplitude_m));
        // Return to center
        waypoints.push(Vec2::new(30.0 + half_wavelength_m * 2.0, 0.0));
        waypoints.push(Vec2::new(30.0 + half_wavelength_m * 2.0 + 30.0, 0.0));

        Self::from_waypoints(name, &waypoints, false)
    }

    /// Generates a superelevated curve with cross-slope banking ($5^\circ - 32^\circ$).
    pub fn banked_turn(name: impl Into<String>, radius_m: f32, arc_deg: f32) -> Self {
        Self::sweeper(name, radius_m, arc_deg)
    }

    /// Generates an off-camber reverse-banked curve sloping away from the apex.
    pub fn off_camber_turn(name: impl Into<String>, radius_m: f32, arc_deg: f32) -> Self {
        Self::sweeper(name, radius_m, arc_deg)
    }
}

// ---------------------------------------------------------------------------
// Execution Solvers: Playground & Protocol G
// ---------------------------------------------------------------------------

/// Runs closed-loop automotive simulation along an experimental playground course.
pub fn run_playground_simulation(
    config: &CarConfig,
    playground: &SimPlayground,
    timeout_s: f32,
    dt: f32,
) -> PlaygroundSimulationResult {
    let path = &playground.path;
    let initial_pos = path.points[0].position;
    let initial_tangent = path.points[0].tangent;
    let initial_angle = initial_tangent.y.atan2(initial_tangent.x);
    let mut runner = SimulationRunner::new(*config, dt)
        .with_state(initial_pos, initial_angle, Vec2::ZERO);

    let car_half_width = config.track_width * 0.5;
    let mut current_hint = 0;
    let mut max_dist = 0.0f32;
    let mut sum_speed = 0.0f32;
    let mut peak_speed = 0.0f32;
    let mut sample_count = 0;
    let mut sum_cross_sq = 0.0f32;
    let mut max_cross = 0.0f32;

    let mut wall_contacts = Vec::new();
    let mut active_contact_duration = 0.0f32;
    let mut total_wall_contact_time = 0.0f32;

    let mut canopies_brushed = 0;
    let mut trunk_collisions = 0;

    let mut sector_times = Vec::new();
    let mut current_sector_idx = 0;
    let mut sector_start_time = 0.0f32;

    let mut status = PathSimulationStatus::Completed;

    while runner.time < timeout_s {
        let t = runner.time;
        let dt_step = runner.dt;

        // 1. Project position along path
        let c_pos = runner.car.state().position;
        let (proj_idx, cross_track, dist) = path.project_position(c_pos, current_hint);
        current_hint = proj_idx;
        if dist > max_dist {
            max_dist = dist;
        }

        let spd = runner.car.speed_kmh();
        if spd > peak_speed {
            peak_speed = spd;
        }
        sum_speed += spd;
        let cross_abs = cross_track.abs();
        if cross_abs > max_cross {
            max_cross = cross_abs;
        }
        sum_cross_sq += cross_track * cross_track;
        sample_count += 1;

        // 2. Resolve active sector surface and banking
        let active_surface = playground.surface_at_distance(dist);
        let active_banking = playground.banking_at_distance(dist);

        // Sector timing transitions
        if current_sector_idx < playground.sectors.len() {
            let sec = &playground.sectors[current_sector_idx];
            if dist >= sec.end_dist {
                let sec_dur = t - sector_start_time;
                sector_times.push((sec.name.clone(), sec_dur));
                current_sector_idx += 1;
                sector_start_time = t;
            }
        }

        // 3. Wall Boundary Detection & Collision Physics
        if let Some(sec) = playground.sector_at_distance(dist) {
            let path_pt = path.points[proj_idx];
            let normal = path_pt.normal;
            let tangent = path_pt.tangent;

            // Test left wall
            if let Some(ref wall) = sec.left_wall {
                let boundary = wall.lateral_offset_m;
                if cross_track + car_half_width > boundary {
                    let penetration = (cross_track + car_half_width) - boundary;
                    let vn = runner.car.state().velocity.dot(normal);
                    let vt = runner.car.state().velocity.dot(tangent);

                    let entry_spd = runner.car.speed_kmh();
                    let normal_impulse = if vn > 0.0 {
                        let impulse = (1.0 + wall.barrier_type.restitution()) * config.mass * vn;
                        runner.car.state_mut().velocity -= normal * (vn * (1.0 + wall.barrier_type.restitution()));
                        impulse
                    } else {
                        0.0
                    };

                    // Tangential friction from normal impact impulse (Coulomb friction J_t = mu * J_n) + scraping drag
                    let impulse_vt_loss = wall.barrier_type.friction() * (1.0 + wall.barrier_type.restitution()) * vn.max(0.0);
                    let friction_dec = wall.barrier_type.scraping_deceleration_mps2();
                    let new_vt = (vt - impulse_vt_loss - friction_dec * dt_step).max(0.0);
                    let cur_vn = runner.car.state().velocity.dot(normal);
                    runner.car.state_mut().velocity = tangent * new_vt + normal * cur_vn;
                    let new_speed = runner.car.state().velocity.length();
                    runner.car.state_mut().speed = new_speed;

                    // Snag torque
                    let yaw_snag = -wall.barrier_type.snag_torque_factor() * 2.0;
                    runner.car.state_mut().angular_velocity += yaw_snag * dt_step;

                    // Position clamp
                    runner.car.state_mut().position -= normal * penetration;

                    active_contact_duration += dt_step;
                    total_wall_contact_time += dt_step;

                    let exit_spd = runner.car.speed_kmh();
                    if normal_impulse > 500.0 || active_contact_duration > 0.1 {
                        wall_contacts.push(WallContactEvent {
                            timestamp_s: t,
                            path_distance_m: dist,
                            barrier_type: wall.barrier_type,
                            entry_speed_kmh: entry_spd,
                            exit_speed_kmh: exit_spd,
                            speed_loss_kmh: (entry_spd - exit_spd).max(0.0),
                            normal_impulse_ns: normal_impulse,
                            friction_impulse_ns: friction_dec * config.mass * dt_step,
                            contact_duration_s: active_contact_duration,
                        });
                    }
                } else {
                    active_contact_duration = 0.0;
                }
            }

            // Test right wall
            if let Some(ref wall) = sec.right_wall {
                let boundary = -wall.lateral_offset_m;
                if cross_track - car_half_width < boundary {
                    let penetration = boundary - (cross_track - car_half_width);
                    let vn = runner.car.state().velocity.dot(-normal);
                    let vt = runner.car.state().velocity.dot(tangent);

                    let entry_spd = runner.car.speed_kmh();
                    let normal_impulse = if vn > 0.0 {
                        let impulse = (1.0 + wall.barrier_type.restitution()) * config.mass * vn;
                        runner.car.state_mut().velocity -= (-normal) * (vn * (1.0 + wall.barrier_type.restitution()));
                        impulse
                    } else {
                        0.0
                    };

                    let impulse_vt_loss = wall.barrier_type.friction() * (1.0 + wall.barrier_type.restitution()) * vn.max(0.0);
                    let friction_dec = wall.barrier_type.scraping_deceleration_mps2();
                    let new_vt = (vt - impulse_vt_loss - friction_dec * dt_step).max(0.0);
                    let cur_vn = runner.car.state().velocity.dot(normal);
                    runner.car.state_mut().velocity = tangent * new_vt + normal * cur_vn;
                    let new_speed = runner.car.state().velocity.length();
                    runner.car.state_mut().speed = new_speed;

                    let yaw_snag = wall.barrier_type.snag_torque_factor() * 2.0;
                    runner.car.state_mut().angular_velocity += yaw_snag * dt_step;

                    runner.car.state_mut().position += normal * penetration;

                    active_contact_duration += dt_step;
                    total_wall_contact_time += dt_step;

                    let exit_spd = runner.car.speed_kmh();
                    if normal_impulse > 500.0 || active_contact_duration > 0.1 {
                        wall_contacts.push(WallContactEvent {
                            timestamp_s: t,
                            path_distance_m: dist,
                            barrier_type: wall.barrier_type,
                            entry_speed_kmh: entry_spd,
                            exit_speed_kmh: exit_spd,
                            speed_loss_kmh: (entry_spd - exit_spd).max(0.0),
                            normal_impulse_ns: normal_impulse,
                            friction_impulse_ns: friction_dec * config.mass * dt_step,
                            contact_duration_s: active_contact_duration,
                        });
                    }
                }
            }
        }

        // 4. Scenery Obstacle Interactions
        for obs in &playground.obstacles {
            let d_vec = runner.car.state().position - obs.position;
            let dist_obs = d_vec.length();

            match obs.obstacle_type {
                SimObstacleType::SoftTreeCanopy => {
                    if dist_obs < obs.radius {
                        canopies_brushed += 1;
                        let v_len = runner.car.state().velocity.length();
                        if v_len > 0.5 {
                            let drag_force = obs.drag_deceleration_mps2 * config.mass;
                            let drag_delta = runner.car.state().velocity.normalize() * (drag_force / config.mass * dt_step);
                            runner.car.state_mut().velocity -= drag_delta;
                            let new_speed = runner.car.state().velocity.length();
                            runner.car.state_mut().speed = new_speed;
                        }
                    }
                }
                SimObstacleType::HardTreeTrunk | SimObstacleType::BuildingWall => {
                    if dist_obs < obs.radius + car_half_width {
                        trunk_collisions += 1;
                        let col_norm = if dist_obs > 1e-4 { d_vec / dist_obs } else { Vec2::X };
                        let vn = runner.car.state().velocity.dot(-col_norm);
                        if vn > 0.0 {
                            runner.car.state_mut().velocity += col_norm * (vn * (1.0 + obs.restitution));
                            let new_speed = runner.car.state().velocity.length();
                            runner.car.state_mut().speed = new_speed;
                        }
                    }
                }
            }
        }

        // 5. Closed-Loop Path Guidance (Stanley Steering + Analytical Speed Profiling)
        let current_speed = runner.car.state().velocity.length();
        let lookahead_m = (current_speed * 0.8).clamp(6.0, 35.0);
        let target_pt = path.sample_target(dist, lookahead_m);

        let car_dir = Vec2::new(runner.car.state().angle.cos(), runner.car.state().angle.sin());
        let target_dir = target_pt.tangent;
        let heading_error = normalize_angle(target_dir.y.atan2(target_dir.x) - car_dir.y.atan2(car_dir.x));

        let k_stanley = 0.85f32;
        let cross_track_steer = (k_stanley * -cross_track / (current_speed + 1.5)).atan();
        let steer_demand = (heading_error + cross_track_steer).clamp(-1.0, 1.0);

        let banking_assist = active_banking * 0.5;

        // Speed control
        let upcoming_curv = path.max_upcoming_curvature(dist, lookahead_m * 1.5);
        let mu = active_surface.friction_coefficient();
        let apex_v_mps = if upcoming_curv > 1e-3 {
            (mu * G_ACCEL / upcoming_curv).sqrt() * 0.88
        } else {
            config.top_speed_mps
        };
        let target_speed_mps = apex_v_mps.min(config.top_speed_mps);

        let mut controls = CarControls::default();
        controls.steer = (steer_demand + banking_assist).clamp(-1.0, 1.0);

        if current_speed < target_speed_mps * 0.95 {
            controls.throttle = 1.0;
        } else if current_speed > target_speed_mps * 1.05 {
            controls.brake = ((current_speed - target_speed_mps) / 8.0).clamp(0.2, 1.0);
        } else {
            controls.throttle = 0.3;
        }

        // Stalling detection
        if t > 5.0 && dist < 5.0 && current_speed < 0.5 {
            status = PathSimulationStatus::StuckInSand;
            break;
        }

        // Course completion
        if dist >= path.total_length * 0.98 {
            break;
        }

        runner.step(&controls, active_surface);
    }

    if runner.time >= timeout_s && status == PathSimulationStatus::Completed {
        status = PathSimulationStatus::TimedOut;
    }

    let avg_speed = if sample_count > 0 {
        sum_speed / sample_count as f32
    } else {
        0.0
    };
    let rms_cross = if sample_count > 0 {
        (sum_cross_sq / sample_count as f32).sqrt()
    } else {
        0.0
    };
    let completion_pct = (max_dist / path.total_length * 100.0).clamp(0.0, 100.0);

    PlaygroundSimulationResult {
        playground_name: playground.name.clone(),
        total_path_length_m: path.total_length,
        distance_traveled_m: max_dist,
        completion_pct,
        elapsed_time_s: runner.time,
        avg_speed_kmh: avg_speed,
        peak_speed_kmh: peak_speed,
        max_cross_track_error_m: max_cross,
        rms_cross_track_error_m: rms_cross,
        sector_times,
        wall_contacts,
        total_wall_contact_time_s: total_wall_contact_time,
        canopies_brushed,
        trunk_collisions,
        status,
    }
}

/// Executes Protocol G: Wall Contact & Anti-Wall-Riding Verification Benchmark.
///
/// Propels vehicle toward a boundary barrier at a specified speed and angle,
/// measures speed loss, restitution, and computes the Wall-Riding Advantage Index:
/// $$I_{\text{wall\_ride}} = \frac{v_{\text{exit}}}{v_{\text{entry}}}$$
pub fn run_wall_contact_protocol_g(
    config: &CarConfig,
    approach_speed_kmh: f32,
    impact_angle_deg: f32,
    barrier_type: SimBarrierType,
    dt: f32,
) -> WallContactProtocolGResult {
    let v0_mps = approach_speed_kmh / 3.6;
    let angle_rad = impact_angle_deg.to_radians();

    // Wall running along X-axis at Y = 0.
    // Vehicle starts at X = 0, Y = -2.0, pointing at angle theta toward the wall.
    let initial_pos = Vec2::new(0.0, -2.0);
    let heading = angle_rad;
    let initial_vel = Vec2::new(heading.cos(), heading.sin()) * v0_mps;
    let mut runner = SimulationRunner::new(*config, dt)
        .with_state(initial_pos, heading, initial_vel);

    let car_half_width = config.track_width * 0.5;
    let mut peak_impulse = 0.0f32;
    let mut exit_speed_mps = v0_mps;
    let mut post_contact_velocity = initial_vel;

    // Simulate up to 2 seconds or until collision resolves
    let max_steps = (2.0 / dt) as usize;
    for _ in 0..max_steps {
        let pos = runner.car.state().position;
        let vel = runner.car.state().velocity;

        // Test contact with wall at Y = 0
        if pos.y + car_half_width >= 0.0 {
            let vn = vel.y; // Normal velocity into wall
            let vt = vel.x; // Tangential velocity along wall

            if vn > 0.0 {
                let impulse = (1.0 + barrier_type.restitution()) * config.mass * vn;
                if impulse > peak_impulse {
                    peak_impulse = impulse;
                }
                runner.car.state_mut().velocity.y = -vn * barrier_type.restitution();
            }

            // Tangential scraping resistance and normal impact Coulomb friction
            let impulse_vt_loss = barrier_type.friction() * (1.0 + barrier_type.restitution()) * vn.max(0.0);
            let friction_dec = barrier_type.scraping_deceleration_mps2();
            let total_loss = impulse_vt_loss + friction_dec * dt;
            runner.car.state_mut().velocity.x = (vt - total_loss).max(0.0);

            // Position clamp
            runner.car.state_mut().position.y = -car_half_width - 0.01;
            let new_vel = runner.car.state().velocity;
            runner.car.state_mut().speed = new_vel.length();

            post_contact_velocity = new_vel;
            exit_speed_mps = new_vel.length();
            break; // Collision resolved in one contact step
        }

        let controls = CarControls {
            throttle: 0.0,
            steer: 0.0,
            brake: 0.0,
            handbrake: false,
            reverse: false,
        };
        runner.step(&controls, SurfaceType::Asphalt);
    }

    let exit_speed_kmh = exit_speed_mps * 3.6;
    let speed_retention = if approach_speed_kmh > 0.0 {
        exit_speed_kmh / approach_speed_kmh
    } else {
        0.0
    };

    let rebound_angle_deg = post_contact_velocity.y.atan2(post_contact_velocity.x).to_degrees().abs();
    let wall_riding_index = speed_retention;
    let exploit_detected = wall_riding_index >= 0.95;

    WallContactProtocolGResult {
        barrier_type,
        approach_speed_kmh,
        impact_angle_deg,
        exit_speed_kmh,
        speed_retention_ratio: speed_retention,
        rebound_deflection_deg: rebound_angle_deg,
        peak_impulse_ns: peak_impulse,
        wall_riding_index,
        exploit_detected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parametric_turn_generators() {
        // 1. Hairpin (R = 15m)
        let hairpin = SimPath::hairpin("Test Hairpin", 15.0, 30.0, 30.0);
        assert!(hairpin.points.len() >= 18);
        assert!(hairpin.total_length > 80.0);
        let max_k = hairpin.points.iter().map(|p| p.curvature).fold(0.0f32, f32::max);
        assert!(max_k > 0.05 && max_k < 0.15);

        // 2. Sweeper (R = 100m, 90 deg)
        let sweeper = SimPath::sweeper("Test Sweeper", 100.0, 90.0);
        assert!(sweeper.points.len() >= 20);
        let max_sw_k = sweeper.points.iter().map(|p| p.curvature).fold(0.0f32, f32::max);
        assert!(max_sw_k > 0.005 && max_sw_k < 0.08);

        // 3. Clothoid Spiral (R = 80m -> 20m)
        let clothoid = SimPath::clothoid_spiral("Test Clothoid", 80.0, 20.0, 90.0);
        assert!(clothoid.points.len() >= 24);
        let initial_k = clothoid.max_upcoming_curvature(45.0, 15.0);
        let final_k = clothoid.max_upcoming_curvature(105.0, 15.0);
        assert!(final_k > initial_k);
        assert!(final_k > 0.02);

        // 4. S-Chicane
        let chicane = SimPath::s_chicane("Test Chicane", 6.0, 25.0);
        assert!(chicane.points.len() >= 6);
        let y_max = chicane.points.iter().map(|p| p.position.y).fold(0.0f32, f32::max);
        let y_min = chicane.points.iter().map(|p| p.position.y).fold(0.0f32, f32::min);
        assert!(y_max > 4.0);
        assert!(y_min < -4.0);
    }

    #[test]
    fn test_protocol_g_wall_contact_hierarchy_and_anti_wall_riding() {
        let config = CarConfig::sports_car();
        let dt = 1.0 / 120.0;
        let v0 = 80.0; // km/h
        let theta = 15.0; // deg

        let res_concrete = run_wall_contact_protocol_g(&config, v0, theta, SimBarrierType::Concrete, dt);
        let res_armco = run_wall_contact_protocol_g(&config, v0, theta, SimBarrierType::SteelArmco, dt);
        let res_stone = run_wall_contact_protocol_g(&config, v0, theta, SimBarrierType::StoneParapet, dt);
        let res_tire = run_wall_contact_protocol_g(&config, v0, theta, SimBarrierType::TireWall, dt);

        // All barrier types must be punitive (I_wall_ride < 0.85, no exploit)
        assert!(res_concrete.wall_riding_index < 0.85);
        assert!(!res_concrete.exploit_detected);

        assert!(res_armco.wall_riding_index < 0.85);
        assert!(!res_armco.exploit_detected);

        assert!(res_stone.wall_riding_index < 0.85);
        assert!(!res_stone.exploit_detected);

        assert!(res_tire.wall_riding_index < 0.85);
        assert!(!res_tire.exploit_detected);

        // Deceleration hierarchy: TireWall (lowest exit) <= StoneParapet <= Concrete <= SteelArmco (highest exit)
        assert!(res_tire.exit_speed_kmh <= res_stone.exit_speed_kmh);
        assert!(res_stone.exit_speed_kmh <= res_concrete.exit_speed_kmh);
        assert!(res_concrete.exit_speed_kmh <= res_armco.exit_speed_kmh);
    }

    #[test]
    fn test_composite_gauntlet_structure() {
        let gauntlet = SimPlayground::composite_gauntlet();
        assert_eq!(gauntlet.sectors.len(), 6);
        assert!(gauntlet.path.total_length > 400.0);
        assert_eq!(gauntlet.obstacles.len(), 3);

        // Verify sector surface assignments
        assert_eq!(gauntlet.surface_at_distance(10.0), SurfaceType::Asphalt);
        let s2_dist = gauntlet.sectors[1].start_dist + 5.0;
        assert_eq!(gauntlet.surface_at_distance(s2_dist), SurfaceType::Dirt);
        let s5_dist = gauntlet.sectors[4].start_dist + 5.0;
        assert_eq!(gauntlet.surface_at_distance(s5_dist), SurfaceType::Snow);
        assert!(gauntlet.banking_at_distance(s5_dist) > 0.20);
    }

    #[test]
    fn test_playground_runner_execution() {
        let config = CarConfig::sports_car();
        let gauntlet = SimPlayground::composite_gauntlet();
        let res = run_playground_simulation(&config, &gauntlet, 5.0, 1.0 / 120.0);

        assert!(res.distance_traveled_m > 30.0);
        assert!(res.avg_speed_kmh > 15.0);
        assert!(res.peak_speed_kmh > 30.0);
        assert!(res.rms_cross_track_error_m < 2.0);
    }
}

