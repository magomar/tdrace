use macroquad::color::Color;
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};

/// Responsive UI scaling and viewport adaptation for mobile and desktop screens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiScaler {
    pub screen_w: f32,
    pub screen_h: f32,
    pub scale: f32,
    pub safe_pad_x: f32,
    pub safe_pad_y: f32,
    pub is_mobile_aspect: bool,
}

impl UiScaler {
    /// Reference design baseline: 1280 x 720 (16:9 standard HD).
    pub const BASE_WIDTH: f32 = 1280.0;
    pub const BASE_HEIGHT: f32 = 720.0;

    /// Minimum ergonomic touch target size in pixels (48 CSS pixels / dp standard).
    pub const MIN_TOUCH_SIZE: f32 = 44.0;

    pub fn new(sw: f32, sh: f32) -> Self {
        let sw_clamped = sw.max(320.0);
        let sh_clamped = sh.max(240.0);

        let scale_w = sw_clamped / Self::BASE_WIDTH;
        let scale_h = sh_clamped / Self::BASE_HEIGHT;
        // Balance scale between width and height, bias slightly toward minimum to preserve screen space
        let raw_scale = scale_w.min(scale_h);
        let scale = raw_scale.clamp(0.65, 2.50);

        let aspect = sw_clamped / sh_clamped;
        let is_mobile_aspect = aspect > 2.0 || aspect < 1.35;

        // Safe area margins for mobile notches / rounded screen corners
        let safe_pad_x = if is_mobile_aspect { (sw_clamped * 0.035).clamp(16.0, 48.0) } else { 18.0 * scale };
        let safe_pad_y = if is_mobile_aspect { (sh_clamped * 0.035).clamp(14.0, 36.0) } else { 18.0 * scale };

        Self {
            screen_w: sw_clamped,
            screen_h: sh_clamped,
            scale,
            safe_pad_x,
            safe_pad_y,
            is_mobile_aspect,
        }
    }

    /// Scales a baseline pixel dimension proportionally.
    #[inline]
    pub fn s(&self, val: f32) -> f32 {
        val * self.scale
    }

    /// Scales a font size ensuring it never drops below readable thresholds.
    #[inline]
    pub fn font_s(&self, size: f32) -> f32 {
        (size * self.scale).max(11.0)
    }

    /// Enforces minimum mobile ergonomic touch target size.
    #[inline]
    pub fn touch_target(&self, size: f32) -> f32 {
        (size * self.scale).max(Self::MIN_TOUCH_SIZE)
    }

    /// Draws a modern glassmorphism panel with glowing high-contrast borders and translucent backdrop.
    pub fn draw_glass_card(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        bg_col: Color,
        border_col: Color,
        border_thickness: f32,
    ) {
        // Drop shadow
        draw_rectangle(
            x + self.s(2.0),
            y + self.s(3.0),
            w,
            h,
            Color::new(0.0, 0.0, 0.0, 0.35),
        );
        // Translucent card body
        draw_rectangle(x, y, w, h, bg_col);
        // Modern border outline
        draw_rectangle_lines(x, y, w, h, border_thickness * self.scale, border_col);
    }
}
