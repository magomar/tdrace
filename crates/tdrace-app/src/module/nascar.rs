use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::{
    bristol_motor_speedway, daytona_superspeedway, talladega_superspeedway, watkins_glen_nascar,
};

use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::{BotProfile, DriverCharacter, DriverStats};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// NASCAR Cup Series & Trans-Am TA1 Game Module.
pub struct NascarGameModule;

impl NascarGameModule {
    pub fn new() -> Self {
        Self
    }

    /// 850 BHP NASCAR Cup Next-Gen Pushrod V8 Spec
    pub fn car_stock_car() -> CarConfig {
        CarConfig::stock_car_ta1()
    }

    /// 850 BHP Trans-Am TA1 Spaceframe V8 Road Course Spec
    pub fn car_trans_am() -> CarConfig {
        CarConfig::trans_am_ta1()
    }
}

impl Default for NascarGameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModule for NascarGameModule {
    fn id(&self) -> &'static str {
        "nascar"
    }

    fn title(&self) -> &'static str {
        "NASCAR CUP SERIES & TRANS-AM TA1"
    }

    fn subtitle(&self) -> &'static str {
        "850 BHP V8 Stock Cars, Pack Drafting, 320 km/h Superspeedways & Trans-Am Road Courses"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(1.0, 0.82, 0.08, 1.0), // Golden Yellow
            secondary_accent: Color::new(0.06, 0.42, 0.92, 1.0), // Daytona Blue
            header_badge: "NASCAR CUP SERIES",
            background_tint: Color::new(0.08, 0.07, 0.06, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![
            VehicleModelDefinition {
                id: "nascar_cup_v8",
                name: "NASCAR Cup Next-Gen V8",
                tag: "850 BHP PUSHROD V8",
                description: "Modern NASCAR Cup Series stock car: 850 BHP naturally aspirated 5.9L pushrod V8, 1260 kg, decklid blade ducktail spoiler, roof escape flaps, 320 km/h superspeedway package.",
                config: Self::car_stock_car(),
                visual_type: VehicleVisualType::StockCar {
                    tall_wing: false,
                    roof_fins: true,
                    window_net: true,
                },
                stats: (0.97, 0.90, 0.86, 0.88),
                default_schemes: vec![
                    CarColorScheme::stock_car_daytona_blue(),
                    CarColorScheme::stock_car_racing_red(),
                    CarColorScheme::stock_car_sunset_orange(),
                    CarColorScheme::stock_car_intimidator_black(),
                    CarColorScheme::stock_car_carolina_blue(),
                ],
            },
            VehicleModelDefinition {
                id: "trans_am_ta1",
                name: "Trans-Am TA1 Spaceframe V8",
                tag: "850 BHP SPACEFRAME",
                description: "Pure American road racing silhouette monster: tube-frame chassis, high-mount carbon GT wing, lightweight bodywork, quick-ratio steering, side boom tubes.",
                config: Self::car_trans_am(),
                visual_type: VehicleVisualType::StockCar {
                    tall_wing: true,
                    roof_fins: false,
                    window_net: true,
                },
                stats: (0.95, 0.92, 0.91, 0.85),
                default_schemes: vec![
                    CarColorScheme::stock_car_intimidator_black(),
                    CarColorScheme::stock_car_sunset_orange(),
                    CarColorScheme::stock_car_racing_red(),
                    CarColorScheme::stock_car_daytona_blue(),
                ],
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "nascar_cup_v8"
    }

    fn default_off_track_surface(&self) -> SurfaceType {
        SurfaceType::Grass
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        vec![
            TrackDefinition {
                id: "daytona_superspeedway",
                title: "Daytona International Speedway",
                tag: "SUPERSPEEDWAY 31° BANK",
                description: "The World Center of Racing: 4.0 km tri-oval featuring 31° high banks, 18° tri-oval frontstretch, and intense high-speed pack drafting.",
                category: "Superspeedway",
                default_laps: 10,
                generator: daytona_superspeedway,
            },
            TrackDefinition {
                id: "talladega_superspeedway",
                title: "Talladega Superspeedway",
                tag: "SUPERSPEEDWAY 33° BANK",
                description: "The biggest, fastest superspeedway in motorsport: 4.3 km tri-oval with extreme 33° banking where slipstream draft slingshots decide victory.",
                category: "Superspeedway",
                default_laps: 10,
                generator: talladega_superspeedway,
            },
            TrackDefinition {
                id: "watkins_glen_nascar",
                title: "Watkins Glen International",
                tag: "NASCAR ROAD COURSE",
                description: "Historic 5.4 km upstate New York road course: the uphill Esses, Inner Loop Bus Stop chicane, high-speed Carousel, and Boot complex.",
                category: "Road Course",
                default_laps: 6,
                generator: watkins_glen_nascar,
            },
            TrackDefinition {
                id: "bristol_motor_speedway",
                title: "Bristol Motor Speedway",
                tag: "SHORT TRACK 28° BANK",
                description: "The Last Great Colosseum: 0.85 km high-banked concrete bowl with 28° turns, brutal bumper-to-bumper contact, and lightning fast 15-second laps.",
                category: "Short Track",
                default_laps: 15,
                generator: bristol_motor_speedway,
            },
        ]
    }

    fn default_track_id(&self) -> &'static str {
        "daytona_superspeedway"
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        vec![
            DriverCharacter {
                id: "dale_vance",
                name: "Dale 'The Intimidator' Vance",
                alias: "The Intimidator",
                bio: "Feared black #3 stock car legend. Master of the aggressive bumper tap, high-speed drafting lock, and relentless high-line intimidation.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_intimidator_black(),
                profile: BotProfile {
                    name: "Dale 'The Intimidator' Vance",
                    lookahead_time: 0.34,
                    speed_factor: 1.05,
                    steering_kp: 2.8,
                    steering_kd: 0.08,
                    brake_margin: 0.95,
                    aggression: 0.98,
                    avoidance_distance: 4.8,
                },
                stats: DriverStats {
                    speed: 0.98,
                    aggression: 0.98,
                    precision: 0.93,
                    defense: 0.96,
                },
            },
            DriverCharacter {
                id: "chase_gordon",
                name: "Chase 'Rainbow' Gordon",
                alias: "Rainbow Flash",
                bio: "Precision road course virtuoso and aerodynamic drafting master. Lethal slingshot overtakes at Watkins Glen and Daytona.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_racing_red(),
                profile: BotProfile {
                    name: "Chase 'Rainbow' Gordon",
                    lookahead_time: 0.38,
                    speed_factor: 1.04,
                    steering_kp: 2.7,
                    steering_kd: 0.09,
                    brake_margin: 0.99,
                    aggression: 0.88,
                    avoidance_distance: 5.2,
                },
                stats: DriverStats {
                    speed: 0.98,
                    aggression: 0.88,
                    precision: 0.97,
                    defense: 0.91,
                },
            },
            DriverCharacter {
                id: "richard_pettyfield",
                name: "Richard 'The King' Pettyfield",
                alias: "The King",
                bio: "The 200-win patriarch of American stock car racing. Runs the famous Carolina Petty Blue #43 with unmatched pack drafting defense.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_carolina_blue(),
                profile: BotProfile {
                    name: "Richard 'The King' Pettyfield",
                    lookahead_time: 0.39,
                    speed_factor: 1.03,
                    steering_kp: 2.6,
                    steering_kd: 0.08,
                    brake_margin: 1.00,
                    aggression: 0.89,
                    avoidance_distance: 5.4,
                },
                stats: DriverStats {
                    speed: 0.96,
                    aggression: 0.89,
                    precision: 0.96,
                    defense: 0.98,
                },
            },
            DriverCharacter {
                id: "rowdy_busch",
                name: "Rowdy 'Wild Thing' Busch",
                alias: "Wild Thing",
                bio: "Raw aggression and fearless dive-bombs into turn 1. Thrives in chaotic multicar packs and high-banked superspeedway shootouts.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_sunset_orange(),
                profile: BotProfile {
                    name: "Rowdy 'Wild Thing' Busch",
                    lookahead_time: 0.33,
                    speed_factor: 1.04,
                    steering_kp: 2.9,
                    steering_kd: 0.07,
                    brake_margin: 0.94,
                    aggression: 0.97,
                    avoidance_distance: 4.6,
                },
                stats: DriverStats {
                    speed: 0.97,
                    aggression: 0.97,
                    precision: 0.90,
                    defense: 0.92,
                },
            },
            DriverCharacter {
                id: "jimmie_johnson",
                name: "Jimmie 'Seven-Time' Johnson",
                alias: "Seven-Time",
                bio: "Seven-time Cup champion renowned for surgical consistency, tire management, and late-race charges on superspeedway restarts.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_daytona_blue(),
                profile: BotProfile {
                    name: "Jimmie 'Seven-Time' Johnson",
                    lookahead_time: 0.40,
                    speed_factor: 1.03,
                    steering_kp: 2.7,
                    steering_kd: 0.09,
                    brake_margin: 1.01,
                    aggression: 0.86,
                    avoidance_distance: 5.5,
                },
                stats: DriverStats {
                    speed: 0.96,
                    aggression: 0.86,
                    precision: 0.99,
                    defense: 0.95,
                },
            },
            DriverCharacter {
                id: "tony_stewart",
                name: "Tony 'Smoke' Stewart",
                alias: "Smoke",
                bio: "Dirt track and short-track brawler. Fearless high-line slider who thrives under the lights at Bristol Motor Speedway.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_sunset_orange(),
                profile: BotProfile {
                    name: "Tony 'Smoke' Stewart",
                    lookahead_time: 0.35,
                    speed_factor: 1.03,
                    steering_kp: 2.8,
                    steering_kd: 0.08,
                    brake_margin: 0.96,
                    aggression: 0.95,
                    avoidance_distance: 4.9,
                },
                stats: DriverStats {
                    speed: 0.96,
                    aggression: 0.95,
                    precision: 0.94,
                    defense: 0.93,
                },
            },
            DriverCharacter {
                id: "bobby_allison",
                name: "Bobby 'Alabama' Allison",
                alias: "Alabama Gang",
                bio: "Legendary leader of the Alabama Gang. Superspeedway high-bank specialist with ice-cold nerves in 3-wide pack racing.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_racing_red(),
                profile: BotProfile {
                    name: "Bobby 'Alabama' Allison",
                    lookahead_time: 0.37,
                    speed_factor: 1.03,
                    steering_kp: 2.7,
                    steering_kd: 0.08,
                    brake_margin: 0.99,
                    aggression: 0.91,
                    avoidance_distance: 5.2,
                },
                stats: DriverStats {
                    speed: 0.95,
                    aggression: 0.91,
                    precision: 0.95,
                    defense: 0.94,
                },
            },
            DriverCharacter {
                id: "bubba_wallace",
                name: "Bubba 'The Rocket' Wallace",
                alias: "The Rocket",
                bio: "Aggressive superspeedway draft pusher who leads train drafts and charges to the front in restrictor plate pack racing.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_carolina_blue(),
                profile: BotProfile {
                    name: "Bubba 'The Rocket' Wallace",
                    lookahead_time: 0.35,
                    speed_factor: 1.04,
                    steering_kp: 2.7,
                    steering_kd: 0.08,
                    brake_margin: 0.98,
                    aggression: 0.94,
                    avoidance_distance: 5.0,
                },
                stats: DriverStats {
                    speed: 0.96,
                    aggression: 0.94,
                    precision: 0.92,
                    defense: 0.92,
                },
            },
            DriverCharacter {
                id: "joey_logano",
                name: "Joey 'Sliced Bread' Logano",
                alias: "Sliced Bread",
                bio: "Fierce blocker and tactical restart master. Known for ultra-aggressive bump drafting and defending every inch of asphalt.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_daytona_blue(),
                profile: BotProfile {
                    name: "Joey 'Sliced Bread' Logano",
                    lookahead_time: 0.36,
                    speed_factor: 1.03,
                    steering_kp: 2.8,
                    steering_kd: 0.08,
                    brake_margin: 0.98,
                    aggression: 0.96,
                    avoidance_distance: 4.8,
                },
                stats: DriverStats {
                    speed: 0.95,
                    aggression: 0.96,
                    precision: 0.93,
                    defense: 0.97,
                },
            },
            DriverCharacter {
                id: "bill_elliott",
                name: "Bill 'Awesome Bill' Elliott",
                alias: "Awesome Bill from Dawsonville",
                bio: "All-time qualifying speed record holder at Talladega (212.809 mph / 342.5 km/h). Untouchable straight-line top speed.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_racing_red(),
                profile: BotProfile {
                    name: "Bill 'Awesome Bill' Elliott",
                    lookahead_time: 0.39,
                    speed_factor: 1.06,
                    steering_kp: 2.6,
                    steering_kd: 0.08,
                    brake_margin: 1.00,
                    aggression: 0.88,
                    avoidance_distance: 5.3,
                },
                stats: DriverStats {
                    speed: 0.99,
                    aggression: 0.88,
                    precision: 0.96,
                    defense: 0.93,
                },
            },
            DriverCharacter {
                id: "cale_yarborough",
                name: "Cale 'The Iron Man' Yarborough",
                alias: "The Iron Man",
                bio: "Triple consecutive Cup champion. Hard as iron, never gives an inch on the high banking, and trades paint without flinching.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_sunset_orange(),
                profile: BotProfile {
                    name: "Cale 'The Iron Man' Yarborough",
                    lookahead_time: 0.34,
                    speed_factor: 1.04,
                    steering_kp: 2.8,
                    steering_kd: 0.08,
                    brake_margin: 0.96,
                    aggression: 0.96,
                    avoidance_distance: 4.8,
                },
                stats: DriverStats {
                    speed: 0.97,
                    aggression: 0.96,
                    precision: 0.92,
                    defense: 0.95,
                },
            },
            DriverCharacter {
                id: "rusty_wallace",
                name: "Rusty 'Thunder' Wallace",
                alias: "Thunder",
                bio: "Aggressive short-track and road course warrior. Legendary mastery of high-downforce braking zones and curb hops.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_intimidator_black(),
                profile: BotProfile {
                    name: "Rusty 'Thunder' Wallace",
                    lookahead_time: 0.37,
                    speed_factor: 1.03,
                    steering_kp: 2.7,
                    steering_kd: 0.09,
                    brake_margin: 0.98,
                    aggression: 0.93,
                    avoidance_distance: 5.1,
                },
                stats: DriverStats {
                    speed: 0.95,
                    aggression: 0.93,
                    precision: 0.95,
                    defense: 0.94,
                },
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::Championship {
                name: "NASCAR Cup Series Championship".to_string(),
                point_system: PointSystem::NascarCup { stage_win_bonus: true },
                track_ids: vec![
                    "daytona_superspeedway".to_string(),
                    "talladega_superspeedway".to_string(),
                    "watkins_glen_nascar".to_string(),
                    "bristol_motor_speedway".to_string(),
                ],
                laps_per_round: 10,
            },
            TournamentFormat::EliminationCup {
                elimination_interval: 3,
            },
            TournamentFormat::Championship {
                name: "Trans-Am TA1 National Challenge".to_string(),
                point_system: PointSystem::NascarCup { stage_win_bonus: false },
                track_ids: vec![
                    "watkins_glen_nascar".to_string(),
                    "daytona_superspeedway".to_string(),
                    "talladega_superspeedway".to_string(),
                    "bristol_motor_speedway".to_string(),
                ],
                laps_per_round: 8,
            },
            TournamentFormat::QuickRace {
                default_laps: 6,
                default_bots: 11,
            },
            TournamentFormat::TimeAttack,
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::nascar_v8_pushrod()
    }
}
