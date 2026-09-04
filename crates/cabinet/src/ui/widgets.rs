use macroquad::color::Color;
use macroquad::input::{is_key_pressed, is_mouse_button_down, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

#[inline]
fn safe_key_pressed(key: KeyCode) -> bool {
    std::panic::catch_unwind(|| is_key_pressed(key)).unwrap_or(false)
}

#[inline]
fn safe_mouse_pos() -> (f32, f32) {
    std::panic::catch_unwind(mouse_position).unwrap_or((-1000.0, -1000.0))
}

#[inline]
fn safe_mouse_down(btn: MouseButton) -> bool {
    std::panic::catch_unwind(|| is_mouse_button_down(btn)).unwrap_or(false)
}

#[inline]
fn safe_mouse_pressed(btn: MouseButton) -> bool {
    std::panic::catch_unwind(|| is_mouse_button_pressed(btn)).unwrap_or(false)
}

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

/// Interactive horizontal slider widget with custom range, stepping, percentage/decimal formatting,
/// mouse drag/click support, and keyboard/gamepad directional stepping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliderWidget {
    pub label: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub suffix: String,
    pub is_dragging: bool,
}

impl SliderWidget {
    /// Creates a new slider with specified range, step increment, and initial value.
    pub fn new(label: impl Into<String>, min: f32, max: f32, step: f32, initial_val: f32) -> Self {
        let label = label.into();
        let step = if step <= 0.0 { 0.01 } else { step };
        let min_val = min.min(max);
        let max_val = max.max(min);
        let value = initial_val.clamp(min_val, max_val);
        Self {
            label,
            value,
            min: min_val,
            max: max_val,
            step,
            suffix: String::new(),
            is_dragging: false,
        }
    }

    /// Convenience constructor for 0.0 ..= 1.0 percentage sliders (step 0.05, suffix "%").
    pub fn percentage(label: impl Into<String>, initial_ratio: f32) -> Self {
        Self::new(label, 0.0, 1.0, 0.05, initial_ratio).with_suffix("%")
    }

    /// Sets custom unit/display suffix (e.g. "%", "ms", "x", "px").
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    /// Sets value directly, clamping to [min, max] and snapping to nearest step.
    pub fn set_value(&mut self, val: f32) {
        let clamped = val.clamp(self.min, self.max);
        let steps = ((clamped - self.min) / self.step).round();
        let snapped = self.min + steps * self.step;
        self.value = snapped.clamp(self.min, self.max);
    }

    /// Returns current value normalized in 0.0..=1.0.
    #[inline]
    pub fn normalized(&self) -> f32 {
        let range = self.max - self.min;
        if range.abs() < 1e-6 {
            0.0
        } else {
            ((self.value - self.min) / range).clamp(0.0, 1.0)
        }
    }

    /// Sets value from normalized 0.0..=1.0 ratio.
    pub fn set_normalized(&mut self, ratio: f32) {
        let val = self.min + ratio.clamp(0.0, 1.0) * (self.max - self.min);
        self.set_value(val);
    }

    /// Increments value by step. Returns true if value changed.
    pub fn step_up(&mut self) -> bool {
        let old_val = self.value;
        self.set_value(self.value + self.step);
        (self.value - old_val).abs() > 1e-5
    }

    /// Decrements value by step. Returns true if value changed.
    pub fn step_down(&mut self) -> bool {
        let old_val = self.value;
        self.set_value(self.value - self.step);
        (self.value - old_val).abs() > 1e-5
    }

    /// Formatted value string. If suffix is "%", formats as e.g. "80%". Otherwise formats with clean decimals.
    pub fn formatted_value(&self) -> String {
        if self.suffix == "%" {
            let pct = (self.normalized() * 100.0).round() as i32;
            format!("{}%", pct)
        } else if self.step >= 1.0 {
            format!("{}{}", self.value.round() as i64, self.suffix)
        } else if self.step >= 0.1 {
            format!("{:.1}{}", self.value, self.suffix)
        } else {
            format!("{:.2}{}", self.value, self.suffix)
        }
    }

