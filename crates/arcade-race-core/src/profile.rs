//! # Driver Profile & Classic Academy Models
//!
//! Provides the data models for the Classic Academy curriculum, lesson progress,
//! medal achievements, and rookie racing license validation (Spec 060, Spec 064).

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Identifies the progressive lessons within the Classic Academy curriculum.
///
/// Tier 1 is the Class D course of Spec 060. Tier 2 (surface mastery) and Tier 3 (tyre discipline)
/// are Spec 064. `index()` counts across all tiers, four lessons per tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AcademyLessonId {
    Lesson1ApexLine,
    Lesson2BrakingChicane,
    Lesson3SurfaceTransition,
    Lesson4GraduationSprint,
    Acad2DirtDrift,
    Acad2ScandinavianFlick,
    Acad2RampBalance,
    Acad2MixedQualifier,
    Acad3ThermalThrottle,
    Acad3OvalStint,
    Acad3WornRubber,
    Acad3TyreConservation,
}

/// Number of Academy tiers in the base game (Spec 064). Tier 4 belongs to the GT DLC.
pub const ACADEMY_TIER_COUNT: u8 = 3;
/// Lessons per tier.
pub const ACADEMY_LESSONS_PER_TIER: usize = 4;
/// Bronze or better on this many lessons of a tier passes the tier.
pub const ACADEMY_TIER_PASS_COUNT: usize = 3;
/// Most Test Driver Stipends paid per profile per local calendar day, across all tiers.
pub const ACADEMY_DAILY_STIPEND_CAP: u8 = 5;

impl AcademyLessonId {
    /// The Tier 1 lessons (Spec 060). Use [`tier_lessons`](Self::tier_lessons) for the other tiers.
    pub const ALL: [AcademyLessonId; 4] = [
        AcademyLessonId::Lesson1ApexLine,
        AcademyLessonId::Lesson2BrakingChicane,
        AcademyLessonId::Lesson3SurfaceTransition,
        AcademyLessonId::Lesson4GraduationSprint,
    ];

    const TIER_2: [AcademyLessonId; 4] = [
        AcademyLessonId::Acad2DirtDrift,
        AcademyLessonId::Acad2ScandinavianFlick,
        AcademyLessonId::Acad2RampBalance,
        AcademyLessonId::Acad2MixedQualifier,
    ];

    const TIER_3: [AcademyLessonId; 4] = [
        AcademyLessonId::Acad3ThermalThrottle,
        AcademyLessonId::Acad3OvalStint,
        AcademyLessonId::Acad3WornRubber,
        AcademyLessonId::Acad3TyreConservation,
    ];

    /// The four lessons of a tier (1 to 3), in play order. Empty for any other tier.
    pub fn tier_lessons(tier: u8) -> &'static [AcademyLessonId] {
        match tier {
            1 => &Self::ALL,
            2 => &Self::TIER_2,
            3 => &Self::TIER_3,
            _ => &[],
        }
    }

    /// The tier this lesson belongs to (1 to 3).
    pub fn tier(&self) -> u8 {
        match self {
            Self::Lesson1ApexLine
            | Self::Lesson2BrakingChicane
            | Self::Lesson3SurfaceTransition
            | Self::Lesson4GraduationSprint => 1,
            Self::Acad2DirtDrift
            | Self::Acad2ScandinavianFlick
            | Self::Acad2RampBalance
            | Self::Acad2MixedQualifier => 2,
            Self::Acad3ThermalThrottle
            | Self::Acad3OvalStint
            | Self::Acad3WornRubber
            | Self::Acad3TyreConservation => 3,
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::Lesson1ApexLine => "lesson_1_apex_line",
            Self::Lesson2BrakingChicane => "lesson_2_braking_chicane",
            Self::Lesson3SurfaceTransition => "lesson_3_surface_transition",
            Self::Lesson4GraduationSprint => "lesson_4_graduation_sprint",
            Self::Acad2DirtDrift => "acad_2_1",
            Self::Acad2ScandinavianFlick => "acad_2_2",
            Self::Acad2RampBalance => "acad_2_3",
            Self::Acad2MixedQualifier => "acad_2_4",
            Self::Acad3ThermalThrottle => "acad_3_1",
            Self::Acad3OvalStint => "acad_3_2",
            Self::Acad3WornRubber => "acad_3_3",
            Self::Acad3TyreConservation => "acad_3_4",
        }
    }

    /// Position over all tiers: Tier 1 is 0 to 3, Tier 2 is 4 to 7, Tier 3 is 8 to 11.
    pub fn index(&self) -> usize {
        match self {
            Self::Lesson1ApexLine => 0,
            Self::Lesson2BrakingChicane => 1,
            Self::Lesson3SurfaceTransition => 2,
            Self::Lesson4GraduationSprint => 3,
            Self::Acad2DirtDrift => 4,
            Self::Acad2ScandinavianFlick => 5,
            Self::Acad2RampBalance => 6,
            Self::Acad2MixedQualifier => 7,
            Self::Acad3ThermalThrottle => 8,
            Self::Acad3OvalStint => 9,
            Self::Acad3WornRubber => 10,
            Self::Acad3TyreConservation => 11,
        }
    }

    pub fn from_index(idx: usize) -> Option<Self> {
        let tier = (idx / ACADEMY_LESSONS_PER_TIER) as u8 + 1;
        Self::tier_lessons(tier).get(idx % ACADEMY_LESSONS_PER_TIER).copied()
    }
}

