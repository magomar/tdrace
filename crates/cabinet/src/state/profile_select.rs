use macroquad::color::Color;
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use crate::input::NavGrid2D;
use crate::profile::country::{draw_country_banner, CountryRegistry};
use crate::profile::data::ColorScheme;
use crate::profile::manager::ProfileManager;
use crate::state::stack::{CabinetContext, CabinetScreen, ScreenAction};
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Layout for profile selection modal.
#[derive(Debug, Clone, Copy)]
pub struct ProfileSelectLayout {
    pub box_x: f32,
    pub box_y: f32,
    pub box_w: f32,
    pub box_h: f32,
    pub slots_rect: (f32, f32, f32, f32),
    pub dossier_rect: (f32, f32, f32, f32),
    pub slot_h: f32,
    pub btn_select_rect: (f32, f32, f32, f32),
    pub btn_country_rect: (f32, f32, f32, f32),
    pub btn_livery_rect: (f32, f32, f32, f32),
    pub btn_close_rect: (f32, f32, f32, f32),
}

/// Computes responsive layout for the Profile Select modal.
pub fn profile_select_layout(sw: f32, sh: f32) -> ProfileSelectLayout {
    let scaler = UiScaler::new(sw, sh);
    let box_w = (sw * 0.74).clamp(scaler.s(680.0), scaler.s(880.0));
    let box_h = (sh * 0.82).clamp(scaler.s(480.0), scaler.s(580.0));
    let box_x = (sw - box_w) * 0.5;
    let box_y = (sh - box_h) * 0.5;

    let col_gap = scaler.s(20.0);
    let slots_w = scaler.s(260.0);
    let dossier_w = box_w - slots_w - col_gap - scaler.s(44.0);

    let slots_x = box_x + scaler.s(22.0);
    let slots_y = box_y + scaler.s(70.0);
    let slots_h = box_h - scaler.s(90.0);

    let dossier_x = slots_x + slots_w + col_gap;
    let dossier_y = slots_y;
    let dossier_h = slots_h;

    let slot_h = scaler.s(64.0);

    let btn_h = scaler.s(42.0);
    let btn_w = (dossier_w - scaler.s(15.0)) * 0.5;
    let row1_y = dossier_y + dossier_h - btn_h * 2.0 - scaler.s(18.0);
    let row2_y = row1_y + btn_h + scaler.s(10.0);

    let btn_select_rect = (dossier_x, row1_y, btn_w, btn_h);
    let btn_country_rect = (dossier_x + btn_w + scaler.s(15.0), row1_y, btn_w, btn_h);
    let btn_livery_rect = (dossier_x, row2_y, btn_w, btn_h);
    let btn_close_rect = (dossier_x + btn_w + scaler.s(15.0), row2_y, btn_w, btn_h);

    ProfileSelectLayout {
        box_x,
        box_y,
        box_w,
        box_h,
        slots_rect: (slots_x, slots_y, slots_w, slots_h),
        dossier_rect: (dossier_x, dossier_y, dossier_w, dossier_h),
        slot_h,
        btn_select_rect,
        btn_country_rect,
        btn_livery_rect,
        btn_close_rect,
    }
}

/// Generic arcade profile selection and pilot customizer modal.
///
/// Features:
/// - Visual multi-slot profile management (inspect, switch active profile).
/// - Shows driver name, callsign alias, nationality flag, and 3-tone livery swatch.
/// - In-place nationality cycling through ISO country database.
/// - In-place livery customization cycling through color scheme presets.
/// - 2D orthogonal navigation with keyboard, Gamepad, and mouse hit testing.
pub struct ProfileSelectModal {
    pub manager: ProfileManager,
    pub highlighted_slot: usize,
    pub nav: NavGrid2D, // Col 0: Slots, Col 1: Action buttons (4 buttons)
    pub is_saved: bool,
}

impl ProfileSelectModal {
    /// Creates a profile select modal from an existing ProfileManager.
    pub fn new(manager: &ProfileManager) -> Self {
        let count = manager.profiles.len();
        let highlighted = manager.active_index.min(count.saturating_sub(1));
        let mut nav = NavGrid2D::new(vec![count.max(1), 4]);
        nav.set_focus(0, highlighted);

        Self {
            manager: manager.clone(),
            highlighted_slot: highlighted,
            nav,
            is_saved: false,
        }
    }

    /// Syncs modal changes back to the target ProfileManager upon closing.
    pub fn apply_to_manager(&self, target: &mut ProfileManager) {
        *target = self.manager.clone();
    }

