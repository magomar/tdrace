use glam::Vec2;
use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines};
use tdrace_core::physics::car::Car;
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::Track;

use super::font::Fonts;
use super::scaler::UiScaler;
use crate::render::color::{CarColorScheme, Palette};
use crate::render::marker::PlayerVisibilityOptions;
use race_ui::hud::widgets::{render_lap_timer, render_minimap, render_position_and_lap};

pub use race_ui::hud::widgets::format_lap_time;

/// Active Personal Best lap achievement notification payload for HUD display.
#[derive(Debug, Clone, PartialEq)]
pub struct PersonalBestNotification {
    /// The lap number completed that established this personal best.
    pub completed_lap: u32,
    /// The lap time in seconds.
    pub lap_time: f32,
    /// Time delta improvement compared to previous personal best (positive = seconds faster). None if initial track record.
    pub delta: Option<f32>,
    /// Remaining display time in seconds.
    pub timer: f32,
    /// Total duration of the notification in seconds.
    pub duration: f32,
}

/// Active visibility aid toggle notification payload for HUD display.
#[derive(Debug, Clone, PartialEq)]
pub struct VisibilityToast {
    pub text: String,
    pub is_on: bool,
    pub timer: f32,
    pub duration: f32,
}

/// Renders the complete modern arcade racing HUD with responsive scaling and vector typography.
#[allow(clippy::too_many_arguments)]
pub fn render_hud(
    fonts: &Fonts,
    track: &Track,
    all_cars: &[Car],
    color_schemes: &[CarColorScheme],
    player_car: &Car,
    player_progress: &TrackProgressTracker,
    position: usize,
    total_racers: usize,
    total_laps: u32,
    is_time_attack: bool,
    countdown_timer: Option<f32>,
    gamepad_connected: bool,
    pb_notification: Option<&PersonalBestNotification>,
    visibility_toast: Option<&VisibilityToast>,
    _visibility_options: &PlayerVisibilityOptions,
    _session_time: f32,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // 1. Position Counter & Lap Counter (Top Left)
    render_position_and_lap(
        fonts,
        &scaler,
        scaler.safe_pad_x,
        scaler.safe_pad_y,
        position,
        total_racers,
        player_progress.current_lap,
        total_laps,
        is_time_attack,
    );

    // 2. Lap Timing & Sector Splits (Top Center)
    render_lap_timer(
        fonts,
        &scaler,
        sw * 0.5,
        scaler.safe_pad_y,
        player_progress,
    );

    // 2b. Personal Best Notification Toast (Under Lap Timer)
    if let Some(pb) = pb_notification {
        render_personal_best_toast(
            fonts,
            &scaler,
            sw * 0.5,
            scaler.safe_pad_y + scaler.s(90.0),
            pb,
        );
    }

    // 2c. Player Car Visibility Aid Toggle Notification Toast (Under Lap Timer or PB Toast)
    if let Some(vt) = visibility_toast {
        let toast_y = if pb_notification.is_some() {
            scaler.safe_pad_y + scaler.s(155.0)
        } else {
            scaler.safe_pad_y + scaler.s(90.0)
        };
        render_visibility_toast(fonts, &scaler, sw * 0.5, toast_y, vt);
    }

    // 3. Mini-Map Radar (Top Right)
    let map_w = scaler.s(175.0);
    let map_h = scaler.s(145.0);
    render_minimap(
        fonts,
        &scaler,
        sw - map_w - scaler.safe_pad_x,
        scaler.safe_pad_y,
        map_w,
        map_h,
        track,
        all_cars,
        color_schemes,
    );

    // 4. Speedometer & Cluster (Bottom Right)
    let speedo_cx = sw - scaler.s(110.0) - scaler.safe_pad_x;
    let speedo_cy = sh - scaler.s(110.0) - scaler.safe_pad_y;
    render_speedometer(fonts, &scaler, speedo_cx, speedo_cy, player_car, gamepad_connected);

    // 5. Controls tooltip (Bottom Left)
    render_controls_guide(fonts, &scaler, scaler.safe_pad_x, sh - scaler.s(22.0) - scaler.safe_pad_y * 0.5);

    // 6. Warnings & Alerts (Wrong Way, Off Track)
    render_warning_alerts(fonts, &scaler, sw, sh, player_progress);

    // 7. Race Countdown Animation ("3", "2", "1", "GO!")
    if let Some(cd) = countdown_timer {
        render_countdown(fonts, &scaler, sw, sh, cd);
    }
}


