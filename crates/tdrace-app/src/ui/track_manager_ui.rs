use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::draw_rectangle;

use super::font::Fonts;
use super::scaler::UiScaler;
use crate::render::color::Palette;
use crate::track_manager::TrackManager;
use crate::ui::menu::TrackChoice;

/// Available category tabs in the Track Manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackManagerTab {
    Main,
    Drafts,
}

/// Modals that can be displayed as overlays in the Track Manager.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackManagerModal {
    None,
    EditMetadata {
        track_id: String,
        name_input: String,
        desc_input: String,
        active_field: usize,
        cursor_timer: f32,
    },
    ConfirmDelete {
        track_id: String,
        track_title: String,
    },
}

/// Action dispatched from Track Manager interactions.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackManagerAction {
    None,
    SwitchTab(TrackManagerTab),
    SelectIndex(usize),
    RaceTrack(TrackChoice),
    EditInStudio(TrackChoice),
    PromoteTrack(String),
    DemoteTrack(String),
    OpenEditModal {
        track_id: String,
        name: String,
        description: String,
    },
    SaveMetadata {
        track_id: String,
        new_name: String,
        new_description: String,
    },
    PromptDelete {
        track_id: String,
        track_title: String,
    },
    ConfirmDelete(String),
    CreateNewDraft,
    BackToMenu,
}

