//! # Classic Academy Engine & Challenge Evaluation (Spec 060, Spec 064)
//!
//! Provides the challenge runtime, real-time pace delta calculation,
//! contact/clean-attempt validation, and milestone bounty evaluation
//! for the Classic Arcade Academy curriculum.

use serde::{Deserialize, Serialize};
pub use tdrace_core::profile::{
    AcademyAttemptResult, AcademyLessonDef, AcademyLessonId, AcademyLessonProgress, AcademyMedal,
    AcademyObjective, AcademySessionRules, ClassicAcademyProgress, LicenseGrade, ACADEMY_TIER_COUNT,
};
use tdrace_core::physics::car::LANDING_SPEED_LIMIT_MPS;
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
    /// Hardest landing of the attempt, as impulse (touchdown speed x 100, like a collision impulse).
    pub max_landing_impulse: f32,
    /// Best single drift banked in the attempt, in drift points.
    pub drift_score: f32,
    /// Worst wheel wear [0.0, 1.0] so far.
    pub worst_wear: f32,
    /// Finishing position, 1 based. 0 until the player finishes.
    pub finish_position: u32,
}

impl AcademyChallengeState {
    pub const MAX_PERMISSIBLE_IMPULSE: f32 = 800.0;
    pub const MAX_PERMISSIBLE_OFF_TRACK_SEC: f32 = 2.0;
    /// Landing impulse above which a landing is rough: the touchdown speed that starts to damage
    /// a suspension (Spec 078).
    pub const MAX_PERMISSIBLE_LANDING_IMPULSE: f32 = LANDING_SPEED_LIMIT_MPS * 100.0;

    pub fn new(lesson_id: AcademyLessonId) -> Self {
        Self {
            lesson_id,
            elapsed_time_sec: 0.0,
            heavy_collisions: 0,
            max_collision_impulse: 0.0,
            off_track_seconds: 0.0,
            disqualified: false,
            disqualification_reason: None,
            max_landing_impulse: 0.0,
            drift_score: 0.0,
            worst_wear: 0.0,
            finish_position: 0,
        }
    }

    /// Records a landing. A rough landing invalidates a clean-landing mission (Spec 064); in any other
    /// mission a landing is only noted.
    pub fn register_landing(&mut self, impulse: f32) {
        self.max_landing_impulse = self.max_landing_impulse.max(impulse);
        let clean_landing_mission =
            AcademyLessonDef::find(self.lesson_id).is_some_and(|d| d.objective == AcademyObjective::CleanLanding);
        if clean_landing_mission && impulse > Self::MAX_PERMISSIBLE_LANDING_IMPULSE {
            self.disqualified = true;
            self.disqualification_reason = Some("Attempt Invalidated: Rough Landing".to_string());
        }
    }

