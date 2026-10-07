use glam::Vec2;
use macroquad::color::Color;
use macroquad::input::{
    get_char_pressed, is_key_down, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed,
    mouse_position, KeyCode, MouseButton,
};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use macroquad::window::{screen_height, screen_width};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{JumpRamp, SurfaceLayer};
use tdrace_core::track::presets::{RaceDirection, TrackShape};
use tdrace_core::track::validation::{validate_track, ValidationSeverity};

use crate::editor::camera::EditorCamera;
use crate::editor::inspector::{
    apply_edit, begin_inspector_frame, build_inspector, content_height, row_height, Action as InspectorAction, Common, Edit,
    InspectorModel, Options, Prop, Row, StepperDrag, BODY_PAD, FOOTER_H, HEADER_H, RAMP_PROFILE_H, ROW_H, SECTION_GAP, SECTION_HEADER_H,
    TITLE_H,
};
use crate::editor::state::{EditorState, GridSnapSetting, Selection};
use crate::editor::tools::{EditorToolType, SurfaceShapeType, ToolSettings};
use crate::render::color::Palette;
use crate::track_manager::TrackManager;
use cabinet::input::GamepadSnapshot;
use cabinet::ui::{
    draw_action_button, draw_field_dropdown, draw_field_dropdown_popup, draw_segmented, segment_at, FieldDropdownEvent,
    FieldDropdownInput, LayoutRect, ModalContainer, PageDots, TextInputWidget, Toggle,
};
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
    Templates {
        selected_shape: TrackShape,
        selected_direction: RaceDirection,
        selected_module_idx: usize,
    },
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
        selected_idx: usize,
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
    Warning {
        title: String,
        message: String,
    },
}

impl EditorModal {
    pub fn templates_default(module_id: Option<&str>) -> Self {
        let mod_idx = match module_id {
            Some("gt") | Some("gt_challenge") => 1,
            Some("kart") => 2,
            Some("rally") => 3,
            _ => 0,
        };
        Self::Templates {
            selected_shape: TrackShape::Oval,
            selected_direction: RaceDirection::Right,
            selected_module_idx: mod_idx,
        }
    }
}

/// Actions dispatched from editor UI interactions.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorAction {
    None,
    SetTool(EditorToolType),
    SetSnap(GridSnapSetting),
    CycleZoom,
    NewFromTemplate(String),
    NewTrack {
        shape: TrackShape,
        direction: RaceDirection,
        module_id: String,
    },
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
    ExitToTrackManager,
}

/// Helper to drain any unconsumed characters from macroquad input buffer.
fn drain_char_queue() {
    while std::panic::catch_unwind(get_char_pressed).unwrap_or(None).is_some() {}
}

/// Returns true if the screen mouse coordinate is over any floating editor UI element or active modal.
pub fn is_mouse_over_editor_ui(
    mouse_pos: Vec2,
    sw: f32,
    sh: f32,
    active_tool: EditorToolType,
    is_modal_open: bool,
) -> bool {
    if is_modal_open {
        return true;
    }

    let scaler = UiScaler::new(sw, sh);
    let top_h = scaler.s(46.0);
    let bot_h = scaler.s(32.0);
    let bot_y = sh - bot_h;

    // Top toolbar
    if mouse_pos.y <= top_h {
        return true;
    }

    // Bottom status bar
    if mouse_pos.y >= bot_y {
        return true;
    }

    // Left tool palette (and active sub-palette if applicable)
    let tool_w = scaler.s(165.0);
    let tool_y = top_h + scaler.s(12.0);
    let tool_h = scaler.s(520.0);
    let has_sub = matches!(
        active_tool,
        EditorToolType::SurfaceZone
            | EditorToolType::ArenaFloor
            | EditorToolType::WhoopSection
            | EditorToolType::StuntRamp
            | EditorToolType::RoadSplit
    );
    let tool_bottom = if has_sub {
        let sub_h = scaler.s(180.0);
        let sub_y = tool_y + tool_h + scaler.s(8.0);
        sub_y + sub_h
    } else {
        tool_y + tool_h
    };
    let tool_right = scaler.s(12.0) + tool_w + scaler.s(6.0);
    if mouse_pos.x <= tool_right && mouse_pos.y >= top_h && mouse_pos.y <= tool_bottom + scaler.s(6.0) {
        return true;
    }

    // Right inspector panel
    let insp_w = scaler.s(240.0);
    let insp_x = sw - insp_w - scaler.s(12.0);
    if mouse_pos.x >= insp_x - scaler.s(4.0) && mouse_pos.y >= top_h {
        return true;
    }

    false
}

