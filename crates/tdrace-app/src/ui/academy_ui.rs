//! # Classic Academy User Interface & Graduation Ceremony (Spec 060, Spec 064)
//!
//! Renders the tabbed curriculum selector screen (one tier of 4 missions at a time), medal
//! achievements, challenge HUD pace deltas, the result banner, and the vector National Grassroots
//! License graduation fanfare.

use macroquad::prelude::*;
use crate::game::academy::{
    AcademyAttemptEvaluation, AcademyChallengeState, AcademyLessonDef, AcademyLessonId, AcademyMedal,
    AcademyObjective, AcademyPaceStatus, LicenseGrade,
};
use tdrace_core::profile::{ACADEMY_LESSONS_PER_TIER, ACADEMY_TIER_COUNT, ACADEMY_TIER_PASS_COUNT};
use crate::profile::PlayerProfile;
use crate::render::color::Palette;
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;

/// Computes bounding rectangle (x, y, w, h) for an Academy lesson card. `idx` is the row (0 to 3) in the
/// tier shown.
pub fn academy_card_rect(sw: f32, sh: f32, idx: usize) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let card_w = (sw * 0.78).clamp(scaler.s(480.0), scaler.s(820.0));
    let card_x = (sw - card_w) * 0.5;
    let start_y = scaler.s(156.0);
    let card_gap = scaler.s(10.0);
    let total_cards = ACADEMY_LESSONS_PER_TIER as f32;
    let available_h = (sh - start_y - scaler.s(60.0)).max(scaler.s(260.0));
    let card_h = ((available_h - card_gap * (total_cards - 1.0)) / total_cards).clamp(scaler.s(60.0), scaler.s(92.0));
    let card_y = start_y + (idx as f32) * (card_h + card_gap);
    (card_x, card_y, card_w, card_h)
}

/// Computes bounding rectangle (x, y, w, h) for the tab of a tier (`tier_idx` 0 to 2).
pub fn academy_tab_rect(sw: f32, sh: f32, tier_idx: usize) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let (card_x, _, card_w, _) = academy_card_rect(sw, sh, 0);
    let gap = scaler.s(8.0);
    let tab_w = (card_w - gap * (ACADEMY_TIER_COUNT as f32 - 1.0)) / ACADEMY_TIER_COUNT as f32;
    (card_x + tier_idx as f32 * (tab_w + gap), scaler.s(100.0), tab_w, scaler.s(32.0))
}

/// Licence the player earns by passing a tier.
fn tier_license(tier: u8) -> LicenseGrade {
    match tier {
        1 => LicenseGrade::ClassD,
        2 => LicenseGrade::ClassC,
        _ => LicenseGrade::ClassB,
    }
}

/// Why a tier is locked, shown on the tab bar and on its cards.
pub fn tier_unlock_rule(tier: u8, profile: &PlayerProfile) -> String {
    match tier {
        0 | 1 => String::new(),
        2 => "LOCKED: graduate from Tier 1 first (Bronze or better on Lesson 4, Class D).".to_string(),
        t => format!(
            "LOCKED: Bronze or better on {} of 4 Tier {} missions ({} / 4 so far).",
            ACADEMY_TIER_PASS_COUNT,
            t - 1,
            profile.academy_progress.tier_bronze_count(t - 1)
        ),
    }
}

/// The medal targets of a lesson as one line, in the unit of its objective.
pub fn lesson_targets_text(lesson: &AcademyLessonDef) -> String {
    match lesson.objective {
        AcademyObjective::DriftScore => format!(
            "Gold {:.0} | Silver {:.0} | Bronze {:.0} drift pts",
            lesson.gold_time_sec, lesson.silver_time_sec, lesson.bronze_time_sec
        ),
        AcademyObjective::Podium { max_wear, .. } => {
            format!("1st Gold | 2nd Silver | finish Bronze | WEAR {:.0}% MAX", max_wear * 100.0)
        }
        AcademyObjective::WearCap { max_wear } => format!(
            "Gold {:.1}s | Silver {:.1}s | Bronze {:.1}s | WEAR {:.0}% MAX",
            lesson.gold_time_sec, lesson.silver_time_sec, lesson.bronze_time_sec, max_wear * 100.0
        ),
        AcademyObjective::CleanLanding => format!(
            "Gold {:.1}s | Silver {:.1}s | Bronze {:.1}s | CLEAN LANDING",
            lesson.gold_time_sec, lesson.silver_time_sec, lesson.bronze_time_sec
        ),
        AcademyObjective::BestTime => format!(
            "Gold {:.1}s | Silver {:.1}s | Bronze {:.1}s",
            lesson.gold_time_sec, lesson.silver_time_sec, lesson.bronze_time_sec
        ),
    }
}