    /// Handles keyboard, gamepad, and mouse drag/click interactions.
    /// Returns true if value changed this frame.
    pub fn handle_input(
        &mut self,
        is_focused: bool,
        gamepad_left: bool,
        gamepad_right: bool,
        card_rect: (f32, f32, f32, f32),
    ) -> bool {
        let mut changed = false;

        // Keyboard & Gamepad horizontal adjustment when focused
        if is_focused {
            if safe_key_pressed(KeyCode::Left)
                || safe_key_pressed(KeyCode::A)
                || gamepad_left
            {
                changed |= self.step_down();
            }
            if safe_key_pressed(KeyCode::Right)
                || safe_key_pressed(KeyCode::D)
                || gamepad_right
            {
                changed |= self.step_up();
            }
        }

        // Mouse drag and click handling
        let (x, y, w, h) = card_rect;
        let (mx, my) = safe_mouse_pos();
        let is_over = mx >= x && mx <= x + w && my >= y && my <= y + h;

        if safe_mouse_down(MouseButton::Left) {
            if is_over || self.is_dragging {
                self.is_dragging = true;
                // In a slider card, track occupies horizontal bounds roughly from x + 16 to x + w - 16
                let track_pad = (w * 0.05).clamp(12.0, 24.0);
                let track_w = (w - track_pad * 2.0).max(1.0);
                let track_x = x + track_pad;
                let ratio = ((mx - track_x) / track_w).clamp(0.0, 1.0);
                let old_val = self.value;
                self.set_normalized(ratio);
                if (self.value - old_val).abs() > 1e-5 {
                    changed = true;
                }
            }
        } else {
            self.is_dragging = false;
        }

        changed
    }
}

/// Renders an arcade slider control with glassmorphism track, vibrant progress fill,
/// glowing thumb knob, and left/right value labels.
pub fn draw_slider(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    formatted_val: &str,
    pct: f32,
    is_focused: bool,
    is_hovered: bool,
    accent_color: Color,
) {
    let is_active = is_focused || is_hovered;
    // Outer container card
    scaler.draw_button_card(x, y, w, h, is_focused, is_hovered, accent_color);

    let pad_x = scaler.s(16.0);
    let label_y = y + h * 0.38;

    // Left label
    fonts.draw_ui_bold(
        label,
        x + pad_x,
        label_y,
        scaler.font_s(13.0),
        if is_active { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
    );

    // Right formatted value
    let val_dim = fonts.measure_ui_bold(formatted_val, scaler.font_s(13.0));
    fonts.draw_ui_bold(
        formatted_val,
        x + w - pad_x - val_dim.width,
        label_y,
        scaler.font_s(13.0),
        if is_active { accent_color } else { Palette::NEON_GOLD },
    );

    // Track trough coordinates
    let track_x = x + pad_x;
    let track_w = (w - pad_x * 2.0).max(scaler.s(40.0));
    let track_h = scaler.s(8.0);
    let track_y = y + h * 0.65;

    // Track trough background
    draw_rectangle(track_x, track_y, track_w, track_h, Color::new(0.06, 0.08, 0.12, 0.95));
    draw_rectangle_lines(track_x, track_y, track_w, track_h, 1.0 * scaler.scale, Palette::UI_CARD_BORDER);

    // Filled progress
    let fill_w = (track_w * pct.clamp(0.0, 1.0)).max(0.0);
    if fill_w > 0.0 {
        draw_rectangle(track_x, track_y, fill_w, track_h, accent_color);
    }

    // Thumb knob indicator
    let thumb_x = track_x + fill_w;
    let thumb_w = scaler.s(10.0);
    let thumb_h = scaler.s(18.0);
    let thumb_rect_x = thumb_x - thumb_w * 0.5;
    let thumb_rect_y = track_y + track_h * 0.5 - thumb_h * 0.5;

    let thumb_bg = if is_active { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 0.95) };
    draw_rectangle(thumb_rect_x, thumb_rect_y, thumb_w, thumb_h, thumb_bg);
    draw_rectangle_lines(
        thumb_rect_x,
        thumb_rect_y,
        thumb_w,
        thumb_h,
        if is_active { 2.2 * scaler.scale } else { 1.2 * scaler.scale },
        accent_color,
    );
}