/// Medal tier awarded for completing an academy challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum AcademyMedal {
    #[default]
    None = 0,
    Bronze = 1,
    Silver = 2,
    Gold = 3,
}

impl AcademyMedal {
    pub fn title(&self) -> &'static str {
        match self {
            Self::None => "No Medal",
            Self::Bronze => "Bronze",
            Self::Silver => "Silver",
            Self::Gold => "Gold",
        }
    }

    pub fn stars(&self) -> u32 {
        *self as u32
    }
}

/// Rules an Academy session sets for its race, whatever the player's own settings are (Spec 064).
/// The player's settings are untouched, so they apply again once the session ends.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AcademySessionRules {
    /// Car damage on or off for every car in the session.
    pub damage_enabled: bool,
    /// Multiplies the tread wear step of every car (0.0 = tyres never wear, 1.0 = the normal rate).
    pub wear_rate_scale: f32,
    /// Wear [0.0, 1.0] the player's four tyres start with.
    pub start_wear: f32,
    pub laps: u32,
    /// AI cars on the grid.
    pub bots: usize,
    /// The mandatory Rallycross joker lap rule is on (Spec 082).
    pub joker_required: bool,
}

impl AcademySessionRules {
    /// Tiers 1 and 2: damage off and tyre wear off, one lap, no opponents.
    pub const fn basics() -> Self {
        Self { damage_enabled: false, wear_rate_scale: 0.0, start_wear: 0.0, laps: 1, bots: 0, joker_required: false }
    }

    /// Wear scale of Tier 3 missions.
    pub const TIER_3_WEAR_RATE_SCALE: f32 = 3.0;

    /// Tier 3: damage off, tyre wear scaled up.
    pub const fn tyre_discipline(laps: u32) -> Self {
        Self { wear_rate_scale: Self::TIER_3_WEAR_RATE_SCALE, laps, ..Self::basics() }
    }
}

/// What a mission asks of the player, and so what the medal is measured on (Spec 064).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AcademyObjective {
    /// Total time. The lesson's three targets are seconds.
    BestTime,
    /// Best single drift banked in the session. The lesson's three targets are drift points, so a larger
    /// value is better.
    DriftScore,
    /// Total time with every landing under the landing limit. Targets are seconds.
    CleanLanding,
    /// Total time, with the worst wheel's wear at the flag at or under `max_wear`. Over the cap gives no medal.
    WearCap { max_wear: f32 },
    /// Finishing position (1st Gold, 2nd Silver, up to `max_position` Bronze), with the worst wheel's wear
    /// at the flag at or under `max_wear`. Over the cap, or not finishing, gives no medal.
    Podium { max_position: u32, max_wear: f32 },
}

/// What one Academy attempt achieved, for the medal rule of the lesson's objective.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AcademyAttemptResult {
    pub time_sec: f32,
    /// Best single drift banked.
    pub drift_score: f32,
    /// Worst wheel wear [0.0, 1.0] at the flag.
    pub worst_wear: f32,
    /// Finishing position, 1 based. 0 when the player did not finish.
    pub position: u32,
}

/// Definition of an Academy Lesson.
///
/// `gold_time_sec`, `silver_time_sec` and `bronze_time_sec` are the three medal targets. They are
/// seconds, except for [`AcademyObjective::DriftScore`] (drift points) and [`AcademyObjective::Podium`]
/// (unused: the finishing position decides).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademyLessonDef {
    pub id: AcademyLessonId,
    pub tier: u8,
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
    pub rules: AcademySessionRules,
    pub objective: AcademyObjective,
}

