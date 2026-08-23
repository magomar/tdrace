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

/// Renders the Track & Setup Selection Menu.
pub fn render_track_select_menu(
    selected_track_idx: usize,
    selected_car_idx: usize,
    num_bots: usize,
    is_time_attack: bool,
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

    draw_text("SELECT CIRCUIT [W/S or Up/Down]", col1_x, curr_y, 22.0, Color::new(0.3, 0.9, 1.0, 1.0));
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

    draw_text("SELECT VEHICLE [A/D or Left/Right]", col2_x, c2_y, 22.0, Color::new(0.3, 0.9, 1.0, 1.0));
    c2_y += 30.0;

    for (i, car_opt) in CarChoice::ALL.iter().enumerate() {
        let is_sel = i == selected_car_idx;
        let box_h = 55.0;
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
        draw_text(car_opt.title(), col2_x + 15.0, c2_y + 24.0, 19.0, title_col);
        draw_text(car_opt.description(), col2_x + 15.0, c2_y + 44.0, 12.0, Color::new(0.7, 0.75, 0.8, 1.0));

        c2_y += box_h + 10.0;
    }

    // Mode Setting & Bot Count
    c2_y += 10.0;
    draw_rectangle(col2_x, c2_y, col_w, 75.0, Color::new(0.10, 0.12, 0.18, 0.88));
    draw_rectangle_lines(col2_x, c2_y, col_w, 75.0, 1.5, Color::new(0.3, 0.4, 0.6, 0.7));

    let mode_str = if is_time_attack {
        "GAME MODE: [T] Time Attack (Solo Hotlap)".to_string()
    } else {
        format!("GAME MODE: [T] Race vs AI ({} Bots, [B] to toggle)", num_bots)
    };
    draw_text(&mode_str, col2_x + 15.0, c2_y + 30.0, 17.0, Color::new(0.95, 0.85, 0.2, 1.0));
    draw_text("Controls: [T] Toggle Mode | [B] Change Bot Count (1-7)", col2_x + 15.0, c2_y + 55.0, 13.0, Color::new(0.7, 0.75, 0.8, 1.0));

    // Footer Launch prompt
    let start_prompt = "PRESS [SPACE] OR [ENTER] TO START RACE";
    let pm = measure_text(start_prompt, None, 26, 1.0);
    let p_y = sh - 45.0;
    draw_rectangle((sw - pm.width) * 0.5 - 20.0, p_y - 28.0, pm.width + 40.0, 40.0, Color::new(0.1, 0.6, 0.25, 0.9));
    draw_text(start_prompt, (sw - pm.width) * 0.5, p_y, 26.0, Palette::WHITE);
}

/// Renders the Pause overlay.
pub fn render_pause_menu() {
    let sw = screen_width();
    let sh = screen_height();

    // Dark semi-transparent dim
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.65));

    let box_w = 400.0;
    let box_h = 280.0;
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    draw_rectangle(x, y, box_w, box_h, Color::new(0.08, 0.10, 0.15, 0.95));
    draw_rectangle_lines(x, y, box_w, box_h, 2.5, Color::new(0.3, 0.6, 0.9, 0.9));

    let title = "RACE PAUSED";
    let tm = measure_text(title, None, 30, 1.0);
    draw_text(title, x + (box_w - tm.width) * 0.5, y + 40.0, 30.0, Palette::WHITE);

    let items = [
        "ESC / P : Resume Race",
        "R : Restart Race",
        "M : Main Menu / Track Select",
        "TAB : Toggle Follow / Overview Camera",
        "WASD / Arrows : Drive & Steer",
        "SPACE : Handbrake (Power Drift)",
    ];

    let mut item_y = y + 78.0;
    for item in &items {
        draw_text(item, x + 25.0, item_y, 16.0, Color::new(0.85, 0.90, 0.95, 1.0));
        item_y += 28.0;
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
