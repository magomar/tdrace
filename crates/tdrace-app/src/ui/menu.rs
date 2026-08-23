use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use macroquad::text::{draw_text, measure_text};

use super::hud::format_lap_time;
use crate::render::color::Palette;

use serde::{Deserialize, Serialize};

/// Available track options in track selection menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackChoice {
    ClassicGrandPrix,
    OvalSpeedway,
    DriftPark,
    KartArena,
}

impl TrackChoice {
    pub const ALL: [Self; 4] = [
        Self::ClassicGrandPrix,
        Self::OvalSpeedway,
        Self::DriftPark,
        Self::KartArena,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::ClassicGrandPrix => "Classic Grand Prix",
            Self::OvalSpeedway => "Oval Speedway",
            Self::DriftPark => "Drift Park",
            Self::KartArena => "Kart Arena",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::ClassicGrandPrix => "Flowing GP circuit with high-speed chicane, hairpin sand trap & pit lane.",
            Self::OvalSpeedway => "High-speed banked superspeedway with concrete perimeter barriers.",
            Self::DriftPark => "Technical hairpin slides, wide transitions, and clipping point apexes.",
            Self::KartArena => "Tight, agile 90-degree corners, switchbacks, and aggressive rumble curbs.",
        }
    }
}

/// Available vehicle model options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CarChoice {
    SportsCar,
    DriftCar,
    Kart,
    RallyCar,
}

impl CarChoice {
    pub const ALL: [Self; 4] = [
        Self::SportsCar,
        Self::DriftCar,
        Self::Kart,
        Self::RallyCar,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::SportsCar => "GT Sports Coupe",
            Self::DriftCar => "Tuned Drift Spec",
            Self::Kart => "125cc Shifter Kart",
            Self::RallyCar => "AWD Turbo Rally",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::SportsCar => "Balanced RWD arcade physics, responsive steering, 208 km/h top speed.",
            Self::DriftCar => "High-power slide machine with loose rear, wide lock & quick counter-steer.",
            Self::Kart => "Ultra-lightweight direct steering with extreme apex cornering grip.",
            Self::RallyCar => "All-wheel-drive traction with compliant suspension for mixed surfaces.",
        }
    }
}

/// Racing game modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameModeChoice {
    SinglePlayerTimeAttack,
    RaceVsAiBots { num_bots: usize, total_laps: u32 },
}

/// Standings entry for results screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResultEntry {
    pub position: usize,
    pub car_name: String,
    pub is_player: bool,
    pub total_time: f32,
    pub best_lap: Option<f32>,
    pub delta_to_leader: f32,
}

use crate::audio::AudioSettings;
use tdrace_core::physics::config::AssistProfile;

