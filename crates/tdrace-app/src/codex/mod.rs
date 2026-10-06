//! TDRace Codex data export (specs/087_tdrace_codex_unified_game_encyclopedia_and_technical_reference_portal.md).
//!
//! Serialises the game's own data for the Codex web portal (`portals/codex`). Every number comes from the
//! same catalogue, presets and functions the game uses; nothing is re-computed here.
//! The `export_codex` binary writes the files; `tests/codex_export_tests.rs` checks that the committed copy
//! in `portals/shared/data/codex/` is fresh.

use std::path::Path;

use serde::Serialize;
use tdrace_core::catalog;
use tdrace_core::physics::car::{
    Car, ImpactZone, CHASSIS_DAMAGE_CAPACITY_J, DAMAGE_ENERGY_DEADZONE_J, ENGINE_DAMAGE_CAPACITY_J,
    FIELD_REPAIR_CHASSIS_CAP, FIELD_REPAIR_ENGINE_CAP, FIELD_REPAIR_SUSPENSION_CAP, KERB_BOTTOM_OUT_CAPACITY_J,
    KERB_BOTTOM_OUT_SPEED_MPS, LANDING_CAPACITY_J, LANDING_SPEED_LIMIT_MPS, PUSHROD_COLLAPSE_DRAG_MULTIPLIER,
    PUSHROD_COLLAPSE_HEALTH, SUSPENSION_DAMAGE_CAPACITY_J,
};
use tdrace_core::physics::config::{
    AssistProfile, CarConfig, ChassisSkeleton, DifferentialType, EnginePlacement, SuspensionArchetype,
    SuspensionConfig,
};
use tdrace_core::physics::surface::{CompoundId, SurfaceType};
use tdrace_core::physics::tire::{normalized_grip_curve, TireCompoundConfig, WheelAssembly};
use tdrace_core::track::{Track, TrackKind};

use cabinet::input::filter::{DigitalInputConfig, DigitalInputFilter, SteeringProfile};
use cabinet::input::gamepad::GamepadConfig;
use cabinet::input::mapping::{ArcadeAction, InputMap, InputSource};
use cabinet::net::INTERP_DELAY_SEC;
use race_kit::world::PIT_STOP_REPAIR_AMOUNT;
use race_ui::camera::{CameraConfig, MAX_CAR_SCREEN_OFFSET_FRAC};
use tdrace_core::profile::AcademyLessonDef;

use crate::ai::{DriverCharacter, DriverQuality, DriverTier};
use crate::audio::EngineSoundType;
use crate::catalog::{get_tier_name, RealCarModel, ALL_REAL_CARS, CLASSIC_ARCADE_CARS};
use crate::profile::repair::{
    ItemizedRepairInvoice, REPAIR_PURSE_CAP_SHARE, SAFETY_NET_CREDIT_LIMIT, SAFETY_NET_HEALTH,
};
use crate::profile::ModuleCareerProgress;
use crate::module::{
    autocross::AutocrossGameModule, classic::ClassicGameModule, extreme_offroad::ExtremeOffRoadModule,
    gt::GtWorldChallengeModule, kart::KartGameModule, nascar::NascarGameModule, rally::RallyGameModule,
    GameModule, VehicleVisualType,
};
use crate::series::{manager::EMBEDDED_PRESETS, PointSystem, SeriesDefinition};
use crate::ui::menu::CarChoice;

/// Version of the JSON layout. Bump it when a field changes meaning or is removed.
pub const SCHEMA_VERSION: u32 = 1;

/// Repository path of the committed export.
pub const DEFAULT_OUT_DIR: &str = "portals/shared/data/codex";

/// Modules of the Steam v1 launch. GT, NASCAR and Extreme Off-Road are DLC.
pub const LAUNCH_MODULES: [&str; 4] = ["classic", "kart", "autocross", "rally"];

/// Which content an export holds. The Vault (archived content) is never exported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Every playable module.
    All,
    /// The Steam v1 launch modules only.
    Launch,
}

impl Scope {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "all" => Some(Self::All),
            "launch" => Some(Self::Launch),
            _ => None,
        }
    }

    pub fn includes_module(self, module_id: &str) -> bool {
        match self {
            Self::All => module_id != "vault",
            Self::Launch => LAUNCH_MODULES.contains(&module_id),
        }
    }
}

/// Every exported file has this envelope.
#[derive(Serialize)]
struct Envelope<T: Serialize> {
    schema_version: u32,
    scope: Scope,
    items: Vec<T>,
}

#[derive(Serialize)]
pub struct CodexModule {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub launch: bool,
    pub cars: usize,
    pub circuits: usize,
}

#[derive(Serialize)]
pub struct CodexStats {
    pub speed: f32,
    pub acceleration: f32,
    pub grip: f32,
    pub agility: f32,
    pub braking: f32,
    pub aero: f32,
}

#[derive(Serialize)]
pub struct CodexImages {
    pub reference: Option<String>,
    pub lateral: Option<String>,
    pub thumb: Option<String>,
    pub topdown: Option<String>,
}

#[derive(Serialize)]
pub struct CodexCar {
    pub id: &'static str,
    pub name: &'static str,
    pub manufacturer: &'static str,
    pub year: u16,
    pub module: &'static str,
    pub category: &'static str,
    pub category_name: &'static str,
    pub tier: u8,
    pub tier_name: &'static str,
    pub bhp: u16,
    pub torque_nm: u16,
    pub weight_kg: u16,
    pub top_speed_kmh: u16,
    pub accel_0_100: f32,
    pub drivetrain: &'static str,
    pub engine_desc: &'static str,
    pub aero_downforce: &'static str,
    pub brakes_desc: &'static str,
    pub history_bio: &'static str,
    pub stats: CodexStats,
    pub visual_type: VehicleVisualType,
    pub base_car: CarChoice,
    pub sound: EngineSoundType,
    pub images: CodexImages,
    /// The physics configuration the game builds for this car (`RealCarModel::to_car_config`).
    pub physics: CarConfig,
}

#[derive(Serialize)]
pub struct CodexSurfaceShare {
    pub surface: SurfaceType,
    pub share_pct: f32,
}

#[derive(Serialize)]
pub struct CodexCircuit {
    pub id: &'static str,
    pub name: &'static str,
    pub module: &'static str,
    pub category: &'static str,
    pub tag: &'static str,
    pub category_label: &'static str,
    pub description: &'static str,
    pub kind: &'static str,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub scale: String,
    pub wikipedia_url: Option<String>,
    pub osm_url: Option<String>,
    pub is_inspired: bool,
    pub default_laps: u32,
    pub length_m: f32,
    pub min_width_m: f32,
    pub max_width_m: f32,
    pub surfaces: Vec<CodexSurfaceShare>,
    pub has_jumps: bool,
    pub has_joker: bool,
    pub has_pit_lane: bool,
    pub has_junctions: bool,
    pub image: Option<String>,
}

#[derive(Serialize)]
pub struct CodexSurface {
    pub id: SurfaceType,
    pub name: &'static str,
    pub friction: f32,
    pub rolling_resistance: f32,
    pub drag: f32,
    pub tire_smoke: bool,
    pub debris: bool,
    pub water_splash: bool,
    pub loose: bool,
    pub rigid_pavement: bool,
    pub on_track_hazard: bool,
    pub valid_off_track: bool,
}

/// Sample points `(x, y)` of a curve.
pub type Curve = Vec<[f32; 2]>;

#[derive(Serialize)]
pub struct CodexWheel {
    pub position: &'static str,
    pub radius_m: f32,
    pub width_m: f32,
    pub inertia_kgm2: f32,
    pub compound: CompoundId,
}

#[derive(Serialize)]
pub struct CodexHull {
    pub front_extent_m: f32,
    pub rear_extent_m: f32,
    pub half_width_m: f32,
}

/// Shape of a platform's front tire model (Spec 043). The grip level varies per car.
#[derive(Serialize)]
pub struct CodexTireShape {
    pub peak_slip_angle_deg: f32,
    pub rear_peak_slip_angle_deg: f32,
    pub peak_slip_ratio: f32,
    pub slide_grip: f32,
    pub falloff: f32,
    pub load_sensitivity: f32,
    pub power_slide: f32,
    /// `normalized_grip_curve` against normalized slip (1.0 = peak slip).
    pub curve: Curve,
}

/// A base car preset (`CarChoice`): the chassis, wheels, suspension and drivetrain layout that every
/// car built on it shares. `to_car_config` changes mass, power, grip and aero per car, not these.
#[derive(Serialize)]
pub struct CodexPlatform {
    pub id: CarChoice,
    pub title: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub modules: Vec<&'static str>,
    pub cars: Vec<&'static str>,
    pub wheelbase_m: f32,
    pub track_width_m: f32,
    pub cg_to_front_m: f32,
    pub cg_to_rear_m: f32,
    pub cg_height_m: f32,
    pub total_length_m: f32,
    pub skeleton: ChassisSkeleton,
    pub hull: CodexHull,
    pub wheels: Vec<CodexWheel>,
    pub suspension: SuspensionConfig,
    pub front_differential: DifferentialType,
    pub rear_differential: DifferentialType,
    pub engine_placement: EnginePlacement,
    pub tire: CodexTireShape,
}

#[derive(Serialize)]
pub struct CodexSuspensionArchetype {
    pub id: SuspensionArchetype,
    pub robustness_factor: f32,
    pub part_cost_multiplier: f32,
    /// `SuspensionConfig::for_archetype`: the factory reference setup.
    pub factory: SuspensionConfig,
    pub platforms_front: Vec<CarChoice>,
    pub platforms_rear: Vec<CarChoice>,
}

#[derive(Serialize)]
pub struct CodexAffinity {
    pub surface: SurfaceType,
    pub multiplier: f32,
}

#[derive(Serialize)]
pub struct CodexCompound {
    pub id: CompoundId,
    pub name: &'static str,
    pub badge: &'static str,
    pub accent_rgba: [f32; 4],
    pub wear_rate: f32,
    pub optimal_temp_c: [f32; 2],
    pub overheat_temp_c: f32,
    pub affinity: Vec<CodexAffinity>,
    /// `WheelAssembly::thermal_grip_multiplier` against tread temperature (°C).
    pub thermal_curve: Curve,
    pub platforms: Vec<CarChoice>,
}

#[derive(Serialize)]
pub struct CodexEnginePlacement {
    pub id: EnginePlacement,
    pub repair_cost_multiplier: f32,
    pub platforms: Vec<CarChoice>,
}

#[derive(Serialize)]
pub struct CodexDamageWeights {
    pub placement: EnginePlacement,
    pub chassis: f32,
    pub engine: f32,
    /// [FL, FR, RL, RR]
    pub suspension: [f32; 4],
}

#[derive(Serialize)]
pub struct CodexImpactZone {
    pub id: ImpactZone,
    pub weights: Vec<CodexDamageWeights>,
}

#[derive(Serialize)]
pub struct CodexInvoiceExample {
    pub tier: u32,
    pub base_purse: u64,
    pub health: f32,
    pub placement: EnginePlacement,
    pub archetype: SuspensionArchetype,
    pub invoice: ItemizedRepairInvoice,
}

/// The damage and repair model (Spec 078): one item.
#[derive(Serialize)]
pub struct CodexDamageModel {
    pub id: &'static str,
    pub energy_deadzone_j: f32,
    pub chassis_capacity_j: f32,
    pub engine_capacity_j: f32,
    pub suspension_capacity_j: f32,
    pub kerb_bottom_out_speed_mps: f32,
    pub kerb_bottom_out_capacity_j: f32,
    pub landing_speed_limit_mps: f32,
    pub landing_capacity_j: f32,
    pub pushrod_collapse_health: f32,
    pub pushrod_collapse_drag_multiplier: f32,
    pub zones: Vec<CodexImpactZone>,
    /// `Car::available_engine_power_ratio` against engine health.
    pub engine_power_curve: Curve,
    /// `Car::steering_pull_bias` against front-left minus front-right suspension health.
    pub steering_pull_curve: Curve,
    /// `WheelAssembly::effective_friction` (at optimal temperature, relative) against tread wear.
    pub tire_wear_curve: Curve,
    pub field_repair_caps: FieldRepairCaps,
    pub pit_stop_repair_amount: f32,
    pub purse_cap_share: f64,
    pub safety_net_credit_limit: u64,
    pub safety_net_health: f32,
    pub invoice_examples: Vec<CodexInvoiceExample>,
}

#[derive(Serialize)]
pub struct FieldRepairCaps {
    pub chassis: f32,
    pub engine: f32,
    pub suspension: f32,
}

/// The playable modules in Codex order: launch modules first.
fn modules() -> [&'static dyn GameModule; 7] {
    [
        &ClassicGameModule,
        &KartGameModule,
        &AutocrossGameModule,
        &RallyGameModule,
        &GtWorldChallengeModule,
        &NascarGameModule,
        &ExtremeOffRoadModule,
    ]
}

/// Rounds to one decimal so the JSON does not change on float noise.
fn round1(x: f32) -> f32 {
    (x * 10.0).round() / 10.0
}

/// Web path of an asset under `assets/textures/`, or `None` when the file does not exist.
fn texture(repo_root: &Path, rel: String) -> Option<String> {
    repo_root.join("assets/textures").join(&rel).exists().then(|| format!("/textures/{}", rel))
}

fn car(model: &'static RealCarModel, repo_root: &Path) -> CodexCar {
    let (m, id) = (model.module_id, model.id);
    let s = model.stats;
    CodexCar {
        id,
        name: model.name,
        manufacturer: model.manufacturer,
        year: model.year,
        module: m,
        category: model.category().id(),
        category_name: model.category_name,
        tier: model.tier,
        tier_name: get_tier_name(m, model.tier),
        bhp: model.bhp,
        torque_nm: model.torque_nm,
        weight_kg: model.weight_kg,
        top_speed_kmh: model.top_speed_kmh,
        accel_0_100: model.accel_0_100,
        drivetrain: model.drivetrain,
        engine_desc: model.engine_desc,
        aero_downforce: model.aero_downforce,
        brakes_desc: model.brakes_desc,
        history_bio: model.history_bio,
        stats: CodexStats { speed: s.0, acceleration: s.1, grip: s.2, agility: s.3, braking: s.4, aero: s.5 },
        visual_type: model.visual_type,
        base_car: model.base_car_choice,
        sound: model.sound_type(),
        images: CodexImages {
            reference: None,
            lateral: texture(repo_root, format!("vehicles/laterals/{m}/{id}.png")),
            thumb: texture(repo_root, format!("vehicles/laterals/{m}/{id}_thumb.png")),
            topdown: texture(repo_root, format!("vehicles/topdown/{m}/{id}.png")),
        },
        physics: model.to_car_config(),
    }
}

