use macroquad::color::Color;
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use crate::input::NavGrid2D;
use crate::state::stack::{CabinetContext, CabinetScreen, ScreenAction};
use crate::ui::theme::Palette;

/// Generic universal Pause Modal overlay with Resume and Exit actions.
pub struct UniversalPauseModal {
    pub nav: NavGrid2D, // 2 buttons on X-axis (0: Resume, 1: Exit)
    pub subtitle: String,
}

impl Default for UniversalPauseModal {
    fn default() -> Self {
        Self::new("GAME PAUSED")
    }
}

impl UniversalPauseModal {
    pub fn new(subtitle: &str) -> Self {
        Self {
            nav: NavGrid2D::new(vec![1, 1]), // 2 columns (Resume, Exit), 1 item each
            subtitle: subtitle.to_string(),
        }
    }
}

impl CabinetScreen for UniversalPauseModal {
    fn name(&self) -> &str {
        "UniversalPauseModal"
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
        let (_, _, _, _, btn_layout) = pause_modal_layout(sw, sh);

        if NavGrid2D::check_mouse_hover(btn_layout.resume_rect) {
            self.nav.set_focus(0, 0);
        }
        if NavGrid2D::check_mouse_hover(btn_layout.exit_rect) {
            self.nav.set_focus(1, 0);
        }

        let resume_clicked = NavGrid2D::check_mouse_click(btn_layout.resume_rect);
        let exit_clicked = NavGrid2D::check_mouse_click(btn_layout.exit_rect);

        if self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed) {
            if self.nav.focused_col == 0 {
                return ScreenAction::Pop;
            } else {
                return ScreenAction::Quit;
            }
        }

        if resume_clicked || ctx.gamepad.btn_start_pressed {
            return ScreenAction::Pop;
        }
        if exit_clicked {
            return ScreenAction::Quit;
        }

        if self.nav.is_cancelled(ctx.gamepad.btn_cancel_pressed) {
            return ScreenAction::Pop;
        }

        ScreenAction::None
    }

    fn draw(&self, ctx: &CabinetContext) {
        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let scaler = ctx.scaler;
        let fonts = ctx.fonts;

        // Dark dim backdrop
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.70));

        let (box_x, box_y, box_w, box_h, btn_layout) = pause_modal_layout(sw, sh);
        scaler.draw_glass_card(box_x, box_y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.2);

        fonts.draw_display_centered_with_shadow(
            &self.subtitle,
            sw * 0.5,
            box_y + scaler.s(38.0),
            scaler.font_s(30.0),
            Palette::WHITE,
            Color::new(0.0, 0.0, 0.0, 0.6),
            scaler.s(2.0),
        );

        let selected_btn = self.nav.focused_col;

        // 1. Resume Button
        let (rx, ry, rw, rh) = btn_layout.resume_rect;
        let is_resume_active = selected_btn == 0;
        let resume_bg = if is_resume_active {
            Color::new(0.14, 0.58, 0.32, 0.98)
        } else {
            Color::new(0.08, 0.36, 0.20, 0.85)
        };
        draw_rectangle(rx, ry, rw, rh, resume_bg);
        draw_rectangle_lines(
            rx,
            ry,
            rw,
            rh,
            if is_resume_active { 2.6 * scaler.scale } else { 1.4 * scaler.scale },
            if is_resume_active { Palette::NEON_GREEN } else { Color::new(0.20, 0.75, 0.40, 0.85) },
        );
        fonts.draw_ui_bold_centered(
            if is_resume_active { "[ENTER] RESUME" } else { "RESUME" },
            rx + rw * 0.5,
            ry + scaler.s(21.0),
            scaler.font_s(14.5),
            Palette::WHITE,
        );
        fonts.draw_ui_regular_centered(
            "Continue | Space / Gamepad A",
            rx + rw * 0.5,
            ry + scaler.s(37.0),
            scaler.font_s(11.0),
            Color::new(0.75, 0.95, 0.80, 0.90),
        );

        // 2. Exit Button
        let (ex, ey, ew, eh) = btn_layout.exit_rect;
        let is_exit_active = selected_btn == 1;
        let exit_bg = if is_exit_active {
            Color::new(0.60, 0.16, 0.16, 0.98)
        } else {
            Color::new(0.35, 0.10, 0.10, 0.85)
        };
        draw_rectangle(ex, ey, ew, eh, exit_bg);
        draw_rectangle_lines(
            ex,
            ey,
            ew,
            eh,
            if is_exit_active { 2.6 * scaler.scale } else { 1.4 * scaler.scale },
            if is_exit_active { Palette::RED } else { Color::new(0.60, 0.25, 0.25, 0.85) },
        );
        fonts.draw_ui_bold_centered(
            if is_exit_active { "[ENTER] EXIT TO MENU" } else { "EXIT TO MENU" },
            ex + ew * 0.5,
            ey + scaler.s(21.0),
            scaler.font_s(14.5),
            Palette::WHITE,
        );
        fonts.draw_ui_regular_centered(
            "Return to Main Menu",
            ex + ew * 0.5,
            ey + scaler.s(37.0),
            scaler.font_s(11.0),
            Color::new(0.95, 0.80, 0.80, 0.90),
        );
    }
}

pub struct PauseButtonLayout {
    pub resume_rect: (f32, f32, f32, f32),
    pub exit_rect: (f32, f32, f32, f32),
}

pub fn pause_modal_layout(sw: f32, sh: f32) -> (f32, f32, f32, f32, PauseButtonLayout) {
    let scaler = crate::ui::UiScaler::new(sw, sh);
    let box_w = (sw * 0.55).clamp(scaler.s(440.0), scaler.s(560.0));
    let box_h = scaler.s(130.0);
    let box_x = (sw - box_w) * 0.5;
    let box_y = (sh - box_h) * 0.5;

    let pad_x = scaler.s(24.0);
    let gap = scaler.s(16.0);
    let btn_w = (box_w - pad_x * 2.0 - gap) * 0.5;
    let btn_h = scaler.s(48.0);
    let btn_y = box_y + scaler.s(58.0);

    let resume_rect = (box_x + pad_x, btn_y, btn_w, btn_h);
    let exit_rect = (box_x + pad_x + btn_w + gap, btn_y, btn_w, btn_h);

    (
        box_x,
        box_y,
        box_w,
        box_h,
        PauseButtonLayout {
            resume_rect,
            exit_rect,
        },
    )
}
