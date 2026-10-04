//! TDRace Codex data export (specs/087_tdrace_codex_unified_game_encyclopedia_and_technical_reference_portal.md).
//!
//! Serialises the game's own data for the Codex web portal (`portals/codex`). Every number comes from the
//! same catalogue, presets and functions the game uses; nothing is re-computed here.
//! The `export_codex` binary writes the files; `tests/codex_export_tests.rs` checks that the committed copy
//! in `portals/shared/data/codex/` is fresh.

use std::path::Path;

use serde::Serialize;
use tdrace_core::catalog;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::{Track, TrackKind};

use crate::audio::EngineSoundType;
use crate::catalog::{get_tier_name, RealCarModel, ALL_REAL_CARS, CLASSIC_ARCADE_CARS};
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

    let mut cars: Vec<CodexCar> = CLASSIC_ARCADE_CARS
        .iter()
        .chain(ALL_REAL_CARS.iter())
        .filter(|c| scope.includes_module(c.module_id))
        .map(|c| car(c, repo_root))
        .collect();
    cars.sort_by_key(|c| module_rank(c.module));

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
    ])
}
