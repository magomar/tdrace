use macroquad::color::Color;
use macroquad::text::{draw_text_ex, load_ttf_font_from_bytes, measure_text, Font, TextDimensions, TextParams};

use crate::ui::symbols::{has_symbols, split_pieces, symbol_icon, TextPiece};

/// Embedded vector TrueType font byte slices (OFL licensed).
pub const FONT_DISPLAY_BYTES: &[u8] = include_bytes!("../../assets/fonts/Rajdhani-Bold.ttf");
pub const FONT_UI_BOLD_BYTES: &[u8] = include_bytes!("../../assets/fonts/Barlow-SemiBold.ttf");
pub const FONT_UI_MEDIUM_BYTES: &[u8] = include_bytes!("../../assets/fonts/Barlow-Medium.ttf");

/// Global typography and font asset manager.
#[derive(Debug, Clone)]
pub struct Fonts {
    /// High-impact techno-arcade font for gauges, timers, position badges, and headers.
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

    /// Renders text with display font.
    pub fn draw_display(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let _ = std::panic::catch_unwind(|| draw_pieces(self.display.as_ref(), text, x, y, size, color));
    }

    /// Renders centered text with display font.
    pub fn draw_display_centered(&self, text: &str, center_x: f32, y: f32, size: f32, color: Color) {
        let dim = self.measure_display(text, size);
        self.draw_display(text, center_x - dim.width * 0.5, y, size, color);
    }

    /// Renders text with display font and high-contrast drop shadow.
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

    /// Renders centered text with display font and drop shadow.
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

    /// Measures text bounding box with display font.
    pub fn measure_display(&self, text: &str, size: f32) -> TextDimensions {
        std::panic::catch_unwind(|| measure_pieces(self.display.as_ref(), text, size))
            .unwrap_or_else(|_| fallback_dimensions(text, size, 0.58))
    }

    /// Renders text with bold UI font.
    pub fn draw_ui_bold(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let _ = std::panic::catch_unwind(|| draw_pieces(self.ui_bold.as_ref(), text, x, y, size, color));
    }

    /// Renders centered text with bold UI font.
    pub fn draw_ui_bold_centered(&self, text: &str, center_x: f32, y: f32, size: f32, color: Color) {
        let dim = self.measure_ui_bold(text, size);
        self.draw_ui_bold(text, center_x - dim.width * 0.5, y, size, color);
    }

    /// Measures text bounding box with bold UI font.
    pub fn measure_ui_bold(&self, text: &str, size: f32) -> TextDimensions {
        std::panic::catch_unwind(|| measure_pieces(self.ui_bold.as_ref(), text, size))
            .unwrap_or_else(|_| fallback_dimensions(text, size, 0.54))
    }

    /// Renders text with regular UI font.
    pub fn draw_ui_regular(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let _ = std::panic::catch_unwind(|| draw_pieces(self.ui_regular.as_ref(), text, x, y, size, color));
    }

    /// Renders centered text with regular UI font.
    pub fn draw_ui_regular_centered(&self, text: &str, center_x: f32, y: f32, size: f32, color: Color) {
        let dim = self.measure_ui_regular(text, size);
        self.draw_ui_regular(text, center_x - dim.width * 0.5, y, size, color);
    }

    /// Measures text bounding box with regular UI font.
    pub fn measure_ui_regular(&self, text: &str, size: f32) -> TextDimensions {
        std::panic::catch_unwind(|| measure_pieces(self.ui_regular.as_ref(), text, size))
            .unwrap_or_else(|_| fallback_dimensions(text, size, 0.52))
    }

    /// Shortens text with a trailing "…" so its bold UI rendering fits within `max_width`.
    pub fn fit_ui_bold(&self, text: &str, size: f32, max_width: f32) -> String {
        fit_to_width(text, max_width, |s| self.measure_ui_bold(s, size).width)
    }

