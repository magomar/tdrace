//! Vector stand-ins for symbols the embedded fonts cannot render.
//!
//! Rajdhani and Barlow only cover Latin text, so arrows, stars, locks, medals and emoji in UI
//! strings used to render as empty "tofu" boxes. [`Fonts`](crate::ui::font::Fonts) splits text
//! into runs and draws each symbol listed here as a small shape sized to the font instead.

use macroquad::color::Color;
use macroquad::math::{vec2, Vec2};
use macroquad::shapes::{
    draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, draw_triangle,
    draw_triangle_lines,
};

/// A symbol drawn as vector shapes in place of a missing font glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolIcon {
    TriLeft,
    TriRight,
    TriUp,
    TriDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Star,
    Check,
    Cross,
    Lock,
    Trophy,
    /// 1 = gold, 2 = silver, 3 = bronze.
    Medal(u8),
    Flag,
    Warning,
    NoEntry,
    Hourglass,
    Gear,
    Bolt,
    Fullscreen,
    Refresh,
    DotFilled,
    DotHollow,
    Person,
    Car,
    Ruler,
    Cart,
    Save,
    /// Zero-width marks (variation selectors, flag letters) that are simply skipped.
    Hidden,
}

/// Maps a character with no glyph in the embedded fonts to its vector stand-in.
pub fn symbol_icon(ch: char) -> Option<SymbolIcon> {
    use SymbolIcon::*;
    Some(match ch {
        '◄' | '◀' => TriLeft,
        '►' | '▶' => TriRight,
        '▲' => TriUp,
        '▼' => TriDown,
        '←' | '⌫' => ArrowLeft,
        '→' | '➔' => ArrowRight,
        '↑' => ArrowUp,
        '↓' => ArrowDown,
        '★' | '⭐' => Star,
        '✓' | '✅' => Check,
        '✗' | '✕' | '❌' => Cross,
        '🔒' => Lock,
        '🏆' => Trophy,
        '🥇' => Medal(1),
        '🥈' => Medal(2),
        '🥉' => Medal(3),
        '🏁' | '🚀' => Flag,
        '⚠' => Warning,
        '🚫' => NoEntry,
        '⏳' => Hourglass,
        '⚙' => Gear,
        '⚡' => Bolt,
        '⛶' => Fullscreen,
        '⟳' => Refresh,
        '●' | '📢' => DotFilled,
        '○' => DotHollow,
        '👤' => Person,
        '🏎' => Car,
        '📐' => Ruler,
        '🛒' => Cart,
        '💾' => Save,
        '\u{FE0F}' | '\u{200D}' | '\u{1F1E6}'..='\u{1F1FF}' => Hidden,
        _ => return None,
    })
}

impl SymbolIcon {
    /// Horizontal space the symbol takes, as a fraction of the font size.
    pub fn advance_em(self) -> f32 {
        use SymbolIcon::*;
        match self {
            Hidden => 0.0,
            DotFilled | DotHollow => 0.5,
            TriLeft | TriRight | TriUp | TriDown | Bolt | Hourglass => 0.62,
            Check | Cross | Lock | Medal(_) | NoEntry | Person | Ruler | Save | Fullscreen => 0.75,
            Car => 1.05,
            _ => 0.82,
        }
    }