impl AcademyLessonDef {
    /// The whole curriculum: Tier 1 (Spec 060, four Class D lessons), Tier 2 and Tier 3 (Spec 064).
    pub fn default_curriculum() -> Vec<Self> {
        let mut lessons = vec![
            Self {
                id: AcademyLessonId::Lesson1ApexLine,
                tier: 1,
                title: "Racing Line & Apex Precision".to_string(),
                description: "Master apex kerb clipping and smooth throttle exit on asphalt.".to_string(),
                track_slug: "gt_velocity_park".to_string(),
                car_slug: "classic_gt".to_string(),
                gold_time_sec: 61.2,
                silver_time_sec: 66.5,
                bronze_time_sec: 74.2,
                bronze_credit_bounty: 1_000,
                silver_credit_bounty: 500,
                gold_credit_bounty: 500,
                base_xp_reward: 100,
                rules: AcademySessionRules::basics(),
                objective: AcademyObjective::BestTime,
            },
            Self {
                id: AcademyLessonId::Lesson2BrakingChicane,
                tier: 1,
                title: "Threshold Braking & Weight Transfer".to_string(),
                description: "Brake from high speed into a downhill chicane without locking wheels.".to_string(),
                track_slug: "gt_ridge_ring".to_string(),
                car_slug: "classic_gt".to_string(),
                gold_time_sec: 62.0,
                silver_time_sec: 67.4,
                bronze_time_sec: 75.2,
                bronze_credit_bounty: 1_500,
                silver_credit_bounty: 750,
                gold_credit_bounty: 750,
                base_xp_reward: 150,
                rules: AcademySessionRules::basics(),
                objective: AcademyObjective::BestTime,
            },
            Self {
                id: AcademyLessonId::Lesson3SurfaceTransition,
                tier: 1,
                title: "Mixed-Surface Transition & Car Control".to_string(),
                description: "Navigate asphalt-to-gravel transition, Scandinavian flick, and jump landings.".to_string(),
                track_slug: "rx_quarry_sprint".to_string(),
                car_slug: "classic_rally".to_string(),
                gold_time_sec: 60.3,
                silver_time_sec: 65.6,
                bronze_time_sec: 73.2,
                bronze_credit_bounty: 2_000,
                silver_credit_bounty: 1_000,
                gold_credit_bounty: 1_000,
                base_xp_reward: 200,
                rules: AcademySessionRules::basics(),
                objective: AcademyObjective::BestTime,
            },
            Self {
                id: AcademyLessonId::Lesson4GraduationSprint,
                tier: 1,
                title: "Academy Graduation Sprint".to_string(),
                description: "2-lap sprint race against the Academy Instructor pace car with clean overtaking.".to_string(),
                track_slug: "ax_meadow_sprint".to_string(),
                car_slug: "classic_ax_mudlark".to_string(),
                gold_time_sec: 104.8,
                silver_time_sec: 114.0,
                bronze_time_sec: 127.2,
                bronze_credit_bounty: 2_500,
                silver_credit_bounty: 1_500,
                gold_credit_bounty: 1_500,
                base_xp_reward: 350,
                rules: AcademySessionRules { laps: 2, bots: 1, ..AcademySessionRules::basics() },
                objective: AcademyObjective::BestTime,
            },
        ];
        lessons.extend(Self::tier_2_curriculum());
        lessons.extend(Self::tier_3_curriculum());
        lessons
    }

    /// Medal bounties split the mission purse 50% Bronze, 25% Silver, 25% Gold (Spec 064, as Spec 060).
    fn with_purse(mut self, purse: u64) -> Self {
        self.bronze_credit_bounty = purse / 2;
        self.silver_credit_bounty = purse / 4;
        self.gold_credit_bounty = purse / 4;
        self
    }

