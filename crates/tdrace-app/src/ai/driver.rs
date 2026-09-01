use crate::ai::BotProfile;
use crate::render::color::CarColorScheme;
use crate::ui::menu::CarChoice;

/// High-level personality and skill stats for a driver character [0.0..1.0].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriverStats {
    pub speed: f32,
    pub aggression: f32,
    pub precision: f32,
    pub defense: f32,
}

/// Predefined motorsport driver character with unique personality, backstory, preferred car, and AI style.
#[derive(Debug, Clone, PartialEq)]
pub struct DriverCharacter {
    pub id: &'static str,
    pub name: &'static str,
    pub alias: &'static str,
    pub bio: &'static str,
    pub preferred_car: CarChoice,
    pub color_scheme: CarColorScheme,
    pub profile: BotProfile,
    pub stats: DriverStats,
}

impl DriverCharacter {
    /// 1. Silvia "Apex" Tanaka — The Precision Master
    pub const SILVIA_TANAKA: Self = Self {
        id: "silvia_tanaka",
        name: "Silvia Tanaka",
        alias: "Apex Tanaka",
        bio: "Former open-wheel champion whose surgical precision and textbook racing lines carve through chicanes like a scalpel.",
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::from_index(1), // Electric Blue
        profile: BotProfile {
            name: "Silvia Tanaka",
            lookahead_time: 0.40,
            speed_factor: 1.02,
            steering_kp: 2.4,
            steering_kd: 0.07,
            brake_margin: 1.02,
            aggression: 0.75,
            avoidance_distance: 6.5,
        },
        stats: DriverStats {
            speed: 0.92,
            aggression: 0.70,
            precision: 0.98,
            defense: 0.85,
        },
    };

    /// 2. Marco "Thunder" Rossi — High-Speed Brawler
    pub const MARCO_ROSSI: Self = Self {
        id: "marco_rossi",
        name: "Marco Rossi",
        alias: "Thunder Rossi",
        bio: "Fearless and aggressive, Marco thrives in wheel-to-wheel combat, braking at the absolute last millisecond into hairpins.",
        preferred_car: CarChoice::RallyCar,
        color_scheme: CarColorScheme::from_index(4), // Sunset Orange
        profile: BotProfile {
            name: "Marco Rossi",
            lookahead_time: 0.32,
            speed_factor: 1.05,
            steering_kp: 2.6,
            steering_kd: 0.05,
            brake_margin: 0.88,
            aggression: 0.95,
            avoidance_distance: 5.0,
        },
        stats: DriverStats {
            speed: 0.96,
            aggression: 0.98,
            precision: 0.75,
            defense: 0.88,
        },
    };

    /// 3. Kenji "Drift King" Sato — Touge Slide Maestro
    pub const KENJI_SATO: Self = Self {
        id: "kenji_sato",
        name: "Kenji Sato",
        alias: "Drift King Kenji",
        bio: "Honed on mountain passes under neon city lights, Kenji turns every apex into a controlled, high-speed sideways drift.",
        preferred_car: CarChoice::DriftCar,
        color_scheme: CarColorScheme::from_index(5), // Synthwave Purple
        profile: BotProfile {
            name: "Kenji Sato",
            lookahead_time: 0.34,
            speed_factor: 1.03,
            steering_kp: 2.5,
            steering_kd: 0.04,
            brake_margin: 0.92,
            aggression: 0.88,
            avoidance_distance: 5.8,
        },
        stats: DriverStats {
            speed: 0.90,
            aggression: 0.88,
            precision: 0.84,
            defense: 0.72,
        },
    };

    /// 4. Elena "Viper" Frost — The Iceman of the Circuit
    pub const ELENA_FROST: Self = Self {
        id: "elena_frost",
        name: "Elena Frost",
        alias: "Viper Frost",
        bio: "Unflappable under pressure, Elena never misses a braking mark and capitalizes ruthlessly on opponents' mistakes.",
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::from_index(7), // Glacier White & Cyan
        profile: BotProfile {
            name: "Elena Frost",
            lookahead_time: 0.38,
            speed_factor: 0.99,
            steering_kp: 2.2,
            steering_kd: 0.06,
            brake_margin: 1.05,
            aggression: 0.70,
            avoidance_distance: 7.2,
        },
        stats: DriverStats {
            speed: 0.88,
            aggression: 0.65,
            precision: 0.95,
            defense: 0.96,
        },
    };

