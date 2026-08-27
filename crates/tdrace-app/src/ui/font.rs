use macroquad::color::Color;
use macroquad::text::{draw_text_ex, load_ttf_font_from_bytes, measure_text, Font, TextDimensions, TextParams};

/// Embedded vector TrueType font byte slices (OFL licensed).
pub const FONT_DISPLAY_BYTES: &[u8] = include_bytes!("../../assets/fonts/Rajdhani-Bold.ttf");
pub const FONT_UI_BOLD_BYTES: &[u8] = include_bytes!("../../assets/fonts/Barlow-SemiBold.ttf");
pub const FONT_UI_MEDIUM_BYTES: &[u8] = include_bytes!("../../assets/fonts/Barlow-Medium.ttf");

/// Global typography and font asset manager.
#[derive(Debug, Clone)]
pub struct Fonts {
    /// High-impact techno-sport font for HUD gauges, lap timers, position badges, and headers.
    pub display: Option<Font>,
    /// Crisp, modern semi-bold sans-serif for UI card headers, buttons, and high-legibility badges.
    pub ui_bold: Option<Font>,
    /// Clean modern sans-serif for descriptions, telemetry, and subtitles.
    pub ui_regular: Option<Font>,
}

impl Default for Fonts {
    fn default() -> Self {
        Self::load_embedded()
    }
}

impl Fonts {
    /// Loads all embedded vector fonts directly from compiled binary bytes.
    pub fn load_embedded() -> Self {
        let display = std::panic::catch_unwind(|| load_ttf_font_from_bytes(FONT_DISPLAY_BYTES).ok())
            .ok()
            .flatten();
        let ui_bold = std::panic::catch_unwind(|| load_ttf_font_from_bytes(FONT_UI_BOLD_BYTES).ok())
            .ok()
            .flatten();
        let ui_regular = std::panic::catch_unwind(|| load_ttf_font_from_bytes(FONT_UI_MEDIUM_BYTES).ok())
            .ok()
            .flatten();

        Self {
            display,
            ui_bold,
            ui_regular,
        }
    }

    /// Renders text with display racing font.
    pub fn draw_display(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let _ = std::panic::catch_unwind(|| {
            draw_text_ex(
                text,
                x,
                y,
                TextParams {
                    font: self.display.as_ref(),
                    font_size: size.round() as u16,
                    font_scale: 1.0,
                    color,
                    ..Default::default()
                },
            );
        });
    }

    /// Renders centered text with display racing font.
    pub fn draw_display_centered(&self, text: &str, center_x: f32, y: f32, size: f32, color: Color) {
        let dim = self.measure_display(text, size);
        self.draw_display(text, center_x - dim.width * 0.5, y, size, color);
    }

    /// Renders text with display racing font and high-contrast drop shadow.
    pub fn draw_display_with_shadow(
        &self,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        shadow_color: Color,
        shadow_offset: f32,
    ) {
        self.draw_display(text, x + shadow_offset, y + shadow_offset, size, shadow_color);
        self.draw_display(text, x, y, size, color);
    }

    /// Renders centered text with display racing font and drop shadow.
    pub fn draw_display_centered_with_shadow(
        &self,
        text: &str,
        center_x: f32,
        y: f32,
        size: f32,
        color: Color,
        shadow_color: Color,
        shadow_offset: f32,
    ) {
        let dim = self.measure_display(text, size);
        let x = center_x - dim.width * 0.5;
        self.draw_display_with_shadow(text, x, y, size, color, shadow_color, shadow_offset);
    }

    /// Measures text bounding box with display racing font.
    pub fn measure_display(&self, text: &str, size: f32) -> TextDimensions {
        std::panic::catch_unwind(|| {
            measure_text(text, self.display.as_ref(), size.round() as u16, 1.0)
        })
        .unwrap_or(TextDimensions {
            width: text.len() as f32 * size * 0.58,
            height: size,
            offset_y: size * 0.8,
        })
    }

    /// Renders text with bold UI font.
    pub fn draw_ui_bold(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let _ = std::panic::catch_unwind(|| {
            draw_text_ex(
                text,
                x,
                y,
                TextParams {
                    font: self.ui_bold.as_ref(),
                    font_size: size.round() as u16,
                    font_scale: 1.0,
                    color,
                    ..Default::default()
                },
            );
        });
    }

    /// Renders centered text with bold UI font.
    pub fn draw_ui_bold_centered(&self, text: &str, center_x: f32, y: f32, size: f32, color: Color) {
        let dim = self.measure_ui_bold(text, size);
        self.draw_ui_bold(text, center_x - dim.width * 0.5, y, size, color);
    }

    /// Measures text bounding box with bold UI font.
    pub fn measure_ui_bold(&self, text: &str, size: f32) -> TextDimensions {
        std::panic::catch_unwind(|| {
            measure_text(text, self.ui_bold.as_ref(), size.round() as u16, 1.0)
        })
        .unwrap_or(TextDimensions {
            width: text.len() as f32 * size * 0.54,
            height: size,
            offset_y: size * 0.8,
        })
    }

    /// Renders text with regular UI font.
    pub fn draw_ui_regular(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let _ = std::panic::catch_unwind(|| {
            draw_text_ex(
                text,
                x,
                y,
                TextParams {
                    font: self.ui_regular.as_ref(),
                    font_size: size.round() as u16,
                    font_scale: 1.0,
                    color,
                    ..Default::default()
                },
            );
        });
    }

    /// Renders centered text with regular UI font.
    pub fn draw_ui_regular_centered(&self, text: &str, center_x: f32, y: f32, size: f32, color: Color) {
        let dim = self.measure_ui_regular(text, size);
        self.draw_ui_regular(text, center_x - dim.width * 0.5, y, size, color);
    }

    /// Measures text bounding box with regular UI font.
    pub fn measure_ui_regular(&self, text: &str, size: f32) -> TextDimensions {
        std::panic::catch_unwind(|| {
            measure_text(text, self.ui_regular.as_ref(), size.round() as u16, 1.0)
        })
        .unwrap_or(TextDimensions {
            width: text.len() as f32 * size * 0.52,
            height: size,
            offset_y: size * 0.8,
        })
    }

    /// Wraps text into multiple lines so that each line's rendered width does not exceed `max_width`.
    pub fn wrap_text(&self, text: &str, font_size: f32, max_width: f32) -> Vec<String> {
        let words = text.split_whitespace().collect::<Vec<_>>();
        if words.is_empty() {
            return Vec::new();
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in words {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else {
                let candidate = format!("{} {}", current_line, word);
                if self.measure_ui_regular(&candidate, font_size).width <= max_width {
                    current_line = candidate;
                } else {
                    lines.push(current_line);
                    current_line = word.to_string();
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Renders multi-line text with regular UI font, wrapping automatically to `max_width`.
    pub fn draw_ui_regular_multiline(
        &self,
        text: &str,
        x: f32,
        start_y: f32,
        font_size: f32,
        line_height: f32,
        max_width: f32,
        color: Color,
    ) -> usize {
        let lines = self.wrap_text(text, font_size, max_width);
        for (i, line) in lines.iter().enumerate() {
            self.draw_ui_regular(line, x, start_y + (i as f32 * line_height), font_size, color);
        }
        lines.len()
    }
}