/// Draws modern hybrid digital-analog speedometer cluster, assist badges, and drift meter.
fn render_speedometer(
    fonts: &Fonts,
    scaler: &UiScaler,
    cx: f32,
    cy: f32,
    car: &Car,
    gamepad_connected: bool,
) {
    let radius = scaler.s(58.0);

    // Background dial with drop shadow
    draw_circle(cx + scaler.s(2.0), cy + scaler.s(3.0), radius, Color::new(0.0, 0.0, 0.0, 0.4));
    draw_circle(cx, cy, radius, Palette::UI_CARD_BG);
    draw_circle_lines(cx, cy, radius, scaler.s(2.0), Palette::UI_CARD_BORDER);

    let speed_kmh = car.speed_kmh();
    let top_kmh = car.config.top_speed_mps * 3.6;
    let ratio = (speed_kmh / top_kmh).clamp(0.0, 1.0);

    // Speed arc gauge
    let start_angle = std::f32::consts::PI * 0.75;
    let sweep = std::f32::consts::PI * 1.5;

    let arc_steps = 28;
    for i in 0..arc_steps {
        let t0 = i as f32 / arc_steps as f32;
        let t1 = (i + 1) as f32 / arc_steps as f32;
        if t0 > ratio {
            break;
        }
        let a0 = start_angle + sweep * t0;
        let a1 = start_angle + sweep * t1.min(ratio);

        let p0 = Vec2::new(cx + a0.cos() * (radius - scaler.s(8.0)), cy + a0.sin() * (radius - scaler.s(8.0)));
        let p1 = Vec2::new(cx + a1.cos() * (radius - scaler.s(8.0)), cy + a1.sin() * (radius - scaler.s(8.0)));

        let arc_col = if t0 > 0.82 {
            Palette::RED
        } else if t0 > 0.55 {
            Palette::NEON_GOLD
        } else {
            Palette::NEON_CYAN
        };
        draw_line(p0.x, p0.y, p1.x, p1.y, scaler.s(4.5), arc_col);
    }

    // Digital speed display
    let speed_str = format!("{:.0}", speed_kmh);
    fonts.draw_display_centered_with_shadow(
        &speed_str,
        cx,
        cy + scaler.s(8.0),
        scaler.font_s(32.0),
        Palette::WHITE,
        Color::new(0.0, 0.0, 0.0, 0.5),
        scaler.s(1.5),
    );

    fonts.draw_ui_bold_centered(
        "KM/H",
        cx,
        cy + scaler.s(26.0),
        scaler.font_s(12.0),
        Palette::UI_TEXT_MUTED,
    );

    // Assist Profile Badge & Intervention Indicators (Top of speedometer)
    let badge_w = scaler.s(60.0);
    let badge_h = scaler.s(20.0);
    let badge_x = cx - radius;
    let badge_y = cy - radius - scaler.s(26.0);

    let (prof_text, prof_col) = if car.config.assists.esc_enabled {
        if car.config.assists.tcs_strength > 0.5 {
            ("ARCADE", Palette::NEON_CYAN)
        } else {
            ("SPORT", Palette::NEON_GOLD)
        }
    } else {
        ("PRO", Palette::RED)
    };

    draw_rectangle(badge_x, badge_y, badge_w, badge_h, Palette::UI_PILL_BG);
    draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 1.2, prof_col);
    fonts.draw_ui_bold(
        prof_text,
        badge_x + scaler.s(6.0),
        badge_y + scaler.s(14.0),
        scaler.font_s(12.0),
        prof_col,
    );

    // Active Gamepad Connected Indicator
    if gamepad_connected {
        let pad_w = scaler.s(52.0);
        let pad_h = scaler.s(18.0);
        let pad_x = badge_x;
        let pad_y = badge_y - scaler.s(22.0);
        draw_rectangle(pad_x, pad_y, pad_w, pad_h, Color::new(0.08, 0.16, 0.12, 0.90));
        draw_rectangle_lines(pad_x, pad_y, pad_w, pad_h, 1.2, Palette::NEON_GREEN);
        fonts.draw_ui_bold(
            "PAD",
            pad_x + scaler.s(14.0),
            pad_y + scaler.s(13.0),
            scaler.font_s(11.0),
            Palette::NEON_GREEN,
        );
    }

    // Active TCS indicator (Flashes gold when intervening)
    if car.state.tcs_active {
        let tcs_x = badge_x + scaler.s(64.0);
        let tcs_w = scaler.s(32.0);
        draw_rectangle(tcs_x, badge_y, tcs_w, badge_h, Palette::NEON_GOLD);
        fonts.draw_ui_bold(
            "TCS",
            tcs_x + scaler.s(5.0),
            badge_y + scaler.s(14.0),
            scaler.font_s(11.0),
            Palette::BLACK,
        );
    }

    // Active ESC indicator (Flashes bright cyan when yaw stabilizing)
    if car.state.esc_active {
        let esc_x = badge_x + scaler.s(98.0);
        let esc_w = scaler.s(32.0);
        draw_rectangle(esc_x, badge_y, esc_w, badge_h, Palette::NEON_CYAN);
        fonts.draw_ui_bold(
            "ESC",
            esc_x + scaler.s(5.0),
            badge_y + scaler.s(14.0),
            scaler.font_s(11.0),
            Palette::BLACK,
        );
    }

    // Drift Score Meter Bar
    if car.state.is_drifting || car.state.drift_score > 0.0 {
        let bar_w = scaler.s(130.0);
        let bar_h = scaler.s(12.0);
        let bar_x = cx - bar_w * 0.5;
        let bar_y = cy + radius + scaler.s(12.0);

        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.1, 0.1, 0.15, 0.85));
        draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 1.2, Palette::NEON_MAGENTA);

        let fill_ratio = (car.state.drift_score / 1000.0).clamp(0.0, 1.0);
        draw_rectangle(
            bar_x + 1.0,
            bar_y + 1.0,
            (bar_w - 2.0) * fill_ratio,
            bar_h - 2.0,
            Palette::NEON_MAGENTA,
        );

        let drift_label = format!("DRIFT: {:.0}", car.state.drift_score);
        fonts.draw_ui_bold(
            &drift_label,
            bar_x,
            bar_y - scaler.s(3.0),
            scaler.font_s(12.5),
            Palette::NEON_MAGENTA,
        );
    }
}