    /// 5. Jax "Oversteer" Reed — The Wildcard Renegade
    pub const JAX_REED: Self = Self {
        id: "jax_reed",
        name: "Jax Reed",
        alias: "Oversteer Reed",
        bio: "A rallycross veteran with lightning reflexes who uses curbs and sand transitions to slingshot past opponents.",
        preferred_car: CarChoice::RallyCar,
        color_scheme: CarColorScheme::from_index(3), // Sunburst Yellow & Crimson
        profile: BotProfile {
            name: "Jax Reed",
            lookahead_time: 0.30,
            speed_factor: 1.04,
            steering_kp: 2.7,
            steering_kd: 0.05,
            brake_margin: 0.86,
            aggression: 0.92,
            avoidance_distance: 5.2,
        },
        stats: DriverStats {
            speed: 0.94,
            aggression: 0.94,
            precision: 0.78,
            defense: 0.76,
        },
    };

    /// 6. Leo "Pocket Rocket" Bianchi — Agile Shifter Prodigy
    pub const LEO_BIANCHI: Self = Self {
        id: "leo_bianchi",
        name: "Leo Bianchi",
        alias: "Pocket Rocket Leo",
        bio: "A prodigy straight from shifter kart leagues, Leo carries ridiculous corner speed through tight 90-degree switchbacks.",
        preferred_car: CarChoice::Kart,
        color_scheme: CarColorScheme::from_index(2), // Viper Green
        profile: BotProfile {
            name: "Leo Bianchi",
            lookahead_time: 0.36,
            speed_factor: 0.97,
            steering_kp: 2.3,
            steering_kd: 0.07,
            brake_margin: 1.08,
            aggression: 0.65,
            avoidance_distance: 6.8,
        },
        stats: DriverStats {
            speed: 0.86,
            aggression: 0.68,
            precision: 0.92,
            defense: 0.80,
        },
    };

    /// 7. Viktor "The Wall" Sterling — Ironclad Veteran
    pub const VIKTOR_STERLING: Self = Self {
        id: "viktor_sterling",
        name: "Viktor Sterling",
        alias: "The Wall Sterling",
        bio: "With three decades of motorsport experience, Viktor makes his car as wide as the track, frustrating any pass attempt.",
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::from_index(6), // Stealth Carbon Black
        profile: BotProfile {
            name: "Viktor Sterling",
            lookahead_time: 0.44,
            speed_factor: 0.95,
            steering_kp: 2.0,
            steering_kd: 0.08,
            brake_margin: 1.14,
            aggression: 0.78,
            avoidance_distance: 7.8,
        },
        stats: DriverStats {
            speed: 0.84,
            aggression: 0.80,
            precision: 0.88,
            defense: 0.99,
        },
    };

    /// 8. Maya "Phoenix" Lin — Telemetry Prodigy
    pub const MAYA_LIN: Self = Self {
        id: "maya_lin",
        name: "Maya Lin",
        alias: "Phoenix Lin",
        bio: "An engineering-minded racer who calculates optimal slip angles in real time, delivering blistering straight-line exits.",
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::from_index(8), // Cyber Magenta & Neon Cyan
        profile: BotProfile {
            name: "Maya Lin",
            lookahead_time: 0.37,
            speed_factor: 1.01,
            steering_kp: 2.3,
            steering_kd: 0.06,
            brake_margin: 1.00,
            aggression: 0.82,
            avoidance_distance: 6.2,
        },
        stats: DriverStats {
            speed: 0.93,
            aggression: 0.82,
            precision: 0.94,
            defense: 0.86,
        },
    };

    /// Complete registry of all 8 predefined driver characters.
    pub const ROSTER: [Self; 8] = [
        Self::SILVIA_TANAKA,
        Self::MARCO_ROSSI,
        Self::KENJI_SATO,
        Self::ELENA_FROST,
        Self::JAX_REED,
        Self::LEO_BIANCHI,
        Self::VIKTOR_STERLING,
        Self::MAYA_LIN,
    ];

    /// Returns a slice of all 8 predefined driver characters.
    pub fn all() -> &'static [Self; 8] {
        &Self::ROSTER
    }

    /// Finds a driver by their unique identifier string.
    pub fn find_by_id(id: &str) -> Option<&'static Self> {
        Self::ROSTER.iter().find(|d| d.id == id)
    }

    /// Selects `n` distinct opponents pseudo-randomly from the roster given a seed.
    pub fn sample_opponents(n: usize, seed: u64) -> Vec<Self> {
        let count = n.min(Self::ROSTER.len());
        let mut available: Vec<Self> = Self::ROSTER.to_vec();

        // Simple deterministic LCG shuffle using seed
        let mut s = seed.wrapping_add(1442695040888963407);
        for i in (1..available.len()).rev() {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (s >> 33) as usize % (i + 1);
            available.swap(i, j);
        }

        available.truncate(count);
        available
    }
}