/// Interactive dropdown and cycle-stepper widget for arcade settings menus.
/// Supports both fast inline arrow cycling (optimal for gamepads/arcade sticks)
/// and full expandable modal popup option menus (optimal for mouse/touch).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DropdownWidget {
    pub label: String,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub is_open: bool,
    pub popup_hovered_index: Option<usize>,
}

impl DropdownWidget {
    pub fn new(label: impl Into<String>, options: Vec<String>, default_index: usize) -> Self {
        let label = label.into();
        let selected_index = if options.is_empty() {
            0
        } else {
            default_index.min(options.len() - 1)
        };
        Self {
            label,
            options,
            selected_index,
            is_open: false,
            popup_hovered_index: None,
        }
    }

    /// Cycles to previous option (wraps around). Returns true if index changed.
    pub fn cycle_prev(&mut self) -> bool {
        if self.options.len() <= 1 {
            return false;
        }
        if self.selected_index == 0 {
            self.selected_index = self.options.len() - 1;
        } else {
            self.selected_index -= 1;
        }
        true
    }

    /// Cycles to next option (wraps around). Returns true if index changed.
    pub fn cycle_next(&mut self) -> bool {
        if self.options.len() <= 1 {
            return false;
        }
        if self.selected_index + 1 >= self.options.len() {
            self.selected_index = 0;
        } else {
            self.selected_index += 1;
        }
        true
    }

    /// Returns currently selected option string.
    pub fn selected_option(&self) -> &str {
        self.options.get(self.selected_index).map(|s| s.as_str()).unwrap_or("")
    }

    /// Sets selected index directly with bounds check.
    pub fn set_selected(&mut self, index: usize) -> bool {
        if index < self.options.len() && index != self.selected_index {
            self.selected_index = index;
            true
        } else {
            false
        }
    }

    /// Toggles the expandable popup state.
    pub fn toggle_open(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open {
            self.popup_hovered_index = Some(self.selected_index);
        }
    }

    /// Closes the expandable popup.
    pub fn close(&mut self) {
        self.is_open = false;
        self.popup_hovered_index = None;
    }