/// Small keyboard and gamepad controls tooltip in lower left corner.
fn render_controls_guide(fonts: &Fonts, scaler: &UiScaler, x: f32, y: f32) {
    let guide = "Q/Up: Gas | A/Down: Brake | O/P: Steer | Space: Handbrake | 1-4: Car Aids | Tab: Cam | Esc: Pause";
    let size = scaler.font_s(13.0);
    // Dark pill behind the text so it stays readable over kerbs, asphalt and grass.
    let text_w = fonts.measure_ui_regular(guide, size).width;
    let pad = scaler.s(6.0);
    draw_rectangle(x - pad, y - size * 0.95, text_w + pad * 2.0, size * 1.35, Color::new(0.0, 0.0, 0.0, 0.45));
    fonts.draw_ui_regular(
        guide,
        x,
        y,
        size,
        Color::new(0.85, 0.88, 0.95, 0.85),
    );
}

/// High-visibility caution banner for Wrong Way alert.
fn render_warning_alerts(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    progress: &TrackProgressTracker,
) {
    if progress.is_wrong_way {
        let banner_w = scaler.s(360.0);
        let banner_h = scaler.s(55.0);
        let x = (sw - banner_w) * 0.5;
        let y = sh * 0.28;

        draw_rectangle(x, y, banner_w, banner_h, Color::new(0.90, 0.12, 0.15, 0.95));
        draw_rectangle_lines(x, y, banner_w, banner_h, 2.5, Palette::WHITE);

        fonts.draw_display_centered_with_shadow(
            "WRONG WAY!",
            sw * 0.5,
            y + scaler.s(40.0),
            scaler.font_s(36.0),
            Palette::WHITE,
            Color::new(0.0, 0.0, 0.0, 0.6),
            scaler.s(2.0),
        );
    }
}

