//! # Classic Academy User Interface & Graduation Ceremony (Spec 060)
//!
//! Renders the 4-lesson curriculum selector screen, medal achievements,
//! challenge HUD pace deltas, and the vector National Grassroots License graduation fanfare.

use macroquad::prelude::*;
use crate::game::academy::{
    AcademyChallengeState, AcademyLessonDef, AcademyLessonId, AcademyMedal, AcademyPaceStatus,
};
use crate::profile::PlayerProfile;
use crate::render::color::Palette;
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;

/// Computes bounding rectangle (x, y, w, h) for an Academy lesson card.
pub fn academy_card_rect(sw: f32, sh: f32, idx: usize) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let card_w = (sw * 0.78).clamp(scaler.s(480.0), scaler.s(820.0));
    let card_x = (sw - card_w) * 0.5;
    let start_y = scaler.s(120.0);
    let card_gap = scaler.s(10.0);
    let total_cards = 4.0;
    let available_h = (sh - start_y - scaler.s(60.0)).max(scaler.s(260.0));
    let card_h = ((available_h - card_gap * (total_cards - 1.0)) / total_cards).clamp(scaler.s(60.0), scaler.s(92.0));
    let card_y = start_y + (idx as f32) * (card_h + card_gap);
    (card_x, card_y, card_w, card_h)
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

/// Renders the Classic Academy curriculum screen.
pub fn render_academy_curriculum_screen(
    fonts: &Fonts,
    profile: &PlayerProfile,
    selected_idx: usize,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

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
        "Master handling fundamentals, earn your National Grassroots License, and win seed funds for your first car.",
        sw * 0.5,
        scaler.s(58.0),
        scaler.font_s(13.0),
        Palette::UI_TEXT_MUTED,
    );

    // Profile Progress Header Strip (Stars & Purse)
    let stars_earned = profile.academy_progress.total_stars();
    let status_str = if profile.has_racing_license() {
        "🎓 LICENSED GRADUATE"
    } else {
        "🔰 ROOKIE CANDIDATE"
    };

    let summary_text = format!(
        "{}  •  ⭐ {} / 12 STARS  •  WALLET: {} Cr  •  POTENTIAL PURSE: 14,500 Cr",
        status_str, stars_earned, profile.credits
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

    // Render 4 Lesson Cards
    let curriculum = AcademyLessonDef::default_curriculum();
    for (i, lesson) in curriculum.iter().enumerate() {
        let (cx, cy, cw, ch) = academy_card_rect(sw, sh, i);
        let is_sel = i == selected_idx;
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
        let lesson_tag = format!("LESSON {}", i + 1);
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

        let targets_str = format!(
            "TARGETS: Gold {:.1}s  |  Silver {:.1}s  |  Bronze {:.1}s",
            lesson.gold_time_sec, lesson.silver_time_sec, lesson.bronze_time_sec
        );
        fonts.draw_ui_bold(
            &targets_str,
            info_x,
            cy + ch * 0.72,
            scaler.font_s(10.5),
            Color::new(0.70, 0.75, 0.85, 1.0),
        );

        // Medal & Best Time Badge (Far Right)
        let status_x = cx + cw - scaler.s(160.0);
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
                let pb_str = format!("PB: {:.2}s", bt);
                fonts.draw_ui_regular(
                    &pb_str,
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
        "[W / S / UP / DOWN] SELECT LESSON  •  [ENTER / SPACE] START CHALLENGE  •  [ESC] BACK",
        sw * 0.5,
        footer_y,
        scaler.font_s(11.5),
        Palette::UI_TEXT_MUTED,
    );
}

/// Renders the Academy challenge in-race HUD overlay (pace delta and incident alerts).
pub fn render_academy_hud(
    fonts: &Fonts,
    lesson_id: AcademyLessonId,
    challenge: &AcademyChallengeState,
    elapsed_time_sec: f32,
    progress_fraction: f32,
) {
    let sw = screen_width();
    let scaler = UiScaler::new(sw, screen_height());

    let def = match AcademyLessonDef::find(lesson_id) {
        Some(d) => d,
        None => return,
    };

    // Academy Header Banner (Center Top)
    let banner_w = scaler.s(380.0);
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

    let pace = challenge.pace_status(elapsed_time_sec, progress_fraction);
    let (pace_text, pace_col) = match pace {
        AcademyPaceStatus::AheadOfGold(d) => (format!("{:.2}s AHEAD OF GOLD", -d), Palette::NEON_GREEN),
        AcademyPaceStatus::AheadOfSilver(d) => (format!("{:.2}s AHEAD OF SILVER", -d), Palette::NEON_CYAN),
        AcademyPaceStatus::AheadOfBronze(d) => (format!("{:.2}s AHEAD OF BRONZE", -d), Palette::NEON_GOLD),
        AcademyPaceStatus::BehindBronze(d) => (format!("+{:.2}s BEHIND BRONZE", d), Palette::RED),
    };

    fonts.draw_ui_bold_centered(
        &format!("CLASSIC ACADEMY: {}", def.title.to_uppercase()),
        sw * 0.5,
        banner_y + scaler.s(18.0),
        scaler.font_s(11.0),
        Palette::NEON_GOLD,
    );

    fonts.draw_display_centered(
        &pace_text,
        sw * 0.5,
        banner_y + scaler.s(38.0),
        scaler.font_s(14.0),
        pace_col,
    );

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