    /// Cycles the nationality of the currently highlighted profile.
    pub fn cycle_country(&mut self, forward: bool) {
        if let Some(profile) = self.manager.profiles.get_mut(self.highlighted_slot) {
            let all_countries = CountryRegistry::ALL;
            let cur_iso = profile.country.as_deref().unwrap_or("ESP");
            let mut idx = 0;
            for (i, c) in all_countries.iter().enumerate() {
                if c.code.eq_ignore_ascii_case(cur_iso) {
                    idx = i;
                    break;
                }
            }
            let new_idx = if forward {
                (idx + 1) % all_countries.len()
            } else {
                (idx + all_countries.len() - 1) % all_countries.len()
            };
            profile.country = Some(all_countries[new_idx].code.to_string());
        }
    }

    /// Cycles the livery palette of the currently highlighted profile.
    pub fn cycle_livery(&mut self) {
        if let Some(profile) = self.manager.profiles.get_mut(self.highlighted_slot) {
            let presets = ColorScheme::PRESETS;
            let mut cur_idx = 0;
            for (i, p) in presets.iter().enumerate() {
                if profile.color_scheme.primary == p.0 && profile.color_scheme.secondary == p.1 {
                    cur_idx = i;
                    break;
                }
            }
            let next_idx = (cur_idx + 1) % presets.len();
            profile.color_scheme = ColorScheme::from_index(next_idx);
        }
    }
}

impl CabinetScreen for ProfileSelectModal {
    fn name(&self) -> &str {
        "ProfileSelectModal"
    }

    fn is_transparent(&self) -> bool {
        true
    }

    fn update(&mut self, ctx: &mut CabinetContext) -> ScreenAction {
        let total_slots = self.manager.profiles.len();
        if self.nav.column_lengths[0] != total_slots {
            self.nav.set_column_len(0, total_slots);
        }

        self.nav.handle_standard_inputs(
            ctx.gamepad.nav_left,
            ctx.gamepad.nav_right,
            ctx.gamepad.nav_up,
            ctx.gamepad.nav_down,
        );

        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let layout = profile_select_layout(sw, sh);

        // Update highlighted slot when navigating column 0
        if self.nav.focused_col == 0 {
            self.highlighted_slot = self.nav.cursor_rows[0].min(total_slots.saturating_sub(1));
        }

        // Mouse hover checks for slots
        for i in 0..total_slots {
            let slot_y = layout.slots_rect.1 + (i as f32 * (layout.slot_h + ctx.scaler.s(8.0)));
            let slot_rect = (layout.slots_rect.0, slot_y, layout.slots_rect.2, layout.slot_h);
            if NavGrid2D::check_mouse_hover(slot_rect) {
                self.nav.set_focus(0, i);
                self.highlighted_slot = i;
                if NavGrid2D::check_mouse_click(slot_rect) {
                    self.manager.select_profile(i);
                }
            }
        }

        // Mouse hover checks for buttons
        if NavGrid2D::check_mouse_hover(layout.btn_select_rect) { self.nav.set_focus(1, 0); }
        if NavGrid2D::check_mouse_hover(layout.btn_country_rect) { self.nav.set_focus(1, 1); }
        if NavGrid2D::check_mouse_hover(layout.btn_livery_rect) { self.nav.set_focus(1, 2); }
        if NavGrid2D::check_mouse_hover(layout.btn_close_rect) { self.nav.set_focus(1, 3); }

        let select_clicked = NavGrid2D::check_mouse_click(layout.btn_select_rect);
        let country_clicked = NavGrid2D::check_mouse_click(layout.btn_country_rect);
        let livery_clicked = NavGrid2D::check_mouse_click(layout.btn_livery_rect);
        let close_clicked = NavGrid2D::check_mouse_click(layout.btn_close_rect);

        // Actions
        if select_clicked || (self.nav.focused_col == 1 && self.nav.cursor_rows[1] == 0 && self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed)) {
            self.manager.select_profile(self.highlighted_slot);
        }