    /// Handles keyboard, gamepad, and mouse interactions.
    /// Returns true if selection changed this frame.
    pub fn handle_input(
        &mut self,
        is_focused: bool,
        gamepad_left: bool,
        gamepad_right: bool,
        gamepad_up: bool,
        gamepad_down: bool,
        gamepad_confirm: bool,
        gamepad_cancel: bool,
        rect: (f32, f32, f32, f32),
        scaler: &UiScaler,
    ) -> bool {
        let mut changed = false;
        let (x, y, w, h) = rect;
        let (mx, my) = safe_mouse_pos();
        let mouse_clicked = safe_mouse_pressed(MouseButton::Left);

        if self.is_open {
            // Popup open mode: vertical navigation and selection
            if safe_key_pressed(KeyCode::Up) || safe_key_pressed(KeyCode::W) || gamepad_up {
                let curr = self.popup_hovered_index.unwrap_or(self.selected_index);
                self.popup_hovered_index = Some(if curr == 0 { self.options.len().saturating_sub(1) } else { curr - 1 });
            }
            if safe_key_pressed(KeyCode::Down) || safe_key_pressed(KeyCode::S) || gamepad_down {
                let curr = self.popup_hovered_index.unwrap_or(self.selected_index);
                self.popup_hovered_index = Some(if curr + 1 >= self.options.len() { 0 } else { curr + 1 });
            }

            if safe_key_pressed(KeyCode::Enter) || safe_key_pressed(KeyCode::KpEnter) || gamepad_confirm {
                if let Some(idx) = self.popup_hovered_index {
                    changed |= self.set_selected(idx);
                }
                self.close();
                return changed;
            }

            if safe_key_pressed(KeyCode::Escape) || gamepad_cancel {
                self.close();
                return false;
            }

            // Mouse hover and click inside popup
            let item_h = scaler.s(32.0);
            let popup_w = (w * 0.58).clamp(scaler.s(160.0), scaler.s(320.0));
            let popup_x = x + w - popup_w - scaler.s(16.0);
            let popup_y = y + h + scaler.s(4.0);

            for (idx, _) in self.options.iter().enumerate() {
                let iy = popup_y + idx as f32 * item_h;
                if mx >= popup_x && mx <= popup_x + popup_w && my >= iy && my <= iy + item_h {
                    self.popup_hovered_index = Some(idx);
                    if mouse_clicked {
                        changed |= self.set_selected(idx);
                        self.close();
                        return changed;
                    }
                }
            }

            // Clicking outside closes popup
            if mouse_clicked {
                let total_popup_h = self.options.len() as f32 * item_h;
                let inside_popup = mx >= popup_x && mx <= popup_x + popup_w && my >= popup_y && my <= popup_y + total_popup_h;
                let inside_card = mx >= x && mx <= x + w && my >= y && my <= y + h;
                if !inside_popup && !inside_card {
                    self.close();
                }
            }
        } else {
            // Closed inline stepper mode
            if is_focused {
                if safe_key_pressed(KeyCode::Left) || safe_key_pressed(KeyCode::A) || gamepad_left {
                    changed |= self.cycle_prev();
                }
                if safe_key_pressed(KeyCode::Right) || safe_key_pressed(KeyCode::D) || gamepad_right {
                    changed |= self.cycle_next();
                }
                if safe_key_pressed(KeyCode::Enter) || safe_key_pressed(KeyCode::KpEnter) || gamepad_confirm {
                    self.toggle_open();
                }
            }

            // Mouse interaction on the stepper controls
            let opt_w = (w * 0.58).clamp(scaler.s(160.0), scaler.s(320.0));
            let opt_x = x + w - opt_w - scaler.s(16.0);
            let opt_h = scaler.s(32.0);
            let opt_y = y + (h - opt_h) * 0.5;
            let arrow_w = scaler.s(28.0);

            let left_arrow_rect = (opt_x, opt_y, arrow_w, opt_h);
            let right_arrow_rect = (opt_x + opt_w - arrow_w, opt_y, arrow_w, opt_h);
            let center_rect = (opt_x + arrow_w, opt_y, opt_w - arrow_w * 2.0, opt_h);

            if mouse_clicked {
                let check = |(rx, ry, rw, rh): (f32, f32, f32, f32)| {
                    mx >= rx && mx <= rx + rw && my >= ry && my <= ry + rh
                };
                if check(left_arrow_rect) {
                    changed |= self.cycle_prev();
                } else if check(right_arrow_rect) {
                    changed |= self.cycle_next();
                } else if check(center_rect) {
                    self.toggle_open();
                }
            }
        }

        changed
    }
}