fn circuit(entry: &'static catalog::EmbeddedCircuit, track: &Track, repo_root: &Path) -> CodexCircuit {
    let (min_w, max_w) = track.width_range();
    let mut surfaces: Vec<CodexSurfaceShare> = track
        .surface_breakdown()
        .into_iter()
        .map(|(surface, pct)| CodexSurfaceShare { surface, share_pct: round1(pct) })
        .collect();
    // surface_breakdown() comes from a HashMap: break percentage ties by name so the output is stable.
    surfaces.sort_by(|a, b| {
        b.share_pct.total_cmp(&a.share_pct).then_with(|| a.surface.name().cmp(b.surface.name()))
    });
    CodexCircuit {
        id: entry.id,
        name: entry.name,
        module: entry.module,
        category: track.car_category.id(),
        tag: entry.tag,
        category_label: entry.category_label,
        description: entry.description,
        kind: match track.kind {
            TrackKind::Circuit => "circuit",
            _ => "arena",
        },
        country_code: track.country_code.clone(),
        country_name: track.country_name.clone(),
        scale: track.scale().to_string(),
        wikipedia_url: track.wikipedia_url.clone(),
        osm_url: track.osm_url.clone(),
        is_inspired: track.is_inspired,
        default_laps: entry.default_laps,
        length_m: round1(track.total_length_m()),
        min_width_m: round1(min_w),
        max_width_m: round1(max_w),
        surfaces,
        has_jumps: !track.geometry.jump_ramps.is_empty(),
        has_joker: track.checkpoints.iter().any(|c| c.is_joker),
        has_pit_lane: track.pit_lane.is_some(),
        has_junctions: track.network.as_ref().is_some_and(|n| !n.junctions.is_empty()),
        image: texture(repo_root, format!("circuits/{}/{}.svg", entry.module, entry.id)),
    }
}

fn surface(s: SurfaceType) -> CodexSurface {
    CodexSurface {
        id: s,
        name: s.name(),
        friction: s.friction_coefficient(),
        rolling_resistance: s.rolling_resistance_multiplier(),
        drag: s.surface_drag_multiplier(),
        tire_smoke: s.produces_tire_smoke(),
        debris: s.produces_debris_particles(),
        water_splash: s.produces_water_splash(),
        loose: s.is_loose_deformable(),
        rigid_pavement: s.is_rigid_pavement(),
        on_track_hazard: s.is_on_track_hazard(),
        valid_off_track: s.is_valid_off_track(),
    }
}

/// Samples `f` at `steps + 1` evenly spaced points from `from` to `to`, rounded to 4 decimals.
fn sample(from: f32, to: f32, steps: usize, mut f: impl FnMut(f32) -> f32) -> Curve {
    (0..=steps)
        .map(|i| {
            let x = from + (to - from) * i as f32 / steps as f32;
            [round4(x), round4(f(x))]
        })
        .collect()
}

fn round4(x: f32) -> f32 {
    (x * 10_000.0).round() / 10_000.0
}

const WHEEL_POSITIONS: [&str; 4] = ["FL", "FR", "RL", "RR"];

fn platform(id: CarChoice, models: &[&'static RealCarModel]) -> CodexPlatform {
    let cfg = models[0].to_car_config();
    let (front, rear, half) = cfg.chassis.to_body_hull(cfg.cg_to_front, cfg.cg_to_rear);
    let rear_tire = cfg.rear_axle.apply(&cfg.tire);
    let mut modules: Vec<&'static str> = Vec::new();
    for m in models {
        if !modules.contains(&m.module_id) {
            modules.push(m.module_id);
        }
    }
    CodexPlatform {
        id,
        title: id.title(),
        tag: id.tag(),
        description: id.description(),
        modules,
        cars: models.iter().map(|m| m.id).collect(),
        wheelbase_m: cfg.wheelbase,
        track_width_m: cfg.track_width,
        cg_to_front_m: cfg.cg_to_front,
        cg_to_rear_m: cfg.cg_to_rear,
        cg_height_m: cfg.cg_height,
        total_length_m: cfg.chassis.total_length(cfg.wheelbase),
        skeleton: cfg.chassis,
        hull: CodexHull { front_extent_m: front, rear_extent_m: rear, half_width_m: half },
        wheels: cfg
            .wheels
            .iter()
            .zip(WHEEL_POSITIONS)
            .map(|(w, position)| CodexWheel {
                position,
                radius_m: w.tire_radius,
                width_m: w.tire_width,
                inertia_kgm2: w.rotational_inertia,
                compound: w.compound.id,
            })
            .collect(),
        suspension: cfg.suspension,
        front_differential: cfg.front_differential,
        rear_differential: cfg.rear_differential,
        engine_placement: cfg.engine_placement,
        tire: CodexTireShape {
            peak_slip_angle_deg: cfg.tire.peak_slip_angle_deg,
            rear_peak_slip_angle_deg: rear_tire.peak_slip_angle_deg,
            peak_slip_ratio: cfg.tire.peak_slip_ratio,
            slide_grip: cfg.tire.slide_grip,
            falloff: cfg.tire.falloff,
            load_sensitivity: cfg.tire.load_sensitivity,
            power_slide: cfg.tire.power_slide,
            curve: sample(0.0, 4.0, 80, |s| normalized_grip_curve(s, &cfg.tire)),
        },
    }
}

/// Groups the exported cars by base preset, in `CarChoice::ALL` order.
fn platforms(models: &[&'static RealCarModel]) -> Vec<CodexPlatform> {
    CarChoice::ALL
        .into_iter()
        .filter_map(|id| {
            let group: Vec<&'static RealCarModel> =
                models.iter().copied().filter(|m| m.base_car_choice == id).collect();
            (!group.is_empty()).then(|| platform(id, &group))
        })
        .collect()
}

fn suspension_archetypes(platforms: &[CodexPlatform]) -> Vec<CodexSuspensionArchetype> {
    SuspensionArchetype::ALL
        .into_iter()
        .map(|a| CodexSuspensionArchetype {
            id: a,
            robustness_factor: a.robustness_factor(),
            part_cost_multiplier: a.part_cost_multiplier(),
            factory: SuspensionConfig::for_archetype(a),
            platforms_front: platforms.iter().filter(|p| p.suspension.front.archetype == a).map(|p| p.id).collect(),
            platforms_rear: platforms.iter().filter(|p| p.suspension.rear.archetype == a).map(|p| p.id).collect(),
        })
        .collect()
}

fn compounds(platforms: &[CodexPlatform]) -> Vec<CodexCompound> {
    CompoundId::ALL
        .into_iter()
        .map(|id| {
            let c = TireCompoundConfig::from_id(id);
            let mut wheel = WheelAssembly::default();
            wheel.config.compound = c;
            CodexCompound {
                id,
                name: id.name(),
                badge: id.badge_code(),
                accent_rgba: id.accent_rgba(),
                wear_rate: c.wear_rate,
                optimal_temp_c: [c.optimal_temp_range.0, c.optimal_temp_range.1],
                overheat_temp_c: c.overheat_temp,
                affinity: SurfaceType::ALL
                    .into_iter()
                    .map(|surface| CodexAffinity { surface, multiplier: c.surface_affinity.get(surface) })
                    .collect(),
                thermal_curve: sample(0.0, 160.0, 32, |t| {
                    wheel.temperature = t;
                    wheel.thermal_grip_multiplier()
                }),
                platforms: platforms
                    .iter()
                    .filter(|p| p.wheels.iter().any(|w| w.compound == id))
                    .map(|p| p.id)
                    .collect(),
            }
        })
        .collect()
}

fn engine_placements(platforms: &[CodexPlatform]) -> Vec<CodexEnginePlacement> {
    EnginePlacement::ALL
        .into_iter()
        .map(|id| CodexEnginePlacement {
            id,
            repair_cost_multiplier: id.repair_cost_multiplier(),
            platforms: platforms.iter().filter(|p| p.engine_placement == id).map(|p| p.id).collect(),
        })
        .collect()
}

fn damage_model() -> CodexDamageModel {
    let mut car = Car::new(CarConfig::default());
    let engine_power_curve = sample(0.0, 1.0, 40, |h| {
        car.state.engine_health = h;
        car.available_engine_power_ratio()
    });
    car.state.engine_health = 1.0;
    let steering_pull_curve = sample(-1.0, 1.0, 40, |d| {
        car.state.suspension_health = [(1.0 + d).min(1.0), (1.0 - d).min(1.0), 1.0, 1.0];
        car.steering_pull_bias()
    });
    let mut wheel = WheelAssembly::default();
    wheel.temperature = wheel.config.compound.optimal_temp_range.0;
    let fresh = wheel.effective_friction(1.0);
    let tire_wear_curve = sample(0.0, 1.0, 20, |w| {
        wheel.wear = w;
        wheel.effective_friction(1.0) / fresh
    });

    let mut invoice_examples = Vec::new();
    for tier in 1..=5u32 {
        let base_purse = ModuleCareerProgress::round_base_purse(tier);
        for health in [0.5f32, 0.0] {
            let (placement, archetype) = (EnginePlacement::FrontEngine, SuspensionArchetype::DoubleWishbone);
            invoice_examples.push(CodexInvoiceExample {
                tier,
                base_purse,
                health,
                placement,
                archetype,
                // A race win (earned purse = base purse) with credits above the safety net limit.
                invoice: ItemizedRepairInvoice::calculate(
                    tier,
                    base_purse,
                    SAFETY_NET_CREDIT_LIMIT * 100,
                    placement,
                    archetype,
                    archetype,
                    health,
                    health,
                    [health; 4],
                ),
            });
        }
    }

    CodexDamageModel {
        id: "damage",
        energy_deadzone_j: DAMAGE_ENERGY_DEADZONE_J,
        chassis_capacity_j: CHASSIS_DAMAGE_CAPACITY_J,
        engine_capacity_j: ENGINE_DAMAGE_CAPACITY_J,
        suspension_capacity_j: SUSPENSION_DAMAGE_CAPACITY_J,
        kerb_bottom_out_speed_mps: KERB_BOTTOM_OUT_SPEED_MPS,
        kerb_bottom_out_capacity_j: KERB_BOTTOM_OUT_CAPACITY_J,
        landing_speed_limit_mps: LANDING_SPEED_LIMIT_MPS,
        landing_capacity_j: LANDING_CAPACITY_J,
        pushrod_collapse_health: PUSHROD_COLLAPSE_HEALTH,
        pushrod_collapse_drag_multiplier: PUSHROD_COLLAPSE_DRAG_MULTIPLIER,
        zones: ImpactZone::ALL
            .into_iter()
            .map(|id| CodexImpactZone {
                id,
                weights: EnginePlacement::ALL
                    .into_iter()
                    .map(|placement| {
                        let (chassis, engine, suspension) = id.damage_weights(placement);
                        CodexDamageWeights { placement, chassis, engine, suspension }
                    })
                    .collect(),
            })
            .collect(),
        engine_power_curve,
        steering_pull_curve,
        tire_wear_curve,
        field_repair_caps: FieldRepairCaps {
            chassis: FIELD_REPAIR_CHASSIS_CAP,
            engine: FIELD_REPAIR_ENGINE_CAP,
            suspension: FIELD_REPAIR_SUSPENSION_CAP,
        },
        pit_stop_repair_amount: race_kit::world::PIT_STOP_REPAIR_AMOUNT,
        purse_cap_share: REPAIR_PURSE_CAP_SHARE,
        safety_net_credit_limit: SAFETY_NET_CREDIT_LIMIT,
        safety_net_health: SAFETY_NET_HEALTH,
        invoice_examples,
    }
}

// ==============================================================================
// 🎮 Driving Section DTOs & Builders (Spec 087 P3)
// ==============================================================================

#[derive(Serialize)]
pub struct CodexControlsData {
    pub id: &'static str,
    pub presets: Vec<CodexControlPreset>,
    pub hotkeys: Vec<CodexHotkey>,
    pub gamepad: CodexGamepadSettings,
}

#[derive(Serialize)]
pub struct CodexControlPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub bindings: Vec<CodexActionBinding>,
}

#[derive(Serialize)]
pub struct CodexActionBinding {
    pub action: &'static str,
    pub label: &'static str,
    pub keys: Vec<String>,
    pub gamepad: Vec<String>,
}

#[derive(Serialize)]
pub struct CodexHotkey {
    pub key: &'static str,
    pub action: &'static str,
    pub context: &'static str,
    pub description: &'static str,
}

#[derive(Serialize)]
pub struct CodexGamepadSettings {
    pub stick_deadzone: f32,
    pub trigger_deadzone: f32,
    pub steer_exponent: f32,
    pub steer_scale: f32,
    pub description: &'static str,
}

#[derive(Serialize)]
pub struct CodexDrivingData {
    pub id: &'static str,
    pub steering_profiles: Vec<CodexSteeringProfile>,
    pub assist_profiles: Vec<CodexAssistProfile>,
    pub aids: CodexHandlingAids,
}

#[derive(Serialize)]
pub struct CodexSteeringProfile {
    pub id: &'static str,
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub steer_time_ms: f32,
    pub return_time_ms: f32,
    pub steer_authority: f32,
    pub center_precision: f32,
    pub pedal_time_ms: f32,
    pub traction_help: f32,
    pub recommended_for: &'static str,
    pub step_response: Vec<[f32; 2]>,
}

#[derive(Serialize)]
pub struct CodexAssistProfile {
    pub id: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub short_name: &'static str,
    pub description: &'static str,
    pub tcs_enabled: bool,
    pub tcs_slip_threshold: f32,
    pub tcs_strength: f32,
    pub tcs_slip_angle_deg: f32,
    pub tcs_drift_bypass: bool,
    pub esc_enabled: bool,
    pub esc_yaw_threshold: f32,
    pub esc_strength: f32,
    pub esc_sideslip_limit_deg: f32,
    pub counter_steer_assist_enabled: bool,
    pub counter_steer_assist_strength: f32,
    pub abs_enabled: bool,
    pub abs_slip_threshold: f32,
    pub abs_strength: f32,
    pub handbrake_bypass: bool,
}

#[derive(Serialize)]
pub struct CodexHandlingAids {
    pub grip_aware_steering_description: &'static str,
    pub low_speed_authority_description: &'static str,
}

#[derive(Serialize)]
pub struct CodexHudData {
    pub id: &'static str,
    pub elements: Vec<CodexHudElement>,
    pub hologram_modes: Vec<CodexHologramMode>,
    pub curve_helper: CodexCurveHelperInfo,
    pub locator_aids: Vec<CodexLocatorAid>,
    pub cameras: Vec<CodexCameraLevel>,
    pub camera_config: CodexCameraConfigData,
}

#[derive(Serialize)]
pub struct CodexHudElement {
    pub id: &'static str,
    pub name: &'static str,
    pub screen_position: &'static str,
    pub hotkey: Option<&'static str>,
    pub description: &'static str,
}

#[derive(Serialize)]
pub struct CodexHologramMode {
    pub id: &'static str,
    pub name: &'static str,
    pub hotkey: &'static str,
    pub description: &'static str,
    pub features: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct CodexCurveHelperInfo {
    pub cycle_hotkey: &'static str,
    pub color_cycle_hotkey: &'static str,
    pub styles: Vec<CodexCurveHelperStyle>,
    pub color_schemes: Vec<CodexCurveColorScheme>,
}

#[derive(Serialize)]
pub struct CodexCurveHelperStyle {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Serialize)]
pub struct CodexCurveColorScheme {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub palette: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct CodexLocatorAid {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Serialize)]
pub struct CodexCameraLevel {
    pub id: &'static str,
    pub name: &'static str,
    pub mode: &'static str,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub description: &'static str,
}

#[derive(Serialize)]
pub struct CodexCameraConfigData {
    pub position_smoothing: f32,
    pub zoom_smoothing: f32,
    pub velocity_lookahead_time: f32,
    pub trauma_decay: f32,
    pub max_shake_offset: f32,
    pub max_car_screen_offset_frac: f32,
}

fn controls_data() -> CodexControlsData {
    let action_specs = [
        (ArcadeAction::Up, "throttle", "Throttle / Accelerate"),
        (ArcadeAction::Down, "brake", "Brake / Reverse"),
        (ArcadeAction::Left, "steer_left", "Steer Left"),
        (ArcadeAction::Right, "steer_right", "Steer Right"),
        (ArcadeAction::Action3, "handbrake", "Handbrake / Drift"),
        (ArcadeAction::Primary, "primary", "Primary / Confirm"),
        (ArcadeAction::Secondary, "secondary", "Secondary / Boost"),
        (ArcadeAction::Pause, "pause", "Pause Game"),
        (ArcadeAction::Menu, "menu", "In-Game Menu"),
    ];

    let build_preset = |id: &'static str, name: &'static str, desc: &'static str, map: InputMap| -> CodexControlPreset {
        let mut bindings = Vec::new();
        for &(action, action_id, label) in &action_specs {
            let mut keys = Vec::new();
            let mut gamepad = Vec::new();
            if let Some(sources) = map.bindings.get(&action) {
                for src in sources {
                    match src {
                        InputSource::Key(k) => keys.push(k.label().to_string()),
                        InputSource::GamepadBtn(b) => gamepad.push(b.label().to_string()),
                        InputSource::GamepadAxisPos(a) => gamepad.push(format!("{}+", a.label())),
                        InputSource::GamepadAxisNeg(a) => gamepad.push(format!("{}-", a.label())),
                    }
                }
            }
            bindings.push(CodexActionBinding {
                action: action_id,
                label,
                keys,
                gamepad,
            });
        }
        CodexControlPreset {
            id,
            name,
            description: desc,
            bindings,
        }
    };

    let presets = vec![
        build_preset(
            "hybrid",
            "Hybrid (QAOP + Arrows + Gamepad)",
            "Default racing layout. Supports two-handed keyboard steering (Q/A throttle/brake, O/P or Arrows steer) alongside full gamepad triggers and stick.",
            InputMap::default_racing(),
        ),
        build_preset(
            "wasd",
            "WASD Racing",
            "Standard PC driving layout with W/S pedals, A/D steering, and Space handbrake.",
            InputMap::wasd_racing(),
        ),
        build_preset(
            "arrows",
            "Arrows Racing",
            "Dedicated Arrow keys layout with Up/Down pedals, Left/Right steering, and Space handbrake.",
            InputMap::arrows_racing(),
        ),
        build_preset(
            "classic",
            "Classic QAOP",
            "Authentic 8-bit microcomputer arcade racing layout with Q/A pedals, O/P steering, and Space handbrake.",
            InputMap::classic_racing(),
        ),
    ];

    let hotkeys = vec![
        CodexHotkey {
            key: "H",
            action: "Cycle Driver Assists",
            context: "In-race",
            description: "Cycles assist profile: Arcade (Full) -> Sport (Mild) -> Pro (OFF).",
        },
        CodexHotkey {
            key: "R",
            action: "Restart Race",
            context: "In-race / Paused",
            description: "Instantly reinitialises the current race session (single player).",
        },
        CodexHotkey {
            key: "Tab",
            action: "Cycle Camera Zoom",
            context: "In-race",
            description: "Cycles follow camera zoom levels: Close -> Medium -> Far -> Overview.",
        },
        CodexHotkey {
            key: "L",
            action: "Toggle Headlights",
            context: "In-race",
            description: "Toggles vehicle headlights for cars equipped with illumination.",
        },
        CodexHotkey {
            key: "C",
            action: "Cycle Control Preset",
            context: "Pause Menu",
            description: "Cycles input control presets (Hybrid, WASD, Arrows, Classic QAOP).",
        },
        CodexHotkey {
            key: "S / P",
            action: "Cycle Steering Profile",
            context: "Pause Menu",
            description: "Cycles keyboard digital steering profiles: Smooth -> Balanced -> Sharp -> Raw.",
        },
        CodexHotkey {
            key: "G",
            action: "Open Gamepad Mapper",
            context: "Pause Menu",
            description: "Launches the interactive gamepad calibration and button binding tool.",
        },
        CodexHotkey {
            key: "Esc",
            action: "Pause / Resume / Back",
            context: "In-race / Menus",
            description: "Opens pause menu during race, or navigates back in menus.",
        },
        CodexHotkey {
            key: "Ctrl + 1",
            action: "Cockpit Hologram: Kinematics",
            context: "In-race",
            description: "Switches cockpit telemetry HUD to Kinematic Linkages & Structural Damage mode.",
        },
        CodexHotkey {
            key: "Ctrl + 2",
            action: "Cockpit Hologram: Dynamics",
            context: "In-race",
            description: "Switches cockpit telemetry HUD to Dynamic Telemetry & Damper Travel mode.",
        },
        CodexHotkey {
            key: "5",
            action: "Cycle Curve Helper Style",
            context: "In-race",
            description: "Cycles approaching curve helper: Rally Pacenote -> Chevrons -> Off.",
        },
        CodexHotkey {
            key: "6",
            action: "Cycle Curve Helper Colors",
            context: "In-race",
            description: "Cycles curve helper color schemes: Traffic -> Synthwave -> Contrast -> Rally.",
        },
        CodexHotkey {
            key: "F1",
            action: "Toggle LIDAR Beams",
            context: "In-race Debug",
            description: "Renders distance-sensing LIDAR rays projected from car perimeter.",
        },
        CodexHotkey {
            key: "F2",
            action: "Toggle Checkpoint Gates",
            context: "In-race Debug",
            description: "Renders checkpoint gate lines, orientation vectors, and racing line progression.",
        },
        CodexHotkey {
            key: "F3",
            action: "Toggle Collision OBBs",
            context: "In-race Debug",
            description: "Visualizes vehicle oriented bounding boxes (OBBs) and wheel contact vectors.",
        },
        CodexHotkey {
            key: "F4",
            action: "Toggle AI Paths & Waypoints",
            context: "In-race Debug",
            description: "Displays bot racing line splines, target waypoints, and overtaking corridors.",
        },
        CodexHotkey {
            key: "F5",
            action: "Toggle Telemetry Panel",
            context: "In-race Debug",
            description: "Displays comprehensive screen-space real-time vehicle telemetry metrics.",
        },
        CodexHotkey {
            key: "F6 / Z",
            action: "Toggle Touch Controls",
            context: "In-race",
            description: "Toggles virtual touch controls overlay on desktop screens.",
        },
        CodexHotkey {
            key: "F8",
            action: "Toggle Split-Screen Layout",
            context: "Split Screen",
            description: "Toggles 2-player split screen view between Vertical and Horizontal.",
        },
        CodexHotkey {
            key: "F11",
            action: "Championship Editor",
            context: "Global",
            description: "Opens the built-in Championship Editor studio.",
        },
        CodexHotkey {
            key: "F12 / Ctrl+D",
            action: "Toggle Dev Mode",
            context: "Global",
            description: "Enables developer debug tools and hot-reloading facilities.",
        },
    ];

    let gp = GamepadConfig::default();
    let gamepad = CodexGamepadSettings {
        stick_deadzone: gp.stick_deadzone,
        trigger_deadzone: gp.trigger_deadzone,
        steer_exponent: gp.steer_exponent,
        steer_scale: gp.steer_scale,
        description: "Analog stick and trigger inputs apply calibrated inner deadzones to prevent drift from resting potentiometer variance. Beyond deadzone, stick input applies a gentle progressive power curve (exponent 1.15) for refined high-speed center precision without sacrificing rapid full-lock agility.",
    };

    CodexControlsData {
        id: "controls",
        presets,
        hotkeys,
        gamepad,
    }
}

