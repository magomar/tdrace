use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_line};
use super::track::draw_quad;
use crate::module::VehicleVisualType;

/// Lighting configuration and emitter profile for a vehicle model or archetype.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleLightingConfig {
    /// Enables front projector headlights / daytime running lights (DRL).
    pub has_headlights: bool,
    /// Enables functional rear red taillights and reactive brake lights.
    pub has_brake_lights: bool,
    /// Enables high-intensity 4-to-5 pod roof-mounted off-road lightbar.
    pub has_roof_lightbar: bool,
    /// Enables hood-mounted auxiliary rally spotlight pods.
    pub has_rally_pods: bool,
    /// Enables high-mount rear amber dust chase strobe for off-road/desert racing.
    pub has_dust_chase_light: bool,
    /// Core color of the front headlights (e.g. Pure White for GT3, Selective Yellow for GTE/LMGT3).
    pub headlight_color: Color,
    /// Enables forward track surface light cone projection onto the 2D racing surface.
    pub project_track_beams: bool,
    /// Angular beam spread half-angle in radians (e.g. 0.18 rad for GT, 0.32 rad for Off-Road).
    pub beam_spread_rad: f32,
    /// Forward reach of the illuminated track cone in world meters (e.g. 14.0m - 18.0m).
    pub beam_range_m: f32,
}

impl Default for VehicleLightingConfig {
    fn default() -> Self {
        Self::gt_touring()
    }
}

impl VehicleLightingConfig {
    /// Zero electrical lighting: used for Karts and NASCAR Cup stock cars.
    pub const fn none() -> Self {
        Self {
            has_headlights: false,
            has_brake_lights: false,
            has_roof_lightbar: false,
            has_rally_pods: false,
            has_dust_chase_light: false,
            headlight_color: Color::new(1.0, 1.0, 1.0, 0.0),
            project_track_beams: false,
            beam_spread_rad: 0.0,
            beam_range_m: 0.0,
        }
    }

    /// Full GT3 / Touring car lighting: high-intensity LED projectors, DRL, responsive brake lights.
    pub fn gt_touring() -> Self {
        Self {
            has_headlights: true,
            has_brake_lights: true,
            has_roof_lightbar: false,
            has_rally_pods: false,
            has_dust_chase_light: false,
            headlight_color: Color::new(0.95, 0.98, 1.0, 0.95),
            project_track_beams: true,
            beam_spread_rad: 0.18,
            beam_range_m: 14.0,
        }
    }

    /// WRC / Rallycross lighting: headlights, reactive brake lights, hood light pods, forward beam.
    pub fn rally() -> Self {
        Self {
            has_headlights: true,
            has_brake_lights: true,
            has_roof_lightbar: false,
            has_rally_pods: true,
            has_dust_chase_light: false,
            headlight_color: Color::new(1.0, 0.98, 0.90, 0.95),
            project_track_beams: true,
            beam_spread_rad: 0.24,
            beam_range_m: 16.0,
        }
    }

    /// Extreme Off-Road lighting: roof lightbar, prerunner spots, brake lights, rear dust chase light.
    pub fn extreme_offroad() -> Self {
        Self {
            has_headlights: true,
            has_brake_lights: true,
            has_roof_lightbar: true,
            has_rally_pods: false,
            has_dust_chase_light: true,
            headlight_color: Color::new(1.0, 0.95, 0.82, 0.98),
            project_track_beams: true,
            beam_spread_rad: 0.32,
            beam_range_m: 18.0,
        }
    }
}