/// Renders the Track & Setup Selection Menu.
pub fn render_track_select_menu(
    selected_track_idx: usize,
    selected_car_idx: usize,
    num_bots: usize,
    is_time_attack: bool,
    assist_profile: AssistProfile,
    audio_settings: &AudioSettings,
) {
    let sw = screen_width();
    let sh = screen_height();

    // Background backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.08, 0.10, 0.14, 0.95));

    // Header Title
    let title = "TDRACE - RETRO ARCADE RACER";
    let tm = measure_text(title, None, 42, 1.0);
    draw_text(title, (sw - tm.width) * 0.5, 60.0, 42.0, Color::new(0.95, 0.85, 0.15, 1.0));

    let subtitle = "GeneRally Inspired Cross-Platform 2D Physics & Visuals";
    let sm = measure_text(subtitle, None, 18, 1.0);
    draw_text(subtitle, (sw - sm.width) * 0.5, 90.0, 18.0, Color::new(0.70, 0.75, 0.85, 1.0));

    // Left Column: Track Selection
    let col1_x = sw * 0.12;
    let col_w = sw * 0.36;
    let mut curr_y = 135.0;

    draw_text("SELECT CIRCUIT [Up/Down]", col1_x, curr_y, 22.0, Color::new(0.3, 0.9, 1.0, 1.0));
    curr_y += 30.0;

    for (i, track_opt) in TrackChoice::ALL.iter().enumerate() {
        let is_sel = i == selected_track_idx;
        let box_h = 60.0;
        let bg_col = if is_sel {
            Color::new(0.20, 0.35, 0.55, 0.9)
        } else {
            Color::new(0.12, 0.15, 0.20, 0.8)
        };
        let border_col = if is_sel {
            Color::new(0.40, 0.85, 1.0, 1.0)
        } else {
            Color::new(0.25, 0.30, 0.40, 0.6)
        };

        draw_rectangle(col1_x, curr_y, col_w, box_h, bg_col);
        draw_rectangle_lines(col1_x, curr_y, col_w, box_h, 2.0, border_col);

        let title_col = if is_sel { Palette::WHITE } else { Color::new(0.8, 0.85, 0.9, 1.0) };
        draw_text(track_opt.title(), col1_x + 15.0, curr_y + 26.0, 20.0, title_col);
        draw_text(track_opt.description(), col1_x + 15.0, curr_y + 48.0, 12.0, Color::new(0.7, 0.75, 0.8, 1.0));

        curr_y += box_h + 12.0;
    }

    // Right Column: Car & Race Settings
    let col2_x = sw * 0.52;
    let mut c2_y = 135.0;

    draw_text("SELECT VEHICLE [Left/Right]", col2_x, c2_y, 22.0, Color::new(0.3, 0.9, 1.0, 1.0));
    c2_y += 30.0;

    for (i, car_opt) in CarChoice::ALL.iter().enumerate() {
        let is_sel = i == selected_car_idx;
        let box_h = 52.0;
        let bg_col = if is_sel {
            Color::new(0.35, 0.25, 0.50, 0.9)
        } else {
            Color::new(0.12, 0.15, 0.20, 0.8)
        };
        let border_col = if is_sel {
            Color::new(0.90, 0.45, 1.0, 1.0)
        } else {
            Color::new(0.25, 0.30, 0.40, 0.6)
        };

        draw_rectangle(col2_x, c2_y, col_w, box_h, bg_col);
        draw_rectangle_lines(col2_x, c2_y, col_w, box_h, 2.0, border_col);

        let title_col = if is_sel { Palette::WHITE } else { Color::new(0.8, 0.85, 0.9, 1.0) };
        draw_text(car_opt.title(), col2_x + 15.0, c2_y + 22.0, 18.0, title_col);
        draw_text(car_opt.description(), col2_x + 15.0, c2_y + 42.0, 12.0, Color::new(0.7, 0.75, 0.8, 1.0));

        c2_y += box_h + 8.0;
    }

    // Mode Setting & Bot Count
    c2_y += 4.0;
    draw_rectangle(col2_x, c2_y, col_w, 58.0, Color::new(0.10, 0.12, 0.18, 0.88));
    draw_rectangle_lines(col2_x, c2_y, col_w, 58.0, 1.5, Color::new(0.3, 0.4, 0.6, 0.7));

    let mode_str = if is_time_attack {
        "GAME MODE: [T] Time Attack (Solo Hotlap)".to_string()
    } else {
        format!("GAME MODE: [T] Race vs AI ({} Bots, [B] to toggle)", num_bots)
    };
    draw_text(&mode_str, col2_x + 15.0, c2_y + 24.0, 16.0, Color::new(0.95, 0.85, 0.2, 1.0));
    draw_text("Controls: [T] Toggle Mode | [B] Change Bot Count (1-7)", col2_x + 15.0, c2_y + 46.0, 12.0, Color::new(0.7, 0.75, 0.8, 1.0));

    // Driver Assists Profile Setting
    c2_y += 66.0;
    draw_rectangle(col2_x, c2_y, col_w, 60.0, Color::new(0.10, 0.12, 0.18, 0.88));
    draw_rectangle_lines(col2_x, c2_y, col_w, 60.0, 1.5, Color::new(0.2, 0.7, 0.9, 0.8));

    let assist_str = format!("DRIVE ASSISTS: [H] {}", assist_profile.title());
    let assist_col = match assist_profile {
        AssistProfile::Arcade => Color::new(0.3, 0.9, 1.0, 1.0),
        AssistProfile::Sport => Color::new(1.0, 0.75, 0.2, 1.0),
        AssistProfile::Pro => Color::new(1.0, 0.4, 0.4, 1.0),
    };
    draw_text(&assist_str, col2_x + 15.0, c2_y + 24.0, 16.0, assist_col);
    draw_text(assist_profile.description(), col2_x + 15.0, c2_y + 46.0, 12.0, Color::new(0.7, 0.75, 0.8, 1.0));

    // Audio Status & Quick Controls
    c2_y += 68.0;
    draw_rectangle(col2_x, c2_y, col_w, 48.0, Color::new(0.10, 0.12, 0.18, 0.88));
    draw_rectangle_lines(col2_x, c2_y, col_w, 48.0, 1.5, Color::new(0.45, 0.35, 0.75, 0.8));

    let mute_text = if audio_settings.is_muted { "MUTED" } else { "ACTIVE" };
    let mute_col = if audio_settings.is_muted { Color::new(1.0, 0.4, 0.4, 1.0) } else { Color::new(0.4, 0.9, 0.5, 1.0) };
    let vol_pct = (audio_settings.master_volume * 100.0).round() as i32;
    draw_text(&format!("AUDIO: [M] Mute [{}] | Vol: [ [ / ] ] {}%", mute_text, vol_pct), col2_x + 15.0, c2_y + 22.0, 15.0, mute_col);
    draw_text("Nightcall Synthwave Soundtrack & Dynamic SFX Engine", col2_x + 15.0, c2_y + 40.0, 11.0, Color::new(0.65, 0.70, 0.80, 1.0));

    // Footer Launch prompt
    let start_prompt = "PRESS [SPACE / ENTER] OR GAMEPAD [A / START] TO RACE";
    let pm = measure_text(start_prompt, None, 24, 1.0);
    let p_y = sh - 35.0;
    draw_rectangle((sw - pm.width) * 0.5 - 20.0, p_y - 26.0, pm.width + 40.0, 38.0, Color::new(0.1, 0.6, 0.25, 0.9));
    draw_text(start_prompt, (sw - pm.width) * 0.5, p_y, 24.0, Palette::WHITE);
}

