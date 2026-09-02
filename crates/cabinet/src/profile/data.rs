use macroquad::color::Color;
use serde::{Deserialize, Serialize};
use crate::ui::theme::{color_to_hex, hex_to_color};

/// 3-tone color scheme for player customization, avatars, vehicles, or sprites.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorScheme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::from_index(0)
    }
}

impl ColorScheme {
    pub const PRESETS: [(Color, Color, Color); 9] = [
        // 0: Racing Red & Gloss White
        (Color::new(0.95, 0.12, 0.15, 1.0), Color::new(0.98, 0.98, 1.0, 1.0), Color::new(1.0, 0.82, 0.15, 1.0)),
        // 1: Electric Blue & Cyber Cyan
        (Color::new(0.12, 0.48, 0.98, 1.0), Color::new(0.25, 0.90, 1.0, 1.0), Color::new(1.0, 0.48, 0.10, 1.0)),
        // 2: Toxic Viper Green & Emerald
        (Color::new(0.15, 0.82, 0.30, 1.0), Color::new(0.08, 0.45, 0.16, 1.0), Color::new(0.98, 0.98, 0.98, 1.0)),
        // 3: Sunburst Yellow & Midnight Black
        (Color::new(1.0, 0.82, 0.08, 1.0), Color::new(0.12, 0.12, 0.15, 1.0), Color::new(0.95, 0.18, 0.18, 1.0)),
        // 4: Hyper Violet & Hot Magenta
        (Color::new(0.65, 0.15, 0.95, 1.0), Color::new(1.0, 0.25, 0.80, 1.0), Color::new(0.20, 0.95, 0.85, 1.0)),
        // 5: Stealth Carbon & Neon Gold
        (Color::new(0.18, 0.20, 0.24, 1.0), Color::new(1.0, 0.82, 0.15, 1.0), Color::new(0.20, 0.90, 1.0, 1.0)),
        // 6: Pure Ghost White & Azure Blue
        (Color::new(0.96, 0.96, 0.98, 1.0), Color::new(0.18, 0.55, 0.95, 1.0), Color::new(0.95, 0.18, 0.22, 1.0)),
        // 7: Neon Sunset Orange & Royal Navy
        (Color::new(1.0, 0.42, 0.08, 1.0), Color::new(0.10, 0.15, 0.35, 1.0), Color::new(0.98, 0.90, 0.15, 1.0)),
        // 8: Cyber Cyan & Hot Pink
        (Color::new(0.10, 0.85, 0.95, 1.0), Color::new(0.95, 0.15, 0.75, 1.0), Color::new(0.98, 0.98, 0.98, 1.0)),
    ];

    pub fn from_index(idx: usize) -> Self {
        let (p, s, a) = Self::PRESETS[idx % Self::PRESETS.len()];
        Self { primary: p, secondary: s, accent: a }
    }

    pub fn to_hex_strings(&self) -> (String, String, String) {
        (
            color_to_hex(self.primary),
            color_to_hex(self.secondary),
            color_to_hex(self.accent),
        )
    }

    pub fn from_hex_strings(primary: &str, secondary: &str, accent: &str) -> Self {
        Self {
            primary: hex_to_color(primary),
            secondary: hex_to_color(secondary),
            accent: hex_to_color(accent),
        }
    }
}

impl Serialize for ColorScheme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex = self.to_hex_strings();
        hex.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ColorScheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (p, s, a) = <(String, String, String)>::deserialize(deserializer)?;
        Ok(Self::from_hex_strings(&p, &s, &a))
    }
}

/// Player Profile representing user identity, nickname, nationality, and color palette.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub id: Option<i64>,
    pub name: String,
    pub alias: String,
    pub country: Option<String>,
    pub color_scheme: ColorScheme,
    pub is_active: bool,
    pub created_at: String,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            id: None,
            name: "Player One".to_string(),
            alias: "Ace".to_string(),
            country: Some("ESP".to_string()),
            color_scheme: ColorScheme::from_index(0),
            is_active: true,
            created_at: String::new(),
        }
    }
}

impl PlayerProfile {
    pub fn new(name: &str, alias: &str, country: Option<&str>, color_scheme: ColorScheme) -> Self {
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
}