/// Resolves the effective lighting configuration for any vehicle based on model ID, visual type, and modality.
pub fn resolve_vehicle_lighting(
    model_id: Option<&str>,
    visual_type: VehicleVisualType,
) -> VehicleLightingConfig {
    if let Some(m_id) = model_id {
        // 1. Explicit model ID and prefix resolution (including classic fantasy cars)
        if m_id.starts_with("kart") || m_id.starts_with("classic_kart") {
            return VehicleLightingConfig::none();
        }
        if m_id.starts_with("nascar") || m_id.starts_with("classic_nascar") {
            return VehicleLightingConfig::none();
        }
        if m_id.starts_with("rally") || m_id.starts_with("classic_rally") {
            return VehicleLightingConfig::rally();
        }
        if m_id.starts_with("offroad")
            || m_id.starts_with("classic_offroad")
            || m_id.contains("baja")
            || m_id.contains("sand_rail")
        {
            return VehicleLightingConfig::extreme_offroad();
        }
        if m_id.starts_with("gt") || m_id.starts_with("classic_gt") {
            return VehicleLightingConfig::gt_touring();
        }

        // 2. Catalog module_id lookup for real cars
        if let Some(model) = crate::catalog::find_model_by_id(m_id) {
            match model.module_id {
                "kart" => return VehicleLightingConfig::none(),
                "nascar" => return VehicleLightingConfig::none(),
                "rally" => return VehicleLightingConfig::rally(),
                "extreme_offroad" => return VehicleLightingConfig::extreme_offroad(),
                "gt" => return VehicleLightingConfig::gt_touring(),
                _ => {}
            }
        }
    }

    // 3. Procedural archetype fallback
    match visual_type {
        VehicleVisualType::GoKart { .. } => VehicleLightingConfig::none(),
        VehicleVisualType::StockCar { .. } => VehicleLightingConfig::none(),
        VehicleVisualType::OpenWheel { .. } => VehicleLightingConfig::none(),
        VehicleVisualType::RallyHatch { .. } => VehicleLightingConfig::rally(),
        VehicleVisualType::SandRail { .. } => VehicleLightingConfig::extreme_offroad(),
        VehicleVisualType::TouringGT { .. } => VehicleLightingConfig::gt_touring(),
    }
}

/// Projects forward headlight cones onto the track surface ahead of the vehicle.
pub fn render_headlight_track_beams(
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
    cfg: &VehicleLightingConfig,
) {
    if !cfg.project_track_beams || cfg.beam_range_m <= 0.0 {
        return;
    }

    let light_w = half_w * 0.55;
    let head_l = pos + fwd * (half_len - 0.05) - right * light_w;
    let head_r = pos + fwd * (half_len - 0.05) + right * light_w;

    let range = cfg.beam_range_m;
    let spread = cfg.beam_spread_rad.max(0.05);

    // Multi-stage distance falloff: near (30% range), mid (65% range), far (100% range)
    let stages = [
        (0.30 * range, 0.04),
        (0.65 * range, 0.02),
        (1.00 * range, 0.008),
    ];

    let base_color = cfg.headlight_color;

    for origin in [head_l, head_r] {
        let mut prev_d = 0.0;
        let mut prev_hw = 0.22;

        for (d, alpha_end) in stages {
            let next_hw = 0.22 + d * spread.tan();
            let p_prev_l = origin + fwd * prev_d - right * prev_hw;
            let p_prev_r = origin + fwd * prev_d + right * prev_hw;
            let p_next_r = origin + fwd * d + right * next_hw;
            let p_next_l = origin + fwd * d - right * next_hw;

            let col = Color::new(base_color.r, base_color.g, base_color.b, alpha_end);
            draw_quad(p_prev_l, p_prev_r, p_next_r, p_next_l, col);

            prev_d = d;
            prev_hw = next_hw;
        }
    }
}

/// Renders hood-mounted quad rally spotlight cluster on the front fascia.
pub fn render_rally_hood_pods(
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
) {
    let pod_center = pos + fwd * (half_len * 0.75);
    let pod_hw = half_w * 0.40;

    // Mounting bracket bar
    let b_l = pod_center - right * pod_hw;
    let b_r = pod_center + right * pod_hw;
    draw_line(b_l.x, b_l.y, b_r.x, b_r.y, 0.04, Color::new(0.12, 0.12, 0.15, 1.0));

    // 4 high-output round rally lights
    for step in [-0.75, -0.25, 0.25, 0.75] {
        let p = pod_center + right * (pod_hw * step);
        // Outer housing
        draw_circle(p.x, p.y, 0.08, Color::new(0.20, 0.20, 0.24, 1.0));
        // Lens glow halo
        draw_circle(p.x, p.y, 0.14, Color::new(1.0, 0.95, 0.75, 0.35));
        // Bright core
        draw_circle(p.x, p.y, 0.05, Color::new(1.0, 1.0, 0.90, 0.98));
    }
}

/// Renders high-mount rear amber dust chase strobe for off-road desert/snow racing.
pub fn render_dust_chase_strobe(
    pos: Vec2,
    fwd: Vec2,
    half_len: f32,
) {
    let chase_pos = pos - fwd * (half_len * 0.58);
    // Subtle pulsing strobe
    let strobe_phase = (pos.x * 25.0 + pos.y * 25.0).sin().abs();
    let alpha = 0.75 + strobe_phase * 0.25;

    // Outer amber glow
    draw_circle(chase_pos.x, chase_pos.y, 0.15, Color::new(1.0, 0.55, 0.0, alpha * 0.45));
    // Inner high-intensity LED
    draw_circle(chase_pos.x, chase_pos.y, 0.07, Color::new(1.0, 0.75, 0.15, alpha));
}
