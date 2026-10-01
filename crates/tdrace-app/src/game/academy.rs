//! # Classic Academy Engine & Challenge Evaluation (Spec 060)
//!
//! Provides the challenge runtime, real-time pace delta calculation,
//! contact/clean-attempt validation, and milestone bounty evaluation
//! for the Classic Arcade Academy curriculum.

use serde::{Deserialize, Serialize};
pub use tdrace_core::profile::{
    AcademyLessonDef, AcademyLessonId, AcademyLessonProgress, AcademyMedal, ClassicAcademyProgress,
    LicenseGrade,
};
use crate::game::RaceSession;
use crate::profile::PlayerProfile;
use crate::ui::menu::{CarChoice, GameMode};

/// Dynamic pace split status relative to medal targets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcademyPaceStatus {
    AheadOfGold(f32),
    AheadOfSilver(f32),
    AheadOfBronze(f32),
    BehindBronze(f32),
}

impl AcademyPaceStatus {
    pub fn delta_seconds(&self) -> f32 {
        match *self {
            Self::AheadOfGold(d) => d,
            Self::AheadOfSilver(d) => d,
            Self::AheadOfBronze(d) => d,
            Self::BehindBronze(d) => d,
        }
    }

    pub fn is_passing(&self) -> bool {
        !matches!(self, Self::BehindBronze(_))
    }
}

/// Real-time state of an ongoing Academy challenge attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcademyChallengeState {
    pub lesson_id: AcademyLessonId,
    pub elapsed_time_sec: f32,
    pub heavy_collisions: u32,
    pub max_collision_impulse: f32,
    pub off_track_seconds: f32,
    pub disqualified: bool,
    pub disqualification_reason: Option<String>,
}

impl AcademyChallengeState {
    pub const MAX_PERMISSIBLE_IMPULSE: f32 = 800.0;
    pub const MAX_PERMISSIBLE_OFF_TRACK_SEC: f32 = 2.0;

    pub fn new(lesson_id: AcademyLessonId) -> Self {
        Self {
            lesson_id,
            elapsed_time_sec: 0.0,
            heavy_collisions: 0,
            max_collision_impulse: 0.0,
            off_track_seconds: 0.0,
            disqualified: false,
            disqualification_reason: None,
        }
    }

    /// Records collision impulse and evaluates attempt cleanliness.
    pub fn register_collision(&mut self, impulse: f32) {
        if impulse > self.max_collision_impulse {
            self.max_collision_impulse = impulse;
        }
        if impulse > Self::MAX_PERMISSIBLE_IMPULSE {
            self.heavy_collisions = self.heavy_collisions.saturating_add(1);
            self.disqualified = true;
            self.disqualification_reason = Some("Attempt Invalidated: Heavy Collision".to_string());
        }
    }

    /// Records off-track time and invalidates if exceeded limit.
    pub fn update_off_track(&mut self, dt: f32) {
        self.off_track_seconds += dt;
        if self.off_track_seconds > Self::MAX_PERMISSIBLE_OFF_TRACK_SEC {
            self.disqualified = true;
            self.disqualification_reason = Some("Attempt Invalidated: Exceeded Track Limits".to_string());
        }
    }

    /// Sets an explicit off-track duration (e.g. from tracker.off_track_timer).
    pub fn set_off_track_seconds(&mut self, seconds: f32) {
        if seconds > self.off_track_seconds {
            self.off_track_seconds = seconds;
        }
        if self.off_track_seconds > Self::MAX_PERMISSIBLE_OFF_TRACK_SEC {
            self.disqualified = true;
            self.disqualification_reason = Some("Attempt Invalidated: Exceeded Track Limits".to_string());
        }
    }

    /// Checks if this attempt is still clean.
    pub fn is_clean(&self) -> bool {
        !self.disqualified
            && self.heavy_collisions == 0
            && self.off_track_seconds <= Self::MAX_PERMISSIBLE_OFF_TRACK_SEC
    }

    /// Computes real-time pace status against target thresholds.
    /// `progress_fraction` is 0.0 to 1.0 (lap progress or sector fraction).
    pub fn pace_status(&self, elapsed_sec: f32, progress_fraction: f32) -> AcademyPaceStatus {
        let def = match AcademyLessonDef::find(self.lesson_id) {
            Some(d) => d,
            None => return AcademyPaceStatus::BehindBronze(0.0),
        };
        let p = progress_fraction.clamp(0.01, 1.0);
        let gold_target = def.gold_time_sec * p;
        let silver_target = def.silver_time_sec * p;
        let bronze_target = def.bronze_time_sec * p;

        if elapsed_sec <= gold_target {
            AcademyPaceStatus::AheadOfGold(elapsed_sec - gold_target)
        } else if elapsed_sec <= silver_target {
            AcademyPaceStatus::AheadOfSilver(elapsed_sec - silver_target)
        } else if elapsed_sec <= bronze_target {
            AcademyPaceStatus::AheadOfBronze(elapsed_sec - bronze_target)
        } else {
            AcademyPaceStatus::BehindBronze(elapsed_sec - bronze_target)
        }
    }
}

