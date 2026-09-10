use macroquad::color::Color;
use macroquad::shapes::draw_rectangle;
use crate::input::NavGrid2D;
use crate::state::stack::{CabinetContext, CabinetScreen, ScreenAction};
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Layout rectangle container for confirmation modal buttons.
#[derive(Debug, Clone, Copy)]
pub struct ConfirmButtonLayout {
    pub cancel_rect: (f32, f32, f32, f32),
    pub confirm_rect: (f32, f32, f32, f32),
}

/// Computes responsive centered dialog box and button rectangles.
pub fn confirm_modal_layout(sw: f32, sh: f32) -> (f32, f32, f32, f32, ConfirmButtonLayout) {
    let scaler = UiScaler::new(sw, sh);
    let box_w = (sw * 0.42).clamp(scaler.s(420.0), scaler.s(580.0));
    let box_h = scaler.s(240.0);
    let box_x = (sw - box_w) * 0.5;
    let box_y = (sh - box_h) * 0.5;

    let btn_w = (box_w - scaler.s(60.0)) * 0.5;
    let btn_h = scaler.s(48.0);
    let btn_y = box_y + box_h - btn_h - scaler.s(28.0);

    let cancel_x = box_x + scaler.s(22.0);
    let confirm_x = box_x + box_w - btn_w - scaler.s(22.0);

    let layout = ConfirmButtonLayout {
        cancel_rect: (cancel_x, btn_y, btn_w, btn_h),
        confirm_rect: (confirm_x, btn_y, btn_w, btn_h),
    };

    (box_x, box_y, box_w, box_h, layout)
}

/// Generic universal confirmation & prompt modal dialog.
///
/// Features:
/// - Safety-first default focus on CANCEL to prevent accidental activation.
/// - Configurable title, explanatory prompt message, and custom button labels.
/// - Custom accent color (e.g. `Palette::NEON_RED` for destructive actions, `Palette::NEON_GOLD` for warnings).
/// - Full 2D orthogonal navigation with keyboard (`A`/`D`, `Left`/`Right`), Gamepad (`D-pad`), and mouse hit testing.
/// - Executes `on_confirm_action` (or returns `ScreenAction::Pop` / `Quit`).
pub struct UniversalConfirmModal {
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub cancel_label: String,
    pub accent_color: Color,
    pub nav: NavGrid2D, // Column 0: Cancel, Column 1: Confirm
    pub on_confirm_action: Option<ScreenAction>,
    pub result: Option<bool>,
}

impl UniversalConfirmModal {
    /// Creates a confirmation modal with default labels and danger/red accent.
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_label: "CONFIRM".to_string(),
            cancel_label: "CANCEL".to_string(),
            accent_color: Palette::NEON_RED,
            nav: NavGrid2D::new(vec![1, 1]), // 2 buttons: [0] Cancel, [1] Confirm
            on_confirm_action: Some(ScreenAction::Pop),
            result: None,
        }
    }

    /// Creates a quit-to-desktop confirmation modal.
    pub fn quit_game() -> Self {
        let mut modal = Self::new("QUIT GAME", "Are you sure you want to exit to desktop?\nUnsaved session progress will be lost.");
        modal.confirm_label = "QUIT TO DESKTOP".to_string();
        modal.cancel_label = "KEEP PLAYING".to_string();
        modal.on_confirm_action = Some(ScreenAction::Quit);
        modal
    }

    /// Sets custom button labels.
    pub fn with_labels(mut self, confirm: impl Into<String>, cancel: impl Into<String>) -> Self {
        self.confirm_label = confirm.into();
        self.cancel_label = cancel.into();
        self
    }

    /// Sets the accent color of the modal card border and confirm button.
    pub fn with_accent(mut self, accent: Color) -> Self {
        self.accent_color = accent;
        self
    }

    /// Sets the `ScreenAction` executed when the user confirms.
    pub fn with_action(mut self, action: ScreenAction) -> Self {
        self.on_confirm_action = Some(action);
        self
    }

    /// Sets initial focus to the confirm button (use with care).
    pub fn focus_confirm(mut self) -> Self {
        self.nav.set_focus(1, 0);
        self
    }
}