/// Start countdown overlay (3, 2, 1, GO!).
fn render_countdown(fonts: &Fonts, scaler: &UiScaler, sw: f32, sh: f32, time_remaining: f32) {
    // LAN: the host has not scheduled the green light yet (see `LAN_WAITING_COUNTDOWN`).
    if time_remaining >= crate::game::LAN_WAITING_COUNTDOWN {
        fonts.draw_display_centered_with_shadow(
            "WAITING FOR RACERS",
            sw * 0.5,
            sh * 0.45,
            scaler.font_s(40.0),
            Palette::NEON_CYAN,
            Color::new(0.0, 0.0, 0.0, 0.7),
            scaler.s(3.0),
        );
        return;
    }
    let (text, color) = if time_remaining > 2.0 {
        ("3", Palette::RED)
    } else if time_remaining > 1.0 {
        ("2", Palette::NEON_GOLD)
    } else if time_remaining > 0.0 {
        ("1", Palette::NEON_CYAN)
    } else {
        ("GO!", Palette::NEON_GREEN)
    };

    let font_size = scaler.font_s(85.0);
    let center_y = sh * 0.45;

    fonts.draw_display_centered_with_shadow(
        text,
        sw * 0.5,
        center_y,
        font_size,
        color,
        Color::new(0.0, 0.0, 0.0, 0.7),
        scaler.s(4.0),
    );
}

/// Draws the celebratory Personal Best lap achievement notification banner.
fn render_personal_best_toast(
    fonts: &Fonts,
    scaler: &UiScaler,
    center_x: f32,
    y: f32,
    notif: &PersonalBestNotification,
) {
    if notif.timer <= 0.0 || notif.duration <= 0.0 {
        return;
    }

    // Smooth entry slide & fade in/out
    let elapsed = notif.duration - notif.timer;
    let fade_in = (elapsed / 0.25).clamp(0.0, 1.0);
    let fade_out = (notif.timer / 0.40).clamp(0.0, 1.0);
    let alpha = fade_in.min(fade_out);

    let enter_offset = (1.0 - fade_in) * scaler.s(-16.0);
    let card_w = scaler.s(360.0);
    let card_h = scaler.s(58.0);
    let x = center_x - card_w * 0.5;
    let toast_y = y + enter_offset;

    // Outer glow & card background
    let bg_color = Color::new(0.05, 0.07, 0.12, 0.94 * alpha);
    let border_color = Color::new(1.0, 0.82, 0.15, 0.95 * alpha);
    scaler.draw_glass_card(x, toast_y, card_w, card_h, bg_color, border_color, 2.0);

    // Header badge: "★ NEW PERSONAL BEST ★"
    let title_color = Color::new(1.0, 0.84, 0.20, alpha);
    fonts.draw_display_centered_with_shadow(
        "★ NEW PERSONAL BEST ★",
        center_x,
        toast_y + scaler.s(22.0),
        scaler.font_s(16.0),
        title_color,
        Color::new(0.0, 0.0, 0.0, 0.6 * alpha),
        scaler.s(1.5),
    );

    // Details line: Lap number + Lap time + Delta
    let time_str = format_lap_time(notif.lap_time);
    let lap_str = format!("LAP {}: {}", notif.completed_lap, time_str);

    if let Some(delta) = notif.delta {
        let delta_str = format!("(-{:.2}s)", delta);
        let combined = format!("{}  {}", lap_str, delta_str);

        let text_dim = fonts.measure_display(&combined, scaler.font_s(20.0));
        let lap_dim = fonts.measure_display(&format!("{}  ", lap_str), scaler.font_s(20.0));
        let start_x = center_x - text_dim.width * 0.5;

        fonts.draw_display_with_shadow(
            &lap_str,
            start_x,
            toast_y + scaler.s(48.0),
            scaler.font_s(20.0),
            Color::new(1.0, 1.0, 1.0, alpha),
            Color::new(0.0, 0.0, 0.0, 0.6 * alpha),
            scaler.s(1.5),
        );

        fonts.draw_display_with_shadow(
            &delta_str,
            start_x + lap_dim.width,
            toast_y + scaler.s(48.0),
            scaler.font_s(20.0),
            Color::new(0.20, 1.0, 0.50, alpha), // Neon Green
            Color::new(0.0, 0.0, 0.0, 0.6 * alpha),
            scaler.s(1.5),
        );
    } else {
        let rec_str = "(NEW RECORD)";
        let combined = format!("{}  {}", lap_str, rec_str);
        let text_dim = fonts.measure_display(&combined, scaler.font_s(20.0));
        let lap_dim = fonts.measure_display(&format!("{}  ", lap_str), scaler.font_s(20.0));
        let start_x = center_x - text_dim.width * 0.5;

        fonts.draw_display_with_shadow(
            &lap_str,
            start_x,
            toast_y + scaler.s(48.0),
            scaler.font_s(20.0),
            Color::new(1.0, 1.0, 1.0, alpha),
            Color::new(0.0, 0.0, 0.0, 0.6 * alpha),
            scaler.s(1.5),
        );

        fonts.draw_display_with_shadow(
            rec_str,
            start_x + lap_dim.width,
            toast_y + scaler.s(48.0),
            scaler.font_s(20.0),
            Color::new(0.30, 0.90, 1.0, alpha), // Neon Cyan
            Color::new(0.0, 0.0, 0.0, 0.6 * alpha),
            scaler.s(1.5),
        );
    }
}