/// Comprehensive outcome of an Academy challenge attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcademyAttemptEvaluation {
    pub lesson_id: AcademyLessonId,
    pub time_sec: f32,
    pub clean_attempt: bool,
    pub medal: AcademyMedal,
    pub credits_awarded: u64,
    pub xp_awarded: u32,
    pub is_new_best: bool,
    pub newly_graduated: bool,
    pub total_stars: u32,
    pub total_credits_balance: u64,
    pub failure_reason: Option<String>,
}

/// Evaluates a completed academy challenge attempt, updates driver profile, and returns receipt.
pub fn evaluate_academy_attempt(
    profile: &mut PlayerProfile,
    lesson_id: AcademyLessonId,
    time_sec: f32,
    challenge_state: &AcademyChallengeState,
) -> AcademyAttemptEvaluation {
    let clean = challenge_state.is_clean();
    let failure_reason = if !clean {
        challenge_state
            .disqualification_reason
            .clone()
            .or_else(|| Some("Invalid attempt: contact or track limits".to_string()))
    } else {
        None
    };

    let prev_progress = profile
        .academy_progress
        .lessons
        .get(&lesson_id)
        .cloned()
        .unwrap_or_default();
    let was_graduated = profile.academy_progress.is_graduated();

    let (medal, credits_payout, xp_payout) =
        profile.academy_progress.record_attempt(lesson_id, time_sec, clean);

    if credits_payout > 0 {
        profile.add_credits(credits_payout);
    }
    if xp_payout > 0 {
        profile.add_driver_xp(xp_payout as u64);
    }

    let mut total_credits_awarded = credits_payout;
    let mut total_xp_awarded = xp_payout;

    if clean {
        // Base distance XP (10% of track length in meters ~ 100 XP)
        let distance_xp = 100u32;
        profile.add_driver_xp(distance_xp as u64);
        total_xp_awarded += distance_xp;

        // Practice stipend on rerun if first-time bounty was already claimed (Spec 060)
        if credits_payout == 0 && medal > AcademyMedal::None {
            let practice_stipend = 1_500u64;
            profile.add_credits(practice_stipend);
            total_credits_awarded += practice_stipend;
        }
    }

    if profile.academy_progress.is_graduated() {
        profile.license_grade = profile.license_grade.max(LicenseGrade::ClassD);
    }

    let is_new_best = clean
        && medal > AcademyMedal::None
        && (prev_progress.best_time_sec.is_none() || time_sec < prev_progress.best_time_sec.unwrap());
    let newly_graduated = !was_graduated && profile.academy_progress.is_graduated();
    let total_stars = profile.academy_progress.total_stars();

    AcademyAttemptEvaluation {
        lesson_id,
        time_sec,
        clean_attempt: clean,
        medal,
        credits_awarded: total_credits_awarded,
        xp_awarded: total_xp_awarded,
        is_new_best,
        newly_graduated,
        total_stars,
        total_credits_balance: profile.credits,
        failure_reason,
    }
}

/// Sets up a race session for an Academy lesson.
pub fn setup_academy_session(
    session: &mut RaceSession,
    lesson_id: AcademyLessonId,
) -> Result<(), String> {
    let def = AcademyLessonDef::find(lesson_id)
        .ok_or_else(|| format!("Lesson {:?} definition not found", lesson_id))?;

    session.active_module_id = "classic";
    session.game_mode = GameMode::StandardRace;
    session.free_car_selection = false;
    session.is_time_attack = false;

    // Load track
    session.track_choice = session.track_manager.track_choice_for_slug(&def.track_slug);
    if let Ok(track) = session.track_manager.load_track_by_slug(&def.track_slug) {
        session.track = track;
    }

    // Configure vehicle
    match def.car_slug.as_str() {
        "classic_gt" => {
            session.car_choice = CarChoice::SportsCar;
            session.selected_car_model_id = Some("classic_gt");
        }
        "classic_rally" => {
            session.car_choice = CarChoice::RallyCar;
            session.selected_car_model_id = Some("classic_rally");
        }
        "classic_ax_mudlark" => {
            session.car_choice = CarChoice::CrossCar;
            session.selected_car_model_id = Some("classic_ax_mudlark");
        }
        _ => {
            session.car_choice = CarChoice::SportsCar;
            session.selected_car_model_id = Some("classic_gt");
        }
    }

    // Set laps & bots
    if lesson_id == AcademyLessonId::Lesson4GraduationSprint {
        session.total_laps = 2;
        session.num_bots = 1;
    } else {
        session.total_laps = 1;
        session.num_bots = 0;
    }

    session.active_academy_lesson = Some(lesson_id);
    session.academy_challenge = Some(AcademyChallengeState::new(lesson_id));
    session.academy_last_evaluation = None;

    Ok(())
}

/// Returns the total potential purse for achieving a given medal across all 4 lessons.
pub fn total_curriculum_purse(medal: AcademyMedal) -> u64 {
    match medal {
        AcademyMedal::None => 0,
        AcademyMedal::Bronze => 7_000,
        AcademyMedal::Silver => 10_750,
        AcademyMedal::Gold => 14_500,
    }
}
