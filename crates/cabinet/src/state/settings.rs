use macroquad::color::Color;
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use crate::audio::AudioSettings;
use crate::input::{GamepadConfig, NavGrid2D};
use crate::state::stack::{CabinetContext, CabinetScreen, ScreenAction};
use crate::ui::theme::Palette;
use crate::ui::widgets::{draw_dropdown, draw_slider, draw_tab_bar, DropdownWidget, SliderWidget, TabBar};

/// Comprehensive, reusable Arcade Settings Modal screen.
/// Implements `CabinetScreen` and unifies `TabBar`, `SliderWidget`, and `DropdownWidget`
/// to provide full in-game configuration of Audio, Controls, Display, and Gameplay options.
pub struct ArcadeSettingsModal {
    pub tab_bar: TabBar,
    pub nav: NavGrid2D,

    // Audio Tab Widgets
    pub master_slider: SliderWidget,
    pub music_slider: SliderWidget,
    pub sfx_slider: SliderWidget,
    pub ui_slider: SliderWidget,
    pub mute_dropdown: DropdownWidget,

    // Controls Tab Widgets
    pub stick_deadzone_slider: SliderWidget,
    pub trigger_deadzone_slider: SliderWidget,
    pub steer_sensitivity_slider: SliderWidget,
    pub steer_exponent_slider: SliderWidget,

    // Display / Theme Widgets
    pub theme_dropdown: DropdownWidget,
    pub scanlines_dropdown: DropdownWidget,
    pub ui_scale_dropdown: DropdownWidget,

    // Gameplay Widgets
    pub assist_dropdown: DropdownWidget,
    pub speed_unit_dropdown: DropdownWidget,
    pub ghost_car_dropdown: DropdownWidget,

    pub is_saved: bool,
}

impl Default for ArcadeSettingsModal {
    fn default() -> Self {
        Self::new(&AudioSettings::default(), &GamepadConfig::default())
    }
}

impl ArcadeSettingsModal {
    pub fn new(audio: &AudioSettings, gamepad: &GamepadConfig) -> Self {
        let tabs = vec![
            "AUDIO".to_string(),
            "CONTROLS".to_string(),
            "DISPLAY".to_string(),
            "GAMEPLAY".to_string(),
        ];
        let tab_bar = TabBar::new(tabs);

        // Grid navigation: 4 columns for the 4 tabs, each with widget count + 1 (for bottom buttons)
        // Tab 0 (Audio): 5 widgets + 1 bottom row = 6 rows
        // Tab 1 (Controls): 4 widgets + 1 bottom row = 5 rows
        // Tab 2 (Display): 3 widgets + 1 bottom row = 4 rows
        // Tab 3 (Gameplay): 3 widgets + 1 bottom row = 4 rows
        let nav = NavGrid2D::new(vec![6, 5, 4, 4]);

        let mute_options = vec!["ACTIVE (UNMUTED)".to_string(), "MUTED".to_string()];
        let mute_idx = if audio.is_muted { 1 } else { 0 };

        let theme_options = vec![
            "Cyberpunk Neon".to_string(),
            "Solar Flare".to_string(),
            "Monokai Dark".to_string(),
            "Synthwave 84".to_string(),
            "High Contrast".to_string(),
        ];

        let scanline_options = vec![
            "Disabled".to_string(),
            "Subtle (25%)".to_string(),
            "Arcade CRT (50%)".to_string(),
            "Retro Glow (75%)".to_string(),
        ];

        let scale_options = vec![
            "Auto (Aspect-Fit)".to_string(),
            "Compact (85%)".to_string(),
            "Standard (100%)".to_string(),
            "Large (120%)".to_string(),
        ];

        let assist_options = vec![
            "Arcade (High Stability)".to_string(),
            "Sport (Balanced Drift)".to_string(),
            "Pro (Raw Physics)".to_string(),
        ];

        let speed_options = vec!["KM/H (Metric)".to_string(), "MPH (Imperial)".to_string()];
        let ghost_options = vec!["Enabled (Best Lap)".to_string(), "Disabled".to_string()];

        Self {
            tab_bar,
            nav,
            master_slider: SliderWidget::percentage("MASTER VOLUME", audio.master_volume),
            music_slider: SliderWidget::percentage("MUSIC VOLUME", audio.music_volume),
            sfx_slider: SliderWidget::percentage("SFX VOLUME", audio.sfx_volume),
            ui_slider: SliderWidget::percentage("UI SOUNDS VOLUME", audio.ui_volume),
            mute_dropdown: DropdownWidget::new("AUDIO OUTPUT", mute_options, mute_idx),

            stick_deadzone_slider: SliderWidget::new("STICK DEADZONE", 0.0, 0.40, 0.02, gamepad.stick_deadzone),
            trigger_deadzone_slider: SliderWidget::new("TRIGGER DEADZONE", 0.0, 0.30, 0.01, gamepad.trigger_deadzone),
            steer_sensitivity_slider: SliderWidget::new("STEER SENSITIVITY", 0.50, 2.00, 0.05, gamepad.steer_scale),
            steer_exponent_slider: SliderWidget::new("STEER EXPONENT", 1.00, 1.50, 0.05, gamepad.steer_exponent),

            theme_dropdown: DropdownWidget::new("COLOR THEME", theme_options, 0),
            scanlines_dropdown: DropdownWidget::new("CRT FILTER", scanline_options, 0),
            ui_scale_dropdown: DropdownWidget::new("UI SCALING", scale_options, 0),

            assist_dropdown: DropdownWidget::new("ASSIST PROFILE", assist_options, 0),
            speed_unit_dropdown: DropdownWidget::new("SPEEDOMETER UNIT", speed_options, 0),
            ghost_car_dropdown: DropdownWidget::new("GHOST REPLAY", ghost_options, 0),

            is_saved: false,
        }
    }