    /// Draws the symbol in a cell starting at `x`, sitting on text baseline `y`.
    pub fn draw(self, x: f32, y: f32, size: f32, color: Color) {
        use SymbolIcon::*;
        let adv = self.advance_em() * size;
        let c = vec2(x + adv * 0.5, y - size * 0.36);
        let h = size * 0.30;
        let t = (size * 0.09).max(1.0);
        let p = |dx: f32, dy: f32| c + vec2(dx * h, dy * h);
        let line = |a: Vec2, b: Vec2, w: f32| draw_line(a.x, a.y, b.x, b.y, w, color);

        match self {
            Hidden => {}
            TriLeft => draw_triangle(p(0.6, -0.85), p(0.6, 0.85), p(-0.8, 0.0), color),
            TriRight => draw_triangle(p(-0.6, -0.85), p(-0.6, 0.85), p(0.8, 0.0), color),
            TriUp => draw_triangle(p(-0.85, 0.6), p(0.85, 0.6), p(0.0, -0.8), color),
            TriDown => draw_triangle(p(-0.85, -0.6), p(0.85, -0.6), p(0.0, 0.8), color),
            ArrowLeft | ArrowRight | ArrowUp | ArrowDown => {
                let dir = match self {
                    ArrowLeft => vec2(-1.0, 0.0),
                    ArrowRight => vec2(1.0, 0.0),
                    ArrowUp => vec2(0.0, -1.0),
                    _ => vec2(0.0, 1.0),
                };
                let side = vec2(-dir.y, dir.x);
                let tip = c + dir * h;
                let back = tip - dir * h * 0.75;
                line(c - dir * h, back, t * 1.2);
                draw_triangle(tip, back + side * h * 0.6, back - side * h * 0.6, color);
            }
            Star => {
                let outer = h * 1.05;
                let inner = h * 0.44;
                let pt = |i: usize| {
                    let r = if i % 2 == 0 { outer } else { inner };
                    let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
                    c + vec2(a.cos(), a.sin()) * r
                };
                for i in 0..10 {
                    draw_triangle(c, pt(i), pt((i + 1) % 10), color);
                }
            }
            Check => {
                line(p(-0.8, 0.05), p(-0.25, 0.6), t * 1.4);
                line(p(-0.25, 0.6), p(0.85, -0.7), t * 1.4);
            }
            Cross => {
                line(p(-0.65, -0.65), p(0.65, 0.65), t * 1.3);
                line(p(-0.65, 0.65), p(0.65, -0.65), t * 1.3);
            }
            Lock => {
                draw_circle_lines(c.x, c.y - h * 0.3, h * 0.45, t, color);
                draw_rectangle(c.x - h * 0.72, c.y - h * 0.2, h * 1.44, h * 1.15, color);
            }
            Trophy => {
                draw_triangle(p(-0.7, -1.0), p(0.7, -1.0), p(0.0, 0.2), color);
                draw_circle(c.x, c.y - h * 0.45, h * 0.55, color);
                draw_circle_lines(c.x - h * 0.72, c.y - h * 0.55, h * 0.26, t * 0.8, color);
                draw_circle_lines(c.x + h * 0.72, c.y - h * 0.55, h * 0.26, t * 0.8, color);
                draw_rectangle(c.x - h * 0.12, c.y, h * 0.24, h * 0.55, color);
                draw_rectangle(c.x - h * 0.5, c.y + h * 0.55, h, h * 0.3, color);
            }
            Medal(rank) => {
                let disc = match rank {
                    1 => Color::new(1.0, 0.82, 0.15, color.a),
                    2 => Color::new(0.80, 0.83, 0.88, color.a),
                    _ => Color::new(0.82, 0.52, 0.26, color.a),
                };
                line(p(-0.55, -1.0), p(-0.1, -0.1), t);
                line(p(0.55, -1.0), p(0.1, -0.1), t);
                draw_circle(c.x, c.y + h * 0.3, h * 0.62, disc);
                draw_circle_lines(c.x, c.y + h * 0.3, h * 0.62, t * 0.6, Color::new(0.0, 0.0, 0.0, 0.35 * color.a));
            }
            Flag => {
                line(p(-0.75, -1.0), p(-0.75, 1.0), t);
                let (x0, y0) = (c.x - h * 0.65, c.y - h);
                let cell = h * 0.4;
                for row in 0..3 {
                    for col in 0..4 {
                        let a = if (row + col) % 2 == 0 { color.a } else { color.a * 0.3 };
                        draw_rectangle(
                            x0 + col as f32 * cell,
                            y0 + row as f32 * cell,
                            cell,
                            cell,
                            Color::new(color.r, color.g, color.b, a),
                        );
                    }
                }
            }
            Warning => {
                draw_triangle_lines(p(0.0, -1.0), p(-1.0, 0.85), p(1.0, 0.85), t, color);
                line(p(0.0, -0.4), p(0.0, 0.25), t * 1.1);
                draw_circle(c.x, c.y + h * 0.55, t * 0.65, color);
            }
            NoEntry => {
                draw_circle_lines(c.x, c.y, h * 0.85, t, color);
                line(p(-0.6, 0.6), p(0.6, -0.6), t);
            }
            Hourglass => {
                line(p(-0.7, -1.0), p(0.7, -1.0), t);
                line(p(-0.7, 1.0), p(0.7, 1.0), t);
                draw_triangle(p(-0.55, -0.9), p(0.55, -0.9), p(0.0, 0.0), color);
                draw_triangle_lines(p(-0.55, 0.9), p(0.55, 0.9), p(0.0, 0.0), t * 0.8, color);
            }
            Gear => {
                for i in 0..8 {
                    let a = i as f32 * std::f32::consts::FRAC_PI_4;
                    let d = vec2(a.cos(), a.sin());
                    line(c + d * h * 0.5, c + d * h * 0.95, t * 2.2);
                }
                draw_circle_lines(c.x, c.y, h * 0.55, t * 1.6, color);
            }
            Bolt => {
                draw_triangle(p(0.35, -1.0), p(-0.6, 0.15), p(0.1, 0.15), color);
                draw_triangle(p(-0.35, 1.0), p(0.6, -0.15), p(-0.1, -0.15), color);
            }
            Fullscreen => {
                for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                    let corner = p(0.85 * sx, 0.85 * sy);
                    line(corner, corner + vec2(-sx * h * 0.55, 0.0), t);
                    line(corner, corner + vec2(0.0, -sy * h * 0.55), t);
                }
            }
            Refresh => {
                let r = h * 0.75;
                let steps = 10;
                let (a0, a1) = (0.6_f32, 5.6_f32);
                let at = |a: f32| c + vec2(a.cos(), a.sin()) * r;
                for i in 0..steps {
                    let s = a0 + (a1 - a0) * i as f32 / steps as f32;
                    let e = a0 + (a1 - a0) * (i + 1) as f32 / steps as f32;
                    line(at(s), at(e), t);
                }
                let end = at(a1);
                draw_triangle(end + vec2(h * 0.45, 0.0), end - vec2(h * 0.2, h * 0.4), end - vec2(h * 0.2, -h * 0.4), color);
            }
            DotFilled => draw_circle(c.x, c.y, h * 0.4, color),
            DotHollow => draw_circle_lines(c.x, c.y, h * 0.4, t, color),
            Person => {
                draw_circle(c.x, c.y - h * 0.5, h * 0.35, color);
                draw_circle(c.x, c.y + h * 0.55, h * 0.55, color);
                draw_rectangle(c.x - h * 0.55, c.y + h * 0.55, h * 1.1, h * 0.35, color);
            }
            Car => {
                draw_rectangle(c.x - h * 1.1, c.y - h * 0.1, h * 2.2, h * 0.5, color);
                draw_triangle(p(-0.6, -0.1), p(-0.3, -0.55), p(0.3, -0.1), color);
                draw_triangle(p(-0.3, -0.55), p(0.4, -0.55), p(0.3, -0.1), color);
                draw_triangle(p(0.4, -0.55), p(0.7, -0.1), p(0.3, -0.1), color);
                draw_circle(c.x - h * 0.6, c.y + h * 0.45, h * 0.26, color);
                draw_circle(c.x + h * 0.6, c.y + h * 0.45, h * 0.26, color);
            }
            Ruler => draw_triangle_lines(p(-0.8, 0.85), p(-0.8, -0.85), p(0.8, 0.85), t, color),
            Cart => {
                line(p(-1.0, -0.8), p(-0.7, -0.8), t);
                line(p(-0.7, -0.8), p(-0.45, 0.35), t);
                line(p(-0.45, 0.35), p(0.75, 0.35), t);
                line(p(0.75, 0.35), p(0.95, -0.5), t);
                line(p(0.95, -0.5), p(-0.6, -0.5), t);
                draw_circle(c.x - h * 0.3, c.y + h * 0.75, h * 0.18, color);
                draw_circle(c.x + h * 0.55, c.y + h * 0.75, h * 0.18, color);
            }
            Save => {
                draw_rectangle_lines(c.x - h * 0.85, c.y - h * 0.85, h * 1.7, h * 1.7, t, color);
                draw_rectangle(c.x - h * 0.45, c.y - h * 0.85, h * 0.9, h * 0.55, color);
                draw_rectangle(c.x - h * 0.5, c.y + h * 0.2, h, h * 0.45, color);
            }
        }
    }
}

