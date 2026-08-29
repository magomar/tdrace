use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};

use super::font::Fonts;
use super::hud::format_lap_time;
use super::scaler::UiScaler;
use crate::audio::AudioSettings;
use crate::render::color::Palette;
use tdrace_core::physics::config::AssistProfile;

/// Available track options in track selection menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackChoice {
    ClassicGrandPrix,
    OvalSpeedway,
    DriftPark,
    KartArena,
    RampRaceway,
    OasisRally,
    OutlawPass,
    Custom { id: String, title: String, description: String, path: String },
}

impl TrackChoice {
    pub const ALL: [Self; 7] = [
        Self::ClassicGrandPrix,
        Self::OvalSpeedway,
        Self::DriftPark,
        Self::KartArena,
        Self::RampRaceway,
        Self::OasisRally,
        Self::OutlawPass,
    ];

    pub fn title(&self) -> &str {
        match self {
            Self::ClassicGrandPrix => "Classic Grand Prix",
            Self::OvalSpeedway => "Oval Speedway",
            Self::DriftPark => "Drift Park",
            Self::KartArena => "Kart Arena",
            Self::RampRaceway => "Ramp Raceway",
            Self::OasisRally => "Oasis Rally",
            Self::OutlawPass => "Outlaw Pass",
            Self::Custom { title, .. } => title.as_str(),
        }
    }

    pub fn tag(&self) -> &str {
        match self {
            Self::ClassicGrandPrix => "FIA GP CIRCUIT",
            Self::OvalSpeedway => "SUPERSPEEDWAY",
            Self::DriftPark => "TECHNICAL DRIFT",
            Self::KartArena => "AGILE SPRINT",
            Self::RampRaceway => "STUNT RAMPS & JUMPS",
            Self::OasisRally => "DESERT DIRT RALLY",
            Self::OutlawPass => "NARROW MOUNTAIN PASS",
            Self::Custom { .. } => "CUSTOM CIRCUIT",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::ClassicGrandPrix => "High-speed sweeping chicanes, hairpin sand traps & tactical pit lane.",
            Self::OvalSpeedway => "Full-throttle banked superspeedway surrounded by concrete barriers.",
            Self::DriftPark => "Technical hairpin slides, wide transitions & dynamic apex clipping zones.",
            Self::KartArena => "Tight 90-degree corners, rapid switchbacks & aggressive rumble curbs.",
            Self::RampRaceway => "High-speed stadium circuit with launch ramps, hazard water puddles, gap jumps & banked turns.",
            Self::OasisRally => "Pure dirt desert rally circuit with oasis water hazards, perilous sand traps & high-sliding rally dynamics.",
            Self::OutlawPass => "Perilous mountain circuit carving through a dramatic narrow canyon pass with tight switchbacks and cliff rock walls.",
            Self::Custom { description, .. } => {
                if description.trim().is_empty() {
                    "User-created custom racing circuit."
                } else {
                    description.as_str()
                }
            }
        }
    }

    pub fn track_id(&self) -> &str {
        match self {
            Self::ClassicGrandPrix => "classic_grand_prix",
            Self::OvalSpeedway => "oval_speedway",
            Self::DriftPark => "drift_park",
            Self::KartArena => "kart_arena",
            Self::RampRaceway => "ramp_raceway",
            Self::OasisRally => "oasis_rally",
            Self::OutlawPass => "outlaw_pass",
            Self::Custom { id, .. } => id.as_str(),
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom { .. })
    }
}

