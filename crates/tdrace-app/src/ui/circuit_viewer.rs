use glam::Vec2;
use macroquad::color::Color;
use macroquad::input::{
    is_key_down, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed, mouse_position,
    mouse_wheel, KeyCode, MouseButton,
};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use macroquad::window::{clear_background, screen_height, screen_width};
use tdrace_core::track::Track;

use super::font::Fonts;
use super::scaler::UiScaler;
use super::track_preview::{compute_track_bounds, surface_preview_color};
use crate::render::color::Palette;

/// Origin screen that launched the full-circuit topdown viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CircuitViewerOrigin {
    #[default]
    Menu,
    TrackManager,
    StartingGrid,
}

/// Runtime state for inspecting a circuit top-down with all in-game graphics.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitViewerState {
    pub origin: CircuitViewerOrigin,
    pub track: Track,
    pub title: String,
    pub module_id: String,
    pub camera_center: Vec2,
    pub camera_zoom: f32,
    pub base_zoom: f32,
    pub base_center: Vec2,
    pub is_panning: bool,
    pub pan_start_screen: Vec2,
    pub pan_start_center: Vec2,
}

impl CircuitViewerState {
    /// Constructs a new viewer state centered on the track with maximum zoom out fitting the screen.
    pub fn new(
        track: Track,
        title: String,
        module_id: String,
        origin: CircuitViewerOrigin,
        sw: f32,
        sh: f32,
    ) -> Self {
        let (min, max) = compute_full_track_bounds(&track);
        let center = (min + max) * 0.5;
        let extent = max - min;
        let width = extent.x.max(100.0);
        let height = extent.y.max(100.0);

        let scaler = UiScaler::new(sw, sh);
        let pad_h = scaler.s(80.0);
        let pad_v = scaler.s(160.0);

        let avail_w = (sw - pad_h).max(100.0);
        let avail_h = (sh - pad_v).max(100.0);

        // Maximum zoom out that fits the full circuit comfortably with margins
        let fit_zoom = (avail_w / width).min(avail_h / height);

        Self {
            origin,
            track,
            title,
            module_id,
            camera_center: center,
            camera_zoom: fit_zoom,
            base_zoom: fit_zoom,
            base_center: center,
            is_panning: false,
            pan_start_screen: Vec2::ZERO,
            pan_start_center: center,
        }
    }

    /// Resets camera center and zoom to the default max-zoom-out overview.
    pub fn reset_to_fit(&mut self) {
        self.camera_center = self.base_center;
        self.camera_zoom = self.base_zoom;
        self.is_panning = false;
    }
}

/// Computes a comprehensive bounding box including spline, obstacles, grandstands, and surface zones.
pub fn compute_full_track_bounds(track: &Track) -> (Vec2, Vec2) {
    let (mut min, mut max) = compute_track_bounds(track);

    for obs in &track.geometry.obstacles {
        match &obs.shape {
            tdrace_core::track::geometry::ObstacleShape::Circle { center, radius } => {
                min = min.min(*center - Vec2::splat(*radius));
                max = max.max(*center + Vec2::splat(*radius));
            }
            tdrace_core::track::geometry::ObstacleShape::Box { center, half_extents, .. } => {
                let diag = half_extents.length();
                min = min.min(*center - Vec2::splat(diag));
                max = max.max(*center + Vec2::splat(diag));
            }
            tdrace_core::track::geometry::ObstacleShape::Polygon { vertices } => {
                for v in vertices {
                    min = min.min(*v);
                    max = max.max(*v);
                }
            }
        }
    }

    for gs in &track.geometry.grandstands {
        for corner in gs.corners() {
            min = min.min(corner);
            max = max.max(corner);
        }
    }

    for tree in &track.geometry.trees {
        let r = tree.canopy_radius();
        min = min.min(tree.position - Vec2::splat(r));
        max = max.max(tree.position + Vec2::splat(r));
    }

    (min, max)
}

