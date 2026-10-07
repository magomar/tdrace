//! Bot driver AI: the line-following controller, driver tiers and styles, and the human layer.
//!
//! Moved from `tdrace-app/src/ai/` for spec 056 (`specs/056_racekit_headless_race_world.md`).
//! It drives any vehicle that implements [`BotVehicle`] (spec 065), including `wheelbase::Car`.

pub mod humanize;
pub mod rng;

pub use humanize::{BotDrivingStats, HumanDriver, HumanTraits, MistakeKind};
pub use rng::LcgRng;

use arcade_race_core::Body2D;
use arcade_race_core::track::{LineSegment, Track, TrackLayout, TrackNetwork, TrackSpline};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use wheelbase::{normalize_angle, Car, CarControls};

/// What the bot driver needs from a vehicle besides its [`Body2D`] state. Spec 065.
pub trait BotVehicle: Body2D {
    /// Unit vector to the vehicle's right.
    fn right_vector(&self) -> Vec2;
    /// Top speed on the flat, in m/s.
    fn top_speed_mps(&self) -> f32;
    /// Tyre (or hoof) grip coefficient used for braking and cornering limits.
    fn grip(&self) -> f32;
    /// Grip the controller plans its corner speeds with (in g). The default is the value tuned for
    /// cars; a vehicle with much less grip returns its own, somewhat below [`BotVehicle::grip`].
    fn planning_grip(&self) -> f32 {
        0.78
    }
    /// Maximum tire wear across wheels [0.0..1.0]. The default returns 0.0.
    fn max_tire_wear(&self) -> f32 {
        0.0
    }
    /// Chassis health [0.0..1.0]. The default returns 1.0.
    fn health(&self) -> f32 {
        1.0
    }
}

impl BotVehicle for Car {
    #[inline]
    fn right_vector(&self) -> Vec2 {
        Car::right_vector(self)
    }
    #[inline]
    fn top_speed_mps(&self) -> f32 {
        self.config.top_speed_mps
    }
    #[inline]
    fn grip(&self) -> f32 {
        self.config.tire.grip
    }
    #[inline]
    fn max_tire_wear(&self) -> f32 {
        self.state.wheels.iter().map(|w| w.wear).fold(0.0, f32::max)
    }
    #[inline]
    fn health(&self) -> f32 {
        self.state.health
    }
}

/// Tactical philosophy and driving personality (6 styles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

    /// Compact badge tag (e.g. "T1" .. "T5") for in-game floating nameplates.
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Rookie => "T1",
            Self::Amateur => "T2",
            Self::Contender => "T3",
            Self::Pro => "T4",
            Self::Legend => "T5",
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
                // Spec 046: 0.88 -> 0.82, so a keyboard driver on Balanced beats a Tier 1 grid. 0.80 since
                // curbs no longer add grip (tdrace-le75): the keyboard driver lost 1.1 s on Ridge Ring.
                pace_limit: 0.80,
                brake_padding: 0.22,
                avoidance_padding: 2.5,
                consistency: 0.60,
                composure: 0.48,
            },
            DriverTier::Amateur => Self {
                tier,
                pace_limit: 0.90,
                brake_padding: 0.12,
                avoidance_padding: 1.4,
                consistency: 0.72,
                composure: 0.60,
            },
            DriverTier::Contender => Self {
                tier,
                pace_limit: 0.96,
                brake_padding: 0.06,
                avoidance_padding: 0.7,
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
    /// Human variation and mistakes (spec 046). `HumanTraits::none()` for the fixed presets.
    pub traits: HumanTraits,
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
            traits: HumanTraits::none(),
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
            traits: HumanTraits::none(),
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
            traits: HumanTraits::none(),
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
            traits: HumanTraits::none(),
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
            traits: HumanTraits::none(),
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
            traits: HumanTraits::none(),
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
            traits: HumanTraits::none(),
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
            traits: HumanTraits::none(),
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
            traits: HumanTraits::for_style_and_quality(style, quality, style_aggression),
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

/// The largest heading change between the road at the bot and at its steering target.
const MAX_TARGET_TURN_RAD: f32 = 75.0 * std::f32::consts::PI / 180.0;
/// The shortest look-ahead when a tight turn shortens it (m).
const MIN_TIGHT_LOOKAHEAD_M: f32 = 3.0;
/// How far the line to the target stays from a close wall: half a car plus a margin (m).
const WALL_CLEARANCE_M: f32 = 1.2;
/// How strongly a car closer than WALL_CLEARANCE_M to a close wall aims away from it (m per m).
const CAR_WALL_PUSH: f32 = 3.0;
/// How far outside its own road a bot's car must be before it follows another branch it is on (m).
const OFF_ROUTE_MARGIN_M: f32 = 2.0;
/// How far past the edge of a branch road a bot's car still counts as on that branch (m).
const BRANCH_REACH_M: f32 = 8.0;
/// A slow bot pointing farther than this from its target turns round with a three-point turn (rad).
const TURN_START_RAD: f32 = 1.75;
/// The turn ends, driving forward, once the nose points this close to the target (rad).
const TURN_DONE_RAD: f32 = 0.6;
/// Reversing ends once the nose points this close to the target: full lock forward finishes the turn (rad).
const TURN_REVERSE_DONE_RAD: f32 = 1.0;
/// Below this speed a bot that should be moving is stuck (m/s).
const STUCK_SPEED: f32 = 0.4;
/// Slowest speed at which a bot starts a turn by itself (m/s).
const TURN_START_SPEED: f32 = 3.0;
/// How far past the front (or rear) bumper a turn looks for the edge of the drivable ground (m).
const TURN_PROBE_M: f32 = 1.0;
/// Drivable run-off past a road edge without a wall (m).
const TURN_RUNOFF_M: f32 = 3.0;
/// Gap a turn keeps from a wall (m).
const TURN_WALL_GAP_M: f32 = 0.3;
/// Pedal during a turn: full throttle at full lock spins a rear-drive car on the spot.
const TURN_THROTTLE: f32 = 0.6;
/// Shortest leg that the edge of the drivable ground can end (s).
const TURN_MIN_LEG_S: f32 = 0.3;
/// Longest single forward or reverse leg, and longest whole turn (s).
const TURN_LEG_S: f32 = 3.0;
const TURN_MAX_S: f32 = 12.0;

/// A three-point turn in progress (tdrace-le75): forward at full lock towards the target until the front
/// nears the edge of the drivable ground, then reverse at the opposite lock until the rear does, and so on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThreePointTurn {
    pub reversing: bool,
    /// Forward steering of the whole turn. Chosen once: near 180 degrees from the target the shorter way
    /// round flips from tick to tick, and each leg undid the last one.
    steer: f32,
    leg_time: f32,
    total_time: f32,
    blocked_time: f32,
}