impl CabinetScreen for UniversalConfirmModal {
    fn name(&self) -> &str {
        "UniversalConfirmModal"
    }

    fn is_transparent(&self) -> bool {
        true
    }

    fn update(&mut self, ctx: &mut CabinetContext) -> ScreenAction {
        self.nav.handle_standard_inputs(
            ctx.gamepad.nav_left,
            ctx.gamepad.nav_right,
            ctx.gamepad.nav_up,
            ctx.gamepad.nav_down,
        );

        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let (_, _, _, _, btn_layout) = confirm_modal_layout(sw, sh);

        if NavGrid2D::check_mouse_hover(btn_layout.cancel_rect) {
            self.nav.set_focus(0, 0);
        }
        if NavGrid2D::check_mouse_hover(btn_layout.confirm_rect) {
            self.nav.set_focus(1, 0);
        }

        let cancel_clicked = NavGrid2D::check_mouse_click(btn_layout.cancel_rect);
        let confirm_clicked = NavGrid2D::check_mouse_click(btn_layout.confirm_rect);

        // Escape / B button cancels immediately
        if cancel_clicked || self.nav.is_cancelled(ctx.gamepad.btn_cancel_pressed || ctx.gamepad.btn_b_pressed) {
            self.result = Some(false);
            return ScreenAction::Pop;
        }

        // Mouse click on confirm
        if confirm_clicked {
            self.result = Some(true);
            return self.on_confirm_action.take().unwrap_or(ScreenAction::Pop);
        }

        // Enter / Gamepad A on focused button
        if self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed) {
            if self.nav.focused_col == 0 {
                self.result = Some(false);
                return ScreenAction::Pop;
            } else {
                self.result = Some(true);
                return self.on_confirm_action.take().unwrap_or(ScreenAction::Pop);
            }
        }

        ScreenAction::None
    }

    fn draw(&self, ctx: &CabinetContext) {
        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let scaler = ctx.scaler;
        let fonts = ctx.fonts;

        // Dark dim backdrop
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.75));

        let (box_x, box_y, box_w, box_h, btn_layout) = confirm_modal_layout(sw, sh);

        // Glassmorphism modal card
        scaler.draw_glass_card(box_x, box_y, box_w, box_h, Palette::UI_CARD_BG, self.accent_color, 2.2);

        // Title
        fonts.draw_display_centered_with_shadow(
            &self.title,
            sw * 0.5,
            box_y + scaler.s(38.0),
            scaler.font_s(26.0),
            Palette::WHITE,
            Color::new(0.0, 0.0, 0.0, 0.6),
            scaler.s(2.0),
        );

        // Message body (split lines if newline present)
        let lines: Vec<&str> = self.message.split('\n').collect();
        let start_y = box_y + scaler.s(76.0);
        let line_spacing = scaler.s(20.0);
        for (i, line) in lines.iter().enumerate() {
            fonts.draw_ui_regular_centered(
                line,
                sw * 0.5,
                start_y + (i as f32 * line_spacing),
                scaler.font_s(13.5),
                Palette::UI_TEXT_MUTED,
            );
        }

        // Buttons
        let cancel_focused = self.nav.focused_col == 0;
        let cancel_hovered = NavGrid2D::check_mouse_hover(btn_layout.cancel_rect);
        let (cx, cy, cw, ch) = btn_layout.cancel_rect;
        scaler.draw_button_card(cx, cy, cw, ch, cancel_focused, cancel_hovered, Palette::NEON_CYAN);
        fonts.draw_ui_bold_centered(
            &self.cancel_label,
            cx + cw * 0.5,
            cy + ch * 0.60,
            scaler.font_s(14.0),
            Palette::WHITE,
        );

        let confirm_focused = self.nav.focused_col == 1;
        let confirm_hovered = NavGrid2D::check_mouse_hover(btn_layout.confirm_rect);
        let (fx, fy, fw, fh) = btn_layout.confirm_rect;
        scaler.draw_button_card(fx, fy, fw, fh, confirm_focused, confirm_hovered, self.accent_color);
        fonts.draw_ui_bold_centered(
            &self.confirm_label,
            fx + fw * 0.5,
            fy + fh * 0.60,
            scaler.font_s(14.0),
            Palette::WHITE,
        );
    }
}