    /// Resets all values back to initial factory defaults.
    pub fn restore_defaults(&mut self) {
        let def_audio = AudioSettings::default();
        let def_gp = GamepadConfig::default();

        self.master_slider.set_normalized(def_audio.master_volume);
        self.music_slider.set_normalized(def_audio.music_volume);
        self.sfx_slider.set_normalized(def_audio.sfx_volume);
        self.ui_slider.set_normalized(def_audio.ui_volume);
        self.mute_dropdown.set_selected(if def_audio.is_muted { 1 } else { 0 });

        self.stick_deadzone_slider.set_value(def_gp.stick_deadzone);
        self.trigger_deadzone_slider.set_value(def_gp.trigger_deadzone);
        self.steer_sensitivity_slider.set_value(def_gp.steer_scale);
        self.steer_exponent_slider.set_value(def_gp.steer_exponent);

        self.theme_dropdown.set_selected(0);
        self.scanlines_dropdown.set_selected(0);
        self.ui_scale_dropdown.set_selected(0);
        self.assist_dropdown.set_selected(0);
        self.speed_unit_dropdown.set_selected(0);
        self.ghost_car_dropdown.set_selected(0);
    }

    /// Applies configured values to an external `AudioSettings` struct.
    pub fn apply_to_audio(&self, audio: &mut AudioSettings) {
        audio.master_volume = self.master_slider.normalized();
        audio.music_volume = self.music_slider.normalized();
        audio.sfx_volume = self.sfx_slider.normalized();
        audio.ui_volume = self.ui_slider.normalized();
        audio.is_muted = self.mute_dropdown.selected_index == 1;
    }

    /// Applies configured values to an external `GamepadConfig` struct.
    pub fn apply_to_gamepad(&self, gp: &mut GamepadConfig) {
        gp.stick_deadzone = self.stick_deadzone_slider.value;
        gp.trigger_deadzone = self.trigger_deadzone_slider.value;
        gp.steer_scale = self.steer_sensitivity_slider.value;
        gp.steer_exponent = self.steer_exponent_slider.value;
    }
}

impl CabinetScreen for ArcadeSettingsModal {
    fn name(&self) -> &str {
        "ArcadeSettingsModal"
    }

    fn is_transparent(&self) -> bool {
        true
    }