/// Renders the complete circuit viewer screen (world top-down pass followed by HUD overlay).
pub fn render_circuit_viewer_screen(fonts: &Fonts, state: &CircuitViewerState) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // 1. Clear background terrain using the track's authentic default surface color
    let backdrop_col = crate::render::get_track_backdrop_color(state.track.default_surface);
    clear_background(backdrop_col);

    // 2. Set 2D topdown camera focused on camera_center with camera_zoom
    let zoom_x = (2.0 * state.camera_zoom) / sw;
    let zoom_y = (-2.0 * state.camera_zoom) / sh; // +Y is up in world space

    let camera = macroquad::camera::Camera2D {
        target: macroquad::prelude::Vec2::new(state.camera_center.x, state.camera_center.y),
        zoom: macroquad::prelude::Vec2::new(zoom_x, zoom_y),
        offset: macroquad::prelude::Vec2::ZERO,
        rotation: 0.0,
        render_target: None,
        viewport: None,
    };
    macroquad::camera::set_camera(&camera);

    // 3. Render all in-game graphics layers without viewport culling
    // Ground track & surfaces (textures, curbs, runoff, hazard zones, timing lines)
    crate::render::render_ground_track_culled(&state.track, None);

    // Scenery: Grandstand shadows, tree shadows, grandstands, barriers, tree trunks
    crate::render::render_grandstand_shadows_culled(&state.track, None);
    crate::render::render_tree_shadows_culled(&state.track, None);
    crate::render::render_grandstands_culled(&state.track, None);
    crate::render::render_ground_barriers_and_obstacles_culled(&state.track, None);
    crate::render::render_tree_trunks_culled(&state.track, None);

    // Elevated overpass bridges & elevated barriers
    crate::render::render_elevated_track_culled(&state.track, None);
    crate::render::render_elevated_barriers_and_obstacles_culled(&state.track, None);

    // Tree foliage canopies
    crate::render::render_tree_canopies_culled(&state.track, &[], None);

    // 4. Reset camera back to screen-space coordinates for HUD
    macroquad::camera::set_default_camera();

    // 5. Draw top-down view HUD overlay
    render_viewer_hud(fonts, &scaler, state, sw, sh);
}