/// The stored personal best of a lesson as text, in the unit of its objective.
fn lesson_best_text(lesson: &AcademyLessonDef, best: f32) -> String {
    match lesson.objective {
        AcademyObjective::DriftScore => format!("BEST: {:.0} pts", best),
        AcademyObjective::Podium { .. } => format!("BEST: P{:.0}", best),
        _ => format!("PB: {:.2}s", best),
    }
}

/// Returns bounding rectangle for the graduation showroom action button.
pub fn graduation_showroom_button_rect(sw: f32, sh: f32) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let btn_w = scaler.s(340.0);
    let btn_h = scaler.s(42.0);
    let btn_x = (sw - btn_w) * 0.5;
    let btn_y = sh * 0.5 + scaler.s(155.0);
    (btn_x, btn_y, btn_w, btn_h)
}

/// Renders the Classic Academy curriculum screen: the tier tabs and the four missions of the tier that
/// `selected_idx` (a lesson index over all tiers) points into.
pub fn render_academy_curriculum_screen(
    fonts: &Fonts,
    profile: &PlayerProfile,
    selected_idx: usize,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);
    let selected_tier = AcademyLessonId::from_index(selected_idx).map_or(1, |id| id.tier());

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Header Title & Description
    fonts.draw_display_centered_with_shadow(
        "CLASSIC ARCADE ACADEMY • DRIVER LICENSING",
        sw * 0.5,
        scaler.s(32.0),
        scaler.font_s(26.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.7),
        scaler.s(2.0),
    );

    fonts.draw_ui_regular_centered(
        "Master handling fundamentals, earn your racing licences, and win seed funds for your first cars.",
        sw * 0.5,
        scaler.s(58.0),
        scaler.font_s(13.0),
        Palette::UI_TEXT_MUTED,
    );

    // Profile Progress Header Strip (Stars & Purse)
    let stars_earned = profile.academy_progress.total_stars();
    let max_stars = AcademyLessonDef::default_curriculum().len() as u32 * 3;
    let tier_purse: u64 = AcademyLessonDef::tier_curriculum(selected_tier)
        .iter()
        .map(|l| l.bronze_credit_bounty + l.silver_credit_bounty + l.gold_credit_bounty)
        .sum();
    let status_str = if profile.has_racing_license() {
        format!("🎓 {} DRIVER", profile.license_grade.max(LicenseGrade::ClassD).badge())
    } else {
        "🔰 ROOKIE CANDIDATE".to_string()
    };

    let summary_text = format!(
        "{}  •  ⭐ {} / {} STARS  •  WALLET: {} Cr  •  TIER {} PURSE: {} Cr",
        status_str, stars_earned, max_stars, profile.credits, selected_tier, tier_purse
    );

    fonts.draw_ui_bold_centered(
        &summary_text,
        sw * 0.5,
        scaler.s(85.0),
        scaler.font_s(13.5),
        if profile.has_racing_license() {
            Palette::NEON_CYAN
        } else {
            Palette::NEON_GOLD
        },
    );

    // Tier tabs
    for tier in 1..=ACADEMY_TIER_COUNT {
        let (tx, ty, tw, th) = academy_tab_rect(sw, sh, tier as usize - 1);
        let is_sel = tier == selected_tier;
        let unlocked = profile.academy_progress.is_tier_unlocked(tier);
        let bg = if is_sel { Palette::UI_CARD_BG_HOVER } else { Color::new(0.05, 0.07, 0.10, 0.85) };
        let border = if is_sel { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER };
        scaler.draw_glass_card(tx, ty, tw, th, bg, border, if is_sel { 2.2 } else { 1.0 });
        let label = format!(
            "{}TIER {} • {}",
            if unlocked { "" } else { "🔒 " },
            tier,
            tier_license(tier).badge()
        );
        fonts.draw_ui_bold_centered(
            &label,
            tx + tw * 0.5,
            ty + th * 0.65,
            scaler.font_s(12.5),
            if is_sel { Palette::NEON_GOLD } else if unlocked { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
        );
    }
    if !profile.academy_progress.is_tier_unlocked(selected_tier) {
        fonts.draw_ui_bold_centered(
            &tier_unlock_rule(selected_tier, profile),
            sw * 0.5,
            scaler.s(148.0),
            scaler.font_s(11.5),
            Palette::RED,
        );
    }

    // Render the 4 mission cards of the tier
    let curriculum = AcademyLessonDef::tier_curriculum(selected_tier);
    for (row, lesson) in curriculum.iter().enumerate() {
        let (cx, cy, cw, ch) = academy_card_rect(sw, sh, row);
        let is_sel = lesson.id.index() == selected_idx;
        let is_unlocked = profile.academy_progress.is_lesson_unlocked(lesson.id);
        let progress = profile.academy_progress.lessons.get(&lesson.id);
        let medal = progress.map(|p| p.highest_medal).unwrap_or(AcademyMedal::None);
        let best_time = progress.and_then(|p| p.best_time_sec);

        let bg_col = if !is_unlocked {
            Color::new(0.03, 0.04, 0.06, 0.60)
        } else if is_sel {
            Palette::UI_CARD_BG_HOVER
        } else {
            Color::new(0.06, 0.08, 0.12, 0.85)
        };

        let border_col = if !is_unlocked {
            Color::new(0.18, 0.20, 0.25, 0.5)
        } else if is_sel {
            Palette::NEON_GOLD
        } else {
            Palette::UI_CARD_BORDER
        };

        scaler.draw_glass_card(cx, cy, cw, ch, bg_col, border_col, if is_sel { 2.4 } else { 1.0 });

        if is_sel && is_unlocked {
            draw_rectangle(cx, cy, scaler.s(6.0), ch, Palette::NEON_GOLD);
        }

        // Lesson Badge & Index
        let badge_x = cx + scaler.s(20.0);
        let badge_y = cy + ch * 0.35;
        let lesson_tag = if selected_tier == 1 {
            format!("LESSON {}", row + 1)
        } else {
            format!("MISSION {}-{}", selected_tier, row + 1)
        };
        fonts.draw_ui_bold(
            &lesson_tag,
            badge_x,
            badge_y,
            scaler.font_s(11.0),
            if is_unlocked { Palette::NEON_GOLD } else { Palette::UI_TEXT_MUTED },
        );

        // Lesson Title & Circuit
        let title_y = cy + ch * 0.65;
        let title_str = format!("{} ({})", lesson.title, lesson.track_slug);
        fonts.draw_display(
            &title_str,
            badge_x,
            title_y,
            scaler.font_s(16.0),
            if !is_unlocked {
                Color::new(0.40, 0.45, 0.50, 1.0)
            } else if is_sel {
                Palette::WHITE
            } else {
                Color::new(0.85, 0.90, 0.95, 1.0)
            },
        );

        // Vehicle & Targets / Rewards (Right Side)
        let info_x = cx + cw * 0.45;
        let desc_y = cy + ch * 0.40;
        fonts.draw_ui_regular(
            &lesson.description,
            info_x,
            desc_y,
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );

        fonts.draw_ui_bold(
            &lesson_targets_text(lesson),
            info_x,
            cy + ch * 0.72,
            scaler.font_s(10.5),
            Color::new(0.70, 0.75, 0.85, 1.0),
        );

        // Medal & Best Time Badge (Far Right)
        let status_x = cx + cw - scaler.s(130.0);
        if !is_unlocked {
            fonts.draw_ui_bold(
                "🔒 LOCKED",
                status_x,
                cy + ch * 0.55,
                scaler.font_s(13.0),
                Palette::UI_TEXT_MUTED,
            );
        } else {
            let medal_text = match medal {
                AcademyMedal::None => "NO MEDAL",
                AcademyMedal::Bronze => "🥉 BRONZE",
                AcademyMedal::Silver => "🥈 SILVER",
                AcademyMedal::Gold => "🥇 GOLD",
            };
            let medal_col = match medal {
                AcademyMedal::None => Palette::UI_TEXT_MUTED,
                AcademyMedal::Bronze => Color::new(0.80, 0.50, 0.20, 1.0),
                AcademyMedal::Silver => Color::new(0.75, 0.82, 0.90, 1.0),
                AcademyMedal::Gold => Palette::NEON_GOLD,
            };

            fonts.draw_ui_bold(
                medal_text,
                status_x,
                cy + ch * 0.42,
                scaler.font_s(12.5),
                medal_col,
            );

            if let Some(bt) = best_time {
                fonts.draw_ui_regular(
                    &lesson_best_text(lesson, bt),
                    status_x,
                    cy + ch * 0.72,
                    scaler.font_s(11.0),
                    Palette::WHITE,
                );
            } else {
                fonts.draw_ui_regular(
                    "NOT ATTEMPTED",
                    status_x,
                    cy + ch * 0.72,
                    scaler.font_s(10.5),
                    Palette::UI_TEXT_MUTED,
                );
            }
        }
    }

    // Bottom Navigation Help
    let footer_y = sh - scaler.s(24.0);
    fonts.draw_ui_bold_centered(
        "[W / S / UP / DOWN] SELECT MISSION  •  [A / D / LEFT / RIGHT / Q / E] TIER  •  [ENTER / SPACE] START  •  [ESC] BACK",
        sw * 0.5,
        footer_y,
        scaler.font_s(11.5),
        Palette::UI_TEXT_MUTED,
    );
}

