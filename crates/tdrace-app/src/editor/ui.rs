use glam::Vec2;
use macroquad::color::Color;
use macroquad::input::{
    get_char_pressed, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed,
    mouse_position, KeyCode, MouseButton,
};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use macroquad::window::{screen_height, screen_width};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{JumpRamp, SurfaceLayer, SurfaceShape};
use tdrace_core::track::validation::{validate_track, ValidationSeverity};

use crate::editor::camera::EditorCamera;
use crate::editor::state::{EditorState, GridSnapSetting, Selection};
use crate::editor::tools::{EditorToolType, SurfaceShapeType, ToolSettings};
use crate::render::color::Palette;
use crate::track_manager::TrackManager;
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;

/// Dimension / orientation properties of a jump ramp that can be edited in a modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RampPropertyModal {
    Angle,
    Length,
    Width,
    Height,
    Pitch,
    LaunchSpeed,
}

/// Modals that can be displayed as overlays on top of the editor viewport.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorModal {
    None,
    Templates,
    SaveAs {
        input_name: String,
        input_filename: String,
        input_description: String,
        active_field: usize,
        overwrite: bool,
        custom_filename_edited: bool,
        exit_on_save: bool,
    },
    OpenTrack {
        selected_tab: usize,
        page: usize,
    },
    Diagnostics,
    Help,
    UnsavedChanges,
    SetRampAngle {
        input_angle: String,
    },
    SetRampProperty {
        property: RampPropertyModal,
        input_val: String,
    },
}

/// Actions dispatched from editor UI interactions.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorAction {
    None,
    SetTool(EditorToolType),
    SetSnap(GridSnapSetting),
    CycleZoom,
    NewFromTemplate(String),
    SaveTrack {
        name: String,
        filename: String,
        description: String,
        overwrite: bool,
        exit_after: bool,
    },
    OpenTrack(crate::ui::menu::TrackChoice),
    DeleteTrack(String),
    Validate,
    StartTestDrive,
    FocusCamera,
    ToggleHelp,
    ExitToMenu,
}