    /// Medal targets of all tiers come from the headless reference run of
    /// `tdrace-app/tests/academy_calibration_tests.rs` (Gold about 3% over the reference lap, Silver 12%,
    /// Bronze 25%). Rerun `print_reference_table` there after any change to a circuit, a car or the tyre model.
    fn tier_2_curriculum() -> Vec<Self> {
        let lesson = |id, title: &str, description: &str, track: &str, car: &str, targets: (f32, f32, f32), xp, rules, objective| Self {
            id,
            tier: 2,
            title: title.to_string(),
            description: description.to_string(),
            track_slug: track.to_string(),
            car_slug: car.to_string(),
            gold_time_sec: targets.0,
            silver_time_sec: targets.1,
            bronze_time_sec: targets.2,
            bronze_credit_bounty: 0,
            silver_credit_bounty: 0,
            gold_credit_bounty: 0,
            base_xp_reward: xp,
            rules,
            objective,
        };
        vec![
            lesson(
                AcademyLessonId::Acad2DirtDrift,
                "Dirt Drift Angle",
                "Hold a long controlled slide on the dirt and bank it.",
                "ax_clay_bowl",
                "classic_ax_brawler",
                (16.0, 12.0, 8.0),
                250,
                AcademySessionRules::basics(),
                AcademyObjective::DriftScore,
            )
            .with_purse(5_000),
            lesson(
                AcademyLessonId::Acad2ScandinavianFlick,
                "The Scandinavian Flick",
                "One clean lap across the tarmac-to-gravel transitions.",
                "rx_hilltop_leap",
                "classic_rally",
                (77.7, 84.5, 94.3),
                300,
                AcademySessionRules::basics(),
                AcademyObjective::BestTime,
            )
            .with_purse(5_500),
            lesson(
                AcademyLessonId::Acad2RampBalance,
                "Ramp Takeoff and Balance",
                "One lap with a clean landing off every big ramp.",
                "at_dune_sea",
                "classic_offroad",
                (57.9, 62.9, 70.2),
                350,
                AcademySessionRules::basics(),
                AcademyObjective::CleanLanding,
            )
            .with_purse(6_500),
            lesson(
                AcademyLessonId::Acad2MixedQualifier,
                "Mixed-Surface Qualifier",
                "Two laps with the mandatory joker lap. The penalty counts.",
                "rx_canyon_flyer",
                "classic_rally",
                (150.1, 163.2, 182.1),
                450,
                AcademySessionRules { laps: 2, joker_required: true, ..AcademySessionRules::basics() },
                AcademyObjective::BestTime,
            )
            .with_purse(8_000),
        ]
    }

    fn tier_3_curriculum() -> Vec<Self> {
        let lesson = |id, title: &str, description: &str, track: &str, car: &str, targets: (f32, f32, f32), xp, rules, objective| Self {
            id,
            tier: 3,
            title: title.to_string(),
            description: description.to_string(),
            track_slug: track.to_string(),
            car_slug: car.to_string(),
            gold_time_sec: targets.0,
            silver_time_sec: targets.1,
            bronze_time_sec: targets.2,
            bronze_credit_bounty: 0,
            silver_credit_bounty: 0,
            gold_credit_bounty: 0,
            base_xp_reward: xp,
            rules,
            objective,
        };
        vec![
            lesson(
                AcademyLessonId::Acad3ThermalThrottle,
                "Thermal Throttle Care",
                "Three-lap pace run. Keep the worst tyre under the cap.",
                "gt_ridge_ring",
                "classic_gt",
                (178.4, 194.0, 216.5),
                500,
                AcademySessionRules::tyre_discipline(3),
                AcademyObjective::WearCap { max_wear: 0.40 },
            )
            .with_purse(10_000),
            lesson(
                AcademyLessonId::Acad3OvalStint,
                "Oval Stint Management",
                "Five-lap oval stint. Keep the worst tyre under the cap.",
                "stock_tri_oval_speedway",
                "classic_nascar",
                (193.4, 210.3, 234.7),
                550,
                AcademySessionRules::tyre_discipline(5),
                AcademyObjective::WearCap { max_wear: 0.40 },
            )
            .with_purse(12_000),
            lesson(
                AcademyLessonId::Acad3WornRubber,
                "Nursing Worn Rubber",
                "Start on 65% worn tyres. Two clean laps.",
                "gt_coastal_grand_prix",
                "classic_gt",
                (155.3, 168.9, 188.5),
                600,
                AcademySessionRules { start_wear: 0.65, ..AcademySessionRules::tyre_discipline(2) },
                AcademyObjective::BestTime,
            )
            .with_purse(13_000),
            lesson(
                AcademyLessonId::Acad3TyreConservation,
                "Tyre Conservation Sprint",
                "Four-lap sprint vs 3 AI cars. Finish up front, tyres under cap.",
                "stock_roval",
                "classic_nascar",
                (0.0, 0.0, 0.0),
                750,
                AcademySessionRules { bots: 3, ..AcademySessionRules::tyre_discipline(4) },
                AcademyObjective::Podium { max_position: 4, max_wear: 0.40 },
            )
            .with_purse(15_000),
        ]
    }

