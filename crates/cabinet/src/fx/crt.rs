use macroquad::color::Color;
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};

/// Visual intensity presets for CRT scanlines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanlineMode {
    #[default]
    Disabled,
    /// Subtle 25% scanline density for modern displays.
    Subtle,
    /// Classic 50% arcade cathode ray tube scanlines.
    ArcadeCrt,
    /// High-contrast 75% retro glow scanlines.
    RetroGlow,
}

impl ScanlineMode {
    /// Maps a dropdown selection index (0..4) to a `ScanlineMode`.
    pub fn from_index(index: usize) -> Self {
        match index {
            1 => ScanlineMode::Subtle,
            2 => ScanlineMode::ArcadeCrt,
            3 => ScanlineMode::RetroGlow,
            _ => ScanlineMode::Disabled,
        }
    }

    /// Maps this `ScanlineMode` to its settings dropdown index.
    pub fn to_index(&self) -> usize {
        match self {
            ScanlineMode::Disabled => 0,
            ScanlineMode::Subtle => 1,
            ScanlineMode::ArcadeCrt => 2,
            ScanlineMode::RetroGlow => 3,
        }
    }

    /// Default baseline opacity for dark scanline bands.
    pub fn opacity(&self) -> f32 {
        match self {
            ScanlineMode::Disabled => 0.0,
            ScanlineMode::Subtle => 0.18,
            ScanlineMode::ArcadeCrt => 0.35,
            ScanlineMode::RetroGlow => 0.55,
        }
    }

    /// Human-readable label for menus.
    pub fn label(&self) -> &'static str {
        match self {
            ScanlineMode::Disabled => "Disabled",
            ScanlineMode::Subtle => "Subtle (25%)",
            ScanlineMode::ArcadeCrt => "Arcade CRT (50%)",
            ScanlineMode::RetroGlow => "Retro Glow (75%)",
        }
    }
}

/// Configuration for CRT scanlines and post-processing overlays.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrtConfig {
    /// Scanline intensity mode.
    pub mode: ScanlineMode,
    /// Optional manual opacity override (0.0 to 1.0).
    pub custom_opacity: Option<f32>,
    /// Distance in pixels between scanlines (default: 3.0).
    pub line_spacing: f32,
    /// Thickness of each dark scanline (default: 1.0).
    pub line_thickness: f32,
    /// Vignette edge-darkening intensity (0.0 to 1.0, default: 0.25).
    pub vignette_intensity: f32,
    /// Number of concentric vignette border rings (default: 5).
    pub vignette_steps: usize,
    /// Optional phosphor color cast (e.g. subtle green, amber, or cyan glow).
    pub phosphor_tint: Option<Color>,
    /// Rolling interference bar speed in pixels per second (0.0 = static/disabled).
    pub roll_speed: f32,
    /// Height of the rolling interference bar (default: 60.0).
    pub roll_bar_height: f32,
    /// Opacity of the rolling interference bar (default: 0.08).
    pub roll_bar_opacity: f32,
}

impl Default for CrtConfig {
    fn default() -> Self {
        Self {
            mode: ScanlineMode::Disabled,
            custom_opacity: None,
            line_spacing: 3.0,
            line_thickness: 1.0,
            vignette_intensity: 0.25,
            vignette_steps: 5,
            phosphor_tint: None,
            roll_speed: 0.0,
            roll_bar_height: 60.0,
            roll_bar_opacity: 0.08,
        }
    }
}

/// Standalone post-processing overlay rendering CRT scanlines, vignette curvature,
/// and phosphor hum bars.
#[derive(Debug, Clone)]
pub struct CrtOverlay {
    pub config: CrtConfig,
    pub roll_offset: f32,
}

impl Default for CrtOverlay {
    fn default() -> Self {
        Self::new(CrtConfig::default())
    }
}

impl CrtOverlay {
    /// Creates a new CRT overlay with the given configuration.
    pub fn new(config: CrtConfig) -> Self {
        Self {
            config,
            roll_offset: 0.0,
        }
    }

    /// Creates an overlay initialized with a preset scanline mode.
    pub fn with_mode(mode: ScanlineMode) -> Self {
        let mut config = CrtConfig::default();
        config.mode = mode;
        Self::new(config)
    }

    /// Advances the rolling scanline bar animation.
    pub fn update(&mut self, dt: f32) {
        if self.config.roll_speed.abs() > 0.001 {
            self.roll_offset += self.config.roll_speed * dt;
        }
    }

    /// Resolves the effective scanline opacity.
    pub fn effective_opacity(&self) -> f32 {
        self.config
            .custom_opacity
            .unwrap_or_else(|| self.config.mode.opacity())
    }

    /// Returns true if any visual effect is active and should be drawn.
    pub fn is_active(&self) -> bool {
        self.effective_opacity() > 0.001
            || self.config.vignette_intensity > 0.001
            || self.config.phosphor_tint.is_some()
            || (self.config.roll_speed.abs() > 0.001 && self.config.roll_bar_opacity > 0.001)
    }