fn driving_data() -> CodexDrivingData {
    let steering_profiles = SteeringProfile::PRESETS
        .into_iter()
        .map(|profile| {
            let cfg = DigitalInputConfig::from_profile(profile);
            let (id, display_name, description, recommended_for) = match profile {
                SteeringProfile::Smooth => (
                    "smooth",
                    "Smooth",
                    "Relaxed steering rise, short of the front grip limit, and maximum traction help. Ideal for high-speed stability and beginners.",
                    "Heavy GT cars, stock cars on high-speed ovals, keyboard beginners",
                ),
                SteeringProfile::Balanced => (
                    "balanced",
                    "Balanced",
                    "Default calibrated response: right on the front tyre grip limit with moderate steering speed and traction assist.",
                    "Standard racing across all disciplines, touring cars, sports cars",
                ),
                SteeringProfile::Sharp => (
                    "sharp",
                    "Sharp",
                    "Quick steering response pushing slightly past the peak slip angle, with light traction help. Allows snappy weight transfer.",
                    "Autocross, tight twisty kart circuits, agile hot hatches",
                ),
                SteeringProfile::Raw => (
                    "raw",
                    "Raw",
                    "Near-instant digital key response, well past the front grip limit with zero traction help. Maximum drift initiation authority.",
                    "Rallycross Scandinavian flicks, power sliding, expert keyboard drifters",
                ),
                _ => unreachable!(),
            };

            let mut filter = DigitalInputFilter::new(cfg);
            let mut step_response = Vec::with_capacity(61);
            let dt = 0.005; // 5ms steps
            for i in 0..=60 {
                let t_ms = i as f32 * 5.0;
                if i == 0 {
                    step_response.push([0.0, 0.0]);
                } else {
                    let (steer, _, _) = filter.update(1.0, 0.0, 0.0, dt);
                    step_response.push([t_ms, (steer * 1000.0).round() / 1000.0]);
                }
            }

            CodexSteeringProfile {
                id,
                name: id,
                display_name,
                description,
                steer_time_ms: cfg.steer_time_ms,
                return_time_ms: (cfg.steer_time_ms * DigitalInputConfig::RETURN_TIME_FRACTION * 10.0).round() / 10.0,
                steer_authority: cfg.steer_authority,
                center_precision: cfg.center_precision,
                pedal_time_ms: cfg.pedal_time_ms,
                traction_help: cfg.traction_help,
                recommended_for,
                step_response,
            }
        })
        .collect();

    let assist_profiles = AssistProfile::ALL
        .into_iter()
        .map(|mode| {
            let cfg = mode.to_config();
            let id = match mode {
                AssistProfile::Arcade => "arcade",
                AssistProfile::Sport => "sport",
                AssistProfile::Pro => "pro",
            };
            CodexAssistProfile {
                id,
                name: mode.short_name(),
                title: mode.title(),
                short_name: mode.short_name(),
                description: mode.description(),
                tcs_enabled: cfg.tcs_enabled,
                tcs_slip_threshold: cfg.tcs_slip_threshold,
                tcs_strength: cfg.tcs_strength,
                tcs_slip_angle_deg: cfg.tcs_slip_angle_deg,
                tcs_drift_bypass: cfg.tcs_drift_bypass,
                esc_enabled: cfg.esc_enabled,
                esc_yaw_threshold: cfg.esc_yaw_threshold,
                esc_strength: cfg.esc_strength,
                esc_sideslip_limit_deg: cfg.esc_sideslip_limit_deg,
                counter_steer_assist_enabled: cfg.counter_steer_assist_enabled,
                counter_steer_assist_strength: cfg.counter_steer_assist_strength,
                abs_enabled: cfg.abs_enabled,
                abs_slip_threshold: cfg.abs_slip_threshold,
                abs_strength: cfg.abs_strength,
                handbrake_bypass: cfg.handbrake_bypass,
            }
        })
        .collect();

    let aids = CodexHandlingAids {
        grip_aware_steering_description: "Grip-aware steering dynamically maps full controller or keyboard input to the useful slip angle of the front tires at current vehicle speed and normal load. At high speeds, holding full lock would induce catastrophic front tire scrub and terminal understeer; grip-aware steering ensures your input commands maximum available lateral force without front tire plowing.",
        low_speed_authority_description: "Spec 072 introduces low-speed authority expansion: when vehicle speed drops below 12 m/s (~43 km/h), the steering authority floor progressively expands up to the geometric Ackermann lock angle. This provides tight hairpin turn-in, low-speed parking, and hairpin pivot authority while smoothly tapering back to high-speed grip calibration as speed builds.",
    };

    CodexDrivingData {
        id: "driving",
        steering_profiles,
        assist_profiles,
        aids,
    }
}

