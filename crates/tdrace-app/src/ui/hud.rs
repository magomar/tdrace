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

/// Formats seconds into mm:ss.xx time string.
pub fn format_lap_time(time_sec: f32) -> String {
    if time_sec <= 0.0 || !time_sec.is_finite() {
        return "--:--.--".to_string();
    }
    let minutes = (time_sec / 60.0).floor() as u32;
    let seconds = (time_sec % 60.0).floor() as u32;
    let millis = ((time_sec * 100.0) % 100.0).floor() as u32;
    format!("{:02}:{:02}.{:02}", minutes, seconds, millis)
}

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

/// Draws Position Counter (e.g. "POS 1 / 8") and Lap Progress Badge.
#[allow(clippy::too_many_arguments)]
fn render_position_and_lap(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    pos: usize,
    total: usize,
    lap: u32,
    total_laps: u32,
    is_time_attack: bool,
) {
    let box_w = scaler.s(180.0);
    let box_h = scaler.s(80.0);

    // Modern glassmorphism card
    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.8);

    if is_time_attack {
        // Mode badge
        let badge_w = scaler.s(110.0);
        let badge_h = scaler.s(20.0);
        draw_rectangle(x + scaler.s(12.0), y + scaler.s(12.0), badge_w, badge_h, Color::new(0.1, 0.35, 0.45, 0.85));
        draw_rectangle_lines(x + scaler.s(12.0), y + scaler.s(12.0), badge_w, badge_h, 1.2, Palette::NEON_CYAN);
        fonts.draw_ui_bold(
            "TIME ATTACK",
            x + scaler.s(18.0),
            y + scaler.s(26.0),
            scaler.font_s(13.0),
            Palette::NEON_CYAN,
        );

        let lap_str = format!("LAP {}", lap);
        fonts.draw_display(
            &lap_str,
            x + scaler.s(14.0),
            y + scaler.s(65.0),
            scaler.font_s(32.0),
            Palette::WHITE,
        );
    } else {
        // Position Accent Pill
        let pos_color = match pos {
            1 => Palette::NEON_GOLD,
            2 => Color::new(0.88, 0.92, 0.98, 1.0), // Silver
            3 => Color::new(0.92, 0.60, 0.30, 1.0), // Bronze
            _ => Palette::WHITE,
        };

        // Header label
        fonts.draw_ui_regular(
            "POSITION",
            x + scaler.s(14.0),
            y + scaler.s(22.0),
            scaler.font_s(12.0),
            Palette::UI_TEXT_MUTED,
        );

        // Position Big Display: e.g. "P1" or "1"
        let pos_str = format!("P{}", pos);
        fonts.draw_display(
            &pos_str,
            x + scaler.s(14.0),
            y + scaler.s(55.0),
            scaler.font_s(36.0),
            pos_color,
        );

        let total_str = format!("/ {}", total);
        fonts.draw_ui_bold(
            &total_str,
            x + scaler.s(68.0),
            y + scaler.s(50.0),
            scaler.font_s(18.0),
            Palette::UI_TEXT_MUTED,
        );

        // Lap Sub-Badge
        let lap_str = format!("LAP {} / {}", lap.min(total_laps), total_laps);
        fonts.draw_ui_bold(
            &lap_str,
            x + scaler.s(14.0),
            y + scaler.s(73.0),
            scaler.font_s(14.0),
            Palette::NEON_CYAN,
        );
    }
}