/// Renders the Pause overlay with Assist Profile selection and Audio status.
pub fn render_pause_menu(assist_profile: AssistProfile, audio_settings: &AudioSettings) {
    let sw = screen_width();
    let sh = screen_height();

    // Dark semi-transparent dim
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.65));

    let box_w = 460.0;
    let box_h = 375.0;
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    draw_rectangle(x, y, box_w, box_h, Color::new(0.08, 0.10, 0.15, 0.95));
    draw_rectangle_lines(x, y, box_w, box_h, 2.5, Color::new(0.3, 0.6, 0.9, 0.9));

    let title = "RACE PAUSED";
    let tm = measure_text(title, None, 30, 1.0);
    draw_text(title, x + (box_w - tm.width) * 0.5, y + 38.0, 30.0, Palette::WHITE);

    let assist_item = format!("H / R3 : Toggle Assists [{}]", assist_profile.short_name());
    let mute_text = if audio_settings.is_muted { "MUTED" } else { "ACTIVE" };
    let vol_pct = (audio_settings.master_volume * 100.0).round() as i32;
    let audio_item = format!("M : Toggle Audio [{}] | [ / ] : Vol {}%", mute_text, vol_pct);

    let items = [
        "ESC / START / A : Resume Race".to_string(),
        "C / K : Controls & Gamepad Guide".to_string(),
        "R / Y : Restart Race".to_string(),
        "M / B : Main Menu / Track Select".to_string(),
        assist_item,
        audio_item,
        "TAB / Left Stick Click : Camera View".to_string(),
        "Q/A/O/P / Arrows / Stick & Triggers : Drive".to_string(),
        "SPACE / B : Handbrake | Z / LB : Reverse".to_string(),
    ];

    let mut item_y = y + 70.0;
    for item in &items {
        draw_text(item, x + 25.0, item_y, 14.5, Color::new(0.85, 0.90, 0.95, 1.0));
        item_y += 24.0;
    }
}