/// A piece of text to draw: either a plain run for the font, or a single vector symbol.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextPiece<'a> {
    Text(&'a str),
    Symbol(SymbolIcon),
}

/// Returns true when the text has at least one character that needs a vector stand-in.
pub fn has_symbols(text: &str) -> bool {
    !text.is_ascii() && text.chars().any(|ch| symbol_icon(ch).is_some())
}

/// Splits text into plain runs and symbols, in order.
pub fn split_pieces(text: &str) -> Vec<TextPiece<'_>> {
    let mut pieces = Vec::new();
    let mut run_start = 0;
    for (i, ch) in text.char_indices() {
        if let Some(icon) = symbol_icon(ch) {
            if run_start < i {
                pieces.push(TextPiece::Text(&text[run_start..i]));
            }
            if icon != SymbolIcon::Hidden {
                pieces.push(TextPiece::Symbol(icon));
            }
            run_start = i + ch.len_utf8();
        }
    }
    if run_start < text.len() {
        pieces.push(TextPiece::Text(&text[run_start..]));
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_has_no_symbols() {
        assert!(!has_symbols("PRESS [ENTER] TO RACE • 919m — Ñ"));
        assert_eq!(split_pieces("Quick Race"), vec![TextPiece::Text("Quick Race")]);
    }

    #[test]
    fn symbols_split_out_of_text_runs() {
        assert!(has_symbols("◄ BACK"));
        assert_eq!(
            split_pieces("◄ BACK ►"),
            vec![
                TextPiece::Symbol(SymbolIcon::TriLeft),
                TextPiece::Text(" BACK "),
                TextPiece::Symbol(SymbolIcon::TriRight),
            ]
        );
        assert_eq!(
            split_pieces("T3 (★★★)"),
            vec![
                TextPiece::Text("T3 ("),
                TextPiece::Symbol(SymbolIcon::Star),
                TextPiece::Symbol(SymbolIcon::Star),
                TextPiece::Symbol(SymbolIcon::Star),
                TextPiece::Text(")"),
            ]
        );
    }

    #[test]
    fn variation_selectors_and_flag_letters_are_dropped() {
        assert_eq!(
            split_pieces("⚙️ CONFIG"),
            vec![TextPiece::Symbol(SymbolIcon::Gear), TextPiece::Text(" CONFIG")]
        );
        assert_eq!(split_pieces("🇪🇸"), Vec::<TextPiece>::new());
    }

    #[test]
    fn every_symbol_has_a_positive_advance_except_hidden() {
        for ch in ['◄', '►', '▲', '▼', '→', '★', '✓', '✗', '🔒', '🏆', '🥇', '🥈', '🥉', '🏁', '⚠', '⏳', '⟳', '●', '○'] {
            let icon = symbol_icon(ch).unwrap();
            assert!(icon.advance_em() > 0.0, "{ch} must take space");
        }
        assert_eq!(SymbolIcon::Hidden.advance_em(), 0.0);
    }
}
