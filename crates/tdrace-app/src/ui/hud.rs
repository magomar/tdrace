use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines};
use macroquad::text::{draw_text, measure_text};
use glam::Vec2;
use tdrace_core::physics::car::Car;
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::Track;

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

/// Renders the complete arcade racing HUD (Speedometer, Lap Timer, Split Delta,
/// Mini-map, Position Counter, Warnings, and Countdown).
#[allow(clippy::too_many_arguments)]
pub fn render_hud(
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
) {
    let sw = screen_width();
    let sh = screen_height();

    // 1. Position Counter & Lap Counter (Top Left)
    render_position_and_lap(18.0, 18.0, position, total_racers, player_progress.current_lap, total_laps, is_time_attack);

    // 2. Lap Timing & Sector Splits (Top Center)
    render_lap_timer(sw * 0.5, 18.0, player_progress);

    // 3. Mini-Map (Top Right)
    render_minimap(sw - 190.0, 18.0, 170.0, 140.0, track, all_cars, color_schemes);

    // 4. Speedometer & Drift Meter (Bottom Right)
    render_speedometer(sw - 160.0, sh - 140.0, player_car, gamepad_connected);

    // 5. Controls help tooltip (Bottom Left)
    render_controls_guide(18.0, sh - 35.0);

    // 6. Warnings & Alerts (Wrong Way, Off Track)
    render_warning_alerts(sw, sh, player_progress);

    // 7. Race Countdown Animation ("3", "2", "1", "GO!")
    if let Some(cd) = countdown_timer {
        render_countdown(sw, sh, cd);
    }
}

/// Draws Position Counter (e.g. "P1 / 8") and Lap Counter.
fn render_position_and_lap(
    x: f32,
    y: f32,
    pos: usize,
    total: usize,
    lap: u32,
    total_laps: u32,
    is_time_attack: bool,
) {
    let box_w = 170.0;
    let box_h = 75.0;

    draw_rectangle(x, y, box_w, box_h, Color::new(0.06, 0.08, 0.12, 0.88));
    draw_rectangle_lines(x, y, box_w, box_h, 2.0, Color::new(0.25, 0.35, 0.50, 0.9));

    if is_time_attack {
        draw_text("TIME ATTACK", x + 12.0, y + 26.0, 18.0, Color::new(0.3, 0.9, 1.0, 1.0));
        let lap_str = format!("LAP {}", lap);
        draw_text(&lap_str, x + 12.0, y + 58.0, 26.0, Palette::WHITE);
    } else {
        // Position P1, P2...
        let pos_str = format!("P{}", pos);
        let pos_color = match pos {
            1 => Color::new(1.0, 0.85, 0.1, 1.0), // Gold
            2 => Color::new(0.85, 0.88, 0.92, 1.0), // Silver
            3 => Color::new(0.85, 0.55, 0.25, 1.0), // Bronze
            _ => Palette::WHITE,
        };
        draw_text(&pos_str, x + 12.0, y + 42.0, 36.0, pos_color);

        let total_str = format!("/ {}", total);
        draw_text(&total_str, x + 70.0, y + 36.0, 20.0, Color::new(0.7, 0.7, 0.75, 1.0));

        let lap_str = format!("LAP {}/{}", lap.min(total_laps), total_laps);
        draw_text(&lap_str, x + 12.0, y + 66.0, 18.0, Color::new(0.8, 0.85, 0.9, 1.0));
    }
}

/// Draws current lap time, best lap time, and last lap time.
fn render_lap_timer(center_x: f32, y: f32, progress: &TrackProgressTracker) {
    let box_w = 260.0;
    let box_h = 80.0;
    let x = center_x - box_w * 0.5;

    draw_rectangle(x, y, box_w, box_h, Color::new(0.06, 0.08, 0.12, 0.88));
    draw_rectangle_lines(x, y, box_w, box_h, 2.0, Color::new(0.25, 0.35, 0.50, 0.9));

    // Current lap timer (large font)
    let current_str = format_lap_time(progress.lap_time);
    let measure = measure_text(&current_str, None, 30, 1.0);
    let text_x = center_x - measure.width * 0.5;
    draw_text(&current_str, text_x, y + 34.0, 30.0, Palette::WHITE);

    // Best lap & Last lap
    let best_str = format!("BEST: {}", format_lap_time(progress.best_lap_time.unwrap_or(0.0)));
    let last_str = format!("LAST: {}", format_lap_time(progress.last_lap_time.unwrap_or(0.0)));

    draw_text(&best_str, x + 10.0, y + 68.0, 14.0, Color::new(1.0, 0.85, 0.15, 1.0));
    draw_text(&last_str, x + 135.0, y + 68.0, 14.0, Color::new(0.75, 0.80, 0.85, 1.0));
}