/// Helper to drain any unconsumed characters from macroquad input buffer.
fn drain_char_queue() {
    while std::panic::catch_unwind(get_char_pressed).unwrap_or(None).is_some() {}
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

    // Drain accumulated characters whenever no text-input modal is active
    if !matches!(*active_modal, EditorModal::SaveAs { .. } | EditorModal::SetRampAngle { .. }) {
        drain_char_queue();
    }

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

    // Track Name & Loaded Source File label
    let file_tag = if let Some(path) = &state.current_file_path {
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(path.as_str());
        format!("Track: {} [{}]", state.track.name, filename)
    } else {
        format!("Track: {}", state.track.name)
    };
    fonts.draw_ui_bold(
        &file_tag,
        tb_x,
        scaler.s(26.0),
        scaler.font_s(14.0),
        Palette::WHITE,
    );
    tb_x += scaler.s(200.0);

    // Top action buttons
    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(65.0), scaler.s(30.0), "NEW", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, bg_mouse_clicked) {
        *active_modal = EditorModal::Templates;
    }
    tb_x += scaler.s(72.0);

    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(65.0), scaler.s(30.0), "OPEN", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, bg_mouse_clicked) {
        let _ = track_manager.scan_custom_tracks();
        *active_modal = EditorModal::OpenTrack {
            selected_tab: 0,
            page: 0,
        };
    }
    tb_x += scaler.s(72.0);

    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(65.0), scaler.s(30.0), "SAVE", Palette::UI_CARD_BG, Palette::NEON_GREEN, mouse_pos, bg_mouse_clicked) {
        drain_char_queue();
        let is_existing = state.current_file_path.is_some();
        let initial_filename = if let Some(ref p) = state.current_file_path {
            std::path::Path::new(p)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        } else {
            TrackManager::sanitize_slug(&state.track.name)
        };
        *active_modal = EditorModal::SaveAs {
            input_name: state.track.name.clone(),
            input_filename: initial_filename,
            input_description: state.track.description.clone(),
            active_field: 0,
            overwrite: is_existing,
            custom_filename_edited: is_existing,
            exit_on_save: false,
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
    let exit_w = scaler.s(85.0);
    let td_x = sw - test_drive_w - exit_w - scaler.s(24.0);

    if draw_ui_btn(fonts, &scaler, td_x, scaler.s(8.0), test_drive_w, scaler.s(30.0), "TEST DRIVE [Space]", Color::new(0.12, 0.65, 0.32, 0.95), Palette::NEON_GREEN, mouse_pos, bg_mouse_clicked) {
        dispatched_action = EditorAction::StartTestDrive;
    }

    if draw_ui_btn(fonts, &scaler, sw - exit_w - scaler.s(12.0), scaler.s(8.0), exit_w, scaler.s(30.0), "EXIT [Esc]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, bg_mouse_clicked) {
        if state.is_dirty {
            *active_modal = EditorModal::UnsavedChanges;
        } else {
            dispatched_action = EditorAction::ExitToMenu;
        }
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

    // 2b. Surface Zone Active Sub-Palette (when Surface Zone tool is active)
    if tools.active_tool == EditorToolType::SurfaceZone {
        let sub_h = scaler.s(180.0);
        let sub_y = tool_y + tool_h + scaler.s(8.0);
        scaler.draw_glass_card(scaler.s(12.0), sub_y, tool_w, sub_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

        fonts.draw_ui_bold("SHAPES", scaler.s(22.0), sub_y + scaler.s(16.0), scaler.font_s(11.5), Palette::NEON_CYAN);

        let shape_buttons = [
            (SurfaceShapeType::Square, "Square [Ctrl+S]"),
            (SurfaceShapeType::Circle, "Circle [Ctrl+C]"),
            (SurfaceShapeType::Triangle, "Triangle [Ctrl+T]"),
            (SurfaceShapeType::Polygon, "Polygon [Ctrl+P]"),
        ];

        let mut btn_y = sub_y + scaler.s(22.0);
        for (st, label) in shape_buttons {
            let is_active = tools.active_surface_shape == st;
            let bg_col = if is_active { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG };
            let border_col = if is_active { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };
            if draw_ui_btn(fonts, &scaler, scaler.s(20.0), btn_y, tool_w - scaler.s(16.0), scaler.s(24.0), label, bg_col, border_col, mouse_pos, bg_mouse_clicked) {
                tools.active_surface_shape = st;
                tools.active_polygon_vertices.clear();
            }
            btn_y += scaler.s(27.0);
        }

        let is_front = tools.active_surface_layer == SurfaceLayer::AboveTrack;
        let layer_label = if is_front { "Layer: FRONT (Over)" } else { "Layer: BACK (Under)" };
        let layer_border = if is_front { Palette::NEON_GREEN } else { Palette::NEON_CYAN };
        if draw_ui_btn(fonts, &scaler, scaler.s(20.0), btn_y + scaler.s(4.0), tool_w - scaler.s(16.0), scaler.s(26.0), layer_label, Palette::UI_CARD_BG, layer_border, mouse_pos, bg_mouse_clicked) {
            tools.active_surface_layer = if is_front { SurfaceLayer::BelowTrack } else { SurfaceLayer::AboveTrack };
        }
    }

    // 3. RIGHT INSPECTOR PANEL (when entity or circuit is selected)
    let insp_w = scaler.s(240.0);
    let insp_x = sw - insp_w - scaler.s(12.0);
    let insp_y = top_h + scaler.s(12.0);
    let insp_h = sh - top_h - scaler.s(54.0);

    scaler.draw_glass_card(insp_x, insp_y, insp_w, insp_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

    render_inspector(fonts, &scaler, insp_x, insp_y, insp_w, insp_h, state, tools, mouse_pos, bg_mouse_clicked, active_modal);

    // 4. BOTTOM STATUS BAR
    let bot_h = scaler.s(32.0);
    let bot_y = sh - bot_h;
    scaler.draw_glass_card(0.0, bot_y, sw, bot_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.0);

    let world_mouse = camera.screen_to_world(mouse_pos, sw, sh);
    let total_len = state.track.spline.total_length();
    let val = validate_track(&state.track);
    let is_valid = val.iter().all(|e| e.severity != ValidationSeverity::Error);
    let val_str = if is_valid {
        "[OK] Circuit Valid"
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
    if is_modal_open {
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.78));
        match active_modal {
            EditorModal::Templates => {
                if let Some(action) = render_template_modal(fonts, &scaler, sw, sh, mouse_pos, mouse_clicked) {
                    dispatched_action = action;
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::SaveAs {
                input_name,
                input_filename,
                input_description,
                active_field,
                overwrite,
                custom_filename_edited,
                exit_on_save,
            } => {
                let exit_on_save_val = *exit_on_save;
                if let Some(action) = render_save_modal(
                    fonts,
                    &scaler,
                    sw,
                    sh,
                    input_name,
                    input_filename,
                    input_description,
                    active_field,
                    overwrite,
                    custom_filename_edited,
                    exit_on_save_val,
                    state.current_file_path.as_deref(),
                    track_manager,
                    mouse_pos,
                    mouse_clicked,
                ) {
                    dispatched_action = action;
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::OpenTrack { selected_tab, page } => {
                if let Some(action) = render_open_modal(
                    fonts,
                    &scaler,
                    sw,
                    sh,
                    selected_tab,
                    page,
                    track_manager,
                    mouse_pos,
                    mouse_clicked,
                ) {
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
            EditorModal::UnsavedChanges => {
                if let Some(action) = render_unsaved_changes_modal(
                    fonts,
                    &scaler,
                    sw,
                    sh,
                    state,
                    active_modal,
                    mouse_pos,
                    mouse_clicked,
                ) {
                    dispatched_action = action;
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::SetRampAngle { input_angle } => {
                if render_set_ramp_angle_modal(
                    fonts,
                    &scaler,
                    sw,
                    sh,
                    state,
                    tools,
                    input_angle,
                    mouse_pos,
                    mouse_clicked,
                ) {
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::SetRampProperty { property, input_val } => {
                if render_set_ramp_property_modal(
                    fonts,
                    &scaler,
                    sw,
                    sh,
                    state,
                    tools,
                    *property,
                    input_val,
                    mouse_pos,
                    mouse_clicked,
                ) {
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::None => {}
        }

        if is_key_pressed(KeyCode::Escape) && *active_modal != EditorModal::None && *active_modal != EditorModal::UnsavedChanges {
            *active_modal = EditorModal::None;
        }
    } else {
        if is_key_pressed(KeyCode::Escape) && dispatched_action == EditorAction::None {
            if state.is_dirty {
                *active_modal = EditorModal::UnsavedChanges;
            } else {
                dispatched_action = EditorAction::ExitToMenu;
            }
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
    active_modal: &mut EditorModal,
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
                fonts.draw_ui_bold("Road Width:", x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_CYAN);
                if draw_ui_btn(fonts, scaler, x + scaler.s(120.0), curr_y, scaler.s(45.0), scaler.s(22.0), "-1m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].width = (road_w - 1.0).max(4.0);
                    tools.new_waypoint_width = state.track.spline.waypoints[idx].width;
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(170.0), curr_y, scaler.s(45.0), scaler.s(22.0), "+1m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].width = (road_w + 1.0).min(30.0);
                    tools.new_waypoint_width = state.track.spline.waypoints[idx].width;
                    state.rebuild_geometry();
                }
                curr_y += scaler.s(26.0);

                let wid_pct = ((road_w - 4.0) / 26.0).clamp(0.0, 1.0);
                let (drag_w_pct, _) = draw_slider_with_input_field(
                    fonts,
                    scaler,
                    x + scaler.s(12.0),
                    curr_y,
                    w - scaler.s(24.0),
                    scaler.s(20.0),
                    wid_pct,
                    &format!("{:.1}m", road_w),
                    false,
                    mouse_pos,
                    clicked,
                );
                if let Some(p) = drag_w_pct {
                    state.record_undo();
                    state.track.spline.waypoints[idx].width = (4.0 + p * 26.0).clamp(4.0, 30.0);
                    tools.new_waypoint_width = state.track.spline.waypoints[idx].width;
                    state.rebuild_geometry();
                }
                curr_y += scaler.s(26.0);

                // Banking controls
                let bank_deg = state.track.spline.waypoints[idx].bank_angle;
                fonts.draw_ui_bold("Banking (Superelevation):", x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_GOLD);
                curr_y += scaler.s(20.0);

                let bank_pct = ((bank_deg + 45.0) / 90.0).clamp(0.0, 1.0);
                let (drag_b_pct, _) = draw_slider_with_input_field(
                    fonts,
                    scaler,
                    x + scaler.s(12.0),
                    curr_y,
                    w - scaler.s(24.0),
                    scaler.s(20.0),
                    bank_pct,
                    &format!("{:+.1}°", bank_deg),
                    true,
                    mouse_pos,
                    clicked,
                );
                if let Some(p) = drag_b_pct {
                    state.record_undo();
                    state.track.spline.waypoints[idx].bank_angle = (-45.0 + p * 90.0).clamp(-45.0, 45.0);
                    tools.new_waypoint_bank_angle = state.track.spline.waypoints[idx].bank_angle;
                    state.rebuild_geometry();
                }
                curr_y += scaler.s(24.0);

                let btn_q_w = (w - scaler.s(36.0)) * 0.25;
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_q_w, scaler.s(22.0), "-5°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].bank_angle = (bank_deg - 5.0).clamp(-45.0, 45.0);
                    tools.new_waypoint_bank_angle = state.track.spline.waypoints[idx].bank_angle;
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_q_w, curr_y, btn_q_w, scaler.s(22.0), "-1°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].bank_angle = (bank_deg - 1.0).clamp(-45.0, 45.0);
                    tools.new_waypoint_bank_angle = state.track.spline.waypoints[idx].bank_angle;
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_q_w * 2.0, curr_y, btn_q_w, scaler.s(22.0), "+1°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].bank_angle = (bank_deg + 1.0).clamp(-45.0, 45.0);
                    tools.new_waypoint_bank_angle = state.track.spline.waypoints[idx].bank_angle;
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_q_w * 3.0, curr_y, btn_q_w, scaler.s(22.0), "+5°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].bank_angle = (bank_deg + 5.0).clamp(-45.0, 45.0);
                    tools.new_waypoint_bank_angle = state.track.spline.waypoints[idx].bank_angle;
                    state.rebuild_geometry();
                }
                curr_y += scaler.s(26.0);

                let btn_p_w = (w - scaler.s(36.0)) * 0.20;
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_p_w, scaler.s(20.0), "0°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].bank_angle = 0.0;
                    tools.new_waypoint_bank_angle = 0.0;
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_p_w, curr_y, btn_p_w, scaler.s(20.0), "10°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].bank_angle = 10.0;
                    tools.new_waypoint_bank_angle = 10.0;
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_p_w * 2.0, curr_y, btn_p_w, scaler.s(20.0), "18°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].bank_angle = 18.0;
                    tools.new_waypoint_bank_angle = 18.0;
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_p_w * 3.0, curr_y, btn_p_w, scaler.s(20.0), "22°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].bank_angle = 22.0;
                    tools.new_waypoint_bank_angle = 22.0;
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(20.0) + btn_p_w * 4.0, curr_y, btn_p_w, scaler.s(20.0), "+/-", Palette::UI_CARD_BG, Palette::NEON_GOLD, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].bank_angle = -state.track.spline.waypoints[idx].bank_angle;
                    tools.new_waypoint_bank_angle = state.track.spline.waypoints[idx].bank_angle;
                    state.rebuild_geometry();
                }
                curr_y += scaler.s(28.0);

                // Curbs toggles
                let lc = state.track.spline.waypoints[idx].left_curb;
                let rc = state.track.spline.waypoints[idx].right_curb;
                let half_btn_w = (w - scaler.s(30.0)) * 0.5;
                let lc_lbl = if lc { "[X] L Curb" } else { "[ ] L Curb" };
                let rc_lbl = if rc { "[X] R Curb" } else { "[ ] R Curb" };
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, half_btn_w, scaler.s(22.0), lc_lbl, if lc { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG }, if lc { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER }, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].left_curb = !lc;
                    tools.new_waypoint_left_curb = !lc;
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0) + half_btn_w + scaler.s(6.0), curr_y, half_btn_w, scaler.s(22.0), rc_lbl, if rc { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG }, if rc { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER }, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].right_curb = !rc;
                    tools.new_waypoint_right_curb = !rc;
                    state.rebuild_geometry();
                }
                curr_y += scaler.s(26.0);

                // Walls toggles
                let lw = state.track.spline.waypoints[idx].left_wall;
                let rw = state.track.spline.waypoints[idx].right_wall;
                let lw_lbl = if lw { "[X] L Wall" } else { "[ ] L Wall" };
                let rw_lbl = if rw { "[X] R Wall" } else { "[ ] R Wall" };
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, half_btn_w, scaler.s(22.0), lw_lbl, if lw { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG }, if lw { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER }, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].left_wall = !lw;
                    tools.new_waypoint_left_wall = !lw;
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0) + half_btn_w + scaler.s(6.0), curr_y, half_btn_w, scaler.s(22.0), rw_lbl, if rw { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG }, if rw { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER }, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].right_wall = !rw;
                    tools.new_waypoint_right_wall = !rw;
                    state.rebuild_geometry();
                }
                curr_y += scaler.s(28.0);

                // Wall Distance (Separation from track edge)
                let wp_l_dist = state.track.spline.waypoints[idx].left_wall_distance;
                let wp_r_dist = state.track.spline.waypoints[idx].right_wall_distance;
                let effective_dist = wp_l_dist.or(wp_r_dist).unwrap_or(state.barrier_offset);

                let dist_label = match (wp_l_dist, wp_r_dist) {
                    (Some(l), Some(r)) if (l - r).abs() < 1e-3 => {
                        if l <= 0.01 {
                            "Wall Dist: 0.0m (Flush)".to_string()
                        } else {
                            format!("Wall Dist: {:.1}m", l)
                        }
                    }
                    (Some(l), Some(r)) => format!("Wall Dist: L {:.1}m / R {:.1}m", l, r),
                    (Some(l), None) => format!("Wall Dist: L {:.1}m / R Def ({:.1}m)", l, state.barrier_offset),
                    (None, Some(r)) => format!("Wall Dist: L Def ({:.1}m) / R {:.1}m", state.barrier_offset, r),
                    (None, None) => format!("Wall Dist: Default ({:.1}m)", state.barrier_offset),
                };

                fonts.draw_ui_bold(&dist_label, x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_CYAN);
                curr_y += scaler.s(20.0);

                let dist_pct = (effective_dist / 15.0).clamp(0.0, 1.0);
                let (drag_d_pct, _) = draw_slider_with_input_field(
                    fonts,
                    scaler,
                    x + scaler.s(12.0),
                    curr_y,
                    w - scaler.s(24.0),
                    scaler.s(20.0),
                    dist_pct,
                    &format!("{:.1}m", effective_dist),
                    false,
                    mouse_pos,
                    clicked,
                );
                if let Some(p) = drag_d_pct {
                    state.record_undo();
                    let val = (p * 15.0).clamp(0.0, 20.0);
                    state.track.spline.waypoints[idx].left_wall_distance = Some(val);
                    state.track.spline.waypoints[idx].right_wall_distance = Some(val);
                    tools.new_waypoint_left_wall_distance = Some(val);
                    tools.new_waypoint_right_wall_distance = Some(val);
                    state.rebuild_geometry();
                }
                curr_y += scaler.s(24.0);

                let btn_q_w = (w - scaler.s(36.0)) * 0.25;
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_q_w, scaler.s(22.0), "-1m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    let cur = state.track.spline.waypoints[idx].left_wall_distance.unwrap_or(state.barrier_offset);
                    let new_val = (cur - 1.0).max(0.0);
                    state.track.spline.waypoints[idx].left_wall_distance = Some(new_val);
                    state.track.spline.waypoints[idx].right_wall_distance = Some(new_val);
                    tools.new_waypoint_left_wall_distance = Some(new_val);
                    tools.new_waypoint_right_wall_distance = Some(new_val);
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_q_w, curr_y, btn_q_w, scaler.s(22.0), "-0.5m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    let cur = state.track.spline.waypoints[idx].left_wall_distance.unwrap_or(state.barrier_offset);
                    let new_val = (cur - 0.5).max(0.0);
                    state.track.spline.waypoints[idx].left_wall_distance = Some(new_val);
                    state.track.spline.waypoints[idx].right_wall_distance = Some(new_val);
                    tools.new_waypoint_left_wall_distance = Some(new_val);
                    tools.new_waypoint_right_wall_distance = Some(new_val);
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_q_w * 2.0, curr_y, btn_q_w, scaler.s(22.0), "+0.5m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    let cur = state.track.spline.waypoints[idx].left_wall_distance.unwrap_or(state.barrier_offset);
                    let new_val = (cur + 0.5).min(20.0);
                    state.track.spline.waypoints[idx].left_wall_distance = Some(new_val);
                    state.track.spline.waypoints[idx].right_wall_distance = Some(new_val);
                    tools.new_waypoint_left_wall_distance = Some(new_val);
                    tools.new_waypoint_right_wall_distance = Some(new_val);
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_q_w * 3.0, curr_y, btn_q_w, scaler.s(22.0), "+1m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    let cur = state.track.spline.waypoints[idx].left_wall_distance.unwrap_or(state.barrier_offset);
                    let new_val = (cur + 1.0).min(20.0);
                    state.track.spline.waypoints[idx].left_wall_distance = Some(new_val);
                    state.track.spline.waypoints[idx].right_wall_distance = Some(new_val);
                    tools.new_waypoint_left_wall_distance = Some(new_val);
                    tools.new_waypoint_right_wall_distance = Some(new_val);
                    state.rebuild_geometry();
                }
                curr_y += scaler.s(26.0);

                let btn_p_w = (w - scaler.s(36.0)) * 0.25;
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_p_w, scaler.s(20.0), "0m Flush", Palette::UI_CARD_BG, Palette::NEON_GOLD, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].left_wall_distance = Some(0.0);
                    state.track.spline.waypoints[idx].right_wall_distance = Some(0.0);
                    tools.new_waypoint_left_wall_distance = Some(0.0);
                    tools.new_waypoint_right_wall_distance = Some(0.0);
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_p_w, curr_y, btn_p_w, scaler.s(20.0), "1.5m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].left_wall_distance = Some(1.5);
                    state.track.spline.waypoints[idx].right_wall_distance = Some(1.5);
                    tools.new_waypoint_left_wall_distance = Some(1.5);
                    tools.new_waypoint_right_wall_distance = Some(1.5);
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_p_w * 2.0, curr_y, btn_p_w, scaler.s(20.0), "4.0m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].left_wall_distance = Some(4.0);
                    state.track.spline.waypoints[idx].right_wall_distance = Some(4.0);
                    tools.new_waypoint_left_wall_distance = Some(4.0);
                    tools.new_waypoint_right_wall_distance = Some(4.0);
                    state.rebuild_geometry();
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_p_w * 3.0, curr_y, btn_p_w, scaler.s(20.0), "Reset", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.spline.waypoints[idx].left_wall_distance = None;
                    state.track.spline.waypoints[idx].right_wall_distance = None;
                    tools.new_waypoint_left_wall_distance = None;
                    tools.new_waypoint_right_wall_distance = None;
                    state.rebuild_geometry();
                }
                curr_y += scaler.s(28.0);

                // Surface selector
                let current_surf = state.track.spline.waypoints[idx].surface.unwrap_or(SurfaceType::Asphalt);
                fonts.draw_ui_bold(&format!("Surface: {}", current_surf.name()), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_CYAN);
                curr_y += scaler.s(20.0);

                let surfaces = [
                    (SurfaceType::Asphalt, "Asphalt"),
                    (SurfaceType::Dirt, "Dirt"),
                    (SurfaceType::Sand, "Sand"),
                    (SurfaceType::Grass, "Grass"),
                    (SurfaceType::Ice, "Ice"),
                    (SurfaceType::Water, "Water"),
                ];

                for chunk in surfaces.chunks(2) {
                    let (st1, label1) = chunk[0];
                    let is_active1 = current_surf == st1;
                    if draw_ui_btn(
                        fonts,
                        scaler,
                        x + scaler.s(12.0),
                        curr_y,
                        half_btn_w,
                        scaler.s(22.0),
                        label1,
                        if is_active1 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                        if is_active1 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER },
                        mouse_pos,
                        clicked,
                    ) {
                        state.record_undo();
                        state.track.spline.waypoints[idx].surface = Some(st1);
                        tools.active_surface = st1;
                        state.rebuild_geometry();
                    }

                    if chunk.len() > 1 {
                        let (st2, label2) = chunk[1];
                        let is_active2 = current_surf == st2;
                        if draw_ui_btn(
                            fonts,
                            scaler,
                            x + scaler.s(12.0) + half_btn_w + scaler.s(6.0),
                            curr_y,
                            half_btn_w,
                            scaler.s(22.0),
                            label2,
                            if is_active2 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                            if is_active2 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER },
                            mouse_pos,
                            clicked,
                        ) {
                            state.record_undo();
                            state.track.spline.waypoints[idx].surface = Some(st2);
                            tools.active_surface = st2;
                            state.rebuild_geometry();
                        }
                    }
                    curr_y += scaler.s(26.0);
                }
                curr_y += scaler.s(6.0);

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DUPLICATE WAYPOINT [Ctrl+D]", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                    tools.duplicate_selected(state);
                }
                curr_y += scaler.s(32.0);

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DELETE WAYPOINT [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                    tools.delete_selected(state);
                }
            }
        }
        Selection::MultipleWaypoints(ref indices) => {
            let count = indices.len();
            fonts.draw_ui_bold(&format!("Selected Segments ({})", count), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
            curr_y += scaler.s(22.0);

            fonts.draw_ui_regular(&format!("Waypoints: {:?}", indices), x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
            curr_y += scaler.s(18.0);

            fonts.draw_ui_bold("BATCH WIDTH:", x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::NEON_CYAN);
            curr_y += scaler.s(18.0);
            let half_btn_w = (w - scaler.s(30.0)) * 0.5;
            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, half_btn_w, scaler.s(24.0), "-1m Width", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_adjust_width(state, -1.0);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + half_btn_w, curr_y, half_btn_w, scaler.s(24.0), "+1m Width", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_adjust_width(state, 1.0);
            }
            curr_y += scaler.s(30.0);

            fonts.draw_ui_bold("BATCH BANKING:", x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::NEON_GOLD);
            curr_y += scaler.s(18.0);
            let btn_q_w = (w - scaler.s(36.0)) * 0.25;
            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_q_w, scaler.s(22.0), "-5°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_adjust_banking(state, -5.0);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_q_w, curr_y, btn_q_w, scaler.s(22.0), "-1°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_adjust_banking(state, -1.0);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_q_w * 2.0, curr_y, btn_q_w, scaler.s(22.0), "+1°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_adjust_banking(state, 1.0);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_q_w * 3.0, curr_y, btn_q_w, scaler.s(22.0), "+5°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_adjust_banking(state, 5.0);
            }
            curr_y += scaler.s(26.0);

            let btn_p_w = (w - scaler.s(36.0)) * 0.20;
            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_p_w, scaler.s(20.0), "0°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_banking(state, 0.0);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_p_w, curr_y, btn_p_w, scaler.s(20.0), "10°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_banking(state, 10.0);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_p_w * 2.0, curr_y, btn_p_w, scaler.s(20.0), "18°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_banking(state, 18.0);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_p_w * 3.0, curr_y, btn_p_w, scaler.s(20.0), "22°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_banking(state, 22.0);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(20.0) + btn_p_w * 4.0, curr_y, btn_p_w, scaler.s(20.0), "+/-", Palette::UI_CARD_BG, Palette::NEON_GOLD, mouse_pos, clicked) {
                tools.batch_invert_banking(state);
            }
            curr_y += scaler.s(28.0);

            fonts.draw_ui_bold("BATCH CURBS:", x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::NEON_CYAN);
            curr_y += scaler.s(18.0);
            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, half_btn_w, scaler.s(24.0), "Both Curbs", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                tools.batch_set_curbs(state, true, true);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + half_btn_w, curr_y, half_btn_w, scaler.s(24.0), "No Curbs", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_curbs(state, false, false);
            }
            curr_y += scaler.s(30.0);

            fonts.draw_ui_bold("BATCH WALLS:", x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::NEON_CYAN);
            curr_y += scaler.s(18.0);
            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, half_btn_w, scaler.s(24.0), "Both Walls", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                tools.batch_set_walls(state, true, true);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + half_btn_w, curr_y, half_btn_w, scaler.s(24.0), "No Walls", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_walls(state, false, false);
            }
            curr_y += scaler.s(28.0);
            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, half_btn_w, scaler.s(24.0), "L Wall Only", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_walls(state, true, false);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + half_btn_w, curr_y, half_btn_w, scaler.s(24.0), "R Wall Only", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_walls(state, false, true);
            }
            curr_y += scaler.s(30.0);

            fonts.draw_ui_bold("BATCH WALL DISTANCE:", x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::NEON_CYAN);
            curr_y += scaler.s(18.0);
            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, half_btn_w, scaler.s(24.0), "-1m Dist", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_adjust_wall_distances(state, -1.0);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + half_btn_w, curr_y, half_btn_w, scaler.s(24.0), "+1m Dist", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_adjust_wall_distances(state, 1.0);
            }
            curr_y += scaler.s(28.0);

            let btn_q_w = (w - scaler.s(36.0)) * 0.25;
            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_q_w, scaler.s(22.0), "0m Flush", Palette::UI_CARD_BG, Palette::NEON_GOLD, mouse_pos, clicked) {
                tools.batch_set_wall_distances(state, Some(0.0), Some(0.0));
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_q_w, curr_y, btn_q_w, scaler.s(22.0), "1.5m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_wall_distances(state, Some(1.5), Some(1.5));
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_q_w * 2.0, curr_y, btn_q_w, scaler.s(22.0), "4.0m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_wall_distances(state, Some(4.0), Some(4.0));
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_q_w * 3.0, curr_y, btn_q_w, scaler.s(22.0), "Reset", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.batch_set_wall_distances(state, None, None);
            }
            curr_y += scaler.s(30.0);

            fonts.draw_ui_bold("BATCH SURFACE:", x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::NEON_CYAN);
            curr_y += scaler.s(18.0);

            let surfaces = [
                (SurfaceType::Asphalt, "Asphalt"),
                (SurfaceType::Dirt, "Dirt"),
                (SurfaceType::Sand, "Sand"),
                (SurfaceType::Grass, "Grass"),
                (SurfaceType::Ice, "Ice"),
            ];

            for (st, label) in surfaces {
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(22.0), label, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.batch_set_surface(state, Some(st));
                }
                curr_y += scaler.s(25.0);
            }

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y + scaler.s(6.0), w - scaler.s(24.0), scaler.s(28.0), "DUPLICATE ALL [Ctrl+D]", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                tools.duplicate_selected(state);
            }
            curr_y += scaler.s(36.0);

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DELETE ALL [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                tools.delete_selected(state);
            }
        }
        Selection::SurfaceZone(idx) => {
            if idx < state.track.geometry.surface_zones.len() {
                let (_zone_surface, _is_front, shape_name) = {
                    let zone = &state.track.geometry.surface_zones[idx];
                    let shape_name = match &zone.shape {
                        SurfaceShape::Circle { .. } => "Circle",
                        SurfaceShape::Aabb { .. } => "Square / Box",
                        SurfaceShape::OrientedBox { .. } => "Oriented Box",
                        SurfaceShape::Polygon { vertices } => {
                            if vertices.len() == 3 {
                                "Triangle"
                            } else {
                                "Polygon"
                            }
                        }
                    };
                    (zone.surface, zone.is_above_track(), shape_name)
                };

                fonts.draw_ui_bold(&format!("Surface Zone #{}", idx), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
                curr_y += scaler.s(22.0);

                fonts.draw_ui_regular(&format!("Shape: {}", shape_name), x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.5), Palette::UI_TEXT_MUTED);
                curr_y += scaler.s(18.0);

                fonts.draw_ui_bold("LAYER / DEPTH:", x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::NEON_CYAN);
                curr_y += scaler.s(24.0);

                let is_above = state.track.geometry.surface_zones[idx].is_above_track();
                let layer_text = if is_above { "Layer: FRONT (Above Track)" } else { "Layer: BACK (Below Track)" };
                fonts.draw_ui_bold(layer_text, x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::NEON_GOLD);
                curr_y += scaler.s(18.0);

                let half_btn_w = (w - scaler.s(30.0)) * 0.5;
                if draw_ui_btn(
                    fonts,
                    scaler,
                    x + scaler.s(12.0),
                    curr_y,
                    half_btn_w,
                    scaler.s(24.0),
                    "Bring Front",
                    if is_above { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                    if is_above { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER },
                    mouse_pos,
                    clicked,
                ) {
                    tools.bring_selected_surface_front(state);
                }
                if draw_ui_btn(
                    fonts,
                    scaler,
                    x + scaler.s(18.0) + half_btn_w,
                    curr_y,
                    half_btn_w,
                    scaler.s(24.0),
                    "Send Back",
                    if !is_above { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                    if !is_above { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER },
                    mouse_pos,
                    clicked,
                ) {
                    tools.send_selected_surface_back(state);
                }
                curr_y += scaler.s(30.0);

                fonts.draw_ui_bold("Zone Material:", x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::NEON_CYAN);
                curr_y += scaler.s(18.0);

                let surfaces = [
                    (SurfaceType::Dirt, "Dirt / Gravel Runoff"),
                    (SurfaceType::Sand, "Deep Sand Trap"),
                    (SurfaceType::Grass, "Grass Field"),
                    (SurfaceType::Water, "Water Puddle Hazard"),
                    (SurfaceType::Ice, "Ice Patch Slick"),
                    (SurfaceType::Asphalt, "Tarmac Extension"),
                ];

                let zone_surface = state.track.geometry.surface_zones[idx].surface;
                for (st, label) in surfaces {
                    let is_active = zone_surface == st;
                    let st_bg = if is_active { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG };
                    let st_border = if is_active { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };
                    if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(22.0), label, st_bg, st_border, mouse_pos, clicked) {
                        state.record_undo();
                        state.track.geometry.surface_zones[idx].surface = st;
                    }
                    curr_y += scaler.s(25.0);
                }
                curr_y += scaler.s(6.0);
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y + scaler.s(6.0), w - scaler.s(24.0), scaler.s(28.0), "DUPLICATE ZONE [Ctrl+D]", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                    tools.duplicate_selected(state);
                }
                curr_y += scaler.s(36.0);

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DELETE ZONE [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
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

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DUPLICATE OBSTACLE [Ctrl+D]", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                    tools.duplicate_selected(state);
                }
                curr_y += scaler.s(32.0);

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DELETE OBSTACLE [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                    tools.delete_selected(state);
                }
            }
        }
        Selection::JumpRamp(idx) => {
            if idx < state.track.geometry.jump_ramps.len() {
                let ramp = &state.track.geometry.jump_ramps[idx];
                fonts.draw_ui_bold(&format!("Jump Ramp #{}", idx), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
                curr_y += scaler.s(22.0);

                let row_w = w - scaler.s(24.0);
                let row_h = scaler.s(20.0);

                // 1. DIRECTION ANGLE (0° – 360°)
                let heading_deg = ramp.angle_deg();
                fonts.draw_ui_bold("Direction Angle:", x + scaler.s(12.0), curr_y + scaler.s(11.0), scaler.font_s(11.0), Palette::NEON_CYAN);
                curr_y += scaler.s(16.0);

                let angle_pct = (heading_deg % 360.0) / 360.0;
                let angle_str = format!("{:.1}°", heading_deg);
                let (drag_pct, input_click) = draw_slider_with_input_field(fonts, scaler, x + scaler.s(12.0), curr_y, row_w, row_h, angle_pct, &angle_str, false, mouse_pos, clicked);
                if let Some(new_pct) = drag_pct {
                    tools.set_selected_jump_ramp_angle_deg(state, new_pct * 360.0);
                }
                if input_click {
                    *active_modal = EditorModal::SetRampProperty {
                        property: RampPropertyModal::Angle,
                        input_val: format!("{:.1}", heading_deg),
                    };
                }
                curr_y += row_h + scaler.s(10.0);

                // 2. LENGTH (2.0m – 50.0m)
                let ramp = &state.track.geometry.jump_ramps[idx];
                let len = ramp.length();
                fonts.draw_ui_bold("Length:", x + scaler.s(12.0), curr_y + scaler.s(11.0), scaler.font_s(11.0), Palette::NEON_CYAN);
                curr_y += scaler.s(16.0);

                let len_pct = ((len - 2.0) / (50.0 - 2.0)).clamp(0.0, 1.0);
                let len_str = format!("{:.1}m", len);
                let (drag_pct, input_click) = draw_slider_with_input_field(fonts, scaler, x + scaler.s(12.0), curr_y, row_w, row_h, len_pct, &len_str, false, mouse_pos, clicked);
                if let Some(new_pct) = drag_pct {
                    let new_len = 2.0 + new_pct * (50.0 - 2.0);
                    tools.set_selected_jump_ramp_length(state, new_len);
                }
                if input_click {
                    *active_modal = EditorModal::SetRampProperty {
                        property: RampPropertyModal::Length,
                        input_val: format!("{:.1}", len),
                    };
                }
                curr_y += row_h + scaler.s(10.0);

                // 3. WIDTH (1.0m – 30.0m)
                let ramp = &state.track.geometry.jump_ramps[idx];
                let wid = ramp.width();
                fonts.draw_ui_bold("Width:", x + scaler.s(12.0), curr_y + scaler.s(11.0), scaler.font_s(11.0), Palette::NEON_CYAN);
                curr_y += scaler.s(16.0);

                let wid_pct = ((wid - 1.0) / (30.0 - 1.0)).clamp(0.0, 1.0);
                let wid_str = format!("{:.1}m", wid);
                let (drag_pct, input_click) = draw_slider_with_input_field(fonts, scaler, x + scaler.s(12.0), curr_y, row_w, row_h, wid_pct, &wid_str, false, mouse_pos, clicked);
                if let Some(new_pct) = drag_pct {
                    let new_wid = 1.0 + new_pct * (30.0 - 1.0);
                    tools.set_selected_jump_ramp_width(state, new_wid);
                }
                if input_click {
                    *active_modal = EditorModal::SetRampProperty {
                        property: RampPropertyModal::Width,
                        input_val: format!("{:.1}", wid),
                    };
                }
                curr_y += row_h + scaler.s(10.0);

                // 4. HEIGHT (0.2m – 10.0m)
                let ramp = &state.track.geometry.jump_ramps[idx];
                let h = ramp.height;
                fonts.draw_ui_bold("Height:", x + scaler.s(12.0), curr_y + scaler.s(11.0), scaler.font_s(11.0), Palette::NEON_GOLD);
                curr_y += scaler.s(16.0);

                let h_pct = ((h - 0.2) / (10.0 - 0.2)).clamp(0.0, 1.0);
                let h_str = format!("{:.1}m", h);
                let (drag_pct, input_click) = draw_slider_with_input_field(fonts, scaler, x + scaler.s(12.0), curr_y, row_w, row_h, h_pct, &h_str, true, mouse_pos, clicked);
                if let Some(new_pct) = drag_pct {
                    let new_h = 0.2 + new_pct * (10.0 - 0.2);
                    tools.set_selected_jump_ramp_height(state, new_h);
                }
                if input_click {
                    *active_modal = EditorModal::SetRampProperty {
                        property: RampPropertyModal::Height,
                        input_val: format!("{:.1}", h),
                    };
                }
                curr_y += row_h + scaler.s(10.0);

                // 5. PITCH (1.0° – 60.0°)
                let ramp = &state.track.geometry.jump_ramps[idx];
                let pitch = ramp.ramp_angle_deg;
                fonts.draw_ui_bold("Pitch Angle:", x + scaler.s(12.0), curr_y + scaler.s(11.0), scaler.font_s(11.0), Palette::NEON_GOLD);
                curr_y += scaler.s(16.0);

                let pitch_pct = ((pitch - 1.0) / (60.0 - 1.0)).clamp(0.0, 1.0);
                let pitch_str = format!("{:.1}°", pitch);
                let (drag_pct, input_click) = draw_slider_with_input_field(fonts, scaler, x + scaler.s(12.0), curr_y, row_w, row_h, pitch_pct, &pitch_str, true, mouse_pos, clicked);
                if let Some(new_pct) = drag_pct {
                    let new_pitch = 1.0 + new_pct * (60.0 - 1.0);
                    tools.set_selected_jump_ramp_pitch_deg(state, new_pitch);
                }
                if input_click {
                    *active_modal = EditorModal::SetRampProperty {
                        property: RampPropertyModal::Pitch,
                        input_val: format!("{:.1}", pitch),
                    };
                }
                curr_y += row_h + scaler.s(10.0);

                // 6. LAUNCH SPEED (1.0m/s – 20.0m/s)
                let ramp = &state.track.geometry.jump_ramps[idx];
                let l_spd = ramp.launch_speed;
                fonts.draw_ui_bold("Launch Speed Boost:", x + scaler.s(12.0), curr_y + scaler.s(11.0), scaler.font_s(11.0), Palette::NEON_GOLD);
                curr_y += scaler.s(16.0);

                let spd_pct = ((l_spd - 1.0) / (20.0 - 1.0)).clamp(0.0, 1.0);
                let spd_str = format!("{:.1}m/s", l_spd);
                let (drag_pct, input_click) = draw_slider_with_input_field(fonts, scaler, x + scaler.s(12.0), curr_y, row_w, row_h, spd_pct, &spd_str, true, mouse_pos, clicked);
                if let Some(new_pct) = drag_pct {
                    let new_spd = 1.0 + new_pct * (20.0 - 1.0);
                    tools.set_selected_jump_ramp_launch_speed(state, new_spd);
                }
                if input_click {
                    *active_modal = EditorModal::SetRampProperty {
                        property: RampPropertyModal::LaunchSpeed,
                        input_val: format!("{:.1}", l_spd),
                    };
                }
                curr_y += row_h + scaler.s(12.0);

                // 6. AUTO-FIT BUTTONS: FIT PITCH & FIT HEIGHT
                let ramp = &state.track.geometry.jump_ramps[idx];
                let fitted_deg = ramp.fitted_pitch_deg();
                let fitted_h = ramp.fitted_height();
                let flat_len = ramp.flat_length();

                let fit_pitch_label = format!("FIT PITCH: {:.1}°", fitted_deg);
                let fit_height_label = format!("FIT HEIGHT: {:.1}m", fitted_h);
                let fit_border = if flat_len > 0.05 { Palette::NEON_GREEN } else { Palette::UI_CARD_BORDER };

                let fit_btn_w = (row_w - scaler.s(6.0)) * 0.5;
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, fit_btn_w, scaler.s(24.0), &fit_pitch_label, Palette::UI_CARD_BG, fit_border, mouse_pos, clicked) {
                    tools.remove_selected_jump_ramp_flat_portion(state);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + fit_btn_w, curr_y, fit_btn_w, scaler.s(24.0), &fit_height_label, Palette::UI_CARD_BG, fit_border, mouse_pos, clicked) {
                    tools.adjust_selected_jump_ramp_height_to_pitch(state);
                }
                curr_y += scaler.s(28.0);

                // 7. RAMP SURFACE MATERIAL PALETTE
                fonts.draw_ui_bold("Ramp Material:", x + scaler.s(12.0), curr_y + scaler.s(11.0), scaler.font_s(11.0), Palette::NEON_CYAN);
                curr_y += scaler.s(16.0);

                let surfaces = [
                    (SurfaceType::Asphalt, "Asphalt"),
                    (SurfaceType::Dirt, "Dirt / Rally"),
                    (SurfaceType::Sand, "Sand"),
                    (SurfaceType::Grass, "Grass"),
                    (SurfaceType::Ice, "Ice"),
                    (SurfaceType::Water, "Water Hazard"),
                ];

                let half_btn_w = (row_w - scaler.s(6.0)) * 0.5;
                for chunk in surfaces.chunks(2) {
                    let (st1, label1) = chunk[0];
                    let is_active1 = state.track.geometry.jump_ramps[idx].surface == st1;
                    if draw_ui_btn(
                        fonts,
                        scaler,
                        x + scaler.s(12.0),
                        curr_y,
                        half_btn_w,
                        scaler.s(22.0),
                        label1,
                        if is_active1 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                        if is_active1 { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER },
                        mouse_pos,
                        clicked,
                    ) {
                        tools.set_selected_jump_ramp_surface(state, st1);
                    }

                    if chunk.len() > 1 {
                        let (st2, label2) = chunk[1];
                        let is_active2 = state.track.geometry.jump_ramps[idx].surface == st2;
                        if draw_ui_btn(
                            fonts,
                            scaler,
                            x + scaler.s(12.0) + half_btn_w + scaler.s(6.0),
                            curr_y,
                            half_btn_w,
                            scaler.s(22.0),
                            label2,
                            if is_active2 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                            if is_active2 { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER },
                            mouse_pos,
                            clicked,
                        ) {
                            tools.set_selected_jump_ramp_surface(state, st2);
                        }
                    }
                    curr_y += scaler.s(25.0);
                }
                curr_y += scaler.s(6.0);

                // 8. LATERAL VIEW DIAGRAM
                fonts.draw_ui_bold("Lateral View Profile:", x + scaler.s(12.0), curr_y + scaler.s(11.0), scaler.font_s(11.0), Palette::NEON_CYAN);
                curr_y += scaler.s(16.0);

                let ramp = &state.track.geometry.jump_ramps[idx];
                draw_ramp_lateral_view(fonts, scaler, x + scaler.s(12.0), curr_y, row_w, scaler.s(80.0), ramp);
                curr_y += scaler.s(86.0);

                // 9. ACTION BUTTONS
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, row_w, scaler.s(26.0), "DUPLICATE RAMP [Ctrl+D]", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                    tools.duplicate_selected(state);
                }
                curr_y += scaler.s(30.0);

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, row_w, scaler.s(26.0), "DELETE RAMP [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
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
                let finish_lbl = if is_finish { "[X] Finish Line" } else { "[ ] Normal Sector" };
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(26.0), finish_lbl, Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                    state.record_undo();
                    state.track.checkpoints[pos].is_finish_line = !is_finish;
                }
            }
            curr_y += scaler.s(32.0);

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DUPLICATE GATE [Ctrl+D]", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                tools.duplicate_selected(state);
            }
            curr_y += scaler.s(32.0);

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DELETE GATE [Del]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) {
                tools.delete_selected(state);
            }
        }
        Selection::GridSlot(slot) => {
            fonts.draw_ui_bold(&format!("Starting Grid Slot #{}", slot), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
            curr_y += scaler.s(32.0);

            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(28.0), "DUPLICATE SLOT [Ctrl+D]", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
                tools.duplicate_selected(state);
            }
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
        Selection::Multi {
            ref waypoints,
            ref surface_zones,
            ref obstacles,
            ref jump_ramps,
            ref checkpoints,
            ref grid_slots,
            pit_box,
        } => {
            let total = waypoints.len()
                + surface_zones.len()
                + obstacles.len()
                + jump_ramps.len()
                + checkpoints.len()
                + grid_slots.len()
                + if pit_box { 1 } else { 0 };

            fonts.draw_ui_bold(
                &format!("Multi-Selection ({})", total),
                x + scaler.s(12.0),
                curr_y + scaler.s(14.0),
                scaler.font_s(13.0),
                Palette::WHITE,
            );
            curr_y += scaler.s(24.0);

            if !waypoints.is_empty() {
                fonts.draw_ui_regular(
                    &format!("• Waypoints: {}", waypoints.len()),
                    x + scaler.s(16.0),
                    curr_y + scaler.s(12.0),
                    scaler.font_s(12.0),
                    Palette::UI_TEXT_MUTED,
                );
                curr_y += scaler.s(18.0);
            }
            if !surface_zones.is_empty() {
                fonts.draw_ui_regular(
                    &format!("• Surface Zones: {}", surface_zones.len()),
                    x + scaler.s(16.0),
                    curr_y + scaler.s(12.0),
                    scaler.font_s(12.0),
                    Palette::UI_TEXT_MUTED,
                );
                curr_y += scaler.s(18.0);
            }
            if !obstacles.is_empty() {
                fonts.draw_ui_regular(
                    &format!("• Obstacles: {}", obstacles.len()),
                    x + scaler.s(16.0),
                    curr_y + scaler.s(12.0),
                    scaler.font_s(12.0),
                    Palette::UI_TEXT_MUTED,
                );
                curr_y += scaler.s(18.0);
            }
            if !jump_ramps.is_empty() {
                fonts.draw_ui_regular(
                    &format!("• Jump Ramps: {}", jump_ramps.len()),
                    x + scaler.s(16.0),
                    curr_y + scaler.s(12.0),
                    scaler.font_s(12.0),
                    Palette::UI_TEXT_MUTED,
                );
                curr_y += scaler.s(18.0);
            }
            if !checkpoints.is_empty() {
                fonts.draw_ui_regular(
                    &format!("• Checkpoints: {}", checkpoints.len()),
                    x + scaler.s(16.0),
                    curr_y + scaler.s(12.0),
                    scaler.font_s(12.0),
                    Palette::UI_TEXT_MUTED,
                );
                curr_y += scaler.s(18.0);
            }
            if !grid_slots.is_empty() {
                fonts.draw_ui_regular(
                    &format!("• Grid Slots: {}", grid_slots.len()),
                    x + scaler.s(16.0),
                    curr_y + scaler.s(12.0),
                    scaler.font_s(12.0),
                    Palette::UI_TEXT_MUTED,
                );
                curr_y += scaler.s(18.0);
            }
            if pit_box {
                fonts.draw_ui_regular(
                    "• Pit Lane Area",
                    x + scaler.s(16.0),
                    curr_y + scaler.s(12.0),
                    scaler.font_s(12.0),
                    Palette::UI_TEXT_MUTED,
                );
                curr_y += scaler.s(18.0);
            }
            curr_y += scaler.s(6.0);

            if !jump_ramps.is_empty() {
                fonts.draw_ui_bold("BATCH RAMPS:", x + scaler.s(12.0), curr_y + scaler.s(12.0), scaler.font_s(11.0), Palette::NEON_CYAN);
                curr_y += scaler.s(18.0);

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, w - scaler.s(24.0), scaler.s(24.0), "SET EXACT ANGLE [0°–365°]", Palette::UI_CARD_BG_HOVER, Palette::NEON_CYAN, mouse_pos, clicked) {
                    *active_modal = EditorModal::SetRampAngle {
                        input_angle: "0.0".to_string(),
                    };
                }
                curr_y += scaler.s(28.0);

                let half_btn_w = (w - scaler.s(30.0)) * 0.5;
                let q_btn_w = (w - scaler.s(42.0)) / 4.0;
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, q_btn_w, scaler.s(22.0), "-1°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.rotate_selected_jump_ramp(state, -std::f32::consts::PI / 180.0);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + q_btn_w, curr_y, q_btn_w, scaler.s(22.0), "+1°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.rotate_selected_jump_ramp(state, std::f32::consts::PI / 180.0);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(24.0) + q_btn_w * 2.0, curr_y, q_btn_w, scaler.s(22.0), "-15°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.rotate_selected_jump_ramp(state, -std::f32::consts::PI / 12.0);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(30.0) + q_btn_w * 3.0, curr_y, q_btn_w, scaler.s(22.0), "+15°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.rotate_selected_jump_ramp(state, std::f32::consts::PI / 12.0);
                }
                curr_y += scaler.s(26.0);

                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, half_btn_w, scaler.s(22.0), "-10% Size", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.scale_selected_jump_ramp_size(state, 0.90);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + half_btn_w, curr_y, half_btn_w, scaler.s(22.0), "+10% Size", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.scale_selected_jump_ramp_size(state, 1.10);
                }
                curr_y += scaler.s(28.0);
            }

            if draw_ui_btn(
                fonts,
                scaler,
                x + scaler.s(12.0),
                curr_y,
                w - scaler.s(24.0),
                scaler.s(28.0),
                "DUPLICATE ALL [Ctrl+D]",
                Palette::UI_CARD_BG,
                Palette::NEON_CYAN,
                mouse_pos,
                clicked,
            ) {
                tools.duplicate_selected(state);
            }
            curr_y += scaler.s(32.0);

            if draw_ui_btn(
                fonts,
                scaler,
                x + scaler.s(12.0),
                curr_y,
                w - scaler.s(24.0),
                scaler.s(28.0),
                "DELETE ALL [Del]",
                Palette::UI_CARD_BG,
                Palette::RED,
                mouse_pos,
                clicked,
            ) {
                tools.delete_selected(state);
            }
        }
        Selection::None => {
            if tools.active_tool == EditorToolType::RoadSpline {
                fonts.draw_ui_bold("Road Spline Tool", x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
                curr_y += scaler.s(24.0);

                fonts.draw_ui_bold(&format!("Next Surface: {}", tools.active_surface.name()), x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_CYAN);
                curr_y += scaler.s(20.0);

                let half_btn_w = (w - scaler.s(30.0)) * 0.5;
                let surfaces = [
                    (SurfaceType::Asphalt, "Asphalt"),
                    (SurfaceType::Dirt, "Dirt"),
                    (SurfaceType::Sand, "Sand"),
                    (SurfaceType::Grass, "Grass"),
                    (SurfaceType::Ice, "Ice"),
                    (SurfaceType::Water, "Water"),
                ];

                for chunk in surfaces.chunks(2) {
                    let (st1, label1) = chunk[0];
                    let is_active1 = tools.active_surface == st1;
                    if draw_ui_btn(
                        fonts,
                        scaler,
                        x + scaler.s(12.0),
                        curr_y,
                        half_btn_w,
                        scaler.s(22.0),
                        label1,
                        if is_active1 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                        if is_active1 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER },
                        mouse_pos,
                        clicked,
                    ) {
                        tools.active_surface = st1;
                    }

                    if chunk.len() > 1 {
                        let (st2, label2) = chunk[1];
                        let is_active2 = tools.active_surface == st2;
                        if draw_ui_btn(
                            fonts,
                            scaler,
                            x + scaler.s(12.0) + half_btn_w + scaler.s(6.0),
                            curr_y,
                            half_btn_w,
                            scaler.s(22.0),
                            label2,
                            if is_active2 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                            if is_active2 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER },
                            mouse_pos,
                            clicked,
                        ) {
                            tools.active_surface = st2;
                        }
                    }
                    curr_y += scaler.s(26.0);
                }
                curr_y += scaler.s(6.0);

                // Default Placement Width
                fonts.draw_ui_bold("Placement Width:", x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_CYAN);
                if draw_ui_btn(fonts, scaler, x + scaler.s(120.0), curr_y, scaler.s(45.0), scaler.s(22.0), "-1m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_width = (tools.new_waypoint_width - 1.0).max(4.0);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(170.0), curr_y, scaler.s(45.0), scaler.s(22.0), "+1m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_width = (tools.new_waypoint_width + 1.0).min(30.0);
                }
                curr_y += scaler.s(26.0);

                let wid_pct = ((tools.new_waypoint_width - 4.0) / 26.0).clamp(0.0, 1.0);
                let (drag_w_pct, _) = draw_slider_with_input_field(
                    fonts,
                    scaler,
                    x + scaler.s(12.0),
                    curr_y,
                    w - scaler.s(24.0),
                    scaler.s(20.0),
                    wid_pct,
                    &format!("{:.1}m", tools.new_waypoint_width),
                    false,
                    mouse_pos,
                    clicked,
                );
                if let Some(p) = drag_w_pct {
                    tools.new_waypoint_width = (4.0 + p * 26.0).clamp(4.0, 30.0);
                }
                curr_y += scaler.s(26.0);

                // Default Placement Banking
                fonts.draw_ui_bold("Placement Banking:", x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(12.0), Palette::NEON_GOLD);
                curr_y += scaler.s(20.0);

                let bank_pct = ((tools.new_waypoint_bank_angle + 45.0) / 90.0).clamp(0.0, 1.0);
                let (drag_b_pct, _) = draw_slider_with_input_field(
                    fonts,
                    scaler,
                    x + scaler.s(12.0),
                    curr_y,
                    w - scaler.s(24.0),
                    scaler.s(20.0),
                    bank_pct,
                    &format!("{:+.1}°", tools.new_waypoint_bank_angle),
                    true,
                    mouse_pos,
                    clicked,
                );
                if let Some(p) = drag_b_pct {
                    tools.new_waypoint_bank_angle = (-45.0 + p * 90.0).clamp(-45.0, 45.0);
                }
                curr_y += scaler.s(24.0);

                let btn_q_w = (w - scaler.s(36.0)) * 0.25;
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_q_w, scaler.s(22.0), "-5°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_bank_angle = (tools.new_waypoint_bank_angle - 5.0).clamp(-45.0, 45.0);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_q_w, curr_y, btn_q_w, scaler.s(22.0), "-1°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_bank_angle = (tools.new_waypoint_bank_angle - 1.0).clamp(-45.0, 45.0);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_q_w * 2.0, curr_y, btn_q_w, scaler.s(22.0), "+1°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_bank_angle = (tools.new_waypoint_bank_angle + 1.0).clamp(-45.0, 45.0);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_q_w * 3.0, curr_y, btn_q_w, scaler.s(22.0), "+5°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_bank_angle = (tools.new_waypoint_bank_angle + 5.0).clamp(-45.0, 45.0);
                }
                curr_y += scaler.s(26.0);

                let btn_p_w = (w - scaler.s(36.0)) * 0.20;
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_p_w, scaler.s(20.0), "0°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_bank_angle = 0.0;
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_p_w, curr_y, btn_p_w, scaler.s(20.0), "10°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_bank_angle = 10.0;
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_p_w * 2.0, curr_y, btn_p_w, scaler.s(20.0), "18°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_bank_angle = 18.0;
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_p_w * 3.0, curr_y, btn_p_w, scaler.s(20.0), "22°", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_bank_angle = 22.0;
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(20.0) + btn_p_w * 4.0, curr_y, btn_p_w, scaler.s(20.0), "+/-", Palette::UI_CARD_BG, Palette::NEON_GOLD, mouse_pos, clicked) {
                    tools.new_waypoint_bank_angle = -tools.new_waypoint_bank_angle;
                }
                curr_y += scaler.s(28.0);

                // Default Placement Wall Distance
                let cur_placement_dist = tools
                    .new_waypoint_left_wall_distance
                    .unwrap_or(state.barrier_offset);
                fonts.draw_ui_bold(
                    &format!("Placement Wall Dist: {:.1}m", cur_placement_dist),
                    x + scaler.s(12.0),
                    curr_y + scaler.s(14.0),
                    scaler.font_s(12.0),
                    Palette::NEON_CYAN,
                );
                curr_y += scaler.s(20.0);

                let dist_pct = (cur_placement_dist / 15.0).clamp(0.0, 1.0);
                let (drag_d_pct, _) = draw_slider_with_input_field(
                    fonts,
                    scaler,
                    x + scaler.s(12.0),
                    curr_y,
                    w - scaler.s(24.0),
                    scaler.s(20.0),
                    dist_pct,
                    &format!("{:.1}m", cur_placement_dist),
                    false,
                    mouse_pos,
                    clicked,
                );
                if let Some(p) = drag_d_pct {
                    let val = (p * 15.0).clamp(0.0, 20.0);
                    tools.new_waypoint_left_wall_distance = Some(val);
                    tools.new_waypoint_right_wall_distance = Some(val);
                }
                curr_y += scaler.s(24.0);

                let btn_pwd_w = (w - scaler.s(36.0)) * 0.25;
                if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_pwd_w, scaler.s(20.0), "0m Flush", Palette::UI_CARD_BG, Palette::NEON_GOLD, mouse_pos, clicked) {
                    tools.new_waypoint_left_wall_distance = Some(0.0);
                    tools.new_waypoint_right_wall_distance = Some(0.0);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_pwd_w, curr_y, btn_pwd_w, scaler.s(20.0), "1.5m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_left_wall_distance = Some(1.5);
                    tools.new_waypoint_right_wall_distance = Some(1.5);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_pwd_w * 2.0, curr_y, btn_pwd_w, scaler.s(20.0), "4.0m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_left_wall_distance = Some(4.0);
                    tools.new_waypoint_right_wall_distance = Some(4.0);
                }
                if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_pwd_w * 3.0, curr_y, btn_pwd_w, scaler.s(20.0), "Default", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                    tools.new_waypoint_left_wall_distance = None;
                    tools.new_waypoint_right_wall_distance = None;
                }
                curr_y += scaler.s(28.0);
            }

            // Global Barrier Offset (Default Wall Distance)
            fonts.draw_ui_bold(
                &format!("Global Wall Offset: {:.1}m", state.barrier_offset),
                x + scaler.s(12.0),
                curr_y + scaler.s(14.0),
                scaler.font_s(13.0),
                Palette::NEON_CYAN,
            );
            curr_y += scaler.s(20.0);

            let bo_pct = (state.barrier_offset / 15.0).clamp(0.0, 1.0);
            let (drag_bo_pct, _) = draw_slider_with_input_field(
                fonts,
                scaler,
                x + scaler.s(12.0),
                curr_y,
                w - scaler.s(24.0),
                scaler.s(20.0),
                bo_pct,
                &format!("{:.1}m", state.barrier_offset),
                false,
                mouse_pos,
                clicked,
            );
            if let Some(p) = drag_bo_pct {
                tools.set_global_barrier_offset(state, p * 15.0);
            }
            curr_y += scaler.s(24.0);

            let btn_bo_w = (w - scaler.s(36.0)) * 0.25;
            if draw_ui_btn(fonts, scaler, x + scaler.s(12.0), curr_y, btn_bo_w, scaler.s(22.0), "-1m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                let new_bo = (state.barrier_offset - 1.0).max(0.0);
                tools.set_global_barrier_offset(state, new_bo);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(14.0) + btn_bo_w, curr_y, btn_bo_w, scaler.s(22.0), "+1m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                let new_bo = (state.barrier_offset + 1.0).min(20.0);
                tools.set_global_barrier_offset(state, new_bo);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(16.0) + btn_bo_w * 2.0, curr_y, btn_bo_w, scaler.s(22.0), "0m", Palette::UI_CARD_BG, Palette::NEON_GOLD, mouse_pos, clicked) {
                tools.set_global_barrier_offset(state, 0.0);
            }
            if draw_ui_btn(fonts, scaler, x + scaler.s(18.0) + btn_bo_w * 3.0, curr_y, btn_bo_w, scaler.s(22.0), "4m", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                tools.set_global_barrier_offset(state, 4.0);
            }
            curr_y += scaler.s(28.0);

            fonts.draw_ui_bold("Global Off-Track Surface", x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::NEON_CYAN);
            curr_y += scaler.s(22.0);

            let offtrack_surfaces = [
                (SurfaceType::Grass, "Grass"),
                (SurfaceType::Sand, "Sand"),
                (SurfaceType::Dirt, "Dirt"),
                (SurfaceType::Asphalt, "Asphalt"),
            ];

            let half_btn_w = (w - scaler.s(30.0)) * 0.5;
            for chunk in offtrack_surfaces.chunks(2) {
                let (st1, label1) = chunk[0];
                let is_active1 = state.track.default_surface == st1;
                if draw_ui_btn(
                    fonts,
                    scaler,
                    x + scaler.s(12.0),
                    curr_y,
                    half_btn_w,
                    scaler.s(22.0),
                    label1,
                    if is_active1 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                    if is_active1 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER },
                    mouse_pos,
                    clicked,
                ) {
                    tools.set_track_default_surface(state, st1);
                }

                if chunk.len() > 1 {
                    let (st2, label2) = chunk[1];
                    let is_active2 = state.track.default_surface == st2;
                    if draw_ui_btn(
                        fonts,
                        scaler,
                        x + scaler.s(12.0) + half_btn_w + scaler.s(6.0),
                        curr_y,
                        half_btn_w,
                        scaler.s(22.0),
                        label2,
                        if is_active2 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                        if is_active2 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER },
                        mouse_pos,
                        clicked,
                    ) {
                        tools.set_track_default_surface(state, st2);
                    }
                }
                curr_y += scaler.s(26.0);
            }
            curr_y += scaler.s(6.0);

            fonts.draw_ui_bold("Predefined Vehicle", x + scaler.s(12.0), curr_y + scaler.s(14.0), scaler.font_s(13.0), Palette::NEON_CYAN);
            curr_y += scaler.s(22.0);

            let cars = [
                ("sports_car", "GT Sport"),
                ("drift_car", "Drift Spec"),
                ("kart", "125cc Kart"),
                ("rally_car", "AWD Rally"),
                ("f1_car", "F1 Hybrid"),
            ];

            let active_car_str = state.track.predefined_car.clone().unwrap_or_else(|| "sports_car".to_string());
            let is_matching_car = |car_id: &str| -> bool {
                match car_id {
                    "sports_car" => matches!(active_car_str.as_str(), "sports_car"),
                    "drift_car" => matches!(active_car_str.as_str(), "drift_car"),
                    "kart" => matches!(active_car_str.as_str(), "kart" | "shifter_kart" | "shifter_kart_125"),
                    "rally_car" => matches!(active_car_str.as_str(), "rally_car" | "wrc_turbo_rally" | "rally"),
                    "f1_car" => matches!(active_car_str.as_str(), "f1_car" | "f1" | "f1_hybrid_26" | "open_wheel"),
                    _ => false,
                }
            };

            for chunk in cars.chunks(2) {
                let (cid1, label1) = chunk[0];
                let is_active1 = is_matching_car(cid1);
                if draw_ui_btn(
                    fonts,
                    scaler,
                    x + scaler.s(12.0),
                    curr_y,
                    half_btn_w,
                    scaler.s(22.0),
                    label1,
                    if is_active1 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                    if is_active1 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER },
                    mouse_pos,
                    clicked,
                ) {
                    tools.set_track_predefined_car(state, Some(cid1.to_string()));
                }

                if chunk.len() > 1 {
                    let (cid2, label2) = chunk[1];
                    let is_active2 = is_matching_car(cid2);
                    if draw_ui_btn(
                        fonts,
                        scaler,
                        x + scaler.s(12.0) + half_btn_w + scaler.s(6.0),
                        curr_y,
                        half_btn_w,
                        scaler.s(22.0),
                        label2,
                        if is_active2 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
                        if is_active2 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER },
                        mouse_pos,
                        clicked,
                    ) {
                        tools.set_track_predefined_car(state, Some(cid2.to_string()));
                    }
                }
                curr_y += scaler.s(26.0);
            }
            curr_y += scaler.s(6.0);

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
                state.rebuild_geometry();
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

