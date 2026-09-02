use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Country metadata including ISO alpha-3 code, English name, and emoji representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountryInfo {
    pub code: &'static str,
    pub name: &'static str,
    pub flag_emoji: &'static str,
}

/// Registry of predefined countries with visual banners.
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
            let canton_w = flag_w * 0.45;
            let canton_h = flag_h * 0.60;
            draw_rectangle(x, y, canton_w, canton_h, Color::new(0.12, 0.22, 0.55, 1.0));
        }
        "GBR" => {
            // UK: Deep blue with white & red cross
            draw_rectangle(x, y, flag_w, flag_h, Color::new(0.08, 0.18, 0.45, 1.0));
            let cw = flag_w * 0.28;
            let ch = flag_h * 0.28;
            draw_rectangle(x + (flag_w - cw) * 0.5, y, cw, flag_h, Palette::WHITE);
            draw_rectangle(x, y + (flag_h - ch) * 0.5, flag_w, ch, Palette::WHITE);
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
        _ => {
            // International checkered pattern
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

    draw_rectangle_lines(x, y, flag_w, flag_h, 1.0, Palette::WHITE);

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