    /// Renders the CRT overlay over the specified screen rectangle.
    pub fn render(&self, x: f32, y: f32, width: f32, height: f32) {
        if width <= 0.0 || height <= 0.0 || !self.is_active() {
            return;
        }

        // 1. Phosphor tint overlay
        if let Some(tint) = self.config.phosphor_tint {
            draw_rectangle(x, y, width, height, tint);
        }

        // 2. Horizontal scanlines
        let opacity = self.effective_opacity();
        if opacity > 0.001 {
            let spacing = self.config.line_spacing.max(1.0);
            let thickness = self.config.line_thickness.max(0.5);
            let scanline_color = Color::new(0.0, 0.0, 0.0, opacity);

            let mut curr_y = y;
            let end_y = y + height;
            while curr_y < end_y {
                draw_rectangle(x, curr_y, width, thickness, scanline_color);
                curr_y += spacing;
            }
        }

        // 3. Rolling cathode hum bar
        if self.config.roll_speed.abs() > 0.001 && self.config.roll_bar_opacity > 0.001 {
            let bar_h = self.config.roll_bar_height.clamp(10.0, height);
            let total_range = height + bar_h;
            let current_pos = y - bar_h + (self.roll_offset.rem_euclid(total_range));
            let bar_y = current_pos.clamp(y, y + height);
            let visible_h = (current_pos + bar_h).min(y + height) - bar_y;

            if visible_h > 0.0 {
                let hum_color = Color::new(0.0, 0.0, 0.0, self.config.roll_bar_opacity);
                draw_rectangle(x, bar_y, width, visible_h, hum_color);
            }
        }

        // 4. Vignette / CRT curvature border rings
        let vig = self.config.vignette_intensity;
        if vig > 0.001 && self.config.vignette_steps > 0 {
            let steps = self.config.vignette_steps as f32;
            for i in 0..self.config.vignette_steps {
                let factor = (i + 1) as f32 / steps;
                let ring_alpha = vig * 0.15 * factor;
                let inset = i as f32 * 2.5;
                let ring_w = width - inset * 2.0;
                let ring_h = height - inset * 2.0;

                if ring_w > 0.0 && ring_h > 0.0 {
                    draw_rectangle_lines(
                        x + inset,
                        y + inset,
                        ring_w,
                        ring_h,
                        3.0,
                        Color::new(0.0, 0.0, 0.0, ring_alpha),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanline_mode_indices_and_opacities() {
        assert_eq!(ScanlineMode::from_index(0), ScanlineMode::Disabled);
        assert_eq!(ScanlineMode::from_index(1), ScanlineMode::Subtle);
        assert_eq!(ScanlineMode::from_index(2), ScanlineMode::ArcadeCrt);
        assert_eq!(ScanlineMode::from_index(3), ScanlineMode::RetroGlow);
        assert_eq!(ScanlineMode::from_index(99), ScanlineMode::Disabled);

        assert_eq!(ScanlineMode::Disabled.to_index(), 0);
        assert_eq!(ScanlineMode::Subtle.to_index(), 1);
        assert_eq!(ScanlineMode::ArcadeCrt.to_index(), 2);
        assert_eq!(ScanlineMode::RetroGlow.to_index(), 3);

        assert_eq!(ScanlineMode::Disabled.opacity(), 0.0);
        assert!(ScanlineMode::Subtle.opacity() > 0.1);
        assert!(ScanlineMode::ArcadeCrt.opacity() > ScanlineMode::Subtle.opacity());
        assert!(ScanlineMode::RetroGlow.opacity() > ScanlineMode::ArcadeCrt.opacity());
    }

    #[test]
    fn test_crt_overlay_defaults_and_activation() {
        let mut overlay = CrtOverlay::default();
        // Default has vignette 0.25 so it is active
        assert!(overlay.is_active());

        overlay.config.vignette_intensity = 0.0;
        assert!(!overlay.is_active());

        overlay.config.mode = ScanlineMode::ArcadeCrt;
        assert!(overlay.is_active());
        assert_eq!(overlay.effective_opacity(), ScanlineMode::ArcadeCrt.opacity());

        // Custom opacity override
        overlay.config.custom_opacity = Some(0.9);
        assert_eq!(overlay.effective_opacity(), 0.9);
    }

    #[test]
    fn test_crt_overlay_roll_animation() {
        let mut overlay = CrtOverlay::new(CrtConfig {
            roll_speed: 100.0,
            ..Default::default()
        });

        assert_eq!(overlay.roll_offset, 0.0);
        overlay.update(0.5);
        assert_eq!(overlay.roll_offset, 50.0);
        overlay.update(0.5);
        assert_eq!(overlay.roll_offset, 100.0);
    }
}
