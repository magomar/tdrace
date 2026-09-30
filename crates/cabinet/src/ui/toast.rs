use macroquad::color::Color;
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};

use crate::ui::layout::LayoutRect;
use crate::ui::theme::Palette;

/// Severity classification for transient toast notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ToastSeverity {
    #[default]
    Info,
    Success,
    Warning,
    Record,
}

impl ToastSeverity {
    /// Distinct accent border and badge color for the severity.
    pub fn accent_color(&self) -> Color {
        match self {
            Self::Info => Palette::NEON_CYAN,
            Self::Success => Palette::GREEN,
            Self::Warning => Palette::NEON_ORANGE,
            Self::Record => Palette::NEON_GOLD,
        }
    }

    /// Symbol prefix glyph for the severity.
    pub fn prefix_glyph(&self) -> &'static str {
        match self {
            Self::Info => "ℹ",
            Self::Success => "✓",
            Self::Warning => "⚠",
            Self::Record => "★",
        }
    }
}

/// An individual toast notification item with time decay and alpha envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToastItem {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub severity: ToastSeverity,
    pub duration_sec: f32,
    pub elapsed_sec: f32,
}

impl ToastItem {
    pub fn new(
        id: u64,
        title: impl Into<String>,
        message: impl Into<String>,
        severity: ToastSeverity,
        duration_sec: f32,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            message: message.into(),
            severity,
            duration_sec: duration_sec.max(0.5),
            elapsed_sec: 0.0,
        }
    }

    /// Advances the timer by dt seconds.
    pub fn tick(&mut self, dt: f32) {
        self.elapsed_sec += dt;
    }

    /// Returns true if the toast has exceeded its duration.
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.elapsed_sec >= self.duration_sec
    }

    /// Computes the alpha envelope (fade-in, sustain, fade-out).
    pub fn alpha(&self) -> f32 {
        let fade_in = 0.25;
        let fade_out = 0.50;

        if self.elapsed_sec < fade_in {
            (self.elapsed_sec / fade_in).clamp(0.0, 1.0)
        } else if self.elapsed_sec > self.duration_sec - fade_out {
            let remaining = (self.duration_sec - self.elapsed_sec).max(0.0);
            (remaining / fade_out).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

/// Overlay manager holding an active queue of transient notifications.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToastOverlay {
    pub toasts: Vec<ToastItem>,
    pub max_visible: usize,
    pub next_id: u64,
    pub base_x: f32,
    pub base_y: f32,
    pub toast_width: f32,
    pub toast_height: f32,
    pub gap: f32,
}

impl ToastOverlay {
    /// Creates a new ToastOverlay positioned at (base_x, base_y).
    pub fn new(base_x: f32, base_y: f32, toast_width: f32, toast_height: f32) -> Self {
        Self {
            toasts: Vec::new(),
            max_visible: 4,
            next_id: 1,
            base_x,
            base_y,
            toast_width,
            toast_height,
            gap: 8.0,
        }
    }

    /// Enqueues a new toast notification.
    pub fn push(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        severity: ToastSeverity,
        duration_sec: f32,
    ) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);

        let item = ToastItem::new(id, title, message, severity, duration_sec);
        self.toasts.push(item);

        // If exceeding max_visible, age out the oldest ones
        while self.toasts.len() > self.max_visible {
            self.toasts.remove(0);
        }

        id
    }

    /// Shortcut for pushing an info toast (3.0s).
    pub fn push_info(&mut self, title: impl Into<String>, message: impl Into<String>) -> u64 {
        self.push(title, message, ToastSeverity::Info, 3.0)
    }

    /// Shortcut for pushing a success toast (3.0s).
    pub fn push_success(&mut self, title: impl Into<String>, message: impl Into<String>) -> u64 {
        self.push(title, message, ToastSeverity::Success, 3.0)
    }

    /// Shortcut for pushing a warning toast (4.0s).
    pub fn push_warning(&mut self, title: impl Into<String>, message: impl Into<String>) -> u64 {
        self.push(title, message, ToastSeverity::Warning, 4.0)
    }

    /// Shortcut for pushing a new lap or personal best record toast (5.0s).
    pub fn push_record(&mut self, title: impl Into<String>, message: impl Into<String>) -> u64 {
        self.push(title, message, ToastSeverity::Record, 5.0)
    }

    /// Dismisses a specific toast by ID.
    pub fn dismiss(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    /// Clears all active toasts.
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Updates elapsed time for all toasts and removes expired ones.
    pub fn tick(&mut self, dt: f32) {
        for t in &mut self.toasts {
            t.tick(dt);
        }
        self.toasts.retain(|t| !t.is_expired());
    }

    /// Returns the bounding rectangle for the toast at visual index `idx`.
    pub fn toast_rect(&self, idx: usize) -> LayoutRect {
        let y = self.base_y + (idx as f32) * (self.toast_height + self.gap);
        LayoutRect::new(self.base_x, y, self.toast_width, self.toast_height)
    }

    /// Renders all active toasts with animated alpha transparency.
    pub fn render_frame(&self, idx: usize, toast: &ToastItem) {
        let r = self.toast_rect(idx);
        let alpha = toast.alpha();

        let bg = Color::new(0.06, 0.08, 0.12, 0.94 * alpha);
        let border = toast.severity.accent_color();
        let border_alpha = Color::new(border.r, border.g, border.b, border.a * alpha);

        draw_rectangle(r.x, r.y, r.w, r.h, bg);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.8, border_alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_alpha_decay_lifecycle() {
        let mut toast = ToastItem::new(1, "Personal Best", "1:12.345", ToastSeverity::Record, 3.0);

        // Frame 0: starting fade-in
        assert_eq!(toast.alpha(), 0.0);

        // At 0.25s: fully faded in
        toast.tick(0.25);
        assert!((toast.alpha() - 1.0).abs() < 1e-4);

        // At 1.5s: sustained fully opaque
        toast.tick(1.25);
        assert_eq!(toast.alpha(), 1.0);

        // At 2.75s: mid fade-out (0.25s remaining out of 0.5s fade-out -> alpha 0.5)
        toast.tick(1.25);
        assert!((toast.alpha() - 0.5).abs() < 1e-4);

        // At 3.0s: fully expired
        toast.tick(0.25);
        assert_eq!(toast.alpha(), 0.0);
        assert!(toast.is_expired());
    }

    #[test]
    fn test_toast_overlay_queue_and_culling() {
        let mut overlay = ToastOverlay::new(1500.0, 50.0, 380.0, 60.0);
        overlay.max_visible = 3;

        let _id1 = overlay.push_info("Info 1", "Msg 1");
        let id2 = overlay.push_success("Success 2", "Msg 2");
        let id3 = overlay.push_warning("Warning 3", "Msg 3");
        assert_eq!(overlay.toasts.len(), 3);

        // Pushing a 4th evicts the oldest (id1)
        let _id4 = overlay.push_record("Record 4", "Msg 4");
        assert_eq!(overlay.toasts.len(), 3);
        assert_eq!(overlay.toasts[0].id, id2);
        assert_eq!(overlay.toasts[1].id, id3);

        // Dismissing id3 removes it
        overlay.dismiss(id3);
        assert_eq!(overlay.toasts.len(), 2);

        // Tick past expiration
        overlay.tick(6.0);
        assert_eq!(overlay.toasts.len(), 0);
    }

    #[test]
    fn test_toast_rect_layout() {
        let overlay = ToastOverlay::new(100.0, 20.0, 200.0, 50.0);
        let r0 = overlay.toast_rect(0);
        assert_eq!(r0, LayoutRect::new(100.0, 20.0, 200.0, 50.0));

        let r1 = overlay.toast_rect(1);
        assert_eq!(r1, LayoutRect::new(100.0, 78.0, 200.0, 50.0));
    }
}