/// Helper to draw an interactive slider bar with a compact input field box next to it.
/// Returns (Some(new_pct) if dragged, true if the input box was clicked to edit).
fn draw_slider_with_input_field(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    total_w: f32,
    h: f32,
    pct: f32,
    val_str: &str,
    is_gold: bool,
    mouse_pos: Vec2,
    clicked: bool,
) -> (Option<f32>, bool) {
    let input_w = scaler.s(60.0);
    let gap = scaler.s(6.0);
    let slider_w = (total_w - input_w - gap).max(20.0);
    let input_x = x + slider_w + gap;

    // 1. Draggable slider on left
    let accent_col = if is_gold { Palette::NEON_GOLD } else { Palette::NEON_CYAN };
    let thumb_col = if is_gold { Palette::NEON_CYAN } else { Palette::NEON_GOLD };

    scaler.draw_glass_card(x, y, slider_w, h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.0);
    let bar_pct = pct.clamp(0.0, 1.0);
    let inner_w = (slider_w - scaler.s(4.0)).max(1.0);
    let inner_h = (h - scaler.s(4.0)).max(1.0);
    macroquad::shapes::draw_rectangle(x + scaler.s(2.0), y + scaler.s(2.0), inner_w * bar_pct, inner_h, accent_col);
    let thumb_x = x + scaler.s(2.0) + inner_w * bar_pct;
    macroquad::shapes::draw_circle(thumb_x, y + h * 0.5, scaler.s(5.5), thumb_col);

    let mouse_over_slider = mouse_pos.x >= x - scaler.s(4.0) && mouse_pos.x <= x + slider_w + scaler.s(4.0) && mouse_pos.y >= y - scaler.s(4.0) && mouse_pos.y <= y + h + scaler.s(4.0);
    let is_down = is_mouse_button_down(MouseButton::Left);
    let new_pct = if is_down && mouse_over_slider {
        let p = ((mouse_pos.x - (x + scaler.s(2.0))) / inner_w).clamp(0.0, 1.0);
        Some(p)
    } else {
        None
    };

    // 2. Input field on right
    let mouse_over_input = mouse_pos.x >= input_x && mouse_pos.x <= input_x + input_w && mouse_pos.y >= y && mouse_pos.y <= y + h;
    let input_bg = if mouse_over_input { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG };
    let input_border = if mouse_over_input { accent_col } else { Palette::UI_CARD_BORDER };

    scaler.draw_glass_card(input_x, y, input_w, h, input_bg, input_border, if mouse_over_input { 1.5 } else { 1.0 });
    fonts.draw_ui_bold_centered(
        val_str,
        input_x + input_w * 0.5,
        y + h * 0.5 + scaler.s(4.0),
        scaler.font_s(11.0),
        if mouse_over_input { Palette::WHITE } else { Color::new(0.90, 0.94, 1.0, 1.0) },
    );

    let input_clicked = mouse_over_input && clicked;
    (new_pct, input_clicked)
}

