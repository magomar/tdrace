pub mod camera;
pub mod state;
pub mod tools;
pub mod ui;

pub use camera::EditorCamera;
pub use state::{EditorState, GridSnapSetting, HistoryStack, Selection};
pub use tools::{render_editor_gizmos, EditorToolType, ObstacleShapeType, SurfaceShapeType, ToolSettings};
pub use ui::{render_editor_ui, EditorAction, EditorModal};

use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::draw_line;

/// Renders the metric CAD-style editor background grid in world coordinates.
pub fn render_editor_grid(camera: &EditorCamera, sw: f32, sh: f32, snap: GridSnapSetting) {
    let top_left = camera.screen_to_world(Vec2::ZERO, sw, sh);
    let bottom_right = camera.screen_to_world(Vec2::new(sw, sh), sw, sh);

    let min_x = top_left.x.min(bottom_right.x);
    let max_x = top_left.x.max(bottom_right.x);
    let min_y = top_left.y.min(bottom_right.y);
    let max_y = top_left.y.max(bottom_right.y);

    // Adaptive grid step depending on camera zoom level
    let (minor_step, major_step) = if camera.zoom > 25.0 {
        (snap.step().unwrap_or(1.0).max(1.0), 5.0)
    } else if camera.zoom > 8.0 {
        (5.0, 25.0)
    } else if camera.zoom > 2.5 {
        (10.0, 50.0)
    } else {
        (50.0, 200.0)
    };

    let col_minor = Color::new(1.0, 1.0, 1.0, 0.04);
    let col_major = Color::new(1.0, 1.0, 1.0, 0.10);
    let col_axis_x = Color::new(0.95, 0.30, 0.30, 0.45); // Red X axis
    let col_axis_y = Color::new(0.30, 0.90, 0.35, 0.45); // Green Y axis

    let start_x = (min_x / minor_step).floor() * minor_step;
    let start_y = (min_y / minor_step).floor() * minor_step;

    // Draw vertical grid lines
    let mut x = start_x;
    while x <= max_x {
        let is_major = (x.abs() % major_step).abs() < 1e-3;
        let is_axis = x.abs() < 1e-3;

        let col = if is_axis {
            col_axis_y
        } else if is_major {
            col_major
        } else {
            col_minor
        };

        let thickness = if is_axis { 0.35 } else if is_major { 0.20 } else { 0.10 };
        draw_line(x, min_y, x, max_y, thickness, col);
        x += minor_step;
    }

    // Draw horizontal grid lines
    let mut y = start_y;
    while y <= max_y {
        let is_major = (y.abs() % major_step).abs() < 1e-3;
        let is_axis = y.abs() < 1e-3;

        let col = if is_axis {
            col_axis_x
        } else if is_major {
            col_major
        } else {
            col_minor
        };

        let thickness = if is_axis { 0.35 } else if is_major { 0.20 } else { 0.10 };
        draw_line(min_x, y, max_x, y, thickness, col);
        y += minor_step;
    }
}