/// Renders an inline arcade cycle-stepper `[ < ] [ Option ] [ > ]` with chevron buttons.
pub fn draw_stepper(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    current_option: &str,
    is_focused: bool,
    is_hovered: bool,
    accent_color: Color,
) {
    let is_active = is_focused || is_hovered;
    scaler.draw_button_card(x, y, w, h, is_focused, is_hovered, accent_color);

    let pad_x = scaler.s(16.0);
    let label_y = y + h * 0.62;

    // Left label
    fonts.draw_ui_bold(
        label,
        x + pad_x,
        label_y,
        scaler.font_s(13.5),
        if is_active { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
    );

    // Stepper container on the right
    let opt_w = (w * 0.58).clamp(scaler.s(160.0), scaler.s(320.0));
    let opt_x = x + w - opt_w - pad_x;
    let opt_h = scaler.s(32.0);
    let opt_y = y + (h - opt_h) * 0.5;
    let arrow_w = scaler.s(28.0);

    // Stepper body backdrop
    draw_rectangle(opt_x, opt_y, opt_w, opt_h, Color::new(0.06, 0.08, 0.12, 0.95));
    draw_rectangle_lines(
        opt_x,
        opt_y,
        opt_w,
        opt_h,
        if is_active { 1.8 * scaler.scale } else { 1.0 * scaler.scale },
        if is_active { accent_color } else { Palette::UI_CARD_BORDER },
    );

    // Left chevron "<" button
    fonts.draw_ui_bold_centered(
        "<",
        opt_x + arrow_w * 0.5,
        opt_y + opt_h * 0.68,
        scaler.font_s(14.0),
        if is_active { accent_color } else { Palette::UI_TEXT_MUTED },
    );

    // Right chevron ">" button
    fonts.draw_ui_bold_centered(
        ">",
        opt_x + opt_w - arrow_w * 0.5,
        opt_y + opt_h * 0.68,
        scaler.font_s(14.0),
        if is_active { accent_color } else { Palette::UI_TEXT_MUTED },
    );

    // Center option label
    fonts.draw_ui_bold_centered(
        current_option,
        opt_x + opt_w * 0.5,
        opt_y + opt_h * 0.68,
        scaler.font_s(12.5),
        Palette::WHITE,
    );
}

/// Renders a dropdown widget: renders the inline stepper header plus an expandable floating popup
/// card if `is_open` is true.
pub fn draw_dropdown(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    options: &[String],
    selected_idx: usize,
    is_open: bool,
    hovered_idx: Option<usize>,
    is_focused: bool,
    is_hovered: bool,
    accent_color: Color,
) {
    let current_opt = options.get(selected_idx).map(|s| s.as_str()).unwrap_or("");
    draw_stepper(scaler, fonts, x, y, w, h, label, current_opt, is_focused, is_hovered, accent_color);

    if is_open && !options.is_empty() {
        let pad_x = scaler.s(16.0);
        let opt_w = (w * 0.58).clamp(scaler.s(160.0), scaler.s(320.0));
        let opt_x = x + w - opt_w - pad_x;
        let item_h = scaler.s(32.0);
        let total_popup_h = options.len() as f32 * item_h;
        let popup_y = y + h + scaler.s(4.0);

        // Glassmorphism popup container card with drop shadow
        scaler.draw_glass_card(
            opt_x,
            popup_y,
            opt_w,
            total_popup_h,
            Color::new(0.07, 0.09, 0.14, 0.98),
            accent_color,
            1.8,
        );

        for (idx, opt) in options.iter().enumerate() {
            let iy = popup_y + idx as f32 * item_h;
            let is_opt_selected = idx == selected_idx;
            let is_opt_hovered = hovered_idx == Some(idx);

            if is_opt_hovered {
                draw_rectangle(
                    opt_x + scaler.s(2.0),
                    iy + scaler.s(1.0),
                    opt_w - scaler.s(4.0),
                    item_h - scaler.s(2.0),
                    Color::new(accent_color.r * 0.25, accent_color.g * 0.25, accent_color.b * 0.25, 0.95),
                );
            }

            let text_color = if is_opt_selected {
                accent_color
            } else if is_opt_hovered {
                Palette::WHITE
            } else {
                Palette::UI_TEXT_MUTED
            };

            let prefix = if is_opt_selected { "> " } else { "  " };
            let display_text = format!("{}{}", prefix, opt);
            fonts.draw_ui_bold(
                &display_text,
                opt_x + scaler.s(12.0),
                iy + item_h * 0.68,
                scaler.font_s(12.5),
                text_color,
            );
        }
    }
}

/// Interactive horizontal tab bar for switching arcade menus and settings categories.
/// Supports bumper/shoulder buttons (`[LB] / [RB]`, `Q / E`), mouse hover and click,
/// and responsive neon pill/underline rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabBar {
    pub tabs: Vec<String>,
    pub active_tab: usize,
}

impl TabBar {
    /// Creates a new tab bar with the given tab labels.
    pub fn new(tabs: Vec<String>) -> Self {
        Self {
            tabs,
            active_tab: 0,
        }
    }

    /// Builder to initialize with specific active tab index.
    pub fn with_active(mut self, active: usize) -> Self {
        if !self.tabs.is_empty() {
            self.active_tab = active.min(self.tabs.len() - 1);
        }
        self
    }

    /// Cycles to the previous tab (wraps around). Returns true if index changed.
    pub fn prev_tab(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        if self.active_tab == 0 {
            self.active_tab = self.tabs.len() - 1;
        } else {
            self.active_tab -= 1;
        }
        true
    }

    /// Cycles to the next tab (wraps around). Returns true if index changed.
    pub fn next_tab(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        if self.active_tab + 1 >= self.tabs.len() {
            self.active_tab = 0;
        } else {
            self.active_tab += 1;
        }
        true
    }

