use std::collections::BTreeMap;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};

use crate::render::color::{CarColorScheme, Palette};
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;

/// Country metadata including ISO alpha-3 code, English name, and emoji representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountryInfo {
    pub code: &'static str,
    pub name: &'static str,
    pub flag_emoji: &'static str,
}

/// Registry of predefined countries with visual banners for motorsport driver profiles.
pub struct CountryRegistry;

impl CountryRegistry {
    pub const ALL: [CountryInfo; 17] = [
        CountryInfo { code: "ESP", name: "Spain", flag_emoji: "🇪🇸" },
        CountryInfo { code: "USA", name: "United States", flag_emoji: "🇺🇸" },
        CountryInfo { code: "GBR", name: "United Kingdom", flag_emoji: "🇬🇧" },
        CountryInfo { code: "DEU", name: "Germany", flag_emoji: "🇩🇪" },
        CountryInfo { code: "FRA", name: "France", flag_emoji: "🇫🇷" },
        CountryInfo { code: "ITA", name: "Italy", flag_emoji: "🇮🇹" },
        CountryInfo { code: "JPN", name: "Japan", flag_emoji: "🇯🇵" },
        CountryInfo { code: "BRA", name: "Brazil", flag_emoji: "🇧🇷" },
        CountryInfo { code: "CAN", name: "Canada", flag_emoji: "🇨🇦" },
        CountryInfo { code: "AUS", name: "Australia", flag_emoji: "🇦🇺" },
        CountryInfo { code: "NLD", name: "Netherlands", flag_emoji: "🇳🇱" },
        CountryInfo { code: "SWE", name: "Sweden", flag_emoji: "🇸🇪" },
        CountryInfo { code: "MEX", name: "Mexico", flag_emoji: "🇲🇽" },
        CountryInfo { code: "ARG", name: "Argentina", flag_emoji: "🇦🇷" },
        CountryInfo { code: "MCO", name: "Monaco", flag_emoji: "🇲🇨" },
        CountryInfo { code: "BEL", name: "Belgium", flag_emoji: "🇧🇪" },
        CountryInfo { code: "FIN", name: "Finland", flag_emoji: "🇫🇮" },
    ];

    /// Finds country metadata by ISO code.
    pub fn find_by_code(code: &str) -> Option<CountryInfo> {
        Self::ALL.iter().copied().find(|c| c.code.eq_ignore_ascii_case(code))
    }
}

/// Player Profile representing driver identity, livery customizations, and nationality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub id: Option<i64>,
    pub name: String,
    pub alias: String,
    pub country: Option<String>,
    pub color_scheme: CarColorScheme,
    pub is_active: bool,
    pub created_at: String,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            id: None,
            name: "Racer One".to_string(),
            alias: "Apex Hunter".to_string(),
            country: Some("ESP".to_string()),
            color_scheme: CarColorScheme::from_index(0),
            is_active: true,
            created_at: String::new(),
        }
    }
}

impl PlayerProfile {
    pub fn new(name: &str, alias: &str, country: Option<&str>, color_scheme: CarColorScheme) -> Self {
        Self {
            id: None,
            name: name.trim().to_string(),
            alias: alias.trim().to_string(),
            country: country.map(|s| s.trim().to_uppercase()),
            color_scheme,
            is_active: false,
            created_at: String::new(),
        }
    }

    /// Returns display country name or fallback.
    pub fn country_name(&self) -> &str {
        match &self.country {
            Some(code) => CountryRegistry::find_by_code(code)
                .map(|c| c.name)
                .unwrap_or("International"),
            None => "International",
        }
    }

    /// Returns country flag emoji or fallback icon.
    pub fn country_emoji(&self) -> &str {
        match &self.country {
            Some(code) => CountryRegistry::find_by_code(code)
                .map(|c| c.flag_emoji)
                .unwrap_or("🏁"),
            None => "🏁",
        }
    }
}

/// Aggregated career statistics computed from persistent race history logs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProfileCareerStats {
    pub total_races: u32,
    pub wins: u32,
    pub podiums: u32,
    pub total_laps: u32,
    pub win_rate: f32,
    pub podium_rate: f32,
    pub best_times: BTreeMap<String, f32>,
}