/// Resolves a TrackChoice to a concrete Track instance for UI preview rendering.
pub fn resolve_track_for_menu(choice: &TrackChoice) -> Option<tdrace_core::track::Track> {
    match choice {
        TrackChoice::ClassicGrandPrix => Some(tdrace_core::track::presets::classic_grand_prix()),
        TrackChoice::OvalSpeedway => Some(tdrace_core::track::presets::oval_speedway()),
        TrackChoice::DriftPark => Some(tdrace_core::track::presets::drift_park()),
        TrackChoice::KartArena => Some(tdrace_core::track::presets::kart_arena()),
        TrackChoice::RampRaceway => Some(tdrace_core::track::presets::ramp_raceway()),
        TrackChoice::OasisRally => Some(tdrace_core::track::presets::oasis_rally()),
        TrackChoice::OutlawPass => Some(tdrace_core::track::presets::outlaw_pass()),
        TrackChoice::Custom { id, path, .. } => {
            if id == "monza" {
                return Some(crate::module::f1::F1GameModule::track_monza());
            }
            if id == "spa" {
                return Some(crate::module::f1::F1GameModule::track_spa());
            }
            if id == "silverstone" {
                return Some(crate::module::f1::F1GameModule::track_silverstone());
            }
            if id == "sahara" {
                return Some(tdrace_core::track::presets::sahara_dunes());
            }
            tdrace_core::track::Track::load_from_file(path).ok()
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

    pub fn tag(&self) -> &'static str {
        match self {
            Self::SportsCar => "BALANCED RWD",
            Self::DriftCar => "PRO SLIDE",
            Self::Kart => "APEX GRIP",
            Self::RallyCar => "AWD ALL-TERRAIN",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::SportsCar => "Balanced RWD arcade dynamics, responsive rack, 208 km/h top speed.",
            Self::DriftCar => "High-power slide machine with loose rear, wide lock & snappy counter-steer.",
            Self::Kart => "Ultra-lightweight direct steering with extreme apex cornering grip.",
            Self::RallyCar => "All-wheel-drive traction with compliant suspension for mixed surfaces.",
        }
    }

    /// Returns normalized stat ratings: (Speed, Acceleration, Grip, Drift) [0.0..1.0]
    pub fn stats(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::SportsCar => (0.85, 0.80, 0.75, 0.65),
            Self::DriftCar => (0.80, 0.85, 0.50, 0.98),
            Self::Kart => (0.65, 0.95, 0.95, 0.40),
            Self::RallyCar => (0.78, 0.90, 0.85, 0.75),
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

use crate::profile::{PlayerProfile, ProfileCareerStats};
use super::profile_ui::render_profile_badge;

/// Renders the modern Track & Setup Selection Menu with glass cards and vector typography.
#[allow(clippy::too_many_arguments)]
pub fn render_track_select_menu(
    fonts: &Fonts,
    active_module_id: &str,
    module_title: &str,
    module_subtitle: &str,
    module_accent: Color,
    available_tracks: &[TrackChoice],
    available_vehicles: &[(&str, &str, &str, (f32, f32, f32, f32))],
    selected_track_idx: usize,
    selected_car_idx: usize,
    num_bots: usize,
    is_time_attack: bool,
    assist_profile: AssistProfile,
    audio_settings: &AudioSettings,
    active_profile: &PlayerProfile,
    active_stats: &ProfileCareerStats,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep modern motorsport gradient backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.05, 0.06, 0.09, 0.98));

    // Header Title
    fonts.draw_display_centered_with_shadow(
        module_title,
        sw * 0.5,
        scaler.s(32.0),
        scaler.font_s(32.0),
        module_accent,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let sub_str = format!("{} • [G / TAB] Switch Motorsport Hub", module_subtitle);
    fonts.draw_ui_regular_centered(
        &sub_str,
        sw * 0.5,
        scaler.s(52.0),
        scaler.font_s(13.0),
        Palette::UI_TEXT_MUTED,
    );

    // Columns geometry
    let col_w = (sw * 0.40).clamp(scaler.s(320.0), scaler.s(480.0));
    let col1_x = (sw * 0.5 - col_w - scaler.s(16.0)).max(scaler.safe_pad_x);
    let col2_x = (sw * 0.5 + scaler.s(16.0)).min(sw - col_w - scaler.safe_pad_x);

    // Active Profile Badge Banner
    let badge_w = col_w * 2.0 + scaler.s(32.0);
    let badge_x = col1_x;
    let badge_y = scaler.s(62.0);
    let badge_h = scaler.s(48.0);
    render_profile_badge(fonts, &scaler, badge_x, badge_y, badge_w, badge_h, active_profile, active_stats);

    // Explicit spacing between Profile Panel and Track / Car selection columns
    let menu_content_y = badge_y + badge_h + scaler.s(18.0);

    // Left Column: Track Selection Cards
    let mut curr_y = menu_content_y;

    fonts.draw_ui_bold(
        "SELECT CIRCUIT [Up/Down] | [T] Track Manager | [E] Studio",
        col1_x,
        curr_y + scaler.s(13.0),
        scaler.font_s(15.0),
        module_accent,
    );
    curr_y += scaler.s(22.0);

    let total_tracks = available_tracks.len();
    let total_items = total_tracks + 1; // +1 for the dedicated Track Manager entry
    let max_visible = 7;
    let start_idx = if total_items <= max_visible {
        0
    } else {
        selected_track_idx
            .saturating_sub(max_visible / 2)
            .min(total_items - max_visible)
    };
    let end_idx = (start_idx + max_visible).min(total_items);

    for i in start_idx..end_idx {
        let is_sel = i == selected_track_idx;
        let box_h = scaler.s(58.0);

        if i < total_tracks {
            let track_opt = &available_tracks[i];
            let loaded_track = resolve_track_for_menu(track_opt);

            let bg_col = if is_sel {
                Palette::UI_CARD_BG_HOVER
            } else {
                Palette::UI_CARD_BG
            };
            let border_col = if is_sel {
                module_accent
            } else {
                Palette::UI_CARD_BORDER
            };

            scaler.draw_glass_card(col1_x, curr_y, col_w, box_h, bg_col, border_col, if is_sel { 2.2 } else { 1.2 });

            // Small Track Vector Thumbnail on right side of card
            let thumb_w = scaler.s(58.0);
            let thumb_h = scaler.s(44.0);
            let thumb_x = col1_x + col_w - thumb_w - scaler.s(8.0);
            let thumb_y = curr_y + scaler.s(7.0);

            if let Some(ref tr) = loaded_track {
                super::track_preview::render_track_thumbnail(&scaler, thumb_x, thumb_y, thumb_w, thumb_h, tr, is_sel);
            }

            // Tag pill & metrics badge (Length + Surface breakdown)
            let tag_col = if is_sel { module_accent } else { Palette::UI_TEXT_MUTED };
            let tag_label = if let Some(ref tr) = loaded_track {
                format!("{} • {:.0}m • {}", track_opt.tag(), tr.total_length_m(), tr.surface_summary_string())
            } else {
                track_opt.tag().to_string()
            };
            fonts.draw_ui_bold(
                &tag_label,
                col1_x + scaler.s(14.0),
                curr_y + scaler.s(16.0),
                scaler.font_s(10.0),
                tag_col,
            );

            // Track title
            let title_col = if is_sel { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 1.0) };
            fonts.draw_ui_bold(
                track_opt.title(),
                col1_x + scaler.s(14.0),
                curr_y + scaler.s(34.0),
                scaler.font_s(15.5),
                title_col,
            );

            // Description
            fonts.draw_ui_regular(
                track_opt.description(),
                col1_x + scaler.s(14.0),
                curr_y + scaler.s(49.0),
                scaler.font_s(10.5),
                Palette::UI_TEXT_MUTED,
            );
        } else {
            // Dedicated Track Manager Card with distinct background color
            let tm_bg = if is_sel {
                Color::new(0.32, 0.12, 0.52, 0.95)
            } else {
                Color::new(0.18, 0.08, 0.30, 0.88)
            };
            let tm_border = if is_sel {
                Palette::NEON_GOLD
            } else {
                Palette::NEON_MAGENTA
            };

            scaler.draw_glass_card(col1_x, curr_y, col_w, box_h, tm_bg, tm_border, if is_sel { 2.4 } else { 1.5 });

            fonts.draw_ui_bold(
                "CIRCUIT HUB & WORKSHOP [T]",
                col1_x + scaler.s(14.0),
                curr_y + scaler.s(16.0),
                scaler.font_s(10.5),
                if is_sel { Palette::NEON_GOLD } else { Palette::NEON_MAGENTA },
            );

            fonts.draw_ui_bold(
                "Track Manager",
                col1_x + scaler.s(14.0),
                curr_y + scaler.s(34.0),
                scaler.font_s(16.0),
                Palette::WHITE,
            );

            fonts.draw_ui_regular(
                "Manage approved tracks, promote drafts & edit circuit info.",
                col1_x + scaler.s(14.0),
                curr_y + scaler.s(49.0),
                scaler.font_s(11.0),
                if is_sel { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
            );
        }

        curr_y += box_h + scaler.s(6.0);
    }

    // Right Column: Vehicle & Dynamics Settings
    let mut c2_y = menu_content_y;

    fonts.draw_ui_bold(
        "SELECT VEHICLE [Left/Right]",
        col2_x,
        c2_y + scaler.s(13.0),
        scaler.font_s(16.0),
        module_accent,
    );
    c2_y += scaler.s(22.0);

    for (i, &(v_title, v_tag, v_desc, (spd, acc, grip, drift))) in available_vehicles.iter().enumerate() {
        let is_sel = i == selected_car_idx;
        let box_h = scaler.s(58.0);
        let bg_col = if is_sel {
            Palette::UI_CARD_BG_HOVER
        } else {
            Palette::UI_CARD_BG
        };
        let border_col = if is_sel {
            module_accent
        } else {
            Palette::UI_CARD_BORDER
        };

        scaler.draw_glass_card(col2_x, c2_y, col_w, box_h, bg_col, border_col, if is_sel { 2.2 } else { 1.2 });

        // Tag pill
        let tag_col = if is_sel { module_accent } else { Palette::UI_TEXT_MUTED };
        fonts.draw_ui_bold(
            v_tag,
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(16.0),
            scaler.font_s(11.0),
            tag_col,
        );

        let title_col = if is_sel { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 1.0) };
        fonts.draw_ui_bold(
            v_title,
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(34.0),
            scaler.font_s(17.0),
            title_col,
        );

        // Stats mini-bars on right of card
        let stat_x = col2_x + col_w - scaler.s(120.0);
        render_stat_bar(scaler, fonts, stat_x, c2_y + scaler.s(12.0), "SPD", spd, Palette::NEON_CYAN);
        render_stat_bar(scaler, fonts, stat_x, c2_y + scaler.s(24.0), "ACC", acc, Palette::NEON_GOLD);
        render_stat_bar(scaler, fonts, stat_x, c2_y + scaler.s(36.0), "GRP", grip, Palette::NEON_GREEN);
        render_stat_bar(scaler, fonts, stat_x, c2_y + scaler.s(48.0), "DFT", drift, Palette::NEON_MAGENTA);

        fonts.draw_ui_regular(
            v_desc,
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(49.0),
            scaler.font_s(11.5),
            Palette::UI_TEXT_MUTED,
        );

        c2_y += box_h + scaler.s(8.0);
    }

    // Championship / Season Action Banner
    let (champ_title, champ_sub, champ_col) = match active_module_id {
        "f1" => ("[F] START FIA FORMULA 1 WORLD CHAMPIONSHIP (4 ROUNDS)", "Official Season • Qualifying • Standings • 25pt Matrix", Palette::RED),
        "rally" => ("[F] START WRC STAGE RALLY TIME TRIAL", "Timed Stages • Split Sectors • Time Penalties", Palette::NEON_GOLD),
        "kart" => ("[F] START SPRINT ELIMINATION CUP", "Knockout Heats • Last Place Elimination", Palette::NEON_GREEN),
        _ => ("[G / TAB] MOTORSPORT GRAND HUB", "Switch between F1, Rally, Karting & Classic Arcade", Palette::NEON_CYAN),
    };
    c2_y += scaler.s(2.0);
    let champ_h = scaler.s(48.0);
    scaler.draw_glass_card(col2_x, c2_y, col_w, champ_h, Palette::UI_CARD_BG, champ_col, 1.8);
    fonts.draw_ui_bold(champ_title, col2_x + scaler.s(14.0), c2_y + scaler.s(20.0), scaler.font_s(13.5), champ_col);
    fonts.draw_ui_regular(champ_sub, col2_x + scaler.s(14.0), c2_y + scaler.s(38.0), scaler.font_s(11.5), Palette::UI_TEXT_MUTED);
    c2_y += champ_h + scaler.s(8.0);

    // Mode Setting & Bot Count Card
    let mode_h = scaler.s(52.0);
    scaler.draw_glass_card(col2_x, c2_y, col_w, mode_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.5);

    let mode_str = if is_time_attack {
        "GAME MODE: [X] Time Attack (Solo Hotlap)".to_string()
    } else {
        format!("GAME MODE: [X] Race vs AI ({} Bots, [B] to toggle)", num_bots)
    };
    fonts.draw_ui_bold(
        &mode_str,
        col2_x + scaler.s(14.0),
        c2_y + scaler.s(20.0),
        scaler.font_s(14.5),
        Palette::NEON_GOLD,
    );
    fonts.draw_ui_regular(
        "Mode: [X] | Bots: [B] | [D] Driver Dossier & Roster",
        col2_x + scaler.s(14.0),
        c2_y + scaler.s(38.0),
        scaler.font_s(11.5),
        Palette::UI_TEXT_MUTED,
    );

    // Driver Assists Profile Setting Card
    c2_y += mode_h + scaler.s(8.0);
    let assist_h = scaler.s(52.0);
    scaler.draw_glass_card(col2_x, c2_y, col_w, assist_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.5);

    let assist_str = format!("DRIVE ASSISTS: [H] {}", assist_profile.title());
    let assist_col = match assist_profile {
        AssistProfile::Arcade => Palette::NEON_CYAN,
        AssistProfile::Sport => Palette::NEON_GOLD,
        AssistProfile::Pro => Palette::RED,
    };
    fonts.draw_ui_bold(
        &assist_str,
        col2_x + scaler.s(14.0),
        c2_y + scaler.s(20.0),
        scaler.font_s(14.5),
        assist_col,
    );
    fonts.draw_ui_regular(
        assist_profile.description(),
        col2_x + scaler.s(14.0),
        c2_y + scaler.s(38.0),
        scaler.font_s(11.5),
        Palette::UI_TEXT_MUTED,
    );

    // Audio Status Card
    c2_y += assist_h + scaler.s(8.0);
    let audio_h = scaler.s(46.0);
    scaler.draw_glass_card(col2_x, c2_y, col_w, audio_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.5);

    let mute_text = if audio_settings.is_muted { "MUTED" } else { "ACTIVE" };
    let mute_col = if audio_settings.is_muted { Palette::RED } else { Palette::NEON_GREEN };
    let vol_pct = (audio_settings.master_volume * 100.0).round() as i32;
    fonts.draw_ui_bold(
        &format!("AUDIO: [M] Mute [{}] | Volume: [ [ / ] ] {}%", mute_text, vol_pct),
        col2_x + scaler.s(14.0),
        c2_y + scaler.s(20.0),
        scaler.font_s(14.0),
        mute_col,
    );
    fonts.draw_ui_regular(
        "Nightcall Synthwave Soundtrack & Dynamic SFX Engine",
        col2_x + scaler.s(14.0),
        c2_y + scaler.s(36.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    // Footer Launch prompt button
    let start_prompt = "PRESS [SPACE / ENTER] OR GAMEPAD [A / START] TO RACE";
    let btn_w = scaler.s(460.0);
    let btn_h = scaler.s(40.0);
    let btn_x = (sw - btn_w) * 0.5;
    let btn_y = sh - btn_h - scaler.s(14.0);

    draw_rectangle(btn_x, btn_y, btn_w, btn_h, Color::new(0.12, 0.65, 0.32, 0.95));
    draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, Palette::NEON_GREEN);

    fonts.draw_ui_bold_centered(
        start_prompt,
        sw * 0.5,
        btn_y + scaler.s(25.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );
}

fn render_stat_bar(
    scaler: UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    label: &str,
    val: f32,
    color: Color,
) {
    fonts.draw_ui_bold(label, x, y + scaler.s(7.0), scaler.font_s(9.0), Palette::UI_TEXT_MUTED);
    let bar_x = x + scaler.s(24.0);
    let bar_w = scaler.s(80.0);
    let bar_h = scaler.s(6.0);

    draw_rectangle(bar_x, y, bar_w, bar_h, Color::new(0.1, 0.12, 0.16, 0.9));
    draw_rectangle(bar_x, y, bar_w * val.clamp(0.0, 1.0), bar_h, color);
}

/// Bounding rectangles for interactive Pause Menu action buttons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PauseMenuButtonLayout {
    pub resume_rect: (f32, f32, f32, f32),
    pub exit_rect: (f32, f32, f32, f32),
}

/// Calculates the layout dimensions and button rectangles for the pause menu overlay.
pub fn pause_menu_layout(sw: f32, sh: f32) -> (f32, f32, f32, f32, PauseMenuButtonLayout) {
    let scaler = UiScaler::new(sw, sh);
    let box_w = scaler.s(500.0);
    let box_h = scaler.s(345.0);
    let box_x = (sw - box_w) * 0.5;
    let box_y = (sh - box_h) * 0.5;

    let pad_x = scaler.s(22.0);
    let gap = scaler.s(16.0);
    let btn_w = (box_w - pad_x * 2.0 - gap) * 0.5;
    let btn_h = scaler.s(48.0);
    let btn_y = box_y + scaler.s(58.0);

    let resume_rect = (box_x + pad_x, btn_y, btn_w, btn_h);
    let exit_rect = (box_x + pad_x + btn_w + gap, btn_y, btn_w, btn_h);

    (
        box_x,
        box_y,
        box_w,
        box_h,
        PauseMenuButtonLayout {
            resume_rect,
            exit_rect,
        },
    )
}

/// Renders the modern Pause overlay with Assist Profile selection, Audio status, and Resume/Exit buttons.
pub fn render_pause_menu(fonts: &Fonts, assist_profile: AssistProfile, audio_settings: &AudioSettings) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Dark semi-transparent dim
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.70));

    let (box_x, box_y, box_w, box_h, btn_layout) = pause_menu_layout(sw, sh);

    scaler.draw_glass_card(box_x, box_y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.2);

    let title = "RACE PAUSED";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        box_y + scaler.s(38.0),
        scaler.font_s(30.0),
        Palette::WHITE,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let (mx, my) = std::panic::catch_unwind(macroquad::input::mouse_position).unwrap_or((-1000.0, -1000.0));

    // Resume button
    let (rx, ry, rw, rh) = btn_layout.resume_rect;
    let is_resume_hovered = mx >= rx && mx <= rx + rw && my >= ry && my <= ry + rh;
    let resume_bg = if is_resume_hovered {
        Color::new(0.12, 0.52, 0.28, 0.95)
    } else {
        Color::new(0.08, 0.36, 0.20, 0.90)
    };
    let resume_border = if is_resume_hovered {
        Palette::NEON_GREEN
    } else {
        Color::new(0.20, 0.75, 0.40, 0.85)
    };
    draw_rectangle(rx, ry, rw, rh, resume_bg);
    draw_rectangle_lines(
        rx,
        ry,
        rw,
        rh,
        if is_resume_hovered { 2.4 * scaler.scale } else { 1.8 * scaler.scale },
        resume_border,
    );
    fonts.draw_ui_bold_centered(
        "[ESC] RESUME RACE",
        rx + rw * 0.5,
        ry + scaler.s(21.0),
        scaler.font_s(14.5),
        Palette::WHITE,
    );
    fonts.draw_ui_regular_centered(
        "Continue | Gamepad Start / A",
        rx + rw * 0.5,
        ry + scaler.s(37.0),
        scaler.font_s(11.0),
        Color::new(0.75, 0.95, 0.80, 0.90),
    );

    // Exit button
    let (ex, ey, ew, eh) = btn_layout.exit_rect;
    let is_exit_hovered = mx >= ex && mx <= ex + ew && my >= ey && my <= ey + eh;
    let exit_bg = if is_exit_hovered {
        Color::new(0.55, 0.12, 0.16, 0.95)
    } else {
        Color::new(0.38, 0.08, 0.12, 0.90)
    };
    let exit_border = if is_exit_hovered {
        Palette::RED
    } else {
        Color::new(0.85, 0.25, 0.28, 0.85)
    };
    draw_rectangle(ex, ey, ew, eh, exit_bg);
    draw_rectangle_lines(
        ex,
        ey,
        ew,
        eh,
        if is_exit_hovered { 2.4 * scaler.scale } else { 1.8 * scaler.scale },
        exit_border,
    );
    fonts.draw_ui_bold_centered(
        "[E] EXIT RACE",
        ex + ew * 0.5,
        ey + scaler.s(21.0),
        scaler.font_s(14.5),
        Palette::WHITE,
    );
    fonts.draw_ui_regular_centered(
        "Main Menu | Gamepad B",
        ex + ew * 0.5,
        ey + scaler.s(37.0),
        scaler.font_s(11.0),
        Color::new(0.95, 0.75, 0.75, 0.90),
    );

    // Divider line
    let div_y = ry + rh + scaler.s(14.0);
    draw_rectangle(
        box_x + scaler.s(20.0),
        div_y,
        box_w - scaler.s(40.0),
        scaler.s(1.0),
        Color::new(0.2, 0.3, 0.45, 0.5),
    );

    let assist_item = format!("H / R3 : Toggle Assists [{}]", assist_profile.short_name());
    let mute_text = if audio_settings.is_muted { "MUTED" } else { "ACTIVE" };
    let vol_pct = (audio_settings.master_volume * 100.0).round() as i32;
    let audio_item = format!("M : Toggle Audio [{}] | [ / ] : Vol {}%", mute_text, vol_pct);

    let items = [
        assist_item,
        audio_item,
        "D : Driver Cards & Opponents Dossier".to_string(),
        "C / K : Controls & Gamepad Guide".to_string(),
        "R / Y : Restart Race".to_string(),
        "TAB / Left Stick Click : Camera View".to_string(),
        "Q/A/O/P / Arrows / Stick & Triggers : Drive".to_string(),
        "SPACE / B : Handbrake | Z / LB : Reverse".to_string(),
    ];

    let mut item_y = div_y + scaler.s(22.0);
    for item in &items {
        fonts.draw_ui_bold(
            item,
            box_x + scaler.s(24.0),
            item_y,
            scaler.font_s(13.5),
            Color::new(0.85, 0.90, 0.98, 1.0),
        );
        item_y += scaler.s(22.5);
    }
}