/// Renders the HUD overlay (title, telemetry stats, zoom pill, and controls bar).
fn render_viewer_hud(
    fonts: &Fonts,
    scaler: &UiScaler,
    state: &CircuitViewerState,
    sw: f32,
    sh: f32,
) {
    let accent = match state.module_id.as_str() {
        "gt" | "gt_challenge" => Palette::RED,
        "nascar" => Palette::YELLOW,
        "rally" => Palette::NEON_GOLD,
        "kart" => Palette::NEON_CYAN,
        "extreme_offroad" => Palette::DUNE_ORANGE,
        _ => Palette::NEON_CYAN,
    };

    // --- TOP HEADER BAR ---
    let header_h = scaler.s(64.0);
    let header_y = scaler.safe_pad_y;
    let header_w = sw - scaler.safe_pad_x * 2.0;
    let header_x = scaler.safe_pad_x;

    scaler.draw_glass_card(
        header_x,
        header_y,
        header_w,
        header_h,
        Color::new(0.04, 0.06, 0.10, 0.92),
        accent,
        1.5,
    );

    // Back button
    let back_btn_w = scaler.s(90.0);
    let back_btn_h = scaler.s(36.0);
    let back_btn_x = header_x + scaler.s(14.0);
    let back_btn_y = header_y + (header_h - back_btn_h) * 0.5;

    let mouse_pos = mouse_position();
    let mouse_vec = Vec2::new(mouse_pos.0, mouse_pos.1);
    let back_hover = mouse_vec.x >= back_btn_x
        && mouse_vec.x <= back_btn_x + back_btn_w
        && mouse_vec.y >= back_btn_y
        && mouse_vec.y <= back_btn_y + back_btn_h;

    scaler.draw_glass_card(
        back_btn_x,
        back_btn_y,
        back_btn_w,
        back_btn_h,
        if back_hover {
            Color::new(0.20, 0.24, 0.35, 0.95)
        } else {
            Color::new(0.10, 0.14, 0.22, 0.85)
        },
        if back_hover { Palette::WHITE } else { Palette::UI_CARD_BORDER },
        if back_hover { 1.8 } else { 1.0 },
    );
    fonts.draw_ui_bold_centered(
        "◄ BACK",
        back_btn_x + back_btn_w * 0.5,
        back_btn_y + scaler.s(22.0),
        scaler.font_s(12.0),
        Palette::WHITE,
    );

    // Circuit Title & Category
    let title_x = back_btn_x + back_btn_w + scaler.s(18.0);
    let display_title = if !state.track.name.is_empty() {
        &state.track.name
    } else {
        &state.title
    };
    fonts.draw_display(
        display_title,
        title_x,
        header_y + scaler.s(26.0),
        scaler.font_s(18.0),
        Palette::WHITE,
    );

    let module_badge = match state.module_id.as_str() {
        "gt" | "gt_challenge" => "GT WORLD CHALLENGE",
        "nascar" => "NASCAR CUP SERIES",
        "rally" => "RALLYCROSS WORLD CUP",
        "kart" => "PRO KART SERIES",
        "extreme_offroad" => "EXTREME OFF-ROAD",
        _ => "MOTORSPORT CIRCUIT",
    };
    let len_km = state.track.total_length_m() / 1000.0;
    let waypoints_cnt = state.track.spline.waypoints.len();
    let meta_str = format!(
        "{} • {:.2} km ({:.0} m) • {} Waypoints • Top-Down Full Inspection",
        module_badge,
        len_km,
        state.track.total_length_m(),
        waypoints_cnt
    );
    fonts.draw_ui_regular(
        &meta_str,
        title_x,
        header_y + scaler.s(48.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    // Zoom Controls & Pill (Top-Right)
    let zoom_pill_w = scaler.s(240.0);
    let zoom_pill_x = header_x + header_w - zoom_pill_w - scaler.s(14.0);
    let zoom_pill_y = header_y + (header_h - back_btn_h) * 0.5;

    let zoom_pct = (state.camera_zoom / state.base_zoom) * 100.0;
    let is_fit = (zoom_pct - 100.0).abs() < 2.0;

    // Zoom Out Button [-]
    let btn_step_w = scaler.s(32.0);
    let zoom_out_rect = (zoom_pill_x, zoom_pill_y, btn_step_w, back_btn_h);
    let zoom_out_hover = mouse_vec.x >= zoom_out_rect.0
        && mouse_vec.x <= zoom_out_rect.0 + zoom_out_rect.2
        && mouse_vec.y >= zoom_out_rect.1
        && mouse_vec.y <= zoom_out_rect.1 + zoom_out_rect.3;

    scaler.draw_glass_card(
        zoom_out_rect.0,
        zoom_out_rect.1,
        zoom_out_rect.2,
        zoom_out_rect.3,
        if zoom_out_hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
        Palette::UI_CARD_BORDER,
        1.0,
    );
    fonts.draw_ui_bold_centered(
        "-",
        zoom_out_rect.0 + btn_step_w * 0.5,
        zoom_out_rect.1 + scaler.s(22.0),
        scaler.font_s(14.0),
        Palette::WHITE,
    );

    // Zoom Pill Label
    let pill_mid_x = zoom_pill_x + btn_step_w + scaler.s(4.0);
    let pill_mid_w = zoom_pill_w - btn_step_w * 2.0 - scaler.s(60.0);
    scaler.draw_glass_card(
        pill_mid_x,
        zoom_pill_y,
        pill_mid_w,
        back_btn_h,
        Color::new(0.06, 0.10, 0.18, 0.90),
        if is_fit { Palette::NEON_GREEN } else { Palette::NEON_CYAN },
        1.2,
    );
    let zoom_str = if is_fit {
        "ZOOM: FIT (100%)".to_string()
    } else {
        format!("ZOOM: {:.0}%", zoom_pct)
    };
    fonts.draw_ui_bold_centered(
        &zoom_str,
        pill_mid_x + pill_mid_w * 0.5,
        zoom_pill_y + scaler.s(22.0),
        scaler.font_s(11.0),
        if is_fit { Palette::NEON_GREEN } else { Palette::WHITE },
    );

    // Zoom In Button [+]
    let zoom_in_x = pill_mid_x + pill_mid_w + scaler.s(4.0);
    let zoom_in_rect = (zoom_in_x, zoom_pill_y, btn_step_w, back_btn_h);
    let zoom_in_hover = mouse_vec.x >= zoom_in_rect.0
        && mouse_vec.x <= zoom_in_rect.0 + zoom_in_rect.2
        && mouse_vec.y >= zoom_in_rect.1
        && mouse_vec.y <= zoom_in_rect.1 + zoom_in_rect.3;

    scaler.draw_glass_card(
        zoom_in_rect.0,
        zoom_in_rect.1,
        zoom_in_rect.2,
        zoom_in_rect.3,
        if zoom_in_hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
        Palette::UI_CARD_BORDER,
        1.0,
    );
    fonts.draw_ui_bold_centered(
        "+",
        zoom_in_rect.0 + btn_step_w * 0.5,
        zoom_in_rect.1 + scaler.s(22.0),
        scaler.font_s(14.0),
        Palette::WHITE,
    );

    // Reset Fit Button [FIT]
    let fit_btn_x = zoom_in_x + btn_step_w + scaler.s(4.0);
    let fit_btn_w = scaler.s(48.0);
    let fit_rect = (fit_btn_x, zoom_pill_y, fit_btn_w, back_btn_h);
    let fit_hover = mouse_vec.x >= fit_rect.0
        && mouse_vec.x <= fit_rect.0 + fit_rect.2
        && mouse_vec.y >= fit_rect.1
        && mouse_vec.y <= fit_rect.1 + fit_rect.3;

    scaler.draw_glass_card(
        fit_rect.0,
        fit_rect.1,
        fit_rect.2,
        fit_rect.3,
        if fit_hover { Palette::UI_CARD_BG_HOVER } else { Palette::UI_CARD_BG },
        if is_fit { Palette::NEON_GREEN } else { Palette::UI_CARD_BORDER },
        1.0,
    );
    fonts.draw_ui_bold_centered(
        "FIT",
        fit_rect.0 + fit_btn_w * 0.5,
        fit_rect.1 + scaler.s(22.0),
        scaler.font_s(11.0),
        if is_fit { Palette::NEON_GREEN } else { Palette::WHITE },
    );

    // --- BOTTOM TELEMETRY & CONTROLS FOOTER ---
    let footer_h = scaler.s(48.0);
    let footer_y = sh - footer_h - scaler.safe_pad_y;
    let footer_w = sw - scaler.safe_pad_x * 2.0;
    let footer_x = scaler.safe_pad_x;

    scaler.draw_glass_card(
        footer_x,
        footer_y,
        footer_w,
        footer_h,
        Color::new(0.04, 0.06, 0.10, 0.92),
        Palette::UI_CARD_BORDER,
        1.2,
    );

    // Surface Breakdown bar on left side of footer
    let breakdown = state.track.surface_breakdown();
    let bar_x = footer_x + scaler.s(16.0);
    let bar_w = scaler.s(280.0);
    let bar_h = scaler.s(6.0);
    let bar_y = footer_y + scaler.s(24.0);

    let mut curr_bx = bar_x;
    for (surf, pct) in &breakdown {
        let seg_w = bar_w * (pct / 100.0);
        let col = surface_preview_color(*surf);
        draw_rectangle(curr_bx, bar_y, seg_w, bar_h, col);
        curr_bx += seg_w;
    }
    draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 0.8, Palette::UI_CARD_BORDER);

    let surf_label = format!("SURFACES: {}", state.track.surface_summary_string());
    fonts.draw_ui_bold(
        &surf_label,
        bar_x,
        footer_y + scaler.s(18.0),
        scaler.font_s(9.5),
        Palette::WHITE,
    );

    // Interactive Controls Hints on right side of footer
    let controls_hint = "[ESC / B] Return • [Drag / WASD] Pan • [Scroll Wheel / Triggers] Zoom • [R / Space] Reset Fit";
    fonts.draw_ui_bold(
        controls_hint,
        footer_x + footer_w - scaler.s(550.0),
        footer_y + scaler.s(28.0),
        scaler.font_s(11.0),
        Palette::NEON_CYAN,
    );
}

/// Updates input handling for the Circuit Viewer (pan, zoom, reset, exit).
pub fn handle_circuit_viewer_input(
    state: &mut CircuitViewerState,
    dt: f32,
    gamepad_snapshot: &crate::input::GamepadSnapshot,
) -> bool {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // 1. Back / Exit navigation
    let back_btn_w = scaler.s(90.0);
    let back_btn_h = scaler.s(36.0);
    let back_btn_x = scaler.safe_pad_x + scaler.s(14.0);
    let back_btn_y = scaler.safe_pad_y + (scaler.s(64.0) - back_btn_h) * 0.5;

    let mouse_pos = mouse_position();
    let mouse_vec = Vec2::new(mouse_pos.0, mouse_pos.1);
    let clicked_back = is_mouse_button_pressed(MouseButton::Left)
        && mouse_vec.x >= back_btn_x
        && mouse_vec.x <= back_btn_x + back_btn_w
        && mouse_vec.y >= back_btn_y
        && mouse_vec.y <= back_btn_y + back_btn_h;

    let exit_requested = is_key_pressed(KeyCode::Escape)
        || is_key_pressed(KeyCode::Backspace)
        || is_key_pressed(KeyCode::B)
        || gamepad_snapshot.btn_b_pressed
        || clicked_back;

    if exit_requested {
        return true; // Signal caller to pop back to previous state
    }

    // 2. Reset to Fit
    let zoom_pill_w = scaler.s(240.0);
    let zoom_pill_x = sw - scaler.safe_pad_x - zoom_pill_w - scaler.s(14.0);
    let zoom_pill_y = scaler.safe_pad_y + (scaler.s(64.0) - back_btn_h) * 0.5;
    let fit_btn_x = zoom_pill_x + scaler.s(32.0) + (zoom_pill_w - scaler.s(64.0) - scaler.s(60.0)) + scaler.s(8.0) + scaler.s(32.0);
    let fit_btn_w = scaler.s(48.0);

    let clicked_fit = is_mouse_button_pressed(MouseButton::Left)
        && mouse_vec.x >= fit_btn_x
        && mouse_vec.x <= fit_btn_x + fit_btn_w
        && mouse_vec.y >= zoom_pill_y
        && mouse_vec.y <= zoom_pill_y + back_btn_h;

    if is_key_pressed(KeyCode::R)
        || is_key_pressed(KeyCode::Space)
        || gamepad_snapshot.btn_y_pressed
        || clicked_fit
    {
        state.reset_to_fit();
    }

    // 3. Zoom handling
    let min_zoom = state.base_zoom * 0.5;
    let max_zoom = state.base_zoom * 20.0;

    let wheel = mouse_wheel().1;
    let mut zoom_factor = 1.0;

    if wheel > 0.0 {
        zoom_factor *= 1.15;
    } else if wheel < 0.0 {
        zoom_factor /= 1.15;
    }

    // Keyboard zoom shortcuts
    if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::PageUp) || is_key_pressed(KeyCode::E) {
        zoom_factor *= 1.25;
    }
    if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::PageDown) || is_key_pressed(KeyCode::Q) {
        zoom_factor /= 1.25;
    }

    // Interactive button clicks [-] and [+]
    let btn_step_w = scaler.s(32.0);
    let clicked_zoom_out = is_mouse_button_pressed(MouseButton::Left)
        && mouse_vec.x >= zoom_pill_x
        && mouse_vec.x <= zoom_pill_x + btn_step_w
        && mouse_vec.y >= zoom_pill_y
        && mouse_vec.y <= zoom_pill_y + back_btn_h;

    let zoom_in_x = zoom_pill_x + btn_step_w + (zoom_pill_w - btn_step_w * 2.0 - scaler.s(60.0)) + scaler.s(8.0);
    let clicked_zoom_in = is_mouse_button_pressed(MouseButton::Left)
        && mouse_vec.x >= zoom_in_x
        && mouse_vec.x <= zoom_in_x + btn_step_w
        && mouse_vec.y >= zoom_pill_y
        && mouse_vec.y <= zoom_pill_y + back_btn_h;

    if clicked_zoom_in {
        zoom_factor *= 1.25;
    }
    if clicked_zoom_out {
        zoom_factor /= 1.25;
    }

    // Gamepad triggers zoom
    if gamepad_snapshot.throttle > 0.2 || gamepad_snapshot.btn_rb_down {
        zoom_factor *= 1.0 + 1.2 * dt;
    }
    if gamepad_snapshot.brake > 0.2 || gamepad_snapshot.btn_lb_down {
        zoom_factor /= 1.0 + 1.2 * dt;
    }

    if (zoom_factor - 1.0).abs() > 0.001 {
        let new_zoom = (state.camera_zoom * zoom_factor).clamp(min_zoom, max_zoom);
        state.camera_zoom = new_zoom;
    }

    // 4. Pan handling
    // Mouse drag
    let in_ui_header = mouse_vec.y < scaler.safe_pad_y + scaler.s(70.0);
    let in_ui_footer = mouse_vec.y > sh - scaler.safe_pad_y - scaler.s(60.0);

    if is_mouse_button_pressed(MouseButton::Left) && !in_ui_header && !in_ui_footer {
        state.is_panning = true;
        state.pan_start_screen = mouse_vec;
        state.pan_start_center = state.camera_center;
    }

    if is_mouse_button_down(MouseButton::Left) && state.is_panning {
        let delta_screen = mouse_vec - state.pan_start_screen;
        let delta_world = Vec2::new(-delta_screen.x / state.camera_zoom, delta_screen.y / state.camera_zoom);
        state.camera_center = state.pan_start_center + delta_world;
    } else {
        state.is_panning = false;
    }

    // Keyboard WASD / Arrow keys
    let pan_speed_screen = scaler.s(450.0); // pixels per sec
    let mut pan_dir = Vec2::ZERO;

    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        pan_dir.y += 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        pan_dir.y -= 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        pan_dir.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        pan_dir.x += 1.0;
    }

    // Gamepad Left Stick
    if gamepad_snapshot.steer.abs() > 0.15 {
        pan_dir.x += gamepad_snapshot.steer;
    }
    if gamepad_snapshot.stick_y.abs() > 0.15 {
        pan_dir.y -= gamepad_snapshot.stick_y;
    }

    if pan_dir.length_squared() > 0.001 {
        let world_step = pan_dir.normalize() * (pan_speed_screen * dt / state.camera_zoom);
        state.camera_center += world_step;
    }

    false
}