    /// The lessons of one tier (1 to 3), in play order.
    pub fn tier_curriculum(tier: u8) -> Vec<Self> {
        Self::default_curriculum().into_iter().filter(|l| l.tier == tier).collect()
    }

    /// Finds a lesson definition by its ID in the default curriculum.
    pub fn find(id: AcademyLessonId) -> Option<Self> {
        Self::default_curriculum().into_iter().find(|l| l.id == id)
    }

    /// Credits of the Test Driver Stipend for re-running a cleared mission of this tier (Spec 060, Spec 064).
    pub fn stipend_credits(&self) -> u64 {
        match self.tier {
            1 => 1_500,
            2 => 2_000,
            _ => 3_000,
        }
    }

    /// Whether the clean-attempt rule (no heavy collision, off-track under 2 s) applies. A race for
    /// position is not scored on a clean lap.
    pub fn requires_clean_attempt(&self) -> bool {
        !matches!(self.objective, AcademyObjective::Podium { .. })
    }

    /// Evaluates the medal earned for a given completion time.
    pub fn evaluate_medal(&self, time_sec: f32) -> AcademyMedal {
        if time_sec <= self.gold_time_sec {
            AcademyMedal::Gold
        } else if time_sec <= self.silver_time_sec {
            AcademyMedal::Silver
        } else if time_sec <= self.bronze_time_sec {
            AcademyMedal::Bronze
        } else {
            AcademyMedal::None
        }
    }

    /// Evaluates the medal an attempt earned under this lesson's objective.
    pub fn evaluate_result(&self, result: &AcademyAttemptResult) -> AcademyMedal {
        match self.objective {
            AcademyObjective::BestTime | AcademyObjective::CleanLanding => self.evaluate_medal(result.time_sec),
            AcademyObjective::DriftScore => {
                if result.drift_score >= self.gold_time_sec {
                    AcademyMedal::Gold
                } else if result.drift_score >= self.silver_time_sec {
                    AcademyMedal::Silver
                } else if result.drift_score >= self.bronze_time_sec {
                    AcademyMedal::Bronze
                } else {
                    AcademyMedal::None
                }
            }
            AcademyObjective::WearCap { max_wear } => {
                if result.worst_wear > max_wear {
                    AcademyMedal::None
                } else {
                    self.evaluate_medal(result.time_sec)
                }
            }
            AcademyObjective::Podium { max_position, max_wear } => {
                if result.worst_wear > max_wear || result.position == 0 || result.position > max_position {
                    AcademyMedal::None
                } else {
                    match result.position {
                        1 => AcademyMedal::Gold,
                        2 => AcademyMedal::Silver,
                        _ => AcademyMedal::Bronze,
                    }
                }
            }
        }
    }
}

/// Player's progress on an individual Academy lesson.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AcademyLessonProgress {
    pub completed: bool,
    pub best_time_sec: Option<f32>,
    pub highest_medal: AcademyMedal,
    pub bronze_claimed: bool,
    pub silver_claimed: bool,
    pub gold_claimed: bool,
    pub attempts: u32,
}

/// Official motorsport license grade accredited to a driver (Spec 053, 060, 064).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum LicenseGrade {
    #[default]
    None = 0,
    ClassD = 1, // National Grassroots (Karting)
    ClassC = 2, // Junior Competition (Autocross / Rallycross)
    ClassB = 3, // National Pro-Am (Stock Car / Extreme Off-Road)
    ClassA = 4, // International GT (GT Racing)
    ClassS = 5, // FIA Superlicense (Apex Prototypes)
}