/// Renders the complete Track Manager interface.
pub fn render_track_manager_screen(
    fonts: &Fonts,
    track_manager: &TrackManager,
    active_tab: TrackManagerTab,
    selected_idx: usize,
    modal: &TrackManagerModal,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep motorsport backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Header Title
    let title = "🏁 CIRCUIT HUB & TRACK MANAGER";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        scaler.s(34.0),
        scaler.font_s(28.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let subtitle = "Tested & Approved Circuits (Main Menu) vs Experimental Drafts Workshop";
    fonts.draw_ui_regular_centered(
        subtitle,
        sw * 0.5,
        scaler.s(54.0),
        scaler.font_s(13.0),
        Palette::UI_TEXT_MUTED,
    );

    // Main Card Dimensions
    let box_w = (sw * 0.94).clamp(scaler.s(700.0), scaler.s(1100.0));
    let box_h = (sh * 0.78).clamp(scaler.s(450.0), scaler.s(640.0));
    let box_x = (sw - box_w) * 0.5;
    let box_y = scaler.s(66.0);

    scaler.draw_glass_card(box_x, box_y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 1.8);

    // Tab Headers Row
    let tab_w = (box_w - scaler.s(32.0)) * 0.5;
    let tab_h = scaler.s(36.0);
    let tab_y = box_y + scaler.s(12.0);

    let main_count = track_manager.main_track_choices().len();
    let draft_count = track_manager.draft_track_choices().len();

    // Tab 1: Main Tracks (Approved)
    let is_main_active = active_tab == TrackManagerTab::Main;
    let tab1_bg = if is_main_active {
        Palette::UI_CARD_BG_HOVER
    } else {
        Color::new(0.07, 0.09, 0.14, 0.80)
    };
    let tab1_border = if is_main_active {
        Palette::NEON_GREEN
    } else {
        Palette::UI_CARD_BORDER
    };
    scaler.draw_glass_card(box_x + scaler.s(12.0), tab_y, tab_w, tab_h, tab1_bg, tab1_border, if is_main_active { 2.0 } else { 1.0 });

    let tab1_label = format!("⭐ MAIN TRACKS (Approved) [{}] [Key 1 / Left]", main_count);
    fonts.draw_ui_bold(
        &tab1_label,
        box_x + scaler.s(24.0),
        tab_y + scaler.s(23.0),
        scaler.font_s(14.0),
        if is_main_active { Palette::NEON_GREEN } else { Palette::UI_TEXT_MUTED },
    );

    // Tab 2: Drafts & Testing
    let is_draft_active = active_tab == TrackManagerTab::Drafts;
    let tab2_bg = if is_draft_active {
        Palette::UI_CARD_BG_HOVER
    } else {
        Color::new(0.07, 0.09, 0.14, 0.80)
    };
    let tab2_border = if is_draft_active {
        Palette::NEON_GOLD
    } else {
        Palette::UI_CARD_BORDER
    };
    scaler.draw_glass_card(box_x + scaler.s(20.0) + tab_w, tab_y, tab_w, tab_h, tab2_bg, tab2_border, if is_draft_active { 2.0 } else { 1.0 });

    let tab2_label = format!("🧪 DRAFTS & TESTING [{}] [Key 2 / Right]", draft_count);
    fonts.draw_ui_bold(
        &tab2_label,
        box_x + scaler.s(32.0) + tab_w,
        tab_y + scaler.s(23.0),
        scaler.font_s(14.0),
        if is_draft_active { Palette::NEON_GOLD } else { Palette::UI_TEXT_MUTED },
    );

    // Two-Column Content Area below tabs
    let content_y = tab_y + tab_h + scaler.s(12.0);
    let content_h = box_h - (tab_h + scaler.s(34.0));
    let col1_w = (box_w * 0.42).clamp(scaler.s(260.0), scaler.s(420.0));
    let col2_w = box_w - col1_w - scaler.s(36.0);
    let col1_x = box_x + scaler.s(12.0);
    let col2_x = col1_x + col1_w + scaler.s(12.0);

    // Get current tracks for active tab
    let tracks_list = match active_tab {
        TrackManagerTab::Main => track_manager.main_track_choices(),
        TrackManagerTab::Drafts => track_manager.draft_track_choices(),
    };

    // --- LEFT COLUMN: TRACK LIST ---
    scaler.draw_glass_card(col1_x, content_y, col1_w, content_h, Color::new(0.06, 0.08, 0.12, 0.90), Palette::UI_CARD_BORDER, 1.2);

    let list_pad_y = scaler.s(8.0);
    let mut item_y = content_y + list_pad_y;
    let item_h = scaler.s(56.0);

    if tracks_list.is_empty() {
        fonts.draw_ui_regular(
            "No tracks in this category.\nPress [C] to create a new draft track!",
            col1_x + scaler.s(16.0),
            content_y + scaler.s(40.0),
            scaler.font_s(13.0),
            Palette::UI_TEXT_MUTED,
        );
    } else {
        // Scroll / Windowing if more than fits
        let visible_items = 6;
        let start_idx = if tracks_list.len() <= visible_items {
            0
        } else {
            selected_idx.saturating_sub(visible_items / 2).min(tracks_list.len() - visible_items)
        };
        let end_idx = (start_idx + visible_items).min(tracks_list.len());

        for i in start_idx..end_idx {
            let track_choice = &tracks_list[i];
            let is_sel = i == selected_idx;

            let item_bg = if is_sel {
                Palette::UI_CARD_BG_HOVER
            } else {
                Color::new(0.08, 0.10, 0.16, 0.60)
            };
            let item_border = if is_sel {
                if is_main_active { Palette::NEON_GREEN } else { Palette::NEON_GOLD }
            } else {
                Palette::UI_CARD_BORDER
            };

            scaler.draw_glass_card(col1_x + scaler.s(6.0), item_y, col1_w - scaler.s(12.0), item_h, item_bg, item_border, if is_sel { 1.8 } else { 1.0 });

            // Tag Pill
            let tag_text = match track_choice {
                TrackChoice::Custom { .. } => {
                    if is_main_active { "PROMOTED MAIN" } else { "TESTING DRAFT" }
                }
                _ => "OFFICIAL PRESET",
            };
            let tag_col = match track_choice {
                TrackChoice::Custom { .. } => {
                    if is_main_active { Palette::NEON_GREEN } else { Palette::NEON_GOLD }
                }
                _ => Palette::NEON_CYAN,
            };
            fonts.draw_ui_bold(
                tag_text,
                col1_x + scaler.s(16.0),
                item_y + scaler.s(16.0),
                scaler.font_s(10.0),
                tag_col,
            );

            // Title
            fonts.draw_ui_bold(
                track_choice.title(),
                col1_x + scaler.s(16.0),
                item_y + scaler.s(34.0),
                scaler.font_s(15.0),
                if is_sel { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 1.0) },
            );

            // Short metric info
            let metric_str = format!("ID: {}", track_choice.track_id());
            fonts.draw_ui_regular(
                &metric_str,
                col1_x + scaler.s(16.0),
                item_y + scaler.s(48.0),
                scaler.font_s(11.0),
                Palette::UI_TEXT_MUTED,
            );

            item_y += item_h + scaler.s(6.0);
        }
    }

    // --- RIGHT COLUMN: TRACK DOSSIER & METRICS ---
    scaler.draw_glass_card(col2_x, content_y, col2_w, content_h, Color::new(0.06, 0.08, 0.12, 0.90), Palette::UI_CARD_BORDER, 1.2);

    if let Some(selected_track) = tracks_list.get(selected_idx) {
        let pad_x = col2_x + scaler.s(18.0);
        let mut d_y = content_y + scaler.s(22.0);

        // Header: Category Pill & Title
        let (badge_str, badge_col) = match selected_track {
            TrackChoice::Custom { .. } => {
                if is_main_active {
                    ("⭐ PROMOTED TO MAIN (Menu Active)", Palette::NEON_GREEN)
                } else {
                    ("🧪 DRAFT / TESTING (Hidden from Menu)", Palette::NEON_GOLD)
                }
            }
            _ => ("🏆 OFFICIAL PRESET CIRCUIT", Palette::NEON_CYAN),
        };

        fonts.draw_ui_bold(badge_str, pad_x, d_y, scaler.font_s(12.0), badge_col);
        d_y += scaler.s(24.0);

        fonts.draw_display(
            selected_track.title(),
            pad_x,
            d_y,
            scaler.font_s(22.0),
            Palette::WHITE,
        );
        d_y += scaler.s(14.0);

        // Description Box
        let desc_h = scaler.s(62.0);
        let desc_w = col2_w - scaler.s(36.0);
        scaler.draw_glass_card(pad_x, d_y, desc_w, desc_h, Color::new(0.08, 0.10, 0.15, 0.70), Palette::UI_CARD_BORDER, 1.0);

        fonts.draw_ui_regular(
            "CIRCUIT DESCRIPTION [Edit with N]:",
            pad_x + scaler.s(10.0),
            d_y + scaler.s(16.0),
            scaler.font_s(11.0),
            Palette::NEON_CYAN,
        );
        fonts.draw_ui_regular(
            selected_track.description(),
            pad_x + scaler.s(10.0),
            d_y + scaler.s(34.0),
            scaler.font_s(12.5),
            Palette::WHITE,
        );
        d_y += desc_h + scaler.s(16.0);

        // Circuit Metrics Grid
        let custom_info = track_manager.custom_tracks.iter().find(|t| t.id == selected_track.track_id());

        fonts.draw_ui_bold("CIRCUIT SPECIFICATIONS & GEOMETRY:", pad_x, d_y, scaler.font_s(13.0), Palette::NEON_GOLD);
        d_y += scaler.s(14.0);

        let grid_y = d_y;
        let card_w = (desc_w - scaler.s(12.0)) * 0.5;
        let card_h = scaler.s(42.0);

        // Metric Card 1: Track Length & Waypoints
        scaler.draw_glass_card(pad_x, grid_y, card_w, card_h, Color::new(0.07, 0.09, 0.14, 0.70), Palette::UI_CARD_BORDER, 1.0);
        let len_str = if let Some(info) = custom_info {
            format!("{:.0}m Length ({} WPs)", info.length_m, info.waypoint_count)
        } else {
            "Standard Circuit Spline".to_string()
        };
        fonts.draw_ui_regular("LENGTH & NODES", pad_x + scaler.s(8.0), grid_y + scaler.s(14.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
        fonts.draw_ui_bold(&len_str, pad_x + scaler.s(8.0), grid_y + scaler.s(30.0), scaler.font_s(13.0), Palette::WHITE);

        // Metric Card 2: Checkpoints / Timing Gates
        scaler.draw_glass_card(pad_x + card_w + scaler.s(12.0), grid_y, card_w, card_h, Color::new(0.07, 0.09, 0.14, 0.70), Palette::UI_CARD_BORDER, 1.0);
        let cp_str = if let Some(info) = custom_info {
            format!("{} Timing Gates", info.checkpoint_count)
        } else {
            "Multi-Sector Timing".to_string()
        };
        fonts.draw_ui_regular("CHECKPOINTS", pad_x + card_w + scaler.s(20.0), grid_y + scaler.s(14.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
        fonts.draw_ui_bold(&cp_str, pad_x + card_w + scaler.s(20.0), grid_y + scaler.s(30.0), scaler.font_s(13.0), Palette::WHITE);

        // Metric Card 3: Jump Ramps & Obstacles
        let grid_y2 = grid_y + card_h + scaler.s(8.0);
        scaler.draw_glass_card(pad_x, grid_y2, card_w, card_h, Color::new(0.07, 0.09, 0.14, 0.70), Palette::UI_CARD_BORDER, 1.0);
        let obs_str = if let Some(info) = custom_info {
            format!("{} Ramps • {} Obstacles", info.jump_ramp_count, info.obstacle_count)
        } else {
            "Track Hazards Configured".to_string()
        };
        fonts.draw_ui_regular("RAMPS & OBSTACLES", pad_x + scaler.s(8.0), grid_y2 + scaler.s(14.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
        fonts.draw_ui_bold(&obs_str, pad_x + scaler.s(8.0), grid_y2 + scaler.s(30.0), scaler.font_s(13.0), Palette::WHITE);

        // Metric Card 4: Surface & Laps
        scaler.draw_glass_card(pad_x + card_w + scaler.s(12.0), grid_y2, card_w, card_h, Color::new(0.07, 0.09, 0.14, 0.70), Palette::UI_CARD_BORDER, 1.0);
        let surf_str = if let Some(info) = custom_info {
            format!("{:?} • {} Laps", info.default_surface, info.default_laps)
        } else {
            "Asphalt / Dirt Surface".to_string()
        };
        fonts.draw_ui_regular("SURFACE & DEFAULT LAPS", pad_x + card_w + scaler.s(20.0), grid_y2 + scaler.s(14.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
        fonts.draw_ui_bold(&surf_str, pad_x + card_w + scaler.s(20.0), grid_y2 + scaler.s(30.0), scaler.font_s(13.0), Palette::WHITE);

        d_y = grid_y2 + card_h + scaler.s(18.0);

        // Category Status Explanation Box
        let expl_bg = if is_main_active {
            Color::new(0.08, 0.18, 0.12, 0.70)
        } else {
            Color::new(0.18, 0.14, 0.06, 0.70)
        };
        let expl_border = if is_main_active {
            Palette::NEON_GREEN
        } else {
            Palette::NEON_GOLD
        };
        scaler.draw_glass_card(pad_x, d_y, desc_w, scaler.s(44.0), expl_bg, expl_border, 1.2);

        let expl_text = if is_main_active {
            if selected_track.is_custom() {
                "⭐ This custom track is PROMOTED. It appears as an approved circuit in the Main Menu."
            } else {
                "🏆 Built-in official preset circuit. Always available in the Main Menu."
            }
        } else {
            "🧪 This track is in DRAFT mode. Test and refine it here, then press [P] to promote to Main Menu."
        };
        fonts.draw_ui_regular(
            expl_text,
            pad_x + scaler.s(10.0),
            d_y + scaler.s(26.0),
            scaler.font_s(12.0),
            Palette::WHITE,
        );
    }

    // Bottom Action Prompt Bar
    let bar_y = sh - scaler.s(32.0);
    let action_str = if is_main_active {
        "[Enter] RACE | [E] EDIT IN STUDIO | [P] DEMOTE TO DRAFT | [N] EDIT INFO | [C] NEW DRAFT | [Del] DELETE | [Esc] BACK"
    } else {
        "[Enter] RACE | [E] EDIT IN STUDIO | [P] PROMOTE TO MAIN | [N] EDIT INFO | [C] NEW DRAFT | [Del] DELETE | [Esc] BACK"
    };
    fonts.draw_ui_bold_centered(
        action_str,
        sw * 0.5,
        bar_y,
        scaler.font_s(12.0),
        Palette::NEON_CYAN,
    );

    // Modal Overlays
    match modal {
        TrackManagerModal::EditMetadata {
            name_input,
            desc_input,
            active_field,
            cursor_timer,
            ..
        } => {
            render_edit_modal(fonts, &scaler, sw, sh, name_input, desc_input, *active_field, *cursor_timer);
        }
        TrackManagerModal::ConfirmDelete { track_title, .. } => {
            render_delete_modal(fonts, &scaler, sw, sh, track_title);
        }
        TrackManagerModal::None => {}
    }
}

fn render_edit_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    name_input: &str,
    desc_input: &str,
    active_field: usize,
    cursor_timer: f32,
) {
    // Backdrop dimming
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.75));

    let mw = scaler.s(520.0);
    let mh = scaler.s(280.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.2);

    fonts.draw_ui_bold(
        "✏️ EDIT TRACK NAME & DESCRIPTION",
        mx + scaler.s(20.0),
        my + scaler.s(32.0),
        scaler.font_s(18.0),
        Palette::NEON_GOLD,
    );

    fonts.draw_ui_regular(
        "Press [Tab] or [Up/Down] to switch fields • [Enter] Save • [Esc] Cancel",
        mx + scaler.s(20.0),
        my + scaler.s(52.0),
        scaler.font_s(12.0),
        Palette::UI_TEXT_MUTED,
    );

    let is_cursor_visible = (cursor_timer % 0.8) < 0.4;

    // Field 1: Track Name
    let f1_y = my + scaler.s(72.0);
    let f_w = mw - scaler.s(40.0);
    let f_h = scaler.s(44.0);

    let is_f1_active = active_field == 0;
    fonts.draw_ui_bold("TRACK NAME:", mx + scaler.s(20.0), f1_y + scaler.s(12.0), scaler.font_s(12.0), if is_f1_active { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED });
    scaler.draw_glass_card(
        mx + scaler.s(20.0),
        f1_y + scaler.s(16.0),
        f_w,
        f_h,
        Color::new(0.08, 0.10, 0.16, 0.90),
        if is_f1_active { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER },
        if is_f1_active { 2.0 } else { 1.0 },
    );

    let name_display = if is_f1_active && is_cursor_visible {
        format!("{}|", name_input)
    } else {
        name_input.to_string()
    };
    fonts.draw_ui_bold(&name_display, mx + scaler.s(30.0), f1_y + scaler.s(42.0), scaler.font_s(16.0), Palette::WHITE);

    // Field 2: Track Description
    let f2_y = f1_y + f_h + scaler.s(24.0);
    let is_f2_active = active_field == 1;
    fonts.draw_ui_bold("TRACK DESCRIPTION:", mx + scaler.s(20.0), f2_y + scaler.s(12.0), scaler.font_s(12.0), if is_f2_active { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED });
    scaler.draw_glass_card(
        mx + scaler.s(20.0),
        f2_y + scaler.s(16.0),
        f_w,
        f_h,
        Color::new(0.08, 0.10, 0.16, 0.90),
        if is_f2_active { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER },
        if is_f2_active { 2.0 } else { 1.0 },
    );

    let desc_display = if is_f2_active && is_cursor_visible {
        format!("{}|", desc_input)
    } else {
        desc_input.to_string()
    };
    fonts.draw_ui_regular(&desc_display, mx + scaler.s(30.0), f2_y + scaler.s(42.0), scaler.font_s(14.0), Palette::WHITE);

    // Footer buttons
    let btn_y = my + mh - scaler.s(40.0);
    fonts.draw_ui_bold("[Enter] SAVE CHANGES", mx + scaler.s(20.0), btn_y, scaler.font_s(14.0), Palette::NEON_GREEN);
    fonts.draw_ui_bold("[Esc] CANCEL", mx + mw - scaler.s(110.0), btn_y, scaler.font_s(14.0), Palette::RED);
}

fn render_delete_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    track_title: &str,
) {
    // Backdrop dimming
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.75));

    let mw = scaler.s(460.0);
    let mh = scaler.s(180.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::RED, 2.2);

    fonts.draw_ui_bold(
        "⚠️ DELETE CUSTOM CIRCUIT",
        mx + scaler.s(20.0),
        my + scaler.s(34.0),
        scaler.font_s(18.0),
        Palette::RED,
    );

    let confirm_msg = format!("Are you sure you want to permanently delete\n\"{}\"?", track_title);
    fonts.draw_ui_regular(
        &confirm_msg,
        mx + scaler.s(20.0),
        my + scaler.s(68.0),
        scaler.font_s(14.0),
        Palette::WHITE,
    );

    let btn_y = my + mh - scaler.s(28.0);
    fonts.draw_ui_bold("[Enter / Y] YES, DELETE", mx + scaler.s(20.0), btn_y, scaler.font_s(14.0), Palette::RED);
    fonts.draw_ui_bold("[Esc / N] CANCEL", mx + mw - scaler.s(130.0), btn_y, scaler.font_s(14.0), Palette::NEON_CYAN);
}
