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
    CarConfig, ChassisSkeleton, DifferentialType, EnginePlacement, SuspensionArchetype, SuspensionConfig,
};
use tdrace_core::physics::surface::{CompoundId, SurfaceType};
use tdrace_core::physics::tire::{normalized_grip_curve, TireCompoundConfig, WheelAssembly};
use tdrace_core::track::{Track, TrackKind};

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
    pub brake_share: f32,
    pub drive_share: f32,
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
            reference: texture(repo_root, format!("vehicles/references/{m}/{id}.jpg")),
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
                brake_share: w.brake_bias_factor,
                drive_share: w.drive_torque_factor,
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
    ])
}
