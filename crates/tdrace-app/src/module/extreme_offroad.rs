use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::{
    alpine_snow_ridge, arctic_frozen_lake, atacama_sand_basin, baja_500_desert_scrub,
    crandon_short_course, dirt_figure_eight, glacier_crest_pass, glamis_sand_dunes,
    gravel_quarry_chasm, louisiana_mud_swampland, monster_colosseum, mud_slough_arena,
    red_rock_canyon, rovaniemi_ice_ring, sahara_dune_crossing, stunt_city_megastructure,
    supercross_stadium_arena,
};

use super::{
    EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition,
    VehicleVisualType,
};
use crate::ai::{DriverCharacter, DriverFavoriteCar, DriverPersonalityOffsets, DrivingStyle};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// Extreme Off-Road & Stunt Arenas Game Module.
///
/// Features the 300 BHP Sand Rail Buggy, 15 specialized off-road circuits, mud sloughs,
/// arctic ice lakes, supercross stadiums, and multi-level stunt arenas.
pub struct ExtremeOffRoadModule;

impl ExtremeOffRoadModule {
    pub fn new() -> Self {
        Self
    }

    /// 300 BHP Sand Rail Buggy Preset (Ultralight Chromoly Cage, RWD, Paddle Tires, Long-Travel).
    pub fn car_sand_rail() -> CarConfig {
        CarConfig::sand_rail()
    }

    /// Dirt Figure Eight track configured for Extreme Off-Road with 12 grid slots.
    pub fn track_dirt_figure_eight() -> tdrace_core::track::Track {
        let mut track = dirt_figure_eight();
        let (spacing, stagger) = track.default_grid_spacing_and_stagger();
        track.auto_generate_grid(12, spacing, stagger);
        track.module_id = Some("extreme_offroad".to_string());
        track.modules = vec!["extreme_offroad".to_string()];
        track
    }
}