    fn update(&mut self, ctx: &mut CabinetContext) -> ScreenAction {
        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let scaler = ctx.scaler;

        // Cancel / Back closes modal without saving
        if self.nav.is_cancelled(ctx.gamepad.btn_cancel_pressed || ctx.gamepad.btn_b_pressed || ctx.gamepad.btn_back_pressed) {
            return ScreenAction::Pop;
        }

        // Dialog box bounds
        let box_w = (sw * 0.72).clamp(scaler.s(560.0), scaler.s(820.0));
        let box_h = (sh * 0.80).clamp(scaler.s(440.0), scaler.s(600.0));
        let box_x = (sw - box_w) * 0.5;
        let box_y = (sh - box_h) * 0.5;

        // Tab bar rect
        let tab_bar_x = box_x + scaler.s(20.0);
        let tab_bar_y = box_y + scaler.s(52.0);
        let tab_bar_w = box_w - scaler.s(40.0);
        let tab_bar_h = scaler.s(36.0);
        let tab_bar_rect = (tab_bar_x, tab_bar_y, tab_bar_w, tab_bar_h);

        // Tab Bar navigation (Q / E or Gamepad Bumper / Mouse)
        let tab_changed = self.tab_bar.handle_input(false, false, tab_bar_rect);
        if tab_changed {
            self.nav.set_focus(self.tab_bar.active_tab, 0);
        }

        // Check if any dropdown is currently open. If so, let it capture vertical input
        let is_any_dropdown_open = self.mute_dropdown.is_open
            || self.theme_dropdown.is_open
            || self.scanlines_dropdown.is_open
            || self.ui_scale_dropdown.is_open
            || self.assist_dropdown.is_open
            || self.speed_unit_dropdown.is_open
            || self.ghost_car_dropdown.is_open;

        if !is_any_dropdown_open {
            self.nav.handle_standard_inputs(
                ctx.gamepad.nav_left,
                ctx.gamepad.nav_right,
                ctx.gamepad.nav_up,
                ctx.gamepad.nav_down,
            );
        }

        let active_tab = self.tab_bar.active_tab;
        self.nav.focused_col = active_tab;
        let active_row = self.nav.active_row();

        // Content items area
        let content_x = box_x + scaler.s(24.0);
        let content_y = box_y + scaler.s(100.0);
        let content_w = box_w - scaler.s(48.0);
        let row_h = scaler.s(44.0);
        let row_gap = scaler.s(8.0);

        match active_tab {
            0 => {
                // AUDIO: 0: Master, 1: Music, 2: SFX, 3: UI, 4: Mute, 5: Bottom Buttons
                let r0 = (content_x, content_y, content_w, row_h);
                let r1 = (content_x, content_y + (row_h + row_gap), content_w, row_h);
                let r2 = (content_x, content_y + (row_h + row_gap) * 2.0, content_w, row_h);
                let r3 = (content_x, content_y + (row_h + row_gap) * 3.0, content_w, row_h);
                let r4 = (content_x, content_y + (row_h + row_gap) * 4.0, content_w, row_h);

                self.master_slider.handle_input(active_row == 0, ctx.gamepad.nav_left, ctx.gamepad.nav_right, r0);
                self.music_slider.handle_input(active_row == 1, ctx.gamepad.nav_left, ctx.gamepad.nav_right, r1);
                self.sfx_slider.handle_input(active_row == 2, ctx.gamepad.nav_left, ctx.gamepad.nav_right, r2);
                self.ui_slider.handle_input(active_row == 3, ctx.gamepad.nav_left, ctx.gamepad.nav_right, r3);
                self.mute_dropdown.handle_input(
                    active_row == 4,
                    ctx.gamepad.nav_left,
                    ctx.gamepad.nav_right,
                    ctx.gamepad.nav_up,
                    ctx.gamepad.nav_down,
                    ctx.gamepad.btn_confirm_pressed,
                    ctx.gamepad.btn_cancel_pressed,
                    r4,
                    scaler,
                );
            }
            1 => {
                // CONTROLS: 0: Stick Deadzone, 1: Trigger Deadzone, 2: Steer Sensitivity, 3: Steer Exponent, 4: Bottom Buttons
                let r0 = (content_x, content_y, content_w, row_h);
                let r1 = (content_x, content_y + (row_h + row_gap), content_w, row_h);
                let r2 = (content_x, content_y + (row_h + row_gap) * 2.0, content_w, row_h);
                let r3 = (content_x, content_y + (row_h + row_gap) * 3.0, content_w, row_h);

                self.stick_deadzone_slider.handle_input(active_row == 0, ctx.gamepad.nav_left, ctx.gamepad.nav_right, r0);
                self.trigger_deadzone_slider.handle_input(active_row == 1, ctx.gamepad.nav_left, ctx.gamepad.nav_right, r1);
                self.steer_sensitivity_slider.handle_input(active_row == 2, ctx.gamepad.nav_left, ctx.gamepad.nav_right, r2);
                self.steer_exponent_slider.handle_input(active_row == 3, ctx.gamepad.nav_left, ctx.gamepad.nav_right, r3);
            }
            2 => {
                // DISPLAY: 0: Theme, 1: Scanlines, 2: UI Scale, 3: Bottom Buttons
                let r0 = (content_x, content_y, content_w, row_h);
                let r1 = (content_x, content_y + (row_h + row_gap), content_w, row_h);
                let r2 = (content_x, content_y + (row_h + row_gap) * 2.0, content_w, row_h);

                self.theme_dropdown.handle_input(
                    active_row == 0,
                    ctx.gamepad.nav_left,
                    ctx.gamepad.nav_right,
                    ctx.gamepad.nav_up,
                    ctx.gamepad.nav_down,
                    ctx.gamepad.btn_confirm_pressed,
                    ctx.gamepad.btn_cancel_pressed,
                    r0,
                    scaler,
                );
                self.scanlines_dropdown.handle_input(
                    active_row == 1,
                    ctx.gamepad.nav_left,
                    ctx.gamepad.nav_right,
                    ctx.gamepad.nav_up,
                    ctx.gamepad.nav_down,
                    ctx.gamepad.btn_confirm_pressed,
                    ctx.gamepad.btn_cancel_pressed,
                    r1,
                    scaler,
                );
                self.ui_scale_dropdown.handle_input(
                    active_row == 2,
                    ctx.gamepad.nav_left,
                    ctx.gamepad.nav_right,
                    ctx.gamepad.nav_up,
                    ctx.gamepad.nav_down,
                    ctx.gamepad.btn_confirm_pressed,
                    ctx.gamepad.btn_cancel_pressed,
                    r2,
                    scaler,
                );
            }
            _ => {
                // GAMEPLAY: 0: Assist, 1: Speed Units, 2: Ghost Car, 3: Bottom Buttons
                let r0 = (content_x, content_y, content_w, row_h);
                let r1 = (content_x, content_y + (row_h + row_gap), content_w, row_h);
                let r2 = (content_x, content_y + (row_h + row_gap) * 2.0, content_w, row_h);

                self.assist_dropdown.handle_input(
                    active_row == 0,
                    ctx.gamepad.nav_left,
                    ctx.gamepad.nav_right,
                    ctx.gamepad.nav_up,
                    ctx.gamepad.nav_down,
                    ctx.gamepad.btn_confirm_pressed,
                    ctx.gamepad.btn_cancel_pressed,
                    r0,
                    scaler,
                );
                self.speed_unit_dropdown.handle_input(
                    active_row == 1,
                    ctx.gamepad.nav_left,
                    ctx.gamepad.nav_right,
                    ctx.gamepad.nav_up,
                    ctx.gamepad.nav_down,
                    ctx.gamepad.btn_confirm_pressed,
                    ctx.gamepad.btn_cancel_pressed,
                    r1,
                    scaler,
                );
                self.ghost_car_dropdown.handle_input(
                    active_row == 2,
                    ctx.gamepad.nav_left,
                    ctx.gamepad.nav_right,
                    ctx.gamepad.nav_up,
                    ctx.gamepad.nav_down,
                    ctx.gamepad.btn_confirm_pressed,
                    ctx.gamepad.btn_cancel_pressed,
                    r2,
                    scaler,
                );
            }
        }

        // Bottom action buttons layout
        let bottom_btn_y = box_y + box_h - scaler.s(52.0);
        let bottom_btn_h = scaler.s(40.0);
        let btn_gap = scaler.s(16.0);
        let single_btn_w = (box_w - scaler.s(48.0) - btn_gap) * 0.5;

        let reset_rect = (box_x + scaler.s(24.0), bottom_btn_y, single_btn_w, bottom_btn_h);
        let save_rect = (box_x + scaler.s(24.0) + single_btn_w + btn_gap, bottom_btn_y, single_btn_w, bottom_btn_h);

        let is_last_row = active_row == self.nav.column_lengths.get(active_tab).copied().unwrap_or(1) - 1;

        if NavGrid2D::check_mouse_click(reset_rect) {
            self.restore_defaults();
        }
        if NavGrid2D::check_mouse_click(save_rect) || (is_last_row && self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed)) {
            self.is_saved = true;
            return ScreenAction::Pop;
        }

