use tdrace_core::track::validation::{validate_track, TrackValidationError, ValidationSeverity};
use tdrace_core::track::Track;

use crate::module::classic::ClassicGameModule;
use crate::module::f1::F1GameModule;
use crate::module::kart::KartGameModule;
use crate::module::rally::RallyGameModule;
use crate::module::GameModule;

/// Serializes a track to formatted JSON string suitable for preset files or sharing.
pub fn export_track_to_json(track: &Track) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(track)
}

/// Generates standalone Rust source code that recreates the given track definition
/// as a procedural function suitable for inclusion in `tdrace_core::track::presets` or a module crate.
pub fn export_track_to_rust_code(track: &Track, fn_name: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("/// Preset circuit: {}\n", track.name));
    if !track.description.is_empty() {
        out.push_str(&format!("/// {}\n", track.description));
    }
    out.push_str(&format!("pub fn {}() -> Track {{\n", fn_name));
    out.push_str("    let waypoints = vec![\n");
    for wp in &track.spline.waypoints {
        out.push_str(&format!(
            "        TrackWaypoint::new(Vec2::new({:.1}, {:.1}), {:.1})",
            wp.point.x, wp.point.y, wp.width
        ));
        if wp.left_curb || wp.right_curb {
            out.push_str(&format!(".with_curbs({}, {})", wp.left_curb, wp.right_curb));
        }
        if let Some(surf) = wp.surface {
            out.push_str(&format!(".with_surface(SurfaceType::{:?})", surf));
        }
        if let Some(wall) = wp.wall_type {
            out.push_str(&format!(".with_wall_type(BarrierType::{:?})", wall));
        }
        out.push_str(",\n");
    }
    out.push_str("    ];\n\n");

    out.push_str("    let spline = TrackSpline::new(waypoints, true);\n");
    out.push_str("    let (left_walls, right_walls, left_poly, right_poly) =\n");
    out.push_str("        generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);\n\n");

    out.push_str(&format!(
        "    let checkpoints = generate_checkpoints(&spline, {}, 3);\n",
        track.checkpoints.len().max(8)
    ));
    out.push_str(&format!(
        "    let grid_positions = generate_grid_positions(&spline, {}, 8.0, 2.5);\n\n",
        track.grid_positions.len().max(8)
    ));

    out.push_str("    Track {\n");
    out.push_str(&format!("        name: \"{}\".to_string(),\n", track.name));
    out.push_str(&format!("        description: \"{}\".to_string(),\n", track.description));
    out.push_str("        category: TrackCategory::Main,\n");
    out.push_str("        spline,\n");
    out.push_str("        geometry: TrackGeometry {\n");
    out.push_str("            inner_walls: left_walls,\n");
    out.push_str("            outer_walls: right_walls,\n");
    out.push_str("            obstacles: Vec::new(),\n");
    out.push_str("            surface_zones: Vec::new(),\n");
    out.push_str("            jump_ramps: Vec::new(),\n");
    out.push_str("            left_boundary_polyline: left_poly,\n");
    out.push_str("            right_boundary_polyline: right_poly,\n");
    out.push_str("        },\n");
    out.push_str("        checkpoints,\n");
    out.push_str("        grid_positions,\n");
    out.push_str(&format!("        default_surface: SurfaceType::{:?},\n", track.default_surface));
    out.push_str("        pit_box_area: None,\n");
    out.push_str(&format!("        default_laps: {},\n", track.default_laps));
    if let Some(ref car) = track.predefined_car {
        out.push_str(&format!("        predefined_car: Some(\"{}\".to_string()),\n", car));
    } else {
        out.push_str("        predefined_car: None,\n");
    }
    if let Some(ref m) = track.module_id {
        out.push_str(&format!("        module_id: Some(\"{}\".to_string()),\n", m));
    } else {
        out.push_str("        module_id: None,\n");
    }
    out.push_str(&format!(
        "        modules: vec![{}],\n",
        track
            .modules
            .iter()
            .map(|m| format!("\"{}\".to_string()", m))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str("    }\n");
    out.push_str("}\n");

    out
}

/// Validates all official presets across all registered modules, returning any errors or warnings.
pub fn validate_all_official_presets() -> Vec<(String, Vec<TrackValidationError>)> {
    let mut results = Vec::new();
    let modules: Vec<Box<dyn GameModule>> = vec![
        Box::new(ClassicGameModule::new()),
        Box::new(F1GameModule::new()),
        Box::new(RallyGameModule::new()),
        Box::new(KartGameModule::new()),
    ];

    for module in &modules {
        for track_def in module.tracks() {
            let track = (track_def.generator)();
            let diags = validate_track(&track);
            let errors_or_warnings: Vec<_> = diags
                .into_iter()
                .filter(|d| d.severity == ValidationSeverity::Error || d.severity == ValidationSeverity::Warning)
                .collect();
            if !errors_or_warnings.is_empty() {
                results.push((format!("{}:{}", module.id(), track_def.id), errors_or_warnings));
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tdrace_core::track::presets::classic_grand_prix;

    #[test]
    fn test_dev_tools_export_to_json_and_rust() {
        let gp = classic_grand_prix();
        let json = export_track_to_json(&gp).expect("Must serialize track to JSON");
        assert!(json.contains("Classic Grand Prix"));

        let rust_code = export_track_to_rust_code(&gp, "my_generated_gp");
        assert!(rust_code.contains("pub fn my_generated_gp() -> Track"));
        assert!(rust_code.contains("Classic Grand Prix"));
    }
}