    /// Directly sets active tab if valid. Returns true if changed.
    pub fn set_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() && index != self.active_tab {
            self.active_tab = index;
            true
        } else {
            false
        }
    }

    /// Returns active tab index.
    #[inline]
    pub fn active_index(&self) -> usize {
        self.active_tab
    }

    /// Returns label of active tab.
    pub fn active_tab_name(&self) -> &str {
        self.tabs.get(self.active_tab).map(|s| s.as_str()).unwrap_or("")
    }

    /// Handles keyboard, gamepad, and mouse tab switching.
    /// Returns true if active tab changed this frame.
    pub fn handle_input(
        &mut self,
        gamepad_prev: bool,
        gamepad_next: bool,
        rect: (f32, f32, f32, f32),
    ) -> bool {
        let mut changed = false;

        // Keyboard Q/E, PageUp/PageDown, or Gamepad LB/RB
        if safe_key_pressed(KeyCode::Q) || safe_key_pressed(KeyCode::PageUp) || gamepad_prev {
            changed |= self.prev_tab();
        }
        if safe_key_pressed(KeyCode::E) || safe_key_pressed(KeyCode::PageDown) || gamepad_next {
            changed |= self.next_tab();
        }

        // Mouse click on tabs
        let (x, y, w, h) = rect;
        let (mx, my) = safe_mouse_pos();
        if safe_mouse_pressed(MouseButton::Left) && !self.tabs.is_empty() {
            let tab_w = w / self.tabs.len() as f32;
            for (idx, _) in self.tabs.iter().enumerate() {
                let tx = x + idx as f32 * tab_w;
                if mx >= tx && mx <= tx + tab_w && my >= y && my <= y + h {
                    changed |= self.set_tab(idx);
                    break;
                }
            }
        }

        changed
    }
}