/// Draws the visibility aid toggle notification banner.
fn render_visibility_toast(
    fonts: &Fonts,
    scaler: &UiScaler,
    center_x: f32,
    y: f32,
    toast: &VisibilityToast,
) {
    if toast.timer <= 0.0 || toast.duration <= 0.0 {
        return;
    }

    let elapsed = toast.duration - toast.timer;
    let fade_in = (elapsed / 0.15).clamp(0.0, 1.0);
    let fade_out = (toast.timer / 0.25).clamp(0.0, 1.0);
    let alpha = fade_in.min(fade_out);
    if alpha <= 0.01 {
        return;
    }

    let card_w = scaler.s(320.0);
    let card_h = scaler.s(38.0);
    let x = center_x - card_w * 0.5;

    let bg_color = Color::new(0.05, 0.07, 0.12, 0.92 * alpha);
    let border_color = if toast.is_on {
        Color::new(Palette::NEON_CYAN.r, Palette::NEON_CYAN.g, Palette::NEON_CYAN.b, 0.90 * alpha)
    } else {
        Color::new(0.40, 0.45, 0.50, 0.65 * alpha)
    };
    scaler.draw_glass_card(x, y, card_w, card_h, bg_color, border_color, 1.5);

    let text_col = if toast.is_on {
        Color::new(1.0, 1.0, 1.0, alpha)
    } else {
        Color::new(0.70, 0.75, 0.80, alpha)
    };

    fonts.draw_ui_bold(
        &toast.text,
        x + scaler.s(16.0),
        y + scaler.s(25.0),
        scaler.font_s(14.0),
        text_col,
    );

    // Pill badge for ON / OFF
    let badge_w = scaler.s(48.0);
    let badge_h = scaler.s(22.0);
    let badge_x = x + card_w - badge_w - scaler.s(10.0);
    let badge_y = y + scaler.s(8.0);
    let (badge_bg, badge_str, badge_text_col) = if toast.is_on {
        (Color::new(0.10, 0.45, 0.40, 0.85 * alpha), "ON", Palette::NEON_CYAN)
    } else {
        (Color::new(0.20, 0.22, 0.26, 0.85 * alpha), "OFF", Palette::UI_TEXT_MUTED)
    };
    draw_rectangle(badge_x, badge_y, badge_w, badge_h, badge_bg);
    draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 1.0, border_color);
    fonts.draw_ui_bold(
        badge_str,
        badge_x + scaler.s(13.0),
        badge_y + scaler.s(15.5),
        scaler.font_s(12.0),
        badge_text_col,
    );
}