        if country_clicked || (self.nav.focused_col == 1 && self.nav.cursor_rows[1] == 1 && self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed)) {
            self.cycle_country(true);
        }

        if livery_clicked || (self.nav.focused_col == 1 && self.nav.cursor_rows[1] == 2 && self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed)) {
            self.cycle_livery();
        }

        if close_clicked || (self.nav.focused_col == 1 && self.nav.cursor_rows[1] == 3 && self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed)) {
            self.is_saved = true;
            return ScreenAction::Pop;
        }

        // Shortcut: Enter on slot selects it
        if self.nav.focused_col == 0 && self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed) {
            self.manager.select_profile(self.highlighted_slot);
        }

        // Escape or Gamepad B pops
        if self.nav.is_cancelled(ctx.gamepad.btn_cancel_pressed || ctx.gamepad.btn_b_pressed || ctx.gamepad.btn_back_pressed) {
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

        // Dark dim backdrop
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.82));

        let layout = profile_select_layout(sw, sh);

        // Glassmorphism main card
        scaler.draw_glass_card(
            layout.box_x,
            layout.box_y,
            layout.box_w,
            layout.box_h,
            Palette::UI_CARD_BG,
            Palette::NEON_CYAN,
            2.2,
        );

        // Title Header
        fonts.draw_display_centered_with_shadow(
            "PILOT PROFILE MANAGER",
            sw * 0.5,
            layout.box_y + scaler.s(36.0),
            scaler.font_s(26.0),
            Palette::WHITE,
            Color::new(0.0, 0.0, 0.0, 0.6),
            scaler.s(2.0),
        );
        fonts.draw_ui_bold_centered(
            "SELECT ACTIVE PROFILE & CUSTOMIZE PILOT IDENTITY",
            sw * 0.5,
            layout.box_y + scaler.s(56.0),
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );

        // Left Panel: Slots List
        for (i, p) in self.manager.profiles.iter().enumerate() {
            let slot_y = layout.slots_rect.1 + (i as f32 * (layout.slot_h + scaler.s(8.0)));
            let is_focused = self.nav.focused_col == 0 && self.nav.cursor_rows[0] == i;
            let is_hovered = NavGrid2D::check_mouse_hover((layout.slots_rect.0, slot_y, layout.slots_rect.2, layout.slot_h));
            let is_active = self.manager.active_index == i;

            let border_col = if is_active {
                Palette::NEON_GREEN
            } else if is_focused {
                Palette::NEON_CYAN
            } else {
                Palette::UI_CARD_BORDER
            };

            let bg_col = if is_focused || is_hovered {
                Palette::UI_CARD_BG_HOVER
            } else {
                Color::new(0.08, 0.11, 0.16, 0.85)
            };

            scaler.draw_glass_card(layout.slots_rect.0, slot_y, layout.slots_rect.2, layout.slot_h, bg_col, border_col, 1.5);

            // Slot label
            let slot_tag = format!("SLOT {}", i + 1);
            let tag_col = if is_active { Palette::NEON_GREEN } else { Palette::UI_TEXT_MUTED };
            fonts.draw_ui_bold(&slot_tag, layout.slots_rect.0 + scaler.s(12.0), slot_y + scaler.s(20.0), scaler.font_s(10.5), tag_col);

            // Driver Name
            let name_str = if !p.alias.is_empty() { format!("{} \"{}\"", p.name, p.alias) } else { p.name.clone() };
            fonts.draw_ui_bold(&name_str, layout.slots_rect.0 + scaler.s(12.0), slot_y + scaler.s(42.0), scaler.font_s(13.0), Palette::WHITE);

            // Active Badge
            if is_active {
                fonts.draw_ui_bold("★ ACTIVE", layout.slots_rect.0 + layout.slots_rect.2 - scaler.s(65.0), slot_y + scaler.s(20.0), scaler.font_s(10.0), Palette::NEON_GREEN);
            }

            // Flag
            draw_country_banner(
                p.country.as_deref(),
                layout.slots_rect.0 + layout.slots_rect.2 - scaler.s(55.0),
                slot_y + scaler.s(28.0),
                scaler.s(45.0),
                scaler.s(16.0),
                Some(fonts),
                scaler,
            );
        }

        // Right Panel: Dossier of Highlighted Profile
        if let Some(p) = self.manager.profiles.get(self.highlighted_slot) {
            let dx = layout.dossier_rect.0;
            let dy = layout.dossier_rect.1;
            let dw = layout.dossier_rect.2;
            let dh = layout.dossier_rect.3;

            scaler.draw_glass_card(dx, dy, dw, dh, Color::new(0.06, 0.08, 0.12, 0.85), Palette::UI_CARD_BORDER, 1.2);

            let is_active = self.manager.active_index == self.highlighted_slot;
            let status_badge = if is_active { "★ ACTIVE PROFILE" } else { "STANDBY PROFILE" };
            let badge_col = if is_active { Palette::NEON_GREEN } else { Palette::UI_TEXT_MUTED };
            fonts.draw_ui_bold(status_badge, dx + scaler.s(18.0), dy + scaler.s(28.0), scaler.font_s(12.0), badge_col);

            // Full Driver Name & Callsign
            fonts.draw_display(&p.name, dx + scaler.s(18.0), dy + scaler.s(62.0), scaler.font_s(26.0), Palette::WHITE);
            let alias_str = format!("CALLSIGN: {}", p.alias.to_uppercase());
            fonts.draw_ui_bold(&alias_str, dx + scaler.s(18.0), dy + scaler.s(88.0), scaler.font_s(14.0), Palette::NEON_CYAN);

            // Country details
            let country_code = p.country.as_deref().unwrap_or("ESP");
            let country_name = CountryRegistry::find_by_code(country_code)
                .map(|c| c.name)
                .unwrap_or("International");
            fonts.draw_ui_bold("NATIONALITY:", dx + scaler.s(18.0), dy + scaler.s(125.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);
            let nat_str = format!("{} ({})", country_name, country_code);
            fonts.draw_ui_bold(&nat_str, dx + scaler.s(115.0), dy + scaler.s(125.0), scaler.font_s(13.0), Palette::WHITE);

            draw_country_banner(
                Some(country_code),
                dx + dw - scaler.s(85.0),
                dy + scaler.s(110.0),
                scaler.s(60.0),
                scaler.s(22.0),
                Some(fonts),
                scaler,
            );

            // Livery Swatches
            fonts.draw_ui_bold("LIVERY PALETTE:", dx + scaler.s(18.0), dy + scaler.s(165.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);
            let swatch_w = scaler.s(45.0);
            let swatch_h = scaler.s(18.0);
            let swatch_y = dy + scaler.s(150.0);

            draw_rectangle(dx + scaler.s(130.0), swatch_y, swatch_w, swatch_h, p.color_scheme.primary);
            draw_rectangle_lines(dx + scaler.s(130.0), swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);
            fonts.draw_ui_bold("PRI", dx + scaler.s(130.0) + scaler.s(8.0), swatch_y + scaler.s(14.0), scaler.font_s(10.0), Palette::BLACK);

            draw_rectangle(dx + scaler.s(185.0), swatch_y, swatch_w, swatch_h, p.color_scheme.secondary);
            draw_rectangle_lines(dx + scaler.s(185.0), swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);
            fonts.draw_ui_bold("SEC", dx + scaler.s(185.0) + scaler.s(8.0), swatch_y + scaler.s(14.0), scaler.font_s(10.0), Palette::BLACK);

            draw_rectangle(dx + scaler.s(240.0), swatch_y, swatch_w, swatch_h, p.color_scheme.accent);
            draw_rectangle_lines(dx + scaler.s(240.0), swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);
            fonts.draw_ui_bold("ACC", dx + scaler.s(240.0) + scaler.s(8.0), swatch_y + scaler.s(14.0), scaler.font_s(10.0), Palette::BLACK);
        }

        // Action Buttons
        let b0_focused = self.nav.focused_col == 1 && self.nav.cursor_rows[1] == 0;
        let b0_hovered = NavGrid2D::check_mouse_hover(layout.btn_select_rect);
        let (x0, y0, w0, h0) = layout.btn_select_rect;
        scaler.draw_button_card(x0, y0, w0, h0, b0_focused, b0_hovered, Palette::NEON_GREEN);
        fonts.draw_ui_bold_centered("SET AS ACTIVE [ENTER]", x0 + w0 * 0.5, y0 + h0 * 0.60, scaler.font_s(12.0), Palette::WHITE);

        let b1_focused = self.nav.focused_col == 1 && self.nav.cursor_rows[1] == 1;
        let b1_hovered = NavGrid2D::check_mouse_hover(layout.btn_country_rect);
        let (x1, y1, w1, h1) = layout.btn_country_rect;
        scaler.draw_button_card(x1, y1, w1, h1, b1_focused, b1_hovered, Palette::NEON_CYAN);
        fonts.draw_ui_bold_centered("CYCLE COUNTRY [ > ]", x1 + w1 * 0.5, y1 + h1 * 0.60, scaler.font_s(12.0), Palette::WHITE);

        let b2_focused = self.nav.focused_col == 1 && self.nav.cursor_rows[1] == 2;
        let b2_hovered = NavGrid2D::check_mouse_hover(layout.btn_livery_rect);
        let (x2, y2, w2, h2) = layout.btn_livery_rect;
        scaler.draw_button_card(x2, y2, w2, h2, b2_focused, b2_hovered, Palette::NEON_MAGENTA);
        fonts.draw_ui_bold_centered("CYCLE LIVERY [ C ]", x2 + w2 * 0.5, y2 + h2 * 0.60, scaler.font_s(12.0), Palette::WHITE);

        let b3_focused = self.nav.focused_col == 1 && self.nav.cursor_rows[1] == 3;
        let b3_hovered = NavGrid2D::check_mouse_hover(layout.btn_close_rect);
        let (x3, y3, w3, h3) = layout.btn_close_rect;
        scaler.draw_button_card(x3, y3, w3, h3, b3_focused, b3_hovered, Palette::NEON_GOLD);
        fonts.draw_ui_bold_centered("CLOSE [ESC]", x3 + w3 * 0.5, y3 + h3 * 0.60, scaler.font_s(12.0), Palette::WHITE);
    }
}