/// Headline and detail lines of the Academy result banner for an attempt (Spec 064).
pub fn academy_result_summary(eval: &AcademyAttemptEvaluation) -> (String, String) {
    let title = AcademyLessonDef::find(eval.lesson_id).map_or_else(String::new, |d| d.title.to_uppercase());
    let headline = if eval.tyres_worn_out {
        format!("{}: TYRES WORN OUT • NO MEDAL (WORST WHEEL {:.0}%)", title, eval.worst_wear * 100.0)
    } else if !eval.clean_attempt {
        format!(
            "{}: {}",
            title,
            eval.failure_reason.as_deref().unwrap_or("ATTEMPT INVALIDATED").to_uppercase()
        )
    } else if eval.medal == AcademyMedal::None {
        format!("{}: NO MEDAL", title)
    } else {
        format!("{}: {} MEDAL • +{} Cr", title, eval.medal.title().to_uppercase(), eval.credits_awarded)
    };
    let detail = if let Some(grade) = eval.license_earned {
        format!("{} LICENCE GRANTED", grade.badge())
    } else if eval.stipend_limit_reached {
        "DAILY STIPEND LIMIT REACHED (5 PER DAY)".to_string()
    } else {
        format!("WALLET: {} Cr  •  ⭐ {} STARS", eval.total_credits_balance, eval.total_stars)
    };
    (headline, detail)
}

