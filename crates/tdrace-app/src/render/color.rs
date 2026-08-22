use macroquad::color::Color;

/// Curated retro arcade color palette inspired by GeneRally.
pub struct Palette;

impl Palette {
    // Basic constants
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Color = Color::new(0.90, 0.15, 0.15, 1.0);
    pub const GREEN: Color = Color::new(0.15, 0.85, 0.20, 1.0);
    pub const BLUE: Color = Color::new(0.15, 0.45, 0.90, 1.0);
    pub const YELLOW: Color = Color::new(0.95, 0.85, 0.10, 1.0);

    // Terrain
    pub const GRASS: Color = Color::new(0.24, 0.58, 0.26, 1.0);
    pub const GRASS_DARK: Color = Color::new(0.20, 0.50, 0.22, 1.0);
    pub const ASPHALT: Color = Color::new(0.22, 0.23, 0.26, 1.0);
    pub const ASPHALT_LIGHT: Color = Color::new(0.28, 0.29, 0.32, 1.0);
    pub const RUNOFF_ASPHALT: Color = Color::new(0.26, 0.27, 0.30, 1.0);
    pub const SAND: Color = Color::new(0.88, 0.78, 0.52, 1.0);
    pub const SAND_DARK: Color = Color::new(0.80, 0.70, 0.44, 1.0);
    pub const PIT_LANE: Color = Color::new(0.20, 0.21, 0.24, 1.0);

    // Track Markings & Curbs
    pub const CURB_RED: Color = Color::new(0.88, 0.16, 0.16, 1.0);
    pub const CURB_WHITE: Color = Color::new(0.95, 0.95, 0.96, 1.0);
    pub const WHITE_LINE: Color = Color::new(0.92, 0.92, 0.94, 0.9);
    pub const YELLOW_LINE: Color = Color::new(0.96, 0.80, 0.15, 0.9);
    pub const GRID_LINE: Color = Color::new(0.95, 0.95, 0.95, 0.85);

    // Barriers & Shadows
    pub const ARMCO_POST: Color = Color::new(0.45, 0.47, 0.50, 1.0);
    pub const ARMCO_RAIL: Color = Color::new(0.72, 0.75, 0.80, 1.0);
    pub const CONCRETE_WALL: Color = Color::new(0.82, 0.82, 0.80, 1.0);
    pub const CONCRETE_TOP: Color = Color::new(0.92, 0.92, 0.90, 1.0);
    pub const TIRE_WALL: Color = Color::new(0.18, 0.18, 0.20, 1.0);
    pub const TIRE_RIM: Color = Color::new(0.35, 0.35, 0.38, 1.0);
    pub const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.38);
    pub const SOFT_SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.20);

    // FX Colors
    pub const SKIDMARK: Color = Color::new(0.08, 0.08, 0.10, 0.60);
    pub const TIRE_SMOKE: Color = Color::new(0.90, 0.90, 0.92, 0.55);
    pub const DIRT_PARTICLE: Color = Color::new(0.55, 0.45, 0.25, 0.80);
    pub const SPARK: Color = Color::new(1.0, 0.78, 0.15, 1.0);
    pub const SPARK_WHITE: Color = Color::new(1.0, 0.95, 0.70, 1.0);

    // Car Presets (Primary, Secondary, Helmet)
    pub const CAR_COLORS: &[(Color, Color, Color)] = &[
        // 0: Player - Classic Rosso Corsa Red with White Racing Stripes & Yellow Helmet
        (
            Color::new(0.90, 0.12, 0.12, 1.0),
            Color::new(0.95, 0.95, 0.95, 1.0),
            Color::new(0.98, 0.85, 0.10, 1.0),
        ),
        // 1: AI 1 - Electric Blue with Cyan Stripe & Orange Helmet
        (
            Color::new(0.10, 0.45, 0.92, 1.0),
            Color::new(0.30, 0.85, 0.95, 1.0),
            Color::new(0.95, 0.50, 0.10, 1.0),
        ),
        // 2: AI 2 - Viper Green with Dark Green Stripe & White Helmet
        (
            Color::new(0.12, 0.75, 0.25, 1.0),
            Color::new(0.06, 0.40, 0.12, 1.0),
            Color::new(0.95, 0.95, 0.95, 1.0),
        ),
        // 3: AI 3 - Sunburst Yellow with Black Stripe & Red Helmet
        (
            Color::new(0.95, 0.80, 0.05, 1.0),
            Color::new(0.15, 0.15, 0.18, 1.0),
            Color::new(0.90, 0.15, 0.15, 1.0),
        ),
        // 4: AI 4 - Pure Orange with White Stripe & Blue Helmet
        (
            Color::new(0.95, 0.42, 0.08, 1.0),
            Color::new(0.95, 0.95, 0.95, 1.0),
            Color::new(0.15, 0.45, 0.90, 1.0),
        ),
        // 5: AI 5 - Deep Purple with Gold Stripe & Silver Helmet
        (
            Color::new(0.60, 0.15, 0.75, 1.0),
            Color::new(0.92, 0.75, 0.15, 1.0),
            Color::new(0.80, 0.82, 0.85, 1.0),
        ),
        // 6: AI 6 - Stealth Matte Black with Red Stripe & White Helmet
        (
            Color::new(0.14, 0.14, 0.16, 1.0),
            Color::new(0.90, 0.15, 0.15, 1.0),
            Color::new(0.95, 0.95, 0.95, 1.0),
        ),
        // 7: AI 7 - Alpine White with Blue Stripe & Cyan Helmet
        (
            Color::new(0.94, 0.94, 0.96, 1.0),
            Color::new(0.10, 0.45, 0.90, 1.0),
            Color::new(0.20, 0.85, 0.90, 1.0),
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

    pub fn from_index(index: usize) -> Self {
        let n = Palette::CAR_COLORS.len();
        let (p, s, h) = Palette::CAR_COLORS[index % n];
        Self {
            primary: p,
            secondary: s,
            helmet: h,
        }
    }
}