    /// Tracks the worst wheel wear and the best banked drift of the player.
    pub fn observe(&mut self, worst_wear: f32, banked_drift_score: f32) {
        self.worst_wear = self.worst_wear.max(worst_wear);
        self.drift_score = self.drift_score.max(banked_drift_score);
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
    /// The clean attempt would have earned a medal, but the worst tyre was over the mission's wear cap.
    pub tyres_worn_out: bool,
    /// Worst wheel wear [0.0, 1.0] at the flag.
    pub worst_wear: f32,
    /// A cleared mission was re-run and the daily stipend limit was already reached.
    pub stipend_limit_reached: bool,
    /// The licence grade this attempt raised the profile to, if it did.
    pub license_earned: Option<LicenseGrade>,
}

/// Evaluates a completed academy challenge attempt, updates driver profile, and returns receipt.
/// The stipend day is today's local date.
pub fn evaluate_academy_attempt(
    profile: &mut PlayerProfile,
    lesson_id: AcademyLessonId,
    time_sec: f32,
    challenge_state: &AcademyChallengeState,
) -> AcademyAttemptEvaluation {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    evaluate_academy_attempt_on(profile, lesson_id, time_sec, challenge_state, &today)
}

/// [`evaluate_academy_attempt`] for an explicit local day `today` (`YYYY-MM-DD`), which the daily
/// stipend cap counts against.
pub fn evaluate_academy_attempt_on(
    profile: &mut PlayerProfile,
    lesson_id: AcademyLessonId,
    time_sec: f32,
    challenge_state: &AcademyChallengeState,
    today: &str,
) -> AcademyAttemptEvaluation {
    let def = AcademyLessonDef::find(lesson_id);
    // A race for position is not scored on a clean lap.
    let clean = challenge_state.is_clean() || !def.as_ref().is_some_and(|d| d.requires_clean_attempt());
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
    let grade_before = profile.license_grade.max(profile.academy_progress.license_grade);

    let result = AcademyAttemptResult {
        time_sec,
        drift_score: challenge_state.drift_score,
        worst_wear: challenge_state.worst_wear,
        position: challenge_state.finish_position,
    };
    let (medal, credits_payout, xp_payout) =
        profile.academy_progress.record_result(lesson_id, &result, clean);

    if credits_payout > 0 {
        profile.add_credits(credits_payout);
    }
    if xp_payout > 0 {
        profile.add_driver_xp(xp_payout as u64);
    }

    let mut total_credits_awarded = credits_payout;
    let mut total_xp_awarded = xp_payout;
    let mut stipend_limit_reached = false;

    if clean {
        // Base distance XP (10% of track length in meters ~ 100 XP)
        let distance_xp = 100u32;
        profile.add_driver_xp(distance_xp as u64);
        total_xp_awarded += distance_xp;

        // Test Driver Stipend on rerun if first-time bounty was already claimed (Spec 060, Spec 064).
        // At most 5 a day across all tiers.
        if credits_payout == 0 && medal > AcademyMedal::None {
            if profile.academy_progress.try_take_stipend(today) {
                let stipend = def.as_ref().map_or(1_500, |d| d.stipend_credits());
                profile.add_credits(stipend);
                total_credits_awarded += stipend;
            } else {
                stipend_limit_reached = true;
            }
        }
    }

    if profile.academy_progress.is_graduated() {
        profile.license_grade = profile.license_grade.max(LicenseGrade::ClassD);
    }
    profile.license_grade = profile.license_grade.max(profile.academy_progress.license_grade);
    let license_earned = (profile.license_grade > grade_before).then_some(profile.license_grade);

    // The tyre cap decides the medal, so it shows only when the run was otherwise good enough.
    let tyres_worn_out = clean
        && medal == AcademyMedal::None
        && def.as_ref().is_some_and(|d| match d.objective {
            AcademyObjective::WearCap { max_wear } | AcademyObjective::Podium { max_wear, .. } => {
                result.worst_wear > max_wear
                    && d.evaluate_result(&AcademyAttemptResult { worst_wear: 0.0, ..result }) > AcademyMedal::None
            }
            _ => false,
        });

    let is_new_best = clean
        && medal > AcademyMedal::None
        && prev_progress.best_time_sec.is_none_or(|prev| match def.as_ref().map(|d| d.objective) {
            Some(AcademyObjective::DriftScore) => challenge_state.drift_score > prev,
            Some(AcademyObjective::Podium { .. }) => (challenge_state.finish_position as f32) < prev,
            _ => time_sec < prev,
        });
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
        tyres_worn_out,
        worst_wear: result.worst_wear,
        stipend_limit_reached,
        license_earned,
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
    let (car_choice, model_id) = match def.car_slug.as_str() {
        "classic_rally" => (CarChoice::RallyCar, "classic_rally"),
        "classic_ax_mudlark" => (CarChoice::CrossCar, "classic_ax_mudlark"),
        "classic_ax_brawler" => (CarChoice::CrossCar, "classic_ax_brawler"),
        "classic_offroad" => (CarChoice::SandRail, "classic_offroad"),
        "classic_nascar" => (CarChoice::StockCar, "classic_nascar"),
        _ => (CarChoice::SportsCar, "classic_gt"),
    };
    session.car_choice = car_choice;
    session.selected_car_model_id = Some(model_id);

    // Set laps & bots. Damage, wear scale and start wear are applied from `session.academy_rules()`
    // when the race is built, and never touch the player's own settings.
    session.total_laps = def.rules.laps;
    session.num_bots = def.rules.bots;

    session.active_academy_lesson = Some(lesson_id);
    session.academy_challenge = Some(AcademyChallengeState::new(lesson_id));
    session.academy_last_evaluation = None;

    Ok(())
}

/// Returns the total potential purse for achieving a given medal across the 4 Tier 1 lessons.
pub fn total_curriculum_purse(medal: AcademyMedal) -> u64 {
    match medal {
        AcademyMedal::None => 0,
        AcademyMedal::Bronze => 7_000,
        AcademyMedal::Silver => 10_750,
        AcademyMedal::Gold => 14_500,
    }
}