/// Renders the complete split-screen HUD tailored for 2 simultaneous players.
#[allow(clippy::too_many_arguments)]
pub fn render_split_hud(
    fonts: &Fonts,
    track: &Track,
    all_cars: &[Car],
    color_schemes: &[CarColorScheme],
    p1_car: &Car,
    p1_progress: &TrackProgressTracker,
    p1_position: usize,
    p2_car: &Car,
    p2_progress: &TrackProgressTracker,
    p2_position: usize,
    total_racers: usize,
    total_laps: u32,
    countdown_timer: Option<f32>,
    gamepad_connected: bool,
    layout: crate::camera::SplitLayout,
) {
    let sw = screen_width();
    let sh = screen_height();
    let [r1, r2] = layout.screen_rects(sw, sh);

    // 1. Draw sleek divider bar between viewports
    match layout {
        crate::camera::SplitLayout::Vertical => {
            let div_x = r1.2;
            draw_rectangle(div_x - 3.0, 0.0, 6.0, sh, Color::new(0.04, 0.06, 0.10, 0.98));
            draw_line(div_x - 2.0, 0.0, div_x - 2.0, sh, 1.5, Palette::NEON_CYAN);
            draw_line(div_x + 2.0, 0.0, div_x + 2.0, sh, 1.5, Palette::NEON_GOLD);
        }
        crate::camera::SplitLayout::Horizontal => {
            let div_y = r1.3;
            draw_rectangle(0.0, div_y - 3.0, sw, 6.0, Color::new(0.04, 0.06, 0.10, 0.98));
            draw_line(0.0, div_y - 2.0, sw, div_y - 2.0, 1.5, Palette::NEON_CYAN);
            draw_line(0.0, div_y + 2.0, sw, div_y + 2.0, 1.5, Palette::NEON_GOLD);
        }
    }

    // 2. Player 1 Pane HUD
    let scaler1 = UiScaler::new(r1.2, r1.3);
    render_split_player_panel(
        fonts,
        &scaler1,
        track,
        all_cars,
        color_schemes,
        r1.0,
        r1.1,
        r1.2,
        r1.3,
        "P1 • KEYS",
        Palette::NEON_CYAN,
        p1_car,
        p1_progress,
        p1_position,
        total_racers,
        total_laps,
        false,
    );

    // 3. Player 2 Pane HUD
    let scaler2 = UiScaler::new(r2.2, r2.3);
    let p2_tag = if gamepad_connected {
        "P2 • GAMEPAD"
    } else {
        "P2 • GAMEPAD (FALLBACK KEYS)"
    };
    render_split_player_panel(
        fonts,
        &scaler2,
        track,
        all_cars,
        color_schemes,
        r2.0,
        r2.1,
        r2.2,
        r2.3,
        p2_tag,
        Palette::NEON_GOLD,
        p2_car,
        p2_progress,
        p2_position,
        total_racers,
        total_laps,
        gamepad_connected,
    );

    // 4. Shared Center Countdown
    if let Some(cd) = countdown_timer {
        let global_scaler = UiScaler::new(sw, sh);
        render_countdown(fonts, &global_scaler, sw, sh, cd);
    }
}

