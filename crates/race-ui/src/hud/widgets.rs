//! Basic HUD widgets: lap time text, position and lap badge, lap timer and minimap.
//! Moved from `tdrace-app/src/ui/hud.rs` (spec 057).

use arcade_race_core::track::{Track, TrackProgressTracker};
use arcade_race_core::Body2D;
use cabinet::ui::font::Fonts;
use cabinet::ui::scaler::UiScaler;
use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines};

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

/// Draws Position Counter (e.g. "POS 1 / 8") and Lap Progress Badge.
#[allow(clippy::too_many_arguments)]
pub fn render_position_and_lap(
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
pub fn render_lap_timer(
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
pub fn render_minimap<B: Body2D>(
    _fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    track: &Track,
    cars: &[B],
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
        let pt = to_map_pt(car.position());
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