/// Main UI renderer for the Track Editor suite.
pub fn render_editor_ui(
    fonts: &Fonts,
    state: &mut EditorState,
    tools: &mut ToolSettings,
    camera: &mut EditorCamera,
    track_manager: &mut TrackManager,
    active_modal: &mut EditorModal,
    gamepad: &GamepadSnapshot,
) -> EditorAction {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);
    let (mx, my) = mouse_position();
    let mouse_pos = Vec2::new(mx, my);
    let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);

    let is_modal_open = *active_modal != EditorModal::None;
    let bg_mouse_clicked = mouse_clicked && !is_modal_open;

    // Drain accumulated characters whenever no text-input modal or inline bar editing is active
    if !tools.is_editing_text() && !matches!(*active_modal, EditorModal::SaveAs { .. } | EditorModal::SetRampAngle { .. } | EditorModal::SetRampProperty { .. }) {
        drain_char_queue();
    }

    let mut dispatched_action = EditorAction::None;

    // One Escape does one thing: cancelling a polygon or an inline value edit must not also exit.
    let escape_consumed = std::mem::take(&mut tools.escape_consumed) || tools.is_editing_text();

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
        *active_modal = EditorModal::templates_default(state.track.module_id.as_deref());
    }
    tb_x += scaler.s(72.0);

    if draw_ui_btn(fonts, &scaler, tb_x, scaler.s(8.0), scaler.s(65.0), scaler.s(30.0), "OPEN", Palette::UI_CARD_BG, Palette::NEON_CYAN, mouse_pos, bg_mouse_clicked) {
        let _ = track_manager.scan_custom_tracks();
        *active_modal = EditorModal::OpenTrack {
            selected_tab: 0,
            page: 0,
            selected_idx: 0,
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
            dispatched_action = EditorAction::ExitToTrackManager;
        }
    }

    // 2. LEFT TOOL PALETTE
    let tool_w = scaler.s(165.0);
    let tool_y = top_h + scaler.s(12.0);
    let tool_h = scaler.s(520.0);
    scaler.draw_glass_card(scaler.s(12.0), tool_y, tool_w, tool_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

    fonts.draw_ui_bold(
        "TOOLS [1-0,-]",
        scaler.s(22.0),
        tool_y + scaler.s(16.0),
        scaler.font_s(13.0),
        Palette::NEON_CYAN,
    );

    let tools_list = [
        (EditorToolType::Select, "[1] Select & Move"),
        (EditorToolType::RoadSpline, "[2] Road Spline"),
        (EditorToolType::RoadSplit, "[3] Road Split"),
        (EditorToolType::SurfaceZone, "[4] Surface Zone"),
        (EditorToolType::JumpRamp, "[5] Jump Ramp"),
        (EditorToolType::Obstacle, "[6] Obstacle Prop"),
        (EditorToolType::Checkpoint, "[7] Checkpoint Gate"),
        (EditorToolType::PitLane, "[8] Pit Lane"),
        (EditorToolType::ArenaFloor, "[9] Arena Floor"),
        (EditorToolType::WhoopSection, "[0] Whoops Moguls"),
        (EditorToolType::StuntRamp, "[-] Stunt Mega Ramp"),
    ];

    let mut item_y = tool_y + scaler.s(26.0);
    for (tool_type, label) in tools_list {
        let is_active = tools.active_tool == tool_type;
        let bg_col = if is_active { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG };
        let border_col = if is_active { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };

        if draw_ui_btn(fonts, &scaler, scaler.s(20.0), item_y, tool_w - scaler.s(16.0), scaler.s(36.0), label, bg_col, border_col, mouse_pos, bg_mouse_clicked) {
            tools.active_tool = tool_type;
        }
        item_y += scaler.s(41.0);
    }

    // 2b. Road Split Active Sub-Palette (when Road Split tool is active)
    if tools.active_tool == EditorToolType::RoadSplit {
        let sub_h = scaler.s(160.0);
        let sub_y = tool_y + tool_h + scaler.s(8.0);
        scaler.draw_glass_card(scaler.s(12.0), sub_y, tool_w, sub_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

        fonts.draw_ui_bold("ROAD SPLIT", scaler.s(22.0), sub_y + scaler.s(16.0), scaler.font_s(11.5), Palette::NEON_CYAN);

        let mut curr_sub_y = sub_y + scaler.s(22.0);
        fonts.draw_ui_regular(&format!("Angle: {:.0}°", tools.split_divergence_angle), scaler.s(22.0), curr_sub_y + scaler.s(12.0), scaler.font_s(10.5), Palette::WHITE);
        curr_sub_y += scaler.s(16.0);

        let angles = [15.0, 30.0, 45.0, 60.0];
        let btn_w = (tool_w - scaler.s(24.0) - scaler.s(6.0)) * 0.5;
        for chunk in angles.chunks(2) {
            let a1 = chunk[0];
            let is_a1 = (tools.split_divergence_angle - a1).abs() < 1.0;
            if draw_ui_btn(fonts, &scaler, scaler.s(18.0), curr_sub_y, btn_w, scaler.s(20.0), &format!("{:.0}°", a1), if is_a1 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, if is_a1 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER }, mouse_pos, bg_mouse_clicked) {
                tools.split_divergence_angle = a1;
            }
            if chunk.len() > 1 {
                let a2 = chunk[1];
                let is_a2 = (tools.split_divergence_angle - a2).abs() < 1.0;
                if draw_ui_btn(fonts, &scaler, scaler.s(18.0) + btn_w + scaler.s(6.0), curr_sub_y, btn_w, scaler.s(20.0), &format!("{:.0}°", a2), if is_a2 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, if is_a2 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER }, mouse_pos, bg_mouse_clicked) {
                    tools.split_divergence_angle = a2;
                }
            }
            curr_sub_y += scaler.s(24.0);
        }

        if let Some(sock) = tools.active_branch_socket {
            fonts.draw_ui_bold(&format!("Branch J{}#{}", sock.junction_id.0, sock.socket_index), scaler.s(22.0), curr_sub_y + scaler.s(12.0), scaler.font_s(10.5), Palette::NEON_GOLD);
            curr_sub_y += scaler.s(16.0);
            if draw_ui_btn(fonts, &scaler, scaler.s(18.0), curr_sub_y, tool_w - scaler.s(12.0), scaler.s(22.0), "Deselect Branch [Esc]", Palette::UI_CARD_BG, Palette::RED, mouse_pos, bg_mouse_clicked) {
                tools.active_branch_socket = None;
            }
        } else {
            fonts.draw_ui_regular("Right-Click: Split", scaler.s(22.0), curr_sub_y + scaler.s(12.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
            curr_sub_y += scaler.s(14.0);
            fonts.draw_ui_regular("Click Socket: Extend", scaler.s(22.0), curr_sub_y + scaler.s(12.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
        }
    }

    // 2c. Surface Zone Active Sub-Palette (when Surface Zone tool is active)
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

    // 2c. Arena Floor Active Sub-Palette
    if tools.active_tool == EditorToolType::ArenaFloor {
        let sub_h = scaler.s(180.0);
        let sub_y = tool_y + tool_h + scaler.s(8.0);
        scaler.draw_glass_card(scaler.s(12.0), sub_y, tool_w, sub_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

        fonts.draw_ui_bold("ARENA FLOOR", scaler.s(22.0), sub_y + scaler.s(16.0), scaler.font_s(11.5), Palette::NEON_GOLD);

        let surfaces = [
            (SurfaceType::Dirt, "Dirt"),
            (SurfaceType::Gravel, "Gravel"),
            (SurfaceType::MudTrack, "Mud Track"),
            (SurfaceType::DeepMud, "Deep Mud"),
            (SurfaceType::PackedSand, "Packed Sand"),
            (SurfaceType::DeepSand, "Deep Sand"),
            (SurfaceType::PackedSnow, "Packed Snow"),
            (SurfaceType::DeepSnow, "Deep Snow"),
            (SurfaceType::SheetIce, "Sheet Ice"),
            (SurfaceType::Asphalt, "Asphalt"),
            (SurfaceType::Concrete, "Concrete"),
            (SurfaceType::Grass, "Grass"),
        ];
        let half_w = (tool_w - scaler.s(22.0)) * 0.5;
        let mut btn_y = sub_y + scaler.s(22.0);
        for chunk in surfaces.chunks(2) {
            let (st1, l1) = chunk[0];
            let active1 = tools.active_surface == st1;
            if draw_ui_btn(fonts, &scaler, scaler.s(18.0), btn_y, half_w, scaler.s(22.0), l1, if active1 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, if active1 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER }, mouse_pos, bg_mouse_clicked) {
                tools.active_surface = st1;
            }
            if chunk.len() > 1 {
                let (st2, l2) = chunk[1];
                let active2 = tools.active_surface == st2;
                if draw_ui_btn(fonts, &scaler, scaler.s(20.0) + half_w, btn_y, half_w, scaler.s(22.0), l2, if active2 { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, if active2 { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER }, mouse_pos, bg_mouse_clicked) {
                    tools.active_surface = st2;
                }
            }
            btn_y += scaler.s(25.0);
        }

        if tools.active_polygon_vertices.len() >= 3 {
            if draw_ui_btn(fonts, &scaler, scaler.s(18.0), btn_y + scaler.s(4.0), tool_w - scaler.s(16.0), scaler.s(22.0), "Close Arena Hull", Palette::UI_CARD_BG, Palette::NEON_GREEN, mouse_pos, bg_mouse_clicked) {
                tools.finalize_arena_hull(state);
            }
            btn_y += scaler.s(24.0);
        }
        if !tools.active_polygon_vertices.is_empty() {
            if draw_ui_btn(fonts, &scaler, scaler.s(18.0), btn_y + scaler.s(4.0), tool_w - scaler.s(16.0), scaler.s(22.0), "Clear Vertices", Palette::UI_CARD_BG, Palette::RED, mouse_pos, bg_mouse_clicked) {
                tools.active_polygon_vertices.clear();
            }
        }
    }

    // 2d. Whoop Section Active Sub-Palette
    if tools.active_tool == EditorToolType::WhoopSection {
        let sub_h = scaler.s(160.0);
        let sub_y = tool_y + tool_h + scaler.s(8.0);
        scaler.draw_glass_card(scaler.s(12.0), sub_y, tool_w, sub_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

        fonts.draw_ui_bold("WHOOPS CONFIG", scaler.s(22.0), sub_y + scaler.s(16.0), scaler.font_s(11.5), Palette::NEON_CYAN);

        fonts.draw_ui_regular(&format!("Spacing: {:.1}m", tools.whoop_spacing), scaler.s(22.0), sub_y + scaler.s(32.0), scaler.font_s(10.5), Palette::WHITE);
        let third_w = (tool_w - scaler.s(26.0)) / 3.0;
        let mut btn_y = sub_y + scaler.s(38.0);
        let spacings = [3.5, 5.0, 6.5];
        for (i, &sp) in spacings.iter().enumerate() {
            let active = (tools.whoop_spacing - sp).abs() < 0.1;
            if draw_ui_btn(fonts, &scaler, scaler.s(18.0) + (third_w + scaler.s(4.0)) * i as f32, btn_y, third_w, scaler.s(20.0), &format!("{:.1}m", sp), if active { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, if active { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER }, mouse_pos, bg_mouse_clicked) {
                tools.whoop_spacing = sp;
            }
        }

        btn_y += scaler.s(25.0);
        fonts.draw_ui_regular(&format!("Height: {:.2}m", tools.whoop_height), scaler.s(22.0), btn_y + scaler.s(10.0), scaler.font_s(10.5), Palette::WHITE);
        btn_y += scaler.s(16.0);
        let heights = [0.5, 0.7, 1.0];
        for (i, &h) in heights.iter().enumerate() {
            let active = (tools.whoop_height - h).abs() < 0.05;
            if draw_ui_btn(fonts, &scaler, scaler.s(18.0) + (third_w + scaler.s(4.0)) * i as f32, btn_y, third_w, scaler.s(20.0), &format!("{:.1}m", h), if active { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, if active { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER }, mouse_pos, bg_mouse_clicked) {
                tools.whoop_height = h;
            }
        }
    }

    // 2e. Stunt Mega Ramp Active Sub-Palette
    if tools.active_tool == EditorToolType::StuntRamp {
        let sub_h = scaler.s(160.0);
        let sub_y = tool_y + tool_h + scaler.s(8.0);
        scaler.draw_glass_card(scaler.s(12.0), sub_y, tool_w, sub_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

        fonts.draw_ui_bold("STUNT MEGA RAMP", scaler.s(22.0), sub_y + scaler.s(16.0), scaler.font_s(11.5), Palette::NEON_MAGENTA);

        fonts.draw_ui_regular(&format!("Height: {:.1}m", tools.stunt_ramp_height), scaler.s(22.0), sub_y + scaler.s(32.0), scaler.font_s(10.5), Palette::WHITE);
        let third_w = (tool_w - scaler.s(26.0)) / 3.0;
        let mut btn_y = sub_y + scaler.s(38.0);
        let heights = [2.5, 3.5, 5.0];
        for (i, &h) in heights.iter().enumerate() {
            let active = (tools.stunt_ramp_height - h).abs() < 0.1;
            if draw_ui_btn(fonts, &scaler, scaler.s(18.0) + (third_w + scaler.s(4.0)) * i as f32, btn_y, third_w, scaler.s(20.0), &format!("{:.1}m", h), if active { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, if active { Palette::NEON_MAGENTA } else { Palette::UI_CARD_BORDER }, mouse_pos, bg_mouse_clicked) {
                tools.stunt_ramp_height = h;
            }
        }

        btn_y += scaler.s(25.0);
        fonts.draw_ui_regular(&format!("Boost: {:.1}x", tools.stunt_ramp_multiplier), scaler.s(22.0), btn_y + scaler.s(10.0), scaler.font_s(10.5), Palette::WHITE);
        btn_y += scaler.s(16.0);
        let boosts = [1.2, 1.5, 1.8];
        for (i, &b) in boosts.iter().enumerate() {
            let active = (tools.stunt_ramp_multiplier - b).abs() < 0.05;
            if draw_ui_btn(fonts, &scaler, scaler.s(18.0) + (third_w + scaler.s(4.0)) * i as f32, btn_y, third_w, scaler.s(20.0), &format!("{:.1}x", b), if active { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, if active { Palette::NEON_MAGENTA } else { Palette::UI_CARD_BORDER }, mouse_pos, bg_mouse_clicked) {
                tools.stunt_ramp_multiplier = b;
            }
        }
    }

    // 3. RIGHT INSPECTOR PANEL (when entity or circuit is selected)
    let insp_w = scaler.s(240.0);
    let insp_x = sw - insp_w - scaler.s(12.0);
    let insp_y = top_h + scaler.s(12.0);
    let insp_h = sh - top_h - scaler.s(54.0);

    scaler.draw_glass_card(insp_x, insp_y, insp_w, insp_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

    let over_inspector = !is_modal_open
        && mouse_pos.x >= insp_x
        && mouse_pos.x <= insp_x + insp_w
        && mouse_pos.y >= insp_y
        && mouse_pos.y <= insp_y + insp_h;
    begin_inspector_frame(state, tools, over_inspector, bg_mouse_clicked, is_mouse_button_down(MouseButton::Left));

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
        match active_modal {
            EditorModal::Templates {
                ref mut selected_shape,
                ref mut selected_direction,
                ref mut selected_module_idx,
            } => {
                if let Some(action) = render_template_modal(
                    fonts,
                    &scaler,
                    sw,
                    sh,
                    mouse_pos,
                    mouse_clicked,
                    selected_shape,
                    selected_direction,
                    selected_module_idx,
                ) {
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
            EditorModal::OpenTrack {
                selected_tab,
                page,
                selected_idx,
            } => {
                if let Some(action) = render_open_modal(
                    fonts,
                    &scaler,
                    sw,
                    sh,
                    selected_tab,
                    page,
                    selected_idx,
                    track_manager,
                    mouse_pos,
                    mouse_clicked,
                    gamepad,
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
            EditorModal::Warning { title, message } => {
                if render_warning_modal(fonts, &scaler, sw, sh, title, message, mouse_pos, mouse_clicked) {
                    *active_modal = EditorModal::None;
                }
            }
            EditorModal::None => {}
        }

        let is_cancel = is_key_pressed(KeyCode::Escape)
            || gamepad.btn_cancel_pressed
            || gamepad.btn_b_pressed
            || gamepad.btn_back_pressed;
        if (is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) || gamepad.btn_confirm_pressed || gamepad.btn_cancel_pressed)
            && matches!(*active_modal, EditorModal::Warning { .. })
        {
            *active_modal = EditorModal::None;
        } else if is_cancel && *active_modal != EditorModal::None && *active_modal != EditorModal::UnsavedChanges {
            *active_modal = EditorModal::None;
        }
    } else {
        if is_key_pressed(KeyCode::Escape) && !escape_consumed && dispatched_action == EditorAction::None {
            if state.is_dirty {
                *active_modal = EditorModal::UnsavedChanges;
            } else {
                dispatched_action = EditorAction::ExitToTrackManager;
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
    h: f32,
    state: &mut EditorState,
    tools: &mut ToolSettings,
    mouse_pos: Vec2,
    clicked: bool,
    active_modal: &mut EditorModal,
) {
    if let Some(model) = build_inspector(state, tools) {
        render_inspector_model(fonts, scaler, (x, y, w, h), &model, state, tools, mouse_pos, clicked);
        return;
    }

    fonts.draw_ui_bold(
        "INSPECTOR",
        x + scaler.s(12.0),
        y + scaler.s(22.0),
        scaler.font_s(14.0),
        Palette::NEON_GOLD,
    );

    let mut curr_y = y + scaler.s(36.0);

    match state.selection {
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
        // Every other selection is drawn by the model-based inspector (`build_inspector`) above.
        _ => {}
    }
}

/// Renders starter template modal overlay with Shape, Direction, and Module selection.
fn render_template_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    mouse_pos: Vec2,
    clicked: bool,
    selected_shape: &mut TrackShape,
    selected_direction: &mut RaceDirection,
    selected_module_idx: &mut usize,
) -> Option<EditorAction> {
    let mw = scaler.s(600.0);
    let mh = scaler.s(510.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    ModalContainer::new("START NEW CIRCUIT", LayoutRect::new(mx, my, mw, mh)).draw(scaler, fonts, sw, sh);

    // Subtitle
    fonts.draw_ui_regular_centered("Choose circuit layout, race direction, and motorsport module defaults", sw * 0.5, my + scaler.s(48.0), scaler.font_s(12.5), Palette::UI_TEXT_MUTED);

    // Close button (X) top right
    let close_btn_w = scaler.s(28.0);
    let close_x = mx + mw - close_btn_w - scaler.s(16.0);
    let close_y = my + scaler.s(14.0);
    let close_hover = mouse_pos.x >= close_x && mouse_pos.x <= close_x + close_btn_w && mouse_pos.y >= close_y && mouse_pos.y <= close_y + close_btn_w;
    scaler.draw_glass_card(close_x, close_y, close_btn_w, close_btn_w, if close_hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, if close_hover { Palette::RED } else { Palette::UI_CARD_BORDER }, 1.0);
    fonts.draw_ui_bold_centered("X", close_x + close_btn_w * 0.5, close_y + scaler.s(19.0), scaler.font_s(13.0), if close_hover { Palette::RED } else { Palette::UI_TEXT_MUTED });
    if close_hover && clicked {
        return Some(EditorAction::None);
    }

    let mut cur_y = my + scaler.s(66.0);

    // 1. CIRCUIT SHAPE
    fonts.draw_ui_bold("1. CIRCUIT LAYOUT SHAPE", mx + scaler.s(24.0), cur_y + scaler.s(12.0), scaler.font_s(12.5), Palette::NEON_CYAN);
    cur_y += scaler.s(18.0);

    let shape_btn_w = (mw - scaler.s(58.0)) * 0.5;
    let shape_btn_h = scaler.s(44.0);
    let shapes = [
        (TrackShape::Oval, "OVAL SPEEDWAY", "High-speed 2-turn banked oval"),
        (TrackShape::HorizontalEight, "EIGHT (HORIZONTAL)", "Horizontal figure-8 with crossover"),
    ];

    for (i, (s, title, desc)) in shapes.iter().enumerate() {
        let bx = mx + scaler.s(24.0) + i as f32 * (shape_btn_w + scaler.s(10.0));
        let is_selected = *selected_shape == *s;
        let is_hover = mouse_pos.x >= bx && mouse_pos.x <= bx + shape_btn_w && mouse_pos.y >= cur_y && mouse_pos.y <= cur_y + shape_btn_h;
        let bg_col = if is_selected {
            Color::new(0.08, 0.22, 0.28, 0.95)
        } else if is_hover {
            Palette::UI_CARD_BG_HOVER
        } else {
            Palette::UI_PILL_BG
        };
        let border_col = if is_selected { Palette::NEON_CYAN } else if is_hover { Palette::WHITE } else { Palette::UI_CARD_BORDER };
        scaler.draw_glass_card(bx, cur_y, shape_btn_w, shape_btn_h, bg_col, border_col, if is_selected { 1.8 } else { 1.0 });

        fonts.draw_ui_bold(title, bx + scaler.s(14.0), cur_y + scaler.s(18.0), scaler.font_s(13.5), if is_selected { Palette::NEON_CYAN } else { Palette::WHITE });
        fonts.draw_ui_regular(desc, bx + scaler.s(14.0), cur_y + scaler.s(34.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);

        if is_hover && clicked {
            *selected_shape = *s;
        }
    }
    cur_y += shape_btn_h + scaler.s(14.0);

    // 2. RACE DIRECTION
    fonts.draw_ui_bold("2. RACE DIRECTION", mx + scaler.s(24.0), cur_y + scaler.s(12.0), scaler.font_s(12.5), Palette::NEON_CYAN);
    cur_y += scaler.s(18.0);

    let dir_btn_w = (mw - scaler.s(58.0)) * 0.5;
    let dir_btn_h = scaler.s(40.0);
    let dirs = [
        (RaceDirection::Right, "RIGHT (FORWARD +X)", "Drives rightwards from start straight"),
        (RaceDirection::Left, "LEFT (FORWARD -X)", "Drives leftwards from start straight"),
    ];

    for (i, (d, title, desc)) in dirs.iter().enumerate() {
        let bx = mx + scaler.s(24.0) + i as f32 * (dir_btn_w + scaler.s(10.0));
        let is_selected = *selected_direction == *d;
        let is_hover = mouse_pos.x >= bx && mouse_pos.x <= bx + dir_btn_w && mouse_pos.y >= cur_y && mouse_pos.y <= cur_y + dir_btn_h;
        let bg_col = if is_selected {
            Color::new(0.08, 0.22, 0.28, 0.95)
        } else if is_hover {
            Palette::UI_CARD_BG_HOVER
        } else {
            Palette::UI_PILL_BG
        };
        let border_col = if is_selected { Palette::NEON_CYAN } else if is_hover { Palette::WHITE } else { Palette::UI_CARD_BORDER };
        scaler.draw_glass_card(bx, cur_y, dir_btn_w, dir_btn_h, bg_col, border_col, if is_selected { 1.8 } else { 1.0 });

        fonts.draw_ui_bold(title, bx + scaler.s(14.0), cur_y + scaler.s(17.0), scaler.font_s(13.0), if is_selected { Palette::NEON_CYAN } else { Palette::WHITE });
        fonts.draw_ui_regular(desc, bx + scaler.s(14.0), cur_y + scaler.s(31.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);

        if is_hover && clicked {
            *selected_direction = *d;
        }
    }
    cur_y += dir_btn_h + scaler.s(14.0);

    // 3. MOTORSPORT MODULE DEFAULTS
    fonts.draw_ui_bold("3. MOTORSPORT MODULE DEFAULTS", mx + scaler.s(24.0), cur_y + scaler.s(12.0), scaler.font_s(12.5), Palette::NEON_CYAN);
    cur_y += scaler.s(18.0);

    let mod_defs: [(&str, &str, SurfaceType, SurfaceType, &str, &str, Color); 4] = [
        ("classic", "CLASSIC", SurfaceType::Asphalt, SurfaceType::Grass, "GT Sports Coupe (RWD)", "14m track ribbon", Palette::NEON_CYAN),
        ("gt", "GRAND TOURING CHALLENGE", SurfaceType::Asphalt, SurfaceType::Grass, "GT3 Evo Racer (600 BHP)", "15m track ribbon with curbs", Palette::RED),
        ("kart", "KARTING", SurfaceType::Asphalt, SurfaceType::Asphalt, "125cc Go-Kart (Direct)", "10m technical track ribbon", Palette::NEON_MAGENTA),
        ("rally", "RALLYCROSS", SurfaceType::Dirt, SurfaceType::Dirt, "WRC Rally Car (AWD)", "12m loose dirt ribbon", Palette::NEON_GOLD),
    ];

    let chip_w = (mw - scaler.s(72.0)) / 4.0;
    let chip_h = scaler.s(32.0);
    for (idx, (_, label, _, _, _, _, color)) in mod_defs.iter().enumerate() {
        let cx = mx + scaler.s(24.0) + idx as f32 * (chip_w + scaler.s(8.0));
        let is_selected = *selected_module_idx == idx;
        let is_hover = mouse_pos.x >= cx && mouse_pos.x <= cx + chip_w && mouse_pos.y >= cur_y && mouse_pos.y <= cur_y + chip_h;
        let bg_col = if is_selected {
            Color::new(color.r * 0.25, color.g * 0.25, color.b * 0.25, 0.95)
        } else if is_hover {
            Palette::UI_CARD_BG_HOVER
        } else {
            Palette::UI_PILL_BG
        };
        let border_col = if is_selected { *color } else if is_hover { Palette::WHITE } else { Palette::UI_CARD_BORDER };
        scaler.draw_glass_card(cx, cur_y, chip_w, chip_h, bg_col, border_col, if is_selected { 1.8 } else { 1.0 });
        fonts.draw_ui_bold_centered(label, cx + chip_w * 0.5, cur_y + scaler.s(20.0), scaler.font_s(11.5), if is_selected { *color } else { Palette::WHITE });

        if is_hover && clicked {
            *selected_module_idx = idx;
        }
    }
    cur_y += chip_h + scaler.s(8.0);

    // Selected Module Details Banner
    let active_mod = &mod_defs[*selected_module_idx % mod_defs.len()];
    let info_w = mw - scaler.s(48.0);
    let info_h = scaler.s(52.0);
    let info_x = mx + scaler.s(24.0);
    scaler.draw_glass_card(info_x, cur_y, info_w, info_h, Color::new(0.04, 0.07, 0.11, 0.90), active_mod.6, 1.0);

    let line1 = format!(
        "Track Ribbon: {}  •  Off-Track Terrain: {}  •  Width: {}",
        active_mod.2.name(),
        active_mod.3.name(),
        active_mod.5
    );
    let line2 = format!("Default Vehicle: {}", active_mod.4);
    fonts.draw_ui_bold(&line1, info_x + scaler.s(16.0), cur_y + scaler.s(20.0), scaler.font_s(11.5), Palette::WHITE);
    fonts.draw_ui_regular(&line2, info_x + scaler.s(16.0), cur_y + scaler.s(38.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
    cur_y += info_h + scaler.s(14.0);

    // 4. ACTION BUTTONS: [ CREATE CIRCUIT ] & [ CANCEL ]
    let create_btn_w = mw - scaler.s(180.0);
    let cancel_btn_w = scaler.s(120.0);
    let btn_h = scaler.s(42.0);
    let create_x = mx + scaler.s(24.0);
    let cancel_x = create_x + create_btn_w + scaler.s(12.0);

    let create_hover = mouse_pos.x >= create_x && mouse_pos.x <= create_x + create_btn_w && mouse_pos.y >= cur_y && mouse_pos.y <= cur_y + btn_h;
    scaler.draw_glass_card(
        create_x,
        cur_y,
        create_btn_w,
        btn_h,
        if create_hover { Palette::UI_CARD_BG_HOVER } else { Color::new(0.06, 0.20, 0.12, 0.95) },
        if create_hover { Palette::WHITE } else { Palette::NEON_GREEN },
        1.8,
    );
    let btn_label = format!("CREATE {} {} ({})", active_mod.1, selected_shape.label().to_uppercase(), selected_direction.label().to_uppercase());
    fonts.draw_ui_bold_centered(&btn_label, create_x + create_btn_w * 0.5, cur_y + scaler.s(26.0), scaler.font_s(13.5), Palette::NEON_GREEN);

    let cancel_hover = mouse_pos.x >= cancel_x && mouse_pos.x <= cancel_x + cancel_btn_w && mouse_pos.y >= cur_y && mouse_pos.y <= cur_y + btn_h;
    scaler.draw_glass_card(
        cancel_x,
        cur_y,
        cancel_btn_w,
        btn_h,
        if cancel_hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG },
        if cancel_hover { Palette::RED } else { Palette::UI_CARD_BORDER },
        1.0,
    );
    fonts.draw_ui_bold_centered("CANCEL", cancel_x + cancel_btn_w * 0.5, cur_y + scaler.s(26.0), scaler.font_s(12.5), if cancel_hover { Palette::RED } else { Palette::UI_TEXT_MUTED });

    if cancel_hover && clicked {
        return Some(EditorAction::None);
    }

    if create_hover && clicked {
        return Some(EditorAction::NewTrack {
            shape: *selected_shape,
            direction: *selected_direction,
            module_id: active_mod.0.to_string(),
        });
    }

    None
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

    // Draw ramp cross-section body fill with smooth progressive concave curve (t^2)
    let num_curve_segs = 32;
    let slope_col = border_opt.unwrap_or(Palette::NEON_GOLD);

    // 1. Incline curve body fill (sliced trapezoids under the quadratic curve)
    for i in 0..num_curve_segs {
        let t0 = i as f32 / num_curve_segs as f32;
        let t1 = (i + 1) as f32 / num_curve_segs as f32;
        let x0 = origin_x + t0 * inc_w;
        let y0 = ground_y - draw_h * (t0 * t0);
        let x1 = origin_x + t1 * inc_w;
        let y1 = ground_y - draw_h * (t1 * t1);

        macroquad::shapes::draw_triangle(
            macroquad::prelude::Vec2::new(x0, ground_y),
            macroquad::prelude::Vec2::new(x0, y0),
            macroquad::prelude::Vec2::new(x1, y1),
            base_col,
        );
        macroquad::shapes::draw_triangle(
            macroquad::prelude::Vec2::new(x0, ground_y),
            macroquad::prelude::Vec2::new(x1, y1),
            macroquad::prelude::Vec2::new(x1, ground_y),
            base_col,
        );
    }

    // 2. Tabletop flat rectangle body fill (if present)
    if flat_len > 0.05 && inc_w < draw_w - scaler.s(1.0) {
        macroquad::shapes::draw_rectangle(origin_x + inc_w, ground_y - draw_h, draw_w - inc_w, draw_h, base_col);
    }

    // 3. Subtle vertical guide contours along curved incline
    for div in [0.25f32, 0.50, 0.75] {
        let gx = origin_x + div * inc_w;
        let gy = ground_y - draw_h * (div * div);
        macroquad::shapes::draw_line(gx, ground_y, gx, gy, scaler.s(1.0), macroquad::color::Color::new(1.0, 1.0, 1.0, 0.12));
    }

    // 4. Smooth curved incline slope stroke
    for i in 0..num_curve_segs {
        let t0 = i as f32 / num_curve_segs as f32;
        let t1 = (i + 1) as f32 / num_curve_segs as f32;
        let x0 = origin_x + t0 * inc_w;
        let y0 = ground_y - draw_h * (t0 * t0);
        let x1 = origin_x + t1 * inc_w;
        let y1 = ground_y - draw_h * (t1 * t1);
        macroquad::shapes::draw_line(x0, y0, x1, y1, scaler.s(2.2), slope_col);
    }

    // 5. Tabletop stroke (if flat portion exists)
    if flat_len > 0.05 && inc_w < draw_w - scaler.s(1.0) {
        macroquad::shapes::draw_line(p_inc.x, p_inc.y, p_top_exit.x, p_top_exit.y, scaler.s(2.0), Palette::NEON_CYAN);
    }

    // 6. Exit launch lip (vertical drop line)
    macroquad::shapes::draw_line(p_top_exit.x, p_top_exit.y, p_bot_exit.x, p_bot_exit.y, scaler.s(2.0), Palette::NEON_CYAN);

    // Annotations text
    // Length (bottom)
    let len_str = format!("L:{:.1}m", len);
    fonts.draw_ui_regular(&len_str, origin_x + draw_w * 0.5 - scaler.s(16.0), ground_y + scaler.s(13.0), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);

    // Height (right vertical)
    let h_str = format!("H:{:.1}m", height);
    fonts.draw_ui_regular(&h_str, (p_top_exit.x - scaler.s(36.0)).max(origin_x), ground_y - draw_h * 0.5 + scaler.s(3.0), scaler.font_s(9.5), Palette::NEON_GOLD);

    // Pitch (along slope)
    let pitch_str = format!("{:.0}° (Curved)", ramp.ramp_angle_deg);
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

    let (title, subtitle, unit, min_val, max_val) = match property {
        RampPropertyModal::Angle => ("SET JUMP RAMP ANGLE", "Specify direction angle (0° – 360°) • [Enter] to apply", "°", 0.0, 360.0),
        RampPropertyModal::Length => ("SET JUMP RAMP LENGTH", "Specify total ramp length (2.0m – 50.0m) • [Enter] to apply", "m", 2.0, 50.0),
        RampPropertyModal::Width => ("SET JUMP RAMP WIDTH", "Specify ramp width (1.0m – 30.0m) • [Enter] to apply", "m", 1.0, 30.0),
        RampPropertyModal::Height => ("SET JUMP RAMP HEIGHT", "Specify launch height (0.2m – 10.0m) • [Enter] to apply", "m", 0.2, 10.0),
        RampPropertyModal::Pitch => ("SET JUMP RAMP PITCH", "Specify incline pitch angle (1.0° – 60.0°) • [Enter] to apply", "°", 1.0, 60.0),
        RampPropertyModal::LaunchSpeed => ("SET LAUNCH SPEED BOOST", "Specify vertical launch speed (1.0m/s – 20.0m/s) • [Enter] to apply", "m/s", 1.0, 20.0),
    };

    ModalContainer::new(title, LayoutRect::new(mx, my, mw, mh)).draw(scaler, fonts, sw, sh);
    fonts.draw_ui_regular_centered(subtitle, sw * 0.5, my + scaler.s(48.0), scaler.font_s(11.5), Palette::UI_TEXT_MUTED);

    // Text Input Display Box
    let box_x = mx + scaler.s(30.0);
    let box_y = my + scaler.s(68.0);
    let box_w = mw - scaler.s(60.0);
    let box_h = scaler.s(38.0);

    // Platform text input widget for numeric value entry.
    let mut input_widget = TextInputWidget::new(box_x, box_y, box_w, box_h, 8, "")
        .with_filter(|c| c.is_ascii_digit() || c == '.');
    input_widget.set_text(input_val);
    input_widget.is_focused = true;
    input_widget.is_active = true;
    let _ = input_widget.handle_input(macroquad::time::get_frame_time(), None);
    *input_val = input_widget.text.clone();
    input_widget.render_frame();

    let display_str = if input_val.is_empty() {
        format!("0.0{}", unit)
    } else {
        format!("{}{}", input_val, unit)
    };
    fonts.draw_ui_bold(&display_str, box_x + scaler.s(16.0), box_y + scaler.s(24.0), scaler.font_s(17.0), Palette::WHITE);

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

    let title_text = if exit_on_save {
        "SAVE & EXIT CIRCUIT"
    } else {
        "SAVE CIRCUIT"
    };
    ModalContainer::new(title_text, LayoutRect::new(mx, my, mw, mh)).draw(scaler, fonts, sw, sh);

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

    let is_preset = TrackManager::is_preset_slug(&slug) && !track_manager.is_preset_demoted(&slug);
    let dev_mode_blocked = *overwrite && is_preset && !crate::storage::is_dev_mode();
    let is_filename_conflict = !*overwrite && track_manager.track_file_exists(&slug);

    let target_display_path = if *overwrite {
        let p = if is_preset {
            track_manager
                .resolve_preset_git_file(&slug, None)
                .unwrap_or_else(|| track_manager.track_path_for_slug(&slug))
        } else {
            track_manager.track_path_for_slug(&slug)
        };
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
        format!("tracks/{}.json", slug)
    };

    let info_y = f3_box_y + f3_h + scaler.s(12.0);
    if dev_mode_blocked {
        fonts.draw_ui_bold(
            "⚠️ Official presets require Dev Mode to overwrite directly. Change filename to Save As Copy.",
            inp_x,
            info_y,
            scaler.font_s(11.0),
            Palette::YELLOW,
        );
    } else if *overwrite && is_preset {
        fonts.draw_ui_bold(
            &format!("Target: {} (Dev Mode: Updating repository preset & user storage)", target_display_path),
            inp_x,
            info_y,
            scaler.font_s(11.0),
            Palette::NEON_CYAN,
        );
    } else if *overwrite {
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
    let mut overwrite_toggle = Toggle::new("Overwrite", *overwrite);
    if draw_toggle(fonts, scaler, inp_x, toggle_y, inp_w, scaler.s(26.0), &overwrite_toggle, mouse_pos, clicked) {
        overwrite_toggle.toggle();
        *overwrite = overwrite_toggle.is_on;
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

    let (btn_title, btn_color, btn_border) = if dev_mode_blocked {
        (
            "DEV MODE REQUIRED TO OVERWRITE PRESET",
            Color::new(0.35, 0.08, 0.08, 0.95),
            Palette::RED,
        )
    } else if *overwrite {
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
    let can_submit = (!dev_mode_blocked) && (*overwrite || !is_filename_conflict);

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

/// User navigation inputs for the Open Circuit modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenCircuitNavInput {
    Up,
    Down,
    PagePrev,
    PageNext,
    TabPrev,
    TabNext,
    SetTab(usize),
}

/// Updates open circuit modal navigation state based on user inputs.
pub fn update_open_circuit_nav(
    selected_tab: &mut usize,
    page: &mut usize,
    selected_idx: &mut usize,
    total_tabs: usize,
    total_items: usize,
    items_per_page: usize,
    input: OpenCircuitNavInput,
) {
    if total_tabs == 0 {
        return;
    }
    let items_per_page = items_per_page.max(1);
    let total_pages = ((total_items + items_per_page - 1) / items_per_page).max(1);

    match input {
        OpenCircuitNavInput::Up => {
            if total_items > 0 {
                if *selected_idx == 0 {
                    *selected_idx = total_items - 1;
                } else {
                    *selected_idx -= 1;
                }
                *page = *selected_idx / items_per_page;
            }
        }
        OpenCircuitNavInput::Down => {
            if total_items > 0 {
                *selected_idx = (*selected_idx + 1) % total_items;
                *page = *selected_idx / items_per_page;
            }
        }
        OpenCircuitNavInput::PagePrev => {
            if *page > 0 {
                *page -= 1;
                *selected_idx = *page * items_per_page;
            }
        }
        OpenCircuitNavInput::PageNext => {
            if *page + 1 < total_pages {
                *page += 1;
                *selected_idx = (*page * items_per_page).min(total_items.saturating_sub(1));
            }
        }
        OpenCircuitNavInput::TabPrev => {
            *selected_tab = selected_tab.checked_sub(1).unwrap_or(total_tabs - 1);
            *page = 0;
            *selected_idx = 0;
        }
        OpenCircuitNavInput::TabNext => {
            *selected_tab = (*selected_tab + 1) % total_tabs;
            *page = 0;
            *selected_idx = 0;
        }
        OpenCircuitNavInput::SetTab(tab_idx) => {
            if tab_idx < total_tabs && *selected_tab != tab_idx {
                *selected_tab = tab_idx;
                *page = 0;
                *selected_idx = 0;
            }
        }
    }
}

/// Renders open track modal with tabs for all registered modules and drafts.
fn render_open_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    selected_tab: &mut usize,
    page: &mut usize,
    selected_idx: &mut usize,
    track_manager: &TrackManager,
    mouse_pos: Vec2,
    clicked: bool,
    gamepad: &GamepadSnapshot,
) -> Option<EditorAction> {
    let mw = scaler.s(680.0);
    let mh = scaler.s(490.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    ModalContainer::new("OPEN CIRCUIT", LayoutRect::new(mx, my, mw, mh)).draw(scaler, fonts, sw, sh);
    fonts.draw_ui_regular_centered(
        "Browse circuits across all registered motorsport modules and custom circuits",
        sw * 0.5,
        my + scaler.s(45.0),
        scaler.font_s(11.5),
        Palette::UI_TEXT_MUTED,
    );

    let tabs: [(&str, &str); 6] = [
        ("ALL", "all"),
        ("CLASSIC", "classic"),
        ("GT", "gt"),
        ("RALLYCROSS", "rally"),
        ("KARTING", "kart"),
        ("CUSTOM", "custom"),
    ];

    let items_per_page = 5;

    // Direct tab numeric hotkeys (1-6)
    let num_keys = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5, KeyCode::Key6];
    let kp_keys = [KeyCode::Kp1, KeyCode::Kp2, KeyCode::Kp3, KeyCode::Kp4, KeyCode::Kp5, KeyCode::Kp6];
    for (i, (&k, &kp)) in num_keys.iter().zip(kp_keys.iter()).enumerate() {
        if is_key_pressed(k) || is_key_pressed(kp) {
            update_open_circuit_nav(
                selected_tab,
                page,
                selected_idx,
                tabs.len(),
                0,
                items_per_page,
                OpenCircuitNavInput::SetTab(i),
            );
        }
    }

    // Tab cycling (Q / E / LB / RB / Tab / Shift+Tab)
    let tab_next = is_key_pressed(KeyCode::E)
        || is_key_pressed(KeyCode::RightBracket)
        || gamepad.btn_rb_pressed
        || (is_key_pressed(KeyCode::Tab) && !is_key_down(KeyCode::LeftShift) && !is_key_down(KeyCode::RightShift));
    let tab_prev = is_key_pressed(KeyCode::Q)
        || is_key_pressed(KeyCode::LeftBracket)
        || gamepad.btn_lb_pressed
        || (is_key_pressed(KeyCode::Tab) && (is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)));

    if tab_next {
        update_open_circuit_nav(
            selected_tab,
            page,
            selected_idx,
            tabs.len(),
            0,
            items_per_page,
            OpenCircuitNavInput::TabNext,
        );
    } else if tab_prev {
        update_open_circuit_nav(
            selected_tab,
            page,
            selected_idx,
            tabs.len(),
            0,
            items_per_page,
            OpenCircuitNavInput::TabPrev,
        );
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
            update_open_circuit_nav(
                selected_tab,
                page,
                selected_idx,
                tabs.len(),
                0,
                items_per_page,
                OpenCircuitNavInput::SetTab(idx),
            );
        }
    }

    // Get tracks for active tab
    let mod_id = tabs[*selected_tab].1;
    let tracks = if mod_id == "custom" {
        track_manager.custom_track_choices()
    } else {
        track_manager.module_catalog_tracks(mod_id)
    };

    let total_pages = ((tracks.len() + items_per_page - 1) / items_per_page).max(1);
    if *page >= total_pages {
        *page = total_pages - 1;
    }
    if tracks.is_empty() {
        *selected_idx = 0;
    } else if *selected_idx >= tracks.len() {
        *selected_idx = tracks.len() - 1;
    }

    // Circuit Navigation (Up / Down / W / S / Gamepad Stick & D-Pad)
    let nav_up = is_key_pressed(KeyCode::Up)
        || is_key_pressed(KeyCode::W)
        || gamepad.dpad_up_pressed
        || gamepad.nav_up;
    let nav_down = is_key_pressed(KeyCode::Down)
        || is_key_pressed(KeyCode::S)
        || gamepad.dpad_down_pressed
        || gamepad.nav_down;

    if nav_up {
        update_open_circuit_nav(
            selected_tab,
            page,
            selected_idx,
            tabs.len(),
            tracks.len(),
            items_per_page,
            OpenCircuitNavInput::Up,
        );
    } else if nav_down {
        update_open_circuit_nav(
            selected_tab,
            page,
            selected_idx,
            tabs.len(),
            tracks.len(),
            items_per_page,
            OpenCircuitNavInput::Down,
        );
    }

    // Page Navigation (Left / Right / PageUp / PageDown / Gamepad Left/Right)
    let page_prev = is_key_pressed(KeyCode::PageUp)
        || (total_pages > 1 && (is_key_pressed(KeyCode::Left) || gamepad.dpad_left_pressed || gamepad.nav_left));
    let page_next = is_key_pressed(KeyCode::PageDown)
        || (total_pages > 1 && (is_key_pressed(KeyCode::Right) || gamepad.dpad_right_pressed || gamepad.nav_right));

    if page_prev {
        update_open_circuit_nav(
            selected_tab,
            page,
            selected_idx,
            tabs.len(),
            tracks.len(),
            items_per_page,
            OpenCircuitNavInput::PagePrev,
        );
    } else if page_next {
        update_open_circuit_nav(
            selected_tab,
            page,
            selected_idx,
            tabs.len(),
            tracks.len(),
            items_per_page,
            OpenCircuitNavInput::PageNext,
        );
    } else if total_pages <= 1 {
        // Fallback when single page: Left and Right arrows switch tabs
        if is_key_pressed(KeyCode::Left) || gamepad.dpad_left_pressed || gamepad.nav_left {
            update_open_circuit_nav(
                selected_tab,
                page,
                selected_idx,
                tabs.len(),
                tracks.len(),
                items_per_page,
                OpenCircuitNavInput::TabPrev,
            );
        } else if is_key_pressed(KeyCode::Right) || gamepad.dpad_right_pressed || gamepad.nav_right {
            update_open_circuit_nav(
                selected_tab,
                page,
                selected_idx,
                tabs.len(),
                tracks.len(),
                items_per_page,
                OpenCircuitNavInput::TabNext,
            );
        }
    }

    let mut chosen_action = None;

    // Confirm action (Enter / Space / Gamepad A)
    let is_confirm = is_key_pressed(KeyCode::Enter)
        || is_key_pressed(KeyCode::KpEnter)
        || is_key_pressed(KeyCode::Space)
        || gamepad.btn_confirm_pressed
        || gamepad.btn_a_pressed;
    if is_confirm {
        if let Some(choice) = tracks.get(*selected_idx) {
            chosen_action = Some(EditorAction::OpenTrack(choice.clone()));
        }
    }

    // Delete action (Delete / Backspace / Gamepad X)
    let is_delete_key = is_key_pressed(KeyCode::Backspace)
        || is_key_pressed(KeyCode::Delete)
        || gamepad.btn_x_pressed;
    if is_delete_key && chosen_action.is_none() {
        if let Some(choice) = tracks.get(*selected_idx) {
            chosen_action = Some(EditorAction::DeleteTrack(choice.track_id().to_string()));
        }
    }

    let start_idx = *page * items_per_page;
    let end_idx = (start_idx + items_per_page).min(tracks.len());

    let mut ty = tab_y + tab_h + scaler.s(12.0);
    let item_w = mw - scaler.s(40.0);
    let item_h = scaler.s(50.0);
    let item_x = mx + scaler.s(20.0);

    if tracks.is_empty() {
        fonts.draw_ui_regular_centered(
            "No circuits found in this module catalog.",
            sw * 0.5,
            ty + scaler.s(60.0),
            scaler.font_s(13.0),
            Palette::UI_TEXT_MUTED,
        );
    } else {
        for (rel_idx, choice) in tracks[start_idx..end_idx].iter().enumerate() {
            let abs_idx = start_idx + rel_idx;
            let is_sel = *selected_idx == abs_idx;
            let is_hover = mouse_pos.x >= item_x && mouse_pos.x <= item_x + item_w && mouse_pos.y >= ty && mouse_pos.y <= ty + item_h;
            let is_active = is_sel || is_hover;

            let bg_col = if is_active {
                Palette::UI_CARD_BG_HOVER
            } else {
                Color::new(0.04, 0.06, 0.09, 0.95)
            };
            let border_col = if is_active {
                Palette::NEON_CYAN
            } else {
                Palette::UI_CARD_BORDER
            };

            scaler.draw_glass_card(item_x, ty, item_w, item_h, bg_col, border_col, if is_active { 1.8 } else { 1.0 });

            // Active card indicator bar on left edge
            if is_sel {
                macroquad::shapes::draw_rectangle(item_x, ty, scaler.s(3.5), item_h, Palette::NEON_CYAN);
            }

            // Title & Description
            fonts.draw_ui_bold(
                choice.title(),
                item_x + scaler.s(14.0),
                ty + scaler.s(20.0),
                scaler.font_s(13.5),
                if is_active { Palette::NEON_CYAN } else { Palette::WHITE },
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
            let badge_col = if tag.contains("GT") {
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

            if del_clicked {
                chosen_action = Some(EditorAction::DeleteTrack(choice.track_id().to_string()));
            } else if is_hover && clicked {
                *selected_idx = abs_idx;
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

    // Pagination indicator (platform PageDots)
    PageDots::new(*page, total_pages).draw(scaler, sw * 0.5, foot_y + scaler.s(18.0), 5.0, 10.0);

    // Controls helper hint (right)
    let hint_text = if gamepad.is_connected {
        "[D-Pad/Sticks] Select  •  [A] Open  •  [X] Del  •  [LB/RB] Tab"
    } else {
        "[↑/↓] Select  •  [Enter] Open  •  [Del] Del  •  [Q/E/Tab] Tab"
    };
    let hint_w = fonts.measure_ui_regular(hint_text, scaler.font_s(10.0)).width;
    let hint_x = item_x + item_w - hint_w;
    fonts.draw_ui_regular(
        hint_text,
        hint_x,
        foot_y + scaler.s(20.0),
        scaler.font_s(10.0),
        Palette::UI_TEXT_MUTED,
    );

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

    ModalContainer::new("CIRCUIT DIAGNOSTICS", LayoutRect::new(mx, my, mw, mh)).draw(scaler, fonts, sw, sh);

    let val = validate_track(&state.track);
    let is_valid = val.iter().all(|e| e.severity != ValidationSeverity::Error);

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

/// Renders a warning/alert dialog modal overlay.
fn render_warning_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    title: &str,
    message: &str,
    mouse_pos: Vec2,
    clicked: bool,
) -> bool {
    let mw = scaler.s(520.0);
    let mh = scaler.s(240.0);
    let mx = (sw - mw) * 0.5;
    let my = (sh - mh) * 0.5;

    ModalContainer::new(title, LayoutRect::new(mx, my, mw, mh)).draw(scaler, fonts, sw, sh);

    let mut ly = my + scaler.s(68.0);
    for line in message.lines() {
        if line.is_empty() {
            ly += scaler.s(8.0);
            continue;
        }
        let is_hint = line.starts_with('(');
        let col = if is_hint { Palette::UI_TEXT_MUTED } else { Palette::WHITE };
        let font_size = if is_hint { scaler.font_s(11.5) } else { scaler.font_s(13.0) };
        fonts.draw_ui_regular_centered(line, sw * 0.5, ly, font_size, col);
        ly += scaler.s(20.0);
    }

    let btn_w = scaler.s(160.0);
    let btn_h = scaler.s(34.0);
    let btn_x = (sw - btn_w) * 0.5;
    let btn_y = my + mh - scaler.s(48.0);

    draw_ui_btn(
        fonts,
        scaler,
        btn_x,
        btn_y,
        btn_w,
        btn_h,
        "OK [Esc / Enter]",
        Palette::UI_CARD_BG,
        Palette::NEON_CYAN,
        mouse_pos,
        clicked,
    )
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

    ModalContainer::new("UNSAVED CHANGES", LayoutRect::new(mx, my, mw, mh)).draw(scaler, fonts, sw, sh);

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
        return Some(EditorAction::ExitToTrackManager);
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

    ModalContainer::new("EDITOR CONTROLS & SHORTCUTS", LayoutRect::new(mx, my, mw, mh)).draw(scaler, fonts, sw, sh);

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
/// Rendering is delegated to the platform [`draw_action_button`] primitive; the button's
/// `border` colour doubles as its accent, and a non-default border marks an active/selected
/// control so the platform card keeps its accent glow even when not hovered.
fn draw_ui_btn(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    _bg: Color,
    border: Color,
    mouse_pos: Vec2,
    clicked: bool,
) -> bool {
    let is_hover = mouse_pos.x >= x && mouse_pos.x <= x + w && mouse_pos.y >= y && mouse_pos.y <= y + h;
    let is_focused = border != Palette::UI_CARD_BORDER;

    draw_action_button(scaler, fonts, x, y, w, h, label, None, is_focused, is_hover, border);

    is_hover && clicked
}

/// Renders a boolean flag as a platform [`Toggle`] action button, returning true when clicked.
fn draw_toggle(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    toggle: &Toggle,
    mouse_pos: Vec2,
    clicked: bool,
) -> bool {
    let is_hover = mouse_pos.x >= x && mouse_pos.x <= x + w && mouse_pos.y >= y && mouse_pos.y <= y + h;
    let accent = if toggle.is_on { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };
    let state = if toggle.is_on { "ON" } else { "OFF" };

    draw_action_button(scaler, fonts, x, y, w, h, toggle.label.as_str(), Some(state), toggle.is_on, is_hover, accent);

    is_hover && clicked
}

/// Returns the toggle button label for a checkpoint gate's finish line flag.
/// Consistently displays "Finish Line" regardless of whether the checkpoint is
/// currently designated as the finish line or a normal sector gate.
pub fn checkpoint_finish_line_label(is_finish: bool) -> &'static str {
    if is_finish {
        "[X] Finish Line"
    } else {
        "[ ] Finish Line"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_modals_state_transitions() {
        let mut modal = EditorModal::None;
        assert_eq!(modal, EditorModal::None);

        modal = EditorModal::templates_default(Some("classic"));
        assert_eq!(modal, EditorModal::templates_default(Some("classic")));

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
            selected_idx: 3,
        };
        if let EditorModal::OpenTrack { selected_tab, page, selected_idx } = &modal {
            assert_eq!(*selected_tab, 2);
            assert_eq!(*page, 1);
            assert_eq!(*selected_idx, 3);
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

        modal = EditorModal::Warning {
            title: "FINISH LINE REQUIRED".to_string(),
            message: "No finish line exists on this circuit.".to_string(),
        };
        if let EditorModal::Warning { title, message } = &modal {
            assert_eq!(title, "FINISH LINE REQUIRED");
            assert_eq!(message, "No finish line exists on this circuit.");
        } else {
            panic!("Expected Warning modal");
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

        let act_exit_tm = EditorAction::ExitToTrackManager;
        assert_eq!(act_exit_tm, EditorAction::ExitToTrackManager);
    }

    #[test]
    fn test_checkpoint_finish_line_label_consistency() {
        assert_eq!(checkpoint_finish_line_label(true), "[X] Finish Line");
        assert_eq!(checkpoint_finish_line_label(false), "[ ] Finish Line");
    }

    #[test]
    fn test_open_circuit_navigation() {
        let mut tab = 0;
        let mut page = 0;
        let mut sel = 0;
        let total_tabs = 6;
        let total_items = 93; // Matches screenshot: 93 circuits across 19 pages
        let items_per_page = 5;

        // Nav Down moves down item by item
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::Down);
        assert_eq!(sel, 1);
        assert_eq!(page, 0);

        // Advance to 5th item (last on page 0)
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::Down); // 2
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::Down); // 3
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::Down); // 4
        assert_eq!(sel, 4);
        assert_eq!(page, 0);

        // Moving down past the bottom of page 0 automatically flips to page 1 and selects item 5
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::Down);
        assert_eq!(sel, 5);
        assert_eq!(page, 1);

        // Moving up returns to page 0, item 4
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::Up);
        assert_eq!(sel, 4);
        assert_eq!(page, 0);

        // Page navigation
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::PageNext);
        assert_eq!(page, 1);
        assert_eq!(sel, 5);

        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::PagePrev);
        assert_eq!(page, 0);
        assert_eq!(sel, 0);

        // Wrap around on Up at top of list
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::Up);
        assert_eq!(sel, 92);
        assert_eq!(page, 18); // 19th page (0-indexed 18)

        // Wrap around on Down at end of list
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::Down);
        assert_eq!(sel, 0);
        assert_eq!(page, 0);

        // Tab switching resets page and selection
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::Down);
        assert_eq!(sel, 1);
        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::TabNext);
        assert_eq!(tab, 1);
        assert_eq!(page, 0);
        assert_eq!(sel, 0);

        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::TabPrev);
        assert_eq!(tab, 0);
        assert_eq!(page, 0);
        assert_eq!(sel, 0);

        update_open_circuit_nav(&mut tab, &mut page, &mut sel, total_tabs, total_items, items_per_page, OpenCircuitNavInput::SetTab(4));
        assert_eq!(tab, 4);
        assert_eq!(page, 0);
        assert_eq!(sel, 0);
    }
}

// ---------------------------------------------------------------------------------------------
// Model-based inspector (spec 086): header, collapsible sections in a clipped scrolling body,
// fixed footer, and the open surface dropdown drawn on top.
// ---------------------------------------------------------------------------------------------

/// Width of the label column of an inspector row, in reference pixels.
const INSP_LABEL_W: f32 = 58.0;
/// Width of one side toggle (L or R), in reference pixels.
const INSP_SIDE_W: f32 = 37.0;
/// Visible rows of an open inspector dropdown.
const INSP_DROPDOWN_ROWS: usize = 8;

fn point_in(p: Vec2, (x, y, w, h): (f32, f32, f32, f32)) -> bool {
    p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h
}

fn surface_swatch(surface: SurfaceType) -> Color {
    race_ui::render::track::get_surface_zone_colors(surface).0
}

/// Formats a stepper value; whole-number steps show no decimals.
fn format_stepper_value(v: f32, unit: &str, signed: bool, step: f32) -> String {
    let decimals = if step.fract() == 0.0 { 0 } else { 1 };
    if signed {
        format!("{v:+.decimals$}{unit}")
    } else {
        format!("{v:.decimals$}{unit}")
    }
}

/// Option list and current value of the dropdown row for `prop`, if the model has one.
fn dropdown_row(model: &InspectorModel, prop: Prop) -> Option<(Options, Common<usize>)> {
    model.sections.iter().flat_map(|s| &s.rows).find_map(|r| match r {
        Row::Dropdown { prop: p, options, value, .. } if *p == prop => Some((*options, *value)),
        _ => None,
    })
}

/// Inline text entry for a focused stepper. Returns a committed value, if any.
fn stepper_text_entry(tools: &mut ToolSettings, id: &str, min: f32, max: f32, clicked_outside: bool) -> Option<f32> {
    let mut commit = false;
    let mut stop = clicked_outside;
    commit |= clicked_outside;
    while let Some(c) = get_char_pressed() {
        if let Some((_, buf)) = tools.editing_bar.as_mut() {
            let ok = c.is_ascii_digit() || (c == '.' && !buf.contains('.')) || (c == '-' && min < 0.0 && buf.is_empty());
            if ok && buf.len() < 8 {
                buf.push(c);
            }
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        if let Some((_, buf)) = tools.editing_bar.as_mut() {
            buf.pop();
        }
    }
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
        commit = true;
        stop = true;
    }
    if is_key_pressed(KeyCode::Escape) {
        stop = true;
    }
    let value = if commit {
        tools
            .editing_bar
            .as_ref()
            .filter(|(eid, _)| eid == id)
            .and_then(|(_, buf)| buf.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(min, max))
    } else {
        None
    };
    if stop {
        tools.stop_editing_bar();
    }
    value
}

/// Small square `-` / `+` button used by steppers. Returns its hover state.
fn draw_step_button(fonts: &Fonts, scaler: &UiScaler, rect: (f32, f32, f32, f32), glyph: &str, mouse: Vec2) -> bool {
    let hover = point_in(mouse, rect);
    let (x, y, w, h) = rect;
    scaler.draw_glass_card(x, y, w, h, if hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, if hover { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER }, 1.0);
    fonts.draw_ui_bold_centered(glyph, x + w * 0.5, y + h * 0.5 + scaler.font_s(12.0) * 0.35, scaler.font_s(12.0), if hover { Palette::WHITE } else { Palette::UI_TEXT_MUTED });
    hover
}

/// `[-] bar [+]` stepper: relative drag on the bar, click without drag to type, focused wheel,
/// and hold-to-repeat on the buttons (Shift = 5 steps).
#[allow(clippy::too_many_arguments)]
fn draw_inspector_stepper(
    fonts: &Fonts,
    scaler: &UiScaler,
    tools: &mut ToolSettings,
    rect: (f32, f32, f32, f32),
    prop: Prop,
    value: Common<f32>,
    anchor: f32,
    (min, max, step): (f32, f32, f32),
    unit: &str,
    signed: bool,
    mouse: Vec2,
    raw_mouse: Vec2,
    clicked: bool,
) -> Option<Edit> {
    let (x, y, w, h) = rect;
    let id = prop.id();
    let btn_w = scaler.s(18.0);
    let gap = scaler.s(3.0);
    let minus = (x, y, btn_w, h);
    let bar = (x + btn_w + gap, y, (w - 2.0 * (btn_w + gap)).max(1.0), h);
    let plus = (bar.0 + bar.2 + gap, y, btn_w, h);
    let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
    let mult = if shift { 5.0 } else { 1.0 };
    let down = is_mouse_button_down(MouseButton::Left);
    let now = macroquad::time::get_time();
    let mut edit = None;

    let over_minus = draw_step_button(fonts, scaler, minus, "-", mouse);
    let over_plus = draw_step_button(fonts, scaler, plus, "+", mouse);
    if tools.inspector.hold.poll(&format!("{id}:dec"), clicked, down, over_minus, now) {
        edit = Some(Edit::Step(prop, -step * mult));
    }
    if tools.inspector.hold.poll(&format!("{id}:inc"), clicked, down, over_plus, now) {
        edit = Some(Edit::Step(prop, step * mult));
    }

    let over_bar = point_in(mouse, bar);
    let editing = tools.editing_bar.as_ref().is_some_and(|(eid, _)| *eid == id);
    if editing {
        if let Some(v) = stepper_text_entry(tools, &id, min, max, clicked && !over_bar) {
            edit = Some(Edit::Set(prop, v));
        }
    } else if clicked && over_bar {
        tools.select_bar(&id);
        let start = value.same().unwrap_or(anchor);
        tools.inspector.stepper_drag = Some(StepperDrag { prop, start_x: raw_mouse.x, start_value: start, moved: false, last_value: start });
    }

    // Drag: the press captured the pointer; the value follows it until release, even off the bar.
    if let Some(mut drag) = tools.inspector.stepper_drag.filter(|d| d.prop == prop) {
        if down {
            let dx = raw_mouse.x - drag.start_x;
            if dx.abs() > scaler.s(3.0) {
                drag.moved = true;
            }
            if drag.moved {
                let raw = drag.start_value + dx / bar.2 * (max - min);
                let v = ((raw / step).round() * step).clamp(min, max);
                if (v - drag.last_value).abs() > f32::EPSILON || value.same().is_none() {
                    edit = Some(Edit::Set(prop, v));
                    drag.last_value = v;
                }
            }
            tools.inspector.stepper_drag = Some(drag);
        } else {
            tools.inspector.stepper_drag = None;
            if !drag.moved {
                let text = value.same().map(|v| format_stepper_value(v, "", false, step)).unwrap_or_default();
                tools.start_editing_bar(&id, &text);
            }
        }
    }

    // Wheel: only while this stepper has focus and the pointer is over the inspector (W1).
    let focused = tools.is_bar_selected(&id);
    let wheel_y = std::panic::catch_unwind(macroquad::input::mouse_wheel).unwrap_or((0.0, 0.0)).1;
    if focused && tools.inspector.hovered && wheel_y.abs() > 0.01 && !tools.is_editing_text() {
        edit = Some(Edit::Step(prop, step * mult * wheel_y.signum()));
    }

    // Bar body.
    let (bx, by, bw, bh) = bar;
    let border = if focused { Palette::NEON_CYAN } else if over_bar { Palette::WHITE } else { Palette::UI_CARD_BORDER };
    scaler.draw_glass_card(bx, by, bw, bh, Palette::UI_PILL_BG, border, if focused { 1.6 } else { 1.0 });
    if let Some(v) = value.same() {
        let frac = ((v - min) / (max - min)).clamp(0.0, 1.0);
        draw_rectangle(bx + 1.0, by + 1.0, (bw - 2.0) * frac, bh - 2.0, Color::new(0.20, 0.90, 1.0, 0.38));
    }
    let size = scaler.font_s(11.5);
    let text_y = by + bh * 0.5 + size * 0.35;
    if let Some((_, buf)) = tools.editing_bar.as_ref().filter(|(eid, _)| *eid == id) {
        let cursor = if (now * 2.5).fract() < 0.5 { "|" } else { "" };
        fonts.draw_ui_bold_centered(&format!("{buf}{cursor}"), bx + bw * 0.5, text_y, size, Palette::WHITE);
    } else {
        let label = value.same().map(|v| format_stepper_value(v, unit, signed, step)).unwrap_or_else(|| "—".to_string());
        let dim = fonts.measure_ui_bold(&label, size);
        let col = if value.same().is_some() { Palette::WHITE } else { Palette::UI_TEXT_MUTED };
        fonts.draw_ui_bold(&label, bx + bw - dim.width - scaler.s(6.0), text_y, size, col);
    }
    edit
}

/// Footer action button: bold label plus a muted shortcut hint. Returns true when clicked.
#[allow(clippy::too_many_arguments)]
fn draw_footer_button(fonts: &Fonts, scaler: &UiScaler, rect: (f32, f32, f32, f32), label: &str, shortcut: &str, accent: Color, mouse: Vec2, clicked: bool) -> bool {
    let (x, y, w, h) = rect;
    let hover = point_in(mouse, rect);
    let bg = if hover { Color::new(accent.r * 0.18, accent.g * 0.18, accent.b * 0.18, 0.95) } else { Palette::UI_PILL_BG };
    scaler.draw_glass_card(x, y, w, h, bg, if hover { accent } else { Color::new(accent.r, accent.g, accent.b, 0.7) }, 1.2);
    let label_size = scaler.font_s(12.0);
    let hint_size = scaler.font_s(10.5);
    let gap = scaler.s(5.0);
    let label_w = fonts.measure_ui_bold(label, label_size).width;
    let hint_w = fonts.measure_ui_regular(shortcut, hint_size).width;
    let start = x + (w - label_w - gap - hint_w) * 0.5;
    let baseline = y + h * 0.5 + label_size * 0.35;
    fonts.draw_ui_bold(label, start, baseline, label_size, Palette::WHITE);
    fonts.draw_ui_regular(shortcut, start + label_w + gap, baseline, hint_size, Palette::UI_TEXT_MUTED);
    hover && clicked
}

/// Draws one inspector row at `y`. Returns the edit it produced this frame, if any.
#[allow(clippy::too_many_arguments)]
fn draw_inspector_row(
    fonts: &Fonts,
    scaler: &UiScaler,
    tools: &mut ToolSettings,
    row: &Row,
    x: f32,
    y: f32,
    w: f32,
    mouse: Vec2,
    raw_mouse: Vec2,
    clicked: bool,
) -> Option<Edit> {
    let h = scaler.s(ROW_H);
    let label_size = scaler.font_s(11.5);
    let baseline = y + h * 0.5 + label_size * 0.35;
    let ctrl_x = x + scaler.s(INSP_LABEL_W);
    let ctrl_w = w - scaler.s(INSP_LABEL_W);
    let draw_label = |text: &str| fonts.draw_ui_regular(text, x, baseline, label_size, Palette::UI_TEXT_MUTED);

    match row {
        Row::Info(text) => {
            let fitted = fonts.fit_ui_regular(text, scaler.font_s(11.0), w);
            fonts.draw_ui_regular(&fitted, x, y + scaler.s(12.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
            None
        }
        Row::Stepper { prop, label, value, anchor, min, max, step, unit, signed } => {
            draw_label(label);
            draw_inspector_stepper(fonts, scaler, tools, (ctrl_x, y, ctrl_w, h), *prop, *value, *anchor, (*min, *max, *step), unit, *signed, mouse, raw_mouse, clicked)
        }
        Row::Dropdown { prop, label, options, value } => {
            draw_label(label);
            let field = (ctrl_x, y, ctrl_w, h);
            let is_open = tools.inspector.dropdown_open() && tools.inspector.dropdown_prop == Some(*prop);
            if is_open {
                tools.inspector.dropdown_field = field;
            }
            let hovered = point_in(mouse, field);
            let current = value.same();
            let swatch = current.and_then(|i| options.surface(i)).map(surface_swatch);
            draw_field_dropdown(scaler, fonts, field.0, field.1, field.2, field.3, current.map(|i| options.label(i)).unwrap_or(""), swatch, current.is_none(), is_open, hovered, Palette::NEON_CYAN);
            if clicked && hovered && !is_open {
                tools.inspector.dropdown.open(current, options.len(), INSP_DROPDOWN_ROWS);
                tools.inspector.dropdown_prop = Some(*prop);
                tools.inspector.dropdown_field = field;
            }
            None
        }
        Row::Toggle { prop, label, value } => {
            draw_label(label);
            let side_w = scaler.s(INSP_SIDE_W) * 2.0;
            let rect = (x + w - side_w, y, side_w, h);
            let hover = point_in(mouse, rect);
            let on = matches!(value, Common::Same(true));
            let text = match value {
                Common::Same(true) => "on",
                Common::Same(false) => "off",
                _ => "—",
            };
            let bg = if on { Color::new(0.20, 0.90, 1.0, 0.14) } else if hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG };
            scaler.draw_glass_card(rect.0, rect.1, rect.2, rect.3, bg, if hover { Palette::WHITE } else if on { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER }, 1.0);
            fonts.draw_ui_bold_centered(text, rect.0 + side_w * 0.5, baseline, scaler.font_s(11.0), if on { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED });
            (hover && clicked).then(|| Edit::Flag(*prop, !value.same().unwrap_or(false)))
        }
        Row::Buttons(buttons) => {
            let gap = scaler.s(4.0);
            let bw = (w - gap * (buttons.len().saturating_sub(1)) as f32) / buttons.len().max(1) as f32;
            let mut edit = None;
            for (i, button) in buttons.iter().enumerate() {
                let rect = (x + i as f32 * (bw + gap), y, bw, h);
                let hover = point_in(mouse, rect);
                let accent = if button.highlight { Palette::NEON_GREEN } else if hover { Palette::WHITE } else { Palette::UI_CARD_BORDER };
                scaler.draw_glass_card(rect.0, rect.1, rect.2, rect.3, if hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG }, accent, 1.0);
                let fitted = fonts.fit_ui_bold(&button.label, scaler.font_s(11.0), bw - scaler.s(6.0));
                fonts.draw_ui_bold_centered(&fitted, rect.0 + bw * 0.5, baseline, scaler.font_s(11.0), Palette::WHITE);
                if hover && clicked {
                    edit = Some(button.edit);
                }
            }
            edit
        }
        Row::RampProfile(ramp) => {
            draw_ramp_lateral_view(fonts, scaler, x, y, w, scaler.s(RAMP_PROFILE_H), ramp);
            None
        }
        Row::Segmented { prop, label, labels, value } => {
            draw_label(label);
            let hovered = segment_at(ctrl_x, y, ctrl_w, h, labels.len(), (mouse.x, mouse.y));
            draw_segmented(scaler, fonts, ctrl_x, y, ctrl_w, h, labels, value.same(), hovered, Palette::NEON_CYAN);
            hovered.filter(|_| clicked).map(|i| Edit::Pick(*prop, i))
        }
        Row::Sides { label, left, right } => {
            draw_label(label);
            let side_w = scaler.s(INSP_SIDE_W);
            let mut edit = None;
            for (k, (prop, value)) in [left, right].into_iter().enumerate() {
                let rect = (x + w - side_w * (2 - k) as f32, y, side_w, h);
                let hover = point_in(mouse, rect);
                let (text, accent) = match value {
                    Common::Same(true) => ("on", Palette::NEON_CYAN),
                    Common::Same(false) => ("off", Palette::UI_CARD_BORDER),
                    _ => ("—", Palette::UI_CARD_BORDER),
                };
                let on = matches!(value, Common::Same(true));
                let bg = if on { Color::new(0.20, 0.90, 1.0, 0.14) } else if hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG };
                scaler.draw_glass_card(rect.0, rect.1, rect.2, rect.3, bg, if hover { Palette::WHITE } else { accent }, 1.0);
                fonts.draw_ui_bold_centered(text, rect.0 + side_w * 0.5, baseline, scaler.font_s(11.0), if on { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED });
                if hover && clicked {
                    edit = Some(Edit::Flag(*prop, !value.same().unwrap_or(false)));
                }
            }
            edit
        }
        Row::Chips { label, chips } => {
            draw_label(label);
            let gap = scaler.s(3.0);
            let chip_w = (ctrl_w - gap * (chips.len().saturating_sub(1)) as f32) / chips.len().max(1) as f32;
            let mut edit = None;
            for (i, chip) in chips.iter().enumerate() {
                let rect = (ctrl_x + i as f32 * (chip_w + gap), y, chip_w, h);
                let hover = point_in(mouse, rect);
                let accent = if chip.active { Palette::NEON_CYAN } else if hover { Palette::WHITE } else { Palette::UI_CARD_BORDER };
                let bg = if chip.active { Color::new(0.20, 0.90, 1.0, 0.14) } else if hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_PILL_BG };
                scaler.draw_glass_card(rect.0, rect.1, rect.2, rect.3, bg, accent, 1.0);
                fonts.draw_ui_bold_centered(&chip.label, rect.0 + chip_w * 0.5, baseline, scaler.font_s(11.0), if chip.active { Palette::NEON_CYAN } else { Palette::WHITE });
                if hover && clicked {
                    edit = Some(chip.edit);
                }
            }
            edit
        }
    }
}

/// Draws the model-based inspector into the card `(x, y, w, h)` and applies the edits it produced.
#[allow(clippy::too_many_arguments)]
fn render_inspector_model(
    fonts: &Fonts,
    scaler: &UiScaler,
    (x, y, w, h): (f32, f32, f32, f32),
    model: &InspectorModel,
    state: &mut EditorState,
    tools: &mut ToolSettings,
    mouse: Vec2,
    clicked: bool,
) {
    let pad = scaler.s(12.0);
    let inner_w = w - pad * 2.0;
    let wheel_y = std::panic::catch_unwind(macroquad::input::mouse_wheel).unwrap_or((0.0, 0.0)).1;
    let mut clicked = clicked;
    let mut edits: Vec<Edit> = Vec::new();

    fonts.draw_ui_bold("INSPECTOR", x + pad, y + scaler.s(22.0), scaler.font_s(14.0), Palette::NEON_GOLD);
    let header_y = y + scaler.s(TITLE_H);
    fonts.draw_ui_bold(&model.title, x + pad, header_y + scaler.s(14.0), scaler.font_s(13.0), Palette::WHITE);
    let right_text = if model.count > 1 { Some(format!("{} selected", model.count)) } else { model.subtitle.map(str::to_string) };
    if let Some(text) = right_text {
        let dim = fonts.measure_ui_bold(&text, scaler.font_s(11.0));
        fonts.draw_ui_bold(&text, x + w - pad - dim.width, header_y + scaler.s(14.0), scaler.font_s(11.0), Palette::NEON_CYAN);
    }
    draw_rectangle(x + pad, header_y + scaler.s(HEADER_H) - 1.0, inner_w, 1.0, Palette::UI_CARD_BORDER);

    let body_top = header_y + scaler.s(HEADER_H);
    let footer_h = if model.footer.is_some() { scaler.s(FOOTER_H) } else { 0.0 };
    let body_h = (y + h - footer_h - body_top).max(0.0);
    let body = (x, body_top, w, body_h);
    let popup_bounds = (x + scaler.s(4.0), body_top, w - scaler.s(8.0), y + h - scaler.s(4.0) - body_top);
    let item_h = scaler.s(ROW_H);

    // The open dropdown takes input first: its list covers the rows below it.
    let mut wheel_used = false;
    if tools.inspector.dropdown_open() {
        if let Some((prop, (options, selected))) = tools.inspector.dropdown_prop.and_then(|p| dropdown_row(model, p).map(|row| (p, row))) {
            let field = tools.inspector.dropdown_field;
            let layout = tools.inspector.dropdown.popup_layout(field, options.len(), INSP_DROPDOWN_ROWS, popup_bounds, item_h);
            wheel_used = layout.contains((mouse.x, mouse.y));
            let input = FieldDropdownInput {
                mouse: (mouse.x, mouse.y),
                clicked,
                wheel: wheel_y,
                up: is_key_pressed(KeyCode::Up),
                down: is_key_pressed(KeyCode::Down),
                confirm: is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter),
                cancel: false,
            };
            match tools.inspector.dropdown.handle_input(field, options.len(), selected.same(), INSP_DROPDOWN_ROWS, popup_bounds, item_h, input) {
                FieldDropdownEvent::None => {}
                FieldDropdownEvent::Picked(i) => {
                    edits.push(Edit::Pick(prop, i));
                    clicked = false;
                }
                FieldDropdownEvent::Opened | FieldDropdownEvent::Closed => clicked = false,
            }
            if !tools.inspector.dropdown_open() {
                tools.inspector.dropdown_prop = None;
            }
        } else {
            tools.inspector.close_dropdown();
        }
    }

    // Scroll: the wheel scrolls the body unless a focused stepper or the dropdown list takes it.
    let content_h = scaler.s(content_height(model, |s| tools.inspector.is_collapsed(s)));
    let max_scroll = ((content_h - body_h) / scaler.s(1.0)).max(0.0);
    let focus_in_model = tools.selected_bar.as_deref().is_some_and(|id| id.starts_with("insp:"));
    if wheel_y.abs() > 0.01 && !wheel_used && !focus_in_model && point_in(mouse, body) {
        tools.inspector.scroll -= wheel_y.signum() * 40.0;
    }
    tools.inspector.scroll = tools.inspector.scroll.clamp(0.0, max_scroll);

    // Controls outside the visible body get no pointer.
    let in_body = point_in(mouse, body) && !tools.inspector.dropdown_open();
    let body_mouse = if in_body { mouse } else { Vec2::new(-1.0e6, -1.0e6) };
    let body_clicked = clicked && in_body;

    cabinet::ui::scaler::begin_clip_rect(x, body_top, w, body_h);
    let mut cy = body_top + scaler.s(BODY_PAD) - scaler.s(tools.inspector.scroll);
    for section in &model.sections {
        cy += scaler.s(SECTION_GAP);
        let header = (x + pad, cy, inner_w, scaler.s(SECTION_HEADER_H));
        let collapsed = tools.inspector.is_collapsed(section);
        let hover = point_in(body_mouse, header);
        let tri_x = x + pad + scaler.s(3.0);
        let tri_y = cy + scaler.s(SECTION_HEADER_H) * 0.5;
        let k = scaler.s(3.0);
        let tri_col = if hover { Palette::WHITE } else { Palette::UI_TEXT_MUTED };
        if collapsed {
            macroquad::shapes::draw_triangle(macroquad::math::vec2(tri_x - k * 0.6, tri_y - k), macroquad::math::vec2(tri_x - k * 0.6, tri_y + k), macroquad::math::vec2(tri_x + k * 0.8, tri_y), tri_col);
        } else {
            macroquad::shapes::draw_triangle(macroquad::math::vec2(tri_x - k, tri_y - k * 0.6), macroquad::math::vec2(tri_x + k, tri_y - k * 0.6), macroquad::math::vec2(tri_x, tri_y + k * 0.8), tri_col);
        }
        let title_size = scaler.font_s(10.5);
        fonts.draw_ui_bold(section.title, x + pad + scaler.s(12.0), tri_y + title_size * 0.35, title_size, if hover { Palette::WHITE } else { Palette::NEON_CYAN });
        if section.side_columns && !collapsed {
            let side_w = scaler.s(INSP_SIDE_W);
            for (k, t) in ["L", "R"].into_iter().enumerate() {
                fonts.draw_ui_bold_centered(t, x + pad + inner_w - side_w * (2 - k) as f32 + side_w * 0.5, tri_y + title_size * 0.35, title_size, Palette::UI_TEXT_MUTED);
            }
        }
        if hover && body_clicked {
            tools.inspector.toggle_section(section.id);
        }
        cy += scaler.s(SECTION_HEADER_H);
        if collapsed {
            continue;
        }
        for row in &section.rows {
            if let Some(edit) = draw_inspector_row(fonts, scaler, tools, row, x + pad, cy, inner_w, body_mouse, mouse, body_clicked) {
                edits.push(edit);
            }
            cy += scaler.s(row_height(row));
        }
    }
    cabinet::ui::scaler::end_clip_rect();

    if max_scroll > 0.0 {
        let track_h = body_h - scaler.s(8.0);
        let thumb_h = (track_h * body_h / content_h).max(scaler.s(16.0));
        let thumb_y = body_top + scaler.s(4.0) + (track_h - thumb_h) * (tools.inspector.scroll / max_scroll);
        draw_rectangle(x + w - scaler.s(5.0), thumb_y, scaler.s(3.0), thumb_h, Palette::UI_CARD_BORDER);
    }

    if let Some(footer) = model.footer {
        let fy = y + h - footer_h;
        draw_rectangle(x + pad, fy, inner_w, 1.0, Palette::UI_CARD_BORDER);
        let by = fy + scaler.s(7.0);
        let footer_clicked = clicked && !tools.inspector.dropdown_open();
        let mut delete_x = x + pad;
        let mut delete_w = inner_w;
        if footer.duplicate {
            let bw = (inner_w - scaler.s(6.0)) * 0.5;
            if draw_footer_button(fonts, scaler, (x + pad, by, bw, scaler.s(24.0)), "Duplicate", "Ctrl+D", Palette::NEON_CYAN, mouse, footer_clicked) {
                edits.push(Edit::Do(InspectorAction::Duplicate));
            }
            delete_x += bw + scaler.s(6.0);
            delete_w = bw;
        }
        if draw_footer_button(fonts, scaler, (delete_x, by, delete_w, scaler.s(24.0)), footer.delete_label, "Del", Palette::RED, mouse, footer_clicked) {
            edits.push(Edit::Do(InspectorAction::Delete));
        }
    }

    if let Some((options, selected)) = tools.inspector.dropdown_prop.filter(|_| tools.inspector.dropdown_open()).and_then(|p| dropdown_row(model, p)) {
        let layout = tools.inspector.dropdown.popup_layout(tools.inspector.dropdown_field, options.len(), INSP_DROPDOWN_ROWS, popup_bounds, item_h);
        let names: Vec<String> = (0..options.len()).map(|i| options.label(i).to_string()).collect();
        let swatches: Vec<Color> = (0..options.len()).filter_map(|i| options.surface(i)).map(surface_swatch).collect();
        draw_field_dropdown_popup(scaler, fonts, &layout, &names, &swatches, selected.same(), tools.inspector.dropdown.hovered, Palette::NEON_CYAN);
    }

    for edit in edits {
        apply_edit(state, tools, edit);
    }
}