/// Draws an individual player cockpit HUD inside their allocated split-screen rectangle.
#[allow(clippy::too_many_arguments)]
fn render_split_player_panel(
    fonts: &Fonts,
    scaler: &UiScaler,
    track: &Track,
    all_cars: &[Car],
    color_schemes: &[CarColorScheme],
    px: f32,
    py: f32,
    pw: f32,
    ph: f32,
    tag: &str,
    accent: Color,
    car: &Car,
    progress: &TrackProgressTracker,
    pos: usize,
    total_racers: usize,
    total_laps: u32,
    is_gamepad: bool,
) {
    let pad_x = scaler.safe_pad_x.clamp(8.0, 24.0);
    let pad_y = scaler.safe_pad_y.clamp(8.0, 24.0);

    // Top-Left: Player Badge, Position, Lap
    let card_w = scaler.s(165.0).min(pw * 0.40);
    let card_h = scaler.s(74.0);
    let card_x = px + pad_x;
    let card_y = py + pad_y;

    scaler.draw_glass_card(card_x, card_y, card_w, card_h, Palette::UI_CARD_BG, accent, 1.8);
    fonts.draw_ui_bold(
        tag,
        card_x + scaler.s(10.0),
        card_y + scaler.s(18.0),
        scaler.font_s(11.5),
        accent,
    );

    let pos_str = format!("P{}", pos);
    let pos_color = match pos {
        1 => Palette::NEON_GOLD,
        2 => Color::new(0.88, 0.92, 0.98, 1.0),
        _ => Palette::WHITE,
    };
    fonts.draw_display(
        &pos_str,
        card_x + scaler.s(10.0),
        card_y + scaler.s(48.0),
        scaler.font_s(28.0),
        pos_color,
    );

    let total_str = format!("/ {}", total_racers);
    fonts.draw_ui_bold(
        &total_str,
        card_x + scaler.s(52.0),
        card_y + scaler.s(44.0),
        scaler.font_s(15.0),
        Palette::UI_TEXT_MUTED,
    );

    let lap_str = format!("LAP {} / {}", progress.current_lap.min(total_laps), total_laps);
    fonts.draw_ui_bold(
        &lap_str,
        card_x + scaler.s(10.0),
        card_y + scaler.s(67.0),
        scaler.font_s(12.5),
        Palette::WHITE,
    );

    // Top-Right: Lap Timing
    let timer_w = scaler.s(170.0).min(pw * 0.40);
    let timer_h = scaler.s(60.0);
    let timer_x = px + pw - timer_w - pad_x;
    let timer_y = py + pad_y;

    scaler.draw_glass_card(timer_x, timer_y, timer_w, timer_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.5);
    fonts.draw_ui_regular(
        "CURRENT LAP",
        timer_x + scaler.s(10.0),
        timer_y + scaler.s(16.0),
        scaler.font_s(10.5),
        Palette::UI_TEXT_MUTED,
    );
    let lap_time_str = format_lap_time(progress.lap_time);
    fonts.draw_display(
        &lap_time_str,
        timer_x + scaler.s(10.0),
        timer_y + scaler.s(42.0),
        scaler.font_s(22.0),
        Palette::WHITE,
    );
    let best_str = progress.best_lap_time.map(|b| format!("BEST {}", format_lap_time(b))).unwrap_or_else(|| "BEST --:--.--".to_string());
    fonts.draw_ui_bold(
        &best_str,
        timer_x + scaler.s(10.0),
        timer_y + scaler.s(54.0),
        scaler.font_s(10.5),
        Palette::NEON_CYAN,
    );

    // Bottom-Left: Mini-Map Radar
    let map_w = scaler.s(130.0).min(pw * 0.32);
    let map_h = scaler.s(105.0).min(ph * 0.28);
    let map_x = px + pad_x;
    let map_y = py + ph - map_h - pad_y;
    render_minimap(fonts, scaler, map_x, map_y, map_w, map_h, track, all_cars, color_schemes);

    // Bottom-Right: Speedometer Cluster
    let speedo_cx = px + pw - scaler.s(75.0) - pad_x;
    let speedo_cy = py + ph - scaler.s(75.0) - pad_y;
    render_speedometer(fonts, scaler, speedo_cx, speedo_cy, car, is_gamepad);

    // Warnings (Wrong Way, Off Track)
    render_warning_alerts(fonts, scaler, pw, ph, progress);
}