/// Renders the Race Results / Podium Standings screen.
pub fn render_results_screen(
    track_name: &str,
    results: &[RaceResultEntry],
    is_time_attack: bool,
) {
    let sw = screen_width();
    let sh = screen_height();

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.05, 0.07, 0.10, 0.92));

    let box_w = (sw * 0.75).clamp(500.0, 800.0);
    let box_h = (sh * 0.80).clamp(400.0, 650.0);
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    draw_rectangle(x, y, box_w, box_h, Color::new(0.08, 0.10, 0.16, 0.98));
    draw_rectangle_lines(x, y, box_w, box_h, 2.5, Color::new(0.95, 0.80, 0.15, 0.9));

    let title = if is_time_attack {
        "TIME ATTACK SESSION COMPLETE"
    } else {
        "RACE RESULTS & STANDINGS"
    };
    let tm = measure_text(title, None, 30, 1.0);
    draw_text(title, x + (box_w - tm.width) * 0.5, y + 45.0, 30.0, Color::new(1.0, 0.85, 0.15, 1.0));

    let track_label = format!("Circuit: {}", track_name);
    draw_text(&track_label, x + 30.0, y + 80.0, 18.0, Color::new(0.7, 0.8, 0.9, 1.0));

    // Table Header
    let mut row_y = y + 120.0;
    draw_rectangle(x + 20.0, row_y - 20.0, box_w - 40.0, 28.0, Color::new(0.15, 0.20, 0.30, 0.9));
    draw_text("POS", x + 35.0, row_y, 16.0, Palette::WHITE);
    draw_text("DRIVER / VEHICLE", x + 90.0, row_y, 16.0, Palette::WHITE);
    draw_text("TOTAL TIME", x + box_w - 320.0, row_y, 16.0, Palette::WHITE);
    draw_text("BEST LAP", x + box_w - 190.0, row_y, 16.0, Palette::WHITE);
    draw_text("GAP", x + box_w - 75.0, row_y, 16.0, Palette::WHITE);

    row_y += 24.0;

    for res in results {
        let (row_bg, text_col) = if res.is_player {
            (Color::new(0.20, 0.35, 0.20, 0.85), Color::new(0.95, 0.95, 0.2, 1.0))
        } else {
            (Color::new(0.10, 0.12, 0.18, 0.6), Color::new(0.85, 0.90, 0.95, 1.0))
        };

        draw_rectangle(x + 20.0, row_y - 16.0, box_w - 40.0, 28.0, row_bg);

        let pos_str = format!("P{}", res.position);
        draw_text(&pos_str, x + 35.0, row_y + 4.0, 16.0, text_col);
        draw_text(&res.car_name, x + 90.0, row_y + 4.0, 16.0, text_col);

        let total_str = format_lap_time(res.total_time);
        draw_text(&total_str, x + box_w - 320.0, row_y + 4.0, 16.0, text_col);

        let best_str = format_lap_time(res.best_lap.unwrap_or(0.0));
        draw_text(&best_str, x + box_w - 190.0, row_y + 4.0, 16.0, text_col);

        let gap_str = if res.position == 1 {
            "-".to_string()
        } else {
            format!("+{:.2}s", res.delta_to_leader)
        };
        draw_text(&gap_str, x + box_w - 75.0, row_y + 4.0, 16.0, text_col);

        row_y += 32.0;
    }

    // Bottom action prompt
    let prompt = "Press [SPACE] or [R] to Restart | [M] for Main Menu";
    let pm = measure_text(prompt, None, 18, 1.0);
    draw_text(prompt, x + (box_w - pm.width) * 0.5, y + box_h - 25.0, 18.0, Palette::WHITE);
}