fn hud_data() -> CodexHudData {
    let elements = vec![
        CodexHudElement {
            id: "speedometer",
            name: "Speedometer & Cluster",
            screen_position: "Bottom Right",
            hotkey: None,
            description: "Hybrid digital/analog speed gauge with speed in km/h, gear indicator, rpm bar, electronic assist active indicators (TCS/ESC/ABS), and drift meter.",
        },
        CodexHudElement {
            id: "position_and_lap",
            name: "Position & Lap Counter",
            screen_position: "Top Left",
            hotkey: None,
            description: "Displays current race position with podium accent colors (Gold, Silver, Bronze), total field count, current lap, and total laps.",
        },
        CodexHudElement {
            id: "lap_timer",
            name: "Lap Timer & Sector Splits",
            screen_position: "Top Center",
            hotkey: None,
            description: "High-precision timer displaying current lap time, last lap, personal best, and dynamic sector split time deltas (green = faster, red = slower).",
        },
        CodexHudElement {
            id: "minimap",
            name: "Mini-Map Radar",
            screen_position: "Top Right",
            hotkey: None,
            description: "Track overview with player and AI position pips, orientation vectors, pit lane route, and sector boundary markers.",
        },
        CodexHudElement {
            id: "compound_badge",
            name: "Compound Badge",
            screen_position: "Inside Cockpit Hologram",
            hotkey: None,
            description: "Displays current tire compound (e.g. Spliced Slick, Hard Compound, Wet Tread, Gravel Rally) and thermal state.",
        },
        CodexHudElement {
            id: "joker_badge",
            name: "Joker Lap Badge",
            screen_position: "Top Left (below Position)",
            hotkey: None,
            description: "Rallycross tactical badge indicating required vs completed Joker laps (e.g. 'JOKER REQUIRED' in amber, 'JOKER DONE' in green).",
        },
        CodexHudElement {
            id: "pit_messages",
            name: "Pit Lane Alerts & Overlays",
            screen_position: "Center / Top",
            hotkey: None,
            description: "Tactical 'BOX THIS LAP' warning when tire wear exceeds 60% or car health drops below 50%; speed limiter banner (60 km/h) in pit lane; service countdown overlay.",
        },
        CodexHudElement {
            id: "cockpit_hologram",
            name: "Tactical Cockpit Hologram",
            screen_position: "Bottom Left",
            hotkey: Some("Ctrl + 1 / Ctrl + 2"),
            description: "Vector chassis projection showing proportional geometry, engine placement, 4-corner wheel assemblies with thermal/wear status, and switchable kinematics/dynamics modes.",
        },
        CodexHudElement {
            id: "curve_helper",
            name: "Approaching Curve Helper",
            screen_position: "In-World / HUD Ribbon",
            hotkey: Some("5 / 6"),
            description: "Anticipatory curve advisory indicating corner radius, apex direction, braking zones, and rally pacenote severity (1 to 6).",
        },
        CodexHudElement {
            id: "nameplates",
            name: "Opponent Nameplates",
            screen_position: "In-World above Vehicles",
            hotkey: None,
            description: "Floating racer badges showing driver name, position, and gap time delta.",
        },
        CodexHudElement {
            id: "locator_aids",
            name: "Player Locator Aids",
            screen_position: "In-World around Player Vehicle",
            hotkey: None,
            description: "Overhead chevron pointer, ground proximity aura, roof strobe beacon, and radar sonar ping for high-density pack clarity.",
        },
        CodexHudElement {
            id: "debug_overlays",
            name: "Developer & Telemetry Overlays",
            screen_position: "Full Screen",
            hotkey: Some("F1 - F5"),
            description: "LIDAR raycasts (F1), checkpoint gates (F2), collision OBBs (F3), AI paths (F4), and real-time telemetry panel (F5).",
        },
    ];

    let hologram_modes = vec![
        CodexHologramMode {
            id: "kinematics",
            name: "KINEMATICS",
            hotkey: "Ctrl + 1",
            description: "Structural linkage mode. Depicts suspension wishbone / pushrod geometry, buckled fracture deformation lines, dynamic camber skew, and perimeter impact zone damage meters.",
            features: vec![
                "Mechanical linkage geometry matching SuspensionArchetype",
                "Buckled fracture line rendering upon severe impact",
                "Wheel camber skew visualization under cornering load",
                "Perimeter impact zone wear bars (Nose, Tail, Flanks, Corners)",
            ],
        },
        CodexHologramMode {
            id: "dynamics",
            name: "DYNAMICS",
            hotkey: "Ctrl + 2",
            description: "Telemetry & suspension stroke mode. Displays 4-corner damper stroke capsules with real-time compression travel and dynamic Fz normal load transfer glow.",
            features: vec![
                "4-corner damper stroke compression travel bars",
                "Dynamic Fz normal load transfer intensity glow",
                "Tire surface temperature gradient indicators",
                "Tire tread wear depletion gauges",
            ],
        },
    ];

    let curve_helper = CodexCurveHelperInfo {
        cycle_hotkey: "5",
        color_cycle_hotkey: "6",
        styles: vec![
            CodexCurveHelperStyle {
                id: "pacenote",
                name: "Rally Pacenote",
                description: "Displays rally-style corner severity numbers (1 = hairpin to 6 = slight bend) with turn direction arrow.",
            },
            CodexCurveHelperStyle {
                id: "chevrons",
                name: "Severity Chevrons",
                description: "Projects sequential directional chevron arrows along the approaching corner trajectory.",
            },
            CodexCurveHelperStyle {
                id: "off",
                name: "Disabled",
                description: "Hides all curve approach indicators for purist simulation driving.",
            },
        ],
        color_schemes: vec![
            CodexCurveColorScheme {
                id: "traffic",
                name: "Traffic Light",
                description: "Green (flat out) -> Yellow (lift/caution) -> Orange (heavy braking) -> Red (hairpin emergency).",
                palette: vec!["#22c55e", "#eab308", "#f97316", "#ef4444"],
            },
            CodexCurveColorScheme {
                id: "synthwave",
                name: "Synthwave Neon",
                description: "Cyan -> Electric Purple -> Neon Magenta -> Hot Pink.",
                palette: vec!["#06b6d4", "#a855f7", "#ec4899", "#f43f5e"],
            },
            CodexCurveColorScheme {
                id: "contrast",
                name: "High Contrast",
                description: "Pure White -> Bright Amber -> High-vis Yellow -> Stark Crimson.",
                palette: vec!["#ffffff", "#f59e0b", "#eab308", "#dc2626"],
            },
            CodexCurveColorScheme {
                id: "rally",
                name: "Rally Stage",
                description: "Fluorescent Blue -> Chartreuse Green -> Vivid Yellow -> Blaze Orange.",
                palette: vec!["#3b82f6", "#84cc16", "#eab308", "#ea580c"],
            },
        ],
    };

    let locator_aids = vec![
        CodexLocatorAid {
            id: "overhead_chevron",
            name: "Overhead Floating Chevron",
            description: "High-visibility floating arrow directly above the player's car, scaling dynamically with camera zoom.",
        },
        CodexLocatorAid {
            id: "ground_aura",
            name: "Ground Proximity Aura",
            description: "Soft pulsing circular ground glow underneath the vehicle chassis, highlighting position in pack battles and dust clouds.",
        },
        CodexLocatorAid {
            id: "roof_beacon",
            name: "Roof Strobe Beacon",
            description: "High-contrast roof strobe light designed for extreme off-road mud and night racing visibility.",
        },
        CodexLocatorAid {
            id: "adaptive_visibility",
            name: "Adaptive Contrast Scaling",
            description: "Automatically increases aura and chevron luminance when obscured by tire vapor, spray, or off-track foliage.",
        },
        CodexLocatorAid {
            id: "sonar_ping",
            name: "Radar Sonar Ping",
            description: "Periodic concentric sonar pulse on the minimap radiating from the player's vehicle.",
        },
    ];

    let cameras = vec![
        CodexCameraLevel {
            id: "close",
            name: "Close",
            mode: "follow",
            min_zoom: 13.5,
            max_zoom: 22.0,
            description: "Intimate follow camera with prominent car detail and intense motion sense. Best for precision karting and autocross.",
        },
        CodexCameraLevel {
            id: "medium",
            name: "Medium (Default)",
            mode: "follow",
            min_zoom: 10.0,
            max_zoom: 16.5,
            description: "Balanced follow perspective giving adequate forward vision into braking zones while preserving vehicle presence.",
        },
        CodexCameraLevel {
            id: "far",
            name: "Far",
            mode: "follow",
            min_zoom: 7.5,
            max_zoom: 12.0,
            description: "Wide follow camera offering expansive tactical view of upcoming corners and competitor overtakes. Great for GT and high-speed circuits.",
        },
        CodexCameraLevel {
            id: "overview",
            name: "Overview",
            mode: "overview",
            min_zoom: 1.0,
            max_zoom: 1.0,
            description: "Static full-track overview camera fitting the entire circuit within the viewport in classic GeneRally style.",
        },
    ];

    let cam_cfg = CameraConfig::default();
    let camera_config = CodexCameraConfigData {
        position_smoothing: cam_cfg.position_smoothing,
        zoom_smoothing: cam_cfg.zoom_smoothing,
        velocity_lookahead_time: cam_cfg.velocity_lookahead_time,
        trauma_decay: cam_cfg.trauma_decay,
        max_shake_offset: cam_cfg.max_shake_offset,
        max_car_screen_offset_frac: MAX_CAR_SCREEN_OFFSET_FRAC,
    };

    CodexHudData {
        id: "hud",
        elements,
        hologram_modes,
        curve_helper,
        locator_aids,
        cameras,
        camera_config,
    }
}

// --- Racing DTOs (Spec 087 P4) ---

#[derive(Serialize)]
pub struct CodexTierInfo {
    pub tier: u32,
    pub name: String,
    pub required_licence: &'static str,
    pub car_cost_xp: u64,
    pub base_purse: u64,
}

#[derive(Serialize)]
pub struct CodexDisciplineInfo {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub description: &'static str,
    pub launch: bool,
    pub car_count: usize,
    pub circuit_count: usize,
    pub series_count: usize,
    pub tiers: Vec<CodexTierInfo>,
}

#[derive(Serialize)]
pub struct CodexFormatInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub rules: Vec<&'static str>,
    pub scoring_summary: &'static str,
}

#[derive(Serialize)]
pub struct CodexJokerRuleInfo {
    pub mandatory_laps: u32,
    pub time_penalty_sec: f32,
    pub applies_to: &'static str,
    pub hud_indicator: &'static str,
    pub strategy_notes: &'static str,
}

#[derive(Serialize)]
pub struct CodexPitPhase {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Serialize)]
pub struct CodexPitServiceInfo {
    pub repair_amount: f32,
    pub speed_limiter_kmh: f32,
    pub field_repair_chassis_cap: f32,
    pub field_repair_engine_cap: f32,
    pub field_repair_suspension_cap: f32,
    pub phases: Vec<CodexPitPhase>,
}

#[derive(Serialize)]
pub struct CodexPositionPoints {
    pub position: usize,
    pub points: u32,
}

#[derive(Serialize)]
pub struct CodexPointSystemInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub points_table: Vec<CodexPositionPoints>,
    pub fastest_lap_bonus: Option<&'static str>,
    pub stage_win_bonus: Option<&'static str>,
}

#[derive(Serialize)]
pub struct CodexRoundInfo {
    pub order: usize,
    pub track_id: String,
    pub name: Option<String>,
    pub laps: Option<u32>,
}

#[derive(Serialize)]
pub struct CodexSeriesDriver {
    pub id: String,
    pub name: String,
    pub team: String,
    pub is_player: bool,
    pub car_model_id: Option<String>,
    pub country: Option<String>,
    pub ai_style: Option<String>,
    pub ai_tier: Option<u8>,
}

#[derive(Serialize)]
pub struct CodexSeriesPresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub module_id: String,
    pub tier: u32,
    pub laps_per_round: u32,
    pub scoring_system: String,
    pub fastest_lap_bonus: bool,
    pub stage_win_bonus: bool,
    pub round_count: usize,
    pub rounds: Vec<CodexRoundInfo>,
    pub driver_count: usize,
    pub drivers: Vec<CodexSeriesDriver>,
}