/// Renders a horizontal arcade tab bar with glassmorphism tabs, active glowing pill/underline,
/// and shortcut indicators.
pub fn draw_tab_bar(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    tabs: &[String],
    active_tab: usize,
    hovered_tab: Option<usize>,
    accent_color: Color,
) {
    if tabs.is_empty() {
        return;
    }

    // Shelf base container
    scaler.draw_glass_card(
        x,
        y,
        w,
        h,
        Color::new(0.06, 0.08, 0.12, 0.88),
        Palette::UI_CARD_BORDER,
        1.2,
    );

    let tab_w = w / tabs.len() as f32;
    let (mx, my) = safe_mouse_pos();

    for (idx, tab_name) in tabs.iter().enumerate() {
        let tx = x + idx as f32 * tab_w;
        let is_active = idx == active_tab;
        let is_hovered = hovered_tab == Some(idx) || (mx >= tx && mx <= tx + tab_w && my >= y && my <= y + h);

        if is_active {
            // Filled active tab pill
            draw_rectangle(
                tx + scaler.s(2.0),
                y + scaler.s(2.0),
                tab_w - scaler.s(4.0),
                h - scaler.s(4.0),
                Color::new(accent_color.r * 0.25, accent_color.g * 0.25, accent_color.b * 0.25, 0.95),
            );
            // Glowing underline bar
            draw_rectangle(
                tx + scaler.s(8.0),
                y + h - scaler.s(3.0),
                tab_w - scaler.s(16.0),
                scaler.s(3.0),
                accent_color,
            );
        } else if is_hovered {
            // Hover highlight
            draw_rectangle(
                tx + scaler.s(2.0),
                y + scaler.s(2.0),
                tab_w - scaler.s(4.0),
                h - scaler.s(4.0),
                Color::new(0.12, 0.16, 0.24, 0.80),
            );
        }

        // Tab separator divider
        if idx > 0 {
            draw_rectangle(
                tx,
                y + scaler.s(6.0),
                scaler.s(1.0),
                h - scaler.s(12.0),
                Color::new(0.20, 0.28, 0.38, 0.50),
            );
        }

        let text_color = if is_active {
            Palette::WHITE
        } else if is_hovered {
            Color::new(0.88, 0.92, 0.98, 1.0)
        } else {
            Palette::UI_TEXT_MUTED
        };

        fonts.draw_ui_bold_centered(
            tab_name,
            tx + tab_w * 0.5,
            y + h * 0.65,
            scaler.font_s(13.0),
            text_color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_widget_math_and_clamping() {
        let mut slider = SliderWidget::new("Volume", 0.0, 100.0, 10.0, 50.0);
        assert_eq!(slider.value, 50.0);
        assert_eq!(slider.normalized(), 0.5);

        // Step up
        assert!(slider.step_up());
        assert_eq!(slider.value, 60.0);

        // Set out of range
        slider.set_value(150.0);
        assert_eq!(slider.value, 100.0);
        assert_eq!(slider.normalized(), 1.0);
        assert!(!slider.step_up()); // Cannot step above max

        // Step down
        slider.set_value(5.0);
        assert_eq!(slider.value, 10.0); // Snaps to step 10.0
        assert!(slider.step_down());
        assert_eq!(slider.value, 0.0);
        assert!(!slider.step_down()); // Cannot step below min
    }

    #[test]
    fn test_slider_widget_percentage_formatting() {
        let mut slider = SliderWidget::percentage("Music", 0.75);
        assert_eq!(slider.formatted_value(), "75%");

        slider.set_normalized(0.30);
        assert_eq!(slider.formatted_value(), "30%");
    }

    #[test]
    fn test_dropdown_widget_cycling_and_popup() {
        let options = vec![
            "Cyberpunk Neon".to_string(),
            "Solar Flare".to_string(),
            "Monokai Dark".to_string(),
        ];
        let mut dropdown = DropdownWidget::new("Theme", options, 0);
        assert_eq!(dropdown.selected_option(), "Cyberpunk Neon");
        assert_eq!(dropdown.selected_index, 0);

        // Cycle next
        assert!(dropdown.cycle_next());
        assert_eq!(dropdown.selected_option(), "Solar Flare");
        assert_eq!(dropdown.selected_index, 1);

        assert!(dropdown.cycle_next());
        assert_eq!(dropdown.selected_option(), "Monokai Dark");
        assert_eq!(dropdown.selected_index, 2);

        // Wraps around
        assert!(dropdown.cycle_next());
        assert_eq!(dropdown.selected_option(), "Cyberpunk Neon");
        assert_eq!(dropdown.selected_index, 0);

        // Cycle prev wraps around
        assert!(dropdown.cycle_prev());
        assert_eq!(dropdown.selected_option(), "Monokai Dark");
        assert_eq!(dropdown.selected_index, 2);

        // Popup open and selection
        assert!(!dropdown.is_open);
        dropdown.toggle_open();
        assert!(dropdown.is_open);
        assert_eq!(dropdown.popup_hovered_index, Some(2));

        assert!(dropdown.set_selected(1));
        assert_eq!(dropdown.selected_option(), "Solar Flare");

        dropdown.close();
        assert!(!dropdown.is_open);
        assert_eq!(dropdown.popup_hovered_index, None);
    }

    #[test]
    fn test_tab_bar_cycling_and_selection() {
        let tabs = vec![
            "AUDIO".to_string(),
            "CONTROLS".to_string(),
            "DISPLAY".to_string(),
            "GAMEPLAY".to_string(),
        ];
        let mut tab_bar = TabBar::new(tabs);
        assert_eq!(tab_bar.active_index(), 0);
        assert_eq!(tab_bar.active_tab_name(), "AUDIO");

        // Next tab
        assert!(tab_bar.next_tab());
        assert_eq!(tab_bar.active_index(), 1);
        assert_eq!(tab_bar.active_tab_name(), "CONTROLS");

        // Set tab
        assert!(tab_bar.set_tab(3));
        assert_eq!(tab_bar.active_index(), 3);
        assert_eq!(tab_bar.active_tab_name(), "GAMEPLAY");

        // Wrap next
        assert!(tab_bar.next_tab());
        assert_eq!(tab_bar.active_index(), 0);

        // Wrap prev
        assert!(tab_bar.prev_tab());
        assert_eq!(tab_bar.active_index(), 3);
    }
}
