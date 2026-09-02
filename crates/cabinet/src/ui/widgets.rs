use macroquad::color::Color;
use macroquad::shapes::draw_rectangle;
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Renders a horizontal progress/stat bar with label, background track, and vibrant fill.
pub fn draw_stat_bar(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
    pct: f32,
    fill_col: Color,
) {
    let bar_h = scaler.s(10.0);
    let label_w = scaler.s(55.0);

    fonts.draw_ui_bold(label, x, y + scaler.s(9.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);

    let bar_x = x + label_w;
    let actual_bar_w = (w - label_w).max(scaler.s(40.0));

    // Track Background
    draw_rectangle(bar_x, y, actual_bar_w, bar_h, Color::new(0.08, 0.12, 0.18, 0.90));

    // Filled Bar
    let fill_w = (actual_bar_w * pct.clamp(0.0, 1.0)).max(scaler.s(2.0));
    draw_rectangle(bar_x, y, fill_w, bar_h, fill_col);
}

/// Renders an arcade chip / small metadata tag badge.
pub fn draw_chip(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    text_color: Color,
) {
    scaler.draw_glass_card(
        x,
        y,
        w,
        h,
        Color::new(0.06, 0.08, 0.12, 0.85),
        Palette::UI_CARD_BORDER,
        1.0,
    );
    fonts.draw_ui_bold_centered(
        label,
        x + w * 0.5,
        y + h * 0.68,
        scaler.font_s(10.5),
        text_color,
    );
}

/// Renders an interactive action button with title, subtitle/shortcut, and focus glow.
pub fn draw_action_button(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    title: &str,
    subtitle: Option<&str>,
    is_focused: bool,
    is_hovered: bool,
    accent_color: Color,
) {
    scaler.draw_button_card(x, y, w, h, is_focused, is_hovered, accent_color);

    let title_y = if subtitle.is_some() {
        y + h * 0.44
    } else {
        y + h * 0.62
    };

    fonts.draw_ui_bold_centered(
        title,
        x + w * 0.5,
        title_y,
        scaler.font_s(14.0),
        Palette::WHITE,
    );

    if let Some(sub) = subtitle {
        fonts.draw_ui_regular_centered(
            sub,
            x + w * 0.5,
            y + h * 0.80,
            scaler.font_s(10.5),
            Color::new(0.80, 0.88, 0.95, 0.90),
        );
    }
}