impl LicenseGrade {
    pub fn title(&self) -> &'static str {
        match self {
            Self::None => "Unlicensed Rookie",
            Self::ClassD => "Class D (National Grassroots)",
            Self::ClassC => "Class C (Junior Competition)",
            Self::ClassB => "Class B (National Pro-Am)",
            Self::ClassA => "Class A (International GT)",
            Self::ClassS => "Class S (World Superlicense)",
        }
    }

    pub fn badge(&self) -> &'static str {
        match self {
            Self::None => "ROOKIE",
            Self::ClassD => "CLASS D",
            Self::ClassC => "CLASS C",
            Self::ClassB => "CLASS B",
            Self::ClassA => "CLASS A",
            Self::ClassS => "CLASS S",
        }
    }

    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::ClassD,
            2 => Self::ClassC,
            3 => Self::ClassB,
            4 => Self::ClassA,
            5 => Self::ClassS,
            _ => Self::None,
        }
    }

    pub fn required_for_category(category: &str) -> Self {
        match category {
            "kart" => Self::ClassD,
            "autocross" | "ax" | "rally" | "rx" => Self::ClassC,
            "nascar" | "stock" | "extreme_offroad" | "offroad" => Self::ClassB,
            "gt" | "gt_challenge" => Self::ClassA,
            _ => Self::ClassD,
        }
    }
}

/// Global Classic Academy progress stored on the driver profile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ClassicAcademyProgress {
    pub lessons: HashMap<AcademyLessonId, AcademyLessonProgress>,
    pub license_granted: bool,
    pub license_granted_timestamp: Option<String>,
    #[serde(default)]
    pub license_grade: LicenseGrade,
    /// Local calendar day (`YYYY-MM-DD`) that `stipends_today` counts (Spec 064).
    #[serde(default)]
    pub stipend_day: Option<String>,
    /// Test Driver Stipends paid on `stipend_day`.
    #[serde(default)]
    pub stipends_today: u8,
}

impl ClassicAcademyProgress {
    /// Creates a fresh academy record with all lessons uncompleted.
    pub fn new() -> Self {
        let mut lessons = HashMap::new();
        for id in AcademyLessonId::ALL {
            lessons.insert(id, AcademyLessonProgress::default());
        }
        Self {
            lessons,
            license_granted: false,
            license_granted_timestamp: None,
            license_grade: LicenseGrade::None,
            stipend_day: None,
            stipends_today: 0,
        }
    }

    /// Checks if player has graduated and earned the National Grassroots License.
    pub fn is_graduated(&self) -> bool {
        self.license_granted
            || self.license_grade >= LicenseGrade::ClassD
            || self
                .lessons
                .get(&AcademyLessonId::Lesson4GraduationSprint)
                .map(|p| p.completed && p.highest_medal >= AcademyMedal::Bronze)
                .unwrap_or(false)
    }

    /// Whether this lesson has Bronze or better.
    fn has_bronze(&self, id: AcademyLessonId) -> bool {
        self.lessons.get(&id).map(|p| p.completed && p.highest_medal >= AcademyMedal::Bronze).unwrap_or(false)
    }

    /// How many lessons of the tier have Bronze or better.
    pub fn tier_bronze_count(&self, tier: u8) -> usize {
        AcademyLessonId::tier_lessons(tier).iter().filter(|id| self.has_bronze(**id)).count()
    }

    /// Whether Bronze or better is on 3 of the 4 lessons of the tier (Spec 064). Tier 1 passes by graduation.
    pub fn is_tier_passed(&self, tier: u8) -> bool {
        if tier == 1 {
            return self.is_graduated();
        }
        self.tier_bronze_count(tier) >= ACADEMY_TIER_PASS_COUNT
    }

    /// Tier 1 is always open. Tier 2 opens on graduation (Class D). Tier 3 and later open when the
    /// tier before is passed (Spec 064).
    pub fn is_tier_unlocked(&self, tier: u8) -> bool {
        match tier {
            0 | 1 => true,
            t if t > ACADEMY_TIER_COUNT => false,
            t => self.is_tier_passed(t - 1),
        }
    }

    /// Checks if a specific lesson is unlocked for attempting.
    /// Tier 1 is sequential: lesson 1 is always open, the next needs Bronze+ on the one before.
    /// Tiers 2 and 3 open together, once the tier is unlocked.
    pub fn is_lesson_unlocked(&self, id: AcademyLessonId) -> bool {
        if id.tier() > 1 {
            return self.is_tier_unlocked(id.tier());
        }
        let idx = id.index();
        if idx == 0 {
            return true;
        }
        let prev_id = AcademyLessonId::from_index(idx - 1).unwrap();
        self.has_bronze(prev_id)
    }

    /// Returns the total stars earned across all lessons (max 36).
    pub fn total_stars(&self) -> u32 {
        self.lessons.values().map(|p| p.highest_medal.stars()).sum()
    }