/// Draws current lap time, best lap time, and last lap time with high-visibility styling.
fn render_lap_timer(
    fonts: &Fonts,
    scaler: &UiScaler,
    center_x: f32,
    y: f32,
    progress: &TrackProgressTracker,
) {
    let box_w = scaler.s(260.0);
    let box_h = scaler.s(82.0);
    let x = center_x - box_w * 0.5;

    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.8);

    // Current lap timer (large display font)
    let current_str = format_lap_time(progress.lap_time);
    fonts.draw_display_centered_with_shadow(
        &current_str,
        center_x,
        y + scaler.s(38.0),
        scaler.font_s(34.0),
        Palette::WHITE,
        Color::new(0.0, 0.0, 0.0, 0.5),
        scaler.s(1.5),
    );

    // Best lap & Last lap badges
    let best_str = format!("BEST: {}", format_lap_time(progress.best_lap_time.unwrap_or(0.0)));
    let last_str = format!("LAST: {}", format_lap_time(progress.last_lap_time.unwrap_or(0.0)));

    fonts.draw_ui_bold(
        &best_str,
        x + scaler.s(14.0),
        y + scaler.s(68.0),
        scaler.font_s(13.5),
        Palette::NEON_GOLD,
    );

    fonts.draw_ui_bold(
        &last_str,
        x + scaler.s(140.0),
        y + scaler.s(68.0),
        scaler.font_s(13.5),
        Palette::UI_TEXT_MUTED,
    );
}

/// Draws the modern mini-map radar with clean track trace and directional cones.
#[allow(clippy::too_many_arguments)]
fn render_minimap(
    _fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    track: &Track,
    cars: &[Car],
    color_schemes: &[CarColorScheme],
) {
    scaler.draw_glass_card(x, y, w, h, Color::new(0.06, 0.08, 0.12, 0.90), Palette::UI_CARD_BORDER, 1.8);

    if track.spline.samples.is_empty() {
        return;
    }

    // Compute bounding box of track
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for s in &track.spline.samples {
        min_x = min_x.min(s.point.x);
        min_y = min_y.min(s.point.y);
        max_x = max_x.max(s.point.x);
        max_y = max_y.max(s.point.y);
    }

    let track_w = (max_x - min_x).max(10.0);
    let track_h = (max_y - min_y).max(10.0);

    let pad = scaler.s(14.0);
    let map_w = w - pad * 2.0;
    let map_h = h - pad * 2.0;
    let scale = (map_w / track_w).min(map_h / track_h);

    let map_cx = x + w * 0.5;
    let map_cy = y + h * 0.5;
    let track_cx = (min_x + max_x) * 0.5;
    let track_cy = (min_y + max_y) * 0.5;

    let to_map_pt = |pt: Vec2| -> Vec2 {
        let rx = (pt.x - track_cx) * scale;
        let ry = -(pt.y - track_cy) * scale; // Invert Y
        Vec2::new(map_cx + rx, map_cy + ry)
    };

    // Draw track circuit outline with clean modern anti-aliased trace
    let samples = &track.spline.samples;
    for i in 0..samples.len() {
        let p0 = to_map_pt(samples[i].point);
        let p1 = to_map_pt(samples[(i + 1) % samples.len()].point);
        // Outer glow
        draw_line(p0.x, p0.y, p1.x, p1.y, scaler.s(3.5), Color::new(0.2, 0.4, 0.6, 0.4));
        // Core track line
        draw_line(p0.x, p0.y, p1.x, p1.y, scaler.s(2.0), Color::new(0.5, 0.65, 0.85, 0.95));
    }

    // Draw cars on mini-map
    for (i, car) in cars.iter().enumerate() {
        let pt = to_map_pt(car.state.position);
        let is_player = i == 0;
        let col = if is_player {
            Palette::NEON_GOLD
        } else {
            color_schemes.get(i).map(|c| c.primary).unwrap_or(Palette::WHITE)
        };

        let radius = if is_player { scaler.s(5.0) } else { scaler.s(3.5) };
        draw_circle(pt.x, pt.y, radius, col);
        draw_circle_lines(pt.x, pt.y, radius, 1.2, Palette::BLACK);

        // Player heading pointer cone
        if is_player {
            let fwd = car.forward_vector();
            let tip = pt + Vec2::new(fwd.x, -fwd.y) * scaler.s(8.0);
            draw_line(pt.x, pt.y, tip.x, tip.y, scaler.s(2.0), Palette::WHITE);
        }
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
    fonts.draw_ui_regular(
        guide,
        x,
        y,
        scaler.font_s(13.0),
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