/// Draws the mini-map radar in the top right corner.
fn render_minimap(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    track: &Track,
    cars: &[Car],
    color_schemes: &[CarColorScheme],
) {
    draw_rectangle(x, y, w, h, Color::new(0.06, 0.08, 0.12, 0.85));
    draw_rectangle_lines(x, y, w, h, 2.0, Color::new(0.25, 0.35, 0.50, 0.9));

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

    let pad = 14.0;
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

    // Draw track circuit outline
    let samples = &track.spline.samples;
    for i in 0..samples.len() {
        let p0 = to_map_pt(samples[i].point);
        let p1 = to_map_pt(samples[(i + 1) % samples.len()].point);
        draw_line(p0.x, p0.y, p1.x, p1.y, 2.5, Color::new(0.40, 0.45, 0.55, 0.9));
    }

    // Draw cars on mini-map
    for (i, car) in cars.iter().enumerate() {
        let pt = to_map_pt(car.state.position);
        let is_player = i == 0;
        let col = if is_player {
            Color::new(1.0, 0.95, 0.1, 1.0) // Bright yellow for player
        } else {
            color_schemes.get(i).map(|c| c.primary).unwrap_or(Palette::WHITE)
        };

        let radius = if is_player { 4.5 } else { 3.2 };
        draw_circle(pt.x, pt.y, radius, col);
        draw_circle_lines(pt.x, pt.y, radius, 1.0, Palette::BLACK);

        // Player heading pointer
        if is_player {
            let fwd = car.forward_vector();
            let tip = pt + Vec2::new(fwd.x, -fwd.y) * 6.5;
            draw_line(pt.x, pt.y, tip.x, tip.y, 1.8, Palette::WHITE);
        }
    }
}

/// Draws analog + digital speedometer, drift meter, and controller & assist badges.
fn render_speedometer(cx: f32, cy: f32, car: &Car, gamepad_connected: bool) {
    let radius = 60.0;

    // Background dial
    draw_circle(cx, cy, radius, Color::new(0.06, 0.08, 0.12, 0.88));
    draw_circle_lines(cx, cy, radius, 2.0, Color::new(0.25, 0.35, 0.50, 0.9));

    let speed_kmh = car.speed_kmh();
    let top_kmh = car.config.top_speed_mps * 3.6;
    let ratio = (speed_kmh / top_kmh).clamp(0.0, 1.0);

    // Speed arc gauge
    let start_angle = std::f32::consts::PI * 0.75;
    let sweep = std::f32::consts::PI * 1.5;

    let arc_steps = 24;
    for i in 0..arc_steps {
        let t0 = i as f32 / arc_steps as f32;
        let t1 = (i + 1) as f32 / arc_steps as f32;
        if t0 > ratio {
            break;
        }
        let a0 = start_angle + sweep * t0;
        let a1 = start_angle + sweep * t1.min(ratio);

        let p0 = Vec2::new(cx + a0.cos() * (radius - 8.0), cy + a0.sin() * (radius - 8.0));
        let p1 = Vec2::new(cx + a1.cos() * (radius - 8.0), cy + a1.sin() * (radius - 8.0));

        let arc_col = if t0 > 0.8 {
            Color::new(1.0, 0.2, 0.2, 0.9)
        } else if t0 > 0.5 {
            Color::new(1.0, 0.8, 0.1, 0.9)
        } else {
            Color::new(0.2, 0.7, 1.0, 0.9)
        };
        draw_line(p0.x, p0.y, p1.x, p1.y, 4.0, arc_col);
    }

    // Digital speed display
    let speed_str = format!("{:.0}", speed_kmh);
    let measure = measure_text(&speed_str, None, 28, 1.0);
    draw_text(&speed_str, cx - measure.width * 0.5, cy + 8.0, 28.0, Palette::WHITE);
    draw_text("KM/H", cx - 16.0, cy + 26.0, 13.0, Color::new(0.65, 0.70, 0.80, 1.0));

    // Assist Profile Badge & Intervention Indicators (Top left of speedometer)
    let badge_x = cx - radius;
    let badge_y = cy - radius - 26.0;

    let (prof_text, prof_col) = if car.config.assists.esc_enabled {
        if car.config.assists.tcs_strength > 0.5 {
            ("ARCADE", Color::new(0.3, 0.9, 1.0, 0.95))
        } else {
            ("SPORT", Color::new(1.0, 0.75, 0.2, 0.95))
        }
    } else {
        ("PRO", Color::new(1.0, 0.4, 0.4, 0.85))
    };

    draw_rectangle(badge_x, badge_y, 58.0, 20.0, Color::new(0.08, 0.10, 0.14, 0.88));
    draw_rectangle_lines(badge_x, badge_y, 58.0, 20.0, 1.2, prof_col);
    draw_text(prof_text, badge_x + 6.0, badge_y + 14.0, 12.0, prof_col);

    // Active Gamepad Connected Indicator
    if gamepad_connected {
        let pad_x = badge_x;
        let pad_y = badge_y - 20.0;
        draw_rectangle(pad_x, pad_y, 46.0, 16.0, Color::new(0.08, 0.12, 0.10, 0.90));
        draw_rectangle_lines(pad_x, pad_y, 46.0, 16.0, 1.2, Color::new(0.3, 0.9, 0.5, 0.95));
        draw_text("🎮 PAD", pad_x + 4.0, pad_y + 12.0, 10.5, Color::new(0.3, 0.9, 0.5, 1.0));
    }

    // Active TCS indicator (Flashes gold/orange when intervening)
    if car.state.tcs_active {
        let tcs_x = badge_x + 62.0;
        draw_rectangle(tcs_x, badge_y, 28.0, 20.0, Color::new(1.0, 0.6, 0.0, 0.95));
        draw_text("TCS", tcs_x + 3.0, badge_y + 14.0, 11.0, Palette::BLACK);
    }

    // Active ESC indicator (Flashes bright cyan when yaw stabilizing)
    if car.state.esc_active {
        let esc_x = badge_x + 94.0;
        draw_rectangle(esc_x, badge_y, 28.0, 20.0, Color::new(0.2, 0.8, 1.0, 0.95));
        draw_text("ESC", esc_x + 3.0, badge_y + 14.0, 11.0, Palette::BLACK);
    }

    // Drift Score Meter Bar (if drifting)
    if car.state.is_drifting || car.state.drift_score > 0.0 {
        let bar_w = 120.0;
        let bar_h = 10.0;
        let bar_x = cx - bar_w * 0.5;
        let bar_y = cy + radius + 12.0;

        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.1, 0.1, 0.15, 0.8));
        draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 1.2, Color::new(1.0, 0.3, 0.8, 0.9));

        let fill_ratio = (car.state.drift_score / 1000.0).clamp(0.0, 1.0);
        draw_rectangle(bar_x + 1.0, bar_y + 1.0, (bar_w - 2.0) * fill_ratio, bar_h - 2.0, Color::new(1.0, 0.3, 0.8, 0.95));

        let drift_label = format!("DRIFT: {:.0}", car.state.drift_score);
        draw_text(&drift_label, bar_x, bar_y - 3.0, 13.0, Color::new(1.0, 0.4, 0.9, 1.0));
    }
}