/// Renders the Race Results / Podium Standings screen with modern leaderboard cards.
pub fn render_results_screen(
    fonts: &Fonts,
    track_name: &str,
    results: &[RaceResultEntry],
    is_time_attack: bool,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.94));

    let box_w = (sw * 0.80).clamp(scaler.s(520.0), scaler.s(850.0));
    let box_h = (sh * 0.82).clamp(scaler.s(420.0), scaler.s(680.0));
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_GOLD, 2.5);

    let title = if is_time_attack {
        "TIME ATTACK SESSION COMPLETE"
    } else {
        "RACE RESULTS & STANDINGS"
    };
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        y + scaler.s(44.0),
        scaler.font_s(32.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let track_label = format!("Circuit: {}", track_name);
    fonts.draw_ui_bold(
        &track_label,
        x + scaler.s(28.0),
        y + scaler.s(76.0),
        scaler.font_s(16.0),
        Palette::UI_TEXT_MUTED,
    );

    // Table Header
    let mut row_y = y + scaler.s(108.0);
    let hdr_h = scaler.s(28.0);
    draw_rectangle(x + scaler.s(20.0), row_y - scaler.s(20.0), box_w - scaler.s(40.0), hdr_h, Color::new(0.12, 0.16, 0.25, 0.9));
    fonts.draw_ui_bold("POS", x + scaler.s(32.0), row_y, scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_bold("DRIVER / VEHICLE", x + scaler.s(85.0), row_y, scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_bold("TOTAL TIME", x + box_w - scaler.s(320.0), row_y, scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_bold("BEST LAP", x + box_w - scaler.s(190.0), row_y, scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_bold("GAP", x + box_w - scaler.s(75.0), row_y, scaler.font_s(14.0), Palette::WHITE);

    row_y += scaler.s(24.0);

    for res in results {
        let (row_bg, text_col) = if res.is_player {
            (Color::new(0.18, 0.35, 0.22, 0.90), Palette::NEON_GREEN)
        } else {
            (Color::new(0.09, 0.11, 0.16, 0.70), Color::new(0.85, 0.90, 0.95, 1.0))
        };

        draw_rectangle(x + scaler.s(20.0), row_y - scaler.s(16.0), box_w - scaler.s(40.0), scaler.s(28.0), row_bg);

        // Position medal icon or text
        let pos_str = match res.position {
            1 => "P1".to_string(),
            2 => "P2".to_string(),
            3 => "P3".to_string(),
            _ => format!("P{}", res.position),
        };
        fonts.draw_ui_bold(&pos_str, x + scaler.s(28.0), row_y + scaler.s(4.0), scaler.font_s(14.0), text_col);
        fonts.draw_ui_bold(&res.car_name, x + scaler.s(85.0), row_y + scaler.s(4.0), scaler.font_s(14.0), text_col);

        let total_str = format_lap_time(res.total_time);
        fonts.draw_ui_bold(&total_str, x + box_w - scaler.s(320.0), row_y + scaler.s(4.0), scaler.font_s(14.0), text_col);

        let best_str = format_lap_time(res.best_lap.unwrap_or(0.0));
        fonts.draw_ui_bold(&best_str, x + box_w - scaler.s(190.0), row_y + scaler.s(4.0), scaler.font_s(14.0), text_col);

        let gap_str = if res.position == 1 {
            "-".to_string()
        } else {
            format!("+{:.2}s", res.delta_to_leader)
        };
        fonts.draw_ui_bold(&gap_str, x + box_w - scaler.s(75.0), row_y + scaler.s(4.0), scaler.font_s(14.0), text_col);

        row_y += scaler.s(32.0);
    }

    // Bottom action prompt
    let prompt = "Press [SPACE] or [R] to Restart | [M] for Main Menu";
    fonts.draw_ui_bold_centered(
        prompt,
        sw * 0.5,
        y + box_h - scaler.s(22.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );
}

/// Renders the full-screen Controls, Gamepad Mappings, and Assist Settings screen.
pub fn render_controls_screen(
    fonts: &Fonts,
    assist_profile: AssistProfile,
    gamepad_connected: bool,
    gamepad_name: &str,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Background backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.05, 0.06, 0.09, 0.98));

    // Header Title
    let title = "CONTROLS & DRIVING ASSISTS";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        scaler.s(44.0),
        scaler.font_s(36.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let subtitle = "Keyboard & Gamepad Mappings | Electronic Vehicle Dynamics Configuration";
    fonts.draw_ui_regular_centered(
        subtitle,
        sw * 0.5,
        scaler.s(68.0),
        scaler.font_s(15.0),
        Palette::UI_TEXT_MUTED,
    );

    // Controller Status Banner
    let banner_w = sw * 0.82;
    let banner_x = (sw - banner_w) * 0.5;
    let banner_y = scaler.s(85.0);
    let banner_h = scaler.s(34.0);

    if gamepad_connected {
        draw_rectangle(banner_x, banner_y, banner_w, banner_h, Color::new(0.08, 0.22, 0.15, 0.90));
        draw_rectangle_lines(banner_x, banner_y, banner_w, banner_h, 1.5, Palette::NEON_GREEN);
        let text = format!("ACTIVE GAMEPAD DETECTED: {}", gamepad_name);
        fonts.draw_ui_bold(&text, banner_x + scaler.s(16.0), banner_y + scaler.s(22.0), scaler.font_s(14.0), Palette::NEON_GREEN);
    } else {
        draw_rectangle(banner_x, banner_y, banner_w, banner_h, Color::new(0.10, 0.12, 0.18, 0.90));
        draw_rectangle_lines(banner_x, banner_y, banner_w, banner_h, 1.5, Palette::UI_CARD_BORDER);
        let text = "NO GAMEPAD DETECTED — KEYBOARD & TOUCH ACTIVE (PLUG & PLAY READY)";
        fonts.draw_ui_bold(text, banner_x + scaler.s(16.0), banner_y + scaler.s(22.0), scaler.font_s(14.0), Palette::UI_TEXT_MUTED);
    }

    // Left Column: Keyboard Controls
    let col_w = (sw * 0.40).clamp(scaler.s(320.0), scaler.s(480.0));
    let col1_x = (sw * 0.5 - col_w - scaler.s(14.0)).max(scaler.safe_pad_x);
    let col_y = scaler.s(130.0);
    let col_h = sh * 0.54;

    scaler.draw_glass_card(col1_x, col_y, col_w, col_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 1.8);
    fonts.draw_ui_bold("KEYBOARD CONTROLS", col1_x + scaler.s(16.0), col_y + scaler.s(26.0), scaler.font_s(18.0), Palette::NEON_CYAN);

    let kb_rows = [
        ("Accelerate / Gas", "Q / Up Arrow"),
        ("Brake / Decelerate", "A / Down Arrow"),
        ("Steer Left / Right", "O / P or Left / Right"),
        ("Handbrake & Drift", "Spacebar"),
        ("Reverse Gear", "Z"),
        ("Cycle Assist Profile", "H"),
        ("Controls & Assists Guide", "C / K"),
        ("Cycle Camera Zoom", "Tab"),
        ("Instant Session Reset", "R"),
        ("Pause / Resume", "Escape / Pause"),
        ("Audio Mute / Volume", "M / [ and ]"),
        ("Debug Overlays", "F1 - F5"),
    ];

    let mut row_y = col_y + scaler.s(52.0);
    for (action, key) in &kb_rows {
        fonts.draw_ui_regular(action, col1_x + scaler.s(16.0), row_y, scaler.font_s(13.0), Color::new(0.80, 0.85, 0.92, 1.0));
        let km = fonts.measure_ui_bold(key, scaler.font_s(13.0));
        fonts.draw_ui_bold(key, col1_x + col_w - km.width - scaler.s(16.0), row_y, scaler.font_s(13.0), Palette::NEON_GOLD);
        row_y += scaler.s(21.0);
    }

    // Right Column: Gamepad Controls
    let col2_x = (sw * 0.5 + scaler.s(14.0)).min(sw - col_w - scaler.safe_pad_x);
    scaler.draw_glass_card(col2_x, col_y, col_w, col_h, Palette::UI_CARD_BG, Palette::NEON_MAGENTA, 1.8);
    fonts.draw_ui_bold("GAMEPAD CONTROLS", col2_x + scaler.s(16.0), col_y + scaler.s(26.0), scaler.font_s(18.0), Palette::NEON_MAGENTA);

    let gp_rows = [
        ("Proportional Steering", "Left Analog Stick / D-Pad"),
        ("Analog Progressive Throttle", "Right Trigger (RT / R2)"),
        ("Analog Progressive Brake", "Left Trigger (LT / L2)"),
        ("Handbrake & Slide Initiation", "A / Cross Button (or RB)"),
        ("Reverse Gear", "X / Square Button (or LB)"),
        ("Cycle Assist Profile", "Right Stick Click (R3) / Select"),
        ("Cycle Camera Zoom", "Left Stick Click (L3)"),
        ("Pause / Resume Menu", "Start / Menu Button"),
        ("Menu Navigation", "D-Pad / Left Stick"),
        ("Confirm / Start Race", "A / Cross Button (Enter)"),
        ("Back / Cancel", "B / Circle Button (Escape)"),
    ];

    let mut gp_row_y = col_y + scaler.s(52.0);
    for (action, button) in &gp_rows {
        fonts.draw_ui_regular(action, col2_x + scaler.s(16.0), gp_row_y, scaler.font_s(13.0), Color::new(0.80, 0.85, 0.92, 1.0));
        let bm = fonts.measure_ui_bold(button, scaler.font_s(13.0));
        fonts.draw_ui_bold(button, col2_x + col_w - bm.width - scaler.s(16.0), gp_row_y, scaler.font_s(13.0), Palette::NEON_GREEN);
        gp_row_y += scaler.s(21.0);
    }

    // Bottom Panel: Active Drive Assists Profile
    let bot_y = col_y + col_h + scaler.s(12.0);
    let bot_h = scaler.s(85.0);
    scaler.draw_glass_card(banner_x, bot_y, banner_w, bot_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.5);

    let assist_title = format!("ACTIVE DRIVE ASSIST PROFILE: [H / R3] {}", assist_profile.title());
    let assist_col = match assist_profile {
        AssistProfile::Arcade => Palette::NEON_CYAN,
        AssistProfile::Sport => Palette::NEON_GOLD,
        AssistProfile::Pro => Palette::RED,
    };
    fonts.draw_ui_bold(&assist_title, banner_x + scaler.s(18.0), bot_y + scaler.s(24.0), scaler.font_s(16.0), assist_col);
    fonts.draw_ui_regular(assist_profile.description(), banner_x + scaler.s(18.0), bot_y + scaler.s(48.0), scaler.font_s(13.0), Color::new(0.80, 0.85, 0.92, 1.0));
    fonts.draw_ui_regular("Press [H] on keyboard or [R3 / Select] on Gamepad to switch assist difficulty profile anytime!", banner_x + scaler.s(18.0), bot_y + scaler.s(68.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);

    // Footer Return Prompt
    let back_prompt = "PRESS [ESC], [C], [SPACE] OR GAMEPAD [B / A] TO RETURN";
    fonts.draw_ui_bold_centered(
        back_prompt,
        sw * 0.5,
        sh - scaler.s(18.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );
}

use crate::tournament::ChampionshipSession;

/// Renders the Motorsport Grand Hub Module Selection Menu.
pub fn render_module_select_menu(
    fonts: &Fonts,
    selected_idx: usize,
    modules: &[(&str, &str, &str, &str, Color)], // (id, title, tag, description, color)
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Header
    fonts.draw_display_centered_with_shadow(
        "TDRACE MOTORSPORT GRAND HUB",
        sw * 0.5,
        scaler.s(45.0),
        scaler.font_s(36.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    fonts.draw_ui_regular_centered(
        "Select a Motorsport Championship Category or Circuit Studio Workshop",
        sw * 0.5,
        scaler.s(68.0),
        scaler.font_s(14.0),
        Palette::UI_TEXT_MUTED,
    );

    let card_w = (sw * 0.72).clamp(scaler.s(480.0), scaler.s(720.0));
    let card_h = scaler.s(90.0);
    let card_x = (sw - card_w) * 0.5;
    let mut curr_y = scaler.s(95.0);

    for (i, (_id, title, tag, desc, accent_col)) in modules.iter().enumerate() {
        let is_sel = i == selected_idx;
        let bg_col = if is_sel {
            Palette::UI_CARD_BG_HOVER
        } else {
            Palette::UI_CARD_BG
        };
        let border_col = if is_sel {
            *accent_col
        } else {
            Palette::UI_CARD_BORDER
        };

        scaler.draw_glass_card(card_x, curr_y, card_w, card_h, bg_col, border_col, if is_sel { 2.4 } else { 1.2 });

        // Left accent bar
        if is_sel {
            draw_rectangle(card_x, curr_y, scaler.s(6.0), card_h, *accent_col);
        }

        // Tag
        fonts.draw_ui_bold(
            tag,
            card_x + scaler.s(20.0),
            curr_y + scaler.s(22.0),
            scaler.font_s(11.0),
            if is_sel { *accent_col } else { Palette::UI_TEXT_MUTED },
        );

        // Title
        fonts.draw_display(
            title,
            card_x + scaler.s(20.0),
            curr_y + scaler.s(48.0),
            scaler.font_s(20.0),
            if is_sel { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 1.0) },
        );

        // Description
        fonts.draw_ui_regular(
            desc,
            card_x + scaler.s(20.0),
            curr_y + scaler.s(72.0),
            scaler.font_s(12.5),
            if is_sel { Color::new(0.80, 0.85, 0.92, 1.0) } else { Palette::UI_TEXT_MUTED },
        );

        curr_y += card_h + scaler.s(14.0);
    }

    // Footer prompt
    let prompt = "USE [UP/DOWN] TO NAVIGATE | [ENTER/SPACE] LAUNCH MOTORSPORT | [ESC] BACK";
    fonts.draw_ui_bold_centered(
        prompt,
        sw * 0.5,
        sh - scaler.s(24.0),
        scaler.font_s(14.0),
        Palette::NEON_CYAN,
    );
}

/// Renders the Championship Standings table and round victory screen.
pub fn render_championship_standings_screen(
    fonts: &Fonts,
    champ: &ChampionshipSession,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Header
    let title = champ.name.to_uppercase();
    fonts.draw_display_centered_with_shadow(
        &title,
        sw * 0.5,
        scaler.s(45.0),
        scaler.font_s(32.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let subtitle = if champ.is_completed {
        "SEASON FINALE — CHAMPIONSHIP DECIDED!".to_string()
    } else {
        format!("ROUND {} OF {} COMPLETED — DRIVER STANDINGS", champ.current_round, champ.total_rounds())
    };
    fonts.draw_ui_bold_centered(
        &subtitle,
        sw * 0.5,
        scaler.s(70.0),
        scaler.font_s(14.0),
        if champ.is_completed { Palette::NEON_GREEN } else { Palette::NEON_CYAN },
    );

    // Standings Table Card
    let table_w = (sw * 0.80).clamp(scaler.s(520.0), scaler.s(850.0));
    let table_x = (sw - table_w) * 0.5;
    let table_y = scaler.s(90.0);
    let table_h = scaler.s(420.0);

    scaler.draw_glass_card(table_x, table_y, table_w, table_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.5);

    // Table Header Row
    let mut row_y = table_y + scaler.s(28.0);
    fonts.draw_ui_bold("POS", table_x + scaler.s(20.0), row_y, scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
    fonts.draw_ui_bold("DRIVER", table_x + scaler.s(70.0), row_y, scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
    fonts.draw_ui_bold("TEAM / CAR", table_x + scaler.s(280.0), row_y, scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
    fonts.draw_ui_bold("WINS", table_x + table_w - scaler.s(160.0), row_y, scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
    fonts.draw_ui_bold("POINTS", table_x + table_w - scaler.s(75.0), row_y, scaler.font_s(13.0), Palette::NEON_GOLD);

    draw_rectangle(table_x + scaler.s(15.0), row_y + scaler.s(8.0), table_w - scaler.s(30.0), 1.0, Palette::UI_CARD_BORDER);
    row_y += scaler.s(24.0);

    // Table Rows
    for (i, entry) in champ.standings.iter().enumerate().take(10) {
        let pos_str = format!("#{}", i + 1);
        let pos_col = match i {
            0 => Palette::NEON_GOLD,
            1 => Palette::WHITE,
            2 => Palette::NEON_MAGENTA,
            _ => Palette::UI_TEXT_MUTED,
        };

        fonts.draw_ui_bold(&pos_str, table_x + scaler.s(20.0), row_y, scaler.font_s(14.0), pos_col);
        fonts.draw_ui_bold(&entry.driver_name, table_x + scaler.s(70.0), row_y, scaler.font_s(14.0), Palette::WHITE);
        fonts.draw_ui_regular(&entry.team_name, table_x + scaler.s(280.0), row_y, scaler.font_s(13.0), Color::new(0.75, 0.80, 0.88, 1.0));
        fonts.draw_ui_bold(&entry.wins.to_string(), table_x + table_w - scaler.s(150.0), row_y, scaler.font_s(14.0), Palette::WHITE);
        fonts.draw_display(&format!("{} PTS", entry.points), table_x + table_w - scaler.s(85.0), row_y, scaler.font_s(15.0), Palette::NEON_GOLD);

        row_y += scaler.s(28.0);
    }

    // Bottom Action Prompt
    let next_prompt = if champ.is_completed {
        "SEASON COMPLETE! PRESS [ENTER/SPACE] OR GAMEPAD [A] TO RETURN TO MENU"
    } else {
        "PRESS [ENTER/SPACE] OR GAMEPAD [A] TO START NEXT ROUND | [ESC] EXIT"
    };

    fonts.draw_ui_bold_centered(
        next_prompt,
        sw * 0.5,
        sh - scaler.s(26.0),
        scaler.font_s(15.0),
        Palette::NEON_GREEN,
    );
}

/// Renders the exit confirmation modal overlay when pressing Escape or Gamepad B on the Main Menu.
pub fn render_exit_confirm_modal(fonts: &Fonts) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Dark backdrop overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.78));

    let mw = scaler.s(480.0).min(sw - scaler.s(32.0));
    let mh = scaler.s(200.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::RED, 2.2);

    fonts.draw_ui_bold(
        "QUIT TDRACE",
        mx + scaler.s(24.0),
        my + scaler.s(36.0),
        scaler.font_s(20.0),
        Palette::RED,
    );

    fonts.draw_ui_regular(
        "Are you sure you want to exit the application?",
        mx + scaler.s(24.0),
        my + scaler.s(72.0),
        scaler.font_s(15.0),
        Palette::WHITE,
    );

    fonts.draw_ui_regular(
        "Any unsaved progress will be lost.",
        mx + scaler.s(24.0),
        my + scaler.s(96.0),
        scaler.font_s(12.5),
        Palette::UI_TEXT_MUTED,
    );

    let btn_y = my + mh - scaler.s(32.0);
    fonts.draw_ui_bold(
        "[ENTER / SPACE / A] YES, QUIT",
        mx + scaler.s(24.0),
        btn_y,
        scaler.font_s(14.0),
        Palette::RED,
    );
    fonts.draw_ui_bold(
        "[ESC / B] CANCEL",
        mx + mw - scaler.s(140.0),
        btn_y,
        scaler.font_s(14.0),
        Palette::NEON_CYAN,
    );
}