/// Renders a 2D lateral cross-section diagram of a jump ramp showing incline slope, tabletop flat, launch lip, and surface material.
fn draw_ramp_lateral_view(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    ramp: &JumpRamp,
) {
    scaler.draw_glass_card(x, y, w, h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.0);

    let pad_x = scaler.s(18.0);
    let pad_top = scaler.s(18.0);
    let pad_bot = scaler.s(20.0);

    let draw_w = (w - pad_x * 2.0).max(10.0);
    let draw_h = (h - pad_top - pad_bot).max(10.0);

    let ground_y = y + h - pad_bot;
    let origin_x = x + pad_x;

    let len = ramp.length().max(1.0);
    let height = ramp.height.max(0.1);
    let incline_len = ramp.incline_length();
    let flat_len = ramp.flat_length();

    let s_x = draw_w / len;

    let inc_w = (incline_len * s_x).clamp(0.0, draw_w);

    let p0 = Vec2::new(origin_x, ground_y);
    let p_inc = Vec2::new(origin_x + inc_w, ground_y - draw_h);
    let p_top_exit = Vec2::new(origin_x + draw_w, ground_y - draw_h);
    let p_bot_exit = Vec2::new(origin_x + draw_w, ground_y);

    let (base_col, border_opt, _) = crate::render::track::get_ramp_surface_colors(ramp.surface);

    // Ground line
    macroquad::shapes::draw_line(origin_x - scaler.s(6.0), ground_y, origin_x + draw_w + scaler.s(6.0), ground_y, scaler.s(1.2), Palette::UI_TEXT_MUTED);

    // Draw ramp cross-section body fill
    if flat_len > 0.05 && inc_w < draw_w - scaler.s(1.0) {
        // Triangle incline
        let p_inc_base = Vec2::new(origin_x + inc_w, ground_y);
        macroquad::shapes::draw_triangle(
            macroquad::prelude::Vec2::new(p0.x, p0.y),
            macroquad::prelude::Vec2::new(p_inc.x, p_inc.y),
            macroquad::prelude::Vec2::new(p_inc_base.x, p_inc_base.y),
            base_col,
        );
        // Tabletop rectangle
        macroquad::shapes::draw_rectangle(origin_x + inc_w, ground_y - draw_h, draw_w - inc_w, draw_h, base_col);
    } else {
        macroquad::shapes::draw_triangle(
            macroquad::prelude::Vec2::new(p0.x, p0.y),
            macroquad::prelude::Vec2::new(p_top_exit.x, p_top_exit.y),
            macroquad::prelude::Vec2::new(p_bot_exit.x, p_bot_exit.y),
            base_col,
        );
    }

    // Incline slope stroke
    let slope_col = border_opt.unwrap_or(Palette::NEON_GOLD);
    macroquad::shapes::draw_line(p0.x, p0.y, p_inc.x, p_inc.y, scaler.s(2.0), slope_col);

    // Tabletop stroke (if flat portion exists)
    if flat_len > 0.05 && inc_w < draw_w - scaler.s(1.0) {
        macroquad::shapes::draw_line(p_inc.x, p_inc.y, p_top_exit.x, p_top_exit.y, scaler.s(2.0), Palette::NEON_CYAN);
    }

    // Exit launch lip (vertical drop line)
    macroquad::shapes::draw_line(p_top_exit.x, p_top_exit.y, p_bot_exit.x, p_bot_exit.y, scaler.s(2.0), Palette::NEON_CYAN);

    // Annotations text
    // Length (bottom)
    let len_str = format!("L:{:.1}m", len);
    fonts.draw_ui_regular(&len_str, origin_x + draw_w * 0.5 - scaler.s(16.0), ground_y + scaler.s(13.0), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);

    // Height (right vertical)
    let h_str = format!("H:{:.1}m", height);
    fonts.draw_ui_regular(&h_str, (p_top_exit.x - scaler.s(36.0)).max(origin_x), ground_y - draw_h * 0.5 + scaler.s(3.0), scaler.font_s(9.5), Palette::NEON_GOLD);

    // Pitch (along slope)
    let pitch_str = format!("{:.0}°", ramp.ramp_angle_deg);
    fonts.draw_ui_bold(&pitch_str, p0.x + scaler.s(8.0), ground_y - scaler.s(4.0), scaler.font_s(9.5), Palette::NEON_GOLD);

    // Flat label if present
    if flat_len > 0.05 {
        let flat_str = format!("Flat:{:.1}m", flat_len);
        let flat_x = origin_x + inc_w + (draw_w - inc_w) * 0.5 - scaler.s(18.0);
        fonts.draw_ui_bold(&flat_str, flat_x.max(origin_x + inc_w), ground_y - draw_h - scaler.s(3.0), scaler.font_s(9.0), Palette::NEON_CYAN);
    }
}