#[derive(Serialize)]
pub struct CodexCareerTierDetails {
    pub tier: u32,
    pub name: &'static str,
    pub promotion_criteria: &'static str,
    pub starter_car_id: Option<&'static str>,
    pub starter_car_name: Option<&'static str>,
    pub car_cost_xp: u64,
    pub first_time_exploration_xp: u64,
    pub round_base_purse: u64,
    pub unlocked_tracks: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct CodexCareerLadder {
    pub module_id: &'static str,
    pub module_name: &'static str,
    pub tiers: Vec<CodexCareerTierDetails>,
}

#[derive(Serialize)]
pub struct CodexXpEconomyInfo {
    pub distance_divisor: f32,
    pub completion_multiplier: f32,
    pub first_time_bonus_per_tier: u64,
    pub car_cost_per_tier: u64,
    pub formulas: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct CodexRepairEconomyInfo {
    pub purse_cap_share: f64,
    pub safety_net_credit_limit: u64,
    pub safety_net_health: f32,
    pub formulas: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct CodexCareerInfo {
    pub ladders: Vec<CodexCareerLadder>,
    pub xp_economy: CodexXpEconomyInfo,
    pub repair_economy: CodexRepairEconomyInfo,
}

#[derive(Serialize)]
pub struct CodexAcademyLesson {
    pub id: &'static str,
    pub index: usize,
    pub title: String,
    pub description: String,
    pub track_slug: String,
    pub car_slug: String,
    pub gold_time_sec: f32,
    pub silver_time_sec: f32,
    pub bronze_time_sec: f32,
    pub bronze_credit_bounty: u64,
    pub silver_credit_bounty: u64,
    pub gold_credit_bounty: u64,
    pub base_xp_reward: u32,
}

#[derive(Serialize)]
pub struct CodexLicenceGrade {
    pub grade: &'static str,
    pub title: &'static str,
    pub badge: &'static str,
    pub description: &'static str,
    pub required_for: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct CodexAcademyInfo {
    pub curriculum: Vec<CodexAcademyLesson>,
    pub max_permissible_impulse: f32,
    pub max_permissible_off_track_sec: f32,
    pub licence_grades: Vec<CodexLicenceGrade>,
}

#[derive(Serialize)]
pub struct CodexCollisionModeInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Serialize)]
pub struct CodexMultiplayerInfo {
    pub min_players: u32,
    pub max_players: u32,
    pub simulation_rate_hz: u32,
    pub interpolation_delay_ms: u32,
    pub discovery: &'static str,
    pub collision_modes: Vec<CodexCollisionModeInfo>,
    pub host_steps: Vec<&'static str>,
    pub join_steps: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct CodexRacingData {
    pub id: &'static str,
    pub disciplines: Vec<CodexDisciplineInfo>,
    pub formats: Vec<CodexFormatInfo>,
    pub joker_rule: CodexJokerRuleInfo,
    pub pit_service: CodexPitServiceInfo,
    pub point_systems: Vec<CodexPointSystemInfo>,
    pub series: Vec<CodexSeriesPresetInfo>,
    pub career: CodexCareerInfo,
    pub academy: CodexAcademyInfo,
    pub multiplayer: CodexMultiplayerInfo,
}

// --- Rivals DTOs (Spec 087 P4) ---

#[derive(Serialize)]
pub struct CodexDriverStatsInfo {
    pub speed: f32,
    pub aggression: f32,
    pub precision: f32,
    pub defense: f32,
}

#[derive(Serialize)]
pub struct CodexDriverFavoriteCarInfo {
    pub discipline: &'static str,
    pub tier: u8,
    pub model_id: &'static str,
}

#[derive(Serialize)]
pub struct CodexDriverCharacterInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub alias: &'static str,
    pub bio: &'static str,
    pub style: &'static str,
    pub preferred_car: &'static str,
    pub primary_color: [f32; 4],
    pub secondary_color: [f32; 4],
    pub accent_color: [f32; 4],
    pub stats: CodexDriverStatsInfo,
    pub favorite_cars: Vec<CodexDriverFavoriteCarInfo>,
}

#[derive(Serialize)]
pub struct CodexDrivingStyleInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub speed_mult: f32,
    pub aggression: f32,
    pub precision: f32,
    pub defense: f32,
    pub tactical_traits: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct CodexSkillTierInfo {
    pub tier: u8,
    pub name: &'static str,
    pub short_name: &'static str,
    pub tag: &'static str,
    pub pace_limit: f32,
    pub consistency: f32,
    pub composure: f32,
    pub bell_curve_weights: [u8; 5],
}

#[derive(Serialize)]
pub struct CodexMistakeKindInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub trigger_condition: &'static str,
    pub gameplay_impact: &'static str,
    pub recovery_behavior: &'static str,
}

#[derive(Serialize)]
pub struct CodexRivalsData {
    pub id: &'static str,
    pub drivers: Vec<CodexDriverCharacterInfo>,
    pub driving_styles: Vec<CodexDrivingStyleInfo>,
    pub skill_tiers: Vec<CodexSkillTierInfo>,
    pub mistake_kinds: Vec<CodexMistakeKindInfo>,
}

fn racing_data(scope: Scope, cars: &[CodexCar], circuits: &[CodexCircuit]) -> CodexRacingData {
    let disciplines = modules()
        .into_iter()
        .filter(|m| scope.includes_module(m.id()))
        .map(|m| {
            let id = m.id();
            let (desc, tiers) = match id {
                "classic" => (
                    "Retro top-down racing inspired by GeneRally, Super Sprint and Micro Machines, featuring fantasy muscle cars, karts, formula singles, and off-roaders.",
                    vec![CodexTierInfo {
                        tier: 1,
                        name: "Open Competition".to_string(),
                        required_licence: "Class D (National Grassroots)",
                        car_cost_xp: 1_000,
                        base_purse: 5_000,
                    }],
                ),
                "kart" => (
                    "Ultra-low inertia racing with rigid tubular chassis, high lateral g-forces, and split-second steering response from cadet classes up to Division 1 Superkarts.",
                    vec![
                        CodexTierInfo { tier: 1, name: "Cadet 60cc".to_string(), required_licence: "Class D (National Grassroots)", car_cost_xp: 1_000, base_purse: 5_000 },
                        CodexTierInfo { tier: 2, name: "Junior OK-J".to_string(), required_licence: "Class D (National Grassroots)", car_cost_xp: 2_000, base_purse: 12_000 },
                        CodexTierInfo { tier: 3, name: "Senior OK / Shifter".to_string(), required_licence: "Class D (National Grassroots)", car_cost_xp: 3_000, base_purse: 25_000 },
                        CodexTierInfo { tier: 4, name: "KZ2 Shifter Pro".to_string(), required_licence: "Class D (National Grassroots)", car_cost_xp: 4_000, base_purse: 55_000 },
                        CodexTierInfo { tier: 5, name: "Superkart Div 2".to_string(), required_licence: "Class D (National Grassroots)", car_cost_xp: 5_000, base_purse: 120_000 },
                        CodexTierInfo { tier: 6, name: "Superkart World Series".to_string(), required_licence: "Class D (National Grassroots)", car_cost_xp: 6_000, base_purse: 150_000 },
                    ],
                ),
                "autocross" => (
                    "High-octane wheel-to-wheel dirt racing featuring screaming 600cc Cross Cars, Buggy 1600 single-seaters, and 600+ bhp SuperBuggy 4WD prototypes.",
                    vec![
                        CodexTierInfo { tier: 1, name: "Cross Car Junior".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 1_000, base_purse: 5_000 },
                        CodexTierInfo { tier: 2, name: "Cross Car Senior".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 2_000, base_purse: 12_000 },
                        CodexTierInfo { tier: 3, name: "Buggy 1600".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 3_000, base_purse: 25_000 },
                        CodexTierInfo { tier: 4, name: "Touring Autocross".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 4_000, base_purse: 55_000 },
                        CodexTierInfo { tier: 5, name: "SuperBuggy World Series".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 5_000, base_purse: 120_000 },
                    ],
                ),
                "rally" => (
                    "Explosive multi-surface circuits combining tarmac, gravel, jump crests, and tactical joker lap routes from front-wheel-drive Rally4 up to electric RX1e and Apex Group E beasts.",
                    vec![
                        CodexTierInfo { tier: 1, name: "Rally4 Grassroots Cup".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 1_000, base_purse: 5_000 },
                        CodexTierInfo { tier: 2, name: "Supercar Lites Trophy".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 2_000, base_purse: 12_000 },
                        CodexTierInfo { tier: 3, name: "Continental RX Supercars".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 3_000, base_purse: 25_000 },
                        CodexTierInfo { tier: 4, name: "World Rallycross Supercars".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 4_000, base_purse: 55_000 },
                        CodexTierInfo { tier: 5, name: "RX1e Electric Championship".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 5_000, base_purse: 120_000 },
                        CodexTierInfo { tier: 6, name: "Apex Group E Electric Trophy".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 6_000, base_purse: 150_000 },
                        CodexTierInfo { tier: 7, name: "Group B Masters".to_string(), required_licence: "Class C (Junior Competition)", car_cost_xp: 7_000, base_purse: 180_000 },
                    ],
                ),
                "gt" => (
                    "Prestigious sports car and endurance racing spanning production-based GT4, worldwide GT3 pro, high-downforce GT1 classics, and state-of-the-art hybrid Hypercars.",
                    vec![
                        CodexTierInfo { tier: 1, name: "GT4 Clubman Sprint".to_string(), required_licence: "Class A (International GT)", car_cost_xp: 1_000, base_purse: 5_000 },
                        CodexTierInfo { tier: 2, name: "Continental GT3 Sprint Challenge".to_string(), required_licence: "Class A (International GT)", car_cost_xp: 2_000, base_purse: 12_000 },
                        CodexTierInfo { tier: 3, name: "Grand Touring Biturbo Masters".to_string(), required_licence: "Class A (International GT)", car_cost_xp: 3_000, base_purse: 25_000 },
                        CodexTierInfo { tier: 4, name: "Endurance 90s Heritage Trophy".to_string(), required_licence: "Class A (International GT)", car_cost_xp: 4_000, base_purse: 55_000 },
                        CodexTierInfo { tier: 5, name: "Hypercar World GP".to_string(), required_licence: "Class S (World Superlicense)", car_cost_xp: 5_000, base_purse: 120_000 },
                    ],
                ),
                "nascar" => (
                    "Heavy V8 pushrod stock car combat from grassroots quarter-mile bullrings and dirt ovals to high-banked intermediate tracks and 200 mph restrictor-plate superspeedways.",
                    vec![
                        CodexTierInfo { tier: 1, name: "Street Stock Bullring".to_string(), required_licence: "Class B (National Pro-Am)", car_cost_xp: 1_000, base_purse: 5_000 },
                        CodexTierInfo { tier: 2, name: "Late Model Challenge".to_string(), required_licence: "Class B (National Pro-Am)", car_cost_xp: 2_000, base_purse: 12_000 },
                        CodexTierInfo { tier: 3, name: "National Stock Car Tour".to_string(), required_licence: "Class B (National Pro-Am)", car_cost_xp: 3_000, base_purse: 25_000 },
                        CodexTierInfo { tier: 4, name: "Pro Super Truck V8 Series".to_string(), required_licence: "Class B (National Pro-Am)", car_cost_xp: 4_000, base_purse: 55_000 },
                        CodexTierInfo { tier: 5, name: "Premier Stock Car Cup".to_string(), required_licence: "Class B (National Pro-Am)", car_cost_xp: 5_000, base_purse: 120_000 },
                    ],
                ),
                "extreme_offroad" => (
                    "Unrestricted terrain action featuring open tubular sand rails, 1000 bhp Trophy Trucks, Arctic ice conquerors, deep mud boggers, and 12-ton monster crushers.",
                    vec![
                        CodexTierInfo { tier: 1, name: "Desert Sand Sprint".to_string(), required_licence: "Class B (National Pro-Am)", car_cost_xp: 1_000, base_purse: 5_000 },
                        CodexTierInfo { tier: 2, name: "Canyon Raid".to_string(), required_licence: "Class B (National Pro-Am)", car_cost_xp: 2_000, base_purse: 12_000 },
                        CodexTierInfo { tier: 3, name: "Extreme Offroad Cup".to_string(), required_licence: "Class B (National Pro-Am)", car_cost_xp: 3_000, base_purse: 25_000 },
                        CodexTierInfo { tier: 4, name: "Mud Masters".to_string(), required_licence: "Class B (National Pro-Am)", car_cost_xp: 4_000, base_purse: 55_000 },
                        CodexTierInfo { tier: 5, name: "Ultimate Monster Championship".to_string(), required_licence: "Class B (National Pro-Am)", car_cost_xp: 5_000, base_purse: 120_000 },
                    ],
                ),
                _ => ("Motorsport discipline.", Vec::new()),
            };

            let series_count = EMBEDDED_PRESETS
                .iter()
                .filter_map(|&(_, toml)| SeriesDefinition::from_toml(toml).ok())
                .filter(|s| s.series.module_id == id)
                .count();

            CodexDisciplineInfo {
                id,
                title: m.title(),
                subtitle: m.subtitle(),
                description: desc,
                launch: LAUNCH_MODULES.contains(&id),
                car_count: cars.iter().filter(|c| c.module == id).count(),
                circuit_count: circuits.iter().filter(|c| c.module == id).count(),
                series_count,
                tiers,
            }
        })
        .collect();

    let formats = vec![
        CodexFormatInfo {
            id: "laps",
            name: "Lap Sprint & Feature Race",
            tag: "GRID • STANDARD MOTORSPORT",
            description: "Drivers line up on a staggered starting grid and race for a fixed number of laps. The first vehicle to cross the start/finish line after completing all scheduled laps takes the checkered flag.",
            rules: vec![
                "Staggered grid start based on qualifying or reverse championship standings.",
                "Race duration defined by track length and championship regulations (typically 3 to 10 laps).",
                "Full-contact Separating Axis Theorem (SAT) physics or non-contact ghosting.",
                "DNF recorded if vehicle suffers terminal chassis or powertrain integrity loss.",
            ],
            scoring_summary: "Finishing positions determine race purse, championship points, and podium trophies.",
        },
        CodexFormatInfo {
            id: "time_attack",
            name: "Time Attack / Solo Hotlap",
            tag: "SOLO • GHOST CAR TELEMETRY",
            description: "Solo driving against the clock on an empty circuit. Continuous flying laps record personal best lap times, telemetry delta splits, and ghost shadow cars for trajectory comparison.",
            rules: vec![
                "No opponent vehicle collisions or traffic interference.",
                "Flying start initiated as soon as vehicle crosses the start/finish timing gate.",
                "Track limits enforced: exceeding track boundary thresholds invalidates the current lap.",
                "Real-time ghost car mirrors the driver's fastest recorded lap trajectory.",
            ],
            scoring_summary: "Fastest single clean lap recorded on global circuit leaderboards and local Hall of Fame.",
        },
        CodexFormatInfo {
            id: "qualifying",
            name: "Timed Qualifying Session",
            tag: "GRID POSITION • TIMED WINDOW",
            description: "Pre-race timed session where drivers complete fast laps in clean air or light traffic to establish the starting grid order for the championship feature race.",
            rules: vec![
                "Session clock counts down (e.g. 5 to 10 minutes) with open track access.",
                "Best valid single flying lap determines pole position and starting grid slots.",
                "Drivers must manage track positioning for clean air and aerodynamic drafting.",
                "Tire temperature warmup lap required to bring compound into optimal grip window.",
            ],
            scoring_summary: "Establishes starting grid order for the feature race. Pole position awards 1 bonus point in select series.",
        },
        CodexFormatInfo {
            id: "stage_rally",
            name: "Stage Rally / Point-to-Point",
            tag: "POINT-TO-POINT • INTERVAL STARTS",
            description: "Individual point-to-point stages against the stopwatch across demanding surfaces with co-driver pace notes, blind crests, and narrow roadside trees.",
            rules: vec![
                "Staggered interval starts with fixed separation between competitors.",
                "Single continuous run from start timing gate to flying finish line.",
                "Surfaces dynamically transition between tarmac, loose gravel, mud, and hardpack snow.",
                "Exceeding track verges or striking obstacles incurs direct time penalties and vehicle damage.",
            ],
            scoring_summary: "Lowest cumulative elapsed stage time wins the rally event.",
        },
        CodexFormatInfo {
            id: "elimination",
            name: "Knockout / Elimination",
            tag: "SUDDEN DEATH • TAIL-END KNOCKOUT",
            description: "High-pressure survival format: at the end of each lap or elimination countdown timer, the driver in last position is instantly knocked out until only the champion remains.",
            rules: vec![
                "Starts with a full grid of 6 to 12 drivers battling in close quarters.",
                "Every lap or 30-second interval, the last-place vehicle is eliminated with a siren alert.",
                "Drivers must balance aggressive overtaking against defensive survival positioning.",
                "The final two surviving drivers battle in a head-to-head sprint for the trophy.",
            ],
            scoring_summary: "Points and prize purse awarded in reverse order of elimination.",
        },
    ];

    let joker_rule = CodexJokerRuleInfo {
        mandatory_laps: 1,
        time_penalty_sec: 30.0,
        applies_to: "Rallycross circuits with dedicated joker route (e.g., Lydden Hill, Höljes, Mettet, Dreux)",
        hud_indicator: "Cockpit HUD displays JOKER REQUIRED (yellow) until route traversed, then JOKER CLEARED (cyan)",
        strategy_notes: "The joker lap is an alternative, longer detour layout. Taking the joker early allows clean air and undercut potential; taking it late risks rejoining in traffic.",
    };

    let pit_service = CodexPitServiceInfo {
        repair_amount: PIT_STOP_REPAIR_AMOUNT,
        speed_limiter_kmh: 60.0,
        field_repair_chassis_cap: FIELD_REPAIR_CHASSIS_CAP,
        field_repair_engine_cap: FIELD_REPAIR_ENGINE_CAP,
        field_repair_suspension_cap: FIELD_REPAIR_SUSPENSION_CAP,
        phases: vec![
            CodexPitPhase {
                id: "in_transit",
                name: "Pit Lane Transit",
                description: "Vehicle enters the pit lane. The automatic pit speed limiter engages (60 km/h) to prevent speeding infractions.",
            },
            CodexPitPhase {
                id: "stationary_in_box",
                name: "Stationary Service Box",
                description: "Vehicle comes to a complete halt inside the designated pit stall box. Controls are locked while the pit crew refuels and applies 25% field repairs to chassis, engine, and suspension up to regulatory caps.",
            },
            CodexPitPhase {
                id: "service_complete",
                name: "Release & Exit Rejoin",
                description: "Service is complete. The pit crew signals release; driver regains full throttle control and accelerates to the pit exit line before blending safely back into race traffic.",
            },
        ],
    };

    let point_systems = vec![
        CodexPointSystemInfo {
            id: "fia_standard",
            name: "International Standard (Grand Touring Challenge)",
            description: "Official international championship points scale crediting the top 10 finishers. Encourages race-win pursuit with a steep podium gradient and rewards ultimate pace with a fastest-lap bonus point.",
            points_table: (1..=10)
                .map(|p| CodexPositionPoints {
                    position: p,
                    points: PointSystem::FiaStandard { fastest_lap_bonus: false }.points_for_position(p, false),
                })
                .collect(),
            fastest_lap_bonus: Some("+1 bonus point (requires top-10 finish)"),
            stage_win_bonus: None,
        },
        CodexPointSystemInfo {
            id: "motogp",
            name: "MotoGP World Championship",
            description: "Deep top-15 points distribution engineered for motorcycle grand prix and multi-class racing, keeping mid-pack battles mathematically meaningful through the final round.",
            points_table: (1..=15)
                .map(|p| CodexPositionPoints {
                    position: p,
                    points: PointSystem::MotoGp.points_for_position(p, false),
                })
                .collect(),
            fastest_lap_bonus: None,
            stage_win_bonus: None,
        },
        CodexPointSystemInfo {
            id: "classic_arcade",
            name: "Classic Arcade (Top 6)",
            description: "Nostalgic 6-place scoring with steep podium drop-offs modeled after 1990s arcade cabinets and GeneRally circuits. Zero points are awarded for 7th place and below.",
            points_table: (1..=6)
                .map(|p| CodexPositionPoints {
                    position: p,
                    points: PointSystem::ClassicArcade.points_for_position(p, false),
                })
                .collect(),
            fastest_lap_bonus: None,
            stage_win_bonus: None,
        },
        CodexPointSystemInfo {
            id: "nascar_cup",
            name: "Premier Stock Car Cup",
            description: "Authentic 40-driver field progression where every finishing spot yields points. Setting stage_win_bonus awards +10 points to the fastest lap / stage winner.",
            points_table: (1..=36)
                .map(|p| CodexPositionPoints {
                    position: p,
                    points: PointSystem::NascarCup { stage_win_bonus: false }.points_for_position(p, false),
                })
                .collect(),
            fastest_lap_bonus: None,
            stage_win_bonus: Some("+10 bonus points for stage win / fastest lap"),
        },
    ];

    let mut series = Vec::new();
    for &(preset_id, toml_str) in EMBEDDED_PRESETS {
        if let Ok(def) = SeriesDefinition::from_toml(toml_str) {
            if scope.includes_module(&def.series.module_id) {
                let rounds: Vec<CodexRoundInfo> = def.rounds.iter().map(|r| CodexRoundInfo {
                    order: r.order,
                    track_id: r.track_id.clone(),
                    name: r.name.clone(),
                    laps: r.laps,
                }).collect();

                let drivers: Vec<CodexSeriesDriver> = def.drivers.iter().map(|d| CodexSeriesDriver {
                    id: d.id.clone(),
                    name: d.name.clone(),
                    team: d.team.clone(),
                    is_player: d.is_player,
                    car_model_id: d.car_model_id.clone(),
                    country: d.country.clone(),
                    ai_style: d.ai_style.clone(),
                    ai_tier: d.ai_tier,
                }).collect();

                series.push(CodexSeriesPresetInfo {
                    id: preset_id.to_string(),
                    name: def.series.name,
                    description: def.series.description,
                    module_id: def.series.module_id,
                    tier: def.series.tier,
                    laps_per_round: def.series.laps_per_round,
                    scoring_system: def.scoring.system,
                    fastest_lap_bonus: def.scoring.fastest_lap_bonus,
                    stage_win_bonus: def.scoring.stage_win_bonus,
                    round_count: rounds.len(),
                    rounds,
                    driver_count: drivers.len(),
                    drivers,
                });
            }
        }
    }

    let all_ladders = vec![
        CodexCareerLadder {
            module_id: "gt",
            module_name: "Grand Touring Challenge & Endurance",
            tiers: vec![
                CodexCareerTierDetails { tier: 1, name: "Tier 1: GT4 Clubman Sprint", promotion_criteria: "Available immediately at career launch.", starter_car_id: Some("gt_yamato_hayate_t1"), starter_car_name: Some("Yamato Hayate GT T1"), car_cost_xp: 1_000, first_time_exploration_xp: 250, round_base_purse: 5_000, unlocked_tracks: vec!["red_bull_ring", "zandvoort", "nurburgring_gp", "portimao_gp", "montreal"] },
                CodexCareerTierDetails { tier: 2, name: "Tier 2: Continental GT3 Sprint Challenge", promotion_criteria: "Finish top 3 in GT4 championship + 2,000 XP in wallet.", starter_car_id: Some("gt_vandorn_arrowhead_t2"), starter_car_name: Some("Vandorn Arrowhead R T2"), car_cost_xp: 2_000, first_time_exploration_xp: 500, round_base_purse: 12_000, unlocked_tracks: vec!["monza", "silverstone", "catalunya"] },
                CodexCareerTierDetails { tier: 3, name: "Tier 3: Grand Touring Biturbo Masters", promotion_criteria: "Finish top 3 in GT3 championship + 3,000 XP in wallet.", starter_car_id: Some("gt_aquila_strale_t3"), starter_car_name: Some("Aquila Strale GT2 T3"), car_cost_xp: 3_000, first_time_exploration_xp: 750, round_base_purse: 25_000, unlocked_tracks: vec!["spa", "cota", "bahrain"] },
                CodexCareerTierDetails { tier: 4, name: "Tier 4: Endurance 90s Heritage Trophy", promotion_criteria: "Finish top 3 in GT2 championship + 4,000 XP in wallet.", starter_car_id: Some("gt_vandorn_aeromax_t4"), starter_car_name: Some("Vandorn Aeromax GT1 T4"), car_cost_xp: 4_000, first_time_exploration_xp: 1_000, round_base_purse: 55_000, unlocked_tracks: vec!["suzuka", "interlagos", "bathurst"] },
                CodexCareerTierDetails { tier: 5, name: "Tier 5: Hypercar World GP", promotion_criteria: "Finish top 3 in GT1 championship + 5,000 XP in wallet.", starter_car_id: Some("gt_vandorn_kronos_t5"), starter_car_name: Some("Vandorn Kronos Hypercar T5"), car_cost_xp: 5_000, first_time_exploration_xp: 1_250, round_base_purse: 120_000, unlocked_tracks: vec!["le_mans_sarthe", "monaco", "marina_bay"] },
            ],
        },
        CodexCareerLadder {
            module_id: "nascar",
            module_name: "Stock Car Championship",
            tiers: vec![
                CodexCareerTierDetails { tier: 1, name: "Tier 1: Street Stock Bullring", promotion_criteria: "Available immediately at career launch.", starter_car_id: Some("nascar_crossbow_montego_t1"), starter_car_name: Some("Crossbow Montego Street Stock T1"), car_cost_xp: 1_000, first_time_exploration_xp: 250, round_base_purse: 5_000, unlocked_tracks: vec!["martinsville_speedway", "bristol_motor_speedway", "eldora_speedway", "bowman_gray_stadium", "lucas_oil_irp"] },
                CodexCareerTierDetails { tier: 2, name: "Tier 2: Late Model Challenge", promotion_criteria: "Finish top 3 in Street Stock championship + 2,000 XP in wallet.", starter_car_id: Some("nascar_crossbow_saber_t2"), starter_car_name: Some("Crossbow Saber Late Model T2"), car_cost_xp: 2_000, first_time_exploration_xp: 500, round_base_purse: 12_000, unlocked_tracks: vec!["charlotte_motor_speedway", "darlington_raceway", "north_wilkesboro_speedway"] },
                CodexCareerTierDetails { tier: 3, name: "Tier 3: National Stock Car Tour", promotion_criteria: "Finish top 3 in Late Model championship + 3,000 XP in wallet.", starter_car_id: Some("nascar_crossbow_predator_t3"), starter_car_name: Some("Crossbow Predator Stock T3"), car_cost_xp: 3_000, first_time_exploration_xp: 750, round_base_purse: 25_000, unlocked_tracks: vec!["iowa_speedway", "watkins_glen_nascar", "road_america"] },
                CodexCareerTierDetails { tier: 4, name: "Tier 4: Pro Super Truck V8 Series", promotion_criteria: "Finish top 3 in National Stock Car championship + 4,000 XP in wallet.", starter_car_id: Some("nascar_crossbow_sierra_t4"), starter_car_name: Some("Crossbow Sierra Super Truck T4"), car_cost_xp: 4_000, first_time_exploration_xp: 1_000, round_base_purse: 55_000, unlocked_tracks: vec!["indianapolis_motor_speedway", "pocono_raceway", "chicago_street_course"] },
                CodexCareerTierDetails { tier: 5, name: "Tier 5: Premier Stock Car Cup", promotion_criteria: "Finish top 3 in Truck championship + 5,000 XP in wallet.", starter_car_id: Some("nascar_crossbow_manta_t5"), starter_car_name: Some("Crossbow Manta Silhouette T5"), car_cost_xp: 5_000, first_time_exploration_xp: 1_250, round_base_purse: 120_000, unlocked_tracks: vec!["daytona_superspeedway", "talladega_superspeedway", "phoenix_raceway"] },
            ],
        },
        CodexCareerLadder {
            module_id: "rally",
            module_name: "Rallycross & All-Terrain",
            tiers: vec![
                CodexCareerTierDetails { tier: 1, name: "Tier 1: Rally4 Grassroots Cup", promotion_criteria: "Available immediately at career launch.", starter_car_id: Some("rally_gallia_200_t1"), starter_car_name: Some("Gallia 200 Rally4 T1"), car_cost_xp: 1_000, first_time_exploration_xp: 250, round_base_purse: 5_000, unlocked_tracks: vec!["holjes_rx", "lydden_hill", "mettet_rx", "dreux_rx", "croft_rx"] },
                CodexCareerTierDetails { tier: 2, name: "Tier 2: Supercar Lites Trophy", promotion_criteria: "Finish top 3 in Rally4 championship + 2,000 XP in wallet.", starter_car_id: Some("rally_nordic_rx_lites_t2"), starter_car_name: Some("Nordic RX Supercar Lites T2"), car_cost_xp: 2_000, first_time_exploration_xp: 500, round_base_purse: 12_000, unlocked_tracks: vec!["hell_rx", "loheac_rx", "lavare_rx"] },
                CodexCareerTierDetails { tier: 3, name: "Tier 3: Continental RX Supercars", promotion_criteria: "Finish top 3 in Supercar Lites championship + 3,000 XP in wallet.", starter_car_id: Some("rally_vortek_turbo_quattro_t7"), starter_car_name: Some("Vortek Turbo Quattro Legend T7"), car_cost_xp: 3_000, first_time_exploration_xp: 750, round_base_purse: 25_000, unlocked_tracks: vec!["estering_rx", "montalegre_rx", "riga_rx"] },
                CodexCareerTierDetails { tier: 4, name: "Tier 4: World Rallycross Supercars", promotion_criteria: "Finish top 3 in Continental RX championship + 4,000 XP in wallet.", starter_car_id: Some("rally_gallia_lyon_t4"), starter_car_name: Some("Gallia Lyon WRX Supercar T4"), car_cost_xp: 4_000, first_time_exploration_xp: 1_000, round_base_purse: 55_000, unlocked_tracks: vec!["nyirad_rx", "kouvola_rx", "killarney_rx"] },
                CodexCareerTierDetails { tier: 5, name: "Tier 5: RX1e / Group E / Heritage", promotion_criteria: "Finish top 3 in World Rallycross championship + 5,000 XP in wallet.", starter_car_id: Some("rally_gallia_volt_t5"), starter_car_name: Some("Gallia Volt RX1e Supercar T5"), car_cost_xp: 5_000, first_time_exploration_xp: 1_250, round_base_purse: 120_000, unlocked_tracks: vec!["catalunya_rx", "lessay_rx", "essay_rx"] },
            ],
        },
        CodexCareerLadder {
            module_id: "kart",
            module_name: "Grassroots & Shifter Karting",
            tiers: vec![
                CodexCareerTierDetails { tier: 1, name: "Tier 1: Cadet 60cc Trophy", promotion_criteria: "Available immediately at career launch.", starter_car_id: Some("kart_blackline_cadet_t1"), starter_car_name: Some("Blackline Cadet 60 T1"), car_cost_xp: 1_000, first_time_exploration_xp: 250, round_base_purse: 5_000, unlocked_tracks: vec!["lonato", "genk", "wackersdorf", "laval_kart", "whilton_mill"] },
                CodexCareerTierDetails { tier: 2, name: "Tier 2: Junior OK-J National", promotion_criteria: "Finish top 3 in Cadet championship + 2,000 XP in wallet.", starter_car_id: Some("kart_verde_apex_t3"), starter_car_name: Some("Verde Kart Apex 125 OK T3"), car_cost_xp: 2_000, first_time_exploration_xp: 500, round_base_purse: 12_000, unlocked_tracks: vec!["sarno", "kristianstad", "seven_laghi"] },
                CodexCareerTierDetails { tier: 3, name: "Tier 3: KZ2 Shifter Pro", promotion_criteria: "Finish top 3 in Junior OK championship + 3,000 XP in wallet.", starter_car_id: Some("kart_rosso_corsa_t4"), starter_car_name: Some("Rosso Kart Corsa 125 KZ2 T4"), car_cost_xp: 3_000, first_time_exploration_xp: 750, round_base_purse: 25_000, unlocked_tracks: vec!["pfi", "franciacorta", "ampfing"] },
                CodexCareerTierDetails { tier: 4, name: "Tier 4: Superkart Division 2", promotion_criteria: "Finish top 3 in KZ2 championship + 4,000 XP in wallet.", starter_car_id: Some("vault_asahi_blade_runner"), starter_car_name: Some("Asahi Blade Runner Mower"), car_cost_xp: 4_000, first_time_exploration_xp: 1_000, round_base_purse: 55_000, unlocked_tracks: vec!["zuera", "silverstone_national_kart", "le_mans_kart"] },
                CodexCareerTierDetails { tier: 5, name: "Tier 5: Superkart World Series", promotion_criteria: "Finish top 3 in Superkart Div 2 championship + 5,000 XP in wallet.", starter_car_id: Some("kart_highland_eagle_t6"), starter_car_name: Some("Highland Eagle Superkart T6"), car_cost_xp: 5_000, first_time_exploration_xp: 1_250, round_base_purse: 120_000, unlocked_tracks: vec!["portimao_kart", "valencia_kart", "campillos"] },
            ],
        },
        CodexCareerLadder {
            module_id: "extreme_offroad",
            module_name: "Extreme Off-Road & Stunt Arenas",
            tiers: vec![
                CodexCareerTierDetails { tier: 1, name: "Tier 1: Desert Sand Sprint", promotion_criteria: "Available immediately at career launch.", starter_car_id: Some("offroad_laurentian_nomad_t1"), starter_car_name: Some("Laurentian Nomad Trophy Buggy T1"), car_cost_xp: 1_000, first_time_exploration_xp: 250, round_base_purse: 5_000, unlocked_tracks: vec!["sahara_dune_crossing", "dirt_figure_eight", "atacama_sand_basin", "glamis_dunes", "crandon_short_course"] },
                CodexCareerTierDetails { tier: 2, name: "Tier 2: Canyon Raid", promotion_criteria: "Finish top 3 in Sand Sprint championship + 2,000 XP in wallet.", starter_car_id: Some("offroad_desert_forge_truck_t2"), starter_car_name: Some("Desert Forge AWD Trophy Truck T2"), car_cost_xp: 2_000, first_time_exploration_xp: 500, round_base_purse: 12_000, unlocked_tracks: vec!["red_rock_canyon", "mud_slough_arena", "baja_500_desert_scrub"] },
                CodexCareerTierDetails { tier: 3, name: "Tier 3: Arctic Ice Challenge", promotion_criteria: "Finish top 3 in Canyon Raid championship + 3,000 XP in wallet.", starter_car_id: Some("offroad_sixstar_blizzard_t3"), starter_car_name: Some("Sixstar Blizzard Ice Racer T3"), car_cost_xp: 3_000, first_time_exploration_xp: 750, round_base_purse: 25_000, unlocked_tracks: vec!["arctic_frozen_lake", "alpine_snow_ridge", "rovaniemi_ice_ring"] },
                CodexCareerTierDetails { tier: 4, name: "Tier 4: Pro4 Mud Masters", promotion_criteria: "Finish top 3 in Arctic Challenge championship + 4,000 XP in wallet.", starter_car_id: Some("offroad_crossbow_ridge_t4"), starter_car_name: Some("Crossbow Ridge Heavy Mud Bogger T4"), car_cost_xp: 4_000, first_time_exploration_xp: 1_000, round_base_purse: 55_000, unlocked_tracks: vec!["supercross_stadium_arena", "gravel_quarry_chasm", "louisiana_mud_swampland"] },
                CodexCareerTierDetails { tier: 5, name: "Tier 5: Monster Colosseum", promotion_criteria: "Finish top 3 in Mud Masters championship + 5,000 XP in wallet.", starter_car_id: Some("offroad_havoc_tomb_raider_t5"), starter_car_name: Some("Havoc Tomb Raider Monster Crusher T5"), car_cost_xp: 5_000, first_time_exploration_xp: 1_250, round_base_purse: 120_000, unlocked_tracks: vec!["monster_colosseum", "glacier_crest_pass", "stunt_city_megastructure"] },
            ],
        },
        CodexCareerLadder {
            module_id: "autocross",
            module_name: "Continental Autocross",
            tiers: vec![
                CodexCareerTierDetails { tier: 1, name: "Tier 1: Cross Car Junior", promotion_criteria: "Available immediately at career launch.", starter_car_id: Some("classic_ax_mudlark"), starter_car_name: Some("Mudlark Cross Car 600"), car_cost_xp: 1_000, first_time_exploration_xp: 250, round_base_purse: 5_000, unlocked_tracks: vec!["ax_meadow_sprint", "ax_quarry_loop", "ax_forest_dash"] },
                CodexCareerTierDetails { tier: 2, name: "Tier 2: Cross Car Senior", promotion_criteria: "Finish top 3 in Junior Cross Car championship + 2,000 XP in wallet.", starter_car_id: Some("classic_ax_mudlark"), starter_car_name: Some("Mudlark Cross Car 600"), car_cost_xp: 2_000, first_time_exploration_xp: 500, round_base_purse: 12_000, unlocked_tracks: vec!["ax_nova_paka", "ax_matschenberg"] },
                CodexCareerTierDetails { tier: 3, name: "Tier 3: Buggy 1600 Championship", promotion_criteria: "Finish top 3 in Senior Cross Car championship + 3,000 XP in wallet.", starter_car_id: Some("classic_ax_mudlark"), starter_car_name: Some("Mudlark Cross Car 600"), car_cost_xp: 3_000, first_time_exploration_xp: 750, round_base_purse: 25_000, unlocked_tracks: vec!["ax_seelow", "ax_vilkyciai"] },
                CodexCareerTierDetails { tier: 4, name: "Tier 4: Touring Autocross Masters", promotion_criteria: "Finish top 3 in Buggy 1600 championship + 4,000 XP in wallet.", starter_car_id: Some("classic_ax_mudlark"), starter_car_name: Some("Mudlark Cross Car 600"), car_cost_xp: 4_000, first_time_exploration_xp: 1_000, round_base_purse: 55_000, unlocked_tracks: vec!["ax_st_georges", "ax_maggiore"] },
                CodexCareerTierDetails { tier: 5, name: "Tier 5: SuperBuggy World Series", promotion_criteria: "Finish top 3 in Touring Autocross championship + 5,000 XP in wallet.", starter_car_id: Some("classic_ax_mudlark"), starter_car_name: Some("Mudlark Cross Car 600"), car_cost_xp: 5_000, first_time_exploration_xp: 1_250, round_base_purse: 120_000, unlocked_tracks: vec!["ax_mollerussa", "ax_prerov"] },
            ],
        },
        CodexCareerLadder {
            module_id: "classic",
            module_name: "Classic Arcade",
            tiers: vec![
                CodexCareerTierDetails { tier: 1, name: "Tier 1: Retro Arcade Cup", promotion_criteria: "Available immediately at career launch.", starter_car_id: Some("classic_gt"), starter_car_name: Some("Apex GT Coupe"), car_cost_xp: 1_000, first_time_exploration_xp: 250, round_base_purse: 5_000, unlocked_tracks: vec!["gt_velocity_park", "gt_ridge_ring", "rx_quarry_sprint", "ax_meadow_sprint"] },
            ],
        },
    ];

    let ladders: Vec<CodexCareerLadder> = all_ladders
        .into_iter()
        .filter(|l| scope.includes_module(l.module_id))
        .collect();

    let xp_economy = CodexXpEconomyInfo {
        distance_divisor: 10.0,
        completion_multiplier: 2.0,
        first_time_bonus_per_tier: 250,
        car_cost_per_tier: 1_000,
        formulas: vec![
            "per_lap_xp = round_to_10(track_length_meters / 10.0)",
            "lap_xp = per_lap_xp * completed_laps",
            "completion_bonus = lap_xp (duplicates lap XP on race finish)",
            "first_time_exploration_bonus = round_to_10(tier * 250 XP)",
            "total_race_xp = lap_xp + completion_bonus + first_time_exploration_bonus",
            "car_purchase_cost = tier * 1,000 XP",
        ],
    };

    let repair_economy = CodexRepairEconomyInfo {
        purse_cap_share: REPAIR_PURSE_CAP_SHARE,
        safety_net_credit_limit: SAFETY_NET_CREDIT_LIMIT,
        safety_net_health: SAFETY_NET_HEALTH,
        formulas: vec![
            "cost_chassis = round_to_10(base_purse(tier) * 0.15 * (1.0 - H_chassis)^1.2)",
            "cost_engine = round_to_10(base_purse(tier) * 0.25 * k_powertrain * (1.0 - H_engine)^1.4)",
            "cost_suspension[i] = round_to_10(base_purse(tier) * 0.08 * k_archetype * (1.0 - H_susp[i])^1.2)",
            "net_repair_deduction = min(total_raw_damage, earned_purse * 0.40) [Sponsor pays the rest]",
            "anti_bankruptcy_safety_net = free repair to 50% health if wallet balance < 1,000 Credits",
        ],
    };

    let curriculum = AcademyLessonDef::default_curriculum()
        .into_iter()
        .map(|l| CodexAcademyLesson {
            id: l.id.slug(),
            index: l.id.index() + 1,
            title: l.title,
            description: l.description,
            track_slug: l.track_slug,
            car_slug: l.car_slug,
            gold_time_sec: l.gold_time_sec,
            silver_time_sec: l.silver_time_sec,
            bronze_time_sec: l.bronze_time_sec,
            bronze_credit_bounty: l.bronze_credit_bounty,
            silver_credit_bounty: l.silver_credit_bounty,
            gold_credit_bounty: l.gold_credit_bounty,
            base_xp_reward: l.base_xp_reward,
        })
        .collect();

    let licence_grades = vec![
        CodexLicenceGrade {
            grade: "ClassD",
            title: "Class D (National Grassroots)",
            badge: "CLASS D",
            description: "Entry-level accredited motorsport licence for grassroots cadet, junior, and shifter karting competitions.",
            required_for: vec!["kart", "classic"],
        },
        CodexLicenceGrade {
            grade: "ClassC",
            title: "Class C (Junior Competition)",
            badge: "CLASS C",
            description: "Junior competition licence authorizing participation in European Autocross cross-cars and mixed-surface Rallycross events.",
            required_for: vec!["autocross", "rally"],
        },
        CodexLicenceGrade {
            grade: "ClassB",
            title: "Class B (National Pro-Am)",
            badge: "CLASS B",
            description: "National Pro-Am credentials for heavy stock cars on high-speed ovals and extreme off-road desert trophy trucks.",
            required_for: vec!["nascar", "extreme_offroad"],
        },
        CodexLicenceGrade {
            grade: "ClassA",
            title: "Class A (International GT)",
            badge: "CLASS A",
            description: "International competition licence required for high-downforce GT4, GT3, GT2, and GT1 endurance machinery.",
            required_for: vec!["gt"],
        },
        CodexLicenceGrade {
            grade: "ClassS",
            title: "Class S (World Superlicense)",
            badge: "CLASS S",
            description: "The pinnacle of motorsport accreditation, granting access to state-of-the-art hybrid Endurance Hypercars and Apex Prototypes.",
            required_for: vec!["gt_hypercar", "apex_prototypes"],
        },
    ];

    let academy = CodexAcademyInfo {
        curriculum,
        max_permissible_impulse: 800.0,
        max_permissible_off_track_sec: 2.0,
        licence_grades,
    };

    let multiplayer = CodexMultiplayerInfo {
        min_players: 2,
        max_players: 8,
        simulation_rate_hz: 60,
        interpolation_delay_ms: (INTERP_DELAY_SEC * 1000.0).round() as u32,
        discovery: "UDP Broadcast on local subnet, port auto-negotiated",
        collision_modes: vec![
            CodexCollisionModeInfo {
                id: "full_sat_solid",
                name: "Solid Body (SAT)",
                description: "Full rigid-body Separating Axis Theorem (SAT) collision resolution between all competitor vehicles with authentic momentum transfer and spin dynamics.",
            },
            CodexCollisionModeInfo {
                id: "ghost_passing",
                name: "Ghost (Non-Contact)",
                description: "Competitor vehicles pass freely through each other without collision contact, preventing first-turn pile-ups and ensuring pure lap-time racing.",
            },
            CodexCollisionModeInfo {
                id: "verge_only",
                name: "Verge Only",
                description: "Vehicle-to-vehicle contact is disabled, but collisions remain fully active against track verges, curbs, barriers, and environmental props.",
            },
        ],
        host_steps: vec![
            "Navigate to Racing -> LAN Multiplayer in the in-game menu.",
            "Select 'Host LAN Lobby'. Choose your driver name, country flag, and preferred car.",
            "Select the official circuit, number of laps, and desired collision mode (Solid Body, Ghost, or Verge Only).",
            "Wait for local network racers to appear in the lobby roster slots (up to 8 players).",
            "When all drivers indicate 'Ready', press [Enter] / Gamepad [Start] to launch the synchronized race.",
        ],
        join_steps: vec![
            "Ensure all machines are connected to the same local Wi-Fi or Ethernet subnet.",
            "Navigate to Racing -> LAN Multiplayer -> 'Join LAN Lobby'.",
            "The client auto-discovers active host beacons via UDP broadcast.",
            "Select the host lobby from the discovered list and choose your vehicle and livery.",
            "Press [R] / Gamepad [X] to toggle 'Ready'. The game will synchronize clock and load the track upon host start.",
        ],
    };

    CodexRacingData {
        id: "racing",
        disciplines,
        formats,
        joker_rule,
        pit_service,
        point_systems,
        series,
        career: CodexCareerInfo {
            ladders,
            xp_economy,
            repair_economy,
        },
        academy,
        multiplayer,
    }
}

fn rivals_data() -> CodexRivalsData {
    let drivers: Vec<CodexDriverCharacterInfo> = DriverCharacter::all_across_modules()
        .into_iter()
        .map(|d| {
            let stats = d.resolve_stats(DriverTier::Pro);
            CodexDriverCharacterInfo {
                id: d.id,
                name: d.name,
                alias: d.alias,
                bio: d.bio,
                style: d.style.as_str(),
                preferred_car: d.preferred_car.title(),
                primary_color: [d.color_scheme.primary.r, d.color_scheme.primary.g, d.color_scheme.primary.b, d.color_scheme.primary.a],
                secondary_color: [d.color_scheme.secondary.r, d.color_scheme.secondary.g, d.color_scheme.secondary.b, d.color_scheme.secondary.a],
                accent_color: [d.color_scheme.helmet.r, d.color_scheme.helmet.g, d.color_scheme.helmet.b, d.color_scheme.helmet.a],
                stats: CodexDriverStatsInfo {
                    speed: (stats.speed * 100.0).round() / 100.0,
                    aggression: (stats.aggression * 100.0).round() / 100.0,
                    precision: (stats.precision * 100.0).round() / 100.0,
                    defense: (stats.defense * 100.0).round() / 100.0,
                },
                favorite_cars: d.favorite_cars.iter().map(|f| CodexDriverFavoriteCarInfo {
                    discipline: f.discipline,
                    tier: f.tier,
                    model_id: f.model_id,
                }).collect(),
            }
        })
        .collect();

    let driving_styles = vec![
        CodexDrivingStyleInfo {
            id: "smooth",
            name: "Smooth",
            description: "Prioritizes textbook geometric racing lines, gentle steering inputs, and momentum preservation. Minimizes unnecessary tire scrub to maintain high corner-exit speeds.",
            speed_mult: 1.00,
            aggression: 0.65,
            precision: 0.98,
            defense: 0.85,
            tactical_traits: vec![
                "Clips apex kerbs with millimeter accuracy.",
                "Executes trail-braking smoothly to maintain vehicle pitch balance.",
                "Rarely makes unforced slide errors; patient when trailing.",
            ],
        },
        CodexDrivingStyleInfo {
            id: "aggressive",
            name: "Aggressive",
            description: "Fearless wheel-to-wheel combatant who brakes at the absolute threshold, divebombs into braking zones, and forces opponents to compromise their line.",
            speed_mult: 1.02,
            aggression: 0.96,
            precision: 0.80,
            defense: 0.88,
            tactical_traits: vec![
                "Late-brakes deep into hairpins and chicanes.",
                "Uses curb bounces and track verges aggressively.",
                "Higher risk of lockups and corner-exit snap oversteer under pressure.",
            ],
        },
        CodexDrivingStyleInfo {
            id: "tenacious",
            name: "Tenacious",
            description: "Ironclad defensive specialist who positions their vehicle as wide as the track, fiercely protecting the inside line and frustrating overtake attempts.",
            speed_mult: 0.96,
            aggression: 0.82,
            precision: 0.88,
            defense: 0.98,
            tactical_traits: vec![
                "Anticipates opponent moves and covers the apex early.",
                "Exceptional composure when under intense rear bumper pressure.",
                "Relentless lap-time consistency over long championship stints.",
            ],
        },
        CodexDrivingStyleInfo {
            id: "calculating",
            name: "Calculating",
            description: "Telemetry-minded tactician who studies opponent delta splits, avoids low-percentage lunges, and ruthlessly capitalizes on mistakes ahead.",
            speed_mult: 1.00,
            aggression: 0.72,
            precision: 0.96,
            defense: 0.90,
            tactical_traits: vec![
                "Optimizes slip angles for maximum straight-line exit velocity.",
                "Capitalizes on competitor contact and errors instantly.",
                "Maintains disciplined tire wear and temperature management.",
            ],
        },
        CodexDrivingStyleInfo {
            id: "bold",
            name: "Bold",
            description: "Audacious cross-discipline daredevil known for spectacular overtakes around the outside, Scandinavian flicks, and high-slip angle car control.",
            speed_mult: 1.01,
            aggression: 0.92,
            precision: 0.78,
            defense: 0.80,
            tactical_traits: vec![
                "Willing to attempt passes where other drivers hesitate.",
                "High slip-angle tolerance with lightning counter-steer reflexes.",
                "Thrives on loose surfaces, dirt ruts, and jump landings.",
            ],
        },
        CodexDrivingStyleInfo {
            id: "balanced",
            name: "Balanced",
            description: "Well-rounded clubman racer blending solid single-lap pace, steady racecraft, clean overtakes, and dependable defensive awareness.",
            speed_mult: 0.97,
            aggression: 0.70,
            precision: 0.85,
            defense: 0.85,
            tactical_traits: vec![
                "Adaptable to varying track conditions and modalities.",
                "Clean, respectful wheel-to-wheel battles.",
                "Dependable performance across diverse performance tiers.",
            ],
        },
    ];

    let skill_tiers = [
        DriverTier::Rookie,
        DriverTier::Amateur,
        DriverTier::Contender,
        DriverTier::Pro,
        DriverTier::Legend,
    ]
    .into_iter()
    .map(|t| {
        let q = DriverQuality::for_tier(t);
        CodexSkillTierInfo {
            tier: t.to_u8(),
            name: t.title(),
            short_name: t.short_name(),
            tag: t.tag(),
            pace_limit: q.pace_limit,
            consistency: q.consistency,
            composure: q.composure,
            bell_curve_weights: t.bell_curve_weights(),
        }
    })
    .collect();

    let mistake_kinds = vec![
        CodexMistakeKindInfo {
            id: "late_brake",
            name: "Late Braking Lockup",
            description: "Over-estimates the car's braking threshold by 30–60% and enters the braking zone carrying excessive velocity, missing the turn-in point.",
            trigger_condition: "High pressure from pursuing cars or aggressive overtake attempts.",
            gameplay_impact: "Runs deep past the apex, leaving the door wide open for an undercut pass.",
            recovery_behavior: "Applies maximum steering angle and trail brakes hard to scrub speed.",
        },
        CodexMistakeKindInfo {
            id: "overdrive",
            name: "Corner Overdrive",
            description: "Enters the corner 8–25% above the tire grip limit, inducing understeer and washing out toward the track verge.",
            trigger_condition: "Carrying too much apex momentum in fast sweepers.",
            gameplay_impact: "Washes wide onto dirty track surfaces, grass, or curbs, losing exit speed.",
            recovery_behavior: "Lifts off the throttle abruptly and counter-steers to re-establish front grip.",
        },
        CodexMistakeKindInfo {
            id: "power_stab",
            name: "Power Oversteer Snap",
            description: "Applies full throttle and handbrake simultaneously on corner exit for 0.6–1.1s, overpowering the rear tires and snapping into an oversteer slide.",
            trigger_condition: "Low-gear corner exits under heavy acceleration.",
            gameplay_impact: "Violent rear slide that burns rear tire temperature and costs straight-line speed.",
            recovery_behavior: "Counter-steers hard to catch the slide before it transitions into a 360-degree spin.",
        },
        CodexMistakeKindInfo {
            id: "over_correct",
            name: "Steering Over-Correction",
            description: "Over-reacts to a minor lateral wiggle with 1.6x steering gain for 0.5s, inducing secondary tank-slapper oscillations.",
            trigger_condition: "Kerb strikes or surface transitions that perturb chassis roll.",
            gameplay_impact: "Erratic slalom weaving that slows the vehicle and destabilizes platform balance.",
            recovery_behavior: "Rapidly relaxes steering input back to center after 500 ms.",
        },
        CodexMistakeKindInfo {
            id: "cautious",
            name: "Hesitation & Early Lift",
            description: "Brakes 5–15 meters earlier than necessary or lifts off the throttle prematurely in apex compression.",
            trigger_condition: "Low driver composure under pressure or wet/low-grip surfaces.",
            gameplay_impact: "Gives away easy momentum to bolder drivers trailing closely behind.",
            recovery_behavior: "Smoothly reapplies throttle once confidence in front grip is confirmed.",
        },
    ];

    CodexRivalsData {
        id: "rivals",
        drivers,
        driving_styles,
        skill_tiers,
        mistake_kinds,
    }
}

fn to_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map(|s| s + "\n").map_err(|e| e.to_string())
}