impl ProfileCareerStats {
    pub fn compute(races: &[RaceHistoryEntry]) -> Self {
        let mut total_races = 0u32;
        let mut wins = 0u32;
        let mut podiums = 0u32;
        let mut total_laps = 0u32;
        let mut best_times: BTreeMap<String, f32> = BTreeMap::new();

        for race in races {
            total_races += 1;
            total_laps += race.laps;

            if race.position == 1 {
                wins += 1;
            }
            if race.position >= 1 && race.position <= 3 {
                podiums += 1;
            }

            if let Some(lap) = race.best_lap {
                let current_best = best_times.entry(race.track_id.clone()).or_insert(lap);
                if lap < *current_best {
                    *current_best = lap;
                }
            }
        }

        let win_rate = if total_races > 0 {
            (wins as f32 / total_races as f32) * 100.0
        } else {
            0.0
        };

        let podium_rate = if total_races > 0 {
            (podiums as f32 / total_races as f32) * 100.0
        } else {
            0.0
        };

        Self {
            total_races,
            wins,
            podiums,
            total_laps,
            win_rate,
            podium_rate,
            best_times,
        }
    }
}

/// Individual race completion entry logged in the history database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaceHistoryEntry {
    pub id: Option<i64>,
    pub profile_id: i64,
    pub track_id: String,
    pub car_name: String,
    pub position: usize,
    pub total_cars: usize,
    pub total_time: f32,
    pub best_lap: Option<f32>,
    pub laps: u32,
    pub is_time_attack: bool,
    pub created_at: String,
}