/// Renders modal overlay for specifying exact jump ramp dimension/orientation property.
fn render_set_ramp_property_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    state: &mut EditorState,
    tools: &mut ToolSettings,
    property: RampPropertyModal,
    input_val: &mut String,
    mouse_pos: Vec2,
    clicked: bool,
) -> bool {
    let mw = scaler.s(440.0);
    let mh = scaler.s(260.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.0);

    let (title, subtitle, unit, min_val, max_val) = match property {
        RampPropertyModal::Angle => ("SET JUMP RAMP ANGLE", "Specify direction angle (0° – 360°) • [Enter] to apply", "°", 0.0, 360.0),
        RampPropertyModal::Length => ("SET JUMP RAMP LENGTH", "Specify total ramp length (2.0m – 50.0m) • [Enter] to apply", "m", 2.0, 50.0),
        RampPropertyModal::Width => ("SET JUMP RAMP WIDTH", "Specify ramp width (1.0m – 30.0m) • [Enter] to apply", "m", 1.0, 30.0),
        RampPropertyModal::Height => ("SET JUMP RAMP HEIGHT", "Specify launch height (0.2m – 10.0m) • [Enter] to apply", "m", 0.2, 10.0),
        RampPropertyModal::Pitch => ("SET JUMP RAMP PITCH", "Specify incline pitch angle (1.0° – 60.0°) • [Enter] to apply", "°", 1.0, 60.0),
        RampPropertyModal::LaunchSpeed => ("SET LAUNCH SPEED BOOST", "Specify vertical launch speed (1.0m/s – 20.0m/s) • [Enter] to apply", "m/s", 1.0, 20.0),
    };

    fonts.draw_display_centered(title, sw * 0.5, my + scaler.s(26.0), scaler.font_s(20.0), Palette::NEON_GOLD);
    fonts.draw_ui_regular_centered(subtitle, sw * 0.5, my + scaler.s(48.0), scaler.font_s(11.5), Palette::UI_TEXT_MUTED);

    // Text Input Display Box
    let box_x = mx + scaler.s(30.0);
    let box_y = my + scaler.s(68.0);
    let box_w = mw - scaler.s(60.0);
    let box_h = scaler.s(38.0);

    scaler.draw_glass_card(box_x, box_y, box_w, box_h, Palette::UI_CARD_BG_HOVER, Palette::NEON_CYAN, 1.5);

    let display_str = if input_val.is_empty() {
        format!("0.0{}", unit)
    } else {
        format!("{}{}", input_val, unit)
    };
    fonts.draw_ui_bold(&display_str, box_x + scaler.s(16.0), box_y + scaler.s(24.0), scaler.font_s(17.0), Palette::WHITE);

    // Typing handling
    while let Some(c) = get_char_pressed() {
        if (c.is_ascii_digit() || c == '.') && input_val.len() < 8 {
            if c != '.' || !input_val.contains('.') {
                input_val.push(c);
            }
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        input_val.pop();
    }

    // Interactive slider in modal
    let slider_y = my + scaler.s(126.0);
    let curr_val: f32 = input_val.parse().unwrap_or(min_val);
    let slider_pct = ((curr_val - min_val) / (max_val - min_val)).clamp(0.0, 1.0);

    macroquad::shapes::draw_rectangle(box_x, slider_y, box_w, scaler.s(8.0), Palette::UI_CARD_BORDER);
    macroquad::shapes::draw_rectangle(box_x, slider_y, box_w * slider_pct, scaler.s(8.0), Palette::NEON_CYAN);
    let thumb_x = box_x + box_w * slider_pct;
    macroquad::shapes::draw_circle(thumb_x, slider_y + scaler.s(4.0), scaler.s(7.0), Palette::NEON_GOLD);

    let is_down = is_mouse_button_down(MouseButton::Left);
    if is_down && mouse_pos.x >= box_x && mouse_pos.x <= box_x + box_w && mouse_pos.y >= slider_y - scaler.s(8.0) && mouse_pos.y <= slider_y + scaler.s(16.0) {
        let pct = ((mouse_pos.x - box_x) / box_w).clamp(0.0, 1.0);
        let new_val = min_val + pct * (max_val - min_val);
        *input_val = format!("{:.1}", new_val);
    }

    let mut apply_and_close = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
    let cancel = is_key_pressed(KeyCode::Escape);

    let btn_w = (box_w - scaler.s(12.0)) * 0.5;
    let btn_y = my + mh - scaler.s(48.0);

    if draw_ui_btn(fonts, scaler, box_x, btn_y, btn_w, scaler.s(32.0), "APPLY [Enter]", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, clicked) {
        apply_and_close = true;
    }

    if draw_ui_btn(fonts, scaler, box_x + btn_w + scaler.s(12.0), btn_y, btn_w, scaler.s(32.0), "CANCEL [Esc]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, clicked) || cancel {
        return true;
    }

    if apply_and_close {
        if let Ok(parsed) = input_val.parse::<f32>() {
            match property {
                RampPropertyModal::Angle => tools.set_selected_jump_ramp_angle_deg(state, parsed),
                RampPropertyModal::Length => tools.set_selected_jump_ramp_length(state, parsed),
                RampPropertyModal::Width => tools.set_selected_jump_ramp_width(state, parsed),
                RampPropertyModal::Height => tools.set_selected_jump_ramp_height(state, parsed),
                RampPropertyModal::Pitch => tools.set_selected_jump_ramp_pitch_deg(state, parsed),
                RampPropertyModal::LaunchSpeed => tools.set_selected_jump_ramp_launch_speed(state, parsed),
            };
        }
        return true;
    }

    false
}

/// Backward compatible wrapper for angle modal.
fn render_set_ramp_angle_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    state: &mut EditorState,
    tools: &mut ToolSettings,
    input_angle: &mut String,
    mouse_pos: Vec2,
    clicked: bool,
) -> bool {
    render_set_ramp_property_modal(
        fonts,
        scaler,
        sw,
        sh,
        state,
        tools,
        RampPropertyModal::Angle,
        input_angle,
        mouse_pos,
        clicked,
    )
}

