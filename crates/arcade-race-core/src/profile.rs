//! # Driver Profile & Classic Academy Models
//!
//! Provides the data models for the Classic Academy curriculum, lesson progress,
//! medal achievements, and rookie racing license validation (Spec 060).

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Identifies the progressive lessons within the Classic Academy curriculum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AcademyLessonId {
    Lesson1ApexLine,
    Lesson2BrakingChicane,
    Lesson3SurfaceTransition,
    Lesson4GraduationSprint,
}

impl AcademyLessonId {
    pub const ALL: [AcademyLessonId; 4] = [
        AcademyLessonId::Lesson1ApexLine,
        AcademyLessonId::Lesson2BrakingChicane,
        AcademyLessonId::Lesson3SurfaceTransition,
        AcademyLessonId::Lesson4GraduationSprint,
    ];

    pub fn slug(&self) -> &'static str {
        match self {
            Self::Lesson1ApexLine => "lesson_1_apex_line",
            Self::Lesson2BrakingChicane => "lesson_2_braking_chicane",
            Self::Lesson3SurfaceTransition => "lesson_3_surface_transition",
            Self::Lesson4GraduationSprint => "lesson_4_graduation_sprint",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Self::Lesson1ApexLine => 0,
            Self::Lesson2BrakingChicane => 1,
            Self::Lesson3SurfaceTransition => 2,
            Self::Lesson4GraduationSprint => 3,
        }
    }

    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Lesson1ApexLine),
            1 => Some(Self::Lesson2BrakingChicane),
            2 => Some(Self::Lesson3SurfaceTransition),
            3 => Some(Self::Lesson4GraduationSprint),
            _ => None,
        }
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

/// Definition of an Academy Lesson.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademyLessonDef {
    pub id: AcademyLessonId,
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

impl AcademyLessonDef {
    /// Canonical 4-lesson curriculum matching Spec 060 Section 3.1 & 1.3.
    pub fn default_curriculum() -> Vec<Self> {
        vec![
            Self {
                id: AcademyLessonId::Lesson1ApexLine,
                title: "Racing Line & Apex Precision".to_string(),
                description: "Master apex kerb clipping and smooth throttle exit on asphalt.".to_string(),
                track_slug: "gt_velocity_park".to_string(),
                car_slug: "classic_gt".to_string(),
                gold_time_sec: 18.5,
                silver_time_sec: 20.0,
                bronze_time_sec: 22.5,
                bronze_credit_bounty: 1_000,
                silver_credit_bounty: 500,
                gold_credit_bounty: 500,
                base_xp_reward: 100,
            },
            Self {
                id: AcademyLessonId::Lesson2BrakingChicane,
                title: "Threshold Braking & Weight Transfer".to_string(),
                description: "Brake from high speed into a downhill chicane without locking wheels.".to_string(),
                track_slug: "gt_ridge_ring".to_string(),
                car_slug: "classic_gt".to_string(),
                gold_time_sec: 24.0,
                silver_time_sec: 26.0,
                bronze_time_sec: 29.0,
                bronze_credit_bounty: 1_500,
                silver_credit_bounty: 750,
                gold_credit_bounty: 750,
                base_xp_reward: 150,
            },
            Self {
                id: AcademyLessonId::Lesson3SurfaceTransition,
                title: "Mixed-Surface Transition & Car Control".to_string(),
                description: "Navigate asphalt-to-gravel transition, Scandinavian flick, and jump landings.".to_string(),
                track_slug: "rx_quarry_sprint".to_string(),
                car_slug: "classic_rally".to_string(),
                gold_time_sec: 32.0,
                silver_time_sec: 35.0,
                bronze_time_sec: 39.0,
                bronze_credit_bounty: 2_000,
                silver_credit_bounty: 1_000,
                gold_credit_bounty: 1_000,
                base_xp_reward: 200,
            },
            Self {
                id: AcademyLessonId::Lesson4GraduationSprint,
                title: "Academy Graduation Sprint".to_string(),
                description: "2-lap sprint race against the Academy Instructor pace car with clean overtaking.".to_string(),
                track_slug: "ax_meadow_sprint".to_string(),
                car_slug: "classic_ax_mudlark".to_string(),
                gold_time_sec: 68.0,
                silver_time_sec: 73.0,
                bronze_time_sec: 80.0,
                bronze_credit_bounty: 2_500,
                silver_credit_bounty: 1_500,
                gold_credit_bounty: 1_500,
                base_xp_reward: 350,
            },
        ]
    }

    /// Finds a lesson definition by its ID in the default curriculum.
    pub fn find(id: AcademyLessonId) -> Option<Self> {
        Self::default_curriculum().into_iter().find(|l| l.id == id)
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

    /// Checks if a specific lesson is unlocked for attempting.
    /// Lesson 1 is always unlocked; subsequent lessons require Bronze+ on the preceding lesson.
    pub fn is_lesson_unlocked(&self, id: AcademyLessonId) -> bool {
        let idx = id.index();
        if idx == 0 {
            return true;
        }
        let prev_id = AcademyLessonId::from_index(idx - 1).unwrap();
        self.lessons
            .get(&prev_id)
            .map(|p| p.completed && p.highest_medal >= AcademyMedal::Bronze)
            .unwrap_or(false)
    }

    /// Returns the total stars earned across all lessons (max 12).
    pub fn total_stars(&self) -> u32 {
        self.lessons.values().map(|p| p.highest_medal.stars()).sum()
    }

    /// Records an attempt on a lesson, updates records, grants licenses, and computes idempotent credit bounties.
    /// Returns `(medal_earned, credits_awarded, xp_awarded)`.
    pub fn record_attempt(
        &mut self,
        lesson_id: AcademyLessonId,
        time_sec: f32,
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

        let medal = lesson_def.evaluate_medal(time_sec);
        if medal == AcademyMedal::None {
            return (AcademyMedal::None, 0, 0);
        }

        progress.completed = true;
        if let Some(prev_best) = progress.best_time_sec {
            if time_sec < prev_best {
                progress.best_time_sec = Some(time_sec);
            }
        } else {
            progress.best_time_sec = Some(time_sec);
        }

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

        (medal, credit_payout, xp_payout)
    }
}
