use macroquad::color::Color;
use macroquad::shapes::draw_rectangle;

use crate::ui::font::Fonts;
use crate::ui::layout::LayoutRect;
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Visual presentation style for a MetricBar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricBarStyle {
    Stat,
    Progress,
}

/// Horizontal stat or progress bar widget displaying label, fill ratio, and value readout.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricBar {
    pub label: String,
    pub ratio: f32, // 0.0..=1.0
    pub display_val: String,
    pub style: MetricBarStyle,
    pub bar_color: Color,
}

impl MetricBar {
    pub fn new(
        label: impl Into<String>,
        ratio: f32,
        display_val: impl Into<String>,
        style: MetricBarStyle,
        bar_color: Color,
    ) -> Self {
        Self {
            label: label.into(),
            ratio: ratio.clamp(0.0, 1.0),
            display_val: display_val.into(),
            style,
            bar_color,
        }
    }

    pub fn stat(
        label: impl Into<String>,
        ratio: f32,
        display_val: impl Into<String>,
        bar_color: Color,
    ) -> Self {
        Self::new(label, ratio, display_val, MetricBarStyle::Stat, bar_color)
    }

    pub fn progress(
        label: impl Into<String>,
        ratio: f32,
        display_val: impl Into<String>,
        bar_color: Color,
    ) -> Self {
        Self::new(label, ratio, display_val, MetricBarStyle::Progress, bar_color)
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(0.0, 1.0);
    }

    /// Renders the metric bar within the given bounds.
    pub fn draw(&self, scaler: &UiScaler, fonts: &Fonts, bounds: LayoutRect) {
        let label_w = scaler.s(70.0);
        let val_w = scaler.s(50.0);

        // Label on left
        fonts.draw_ui_bold(
            &self.label,
            bounds.x,
            bounds.y + bounds.h * 0.75,
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );

        // Value on right
        let val_size = scaler.font_s(11.0);
        let val_dim = fonts.measure_ui_bold(&self.display_val, val_size);
        fonts.draw_ui_bold(
            &self.display_val,
            bounds.x + bounds.w - val_dim.width,
            bounds.y + bounds.h * 0.75,
            val_size,
            self.bar_color,
        );

        // Track and fill in between
        let bar_x = bounds.x + label_w;
        let bar_w = (bounds.w - label_w - val_w - scaler.s(10.0)).max(scaler.s(20.0));
        let bar_h = (bounds.h * 0.45).clamp(scaler.s(4.0), scaler.s(12.0));
        let bar_y = bounds.y + (bounds.h - bar_h) * 0.5;

        // Background track
        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.08, 0.12, 0.18, 0.90));

        // Filled ratio
        let fill_w = (bar_w * self.ratio).max(if self.ratio > 0.0 { scaler.s(2.0) } else { 0.0 });
        draw_rectangle(bar_x, bar_y, fill_w, bar_h, self.bar_color);
    }
}

/// Standalone visual progress bar component.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressBar {
    pub ratio: f32,
    pub bar_color: Color,
    pub bg_color: Color,
}

impl ProgressBar {
    pub fn new(ratio: f32, bar_color: Color) -> Self {
        Self {
            ratio: ratio.clamp(0.0, 1.0),
            bar_color,
            bg_color: Color::new(0.08, 0.12, 0.18, 0.90),
        }
    }

    pub fn with_bg_color(mut self, bg_color: Color) -> Self {
        self.bg_color = bg_color;
        self
    }

    pub fn draw(&self, scaler: &UiScaler, bounds: LayoutRect) {
        draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, self.bg_color);
        let fill_w = (bounds.w * self.ratio).max(if self.ratio > 0.0 { scaler.s(2.0) } else { 0.0 });
        draw_rectangle(bounds.x, bounds.y, fill_w, bounds.h, self.bar_color);
    }
}

/// High-impact metric card displaying category label, primary KPI readout, and optional subtext.
#[derive(Debug, Clone, PartialEq)]
pub struct KpiTile {
    pub label: String,
    pub primary_metric: String,
    pub subtext: Option<String>,
    pub accent_color: Color,
}

impl KpiTile {
    pub fn new(
        label: impl Into<String>,
        primary_metric: impl Into<String>,
        accent_color: Color,
    ) -> Self {
        Self {
            label: label.into(),
            primary_metric: primary_metric.into(),
            subtext: None,
            accent_color,
        }
    }

    pub fn with_subtext(mut self, subtext: impl Into<String>) -> Self {
        self.subtext = Some(subtext.into());
        self
    }

    /// Renders the KPI tile inside the given rectangle.
    pub fn draw(&self, scaler: &UiScaler, fonts: &Fonts, bounds: LayoutRect) {
        scaler.draw_glass_card(
            bounds.x,
            bounds.y,
            bounds.w,
            bounds.h,
            Color::new(0.06, 0.09, 0.14, 0.85),
            Palette::UI_CARD_BORDER,
            1.0,
        );

        // Label at top
        fonts.draw_ui_bold(
            &self.label,
            bounds.x + scaler.s(12.0),
            bounds.y + scaler.s(18.0),
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );

        // Primary metric in center
        fonts.draw_ui_bold_centered(
            &self.primary_metric,
            bounds.x + bounds.w * 0.5,
            bounds.y + bounds.h * 0.58,
            scaler.font_s(20.0),
            self.accent_color,
        );

        // Subtext at bottom if provided
        if let Some(ref sub) = self.subtext {
            fonts.draw_ui_regular_centered(
                sub,
                bounds.x + bounds.w * 0.5,
                bounds.y + bounds.h * 0.84,
                scaler.font_s(10.0),
                Palette::UI_TEXT_MUTED,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_bar_clamping_and_styles() {
        let mut bar = MetricBar::stat("Top Speed", 1.5, "280 km/h", Palette::BLUE);
        assert_eq!(bar.ratio, 1.0); // clamped to 1.0
        assert_eq!(bar.style, MetricBarStyle::Stat);

        bar.set_ratio(-0.2);
        assert_eq!(bar.ratio, 0.0); // clamped to 0.0

        bar.set_ratio(0.75);
        assert_eq!(bar.ratio, 0.75);
    }

    #[test]
    fn test_progress_bar_clamping() {
        let p = ProgressBar::new(0.65, Palette::NEON_GREEN);
        assert_eq!(p.ratio, 0.65);
    }

    #[test]
    fn test_kpi_tile_construction() {
        let tile = KpiTile::new("STUNT SCORE", "14,500", Palette::NEON_GOLD)
            .with_subtext("RANK #1 TODAY");
        assert_eq!(tile.label, "STUNT SCORE");
        assert_eq!(tile.primary_metric, "14,500");
        assert_eq!(tile.subtext.as_deref(), Some("RANK #1 TODAY"));
        assert_eq!(tile.accent_color, Palette::NEON_GOLD);
    }
}