/// Renders the Academy result banner over the race results screen.
pub fn render_academy_result_banner(fonts: &Fonts, eval: &AcademyAttemptEvaluation) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);
    let (headline, detail) = academy_result_summary(eval);
    let good = eval.clean_attempt && eval.medal > AcademyMedal::None;

    let banner_w = (sw * 0.80).clamp(scaler.s(520.0), scaler.s(850.0));
    let banner_h = scaler.s(48.0);
    let banner_x = (sw - banner_w) * 0.5;
    let banner_y = scaler.s(10.0);
    scaler.draw_glass_card(
        banner_x,
        banner_y,
        banner_w,
        banner_h,
        if good { Color::new(0.04, 0.10, 0.08, 0.95) } else { Color::new(0.30, 0.05, 0.05, 0.95) },
        if good { Palette::NEON_GREEN } else { Palette::RED },
        2.0,
    );
    fonts.draw_ui_bold_centered(&headline, sw * 0.5, banner_y + scaler.s(20.0), scaler.font_s(13.0), Palette::WHITE);
    fonts.draw_ui_regular_centered(&detail, sw * 0.5, banner_y + scaler.s(39.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
}

/// Renders the Academy challenge in-race HUD overlay (pace delta and incident alerts).
pub fn render_academy_hud(
    fonts: &Fonts,
    lesson_id: AcademyLessonId,
    challenge: &AcademyChallengeState,
    elapsed_time_sec: f32,
    progress_fraction: f32,
    hottest_tire_temp_c: f32,
) {
    let sw = screen_width();
    let scaler = UiScaler::new(sw, screen_height());

    let def = match AcademyLessonDef::find(lesson_id) {
        Some(d) => d,
        None => return,
    };

    // Academy Header Banner (Center Top)
    let wear_cap = match def.objective {
        AcademyObjective::WearCap { max_wear } | AcademyObjective::Podium { max_wear, .. } => Some(max_wear),
        _ => None,
    };
    let banner_w = if wear_cap.is_some() { scaler.s(500.0) } else { scaler.s(380.0) };
    let banner_h = scaler.s(48.0);
    let banner_x = (sw - banner_w) * 0.5;
    let banner_y = scaler.s(16.0);

    scaler.draw_glass_card(
        banner_x,
        banner_y,
        banner_w,
        banner_h,
        Color::new(0.04, 0.06, 0.10, 0.90),
        Palette::NEON_GOLD,
        1.5,
    );

    let (pace_text, pace_col) = match def.objective {
        AcademyObjective::DriftScore => {
            let best = challenge.drift_score;
            let col = if best >= def.gold_time_sec {
                Palette::NEON_GREEN
            } else if best >= def.silver_time_sec {
                Palette::NEON_CYAN
            } else if best >= def.bronze_time_sec {
                Palette::NEON_GOLD
            } else {
                Palette::UI_TEXT_MUTED
            };
            (format!("BEST DRIFT {:.0} • BRONZE AT {:.0}", best, def.bronze_time_sec), col)
        }
        AcademyObjective::Podium { .. } => ("FINISH 1ST FOR GOLD • 2ND FOR SILVER".to_string(), Palette::NEON_GOLD),
        _ => match challenge.pace_status(elapsed_time_sec, progress_fraction) {
            AcademyPaceStatus::AheadOfGold(d) => (format!("{:.2}s AHEAD OF GOLD", -d), Palette::NEON_GREEN),
            AcademyPaceStatus::AheadOfSilver(d) => (format!("{:.2}s AHEAD OF SILVER", -d), Palette::NEON_CYAN),
            AcademyPaceStatus::AheadOfBronze(d) => (format!("{:.2}s AHEAD OF BRONZE", -d), Palette::NEON_GOLD),
            AcademyPaceStatus::BehindBronze(d) => (format!("+{:.2}s BEHIND BRONZE", d), Palette::RED),
        },
    };

    fonts.draw_ui_bold_centered(
        &format!("CLASSIC ACADEMY: {}", def.title.to_uppercase()),
        sw * 0.5,
        banner_y + scaler.s(18.0),
        scaler.font_s(11.0),
        Palette::NEON_GOLD,
    );

    // The wear line shares the pace row, so the banner stays as high as the one without a wear cap.
    let pace_x = if wear_cap.is_some() { sw * 0.5 - banner_w * 0.18 } else { sw * 0.5 };
    fonts.draw_display_centered(
        &pace_text,
        pace_x,
        banner_y + scaler.s(38.0),
        scaler.font_s(14.0),
        pace_col,
    );

    if let Some(max_wear) = wear_cap {
        crate::ui::hud::render_academy_wear_line(
            fonts,
            &scaler,
            sw * 0.5 + banner_w * 0.30,
            banner_y + scaler.s(38.0),
            challenge.worst_wear,
            max_wear,
            hottest_tire_temp_c,
        );
    }

    // Incident / Contact Disqualification Banner
    if !challenge.is_clean() {
        let alert_w = scaler.s(460.0);
        let alert_h = scaler.s(36.0);
        let alert_x = (sw - alert_w) * 0.5;
        let alert_y = banner_y + banner_h + scaler.s(12.0);

        let msg = challenge
            .disqualification_reason
            .as_deref()
            .unwrap_or("ATTEMPT INVALIDATED");

        scaler.draw_glass_card(
            alert_x,
            alert_y,
            alert_w,
            alert_h,
            Color::new(0.35, 0.05, 0.05, 0.95),
            Palette::RED,
            2.0,
        );

        fonts.draw_ui_bold_centered(
            msg,
            sw * 0.5,
            alert_y + scaler.s(23.0),
            scaler.font_s(12.0),
            Palette::WHITE,
        );
    }
}

/// Renders the vector National Grassroots Racing License graduation ceremony card.
pub fn render_graduation_ceremony_modal(
    fonts: &Fonts,
    profile: &PlayerProfile,
    total_purse_awarded: u64,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Dim background
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.02, 0.03, 0.05, 0.88));

    // Vector License Card dimensions
    let card_w = scaler.s(560.0);
    let card_h = scaler.s(360.0);
    let card_x = (sw - card_w) * 0.5;
    let card_y = (sh - card_h) * 0.5 - scaler.s(20.0);

    // Outer Glowing Border
    scaler.draw_glass_card(
        card_x,
        card_y,
        card_w,
        card_h,
        Color::new(0.06, 0.09, 0.15, 0.98),
        Palette::NEON_GOLD,
        2.5,
    );

    // Card Header Banner
    draw_rectangle(card_x, card_y, card_w, scaler.s(52.0), Color::new(0.09, 0.13, 0.22, 0.95));
    draw_rectangle(card_x, card_y + scaler.s(52.0), card_w, scaler.s(2.0), Palette::NEON_GOLD);

    fonts.draw_display_centered(
        "NATIONAL MOTORSPORT FEDERATION",
        sw * 0.5,
        card_y + scaler.s(22.0),
        scaler.font_s(13.0),
        Palette::NEON_GOLD,
    );
    fonts.draw_display_centered(
        "OFFICIAL NATIONAL GRASSROOTS RACING LICENSE",
        sw * 0.5,
        card_y + scaler.s(42.0),
        scaler.font_s(15.0),
        Palette::WHITE,
    );

    // Card Body Details
    let left_x = card_x + scaler.s(40.0);

    // Driver Name & Title
    fonts.draw_ui_bold(
        "DRIVER IDENTITY:",
        left_x,
        card_y + scaler.s(90.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );
    fonts.draw_display(
        &format!("{} \"{}\"", profile.name, profile.alias),
        left_x,
        card_y + scaler.s(116.0),
        scaler.font_s(20.0),
        Palette::WHITE,
    );

    // License Category & Status
    fonts.draw_ui_bold(
        "SANCTIONED CATEGORY: GRASSROOTS ENTRY (KARTING / AUTOCROSS / RALLYCROSS)",
        left_x,
        card_y + scaler.s(150.0),
        scaler.font_s(11.5),
        Palette::NEON_CYAN,
    );

    let stars_str = format!("ACADEMY EVALUATION: ⭐ {} / 12 STARS ACCREDITED", profile.academy_progress.total_stars());
    fonts.draw_ui_bold(
        &stars_str,
        left_x,
        card_y + scaler.s(175.0),
        scaler.font_s(12.0),
        Palette::NEON_GOLD,
    );

    // Payout Box
    let payout_box_y = card_y + scaler.s(200.0);
    let payout_box_w = card_w - scaler.s(80.0);
    let payout_box_h = scaler.s(50.0);
    draw_rectangle(left_x, payout_box_y, payout_box_w, payout_box_h, Color::new(0.04, 0.06, 0.10, 0.85));
    draw_rectangle_lines(left_x, payout_box_y, payout_box_w, payout_box_h, 1.0, Palette::NEON_GREEN);

    let payout_text = format!("SEED PRIZE PURSE: +{} Cr CREDITED TO WALLET", total_purse_awarded);
    fonts.draw_ui_bold_centered(
        &payout_text,
        sw * 0.5,
        payout_box_y + scaler.s(22.0),
        scaler.font_s(13.0),
        Palette::NEON_GREEN,
    );
    let balance_text = format!("CURRENT BANK BALANCE: {} Cr", profile.credits);
    fonts.draw_ui_regular_centered(
        &balance_text,
        sw * 0.5,
        payout_box_y + scaler.s(40.0),
        scaler.font_s(11.0),
        Palette::WHITE,
    );

    // Showroom Primary Action Button
    let (btn_x, btn_y, btn_w, btn_h) = graduation_showroom_button_rect(sw, sh);
    scaler.draw_glass_card(btn_x, btn_y, btn_w, btn_h, Palette::NEON_GOLD, Palette::WHITE, 2.0);
    fonts.draw_display_centered(
        "VISIT SHOWROOM (BUY STARTER CAR) ▶",
        sw * 0.5,
        btn_y + scaler.s(26.0),
        scaler.font_s(14.0),
        Color::new(0.05, 0.06, 0.09, 1.0),
    );

    // Secondary Return Text
    fonts.draw_ui_regular_centered(
        "Press [ENTER / SPACE] to visit Showroom  •  Press [ESC] to return to Menu",
        sw * 0.5,
        btn_y + btn_h + scaler.s(22.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );
}