impl Default for ExtremeOffRoadModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModule for ExtremeOffRoadModule {
    fn id(&self) -> &'static str {
        "extreme_offroad"
    }

    fn title(&self) -> &'static str {
        "EXTREME OFF-ROAD & STUNT ARENAS"
    }

    fn subtitle(&self) -> &'static str {
        "Baja Deserts, Ice Lakes, Supercross Triples & Stunt Arenas"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(1.0, 0.40, 0.05, 1.0), // Baja Danger Orange
            secondary_accent: Color::new(0.15, 0.85, 1.0, 1.0), // Electric Cyan / Ice Blue
            header_badge: "EXTREME OFF-ROAD & STUNT ARENAS",
            background_tint: Color::new(0.08, 0.05, 0.03, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![VehicleModelDefinition {
            id: "sand_rail_buggy",
            name: "300 BHP Sand Rail Buggy",
            tag: "300 BHP RWD ULTRALIGHT",
            description: "Ultralight chromoly tube chassis, 300 BHP rear turbo boxer, paddle tires, and long-travel off-road suspension.",
            config: Self::car_sand_rail(),
            visual_type: VehicleVisualType::SandRail {
                lightbar: true,
                whip_antenna: true,
                paddle_tires: true,
            },
            stats: (0.88, 0.96, 0.82, 0.94),
            default_schemes: vec![
                CarColorScheme::from_index(3), // Baja Danger Orange
                CarColorScheme::from_index(1), // Electric Cyan
                CarColorScheme::from_index(6), // Matte Black Stealth
                CarColorScheme::from_index(2), // Racing Red
            ],
            audio_profile: Some(EngineAudioProfile::sand_rail_boxer()),
        }]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "sand_rail_buggy"
    }

    fn default_off_track_surface(&self) -> SurfaceType {
        SurfaceType::Dirt
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        vec![
            TrackDefinition {
                id: "sahara_dune_crossing",
                title: "Sahara Dune Crossing",
                tag: "DESERT RAID",
                description: "High-speed sweeping desert crossing over cresting sand dunes.",
                category: "Desert Raid",
                default_laps: 3,
                generator: sahara_dune_crossing,
            },
            TrackDefinition {
                id: "dirt_figure_eight",
                title: "Dirt Figure Eight",
                tag: "STADIUM FIGURE 8",
                description: "High-speed dirt figure-eight crossover with twin jumps and 18-degree banked outer berms.",
                category: "Stunt Arenas",
                default_laps: 5,
                generator: Self::track_dirt_figure_eight,
            },
            TrackDefinition {
                id: "atacama_sand_basin",
                title: "Atacama Sand Basin",
                tag: "HIGH-SPEED BASIN",
                description: "Massive high-speed desert basin with sweeping sand curves and cresting jumps.",
                category: "Desert Raid",
                default_laps: 3,
                generator: atacama_sand_basin,
            },
            TrackDefinition {
                id: "red_rock_canyon",
                title: "Red Rock Canyon",
                tag: "CANYON RAID",
                description: "Narrow technical gorge between red sandstone towers with rough dirt trails and hairpin climbs.",
                category: "Desert Raid",
                default_laps: 4,
                generator: red_rock_canyon,
            },
            TrackDefinition {
                id: "mud_slough_arena",
                title: "Mud Slough Arena",
                tag: "MUD BOWL ARENA",
                description: "Enclosed stadium mud bowl arena with deep viscous mud ruts, raised dirt berms and tabletop jumps.",
                category: "Mud & Quarry",
                default_laps: 3,
                generator: mud_slough_arena,
            },
            TrackDefinition {
                id: "baja_500_desert_scrub",
                title: "Baja 500 Desert Scrub",
                tag: "BAJA ENDURO",
                description: "Punishing open desert enduro course across arid scrubland, silt flats, washboard whoops, and high-speed jumps.",
                category: "Desert Raid",
                default_laps: 3,
                generator: baja_500_desert_scrub,
            },
            TrackDefinition {
                id: "arctic_frozen_lake",
                title: "Arctic Frozen Lake",
                tag: "ICE ARENA",
                description: "Wide-open frozen glacial lake arena with slick blue ice, compacted snow banks, and perimeter snow berms.",
                category: "Arctic Frost",
                default_laps: 5,
                generator: arctic_frozen_lake,
            },
            TrackDefinition {
                id: "alpine_snow_ridge",
                title: "Alpine Snow Ridge",
                tag: "SNOW RIDGE",
                description: "Sub-zero mountain climb along high snow ridges, icy switchbacks, and sheer cliff edges.",
                category: "Arctic Frost",
                default_laps: 4,
                generator: alpine_snow_ridge,
            },
            TrackDefinition {
                id: "rovaniemi_ice_ring",
                title: "Rovaniemi Ice Ring",
                tag: "FINNISH ICE RING",
                description: "Finnish high-speed ice racing circuit with packed snow chicanes and high-velocity drift arcs.",
                category: "Arctic Frost",
                default_laps: 4,
                generator: rovaniemi_ice_ring,
            },
            TrackDefinition {
                id: "supercross_stadium_arena",
                title: "Supercross Stadium Arena",
                tag: "SUPERCROSS ARENA",
                description: "Indoor supercross colosseum featuring rhythmic triple jumps, whoop sections, and banked bowl turns.",
                category: "Stunt Arenas",
                default_laps: 5,
                generator: supercross_stadium_arena,
            },
            TrackDefinition {
                id: "gravel_quarry_chasm",
                title: "Gravel Quarry Chasm",
                tag: "QUARRY CHASM",
                description: "Multi-tiered industrial quarry chasm with vertical drops, loose gravel slides, and rock walls.",
                category: "Mud & Quarry",
                default_laps: 4,
                generator: gravel_quarry_chasm,
            },
            TrackDefinition {
                id: "louisiana_mud_swampland",
                title: "Louisiana Mud Swampland",
                tag: "SWAMP BASIN",
                description: "Treacherous bayou basin featuring deep mud bogs, slippery cypress roots, and submerged dirt roads.",
                category: "Mud & Quarry",
                default_laps: 4,
                generator: louisiana_mud_swampland,
            },
            TrackDefinition {
                id: "monster_colosseum",
                title: "Monster Colosseum",
                tag: "COLOSSEUM ARENA",
                description: "Massive open-floor monster truck arena with multiple crossing ramps, mud pits, and perimeter grandstands.",
                category: "Stunt Arenas",
                default_laps: 3,
                generator: monster_colosseum,
            },
            TrackDefinition {
                id: "glacier_crest_pass",
                title: "Glacier Crest Pass",
                tag: "GLACIAL PASS",
                description: "Treacherous high-altitude circuit over blue glacial ice crevasses, frozen tunnels, and blinding snow ridges.",
                category: "Arctic Frost",
                default_laps: 4,
                generator: glacier_crest_pass,
            },
            TrackDefinition {
                id: "stunt_city_megastructure",
                title: "Stunt City Megastructure",
                tag: "STUNT MEGASTRUCTURE",
                description: "Colossal multi-level concrete and asphalt stunt arena with high-flyer ramps, elevated cross-bridges, and drift bowls.",
                category: "Stunt Arenas",
                default_laps: 3,
                generator: stunt_city_megastructure,
            },
            TrackDefinition {
                id: "glamis_dunes",
                title: "Glamis Imperial Sand Dunes",
                tag: "DESERT RAID",
                description: "Open California sand bowl with natural razorback dune crests and sweeping high-speed bowls.",
                category: "Desert Raid",
                default_laps: 3,
                generator: glamis_sand_dunes,
            },
            TrackDefinition {
                id: "crandon_short_course",
                title: "Crandon International Off-Road",
                tag: "SHORT COURSE",
                description: "The Big House: iconic Wisconsin short-course track with high-speed clay straights and tabletop jumps.",
                category: "Mud & Quarry",
                default_laps: 3,
                generator: crandon_short_course,
            },
        ]
    }

    fn default_track_id(&self) -> &'static str {
        "sahara_dune_crossing"
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        vec![
            DriverCharacter {
                id: "wyatt_cole",
                name: "Wyatt Cole",
                alias: "Dust Devil",
                bio: "Baja 1000 desert raid legend who rides dune crests at maximum boost with fearless throttle control.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(3),
                offsets: DriverPersonalityOffsets::new(0.01, 0.1, 0.01, 0.00, 0.03, -0.2, 0.02),
                style: DrivingStyle::Bold,
                favorite_cars: WYATT_COLE_FAVORITES,
            },
            DriverCharacter {
                id: "jaxson_rivera",
                name: "Jaxson Rivera",
                alias: "Baja King",
                bio: "Undisputed King of the Baja scrub. Masters rhythm sections, washboard whoops, and high-speed silt beds.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(1),
                offsets: DriverPersonalityOffsets::new(0.00, 0.2, 0.00, -0.01, 0.05, -0.3, 0.03),
                style: DrivingStyle::Balanced,
                favorite_cars: JAXSON_RIVERA_FAVORITES,
            },
            DriverCharacter {
                id: "astrid_lindholm",
                name: "Astrid Lindholm",
                alias: "Ice Queen",
                bio: "Scandinavian ice pilot with ice in her veins, executing pinpoint drift transitions across frozen lakes and snowbanks.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(8),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.00, 0.01, 0.01, 0.0, 0.01),
                style: DrivingStyle::Smooth,
                favorite_cars: ASTRID_LINDHOLM_FAVORITES,
            },
            DriverCharacter {
                id: "bubba_beauregard",
                name: "Bubba Beauregard",
                alias: "Mud Slinger",
                bio: "Deep South swamp buggy veteran who powers through bottomless clay ruts and mud pits without ever getting bogged down.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(5),
                offsets: DriverPersonalityOffsets::new(-0.03, 0.1, 0.01, -0.02, 0.08, -0.5, 0.01),
                style: DrivingStyle::Tenacious,
                favorite_cars: BUBBA_BEAUREGARD_FAVORITES,
            },
            DriverCharacter {
                id: "travis_mcgrath",
                name: "Travis McGrath",
                alias: "Nitro",
                bio: "Freestyle stunt icon and stadium supercross pioneer who attacks triple jumps and megastructures with fearless aerial acrobatics.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(2),
                offsets: DriverPersonalityOffsets::new(0.01, 0.2, 0.01, 0.01, 0.03, -0.2, 0.02),
                style: DrivingStyle::Bold,
                favorite_cars: TRAVIS_MCGRATH_FAVORITES,
            },
            DriverCharacter {
                id: "roxie_vance",
                name: "Roxie Vance",
                alias: "Rock Hound",
                bio: "Red Rock canyon crawler renowned for surgical apex placement through broken bedrock and narrow sandstone chasms.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(4),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.1, 0.00, 0.01, 0.02, -0.2, 0.01),
                style: DrivingStyle::Calculating,
                favorite_cars: ROXIE_VANCE_FAVORITES,
            },
            DriverCharacter {
                id: "sven_lindqvist",
                name: "Sven Lindqvist",
                alias: "Blizzard",
                bio: "Rovaniemi ice ring champion with unmatched throttle feathering and low-friction drift control in blinding snowstorms.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(6),
                offsets: DriverPersonalityOffsets::new(0.00, 0.1, 0.00, -0.01, 0.04, -0.3, 0.01),
                style: DrivingStyle::Tenacious,
                favorite_cars: SVEN_LINDQVIST_FAVORITES,
            },
            DriverCharacter {
                id: "cruz_morales",
                name: "Cruz Morales",
                alias: "Chasm Jumper",
                bio: "Gravel quarry daredevil who launches massive vertical drops and rides high gravel berms at terminal velocity.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(7),
                offsets: DriverPersonalityOffsets::new(0.01, 0.2, 0.01, -0.01, 0.03, -0.2, 0.02),
                style: DrivingStyle::Aggressive,
                favorite_cars: CRUZ_MORALES_FAVORITES,
            },
            DriverCharacter {
                id: "dakota_black",
                name: "Dakota Black",
                alias: "Canyon Hawg",
                bio: "Appalachian rock-crawling daredevil whose brute-force line choices and high-ground clearance conquer sheer granite shelves.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.18, 0.18, 0.20, 1.0),
                    macroquad::color::Color::new(0.85, 0.40, 0.10, 1.0),
                    macroquad::color::Color::new(0.95, 0.95, 0.95, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.00, 0.1, 0.01, -0.01, 0.02, -0.2, 0.01),
                style: DrivingStyle::Aggressive,
                favorite_cars: DAKOTA_BLACK_FAVORITES,
            },
            DriverCharacter {
                id: "colton_haze",
                name: "Colton Haze",
                alias: "Sandstorm",
                bio: "Nevada desert raider famous for wide-open throttle jumps across dry lakebeds and aggressive roosting on switchbacks.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.88, 0.75, 0.25, 1.0),
                    macroquad::color::Color::new(0.20, 0.45, 0.25, 1.0),
                    macroquad::color::Color::new(0.10, 0.10, 0.12, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.00, 0.2, 0.01, -0.01, 0.05, -0.3, 0.01),
                style: DrivingStyle::Balanced,
                favorite_cars: COLTON_HAZE_FAVORITES,
            },
            DriverCharacter {
                id: "elise_roux",
                name: "Elise Roux",
                alias: "Alpine Lynx",
                bio: "French rally-raid pioneer with exceptional suspension damping feel and laser-focused precision on mountain ridges.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.10, 0.70, 0.85, 1.0),
                    macroquad::color::Color::new(0.95, 0.95, 0.95, 1.0),
                    macroquad::color::Color::new(0.10, 0.15, 0.30, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.01, 0.2, 0.00, 0.00, 0.02, -0.1, 0.02),
                style: DrivingStyle::Smooth,
                favorite_cars: ELISE_ROUX_FAVORITES,
            },
            DriverCharacter {
                id: "diego_valdez",
                name: "Diego Valdez",
                alias: "Trophy King",
                bio: "Mexican SCORE champion who masters high-speed whoops sections, riverbed boulder hops, and dusk-to-dawn desert endurance.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.85, 0.15, 0.30, 1.0),
                    macroquad::color::Color::new(0.95, 0.85, 0.15, 1.0),
                    macroquad::color::Color::new(0.10, 0.55, 0.30, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.1, 0.01, 0.00, 0.03, -0.2, 0.02),
                style: DrivingStyle::Calculating,
                favorite_cars: DIEGO_VALDEZ_FAVORITES,
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::Championship {
                name: "Extreme Off-Road World Series".to_string(),
                point_system: PointSystem::FiaStandard { fastest_lap_bonus: false },
                track_ids: vec![
                    "sahara_dune_crossing".to_string(),
                    "dirt_figure_eight".to_string(),
                    "atacama_sand_basin".to_string(),
                    "red_rock_canyon".to_string(),
                    "mud_slough_arena".to_string(),
                    "baja_500_desert_scrub".to_string(),
                    "arctic_frozen_lake".to_string(),
                    "alpine_snow_ridge".to_string(),
                    "rovaniemi_ice_ring".to_string(),
                    "supercross_stadium_arena".to_string(),
                    "gravel_quarry_chasm".to_string(),
                    "louisiana_mud_swampland".to_string(),
                    "monster_colosseum".to_string(),
                    "glacier_crest_pass".to_string(),
                    "stunt_city_megastructure".to_string(),
                ],
                laps_per_round: 3,
            },
            TournamentFormat::StageRally {
                name: "Desert & Ice Raid Tour".to_string(),
                stage_track_ids: vec![
                    "sahara_dune_crossing".to_string(),
                    "atacama_sand_basin".to_string(),
                    "red_rock_canyon".to_string(),
                    "baja_500_desert_scrub".to_string(),
                    "arctic_frozen_lake".to_string(),
                    "alpine_snow_ridge".to_string(),
                    "rovaniemi_ice_ring".to_string(),
                    "glacier_crest_pass".to_string(),
                    "gravel_quarry_chasm".to_string(),
                ],
            },
            TournamentFormat::QuickRace {
                default_laps: 3,
                default_bots: 7,
            },
            TournamentFormat::TimeAttack,
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::sand_rail_boxer()
    }
}