fn envelope<T: Serialize>(scope: Scope, items: Vec<T>) -> Result<String, String> {
    to_json(&Envelope { schema_version: SCHEMA_VERSION, scope, items })
}

/// Builds every Codex data file as `(file name, JSON text)`, in a fixed order.
/// `repo_root` is used only to check which texture files exist.
pub fn export(scope: Scope, repo_root: &Path) -> Result<Vec<(&'static str, String)>, String> {
    // Group by module in Codex order; inside a module keep the game's own list order (stable sort).
    let module_rank = |id: &str| modules().iter().position(|m| m.id() == id).unwrap_or(usize::MAX);

    let mut models: Vec<&'static RealCarModel> = CLASSIC_ARCADE_CARS
        .iter()
        .chain(ALL_REAL_CARS.iter())
        .filter(|c| scope.includes_module(c.module_id))
        .collect();
    models.sort_by_key(|c| module_rank(c.module_id));
    let cars: Vec<CodexCar> = models.iter().map(|c| car(c, repo_root)).collect();
    let platforms = platforms(&models);

    let mut circuits = Vec::new();
    for entry in catalog::circuits().iter().filter(|c| scope.includes_module(c.module)) {
        let track = entry.load().map_err(|e| format!("circuit {}/{}: {:?}", entry.module, entry.id, e))?;
        circuits.push(circuit(entry, &track, repo_root));
    }
    circuits.sort_by_key(|c| module_rank(c.module));

    let surfaces: Vec<CodexSurface> = SurfaceType::ALL.into_iter().map(surface).collect();

    let modules: Vec<CodexModule> = modules()
        .into_iter()
        .filter(|m| scope.includes_module(m.id()))
        .map(|m| CodexModule {
            id: m.id(),
            title: m.title(),
            subtitle: m.subtitle(),
            launch: LAUNCH_MODULES.contains(&m.id()),
            cars: cars.iter().filter(|c| c.module == m.id()).count(),
            circuits: circuits.iter().filter(|c| c.module == m.id()).count(),
        })
        .collect();

    let racing = racing_data(scope, &cars, &circuits);
    let rivals = rivals_data();

    Ok(vec![
        ("modules.json", envelope(scope, modules)?),
        ("cars.json", envelope(scope, cars)?),
        ("circuits.json", envelope(scope, circuits)?),
        ("surfaces.json", envelope(scope, surfaces)?),
        ("suspension.json", envelope(scope, suspension_archetypes(&platforms))?),
        ("tyres.json", envelope(scope, compounds(&platforms))?),
        ("drivetrain.json", envelope(scope, engine_placements(&platforms))?),
        ("damage.json", envelope(scope, vec![damage_model()])?),
        ("chassis.json", envelope(scope, platforms)?),
        ("controls.json", envelope(scope, vec![controls_data()])?),
        ("driving.json", envelope(scope, vec![driving_data()])?),
        ("hud.json", envelope(scope, vec![hud_data()])?),
        ("racing.json", envelope(scope, vec![racing])?),
        ("rivals.json", envelope(scope, vec![rivals])?),
    ])
}