    /// Takes one Test Driver Stipend for the local day `today` (`YYYY-MM-DD`). Returns false once
    /// [`ACADEMY_DAILY_STIPEND_CAP`] stipends were paid that day.
    pub fn try_take_stipend(&mut self, today: &str) -> bool {
        if self.stipend_day.as_deref() != Some(today) {
            self.stipend_day = Some(today.to_string());
            self.stipends_today = 0;
        }
        if self.stipends_today >= ACADEMY_DAILY_STIPEND_CAP {
            return false;
        }
        self.stipends_today += 1;
        true
    }

    /// Grants the licence a passed tier earns. A grade is never lowered (Spec 064).
    fn grant_tier_licenses(&mut self) {
        if self.is_tier_passed(2) {
            self.license_grade = self.license_grade.max(LicenseGrade::ClassC);
        }
        if self.is_tier_passed(3) {
            self.license_grade = self.license_grade.max(LicenseGrade::ClassB);
        }
    }

    /// Records a time-based attempt on a lesson. See [`record_result`](Self::record_result).
    /// Returns `(medal_earned, credits_awarded, xp_awarded)`.
    pub fn record_attempt(
        &mut self,
        lesson_id: AcademyLessonId,
        time_sec: f32,
        clean_attempt: bool,
    ) -> (AcademyMedal, u64, u32) {
        let result = AcademyAttemptResult { time_sec, ..Default::default() };
        self.record_result(lesson_id, &result, clean_attempt)
    }

    /// Records an attempt on a lesson, updates records, grants licenses, and computes idempotent credit bounties.
    /// A medal pays only the bounties not claimed yet, so a better medal pays the difference.
    /// Returns `(medal_earned, credits_awarded, xp_awarded)`.
    pub fn record_result(
        &mut self,
        lesson_id: AcademyLessonId,
        result: &AcademyAttemptResult,
        clean_attempt: bool,
    ) -> (AcademyMedal, u64, u32) {
        let lesson_def = match AcademyLessonDef::find(lesson_id) {
            Some(d) => d,
            None => return (AcademyMedal::None, 0, 0),
        };

        let progress = self.lessons.entry(lesson_id).or_default();
        progress.attempts = progress.attempts.saturating_add(1);

        if !clean_attempt {
            return (AcademyMedal::None, 0, 0);
        }

        let medal = lesson_def.evaluate_result(result);
        if medal == AcademyMedal::None {
            return (AcademyMedal::None, 0, 0);
        }

        // The stored best is a time (lower is better), or drift points (higher is better).
        let (value, higher_is_better) = match lesson_def.objective {
            AcademyObjective::DriftScore => (result.drift_score, true),
            AcademyObjective::Podium { .. } => (result.position as f32, false),
            _ => (result.time_sec, false),
        };
        progress.completed = true;
        progress.best_time_sec = match progress.best_time_sec {
            Some(prev) if (higher_is_better && value <= prev) || (!higher_is_better && value >= prev) => Some(prev),
            _ => Some(value),
        };

        if medal > progress.highest_medal {
            progress.highest_medal = medal;
        }

        let mut credit_payout = 0u64;
        let mut xp_payout = 0u32;

        if medal >= AcademyMedal::Bronze && !progress.bronze_claimed {
            credit_payout += lesson_def.bronze_credit_bounty;
            progress.bronze_claimed = true;
            xp_payout += lesson_def.base_xp_reward;
        }
        if medal >= AcademyMedal::Silver && !progress.silver_claimed {
            credit_payout += lesson_def.silver_credit_bounty;
            progress.silver_claimed = true;
            xp_payout += lesson_def.base_xp_reward / 2;
        }
        if medal >= AcademyMedal::Gold && !progress.gold_claimed {
            credit_payout += lesson_def.gold_credit_bounty;
            progress.gold_claimed = true;
            xp_payout += lesson_def.base_xp_reward / 2;
        }

        // Check graduation on Lesson 4
        if lesson_id == AcademyLessonId::Lesson4GraduationSprint && medal >= AcademyMedal::Bronze {
            self.license_granted = true;
            if self.license_grade < LicenseGrade::ClassD {
                self.license_grade = LicenseGrade::ClassD;
            }
            if self.license_granted_timestamp.is_none() {
                self.license_granted_timestamp = Some("2026-10-01T00:00:00Z".to_string());
            }
        }
        self.grant_tier_licenses();

        (medal, credit_payout, xp_payout)
    }
}