impl ThreePointTurn {
    fn new(reversing: bool, steer: f32) -> Self {
        Self { reversing, steer, leg_time: 0.0, total_time: 0.0, blocked_time: 0.0 }
    }
}

/// Strategic decision mode for AI navigating tracks with multiple branches or Joker laps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BotRouteStrategy {
    /// Follow a fixed, explicit circuit layout ID (e.g. "main", "joker", "infield_short").
    FixedLayout(String),
    /// Rallycross Joker strategy: take Joker route on designated lap or adaptively under heavy traffic.
    RallycrossJoker {
        planned_joker_lap: u32,
        adaptive_traffic_undercut: bool,
    },
    /// Dynamically pick the route variant with the least opposing traffic ahead.
    DynamicTrafficAvoidance,
}

impl Default for BotRouteStrategy {
    fn default() -> Self {
        Self::DynamicTrafficAvoidance
    }
}

/// What a bot reads from its race tracker each tick (spec 082), set with [`BotAiDriver::sync_race_state`].
/// Without it the bot counts laps and jokers itself.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BotRaceState {
    /// Current lap, from 1.
    pub lap: u32,
    /// Joker laps completed.
    pub jokers: u32,
    /// Laps in the race.
    pub total_laps: u32,
    /// Joker laps the race rule requires (0 when the rule is off).
    pub mandatory_jokers: u32,
}

/// The joker layout (if any) and the main layout of `network`. The main layout is the default one, or the
/// first non-joker layout when the default is a joker.
fn joker_and_main_layout_ids(network: &TrackNetwork) -> (Option<String>, String) {
    let is_joker = |l: &TrackLayout| l.id.to_lowercase().contains("joker") || l.display_name.to_lowercase().contains("joker");
    let joker = network.layouts.iter().find(|l| is_joker(l)).map(|l| l.id.clone());
    let main = if network.default_layout_id.to_lowercase().contains("joker") {
        network
            .layouts
            .iter()
            .find(|l| !is_joker(l))
            .map(|l| l.id.clone())
            .unwrap_or_else(|| network.default_layout_id.clone())
    } else {
        network.default_layout_id.clone()
    };
    (joker, main)
}