        ScreenAction::None
    }

    fn draw(&self, ctx: &CabinetContext) {
        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let scaler = ctx.scaler;
        let fonts = ctx.fonts;
        let accent = ctx.theme.accent_primary;

        // Semi-transparent backdrop dimming
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.75));

        // Dialog box
        let box_w = (sw * 0.72).clamp(scaler.s(560.0), scaler.s(820.0));
        let box_h = (sh * 0.80).clamp(scaler.s(440.0), scaler.s(600.0));
        let box_x = (sw - box_w) * 0.5;
        let box_y = (sh - box_h) * 0.5;

        scaler.draw_glass_card(box_x, box_y, box_w, box_h, Palette::UI_CARD_BG, accent, 2.2);

        // Header Title
        fonts.draw_display_centered_with_shadow(
            "ARCADE SETTINGS & PREFERENCES",
            sw * 0.5,
            box_y + scaler.s(32.0),
            scaler.font_s(22.0),
            Palette::WHITE,
            Color::new(0.0, 0.0, 0.0, 0.6),
            scaler.s(2.0),
        );

        // Tab Bar
        let tab_bar_x = box_x + scaler.s(20.0);
        let tab_bar_y = box_y + scaler.s(50.0);
        let tab_bar_w = box_w - scaler.s(40.0);
        let tab_bar_h = scaler.s(36.0);

        draw_tab_bar(
            scaler,
            fonts,
            tab_bar_x,
            tab_bar_y,
            tab_bar_w,
            tab_bar_h,
            &self.tab_bar.tabs,
            self.tab_bar.active_tab,
            None,
            accent,
        );

        // Active tab content area
        let content_x = box_x + scaler.s(24.0);
        let content_y = box_y + scaler.s(96.0);
        let content_w = box_w - scaler.s(48.0);
        let row_h = scaler.s(42.0);
        let row_gap = scaler.s(8.0);

        let active_tab = self.tab_bar.active_tab;
        let active_row = self.nav.active_row();

        match active_tab {
            0 => {
                // AUDIO TAB
                let mut y = content_y;
                draw_slider(scaler, fonts, content_x, y, content_w, row_h, &self.master_slider.label, &self.master_slider.formatted_value(), self.master_slider.normalized(), active_row == 0, false, accent);
                y += row_h + row_gap;
                draw_slider(scaler, fonts, content_x, y, content_w, row_h, &self.music_slider.label, &self.music_slider.formatted_value(), self.music_slider.normalized(), active_row == 1, false, accent);
                y += row_h + row_gap;
                draw_slider(scaler, fonts, content_x, y, content_w, row_h, &self.sfx_slider.label, &self.sfx_slider.formatted_value(), self.sfx_slider.normalized(), active_row == 2, false, accent);
                y += row_h + row_gap;
                draw_slider(scaler, fonts, content_x, y, content_w, row_h, &self.ui_slider.label, &self.ui_slider.formatted_value(), self.ui_slider.normalized(), active_row == 3, false, accent);
                y += row_h + row_gap;
                draw_dropdown(scaler, fonts, content_x, y, content_w, row_h, &self.mute_dropdown.label, &self.mute_dropdown.options, self.mute_dropdown.selected_index, self.mute_dropdown.is_open, self.mute_dropdown.popup_hovered_index, active_row == 4, false, accent);
            }
            1 => {
                // CONTROLS TAB
                let mut y = content_y;
                draw_slider(scaler, fonts, content_x, y, content_w, row_h, &self.stick_deadzone_slider.label, &self.stick_deadzone_slider.formatted_value(), self.stick_deadzone_slider.normalized(), active_row == 0, false, accent);
                y += row_h + row_gap;
                draw_slider(scaler, fonts, content_x, y, content_w, row_h, &self.trigger_deadzone_slider.label, &self.trigger_deadzone_slider.formatted_value(), self.trigger_deadzone_slider.normalized(), active_row == 1, false, accent);
                y += row_h + row_gap;
                draw_slider(scaler, fonts, content_x, y, content_w, row_h, &self.steer_sensitivity_slider.label, &self.steer_sensitivity_slider.formatted_value(), self.steer_sensitivity_slider.normalized(), active_row == 2, false, accent);
                y += row_h + row_gap;
                draw_slider(scaler, fonts, content_x, y, content_w, row_h, &self.steer_exponent_slider.label, &self.steer_exponent_slider.formatted_value(), self.steer_exponent_slider.normalized(), active_row == 3, false, accent);
            }
            2 => {
                // DISPLAY TAB
                let mut y = content_y;
                draw_dropdown(scaler, fonts, content_x, y, content_w, row_h, &self.theme_dropdown.label, &self.theme_dropdown.options, self.theme_dropdown.selected_index, self.theme_dropdown.is_open, self.theme_dropdown.popup_hovered_index, active_row == 0, false, accent);
                y += row_h + row_gap;
                draw_dropdown(scaler, fonts, content_x, y, content_w, row_h, &self.scanlines_dropdown.label, &self.scanlines_dropdown.options, self.scanlines_dropdown.selected_index, self.scanlines_dropdown.is_open, self.scanlines_dropdown.popup_hovered_index, active_row == 1, false, accent);
                y += row_h + row_gap;
                draw_dropdown(scaler, fonts, content_x, y, content_w, row_h, &self.ui_scale_dropdown.label, &self.ui_scale_dropdown.options, self.ui_scale_dropdown.selected_index, self.ui_scale_dropdown.is_open, self.ui_scale_dropdown.popup_hovered_index, active_row == 2, false, accent);
            }
            _ => {
                // GAMEPLAY TAB
                let mut y = content_y;
                draw_dropdown(scaler, fonts, content_x, y, content_w, row_h, &self.assist_dropdown.label, &self.assist_dropdown.options, self.assist_dropdown.selected_index, self.assist_dropdown.is_open, self.assist_dropdown.popup_hovered_index, active_row == 0, false, accent);
                y += row_h + row_gap;
                draw_dropdown(scaler, fonts, content_x, y, content_w, row_h, &self.speed_unit_dropdown.label, &self.speed_unit_dropdown.options, self.speed_unit_dropdown.selected_index, self.speed_unit_dropdown.is_open, self.speed_unit_dropdown.popup_hovered_index, active_row == 1, false, accent);
                y += row_h + row_gap;
                draw_dropdown(scaler, fonts, content_x, y, content_w, row_h, &self.ghost_car_dropdown.label, &self.ghost_car_dropdown.options, self.ghost_car_dropdown.selected_index, self.ghost_car_dropdown.is_open, self.ghost_car_dropdown.popup_hovered_index, active_row == 2, false, accent);
            }
        }

        // Bottom action buttons
        let bottom_btn_y = box_y + box_h - scaler.s(52.0);
        let bottom_btn_h = scaler.s(38.0);
        let btn_gap = scaler.s(16.0);
        let single_btn_w = (box_w - scaler.s(48.0) - btn_gap) * 0.5;

        let reset_rect = (box_x + scaler.s(24.0), bottom_btn_y, single_btn_w, bottom_btn_h);
        let save_rect = (box_x + scaler.s(24.0) + single_btn_w + btn_gap, bottom_btn_y, single_btn_w, bottom_btn_h);

        let is_last_row = active_row == self.nav.column_lengths.get(active_tab).copied().unwrap_or(1) - 1;

        // Reset Button
        let is_reset_hovered = NavGrid2D::check_mouse_hover(reset_rect);
        draw_rectangle(
            reset_rect.0,
            reset_rect.1,
            reset_rect.2,
            reset_rect.3,
            if is_reset_hovered { Color::new(0.35, 0.12, 0.14, 0.95) } else { Color::new(0.20, 0.08, 0.10, 0.85) },
        );
        draw_rectangle_lines(
            reset_rect.0,
            reset_rect.1,
            reset_rect.2,
            reset_rect.3,
            if is_reset_hovered { 2.0 * scaler.scale } else { 1.0 * scaler.scale },
            Palette::RED,
        );
        fonts.draw_ui_bold_centered(
            "RESTORE DEFAULTS",
            reset_rect.0 + reset_rect.2 * 0.5,
            reset_rect.1 + reset_rect.3 * 0.65,
            scaler.font_s(13.0),
            Palette::WHITE,
        );

        // Save & Close Button
        let is_save_focused = is_last_row;
        let is_save_hovered = NavGrid2D::check_mouse_hover(save_rect);
        let is_save_active = is_save_focused || is_save_hovered;

        draw_rectangle(
            save_rect.0,
            save_rect.1,
            save_rect.2,
            save_rect.3,
            if is_save_active { Color::new(0.12, 0.50, 0.28, 0.95) } else { Color::new(0.08, 0.32, 0.18, 0.85) },
        );
        draw_rectangle_lines(
            save_rect.0,
            save_rect.1,
            save_rect.2,
            save_rect.3,
            if is_save_active { 2.4 * scaler.scale } else { 1.2 * scaler.scale },
            if is_save_active { Palette::NEON_GREEN } else { Color::new(0.20, 0.70, 0.35, 0.85) },
        );
        fonts.draw_ui_bold_centered(
            if is_save_active { "[ENTER] SAVE & CLOSE" } else { "SAVE & CLOSE" },
            save_rect.0 + save_rect.2 * 0.5,
            save_rect.1 + save_rect.3 * 0.65,
            scaler.font_s(13.5),
            Palette::WHITE,
        );
    }
}
