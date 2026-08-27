use glam::Vec2;
use macroquad::color::Color;
use macroquad::input::{
    get_char_pressed, is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode,
    MouseButton,
};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use macroquad::window::{screen_height, screen_width};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::validation::{validate_track, ValidationSeverity};

use crate::editor::camera::EditorCamera;
use crate::editor::state::{EditorState, GridSnapSetting, Selection};
use crate::editor::tools::{EditorToolType, ToolSettings};
use crate::render::color::Palette;
use crate::track_manager::TrackManager;
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;

/// Modals that can be displayed as overlays on top of the editor viewport.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorModal {
    None,
    Templates,
    SaveAs { input_name: String },
    OpenTrack,
    Diagnostics,
    Help,
}

/// Actions dispatched from editor UI interactions.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorAction {
    None,
    SetTool(EditorToolType),
    SetSnap(GridSnapSetting),
    CycleZoom,
    NewFromTemplate(String),
    SaveTrack(String),
    OpenTrack(String),
    Validate,
    StartTestDrive,
    FocusCamera,
    ToggleHelp,
    ExitToMenu,
}

/// Main UI renderer for the Track Editor suite.
pub fn render_editor_ui(
    fonts: &Fonts,
    state: &mut EditorState,
    tools: &mut ToolSettings,
    camera: &mut EditorCamera,
    track_manager: &mut TrackManager,
    active_modal: &mut EditorModal,
) -> EditorAction {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);
    let (mx, my) = mouse_position();
    let mouse_pos = Vec2::new(mx, my);
    let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);

    let is_modal_open = *active_modal != EditorModal::None;
    let bg_mouse_clicked = mouse_clicked && !is_modal_open;

    let mut dispatched_action = EditorAction::None;

    // 1. TOP TOOLBAR
    let top_h = scaler.s(46.0);
    scaler.draw_glass_card(0.0, 0.0, sw, top_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

    let mut tb_x = scaler.s(16.0);
    // Editor Title Badge
    fonts.draw_display(
        "CIRCUIT STUDIO",
        tb_x,
        scaler.s(28.0),
        scaler.font_s(20.0),
        Palette::NEON_GOLD,
    );
    tb_x += scaler.s(160.0);

    // Track Name label
    fonts.draw_ui_bold(
        &format!("Track: {}", state.track.name),
        tb_x,
        scaler.s(26.0),
        scaler.font_s(14.0),
        Palette::WHITE,
    );
    tb_x += scaler.s(180.0);

    // Top action buttons
    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(65.0), scaler.s(30.0), "NEW", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, bg_mouse_clicked) {
        *active_modal = EditorModal::Templates;
    }
    tb_x += scaler.s(72.0);

    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(65.0), scaler.s(30.0), "OPEN", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, bg_mouse_clicked) {
        let _ = track_manager.scan_custom_tracks();
        *active_modal = EditorModal::OpenTrack;
    }
    tb_x += scaler.s(72.0);

    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(65.0), scaler.s(30.0), "SAVE", Palette::UI_CARD_BG, Palette::NEON_GREEN, mouse_pos, bg_mouse_clicked) {
        *active_modal = EditorModal::SaveAs {
            input_name: state.track.name.clone(),
        };
    }
    tb_x += scaler.s(72.0);

    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(80.0), scaler.s(30.0), "VALIDATE", Palette::UI_CARD_BG, Palette::YELLOW, mouse_pos, bg_mouse_clicked) {
        *active_modal = EditorModal::Diagnostics;
    }
    tb_x += scaler.s(88.0);

    // Snap Setting Selector
    let snap_str = state.grid_snap.label();
    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(90.0), scaler.s(30.0), snap_str, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, bg_mouse_clicked) {
        state.grid_snap = state.grid_snap.next();
    }
    tb_x += scaler.s(98.0);

    // Zoom Level Selector
    let zoom_str = format!("ZOOM: {}", camera.current_zoom_level().name.to_uppercase());
    let zoom_w = scaler.s(105.0);
    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), zoom_w, scaler.s(30.0), &zoom_str, Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, bg_mouse_clicked) {
        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::MIN);
        for wp in &state.track.spline.waypoints {
            min = min.min(wp.point);
            max = max.max(wp.point);
        }
        let bounds = if min.x <= max.x {
            Some((min, max))
        } else {
            None
        };
        camera.cycle_zoom_level_with_bounds(bounds, sw, sh);
        dispatched_action = EditorAction::CycleZoom;
    }
    tb_x += zoom_w + scaler.s(8.0);

    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(65.0), scaler.s(30.0), "FOCUS [F]", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, bg_mouse_clicked) {
        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::MIN);
        for wp in &state.track.spline.waypoints {
            min = min.min(wp.point);
            max = max.max(wp.point);
        }
        if min.x > max.x {
            min = Vec2::new(-100.0, -100.0);
            max = Vec2::new(100.0, 100.0);
        }
        camera.focus_bounds(min, max, sw, sh);
    }
    tb_x += scaler.s(72.0);

    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(55.0), scaler.s(30.0), "HELP [?]", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, bg_mouse_clicked) {
        *active_modal = EditorModal::Help;
    }

    // Right Side: TEST DRIVE & EXIT buttons
    let test_drive_w = scaler.s(130.0);
    let exit_w = scaler.s(70.0);
    let td_x = sw - test_drive_w - exit_w - scaler.s(24.0);

    if draw_ui_btn(fonts, &scaler, td_x, scaler.s(8.0), test_drive_w, scaler.s(30.0), "TEST DRIVE [Space]", Color::new(0.12, 0.65, 0.32, 0.95), Palette::NEON_GREEN, mouse_pos, bg_mouse_clicked) {
        dispatched_action = EditorAction::StartTestDrive;
    }

    if draw_ui_btn(fonts, &scaler, sw - exit_w - scaler.s(12.0), scaler.s(8.0), exit_w, scaler.s(30.0), "EXIT", Palette::UI_CARD_BG, Palette::RED, mouse_pos, bg_mouse_clicked) {
        dispatched_action = EditorAction::ExitToMenu;
    }

    // 2. LEFT TOOL PALETTE
    let tool_w = scaler.s(165.0);
    let tool_y = top_h + scaler.s(12.0);
    let tool_h = scaler.s(410.0);
    scaler.draw_glass_card(scaler.s(12.0), tool_y, tool_w, tool_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

    fonts.draw_ui_bold(
        "TOOLS [1-8]",
        scaler.s(22.0),
        tool_y + scaler.s(20.0),
        scaler.font_s(13.0),
        Palette::NEON_CYAN,
    );

    let tools_list = [
        (EditorToolType::Select, "[1] Select & Move"),
        (EditorToolType::RoadSpline, "[2] Road Spline"),
        (EditorToolType::SurfaceZone, "[3] Surface Zone"),
        (EditorToolType::JumpRamp, "[4] Jump Ramp"),
        (EditorToolType::Obstacle, "[5] Obstacle Prop"),
        (EditorToolType::Checkpoint, "[6] Checkpoint Gate"),
        (EditorToolType::StartingGrid, "[7] Grid Slot"),
        (EditorToolType::PitLane, "[8] Pit Lane"),
    ];

    let mut item_y = tool_y + scaler.s(32.0);
    for (tool_type, label) in tools_list {
        let is_active = tools.active_tool == tool_type;
        let bg_col = if is_active { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG };
        let border_col = if is_active { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };

        if draw_ui_btn(fonts, &scaler, scaler.s(20.0), item_y, tool_w - scaler.s(16.0), scaler.s(38.0), label, bg_col, border_col, mouse_pos, bg_mouse_clicked) {
            tools.active_tool = tool_type;
        }
        item_y += scaler.s(44.0);
    }

    // 3. RIGHT INSPECTOR PANEL (when entity or circuit is selected)
    let insp_w = scaler.s(240.0);
    let insp_x = sw - insp_w - scaler.s(12.0);
    let insp_y = top_h + scaler.s(12.0);
    let insp_h = sh - top_h - scaler.s(54.0);

    scaler.draw_glass_card(insp_x, insp_y, insp_w, insp_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

    render_inspector(fonts, &scaler, insp_x, insp_y, insp_w, insp_h, state, tools, mouse_pos, bg_mouse_clicked);

    // 4. BOTTOM STATUS BAR
    let bot_h = scaler.s(32.0);
    let bot_y = sh - bot_h;
    scaler.draw_glass_card(0.0, bot_y, sw, bot_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.0);

    let world_mouse = camera.screen_to_world(mouse_pos, sw, sh);
    let total_len = state.track.spline.total_length();
    let val = validate_track(&state.track);
    let is_valid = val.iter().all(|e| e.severity != ValidationSeverity::Error);
    let val_str = if is_valid {
        "✓ Circuit Valid"
    } else {
        "! Issues Detected [V]"
    };
    let val_col = if is_valid { Palette::NEON_GREEN } else { Palette::RED };

    fonts.draw_ui_bold(
        val_str,
        scaler.s(16.0),
        bot_y + scaler.s(20.0),
        scaler.font_s(13.0),
        val_col,
    );

    let info_str = format!(
        "Length: {:.0}m | Waypoints: {} | Checkpoints: {} | Pos: ({:.1}m, {:.1}m) | Zoom: {} ({:.1}x) | Undo: {} / Redo: {}",
        total_len,
        state.track.spline.waypoints.len(),
        state.track.checkpoints.len(),
        world_mouse.x,
        world_mouse.y,
        camera.current_zoom_level().name,
        camera.zoom,
        state.history.undo_count(),
        state.history.redo_count(),
    );

    fonts.draw_ui_regular(
        &info_str,
        scaler.s(160.0),
        bot_y + scaler.s(20.0),
        scaler.font_s(12.0),
        Palette::UI_TEXT_MUTED,
    );

    // 5. MODAL OVERLAYS (rendered on top of all toolbars, panels, and track)
    if *active_modal != EditorModal::None {
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.78));
        match active_modal {
            EditorModal::Templates => {
                if let Some(action) = render_template_modal(fonts, &scaler, sw, sh, mouse_pos, mouse_clicked) {
                    dispatched_action = action;
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::SaveAs { input_name } => {
                if let Some(action) = render_save_modal(fonts, &scaler, sw, sh, input_name, mouse_pos, mouse_clicked) {
                    dispatched_action = action;
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::OpenTrack => {
                if let Some(action) = render_open_modal(fonts, &scaler, sw, sh, track_manager, mouse_pos, mouse_clicked) {
                    dispatched_action = action;
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::Diagnostics => {
                if render_diagnostics_modal(fonts, &scaler, sw, sh, state, mouse_pos, mouse_clicked) {
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::Help => {
                if render_help_modal(fonts, &scaler, sw, sh, mouse_pos, mouse_clicked) {
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::None => {}
        }

        if is_key_pressed(KeyCode::Escape) {
            *active_modal = EditorModal::None;
        }
    }

    dispatched_action
}

/// Renders the property inspector for selected items or circuit settings.
fn render_inspector(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    _h: f32,
    state: &mut EditorState,
    tools: &mut ToolSettings,
    mouse_pos: Vec2,
    clicked: bool,
) {
    fonts.draw_ui_bold(
        "INSPECTOR",
        x + scaler.s(12.0),
        y + scaler.s(22.0),
        scaler.font_s(14.0),
        Palette::NEON_GOLD,
    );

    let mut curr_y = y + scaler.s(36.0);

    match state.selection {
        Selection::Waypoint(idx) => {
            if idx < state.track.spline.waypoints.len() {
                fonts.draw_ui_bold(&format!("Waypoint #{}", idx), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
                curr_y += scaler.s(24.0);

                let p = state.track.spline.waypoints[idx].point;
                fonts.draw_ui_regular(&format!("Pos: ({:.1}, {:.1})", p.x, p.y), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);
                curr_y += scaler.s(22.0);

                let road_w = state.track.spline.waypoints[idx].width;
                fonts.draw_ui_bold(&format!("Width: {:.1}m", road_w), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_CYAN);
                if draw_ui_btn(fonts, scaler, x + scaler.s(120.0), curr_y, scaler.s(45.0), scaler.s(22.0), "-1m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].width = (road_w - 1.0).max(4.0);
                    state.track.rebuild_geometry(2.5, tdrace_core::track::geometry::BarrierType::Armco);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(170.0), curr_y, scaler.s(45.0), scaler.s(22.0), "+1m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].width = (road_w + 1.0).min(30.0);
                    state.track.rebuild_geometry(2.5, tdrace_core::track::geometry::BarrierType::Armco);
                }
                curr_y += scaler.s(32.0);

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DELETE WAYPOINT [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                    tools.delete_selected(state);
                }
            }
        }
        Selection::SurfaceZone(idx) => {
            if idx < state.track.geometry.surface_zones.len() {
                let zone = &state.track.geometry.surface_zones[idx];
                fonts.draw_ui_bold(&format!("Surface Zone #{}", idx), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
                curr_y += scaler.s(24.0);

                fonts.draw_ui_regular(&format!("Type: {:?}", zone.surface), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_CYAN);
                curr_y += scaler.s(26.0);

                let surfaces = [
                    (SurfaceType::Sand, "Sand"),
                    (SurfaceType::Dirt, "Dirt"),
                    (SurfaceType::Water, "Water"),
                    (SurfaceType::Asphalt, "Asphalt"),
                    (SurfaceType::Grass, "Grass"),
                ];

                for (st, label) in surfaces {
                    if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(24.0), label, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                        state.record_undo();
                        state.track.geometry.surface_zones[idx].surface = st;
                    }
                    curr_y += scaler.s(28.0);
                }

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y + scaler.s(10.0), w - scaler.s(24.0), scaler.s(28.0), "DELETE ZONE [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                    tools.delete_selected(state);
                }
            }
        }
        Selection::Obstacle(idx) => {
            if idx < state.track.geometry.obstacles.len() {
                fonts.draw_ui_bold(&format!("Obstacle #{}", idx), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
                curr_y += scaler.s(24.0);

                let obs = &state.track.geometry.obstacles[idx];
                fonts.draw_ui_regular(&format!("Label: {}", obs.name), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);
                curr_y += scaler.s(32.0);

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DELETE OBSTACLE [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                    tools.delete_selected(state);
                }
            }
        }
        Selection::JumpRamp(idx) => {
            if idx < state.track.geometry.jump_ramps.len() {
                fonts.draw_ui_bold(&format!("Jump Ramp #{}", idx), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
                curr_y += scaler.s(24.0);

                let ramp = &state.track.geometry.jump_ramps[idx];
                fonts.draw_ui_regular(&format!("Height: {:.1}m | Angle: {:.0}°", ramp.height, ramp.ramp_angle_deg), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_GOLD);
                curr_y += scaler.s(32.0);

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DELETE RAMP [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                    tools.delete_selected(state);
                }
            }
        }
        Selection::Checkpoint(id) => {
            fonts.draw_ui_bold(&format!("Checkpoint Gate #{}", id), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
            curr_y += scaler.s(24.0);

            let cp_pos = state.track.checkpoints.iter().position(|c| c.id == id);
            if let Some(pos) = cp_pos {
                let is_finish = state.track.checkpoints[pos].is_finish_line;
                let finish_lbl = if is_finish { "[✓] Finish Line" } else { "[ ] Normal Sector" };
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(26.0), finish_lbl, Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.checkpoints[pos].is_finish_line = !is_finish;
                }
            }
            curr_y += scaler.s(32.0);

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DELETE GATE [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                tools.delete_selected(state);
            }
        }
        Selection::GridSlot(slot) => {
            fonts.draw_ui_bold(&format!("Starting Grid Slot #{}", slot), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
            curr_y += scaler.s(32.0);

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DELETE SLOT [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                tools.delete_selected(state);
            }
        }
        Selection::PitBox => {
            fonts.draw_ui_bold("Pit Lane Box", x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
            curr_y += scaler.s(24.0);

            let has_pit = state.track.pit_box_area.is_some();
            fonts.draw_ui_regular(&format!("Configured: {}", has_pit), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_CYAN);
            curr_y += scaler.s(32.0);

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "CLEAR PIT LANE [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                tools.delete_selected(state);
            }
        }
        Selection::None => {
            fonts.draw_ui_bold("Circuit Overview", x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
            curr_y += scaler.s(26.0);

            fonts.draw_ui_regular(&format!("Surface: {:?}", state.track.default_surface), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);
            curr_y += scaler.s(26.0);

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(26.0), "Auto Checkpoints", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                state.record_undo();
                state.track.auto_generate_checkpoints(8, 3);
            }
            curr_y += scaler.s(32.0);

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(26.0), "Auto Grid Slots", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                state.record_undo();
                state.track.auto_generate_grid(8, 8.0, 3.0);
            }
            curr_y += scaler.s(32.0);

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(26.0), "Rebuild Geometry", Palette::UI_CARD_BG, Palette::NEON_GOLD, mouse_pos, clicked) {
                state.record_undo();
                state.track.rebuild_geometry(2.5, tdrace_core::track::geometry::BarrierType::Armco);
            }
        }
    }
}

/// Renders starter template modal overlay.
fn render_template_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    mouse_pos: Vec2,
    clicked: bool,
) -> Option<EditorAction> {
    let mw = scaler.s(520.0);
    let mh = scaler.s(340.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.0);

    fonts.draw_display_centered("START NEW CIRCUIT", sw * 0.5, my + scaler.s(32.0), scaler.font_s(22.0), Palette::NEON_GOLD);
    fonts.draw_ui_regular_centered("Select a starter layout preset or blank canvas", sw * 0.5, my + scaler.s(52.0), scaler.font_s(13.0), Palette::UI_TEXT_MUTED);

    let templates = [
        ("Blank Circuit", "Empty 500x500m grass canvas with starting waypoint loop"),
        ("Classic Grand Prix", "Full asphalt GP layout with chicanes, sand traps, and pit lane"),
        ("Oval Speedway", "High-speed banked superspeedway"),
        ("Oasis Rally", "Desert dirt circuit with water hazards and sand dunes"),
    ];

    let mut ty = my + scaler.s(72.0);
    for (title, desc) in templates {
        let btn_w = mw - scaler.s(40.0);
        let btn_h = scaler.s(50.0);
        let bx = mx + scaler.s(20.0);

        let is_hover = mouse_pos.x >= bx && mouse_pos.x <= bx + btn_w && mouse_pos.y >= ty && mouse_pos.y <= ty + btn_h;
        let bg_col = if is_hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG };

        scaler.draw_glass_card(bx, ty, btn_w, btn_h, bg_col, if is_hover { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER }, 1.2);
        fonts.draw_ui_bold(title, bx + scaler.s(16.0), ty + scaler.s(22.0), scaler.font_s(15.0), Palette::WHITE);
        fonts.draw_ui_regular(desc, bx + scaler.s(16.0), ty + scaler.s(38.0), scaler.font_s(11.5), Palette::UI_TEXT_MUTED);

        if is_hover && clicked {
            return Some(EditorAction::NewFromTemplate(title.to_string()));
        }
        ty += scaler.s(58.0);
    }

    None
}

/// Renders Save As modal overlay with text input.
fn render_save_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    input_name: &mut String,
    mouse_pos: Vec2,
    clicked: bool,
) -> Option<EditorAction> {
    let mw = scaler.s(440.0);
    let mh = scaler.s(220.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::NEON_GREEN, 2.0);

    fonts.draw_display_centered("SAVE CIRCUIT", sw * 0.5, my + scaler.s(32.0), scaler.font_s(22.0), Palette::NEON_GOLD);

    // Typing input handling
    while let Some(c) = get_char_pressed() {
        if !c.is_control() && input_name.len() < 32 {
            input_name.push(c);
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        input_name.pop();
    }

    // Input box
    let inp_w = mw - scaler.s(40.0);
    let inp_h = scaler.s(40.0);
    let inp_x = mx + scaler.s(20.0);
    let inp_y = my + scaler.s(60.0);

    draw_rectangle(inp_x, inp_y, inp_w, inp_h, Color::new(0.04, 0.05, 0.08, 0.95));
    draw_rectangle_lines(inp_x, inp_y, inp_w, inp_h, 1.5, Palette::NEON_CYAN);

    fonts.draw_ui_bold(
        &format!("{}_", input_name),
        inp_x + scaler.s(12.0),
        inp_y + scaler.s(26.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );

    let save_clicked = draw_ui_btn(
        fonts,
        scaler,
        inp_x,
        my + scaler.s(130.0),
        inp_w,
        scaler.s(42.0),
        "SAVE TO DISK [Enter]",
        Color::new(0.12, 0.65, 0.32, 0.95),
        Palette::NEON_GREEN,
        mouse_pos,
        clicked,
    );

    if save_clicked || is_key_pressed(KeyCode::Enter) {
        return Some(EditorAction::SaveTrack(input_name.clone()));
    }

    None
}

/// Renders open track modal.
fn render_open_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    track_manager: &TrackManager,
    mouse_pos: Vec2,
    clicked: bool,
) -> Option<EditorAction> {
    let mw = scaler.s(520.0);
    let mh = scaler.s(400.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.0);

    fonts.draw_display_centered("OPEN CIRCUIT", sw * 0.5, my + scaler.s(32.0), scaler.font_s(22.0), Palette::NEON_GOLD);

    let mut ty = my + scaler.s(60.0);
    let choices = track_manager.all_track_choices();

    for choice in choices.iter().take(6) {
        let btn_w = mw - scaler.s(40.0);
        let btn_h = scaler.s(46.0);
        let bx = mx + scaler.s(20.0);

        let is_hover = mouse_pos.x >= bx && mouse_pos.x <= bx + btn_w && mouse_pos.y >= ty && mouse_pos.y <= ty + btn_h;
        let bg_col = if is_hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG };

        scaler.draw_glass_card(bx, ty, btn_w, btn_h, bg_col, if is_hover { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER }, 1.2);
        fonts.draw_ui_bold(choice.title(), bx + scaler.s(16.0), ty + scaler.s(22.0), scaler.font_s(14.0), Palette::WHITE);
        fonts.draw_ui_regular(choice.tag(), bx + scaler.s(16.0), ty + scaler.s(36.0), scaler.font_s(11.0), Palette::NEON_CYAN);

        if is_hover && clicked {
            return Some(EditorAction::OpenTrack(choice.track_id().to_string()));
        }
        ty += scaler.s(52.0);
    }

    None
}

/// Renders circuit diagnostics modal with actionable issues list.
fn render_diagnostics_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    state: &EditorState,
    mouse_pos: Vec2,
    clicked: bool,
) -> bool {
    let mw = scaler.s(560.0);
    let mh = scaler.s(380.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::YELLOW, 2.0);

    let val = validate_track(&state.track);
    let is_valid = val.iter().all(|e| e.severity != ValidationSeverity::Error);

    fonts.draw_display_centered("CIRCUIT DIAGNOSTICS", sw * 0.5, my + scaler.s(32.0), scaler.font_s(22.0), Palette::NEON_GOLD);

    let status_str = if is_valid {
        "✓ All checks passed! Circuit is 100% race ready."
    } else {
        "! Issues found that prevent championship race qualification."
    };
    let status_col = if is_valid { Palette::NEON_GREEN } else { Palette::RED };

    fonts.draw_ui_bold_centered(status_str, sw * 0.5, my + scaler.s(58.0), scaler.font_s(14.0), status_col);

    let mut ly = my + scaler.s(84.0);

    for err in val.iter().filter(|e| e.severity == ValidationSeverity::Error).take(6) {
        fonts.draw_ui_bold(&format!("• [ERROR] {}", err.message), mx + scaler.s(24.0), ly, scaler.font_s(12.5), Palette::RED);
        ly += scaler.s(22.0);
    }

    for warn in val.iter().filter(|e| e.severity == ValidationSeverity::Warning).take(4) {
        fonts.draw_ui_regular(&format!("• [WARN] {}", warn.message), mx + scaler.s(24.0), ly, scaler.font_s(12.0), Palette::YELLOW);
        ly += scaler.s(20.0);
    }

    draw_ui_btn(fonts, scaler, mx + (mw - scaler.s(160.0)) * 0.5, my + mh - scaler.s(48.0), scaler.s(160.0), scaler.s(34.0), "CLOSE [Esc]", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked)
}

/// Renders Help & Keyboard Shortcuts overlay.
fn render_help_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    mouse_pos: Vec2,
    clicked: bool,
) -> bool {
    let mw = scaler.s(580.0);
    let mh = scaler.s(450.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.0);

    fonts.draw_display_centered("EDITOR CONTROLS & SHORTCUTS", sw * 0.5, my + scaler.s(32.0), scaler.font_s(22.0), Palette::NEON_GOLD);

    let shortcuts = [
        ("Tools 1-8", "Switch between Select, Spline, Surface, Ramp, Obstacle, Checkpoint, Grid, Pit"),
        ("Left Click", "Place entity / Select / Drag handles / Draw surface boxes"),
        ("Middle / Right Drag", "Pan editor camera across the circuit canvas"),
        ("Mouse Scroll Wheel", "Zoom in / Zoom out centered on cursor position"),
        ("Tab Key", "Cycle zoom levels (Close, Medium, Far, Overview)"),
        ("Space / Enter", "Instant Test Drive (Race car directly from starting grid)"),
        ("Ctrl + Z / Ctrl + Y", "Undo / Redo state modifications"),
        ("Delete / Backspace", "Delete selected waypoint, surface zone, ramp, or prop"),
        ("F Key", "Focus and frame the entire circuit bounds within viewport"),
        ("G Key", "Cycle CAD metric grid snap (Off, 1m, 2.5m, 5m, 10m)"),
    ];

    let mut sy = my + scaler.s(68.0);
    for (key, desc) in shortcuts {
        fonts.draw_ui_bold(key, mx + scaler.s(24.0), sy, scaler.font_s(13.0), Palette::NEON_CYAN);
        fonts.draw_ui_regular(desc, mx + scaler.s(180.0), sy, scaler.font_s(12.5), Palette::WHITE);
        sy += scaler.s(28.0);
    }

    draw_ui_btn(fonts, scaler, mx + (mw - scaler.s(160.0)) * 0.5, my + mh - scaler.s(48.0), scaler.s(160.0), scaler.s(34.0), "CLOSE [Esc]", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked)
}

/// Helper function to draw a clickable UI button with hover feedback.
fn draw_ui_btn(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    bg: Color,
    border: Color,
    mouse_pos: Vec2,
    clicked: bool,
) -> bool {
    let is_hover = mouse_pos.x >= x && mouse_pos.x <= x + w && mouse_pos.y >= y && mouse_pos.y <= y + h;

    let final_bg = if is_hover {
        Color::new((bg.r * 1.3).min(1.0), (bg.g * 1.3).min(1.0), (bg.b * 1.3).min(1.0), bg.a)
    } else {
        bg
    };

    draw_rectangle(x, y, w, h, final_bg);
    draw_rectangle_lines(x, y, w, h, if is_hover { 2.0 } else { 1.0 }, border);

    fonts.draw_ui_bold_centered(
        label,
        x + w * 0.5,
        y + h * 0.5 + scaler.s(5.0),
        scaler.font_s(12.0),
        if is_hover { Palette::WHITE } else { Color::new(0.88, 0.92, 0.98, 1.0) },
    );

    is_hover && clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_modals_state_transitions() {
        let mut modal = EditorModal::None;
        assert_eq!(modal, EditorModal::None);

        modal = EditorModal::Templates;
        assert_eq!(modal, EditorModal::Templates);

        modal = EditorModal::SaveAs {
            input_name: "My Custom Circuit".to_string(),
        };
        if let EditorModal::SaveAs { input_name } = &modal {
            assert_eq!(input_name, "My Custom Circuit");
        } else {
            panic!("Expected SaveAs modal");
        }

        modal = EditorModal::Diagnostics;
        assert_eq!(modal, EditorModal::Diagnostics);

        modal = EditorModal::Help;
        assert_eq!(modal, EditorModal::Help);
    }

    #[test]
    fn test_editor_actions_variants() {
        let act = EditorAction::SetTool(EditorToolType::RoadSpline);
        assert_eq!(act, EditorAction::SetTool(EditorToolType::RoadSpline));

        let act_snap = EditorAction::SetSnap(GridSnapSetting::Snap5m);
        assert_eq!(act_snap, EditorAction::SetSnap(GridSnapSetting::Snap5m));

        let act_save = EditorAction::SaveTrack("monaco_gp".to_string());
        assert_eq!(act_save, EditorAction::SaveTrack("monaco_gp".to_string()));
    }
}
