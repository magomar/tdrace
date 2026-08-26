use macroquad::color::Color;

/// Modern arcade motorsport palette with high-visibility accents and glassmorphism styling.
pub struct Palette;

impl Palette {
    // Basic constants
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Color = Color::new(0.95, 0.20, 0.20, 1.0);
    pub const GREEN: Color = Color::new(0.18, 0.90, 0.35, 1.0);
    pub const BLUE: Color = Color::new(0.20, 0.55, 1.0, 1.0);
    pub const YELLOW: Color = Color::new(1.0, 0.85, 0.15, 1.0);

    // Modern Neon & UI Accents
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

    // Modern Terrain & Track Surfaces
    pub const GRASS: Color = Color::new(0.18, 0.48, 0.24, 1.0);
    pub const GRASS_DARK: Color = Color::new(0.14, 0.40, 0.19, 1.0);
    pub const ASPHALT: Color = Color::new(0.16, 0.17, 0.20, 1.0);
    pub const ASPHALT_LIGHT: Color = Color::new(0.22, 0.23, 0.27, 1.0);
    pub const RUNOFF_ASPHALT: Color = Color::new(0.20, 0.22, 0.25, 1.0);
    pub const SAND: Color = Color::new(0.85, 0.74, 0.48, 1.0);
    pub const SAND_DARK: Color = Color::new(0.75, 0.64, 0.38, 1.0);
    pub const PIT_LANE: Color = Color::new(0.15, 0.16, 0.19, 1.0);

    // Modern Track Markings & Curbs
    pub const CURB_RED: Color = Color::new(0.92, 0.15, 0.18, 1.0);
    pub const CURB_WHITE: Color = Color::new(0.98, 0.98, 1.0, 1.0);
    pub const WHITE_LINE: Color = Color::new(0.96, 0.96, 0.98, 0.95);
    pub const YELLOW_LINE: Color = Color::new(1.0, 0.82, 0.10, 0.95);
    pub const GRID_LINE: Color = Color::new(0.98, 0.98, 1.0, 0.90);

    // Barriers & Shadows
    pub const ARMCO_POST: Color = Color::new(0.40, 0.44, 0.48, 1.0);
    pub const ARMCO_RAIL: Color = Color::new(0.75, 0.78, 0.85, 1.0);
    pub const CONCRETE_WALL: Color = Color::new(0.80, 0.82, 0.82, 1.0);
    pub const CONCRETE_TOP: Color = Color::new(0.94, 0.94, 0.94, 1.0);
    pub const TIRE_WALL: Color = Color::new(0.14, 0.15, 0.17, 1.0);
    pub const TIRE_RIM: Color = Color::new(0.38, 0.40, 0.45, 1.0);
    pub const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.42);
    pub const SOFT_SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.22);

    // FX Colors
    pub const SKIDMARK: Color = Color::new(0.06, 0.06, 0.08, 0.65);
    pub const TIRE_SMOKE: Color = Color::new(0.92, 0.93, 0.96, 0.32);
    pub const DIRT_PARTICLE: Color = Color::new(0.58, 0.48, 0.28, 0.85);
    pub const SPARK: Color = Color::new(1.0, 0.82, 0.20, 1.0);
    pub const SPARK_WHITE: Color = Color::new(1.0, 0.98, 0.75, 1.0);

    // Modern Car Presets (Primary, Secondary, Helmet)
    pub const CAR_COLORS: [(Color, Color, Color); 8] = [
        // 0: Player - Hyper Racing Red with Gloss White Stripe & Neon Gold Visor
        (
            Color::new(0.95, 0.12, 0.15, 1.0),
            Color::new(0.98, 0.98, 1.0, 1.0),
            Color::new(1.0, 0.82, 0.15, 1.0),
        ),
        // 1: AI 1 - Electric Blue with Cyber Cyan Stripe & Neon Orange Helmet
        (
            Color::new(0.12, 0.48, 0.98, 1.0),
            Color::new(0.25, 0.90, 1.0, 1.0),
            Color::new(1.0, 0.48, 0.10, 1.0),
        ),
        // 2: AI 2 - Toxic Viper Green with Emerald Stripe & Alpine White Helmet
        (
            Color::new(0.15, 0.82, 0.30, 1.0),
            Color::new(0.08, 0.45, 0.16, 1.0),
            Color::new(0.98, 0.98, 0.98, 1.0),
        ),
        // 3: AI 3 - Sunburst Yellow with Midnight Black Stripe & Crimson Helmet
        (
            Color::new(1.0, 0.82, 0.08, 1.0),
            Color::new(0.12, 0.12, 0.15, 1.0),
            Color::new(0.95, 0.18, 0.18, 1.0),
        ),
        // 4: AI 4 - Sunset Orange with Carbon Stripe & Sky Blue Helmet
        (
            Color::new(1.0, 0.42, 0.08, 1.0),
            Color::new(0.95, 0.95, 0.98, 1.0),
            Color::new(0.20, 0.55, 1.0, 1.0),
        ),
        // 5: AI 5 - Synthwave Purple with Gold Stripe & Silver Helmet
        (
            Color::new(0.65, 0.18, 0.85, 1.0),
            Color::new(1.0, 0.80, 0.18, 1.0),
            Color::new(0.85, 0.88, 0.92, 1.0),
        ),
        // 6: AI 6 - Stealth Carbon Black with Neon Red Stripe & White Helmet
        (
            Color::new(0.12, 0.13, 0.16, 1.0),
            Color::new(0.98, 0.18, 0.18, 1.0),
            Color::new(0.98, 0.98, 0.98, 1.0),
        ),
        // 7: AI 7 - Glacier White with Cyan Stripe & Electric Blue Helmet
        (
            Color::new(0.96, 0.97, 1.0, 1.0),
            Color::new(0.15, 0.50, 0.95, 1.0),
            Color::new(0.25, 0.90, 0.95, 1.0),
        ),
    ];
}

/// Car visual color scheme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarColorScheme {
    pub primary: Color,
    pub secondary: Color,
    pub helmet: Color,
}

impl Default for CarColorScheme {
    fn default() -> Self {
        Self::from_index(0)
    }
}

impl CarColorScheme {
    pub const fn new(primary: Color, secondary: Color, helmet: Color) -> Self {
        Self {
            primary,
            secondary,
            helmet,
        }
    }

    pub const fn from_index(index: usize) -> Self {
        let n = Palette::CAR_COLORS.len();
        let (p, s, h) = Palette::CAR_COLORS[index % n];
        Self {
            primary: p,
            secondary: s,
            helmet: h,
        }
    }
}

