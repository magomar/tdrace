use macroquad::color::Color;

/// High-visibility arcade design tokens and palette constants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette;

impl Palette {
    // Basic Constants
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Color = Color::new(0.95, 0.20, 0.20, 1.0);
    pub const GREEN: Color = Color::new(0.18, 0.90, 0.35, 1.0);
    pub const BLUE: Color = Color::new(0.20, 0.55, 1.0, 1.0);
    pub const YELLOW: Color = Color::new(1.0, 0.85, 0.15, 1.0);

    // Neon Accents
    pub const NEON_CYAN: Color = Color::new(0.20, 0.90, 1.0, 1.0);
    pub const NEON_GOLD: Color = Color::new(1.0, 0.82, 0.15, 1.0);
    pub const NEON_MAGENTA: Color = Color::new(1.0, 0.25, 0.80, 1.0);
    pub const NEON_GREEN: Color = Color::new(0.20, 0.95, 0.45, 1.0);
    pub const NEON_ORANGE: Color = Color::new(1.0, 0.50, 0.10, 1.0);

    // Glassmorphism & UI Backdrops
    pub const UI_CARD_BG: Color = Color::new(0.07, 0.09, 0.14, 0.92);
    pub const UI_CARD_BG_HOVER: Color = Color::new(0.12, 0.16, 0.24, 0.95);
    pub const UI_CARD_BORDER: Color = Color::new(0.25, 0.35, 0.50, 0.85);
    pub const UI_CARD_BORDER_GLOW: Color = Color::new(0.35, 0.80, 1.0, 1.0);
    pub const UI_TEXT_MUTED: Color = Color::new(0.65, 0.72, 0.82, 1.0);
    pub const UI_PILL_BG: Color = Color::new(0.10, 0.13, 0.20, 0.90);

    // Shadows
    pub const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.42);
    pub const SOFT_SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.22);
}

/// Converts a macroquad Color into an 8-character hex string (#RRGGBBAA).
pub fn color_to_hex(c: Color) -> String {
    let r = (c.r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (c.g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (c.b.clamp(0.0, 1.0) * 255.0).round() as u8;
    let a = (c.a.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
}

/// Parses a hex string (#RRGGBB or #RRGGBBAA) into a macroquad Color.
pub fn hex_to_color(hex: &str) -> Color {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let Ok(v) = u32::from_str_radix(clean, 16) {
            let r = ((v >> 16) & 0xFF) as f32 / 255.0;
            let g = ((v >> 8) & 0xFF) as f32 / 255.0;
            let b = (v & 0xFF) as f32 / 255.0;
            return Color::new(r, g, b, 1.0);
        }
    } else if clean.len() == 8 {
        if let Ok(v) = u32::from_str_radix(clean, 16) {
            let r = ((v >> 24) & 0xFF) as f32 / 255.0;
            let g = ((v >> 16) & 0xFF) as f32 / 255.0;
            let b = ((v >> 8) & 0xFF) as f32 / 255.0;
            let a = (v & 0xFF) as f32 / 255.0;
            return Color::new(r, g, b, a);
        }
    }
    Palette::WHITE
}

/// Swappable theme configuration defining colors across UI components.
#[derive(Debug, Clone, PartialEq)]
pub struct CabinetTheme {
    pub name: String,
    pub card_bg: Color,
    pub card_bg_hover: Color,
    pub card_border: Color,
    pub card_border_glow: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub accent_primary: Color,
    pub accent_secondary: Color,
    pub accent_success: Color,
    pub accent_danger: Color,
}

impl Default for CabinetTheme {
    fn default() -> Self {
        Self::cyberpunk_neon()
    }
}

impl CabinetTheme {
    /// Default high-contrast Cyberpunk Neon theme (Cyan, Gold, Magenta).
    pub fn cyberpunk_neon() -> Self {
        Self {
            name: "Cyberpunk Neon".to_string(),
            card_bg: Palette::UI_CARD_BG,
            card_bg_hover: Palette::UI_CARD_BG_HOVER,
            card_border: Palette::UI_CARD_BORDER,
            card_border_glow: Palette::NEON_CYAN,
            text_primary: Palette::WHITE,
            text_muted: Palette::UI_TEXT_MUTED,
            accent_primary: Palette::NEON_CYAN,
            accent_secondary: Palette::NEON_GOLD,
            accent_success: Palette::NEON_GREEN,
            accent_danger: Palette::RED,
        }
    }

    /// Retro Arcade theme (Classic coin-op Amber, Magenta & Indigo).
    pub fn arcade_retro() -> Self {
        Self {
            name: "Arcade Retro".to_string(),
            card_bg: Color::new(0.10, 0.05, 0.15, 0.94),
            card_bg_hover: Color::new(0.18, 0.08, 0.26, 0.96),
            card_border: Color::new(0.60, 0.20, 0.70, 0.85),
            card_border_glow: Color::new(1.0, 0.40, 0.85, 1.0),
            text_primary: Color::new(1.0, 0.95, 0.85, 1.0),
            text_muted: Color::new(0.75, 0.65, 0.78, 1.0),
            accent_primary: Palette::NEON_GOLD,
            accent_secondary: Palette::NEON_MAGENTA,
            accent_success: Palette::NEON_GREEN,
            accent_danger: Color::new(1.0, 0.15, 0.30, 1.0),
        }
    }

    /// Dark Glass theme (Clean, modern minimalist dark UI).
    pub fn dark_glass() -> Self {
        Self {
            name: "Dark Glass".to_string(),
            card_bg: Color::new(0.04, 0.05, 0.07, 0.90),
            card_bg_hover: Color::new(0.08, 0.10, 0.14, 0.94),
            card_border: Color::new(0.20, 0.24, 0.30, 0.80),
            card_border_glow: Color::new(0.40, 0.60, 0.85, 1.0),
            text_primary: Palette::WHITE,
            text_muted: Color::new(0.60, 0.65, 0.72, 1.0),
            accent_primary: Color::new(0.35, 0.65, 1.0, 1.0),
            accent_secondary: Palette::NEON_GOLD,
            accent_success: Palette::GREEN,
            accent_danger: Palette::RED,
        }
    }
}