/// The layout of the branch nearest to the car at `pos` that its own layout `own` does not contain: the car is on it
/// or within `BRANCH_REACH_M` of its edge, and nearer to it than the `own_distance` it is from its own road.
fn layout_of_branch_under(network: &TrackNetwork, own: &TrackLayout, pos: Vec2, own_distance: f32) -> Option<String> {
    network
        .segments
        .iter()
        .filter(|seg| !own.segment_sequence.contains(&seg.id))
        .filter_map(|seg| {
            let q = seg.project_point(pos);
            (q.distance_to_spline < q.track_width * 0.5 + BRANCH_REACH_M && q.distance_to_spline < own_distance)
                .then_some((q.distance_to_spline, seg))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .and_then(|(_, seg)| network.layouts.iter().find(|l| l.segment_sequence.contains(&seg.id)))
        .map(|l| l.id.clone())
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
    /// The three-point turn the bot is making, if any.
    pub turn: Option<ThreePointTurn>,
    /// Track distance at the last 5 m of progress, and the time since (spec 046 watchdog).
    pub progress_mark: f32,
    pub no_progress_timer: f32,
    /// Distance from the road centre at the last progress: getting 2 m closer to the road also counts.
    pub progress_lateral: f32,
    /// Watchdog recoveries since the last progress. Each one turns the other way round.
    pub recovery_attempts: u32,
    pub last_pos: Option<Vec2>,
    pub total_distance_travelled: f32,
    pub human: HumanDriver,
    /// Whether this bot is actively executing a pit stop entry or pit road traverse.
    pub is_pitting: bool,
    /// Whether the bot is physically inside and navigating the pit lane (Spec 084).
    pub is_in_pit_lane: bool,
    /// Whether the bot has stopped in its stall and received fresh tires/repairs.
    pub pit_serviced: bool,
    /// Active route strategy for multi-branch track navigation.
    pub route_strategy: BotRouteStrategy,
    /// Identifier of the currently active circuit layout.
    pub active_layout_id: Option<String>,
    /// Cache of previously resolved layout ID.
    pub cached_layout_id: Option<String>,
    /// Precomputed composite spline for active layout to prevent per-frame allocations.
    pub cached_layout_spline: Option<TrackSpline>,
    /// Current lap tracked by this bot.
    pub current_lap: u32,
    /// Number of joker laps completed by this bot.
    pub joker_laps_taken: u32,
    /// Flag indicating whether the bot is currently executing a Joker lap.
    pub was_in_joker: bool,
    /// Lap and joker count from the race tracker, when the bot races in a world that tracks them.
    pub race_state: Option<BotRaceState>,
    /// Lap the active route was chosen for; the route is chosen once per lap.
    pub route_lap: Option<u32>,
    /// Previous progress distance for detecting lap transitions.
    pub last_progress_dist: f32,
    /// Assigned pit stall index along the team pit road.
    pub pit_stall_idx: usize,
    /// Cooldown timer in seconds after leaving pit lane before considering another pit stop.
    pub pit_cooldown: f32,
    /// Timer measuring duration stationary inside team pit box stall (s).
    pub stall_stop_timer: f32,
}

impl BotAiDriver {
    pub fn new(profile: BotProfile) -> Self {
        Self::with_seed(profile, 0)
    }

    /// A bot whose human layer (spec 046) draws from `seed`. The same seed gives the same race.
    pub fn with_seed(profile: BotProfile, seed: u64) -> Self {
        Self {
            human: HumanDriver::new(profile.traits, seed),
            profile,
            prev_heading_error: 0.0,
            has_prev_heading: false,
            current_target_dist: 0.0,
            avoidance_lateral_bias: 0.0,
            stuck_timer: 0.0,
            turn: None,
            progress_mark: 0.0,
            no_progress_timer: 0.0,
            progress_lateral: 0.0,
            recovery_attempts: 0,
            last_pos: None,
            total_distance_travelled: 0.0,
            is_pitting: false,
            is_in_pit_lane: false,
            pit_serviced: false,
            route_strategy: BotRouteStrategy::default(),
            active_layout_id: None,
            cached_layout_id: None,
            cached_layout_spline: None,
            current_lap: 1,
            joker_laps_taken: 0,
            was_in_joker: false,
            race_state: None,
            route_lap: None,
            last_progress_dist: 0.0,
            pit_stall_idx: 0,
            pit_cooldown: 0.0,
            stall_stop_timer: 0.0,
        }
    }

    pub fn with_in_pit_lane(mut self, in_pit: bool) -> Self {
        self.is_in_pit_lane = in_pit;
        self
    }

    pub fn with_route_strategy(mut self, strategy: BotRouteStrategy) -> Self {
        self.route_strategy = strategy;
        self
    }

    pub fn with_active_layout(mut self, layout_id: impl Into<String>) -> Self {
        self.active_layout_id = Some(layout_id.into());
        self
    }

    /// Takes the lap and joker count from the race tracker and stops the bot's own lap counting.
    pub fn sync_race_state(&mut self, state: BotRaceState) {
        self.current_lap = state.lap;
        self.joker_laps_taken = state.jokers;
        self.race_state = Some(state);
    }

    /// Evaluates route strategy against the track network and opponent cars to select
    /// the active circuit layout. The route is chosen once per lap: the finish line comes before the split
    /// on the RX networks, so a route chosen at the start of a lap never changes inside a branch.
    pub fn decide_active_layout<V: Body2D>(&mut self, track: &Track, other_cars: &[&V]) -> Option<String> {
        let network = track.network.as_ref()?;
        if network.layouts.is_empty() {
            return None;
        }
        if self.route_lap == Some(self.current_lap) && self.active_layout_id.is_some() {
            return self.active_layout_id.clone();
        }
        self.route_lap = Some(self.current_lap);

        // Joker cap and last-lap failsafe. Without a race state the bot owes one joker, as before spec 082.
        let mandatory = self.race_state.map_or(1, |s| s.mandatory_jokers);
        let owes_joker = self.joker_laps_taken < mandatory;
        let last_lap = self.race_state.is_some_and(|s| self.current_lap >= s.total_laps);
        let (joker_id, main_id) = joker_and_main_layout_ids(network);

        match &self.route_strategy {
            BotRouteStrategy::FixedLayout(ref layout_id) => {
                if network.get_layout(layout_id).is_some() {
                    Some(layout_id.clone())
                } else {
                    Some(network.default_layout_id.clone())
                }
            }
            BotRouteStrategy::RallycrossJoker {
                planned_joker_lap,
                adaptive_traffic_undercut,
            } => {
                let main_layout_id = main_id;
                let joker_id = match joker_id {
                    Some(j) => j,
                    None => return Some(main_layout_id),
                };

                if !owes_joker {
                    return Some(main_layout_id);
                }

                let mut take_joker = self.current_lap >= *planned_joker_lap || last_lap;
                if !take_joker && *adaptive_traffic_undercut && self.current_lap >= 2 {
                    if let Some(pos) = self.last_pos {
                        let heavy_traffic = other_cars.iter().any(|opp| {
                            let dist = (opp.position() - pos).length();
                            dist < 20.0
                        });
                        if heavy_traffic {
                            take_joker = true;
                        }
                    }
                }

                if take_joker {
                    Some(joker_id)
                } else {
                    Some(main_layout_id)
                }
            }
            BotRouteStrategy::DynamicTrafficAvoidance => {
                if network.layouts.len() <= 1 {
                    return Some(network.default_layout_id.clone());
                }
                if let Some(joker_id) = joker_id {
                    if !owes_joker {
                        return Some(main_id);
                    }
                    if last_lap {
                        return Some(joker_id);
                    }
                }
                let mut best_layout = network.default_layout_id.clone();
                let mut min_traffic = usize::MAX;

                for layout in &network.layouts {
                    let mut traffic_count = 0;
                    let fallback;
                    let comp_spline = match network.composite_spline_for_layout(&layout.id) {
                        Some(s) => Some(s),
                        None => {
                            fallback = network.build_composite_spline_for_layout(&layout.id);
                            fallback.as_ref()
                        }
                    };

                    if let Some(comp_spline) = comp_spline {
                        for opp in other_cars {
                            let proj = comp_spline.project_point(opp.position());
                            if proj.is_on_track && proj.distance_to_spline < 8.0 {
                                traffic_count += 1;
                            }
                        }
                    }
                    if traffic_count < min_traffic {
                        min_traffic = traffic_count;
                        best_layout = layout.id.clone();
                    }
                }
                Some(best_layout)
            }
        }
    }

    /// Configures the assigned team pit box stall index.
    pub fn with_pit_stall(mut self, stall_idx: usize) -> Self {
        self.pit_stall_idx = stall_idx;
        self
    }

    /// Synchronizes the current race lap number for tactical pit stop gating.
    pub fn with_current_lap(mut self, lap: u32) -> Self {
        self.current_lap = lap;
        self
    }

    /// Evaluates tactical pit stop decision heuristic:
    /// Returns true when tire wear > 70% or chassis health < 60%. Spec 062.
    #[inline]
    pub fn should_pit<V: BotVehicle>(&self, car: &V) -> bool {
        car.max_tire_wear() > 0.70 || car.health() < 0.60
    }

    /// Computes deterministic driving controls (throttle, steer, brake, handbrake)
    /// for the given bot car navigating the track amidst other cars.
    /// Forward steering that turns the nose towards the target (negative for a positive heading error).
    /// After a failed watchdog turn the next one turns the other way round.
    fn turn_steer(&self, heading_error: f32) -> f32 {
        let side = if self.recovery_attempts >= 2 && self.recovery_attempts.is_multiple_of(2) { -1.0 } else { 1.0 };
        -heading_error.signum() * side
    }

    pub fn compute_controls<V: BotVehicle>(
        &mut self,
        car: &V,
        track: &Track,
        other_cars: &[&V],
        dt: f32,
    ) -> CarControls {
        // 0. Resolve active layout and composite spline caching (Spec 084)
        let active_layout_opt = self.decide_active_layout(track, other_cars);
        if let Some(ref target_layout) = active_layout_opt {
            if self.cached_layout_id.as_deref() != Some(target_layout.as_str())
                || self.cached_layout_spline.is_none()
            {
                if let Some(network) = &track.network {
                    let composite = network
                        .composite_spline_for_layout(target_layout)
                        .cloned()
                        .or_else(|| network.build_composite_spline_for_layout(target_layout));
                    if let Some(composite) = composite {
                        self.cached_layout_spline = Some(composite);
                        self.cached_layout_id = Some(target_layout.clone());
                        self.active_layout_id = Some(target_layout.clone());
                    }
                }
            }
        }

        let spline = self.cached_layout_spline.as_ref().unwrap_or(&track.spline);
        if spline.samples.is_empty() {
            return CarControls::default();
        }

        let car_pos = car.position();
        let car_speed = car.speed();
        let car_fwd = car.forward_vector();
        let car_right = car.right_vector();

        // 1. Project onto spline to find current track distance with continuity constraint
        let proj = if self.last_pos.is_some() {
            spline.project_point_continuity(car_pos, self.current_target_dist, 50.0)
        } else {
            spline.project_point(car_pos)
        };
        let curr_dist = proj.progress_distance;

        // A car that is stuck off its route on another branch follows that branch until the next route choice (spec
        // 088). Steering back to the route it left put a holjes_rx bot that ran wide at the split against the
        // joker's inside wall for three minutes. A bot that is still making progress steers back: on killarney_rx
        // it drifts onto the joker every lap and gets back. Stuck means a no-progress watchdog has fired.
        // The new route takes over on the next tick.
        if self.recovery_attempts > 0 && proj.distance_to_spline > proj.track_width * 0.5 + OFF_ROUTE_MARGIN_M {
            let network = track.network.as_ref();
            let own = network.zip(self.active_layout_id.as_deref()).and_then(|(n, id)| n.get_layout(id).map(|l| (n, l)));
            if let Some(id) = own.and_then(|(n, l)| layout_of_branch_under(n, l, car_pos, proj.distance_to_spline)) {
                self.active_layout_id = Some(id);
            }
        }
        self.human.begin_tick(car, spline, &proj, other_cars, dt);

        // Detect lap progression if not explicitly updated
        let lap_len = spline.total_length();
        if self.race_state.is_none() && self.last_progress_dist > 0.0 && lap_len > 10.0 {
            if curr_dist < lap_len * 0.25 && self.last_progress_dist > lap_len * 0.75 {
                self.current_lap += 1;
            }
        }
        self.last_progress_dist = curr_dist;

        if self.pit_cooldown > 0.0 {
            self.pit_cooldown = (self.pit_cooldown - dt).max(0.0);
        }

        // 2. Dynamic lookahead based on speed and profile
        let mut lookahead_dist = (10.0 + car_speed * (self.profile.lookahead_time + 0.10)).clamp(9.0, 45.0);
        // In a very tight turn (a kart hairpin) the line to a target this far ahead crosses the inside of the
        // bend, so the bot steered into the wall at the tip. Aim at most MAX_TARGET_TURN_RAD around the bend,
        // where the waypoints put a wall closer than WALL_CLEARANCE_M to the road (elsewhere the line keeps
        // running over run-off, and Tier 1 stays slower than the keyboard reference, spec 046).
        let near = spline.sample_at_distance(curr_dist);
        let close_wall = |on: bool, d: Option<f32>| on && d.is_some_and(|d| d < WALL_CLEARANCE_M);
        let walled = close_wall(near.left_wall, near.left_wall_distance) || close_wall(near.right_wall, near.right_wall_distance);
        let max_turn_cos = MAX_TARGET_TURN_RAD.cos();
        while walled && lookahead_dist > MIN_TIGHT_LOOKAHEAD_M {
            let ahead = spline.sample_at_distance((curr_dist + lookahead_dist) % spline.total_length());
            if ahead.tangent.dot(proj.tangent) >= max_turn_cos {
                break;
            }
            lookahead_dist -= 1.0;
        }
        let target_dist = (curr_dist + lookahead_dist) % spline.total_length();
        let prev_target_dist = self.current_target_dist;
        self.current_target_dist = target_dist;

        // Lap wrap detection for untracked/standalone execution
        let total_len = spline.total_length();
        if self.race_state.is_none() && prev_target_dist > total_len * 0.75 && target_dist < total_len * 0.25 {
            self.current_lap += 1;
            if self.was_in_joker {
                self.joker_laps_taken += 1;
                self.was_in_joker = false;
            }
        }
        if let Some(ref l_id) = self.active_layout_id {
            if l_id.to_lowercase().contains("joker") {
                self.was_in_joker = true;
            }
        }

        let target_sample = spline.sample_at_distance(target_dist);
        let mut target_point = target_sample.point;
        let mut human_offset = if self.human.is_active() {
            self.human.line_offset(spline, target_dist, target_sample.width, dt)
        } else {
            0.0
        };

        // Pit lane navigation and strategy (Spec 062/077/084)
        if let Some(lane) = &track.pit_lane {
            if lane.spline.total_length() > 1.0 {
                let entry_center = (lane.entry_gate.start + lane.entry_gate.end) * 0.5;
                let entry_proj = spline.project_point(entry_center);
                let dist_to_pit_entry_along_track = (entry_proj.progress_distance - curr_dist).rem_euclid(spline.total_length());

                // Divergence throat avoidance for cars not pitting: keep them away from the pit entrance gore (Spec 084)
                let pit_side = (entry_center - entry_proj.closest_point).dot(entry_proj.normal).signum();
                if !self.is_pitting && !self.is_in_pit_lane && dist_to_pit_entry_along_track < 100.0 {
                    let bias = (1.0 - (dist_to_pit_entry_along_track / 100.0)).clamp(0.0, 1.0);
                    // Dampen human line wandering toward the pit throat gore
                    if human_offset * pit_side > 0.0 {
                        human_offset *= 1.0 - bias;
                    }
                    // Enforce at least 3.0m of lateral repulsion away from the pit entrance gore
                    target_point -= target_sample.normal * (pit_side * bias * 3.5);
                }

                // Gated pit entry decision: Only enter if current_lap > 1, cooldown expired, should_pit is true, and near entry
                if !self.is_pitting && !self.is_in_pit_lane
                    && self.current_lap > 1
                    && self.pit_cooldown <= 0.0
                    && self.should_pit(car)
                    && dist_to_pit_entry_along_track < 100.0
                {
                    self.is_pitting = true;
                }

                let pit_proj = lane.spline.project_point(car_pos);
                let is_on_pit_ribbon = pit_proj.distance_to_spline < (lane.road_width * 0.5 + 0.8);

                // Entry gate transition: mark is_in_pit_lane = true upon crossing entry_gate or entering pit ribbon while pitting (Spec 084)
                let entered_gate = if let Some(last_p) = self.last_pos {
                    let seg = LineSegment::new(last_p, car_pos);
                    lane.entry_gate.intersect_segment(&seg).is_some()
                } else {
                    is_on_pit_ribbon
                };

                if self.is_pitting && !self.is_in_pit_lane {
                    if entered_gate || (is_on_pit_ribbon && !proj.is_on_track) || (self.last_pos.is_none() && is_on_pit_ribbon) {
                        self.is_in_pit_lane = true;
                    }
                }

                if self.is_in_pit_lane {
                    let pit_target_dist = (pit_proj.progress_distance + lookahead_dist).min(lane.spline.total_length());
                    let mut pit_target = lane.spline.sample_at_distance(pit_target_dist).point;

                    if self.is_pitting && !self.pit_serviced && !lane.pit_boxes.is_empty() {
                        let num_boxes = lane.pit_boxes.len();
                        let stall_idx = self.pit_stall_idx % num_boxes;
                        let pbox = &lane.pit_boxes[stall_idx];
                        let dist_to_box = (pbox.position - car_pos).length();
                        if dist_to_box < 15.0 {
                            pit_target = pbox.position;
                        }
                        if dist_to_box <= pbox.stop_radius || pbox.contains_point(car_pos) {
                            if car_speed < 1.0 {
                                self.stall_stop_timer += dt;
                            }
                        }
                        if self.stall_stop_timer >= 2.0 || (car.max_tire_wear() < 0.10 && car.health() >= 0.65) {
                            self.pit_serviced = true;
                        }
                    }

                    target_point = pit_target;

                    let exited_gate = if let Some(last_p) = self.last_pos {
                        let seg = LineSegment::new(last_p, car_pos);
                        lane.exit_gate.intersect_segment(&seg).is_some()
                    } else {
                        false
                    };

                    if exited_gate || pit_proj.progress_distance >= lane.spline.total_length() - 5.0 {
                        self.is_in_pit_lane = false;
                        self.is_pitting = false;
                        self.pit_serviced = false;
                        self.stall_stop_timer = 0.0;
                        self.pit_cooldown = 30.0;
                    }
                } else if self.is_pitting && dist_to_pit_entry_along_track < 100.0 {
                    let blend = (1.0 - (dist_to_pit_entry_along_track / 100.0)).clamp(0.0, 1.0);
                    target_point = target_point.lerp(entry_center, blend * 0.95);
                }
            }
        }

        if !self.is_in_pit_lane {
            target_point += target_sample.normal * human_offset;
        }

        if !self.is_in_pit_lane {
            // Keep the straight line to the target off close walls. Around a bend it passes inside the target
            // (which already sits on the inside of the racing line), and on a kart circuit the wall is 0.3-0.6 m
            // from the road edge, so bots scraped the inner wall and stopped. It only acts within WALL_CLEARANCE_M
            // of a wall, so wide run-off keeps the line. Walls further than WALL_CLEARANCE_M from the road count
            // too: on kart_pine_grove (wall 1.9 m out) a Legend cut the curb and hit the inner wall every lap.
            let mid = spline.sample_at_distance((curr_dist + lookahead_dist * 0.5) % spline.total_length());
            let chord_lat = ((car_pos + target_point) * 0.5 - mid.point).dot(mid.normal); // > 0: left of the centre
            let (wall_on, wall_dist) = if chord_lat > 0.0 {
                (mid.left_wall, mid.left_wall_distance)
            } else {
                (mid.right_wall, mid.right_wall_distance)
            };
            if let (true, Some(d)) = (wall_on, wall_dist) {
                let excess = chord_lat.abs() - (mid.width * 0.5 + d - WALL_CLEARANCE_M);
                if excess > 0.0 {
                    // Moving the target moves the middle of the line by half as much.
                    target_point -= mid.normal * (chord_lat.signum() * excess * 2.0);
                }
            }
            // The same for the car itself, at close walls only: a bot drifting towards a close wall at a shallow
            // angle kept a small heading error and slid along the wall.
            let here = spline.sample_at_distance(curr_dist);
            let car_lat = (car_pos - here.point).dot(here.normal);
            let (wall_on, wall_dist) = if car_lat > 0.0 {
                (here.left_wall, here.left_wall_distance)
            } else {
                (here.right_wall, here.right_wall_distance)
            };
            if let (true, Some(d)) = (wall_on, wall_dist.filter(|d| *d < WALL_CLEARANCE_M)) {
                let excess = car_lat.abs() - (here.width * 0.5 + d - WALL_CLEARANCE_M);
                if excess > 0.0 {
                    target_point -= target_sample.normal * (car_lat.signum() * excess * CAR_WALL_PUSH);
                }
            }
        }

        // 3. Check heading error to target
        let to_target = target_point - car_pos;
        let desired_heading = to_target.y.atan2(to_target.x);
        let heading_error = normalize_angle(desired_heading - car.angle());

        // Stuck / Wall-pin detection & three-point turn recovery (tdrace-le75)
        self.last_pos = Some(car_pos);
        self.total_distance_travelled += car_speed * dt;
        let racing = self.total_distance_travelled > 15.0;
        if !racing {
            self.stuck_timer = 0.0;
            self.turn = None;
        }

        let car_alignment = car_fwd.dot(proj.tangent);
        // Spec 046: a spun bot can stop nose-first against a wall while still on the track.
        let is_stuck_situation = (!proj.is_on_track && car_speed < 1.2)
            || (car_alignment < -0.35 && car_speed < 1.5)
            || (car_alignment < 0.5 && car_speed < 1.2);
        // Only a car that hardly moves is stuck: a slow car on low-grip run-off is still pulling away.
        if self.turn.is_none() && racing && car_speed < STUCK_SPEED && is_stuck_situation {
            self.stuck_timer += dt;
            if self.stuck_timer > 0.8 {
                self.stuck_timer = 0.0;
                self.turn = Some(ThreePointTurn::new(true, self.turn_steer(heading_error)));
            }
        } else {
            self.stuck_timer = (self.stuck_timer - dt * 2.0).max(0.0);
        }

        // Spec 046: a spun bot can also circle slowly against a wall without ever stopping. Driving back
        // towards the road from the run-off is progress too: reversing then threw the bot off again.
        let lap_len = spline.total_length();
        let gained = (curr_dist - self.progress_mark).rem_euclid(lap_len);
        let lateral = proj.lateral_offset.abs();
        if !racing || (gained > 5.0 && gained < 0.5 * lap_len) || lateral < self.progress_lateral - 2.0 {
            self.progress_mark = curr_dist;
            self.progress_lateral = lateral;
            self.no_progress_timer = 0.0;
            if gained > 5.0 && gained < 0.5 * lap_len {
                self.recovery_attempts = 0;
            }
        } else if self.turn.is_none() {
            self.no_progress_timer += dt;
            if self.no_progress_timer > 3.0 {
                self.no_progress_timer = 0.0;
                self.progress_mark = curr_dist;
                self.progress_lateral = lateral;
                self.recovery_attempts += 1;
                self.turn = Some(ThreePointTurn::new(true, self.turn_steer(heading_error)));
            }
        }

        // A slow bot pointing away from its target (after a spin, or back on the road the wrong way round)
        // turns round instead of circling at full lock wider than the road.
        if self.turn.is_none() && racing && car_speed < TURN_START_SPEED && heading_error.abs() > TURN_START_RAD {
            self.turn = Some(ThreePointTurn::new(false, self.turn_steer(heading_error)));
        }
        if let Some(mut turn) = self.turn {
            turn.leg_time += dt;
            turn.total_time += dt;
            turn.blocked_time = if car_speed < STUCK_SPEED && turn.leg_time > 0.5 { turn.blocked_time + dt } else { 0.0 };
            if (!turn.reversing && heading_error.abs() < TURN_DONE_RAD) || turn.total_time > TURN_MAX_S {
                self.turn = None;
            } else {
                // How far a point lies past the edge of the drivable ground: the road, plus the run-off up to a
                // close wall (> 0: past it).
                let past_edge = |p: Vec2| {
                    let pp = spline.project_point_continuity(p, curr_dist, 30.0);
                    let at = spline.sample_at_distance(pp.progress_distance);
                    let (wall, wall_dist) = if pp.lateral_offset > 0.0 {
                        (at.left_wall, at.left_wall_distance)
                    } else {
                        (at.right_wall, at.right_wall_distance)
                    };
                    let room = match (wall, wall_dist) {
                        (true, Some(d)) => (d - TURN_WALL_GAP_M).max(0.0),
                        _ => TURN_RUNOFF_M,
                    };
                    pp.lateral_offset.abs() - (at.width * 0.5 + room)
                };
                let hull = car.hull();
                let probe = if turn.reversing {
                    car_pos - car_fwd * (hull.rear + TURN_PROBE_M)
                } else {
                    car_pos + car_fwd * (hull.front + TURN_PROBE_M)
                };
                // Off the drivable ground, only moving farther out of it counts as reaching the edge. A leg turns the
                // car a little before it can end there, or a car facing the edge only ever reversed.
                let at_edge = turn.leg_time > TURN_MIN_LEG_S && past_edge(probe) > past_edge(car_pos).max(0.0);
                let leg_done = at_edge
                    || turn.blocked_time > 0.4
                    || turn.leg_time > TURN_LEG_S
                    || (turn.reversing && turn.leg_time > 0.5 && heading_error.abs() < TURN_REVERSE_DONE_RAD);
                if leg_done {
                    turn = ThreePointTurn { reversing: !turn.reversing, leg_time: 0.0, blocked_time: 0.0, ..turn };
                }
                self.turn = Some(turn);
                self.human.reset_recovery_line();
                return CarControls {
                    throttle: TURN_THROTTLE,
                    steer: if turn.reversing { -turn.steer } else { turn.steer },
                    brake: 0.0,
                    handbrake: false,
                    reverse: turn.reversing,
                };
            }
        }

        // 3. Physically Exact Autonomous Racing Braking Envelope: v_allowable = sqrt(v_apex^2 + 2*a_brake*d)
        let max_braking_lookahead = (lookahead_dist + (car_speed * car_speed) / 7.5).clamp(35.0, 220.0);
        let a_brake = 6.0 * self.profile.brake_margin; // safe braking deceleration m/s²
        let mu = car.planning_grip();
        let g = 9.81;

        let mut target_speed = car.top_speed_mps();
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
                let m = self.human.corner_modifiers(scan_dist, spline.total_length());
                let mut v_apex = (effective_grip * g * local_radius).sqrt() * self.profile.speed_factor * m.speed_mult;
                let mut a_scan = a_brake;
                // Spec 046 mistakes are set against the car's real grip, not the controller's
                // conservative `mu`.
                if m.over_limit > 0.0 {
                    let limit_grip = car.grip() + bank_rad.tan().clamp(0.0, 0.75);
                    v_apex = v_apex.max((limit_grip * g * local_radius).sqrt() * m.over_limit);
                }
                if m.brake_decel_mult > 0.0 {
                    a_scan = a_scan.max(car.grip() * g * m.brake_decel_mult);
                }
                // Maximum entry speed from distance d: v = sqrt(v_apex^2 + 2 * a * d)
                let v_allowable = (v_apex * v_apex + 2.0 * a_scan * HumanDriver::shifted_distance(dist_ahead, m.brake_shift_m)).sqrt();
                if v_allowable < target_speed {
                    target_speed = v_allowable;
                }
            }
        }

        target_speed = target_speed.clamp(7.0, car.top_speed_mps());

        // Pit lane speed governing and pit box stopping (Spec 062/077/084)
        if let Some(lane) = &track.pit_lane {
            if self.is_in_pit_lane {
                target_speed = target_speed.min(lane.speed_limit);
                if self.is_pitting && !self.pit_serviced && !lane.pit_boxes.is_empty() {
                    let num_boxes = lane.pit_boxes.len();
                    let stall_idx = self.pit_stall_idx % num_boxes;
                    let pbox = &lane.pit_boxes[stall_idx];
                    let dist_to_box = (pbox.position - car_pos).length();
                    if dist_to_box < 15.0 {
                        let stop_speed = (dist_to_box / 15.0).clamp(0.0, 1.0) * lane.speed_limit;
                        target_speed = target_speed.min(stop_speed);
                        if dist_to_box <= pbox.stop_radius || pbox.contains_point(car_pos) {
                            target_speed = 0.0;
                        }
                    }
                }
            } else if self.is_pitting {
                let entry_center = (lane.entry_gate.start + lane.entry_gate.end) * 0.5;
                let entry_proj = spline.project_point(entry_center);
                let dist_to_pit_entry_along_track = (entry_proj.progress_distance - curr_dist).rem_euclid(spline.total_length());
                if dist_to_pit_entry_along_track < 30.0 {
                    target_speed = target_speed.min(lane.speed_limit * 1.2);
                }
            }
        }

        // 4. Multi-car Collision Avoidance & Overtaking
        let mut throttle_limit = 1.0f32;
        let mut extra_brake = 0.0f32;
        let mut avoidance_steer = 0.0f32;

        // Gradually decay previous lateral bias
        self.avoidance_lateral_bias *= 0.95;
        let mut had_pass_target = false;

        // 4b. Opponent cars avoidance
        for &opp in other_cars {
            let to_opp = opp.position() - car_pos;
            let dist = to_opp.length();

            if dist < self.profile.avoidance_distance && dist > 0.05 {
                let opp_fwd_proj = to_opp.dot(car_fwd);
                let opp_lat_proj = to_opp.dot(car_right);

                // Opponent is in front of us
                if opp_fwd_proj > 0.5 {
                    let rel_speed = car_speed - opp.speed();

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
                        let evade_dir = self.human.pass_side(if opp_lat_proj >= 0.0 { 1.0 } else { -1.0 }, opp_lat_proj);
                        let evade_strength = (1.0 - (opp_fwd_proj / 15.0)) * self.profile.aggression * self.human.pass_commit();
                        had_pass_target = true;
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

        self.human.update_pass_timer(had_pass_target, dt);

        // Apply avoidance lateral offset to target point
        if avoidance_steer.abs() > 0.05 {
            target_point += target_sample.normal * (avoidance_steer * 2.5);
        }

        // 5. Steering PID Controller with Error Derivative
        let to_target = target_point - car_pos;
        let desired_heading = to_target.y.atan2(to_target.x);
        let heading_error = normalize_angle(desired_heading - car.angle());

        let d_error = if self.has_prev_heading && dt > 1e-4 {
            normalize_angle(heading_error - self.prev_heading_error) / dt
        } else {
            self.has_prev_heading = true;
            0.0
        };
        self.prev_heading_error = heading_error;

        let steer = -(heading_error * self.profile.steering_kp + d_error * self.profile.steering_kd);
        let steer_cmd = self.human.shape_steer(steer.clamp(-1.0, 1.0), dt);

        // Heading alignment and corner steering throttle limit (prevents spinning when loaded laterally)
        if car_speed > 4.0 {
            if heading_error.abs() > 0.45 {
                let align_factor = (1.0f32 - (heading_error.abs() - 0.45) / 0.85).clamp(0.2, 1.0);
                throttle_limit *= align_factor;
            }
            let steer_traction_limit = if self.human.is_active() {
                if steer_cmd.abs() > 0.20 {
                    (1.0f32 - (steer_cmd.abs() - 0.20) * 0.60).clamp(0.35, 1.0)
                } else {
                    1.0
                }
            } else {
                (1.0f32 - steer_cmd.abs() * 0.55).clamp(0.3, 1.0)
            };
            throttle_limit *= steer_traction_limit;
        }
        // Spec 046: turning round after a spin. Full throttle at full lock only spins a
        // rear-drive car on the spot.
        let turning_round = car_speed <= 6.0 && heading_error.abs() > 0.8 && self.total_distance_travelled > 15.0;
        if heading_error.abs() > 1.15 && car_speed > 6.0 {
            extra_brake = extra_brake.max(0.6);
        }

        // 6. Longitudinal Throttle & Brake Management
        let speed_err = target_speed - car_speed;
        let (throttle_cmd, brake_cmd) = if self.is_in_pit_lane && target_speed <= 0.1 {
            (0.0, 1.0)
        } else if extra_brake > 0.2 {
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

        let throttle_cmd = if turning_round { throttle_cmd.min(0.5) } else { throttle_cmd };
        let (throttle_cmd, brake_cmd, handbrake_cmd) = self.human.shape_pedals(throttle_cmd, brake_cmd);

        CarControls {
            throttle: throttle_cmd,
            steer: steer_cmd,
            brake: brake_cmd,
            handbrake: handbrake_cmd,
            reverse: false,
        }
    }
}