/// Renders the full-screen Controls, Gamepad Mappings, and Assist Settings screen.
pub fn render_controls_screen(
    assist_profile: AssistProfile,
    gamepad_connected: bool,
    gamepad_name: &str,
) {
    let sw = screen_width();
    let sh = screen_height();

    // Background backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.06, 0.08, 0.12, 0.96));

    // Header Title
    let title = "CONTROLS & DRIVING ASSISTS";
    let tm = measure_text(title, None, 36, 1.0);
    draw_text(title, (sw - tm.width) * 0.5, 48.0, 36.0, Color::new(0.95, 0.85, 0.15, 1.0));

    let subtitle = "Keyboard & Gamepad Mappings | Electronic Vehicle Dynamics Configuration";
    let sm = measure_text(subtitle, None, 16, 1.0);
    draw_text(subtitle, (sw - sm.width) * 0.5, 74.0, 16.0, Color::new(0.70, 0.75, 0.85, 1.0));

    // Controller Status Banner
    let banner_w = sw * 0.80;
    let banner_x = (sw - banner_w) * 0.5;
    let banner_y = 92.0;
    let banner_h = 36.0;

    if gamepad_connected {
        draw_rectangle(banner_x, banner_y, banner_w, banner_h, Color::new(0.10, 0.25, 0.18, 0.90));
        draw_rectangle_lines(banner_x, banner_y, banner_w, banner_h, 1.5, Color::new(0.3, 0.9, 0.5, 1.0));
        let text = format!("🎮 ACTIVE GAMEPAD CONNECTED: {}", gamepad_name);
        draw_text(&text, banner_x + 18.0, banner_y + 24.0, 16.0, Color::new(0.3, 0.9, 0.5, 1.0));
    } else {
        draw_rectangle(banner_x, banner_y, banner_w, banner_h, Color::new(0.12, 0.14, 0.20, 0.90));
        draw_rectangle_lines(banner_x, banner_y, banner_w, banner_h, 1.5, Color::new(0.4, 0.5, 0.65, 0.8));
        let text = "🎮 NO GAMEPAD DETECTED — KEYBOARD & TOUCH ACTIVE (PLUG & PLAY READY)";
        draw_text(text, banner_x + 18.0, banner_y + 24.0, 15.0, Color::new(0.7, 0.75, 0.85, 1.0));
    }

    // Left Column: Keyboard Controls
    let col1_x = sw * 0.10;
    let col_w = sw * 0.38;
    let col_y = 142.0;
    let col_h = sh * 0.52;

    draw_rectangle(col1_x, col_y, col_w, col_h, Color::new(0.09, 0.11, 0.16, 0.92));
    draw_rectangle_lines(col1_x, col_y, col_w, col_h, 2.0, Color::new(0.3, 0.7, 0.9, 0.9));

    draw_text("KEYBOARD CONTROLS", col1_x + 16.0, col_y + 28.0, 20.0, Color::new(0.3, 0.9, 1.0, 1.0));

    let kb_rows = [
        ("Accelerate / Gas", "Q / Up Arrow"),
        ("Brake / Decelerate", "A / Down Arrow"),
        ("Steer Left / Right", "O / P or Left / Right"),
        ("Handbrake & Drift", "Spacebar"),
        ("Reverse Gear", "Z"),
        ("Cycle Assist Profile", "H"),
        ("Controls & Assists Guide", "C / K"),
        ("Camera View Toggle", "Tab"),
        ("Instant Session Reset", "R"),
        ("Pause / Resume", "Escape / Pause"),
        ("Audio Mute / Volume", "M / [ and ]"),
        ("Debug Overlays", "F1 - F5"),
    ];

    let mut row_y = col_y + 56.0;
    for (action, key) in &kb_rows {
        draw_text(action, col1_x + 18.0, row_y, 14.0, Color::new(0.80, 0.85, 0.92, 1.0));
        let km = measure_text(key, None, 14, 1.0);
        draw_text(key, col1_x + col_w - km.width - 18.0, row_y, 14.0, Color::new(0.95, 0.85, 0.2, 1.0));
        row_y += 22.0;
    }

    // Right Column: Gamepad Controls
    let col2_x = sw * 0.52;
    draw_rectangle(col2_x, col_y, col_w, col_h, Color::new(0.09, 0.11, 0.16, 0.92));
    draw_rectangle_lines(col2_x, col_y, col_w, col_h, 2.0, Color::new(0.85, 0.45, 0.95, 0.9));

    draw_text("GAMEPAD CONTROLS", col2_x + 16.0, col_y + 28.0, 20.0, Color::new(0.90, 0.45, 1.0, 1.0));

    let gp_rows = [
        ("Proportional Steering", "Left Analog Stick / D-Pad"),
        ("Analog Progressive Throttle", "Right Trigger (RT / R2) or A"),
        ("Analog Progressive Brake", "Left Trigger (LT / L2) or X"),
        ("Handbrake & Slide Initiation", "B / Circle or Right Bumper (RB)"),
        ("Reverse Gear", "Y / Triangle or Left Bumper (LB)"),
        ("Cycle Assist Profile", "Right Stick Click (R3) / Select"),
        ("Camera View Mode", "Left Stick Click (L3)"),
        ("Pause / Resume Menu", "Start / Menu Button"),
        ("Menu Navigation", "D-Pad / Left Stick"),
        ("Confirm / Start Race", "A / Cross Button"),
        ("Back / Cancel", "B / Circle Button"),
    ];

    let mut gp_row_y = col_y + 56.0;
    for (action, button) in &gp_rows {
        draw_text(action, col2_x + 18.0, gp_row_y, 14.0, Color::new(0.80, 0.85, 0.92, 1.0));
        let bm = measure_text(button, None, 14, 1.0);
        draw_text(button, col2_x + col_w - bm.width - 18.0, gp_row_y, 14.0, Color::new(0.4, 0.9, 0.6, 1.0));
        gp_row_y += 22.0;
    }

    // Bottom Panel: Active Drive Assists Profile
    let bot_y = col_y + col_h + 14.0;
    let bot_h = 90.0;
    draw_rectangle(col1_x, bot_y, banner_w, bot_h, Color::new(0.10, 0.12, 0.18, 0.95));
    draw_rectangle_lines(col1_x, bot_y, banner_w, bot_h, 2.0, Color::new(0.3, 0.7, 0.9, 0.8));

    let assist_title = format!("ACTIVE DRIVE ASSIST PROFILE: [H / R3] {}", assist_profile.title());
    let assist_col = match assist_profile {
        AssistProfile::Arcade => Color::new(0.3, 0.9, 1.0, 1.0),
        AssistProfile::Sport => Color::new(1.0, 0.75, 0.2, 1.0),
        AssistProfile::Pro => Color::new(1.0, 0.4, 0.4, 1.0),
    };
    draw_text(&assist_title, col1_x + 20.0, bot_y + 28.0, 18.0, assist_col);
    draw_text(assist_profile.description(), col1_x + 20.0, bot_y + 52.0, 14.0, Color::new(0.80, 0.85, 0.92, 1.0));
    draw_text("Press [H] on keyboard or [R3 / Select] on Gamepad to switch assist difficulty profile anytime!", col1_x + 20.0, bot_y + 74.0, 13.0, Color::new(0.65, 0.70, 0.80, 1.0));

    // Footer Return Prompt
    let back_prompt = "PRESS [ESC], [C], [SPACE] OR GAMEPAD [B / A] TO RETURN";
    let bpm = measure_text(back_prompt, None, 20, 1.0);
    let bp_y = sh - 22.0;
    draw_text(back_prompt, (sw - bpm.width) * 0.5, bp_y, 20.0, Palette::WHITE);
}