/// Small keyboard and gamepad controls tooltip in lower left corner.
fn render_controls_guide(x: f32, y: f32) {
    let guide = "Q/Up: Gas | A/Down: Brake | O/P: Steer | Space: Handbrake | H: Assists | C: Controls | Tab: Cam | Esc: Pause";
    draw_text(guide, x, y, 14.0, Color::new(0.9, 0.9, 0.95, 0.80));
}

/// Warning banners for Wrong Way and Off Track.
fn render_warning_alerts(sw: f32, sh: f32, progress: &TrackProgressTracker) {
    if progress.is_wrong_way {
        let banner_w = 340.0;
        let banner_h = 50.0;
        let x = (sw - banner_w) * 0.5;
        let y = sh * 0.28;

        draw_rectangle(x, y, banner_w, banner_h, Color::new(0.85, 0.10, 0.10, 0.90));
        draw_rectangle_lines(x, y, banner_w, banner_h, 2.0, Palette::WHITE);

        let text = "WRONG WAY!";
        let m = measure_text(text, None, 28, 1.0);
        draw_text(text, x + (banner_w - m.width) * 0.5, y + 35.0, 28.0, Palette::WHITE);
    }
}

/// Start countdown overlay (3, 2, 1, GO!).
fn render_countdown(sw: f32, sh: f32, time_remaining: f32) {
    let (text, color) = if time_remaining > 2.0 {
        ("3", Color::new(1.0, 0.2, 0.2, 1.0))
    } else if time_remaining > 1.0 {
        ("2", Color::new(1.0, 0.75, 0.1, 1.0))
    } else if time_remaining > 0.0 {
        ("1", Color::new(0.2, 0.95, 0.2, 1.0))
    } else {
        ("GO!", Color::new(0.1, 1.0, 0.4, 1.0))
    };

    let font_size = 80;
    let m = measure_text(text, None, font_size, 1.0);
    let x = (sw - m.width) * 0.5;
    let y = sh * 0.45;

    // Shadow & glow
    draw_text(text, x + 4.0, y + 4.0, font_size as f32, Color::new(0.0, 0.0, 0.0, 0.6));
    draw_text(text, x, y, font_size as f32, color);
}