    /// Shortens text with a trailing "…" so its regular UI rendering fits within `max_width`.
    pub fn fit_ui_regular(&self, text: &str, size: f32, max_width: f32) -> String {
        fit_to_width(text, max_width, |s| self.measure_ui_regular(s, size).width)
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

/// Line advance used when a string contains explicit "\n" line breaks.
const LINE_HEIGHT_EM: f32 = 1.35;

/// Draws text, replacing symbols the font has no glyph for with vector icons.
/// Explicit "\n" breaks start a new line below (fonts have no glyph for it).
fn draw_pieces(font: Option<&Font>, text: &str, x: f32, y: f32, size: f32, color: Color) {
    for (i, line) in text.split('\n').enumerate() {
        draw_line_pieces(font, line, x, y + i as f32 * size * LINE_HEIGHT_EM, size, color);
    }
}

/// Draws one line of text, with vector icons for missing glyphs.
fn draw_line_pieces(font: Option<&Font>, text: &str, x: f32, y: f32, size: f32, color: Color) {
    let params = TextParams {
        font,
        font_size: size.round() as u16,
        font_scale: 1.0,
        color,
        ..Default::default()
    };
    if !has_symbols(text) {
        draw_text_ex(text, x, y, params);
        return;
    }
    let mut cursor = x;
    for piece in split_pieces(text) {
        match piece {
            TextPiece::Text(run) => cursor += draw_text_ex(run, cursor, y, params.clone()).width,
            TextPiece::Symbol(icon) => {
                icon.draw(cursor, y, size, color);
                cursor += icon.advance_em() * size;
            }
        }
    }
}

/// Measures text the same way [`draw_pieces`] lays it out: the widest line, and the full
/// height of all lines.
fn measure_pieces(font: Option<&Font>, text: &str, size: f32) -> TextDimensions {
    if !text.contains('\n') {
        return measure_line_pieces(font, text, size);
    }
    let mut dims = TextDimensions { width: 0.0, height: 0.0, offset_y: 0.0 };
    let line_count = text.split('\n').count();
    for (i, line) in text.split('\n').enumerate() {
        let d = measure_line_pieces(font, line, size);
        dims.width = dims.width.max(d.width);
        if i == 0 {
            dims.offset_y = d.offset_y;
        }
    }
    dims.height = dims.offset_y + (line_count - 1) as f32 * size * LINE_HEIGHT_EM;
    dims
}

/// Measures one line of text, including vector icons.
fn measure_line_pieces(font: Option<&Font>, text: &str, size: f32) -> TextDimensions {
    let font_size = size.round() as u16;
    if !has_symbols(text) {
        return measure_text(text, font, font_size, 1.0);
    }
    let mut dims = TextDimensions { width: 0.0, height: 0.0, offset_y: 0.0 };
    for piece in split_pieces(text) {
        match piece {
            TextPiece::Text(run) => {
                let d = measure_text(run, font, font_size, 1.0);
                dims.width += d.width;
                dims.height = dims.height.max(d.height);
                dims.offset_y = dims.offset_y.max(d.offset_y);
            }
            TextPiece::Symbol(icon) => {
                dims.width += icon.advance_em() * size;
                dims.height = dims.height.max(size * 0.7);
                dims.offset_y = dims.offset_y.max(size * 0.7);
            }
        }
    }
    dims
}

/// Drops trailing characters (adding "…") until `measure` reports a width within `max_width`.
fn fit_to_width(text: &str, max_width: f32, measure: impl Fn(&str) -> f32) -> String {
    if text.is_empty() || measure(text) <= max_width {
        return text.to_string();
    }
    let ends: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    let shortened = |n: usize| format!("{}…", text[..ends[n]].trim_end());
    // Binary search for the longest prefix (in chars) that still fits with the ellipsis.
    let (mut lo, mut hi) = (0, ends.len() - 1);
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        if measure(&shortened(mid)) <= max_width {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    shortened(lo)
}

/// Estimate used when no graphics context exists (headless tests).
fn fallback_dimensions(text: &str, size: f32, char_w: f32) -> TextDimensions {
    let width = text
        .chars()
        .map(|ch| match symbol_icon(ch) {
            Some(icon) => icon.advance_em() * size,
            None => ch.len_utf8() as f32 * size * char_w,
        })
        .sum();
    TextDimensions {
        width,
        height: size,
        offset_y: size * 0.8,
    }
}

#[cfg(test)]
mod tests {
    use super::fit_to_width;

    fn char_count(s: &str) -> f32 {
        s.chars().count() as f32
    }

    #[test]
    fn fit_to_width_keeps_text_that_fits() {
        assert_eq!(fit_to_width("Classic Grand Prix", 40.0, char_count), "Classic Grand Prix");
        assert_eq!(fit_to_width("", 0.0, char_count), "");
    }

    #[test]
    fn fit_to_width_cuts_at_the_longest_prefix_and_adds_an_ellipsis() {
        let fitted = fit_to_width("Signature: T1 Apex Phantom GT", 12.0, char_count);
        assert_eq!(fitted, "Signature:…");
        assert!(char_count(&fitted) <= 12.0);
    }

    #[test]
    fn fit_to_width_never_splits_a_multibyte_character() {
        let fitted = fit_to_width("Nürburgring GP-Strecke", 4.0, char_count);
        assert_eq!(fitted, "Nür…");
    }
}