/// Renders a vector country flag banner and code badge at `(x, y)` with given dimensions.
pub fn draw_country_banner(
    country_code: Option<&str>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    fonts: Option<&Fonts>,
    scaler: &UiScaler,
) {
    let flag_w = (w * 0.55).min(h * 1.6);
    let flag_h = h;

    // Draw flag background & border
    draw_rectangle(x, y, flag_w, flag_h, Color::new(0.10, 0.12, 0.16, 0.95));
    draw_rectangle_lines(x, y, flag_w, flag_h, 1.0, Palette::UI_CARD_BORDER);

    let code_upper = country_code.map(|s| s.to_ascii_uppercase());
    let code_str = code_upper.as_deref().unwrap_or("INT");

    match code_str {
        "ESP" => {
            // Spain: Red / Yellow (2x) / Red horizontal stripes
            let stripe_h = flag_h / 4.0;
            draw_rectangle(x, y, flag_w, stripe_h, Color::new(0.85, 0.12, 0.15, 1.0));
            draw_rectangle(x, y + stripe_h, flag_w, stripe_h * 2.0, Color::new(0.98, 0.82, 0.10, 1.0));
            draw_rectangle(x, y + stripe_h * 3.0, flag_w, stripe_h, Color::new(0.85, 0.12, 0.15, 1.0));
        }
        "USA" => {
            // USA: Red/White horizontal stripes + Blue canton
            let num_stripes = 5;
            let sh = flag_h / num_stripes as f32;
            for i in 0..num_stripes {
                let col = if i % 2 == 0 {
                    Color::new(0.85, 0.14, 0.18, 1.0)
                } else {
                    Palette::WHITE
                };
                draw_rectangle(x, y + i as f32 * sh, flag_w, sh, col);
            }
            // Blue canton
            let canton_w = flag_w * 0.45;
            let canton_h = flag_h * 0.60;
            draw_rectangle(x, y, canton_w, canton_h, Color::new(0.12, 0.22, 0.55, 1.0));
        }
        "GBR" => {
            // UK: Deep blue with white & red cross
            draw_rectangle(x, y, flag_w, flag_h, Color::new(0.08, 0.18, 0.45, 1.0));
            // White broad cross
            let cw = flag_w * 0.28;
            let ch = flag_h * 0.28;
            draw_rectangle(x + (flag_w - cw) * 0.5, y, cw, flag_h, Palette::WHITE);
            draw_rectangle(x, y + (flag_h - ch) * 0.5, flag_w, ch, Palette::WHITE);
            // Red inner cross
            let rcw = flag_w * 0.16;
            let rch = flag_h * 0.16;
            draw_rectangle(x + (flag_w - rcw) * 0.5, y, rcw, flag_h, Color::new(0.85, 0.12, 0.18, 1.0));
            draw_rectangle(x, y + (flag_h - rch) * 0.5, flag_w, rch, Color::new(0.85, 0.12, 0.18, 1.0));
        }
        "DEU" => {
            // Germany: Black / Red / Gold horizontal
            let sh = flag_h / 3.0;
            draw_rectangle(x, y, flag_w, sh, Color::new(0.10, 0.10, 0.12, 1.0));
            draw_rectangle(x, y + sh, flag_w, sh, Color::new(0.88, 0.15, 0.15, 1.0));
            draw_rectangle(x, y + sh * 2.0, flag_w, sh, Color::new(0.98, 0.80, 0.10, 1.0));
        }
        "FRA" => {
            // France: Blue / White / Red vertical
            let sw = flag_w / 3.0;
            draw_rectangle(x, y, sw, flag_h, Color::new(0.08, 0.22, 0.65, 1.0));
            draw_rectangle(x + sw, y, sw, flag_h, Palette::WHITE);
            draw_rectangle(x + sw * 2.0, y, sw, flag_h, Color::new(0.92, 0.14, 0.18, 1.0));
        }
        "ITA" => {
            // Italy: Green / White / Red vertical
            let sw = flag_w / 3.0;
            draw_rectangle(x, y, sw, flag_h, Color::new(0.08, 0.58, 0.25, 1.0));
            draw_rectangle(x + sw, y, sw, flag_h, Palette::WHITE);
            draw_rectangle(x + sw * 2.0, y, sw, flag_h, Color::new(0.92, 0.14, 0.18, 1.0));
        }
        "JPN" => {
            // Japan: White field with Red central sun disc
            draw_rectangle(x, y, flag_w, flag_h, Palette::WHITE);
            let radius = (flag_h * 0.30).min(flag_w * 0.30);
            draw_circle(x + flag_w * 0.5, y + flag_h * 0.5, radius, Color::new(0.85, 0.12, 0.18, 1.0));
        }
        "BRA" => {
            // Brazil: Green with Yellow rhombus and Blue circle
            draw_rectangle(x, y, flag_w, flag_h, Color::new(0.08, 0.58, 0.25, 1.0));
            let center_x = x + flag_w * 0.5;
            let center_y = y + flag_h * 0.5;
            let radius = flag_h * 0.24;
            draw_circle(center_x, center_y, radius, Color::new(0.10, 0.22, 0.62, 1.0));
        }
        "CAN" => {
            // Canada: Red / White / Red vertical
            let side_w = flag_w * 0.28;
            let mid_w = flag_w - (side_w * 2.0);
            draw_rectangle(x, y, side_w, flag_h, Color::new(0.88, 0.12, 0.15, 1.0));
            draw_rectangle(x + side_w, y, mid_w, flag_h, Palette::WHITE);
            draw_rectangle(x + flag_w - side_w, y, side_w, flag_h, Color::new(0.88, 0.12, 0.15, 1.0));
            // Small red center accent
            draw_circle(x + flag_w * 0.5, y + flag_h * 0.5, flag_h * 0.16, Color::new(0.88, 0.12, 0.15, 1.0));
        }
        "AUS" => {
            // Australia: Deep blue with British canton and star accents
            draw_rectangle(x, y, flag_w, flag_h, Color::new(0.06, 0.14, 0.40, 1.0));
            draw_rectangle(x, y, flag_w * 0.45, flag_h * 0.50, Color::new(0.85, 0.15, 0.18, 1.0));
            draw_circle(x + flag_w * 0.72, y + flag_h * 0.5, flag_h * 0.12, Palette::WHITE);
        }
        "NLD" => {
            // Netherlands: Red / White / Blue horizontal
            let sh = flag_h / 3.0;
            draw_rectangle(x, y, flag_w, sh, Color::new(0.85, 0.14, 0.18, 1.0));
            draw_rectangle(x, y + sh, flag_w, sh, Palette::WHITE);
            draw_rectangle(x, y + sh * 2.0, flag_w, sh, Color::new(0.12, 0.28, 0.65, 1.0));
        }
        "SWE" => {
            // Sweden: Blue with Yellow Nordic cross
            draw_rectangle(x, y, flag_w, flag_h, Color::new(0.08, 0.38, 0.75, 1.0));
            let cw = flag_w * 0.18;
            let ch = flag_h * 0.20;
            let cross_x = x + flag_w * 0.32;
            draw_rectangle(cross_x, y, cw, flag_h, Color::new(0.98, 0.82, 0.10, 1.0));
            draw_rectangle(x, y + (flag_h - ch) * 0.5, flag_w, ch, Color::new(0.98, 0.82, 0.10, 1.0));
        }
        "MEX" => {
            // Mexico: Green / White / Red vertical
            let sw = flag_w / 3.0;
            draw_rectangle(x, y, sw, flag_h, Color::new(0.06, 0.48, 0.25, 1.0));
            draw_rectangle(x + sw, y, sw, flag_h, Palette::WHITE);
            draw_rectangle(x + sw * 2.0, y, sw, flag_h, Color::new(0.85, 0.14, 0.18, 1.0));
        }
        "ARG" => {
            // Argentina: Light Blue / White / Light Blue horizontal
            let sh = flag_h / 3.0;
            let arg_blue = Color::new(0.45, 0.70, 0.95, 1.0);
            draw_rectangle(x, y, flag_w, sh, arg_blue);
            draw_rectangle(x, y + sh, flag_w, sh, Palette::WHITE);
            draw_circle(x + flag_w * 0.5, y + flag_h * 0.5, flag_h * 0.10, Color::new(0.95, 0.75, 0.10, 1.0));
            draw_rectangle(x, y + sh * 2.0, flag_w, sh, arg_blue);
        }
        "MCO" => {
            // Monaco: Red / White horizontal
            let sh = flag_h * 0.5;
            draw_rectangle(x, y, flag_w, sh, Color::new(0.88, 0.12, 0.15, 1.0));
            draw_rectangle(x, y + sh, flag_w, sh, Palette::WHITE);
        }
        "BEL" => {
            // Belgium: Black / Yellow / Red vertical
            let sw = flag_w / 3.0;
            draw_rectangle(x, y, sw, flag_h, Color::new(0.12, 0.12, 0.14, 1.0));
            draw_rectangle(x + sw, y, sw, flag_h, Color::new(0.98, 0.82, 0.10, 1.0));
            draw_rectangle(x + sw * 2.0, y, sw, flag_h, Color::new(0.88, 0.14, 0.16, 1.0));
        }
        "FIN" => {
            // Finland: White with Blue Nordic cross
            draw_rectangle(x, y, flag_w, flag_h, Palette::WHITE);
            let cw = flag_w * 0.20;
            let ch = flag_h * 0.22;
            let cross_x = x + flag_w * 0.32;
            draw_rectangle(cross_x, y, cw, flag_h, Color::new(0.08, 0.25, 0.65, 1.0));
            draw_rectangle(x, y + (flag_h - ch) * 0.5, flag_w, ch, Color::new(0.08, 0.25, 0.65, 1.0));
        }
        _ => {
            // International / Motorsport checkered pattern
            let cols = 4;
            let rows = 2;
            let cw = flag_w / cols as f32;
            let rh = flag_h / rows as f32;
            for r in 0..rows {
                for c in 0..cols {
                    let col = if (r + c) % 2 == 0 {
                        Color::new(0.12, 0.14, 0.18, 1.0)
                    } else {
                        Palette::WHITE
                    };
                    draw_rectangle(x + c as f32 * cw, y + r as f32 * rh, cw, rh, col);
                }
            }
        }
    }

    // Border around flag
    draw_rectangle_lines(x, y, flag_w, flag_h, 1.0, Palette::WHITE);

    // Render ISO Code badge if fonts provided
    if let Some(f) = fonts {
        let badge_x = x + flag_w + scaler.s(4.0);
        let badge_w = w - flag_w - scaler.s(4.0);
        if badge_w >= scaler.s(16.0) {
            f.draw_ui_bold(
                code_str,
                badge_x,
                y + flag_h * 0.72,
                scaler.font_s(11.0),
                Palette::NEON_GOLD,
            );
        }
    }
}