const WYATT_COLE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_sand_rail_buggy"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_baja_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_subaru_ice_racer"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_mega_mud_truck"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_grave_crusher"),
];

const JAXSON_RIVERA_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_polaris_rzr_pro_r"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_bettantown_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_lancer_evo_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_chevy_k30_mud_bogger"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_max_d_monster"),
];

const ASTRID_LINDHOLM_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_vw_sand_rail"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_mason_awd_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_audi_quattro_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_ford_f250_high_riser"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_bigfoot_crusher"),
];

const BUBBA_BEAUREGARD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_sand_rail_buggy"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_baja_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_subaru_ice_racer"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_mega_mud_truck"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_grave_crusher"),
];

const TRAVIS_MCGRATH_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_polaris_rzr_pro_r"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_bettantown_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_lancer_evo_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_chevy_k30_mud_bogger"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_max_d_monster"),
];

const ROXIE_VANCE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_vw_sand_rail"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_mason_awd_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_audi_quattro_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_ford_f250_high_riser"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_bigfoot_crusher"),
];

const SVEN_LINDQVIST_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_sand_rail_buggy"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_baja_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_audi_quattro_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_mega_mud_truck"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_grave_crusher"),
];

const CRUZ_MORALES_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_polaris_rzr_pro_r"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_bettantown_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_lancer_evo_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_chevy_k30_mud_bogger"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_max_d_monster"),
];

const DAKOTA_BLACK_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_vw_sand_rail"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_mason_awd_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_subaru_ice_racer"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_mega_mud_truck"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_bigfoot_crusher"),
];

const COLTON_HAZE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_sand_rail_buggy"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_baja_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_lancer_evo_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_chevy_k30_mud_bogger"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_grave_crusher"),
];

const ELISE_ROUX_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_polaris_rzr_pro_r"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_mason_awd_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_audi_quattro_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_ford_f250_high_riser"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_max_d_monster"),
];

const DIEGO_VALDEZ_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_vw_sand_rail"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_bettantown_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_subaru_ice_racer"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_mega_mud_truck"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_bigfoot_crusher"),
];