/// Renders Save As modal overlay with name, filename, and description text inputs and overwrite options.
fn render_save_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    input_name: &mut String,
    input_filename: &mut String,
    input_description: &mut String,
    active_field: &mut usize,
    overwrite: &mut bool,
    custom_filename_edited: &mut bool,
    exit_on_save: bool,
    current_file_path: Option<&str>,
    track_manager: &TrackManager,
    mouse_pos: Vec2,
    clicked: bool,
) -> Option<EditorAction> {
    let mw = scaler.s(540.0);
    let mh = scaler.s(440.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::NEON_GREEN, 2.0);

    let title_text = if exit_on_save {
        "SAVE & EXIT CIRCUIT"
    } else {
        "SAVE CIRCUIT"
    };
    fonts.draw_display_centered(title_text, sw * 0.5, my + scaler.s(26.0), scaler.font_s(22.0), Palette::NEON_GOLD);

    // Current loaded file context
    if let Some(loaded_path) = current_file_path {
        let fname = std::path::Path::new(loaded_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(loaded_path);
        fonts.draw_ui_regular_centered(
            &format!("Loaded: {} • [Tab] switch fields", fname),
            sw * 0.5,
            my + scaler.s(44.0),
            scaler.font_s(11.5),
            Palette::NEON_CYAN,
        );
    } else {
        fonts.draw_ui_regular_centered(
            "Specify name, filename, and description • [Tab] switch fields",
            sw * 0.5,
            my + scaler.s(44.0),
            scaler.font_s(11.5),
            Palette::UI_TEXT_MUTED,
        );
    }

    let is_overwrite_locked = *overwrite;

    // Keyboard navigation between fields
    if is_key_pressed(KeyCode::Tab) {
        if is_overwrite_locked {
            *active_field = if *active_field == 0 { 2 } else { 0 };
        } else {
            *active_field = match *active_field {
                0 => 1,
                1 => 2,
                _ => 0,
            };
        }
    }
    if is_key_pressed(KeyCode::Down) {
        if is_overwrite_locked {
            *active_field = 2;
        } else {
            *active_field = (*active_field + 1).min(2);
        }
    }
    if is_key_pressed(KeyCode::Up) {
        if is_overwrite_locked {
            *active_field = 0;
        } else {
            *active_field = active_field.saturating_sub(1);
        }
    }

    // Typing input handling
    while let Some(c) = get_char_pressed() {
        if !c.is_control() {
            if *active_field == 0 && input_name.len() < 32 {
                input_name.push(c);
                if !*custom_filename_edited && !*overwrite {
                    *input_filename = TrackManager::sanitize_slug(input_name);
                }
            } else if *active_field == 1 && !is_overwrite_locked && input_filename.len() < 32 {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    input_filename.push(c.to_ascii_lowercase());
                    *custom_filename_edited = true;
                }
            } else if *active_field == 2 && input_description.len() < 120 {
                input_description.push(c);
            }
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        if *active_field == 0 {
            input_name.pop();
            if !*custom_filename_edited && !*overwrite {
                *input_filename = TrackManager::sanitize_slug(input_name);
            }
        } else if *active_field == 1 && !is_overwrite_locked {
            input_filename.pop();
            *custom_filename_edited = true;
        } else if *active_field == 2 {
            input_description.pop();
        }
    }

    let inp_w = mw - scaler.s(40.0);
    let inp_x = mx + scaler.s(20.0);

    // Field 0: Track Name
    let f1_y = my + scaler.s(56.0);
    let f1_h = scaler.s(32.0);
    let is_f1_active = *active_field == 0;

    fonts.draw_ui_bold(
        "TRACK NAME:",
        inp_x,
        f1_y + scaler.s(10.0),
        scaler.font_s(11.0),
        if is_f1_active { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED },
    );

    let f1_box_y = f1_y + scaler.s(14.0);
    let f1_hover = mouse_pos.x >= inp_x && mouse_pos.x <= inp_x + inp_w && mouse_pos.y >= f1_box_y && mouse_pos.y <= f1_box_y + f1_h;
    if f1_hover && clicked {
        *active_field = 0;
    }

    draw_rectangle(inp_x, f1_box_y, inp_w, f1_h, Color::new(0.04, 0.05, 0.08, 0.95));
    draw_rectangle_lines(
        inp_x,
        f1_box_y,
        inp_w,
        f1_h,
        if is_f1_active { 1.8 } else { 1.0 },
        if is_f1_active { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER },
    );

    let name_text = if is_f1_active {
        format!("{}_", input_name)
    } else {
        input_name.clone()
    };
    fonts.draw_ui_bold(
        &name_text,
        inp_x + scaler.s(10.0),
        f1_box_y + scaler.s(21.0),
        scaler.font_s(13.5),
        Palette::WHITE,
    );

    // Field 1: Filename (.json)
    let f2_y = f1_box_y + f1_h + scaler.s(6.0);
    let f2_h = scaler.s(32.0);
    let is_f2_active = *active_field == 1 && !is_overwrite_locked;

    let f2_box_y = f2_y + scaler.s(14.0);
    let f2_hover = mouse_pos.x >= inp_x && mouse_pos.x <= inp_x + inp_w && mouse_pos.y >= f2_box_y && mouse_pos.y <= f2_box_y + f2_h;

    if is_overwrite_locked {
        let loaded_stem = current_file_path
            .and_then(|p| std::path::Path::new(p).file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or(if input_filename.is_empty() { "custom_track" } else { input_filename.as_str() });

        fonts.draw_ui_bold(
            "FILENAME (.json): [LOCKED ON OVERWRITE]",
            inp_x,
            f2_y + scaler.s(10.0),
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );

        draw_rectangle(inp_x, f2_box_y, inp_w, f2_h, Color::new(0.06, 0.07, 0.09, 0.95));
        draw_rectangle_lines(inp_x, f2_box_y, inp_w, f2_h, 1.0, Palette::UI_CARD_BORDER);

        fonts.draw_ui_regular(
            &format!("{}.json (Overwrites existing file)", loaded_stem),
            inp_x + scaler.s(10.0),
            f2_box_y + scaler.s(21.0),
            scaler.font_s(12.5),
            Palette::NEON_GOLD,
        );
    } else {
        if f2_hover && clicked {
            *active_field = 1;
            *custom_filename_edited = true;
        }

        fonts.draw_ui_bold(
            "FILENAME (.json):",
            inp_x,
            f2_y + scaler.s(10.0),
            scaler.font_s(11.0),
            if is_f2_active { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED },
        );

        draw_rectangle(inp_x, f2_box_y, inp_w, f2_h, Color::new(0.04, 0.05, 0.08, 0.95));
        draw_rectangle_lines(
            inp_x,
            f2_box_y,
            inp_w,
            f2_h,
            if is_f2_active { 1.8 } else { 1.0 },
            if is_f2_active { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER },
        );

        let fname_display = if is_f2_active {
            format!("{}_.json", input_filename)
        } else if input_filename.is_empty() {
            "custom_track.json".to_string()
        } else {
            format!("{}.json", input_filename)
        };

        fonts.draw_ui_regular(
            &fname_display,
            inp_x + scaler.s(10.0),
            f2_box_y + scaler.s(21.0),
            scaler.font_s(12.5),
            if is_f2_active { Palette::WHITE } else { Palette::NEON_CYAN },
        );
    }

    // Field 2: Track Description
    let f3_y = f2_box_y + f2_h + scaler.s(6.0);
    let f3_h = scaler.s(32.0);
    let is_f3_active = *active_field == 2;

    fonts.draw_ui_bold(
        "TRACK DESCRIPTION:",
        inp_x,
        f3_y + scaler.s(10.0),
        scaler.font_s(11.0),
        if is_f3_active { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED },
    );

    let f3_box_y = f3_y + scaler.s(14.0);
    let f3_hover = mouse_pos.x >= inp_x && mouse_pos.x <= inp_x + inp_w && mouse_pos.y >= f3_box_y && mouse_pos.y <= f3_box_y + f3_h;
    if f3_hover && clicked {
        *active_field = 2;
    }

    draw_rectangle(inp_x, f3_box_y, inp_w, f3_h, Color::new(0.04, 0.05, 0.08, 0.95));
    draw_rectangle_lines(
        inp_x,
        f3_box_y,
        inp_w,
        f3_h,
        if is_f3_active { 1.8 } else { 1.0 },
        if is_f3_active { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER },
    );

    if is_f3_active {
        fonts.draw_ui_regular(
            &format!("{}_", input_description),
            inp_x + scaler.s(10.0),
            f3_box_y + scaler.s(21.0),
            scaler.font_s(12.0),
            Palette::WHITE,
        );
    } else if input_description.is_empty() {
        fonts.draw_ui_regular(
            "Optional circuit description...",
            inp_x + scaler.s(10.0),
            f3_box_y + scaler.s(21.0),
            scaler.font_s(12.0),
            Palette::UI_TEXT_MUTED,
        );
    } else {
        fonts.draw_ui_regular(
            input_description,
            inp_x + scaler.s(10.0),
            f3_box_y + scaler.s(21.0),
            scaler.font_s(12.0),
            Palette::WHITE,
        );
    }

    // Check slug and file existence
    let slug = if *overwrite {
        if let Some(loaded_path) = current_file_path {
            std::path::Path::new(loaded_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("custom_track")
                .to_string()
        } else if !input_filename.trim().is_empty() {
            TrackManager::sanitize_slug(input_filename)
        } else {
            TrackManager::sanitize_slug(input_name)
        }
    } else if !input_filename.trim().is_empty() {
        TrackManager::sanitize_slug(input_filename)
    } else {
        TrackManager::sanitize_slug(input_name)
    };

    let is_filename_conflict = !*overwrite && track_manager.track_file_exists(&slug);

    let target_display_path = if *overwrite {
        let p = track_manager.track_path_for_slug(&slug);
        if let Ok(cwd) = std::env::current_dir() {
            let cwd_str = cwd.to_string_lossy();
            let p_str = p.to_string_lossy();
            if p_str.starts_with(cwd_str.as_ref()) {
                p_str.strip_prefix(cwd_str.as_ref()).unwrap_or(&p_str).trim_start_matches('/').to_string()
            } else {
                p_str.to_string()
            }
        } else {
            p.to_string_lossy().to_string()
        }
    } else {
        format!("tracks/drafts/{}.json", slug)
    };

    let info_y = f3_box_y + f3_h + scaler.s(12.0);
    if *overwrite {
        fonts.draw_ui_bold(
            &format!("Target: {} (Will overwrite existing file)", target_display_path),
            inp_x,
            info_y,
            scaler.font_s(11.5),
            Palette::YELLOW,
        );
    } else if is_filename_conflict {
        fonts.draw_ui_bold(
            &format!("❌ {} already exists! Change filename to Save As.", target_display_path),
            inp_x,
            info_y,
            scaler.font_s(11.5),
            Palette::RED,
        );
    } else {
        fonts.draw_ui_regular(
            &format!("✓ Target: {} (New unique file)", target_display_path),
            inp_x,
            info_y,
            scaler.font_s(11.5),
            Palette::NEON_GREEN,
        );
    }

    // Overwrite checkbox / toggle button (always labeled "Overwrite")
    let toggle_y = info_y + scaler.s(10.0);
    let toggle_lbl = if *overwrite {
        "[X] Overwrite"
    } else {
        "[ ] Overwrite"
    };
    let toggle_bg = if *overwrite {
        Color::new(0.35, 0.25, 0.05, 0.9)
    } else {
        Palette::UI_CARD_BG
    };
    let toggle_border = if *overwrite {
        Palette::YELLOW
    } else {
        Palette::UI_CARD_BORDER
    };

    if draw_ui_btn(
        fonts,
        scaler,
        inp_x,
        toggle_y,
        inp_w,
        scaler.s(26.0),
        toggle_lbl,
        toggle_bg,
        toggle_border,
        mouse_pos,
        clicked,
    ) {
        *overwrite = !*overwrite;
        if *overwrite {
            if let Some(loaded_path) = current_file_path {
                if let Some(stem) = std::path::Path::new(loaded_path).file_stem().and_then(|s| s.to_str()) {
                    *input_filename = stem.to_string();
                }
            }
            if *active_field == 1 {
                *active_field = 0;
            }
        } else {
            *active_field = 1;
            *custom_filename_edited = true;
        }
    }

    // Bottom Action Button:
    let btn_y = my + mh - scaler.s(48.0);
    let btn_h = scaler.s(36.0);

    let (btn_title, btn_color, btn_border) = if *overwrite {
        if exit_on_save {
            (
                "OVERWRITE & EXIT [Enter]",
                Color::new(0.45, 0.28, 0.08, 0.95),
                Palette::NEON_GOLD,
            )
        } else {
            (
                "OVERWRITE [Enter]",
                Color::new(0.45, 0.28, 0.08, 0.95),
                Palette::NEON_GOLD,
            )
        }
    } else if is_filename_conflict {
        (
            "CHANGE FILENAME TO SAVE AS",
            Color::new(0.35, 0.08, 0.08, 0.95),
            Palette::RED,
        )
    } else if exit_on_save {
        (
            "SAVE & EXIT [Enter]",
            Color::new(0.12, 0.65, 0.32, 0.95),
            Palette::NEON_GREEN,
        )
    } else {
        (
            "SAVE AS [Enter]",
            Color::new(0.12, 0.65, 0.32, 0.95),
            Palette::NEON_GREEN,
        )
    };

    let mut action_to_dispatch = None;
    let can_submit = *overwrite || !is_filename_conflict;

    let save_clicked = draw_ui_btn(
        fonts,
        scaler,
        inp_x,
        btn_y,
        inp_w,
        btn_h,
        btn_title,
        btn_color,
        btn_border,
        mouse_pos,
        clicked && can_submit,
    );

    if save_clicked && can_submit {
        action_to_dispatch = Some(EditorAction::SaveTrack {
            name: input_name.clone(),
            filename: input_filename.clone(),
            description: input_description.clone(),
            overwrite: *overwrite,
            exit_after: exit_on_save,
        });
    }

    if is_key_pressed(KeyCode::Enter) && action_to_dispatch.is_none() && can_submit {
        action_to_dispatch = Some(EditorAction::SaveTrack {
            name: input_name.clone(),
            filename: input_filename.clone(),
            description: input_description.clone(),
            overwrite: *overwrite,
            exit_after: exit_on_save,
        });
    }

    action_to_dispatch
}

/// Renders open track modal with tabs for all registered modules and drafts.
fn render_open_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    selected_tab: &mut usize,
    page: &mut usize,
    track_manager: &TrackManager,
    mouse_pos: Vec2,
    clicked: bool,
) -> Option<EditorAction> {
    let mw = scaler.s(680.0);
    let mh = scaler.s(490.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.0);

    fonts.draw_display_centered("OPEN CIRCUIT", sw * 0.5, my + scaler.s(26.0), scaler.font_s(22.0), Palette::NEON_GOLD);
    fonts.draw_ui_regular_centered(
        "Browse circuits across all registered motorsport modules and drafts workshop",
        sw * 0.5,
        my + scaler.s(45.0),
        scaler.font_s(11.5),
        Palette::UI_TEXT_MUTED,
    );

    let tabs: [(&str, &str); 6] = [
        ("ALL", "all"),
        ("CLASSIC", "classic"),
        ("F1 GP", "f1"),
        ("RALLY", "rally"),
        ("KARTING", "kart"),
        ("DRAFTS", "drafts"),
    ];

    if is_key_pressed(KeyCode::Left) {
        *selected_tab = selected_tab.checked_sub(1).unwrap_or(tabs.len() - 1);
        *page = 0;
    }
    if is_key_pressed(KeyCode::Right) {
        *selected_tab = (*selected_tab + 1) % tabs.len();
        *page = 0;
    }
    if is_key_pressed(KeyCode::Tab) {
        *selected_tab = (*selected_tab + 1) % tabs.len();
        *page = 0;
    }

    // Render Tab Buttons
    let tab_y = my + scaler.s(58.0);
    let tab_h = scaler.s(26.0);
    let tab_spacing = scaler.s(4.0);
    let total_w = mw - scaler.s(40.0);
    let tab_w = (total_w - tab_spacing * (tabs.len() as f32 - 1.0)) / (tabs.len() as f32);

    for (idx, (tab_label, _)) in tabs.iter().enumerate() {
        let tx = mx + scaler.s(20.0) + (tab_w + tab_spacing) * (idx as f32);
        let is_active = *selected_tab == idx;

        let bg_col = if is_active {
            Color::new(0.08, 0.28, 0.40, 0.95)
        } else {
            Palette::UI_PILL_BG
        };
        let border_col = if is_active {
            Palette::NEON_CYAN
        } else {
            Palette::UI_CARD_BORDER
        };

        if draw_ui_btn(fonts, scaler, tx, tab_y, tab_w, tab_h, tab_label, bg_col, border_col, mouse_pos, clicked) {
            *selected_tab = idx;
            *page = 0;
        }
    }

    // Get tracks for active tab
    let mod_id = tabs[*selected_tab].1;
    let tracks = track_manager.module_catalog_tracks(mod_id);

    let items_per_page = 5;
    let total_pages = ((tracks.len() + items_per_page - 1) / items_per_page).max(1);
    if *page >= total_pages {
        *page = total_pages - 1;
    }
    let start_idx = *page * items_per_page;
    let end_idx = (start_idx + items_per_page).min(tracks.len());

    let mut ty = tab_y + tab_h + scaler.s(12.0);
    let item_w = mw - scaler.s(40.0);
    let item_h = scaler.s(50.0);
    let item_x = mx + scaler.s(20.0);

    let mut chosen_action = None;

    if tracks.is_empty() {
        fonts.draw_ui_regular_centered(
            "No circuits found in this module catalog.",
            sw * 0.5,
            ty + scaler.s(60.0),
            scaler.font_s(13.0),
            Palette::UI_TEXT_MUTED,
        );
    } else {
        for choice in &tracks[start_idx..end_idx] {
            let is_hover = mouse_pos.x >= item_x && mouse_pos.x <= item_x + item_w && mouse_pos.y >= ty && mouse_pos.y <= ty + item_h;
            let bg_col = if is_hover {
                Palette::UI_CARD_BG_HOVER
            } else {
                Color::new(0.04, 0.06, 0.09, 0.95)
            };
            let border_col = if is_hover {
                Palette::NEON_CYAN
            } else {
                Palette::UI_CARD_BORDER
            };

            scaler.draw_glass_card(item_x, ty, item_w, item_h, bg_col, border_col, if is_hover { 1.8 } else { 1.0 });

            // Title & Description
            fonts.draw_ui_bold(
                choice.title(),
                item_x + scaler.s(14.0),
                ty + scaler.s(20.0),
                scaler.font_s(13.5),
                Palette::WHITE,
            );

            let desc = choice.description();
            let truncated_desc = if desc.len() > 70 {
                format!("{}...", &desc[..67])
            } else {
                desc.to_string()
            };
            fonts.draw_ui_regular(
                &truncated_desc,
                item_x + scaler.s(14.0),
                ty + scaler.s(37.0),
                scaler.font_s(11.0),
                Palette::UI_TEXT_MUTED,
            );

            // Badge on right
            let tag = choice.tag();
            let badge_col = if tag.contains("F1") {
                Palette::NEON_GOLD
            } else if tag.contains("RALLY") {
                Palette::YELLOW
            } else if tag.contains("KART") {
                Palette::NEON_CYAN
            } else if tag.contains("CUSTOM") || tag.contains("DRAFT") {
                Palette::NEON_GREEN
            } else {
                Palette::NEON_CYAN
            };

            let del_btn_w = scaler.s(45.0);
            let del_btn_x = item_x + item_w - del_btn_w - scaler.s(8.0);
            let badge_w = scaler.s(125.0);
            fonts.draw_ui_bold(
                tag,
                item_x + item_w - badge_w - del_btn_w - scaler.s(12.0),
                ty + scaler.s(28.0),
                scaler.font_s(11.0),
                badge_col,
            );

            let del_clicked = draw_ui_btn(
                fonts,
                scaler,
                del_btn_x,
                ty + scaler.s(10.0),
                del_btn_w,
                scaler.s(30.0),
                "DEL",
                Palette::UI_CARD_BG,
                Palette::RED,
                mouse_pos,
                clicked,
            );

            if del_clicked || (is_hover && (is_key_pressed(KeyCode::Backspace) || is_key_pressed(KeyCode::Delete))) {
                chosen_action = Some(EditorAction::DeleteTrack(choice.track_id().to_string()));
            } else if is_hover && clicked {
                chosen_action = Some(EditorAction::OpenTrack(choice.clone()));
            }

            ty += item_h + scaler.s(6.0);
        }
    }

    // Footer Pagination & Navigation
    let foot_y = my + mh - scaler.s(42.0);

    // Left info
    fonts.draw_ui_regular(
        &format!("Page {}/{} ({} circuits)", *page + 1, total_pages, tracks.len()),
        item_x,
        foot_y + scaler.s(20.0),
        scaler.font_s(11.5),
        Palette::UI_TEXT_MUTED,
    );

    // Pagination buttons (center)
    if total_pages > 1 {
        let prev_x = sw * 0.5 - scaler.s(85.0);
        let next_x = sw * 0.5 + scaler.s(10.0);
        let nav_w = scaler.s(75.0);
        let nav_h = scaler.s(28.0);

        if *page > 0 {
            if draw_ui_btn(fonts, scaler, prev_x, foot_y + scaler.s(4.0), nav_w, nav_h, "< PREV", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                *page -= 1;
            }
        }
        if *page + 1 < total_pages {
            if draw_ui_btn(fonts, scaler, next_x, foot_y + scaler.s(4.0), nav_w, nav_h, "NEXT >", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked) {
                *page += 1;
            }
        }
    }

    chosen_action
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
        "[OK] All checks passed! Circuit is 100% race ready."
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

/// Renders modal asking the user how to handle unsaved changes before exiting.
fn render_unsaved_changes_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    state: &EditorState,
    active_modal: &mut EditorModal,
    mouse_pos: Vec2,
    clicked: bool,
) -> Option<EditorAction> {
    let mw = scaler.s(500.0);
    let mh = scaler.s(220.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::YELLOW, 2.0);

    fonts.draw_display_centered(
        "UNSAVED CHANGES",
        sw * 0.5,
        my + scaler.s(30.0),
        scaler.font_s(20.0),
        Palette::NEON_GOLD,
    );

    fonts.draw_ui_regular_centered(
        "You have unsaved changes in this circuit.",
        sw * 0.5,
        my + scaler.s(58.0),
        scaler.font_s(13.0),
        Palette::WHITE,
    );
    fonts.draw_ui_regular_centered(
        "Save changes before exiting, or discard and exit?",
        sw * 0.5,
        my + scaler.s(76.0),
        scaler.font_s(12.5),
        Palette::UI_TEXT_MUTED,
    );

    let btn_h = scaler.s(34.0);
    let btn_y1 = my + scaler.s(102.0);
    let btn_y2 = my + scaler.s(144.0);
    let full_btn_w = mw - scaler.s(40.0);
    let half_btn_w = (full_btn_w - scaler.s(12.0)) * 0.5;
    let btn_x = mx + scaler.s(20.0);

    // Save & Exit button (full width top button)
    let save_clicked = draw_ui_btn(
        fonts,
        scaler,
        btn_x,
        btn_y1,
        full_btn_w,
        btn_h,
        "SAVE & EXIT [S / Enter]",
        Color::new(0.12, 0.65, 0.32, 0.95),
        Palette::NEON_GREEN,
        mouse_pos,
        clicked,
    );

    // Discard Changes & Exit button (left half)
    let discard_clicked = draw_ui_btn(
        fonts,
        scaler,
        btn_x,
        btn_y2,
        half_btn_w,
        btn_h,
        "DISCARD & EXIT [D]",
        Color::new(0.45, 0.10, 0.10, 0.95),
        Palette::RED,
        mouse_pos,
        clicked,
    );

    // Cancel / Keep Editing button (right half)
    let cancel_clicked = draw_ui_btn(
        fonts,
        scaler,
        btn_x + half_btn_w + scaler.s(12.0),
        btn_y2,
        half_btn_w,
        btn_h,
        "CANCEL [Esc]",
        Palette::UI_CARD_BG,
        Palette::UI_CARD_BORDER,
        mouse_pos,
        clicked,
    );

    let save_pressed = is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Enter) || save_clicked;
    let discard_pressed = is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::X) || discard_clicked;
    let cancel_pressed = is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::C) || cancel_clicked;

    if save_pressed {
        drain_char_queue();
        if let Some(ref p) = state.current_file_path {
            let stem = std::path::Path::new(p)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            return Some(EditorAction::SaveTrack {
                name: state.track.name.clone(),
                filename: stem,
                description: state.track.description.clone(),
                overwrite: true,
                exit_after: true,
            });
        } else {
            let initial_filename = TrackManager::sanitize_slug(&state.track.name);
            *active_modal = EditorModal::SaveAs {
                input_name: state.track.name.clone(),
                input_filename: initial_filename,
                input_description: state.track.description.clone(),
                active_field: 0,
                overwrite: false,
                custom_filename_edited: false,
                exit_on_save: true,
            };
            return None;
        }
    }

    if discard_pressed {
        return Some(EditorAction::ExitToMenu);
    }

    if cancel_pressed {
        *active_modal = EditorModal::None;
        return None;
    }

    None
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
    let mh = scaler.s(560.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    scaler.draw_glass_card(mx, my, mw, mh, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.0);

    fonts.draw_display_centered("EDITOR CONTROLS & SHORTCUTS", sw * 0.5, my + scaler.s(32.0), scaler.font_s(22.0), Palette::NEON_GOLD);

    let shortcuts = [
        ("Tools 1-8", "Switch between Select, Spline, Surface, Ramp, Obstacle, Checkpoint, Grid, Pit"),
        ("Left Click / Drag", "Select entity / Click & drag area to box-select / Drag to move"),
        ("Right Click / Drag", "Place active element (Waypoints, Zones, Ramps, Props, Gates, Grid, Pit)"),
        ("Ctrl + S / C / T / P", "Select surface shape (Square, Circle, Triangle, Polygon)"),
        ("Ctrl + F / Ctrl + B", "Move surface zone to FRONT (Above Track) or BACK (Below Track)"),
        ("B / [ / ]", "Adjust Banking on selected waypoint(s) (+/- 1°, Shift for 5°, B to cycle presets)"),
        ("R / Shift+R", "Rotate selected Jump Ramp (+/- 15°)"),
        ("Arrow Keys / WASD", "Pan camera across circuit canvas (+Shift for fast pan)"),
        ("Middle Drag", "Pan editor camera across canvas (or Right Drag in Select tool)"),
        ("+ / - Keys", "Progressive zoom in / zoom out (+Shift for fast zoom)"),
        ("Mouse Scroll Wheel", "Zoom in / Zoom out centered on cursor position"),
        ("Tab Key", "Cycle zoom levels (Close, Medium, Far, Overview)"),
        ("Space / Enter", "Instant Test Drive (Race car directly from starting grid)"),
        ("Ctrl + D", "Duplicate selected entity (obstacle, zone, ramp, waypoint, etc.)"),
        ("Ctrl + Z / Ctrl + Y", "Undo / Redo state modifications"),
        ("Delete / Backspace", "Delete selected waypoint, surface zone, ramp, or prop"),
        ("F Key", "Focus and frame the entire circuit bounds within viewport"),
        ("G Key", "Cycle CAD metric grid snap (Off, 1m, 2.5m, 5m, 10m)"),
        ("Esc / E", "Exit track editor / Cancel vertex drawing (prompts to save if unsaved)"),
    ];

    let mut sy = my + scaler.s(64.0);
    for (key, desc) in shortcuts {
        fonts.draw_ui_bold(key, mx + scaler.s(24.0), sy, scaler.font_s(12.5), Palette::NEON_CYAN);
        fonts.draw_ui_regular(desc, mx + scaler.s(180.0), sy, scaler.font_s(12.0), Palette::WHITE);
        sy += scaler.s(25.0);
    }

    draw_ui_btn(fonts, scaler, mx + (mw - scaler.s(160.0)) * 0.5, my + mh - scaler.s(44.0), scaler.s(160.0), scaler.s(34.0), "CLOSE [Esc]", Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, mouse_pos, clicked)
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
            input_filename: "my_custom_circuit".to_string(),
            input_description: "A fast flow circuit.".to_string(),
            active_field: 0,
            overwrite: true,
            custom_filename_edited: true,
            exit_on_save: true,
        };
        if let EditorModal::SaveAs {
            input_name,
            input_filename,
            input_description,
            active_field,
            overwrite,
            custom_filename_edited,
            exit_on_save,
        } = &modal {
            assert_eq!(input_name, "My Custom Circuit");
            assert_eq!(input_filename, "my_custom_circuit");
            assert_eq!(input_description, "A fast flow circuit.");
            assert_eq!(*active_field, 0);
            assert!(overwrite);
            assert!(custom_filename_edited);
            assert!(exit_on_save);
        } else {
            panic!("Expected SaveAs modal");
        }

        modal = EditorModal::OpenTrack {
            selected_tab: 2,
            page: 1,
        };
        if let EditorModal::OpenTrack { selected_tab, page } = &modal {
            assert_eq!(*selected_tab, 2);
            assert_eq!(*page, 1);
        } else {
            panic!("Expected OpenTrack modal");
        }

        modal = EditorModal::Diagnostics;
        assert_eq!(modal, EditorModal::Diagnostics);

        modal = EditorModal::Help;
        assert_eq!(modal, EditorModal::Help);

        modal = EditorModal::UnsavedChanges;
        assert_eq!(modal, EditorModal::UnsavedChanges);

        modal = EditorModal::SetRampAngle {
            input_angle: "135.5".to_string(),
        };
        if let EditorModal::SetRampAngle { input_angle } = &modal {
            assert_eq!(input_angle, "135.5");
        } else {
            panic!("Expected SetRampAngle modal");
        }
    }

    #[test]
    fn test_editor_actions_variants() {
        let act = EditorAction::SetTool(EditorToolType::RoadSpline);
        assert_eq!(act, EditorAction::SetTool(EditorToolType::RoadSpline));

        let act_snap = EditorAction::SetSnap(GridSnapSetting::Snap5m);
        assert_eq!(act_snap, EditorAction::SetSnap(GridSnapSetting::Snap5m));

        let act_save = EditorAction::SaveTrack {
            name: "monaco_gp".to_string(),
            filename: "monaco_gp".to_string(),
            description: "Street circuit in Monte Carlo.".to_string(),
            overwrite: true,
            exit_after: true,
        };
        assert_eq!(
            act_save,
            EditorAction::SaveTrack {
                name: "monaco_gp".to_string(),
                filename: "monaco_gp".to_string(),
                description: "Street circuit in Monte Carlo.".to_string(),
                overwrite: true,
                exit_after: true,
            }
        );

        let act_open = EditorAction::OpenTrack(crate::ui::menu::TrackChoice::ClassicGrandPrix);
        assert_eq!(
            act_open,
            EditorAction::OpenTrack(crate::ui::menu::TrackChoice::ClassicGrandPrix)
        );

        let act_exit = EditorAction::ExitToMenu;
        assert_eq!(act_exit, EditorAction::ExitToMenu);
    }
}
