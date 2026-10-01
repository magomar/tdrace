use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_triangle};
use glam::Vec2;
use wheelbase::car::Car;
use arcade_race_core::track::scenery::{
    Building, BuildingStyle, Grandstand, GrandstandStyle, Rock, RockType, Tree, TreeType,
};
use arcade_race_core::track::Track;

use super::barrier::SHADOW_OFFSET;
use super::color::Palette;
use super::track::draw_quad;

/// Viewport culling test for grandstands.
#[inline]
pub fn is_grandstand_in_view(stand: &Grandstand, view_bounds: Option<(Vec2, Vec2)>) -> bool {
    if let Some((min, max)) = view_bounds {
        let max_r = (stand.length * 0.5).hypot(stand.depth * 0.5) + 3.0;
        let c = stand.center;
        !(c.x + max_r < min.x || c.x - max_r > max.x || c.y + max_r < min.y || c.y - max_r > max.y)
    } else {
        true
    }
}

/// Viewport culling test for trees.
#[inline]
pub fn is_tree_in_view(tree: &Tree, view_bounds: Option<(Vec2, Vec2)>) -> bool {
    if let Some((min, max)) = view_bounds {
        let max_r = tree.canopy_radius() + 3.0;
        let c = tree.position;
        !(c.x + max_r < min.x || c.x - max_r > max.x || c.y + max_r < min.y || c.y - max_r > max.y)
    } else {
        true
    }
}

/// Viewport culling test for rocks.
#[inline]
pub fn is_rock_in_view(rock: &Rock, view_bounds: Option<(Vec2, Vec2)>) -> bool {
    if let Some((min, max)) = view_bounds {
        let max_r = rock.radius() + 3.0;
        let c = rock.position;
        !(c.x + max_r < min.x || c.x - max_r > max.x || c.y + max_r < min.y || c.y - max_r > max.y)
    } else {
        true
    }
}

/// Viewport culling test for buildings.
#[inline]
pub fn is_building_in_view(building: &Building, view_bounds: Option<(Vec2, Vec2)>) -> bool {
    if let Some((min, max)) = view_bounds {
        let max_r = (building.size.x * 0.5).hypot(building.size.y * 0.5) + 4.0;
        let c = building.center;
        !(c.x + max_r < min.x || c.x - max_r > max.x || c.y + max_r < min.y || c.y - max_r > max.y)
    } else {
        true
    }
}

// =========================================================================
// GRANDSTAND RENDERING
// =========================================================================

/// Draws ground drop shadows for all grandstands.
pub fn render_grandstand_shadows_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    let shadow_color = Color::new(0.0, 0.0, 0.0, 0.28);
    let shadow_shift = SHADOW_OFFSET * 1.8;

    for stand in &track.geometry.grandstands {
        if !is_grandstand_in_view(stand, view_bounds) {
            continue;
        }

        let corners = stand.corners();
        let s0 = corners[0] + shadow_shift;
        let s1 = corners[1] + shadow_shift;
        let s2 = corners[2] + shadow_shift;
        let s3 = corners[3] + shadow_shift;
        draw_quad(s0, s1, s2, s3, shadow_color);
    }
}

/// Renders all grandstands (concrete bleachers, seats, crowd, parapets, roof canopy).
pub fn render_grandstands_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    for stand in &track.geometry.grandstands {
        if !is_grandstand_in_view(stand, view_bounds) {
            continue;
        }
        render_grandstand(stand);
    }
}

/// Renders a single spectator grandstand with stepped concrete tiers and crowd seating.
pub fn render_grandstand(stand: &Grandstand) {
    let corners = stand.corners();
    let fl = corners[0]; // Front-Left
    let fr = corners[1]; // Front-Right
    let rr = corners[2]; // Rear-Right
    let rl = corners[3]; // Rear-Left

    // 1. Solid concrete base apron slab
    draw_quad(fl, fr, rr, rl, Color::new(0.68, 0.70, 0.72, 1.0));

    // 2. Stepped concrete tiers along the depth axis (from front to rear)
    let n_tiers = stand.tiers.max(2);
    let left_edge = rl - fl;
    let right_edge = rr - fr;
    let length_dir = stand.length_direction();
    let facing_dir = stand.facing_normal();

    // Stadium crowd / seat colors palette
    let seat_colors = [
        Color::new(0.85, 0.15, 0.18, 0.95), // Racing red
        Color::new(0.12, 0.45, 0.88, 0.95), // Cobalt blue
        Color::new(0.95, 0.78, 0.12, 0.95), // Golden yellow
        Color::new(0.94, 0.94, 0.96, 0.95), // Alpine white
        Color::new(0.15, 0.75, 0.35, 0.95), // Emerald green
        Color::new(0.92, 0.45, 0.12, 0.95), // Neon orange
    ];

    for t in 0..n_tiers {
        let t0 = t as f32 / n_tiers as f32;
        let t1 = (t + 1) as f32 / n_tiers as f32;

        let v0 = fl + left_edge * t0;
        let v1 = fr + right_edge * t0;
        let v2 = fr + right_edge * t1;
        let v3 = fl + left_edge * t1;

        // Depth lighting: front (lower) tiers are shaded, rear (top) tiers are brightly lit
        let shade = t as f32 / (n_tiers - 1) as f32;
        let tier_col = Color::new(
            0.64 + 0.22 * shade,
            0.66 + 0.22 * shade,
            0.69 + 0.22 * shade,
            1.0,
        );
        draw_quad(v0, v1, v2, v3, tier_col);

        // Tier riser joint line (seam separating tiers)
        let riser_col = Color::new(0.40, 0.42, 0.45, 0.9);
        draw_line(v3.x, v3.y, v2.x, v2.y, 0.16, riser_col);

        // Spectator crowd dots / individual seats on this tier
        let tier_center_line_start = v0 + (v3 - v0) * 0.5;
        let tier_center_line_end = v1 + (v2 - v1) * 0.5;
        let tier_len = (tier_center_line_end - tier_center_line_start).length();

        // Spacing for individual seats (~ 0.85m to 1.1m)
        let n_seats = (tier_len / 0.95).floor() as usize;
        if n_seats > 2 {
            let seat_step = tier_len / n_seats as f32;
            let dir = (tier_center_line_end - tier_center_line_start).normalize_or_zero();

            for s in 1..n_seats {
                let dist = s as f32 * seat_step;
                let offset_from_mid = (dist - tier_len * 0.5).abs();

                // Skip central access stairway aisle (1.4m wide) and lateral gangways
                if offset_from_mid < 0.70 || (offset_from_mid > 12.0 && offset_from_mid < 12.8) {
                    continue;
                }

                let seat_pos = tier_center_line_start + dir * dist;

                // Pick color: custom seat_color or pseudo-random stadium crowd pattern
                let col = if let Some(rgb) = stand.seat_color {
                    let var = (((t as usize * 13 + s * 17) % 7) as f32 - 3.0) * 0.04;
                    Color::new(
                        (rgb[0] + var).clamp(0.0, 1.0),
                        (rgb[1] + var).clamp(0.0, 1.0),
                        (rgb[2] + var).clamp(0.0, 1.0),
                        1.0,
                    )
                } else {
                    let idx = (t as usize * 7 + s * 11 + stand.id * 5) % seat_colors.len();
                    seat_colors[idx]
                };

                draw_circle(seat_pos.x, seat_pos.y, 0.22, col);
            }
        }
    }

    // 3. Central access gangway concrete stairs overlay
    let stairs_w = 1.2;
    let stair_fl = stand.center - length_dir * (stairs_w * 0.5) - facing_dir * (stand.depth * 0.5);
    let stair_fr = stand.center + length_dir * (stairs_w * 0.5) - facing_dir * (stand.depth * 0.5);
    let stair_rr = stand.center + length_dir * (stairs_w * 0.5) + facing_dir * (stand.depth * 0.5);
    let stair_rl = stand.center - length_dir * (stairs_w * 0.5) + facing_dir * (stand.depth * 0.5);
    draw_quad(stair_fl, stair_fr, stair_rr, stair_rl, Color::new(0.55, 0.57, 0.60, 0.65));
    draw_line(stair_fl.x, stair_fl.y, stair_rl.x, stair_rl.y, 0.12, Color::new(0.38, 0.40, 0.42, 0.8));
    draw_line(stair_fr.x, stair_fr.y, stair_rr.x, stair_rr.y, 0.12, Color::new(0.38, 0.40, 0.42, 0.8));

    // 4. Concrete Retaining Walls & Parapets
    // Front trackside barrier wall (thick concrete parapet protecting spectators)
    draw_line(fl.x, fl.y, fr.x, fr.y, 0.40, Palette::CONCRETE_WALL);
    draw_line(fl.x, fl.y, fr.x, fr.y, 0.20, Palette::CONCRETE_TOP);

    // End walls (left and right flanks)
    draw_line(fl.x, fl.y, rl.x, rl.y, 0.32, Palette::CONCRETE_WALL);
    draw_line(fr.x, fr.y, rr.x, rr.y, 0.32, Palette::CONCRETE_WALL);

    // Rear wall / back railing
    draw_line(rl.x, rl.y, rr.x, rr.y, 0.32, Palette::CONCRETE_WALL);

    // 5. Architectural Style Specific Overlays
    match stand.style {
        GrandstandStyle::OpenBleachers => {
            // Distinct crisp perimeter railing
            draw_line(rl.x, rl.y, rr.x, rr.y, 0.15, Palette::STEEL_RAIL);
        }
        GrandstandStyle::CoveredStadium => {
            // Rear cantilever roof canopy extending over the top 55% of tiers
            let roof_depth = stand.depth * 0.58;
            let roof_center = stand.center + facing_dir * (stand.depth * 0.5 - roof_depth * 0.5);
            let roof_fwd = length_dir * (stand.length * 0.5 + 0.6);
            let roof_facing = facing_dir * (roof_depth * 0.5);

            let r_fl = roof_center - roof_fwd - roof_facing;
            let r_fr = roof_center + roof_fwd - roof_facing;
            let r_rr = roof_center + roof_fwd + roof_facing;
            let r_rl = roof_center - roof_fwd + roof_facing;

            // Soft shadow cast by roof onto tiers below
            let roof_shadow_shift = facing_dir * -0.5 + length_dir * 0.3;
            draw_quad(
                r_fl + roof_shadow_shift,
                r_fr + roof_shadow_shift,
                r_rr + roof_shadow_shift,
                r_rl + roof_shadow_shift,
                Color::new(0.0, 0.0, 0.0, 0.30),
            );

            // Modern aerodynamic white cantilever roof slab
            draw_quad(r_fl, r_fr, r_rr, r_rl, Color::new(0.92, 0.94, 0.97, 0.95));

            // Sleek architectural roof panel lines and cyan neon trim
            draw_line(r_fl.x, r_fl.y, r_fr.x, r_fr.y, 0.28, Palette::NEON_CYAN);
            draw_line(r_rl.x, r_rl.y, r_rr.x, r_rr.y, 0.22, Palette::STEEL_POST);

            // Transverse support truss beams
            let n_trusses = (stand.length / 8.0).floor().max(2.0) as usize;
            for i in 0..=n_trusses {
                let frac = i as f32 / n_trusses as f32;
                let b0 = r_fl + (r_fr - r_fl) * frac;
                let b1 = r_rl + (r_rr - r_rl) * frac;
                draw_line(b0.x, b0.y, b1.x, b1.y, 0.16, Color::new(0.70, 0.74, 0.80, 0.7));
            }
        }
        GrandstandStyle::HillsideBleachers => {
            // Earthwork / hillside turf edge trim blending into surrounding terrain
            let grass_trim = Color::new(0.16, 0.44, 0.20, 0.85);
            draw_line(rl.x, rl.y, rr.x, rr.y, 0.60, grass_trim);
            draw_line(fl.x, fl.y, rl.x, rl.y, 0.40, grass_trim);
            draw_line(fr.x, fr.y, rr.x, rr.y, 0.40, grass_trim);
        }
    }
}

// =========================================================================
// TREE RENDERING
// =========================================================================

/// Draws ground cast shadows for all trees.
pub fn render_tree_shadows_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    let shadow_color = Color::new(0.0, 0.0, 0.0, 0.24);

    for tree in &track.geometry.trees {
        if !is_tree_in_view(tree, view_bounds) {
            continue;
        }
        render_single_tree_shadow(tree, shadow_color);
    }
}

/// Draws solid wood trunks for all trees (rendered in obstacle layer).
pub fn render_tree_trunks_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    for tree in &track.geometry.trees {
        if !is_tree_in_view(tree, view_bounds) || !tree.has_trunk() {
            continue;
        }

        let pos = tree.position;
        let r = tree.trunk_radius();

        // Dark woody bark outer perimeter
        draw_circle(pos.x, pos.y, r, Color::new(0.28, 0.18, 0.10, 1.0));
        // Inner annual wood ring
        draw_circle(pos.x, pos.y, r * 0.65, Color::new(0.38, 0.26, 0.15, 1.0));
        // Bark texture outline
        draw_circle_lines(pos.x, pos.y, r, 0.08, Color::new(0.18, 0.11, 0.06, 1.0));
    }
}

/// Draws cenital foliage canopies for all trees (rendered ABOVE vehicles).
/// If any vehicle is underneath the canopy, the canopy smoothly fades to 60% opacity.
pub fn render_tree_canopies_culled(track: &Track, cars: &[Car], view_bounds: Option<(Vec2, Vec2)>) {
    for tree in &track.geometry.trees {
        if !is_tree_in_view(tree, view_bounds) {
            continue;
        }

        // Proximity alpha fading: test if any car is beneath foliage
        let is_car_underneath = cars.iter().any(|c| tree.contains_canopy(c.state.position));
        let alpha = if is_car_underneath { 0.60 } else { 1.00 };

        render_single_tree_canopy(tree, alpha);
    }
}

/// Renders a tree's ground cast shadow matching its top-down silhouette.
fn render_single_tree_shadow(tree: &Tree, color: Color) {
    let shift = SHADOW_OFFSET * (1.6 * tree.scale);
    let shadow_pos = tree.position + shift;
    let r = tree.canopy_radius();

    match tree.tree_type {
        TreeType::Pine | TreeType::SnowPine => {
            // Pointed conifer cone shadow (diamond / star)
            draw_circle(shadow_pos.x, shadow_pos.y, r * 0.85, color);
        }
        TreeType::Palm => {
            // Slender curving trunk shadow + starburst fronds shadow
            let trunk_start = tree.position;
            draw_line(trunk_start.x, trunk_start.y, shadow_pos.x, shadow_pos.y, 0.35 * tree.scale, color);
            draw_circle(shadow_pos.x, shadow_pos.y, r * 0.70, color);
        }
        TreeType::Oak | TreeType::Sakura | TreeType::AutumnMaple => {
            // Puffy multi-lobed cloud shadow
            draw_circle(shadow_pos.x, shadow_pos.y, r * 0.88, color);
        }
        TreeType::Bush => {
            // Low, irregular rounded shrub shadow
            draw_circle(shadow_pos.x, shadow_pos.y, r * 0.82, color);
        }
        TreeType::Cactus => {
            // Saguaro shadow: stem + arm lines
            let arm_len = r * 0.6;
            let dir = Vec2::new(tree.rotation.cos(), tree.rotation.sin());
            let norm = Vec2::new(-dir.y, dir.x);
            draw_circle(shadow_pos.x, shadow_pos.y, r * 0.45, color);
            draw_line(
                shadow_pos.x - dir.x * arm_len,
                shadow_pos.y - dir.y * arm_len,
                shadow_pos.x + dir.x * arm_len,
                shadow_pos.y + dir.y * arm_len,
                0.4 * tree.scale,
                color,
            );
            draw_line(
                shadow_pos.x,
                shadow_pos.y,
                shadow_pos.x + norm.x * arm_len,
                shadow_pos.y + norm.y * arm_len,
                0.35 * tree.scale,
                color,
            );
        }
        TreeType::Cypress => {
            // Elongated narrow flame shadow
            let dir = Vec2::new(tree.rotation.cos(), tree.rotation.sin());
            let p0 = shadow_pos - dir * (r * 0.7);
            let p1 = shadow_pos + dir * (r * 0.7);
            draw_line(p0.x, p0.y, p1.x, p1.y, r * 0.75, color);
        }
    }
}

/// Renders the cenital (top-down) vector canopy for a tree species.
fn render_single_tree_canopy(tree: &Tree, alpha: f32) {
    let pos = tree.position;
    let r = tree.canopy_radius();

    match tree.tree_type {
        TreeType::Pine => {
            // Coniferous evergreen: tiered radial star/spoke needle clusters with sharp apex
            render_pine_canopy(pos, r, tree.rotation, alpha);
        }
        TreeType::SnowPine => {
            // Snow-dusted coniferous evergreen
            render_snow_pine_canopy(pos, r, tree.rotation, alpha);
        }
        TreeType::Bush => {
            // Dense low shrub
            render_bush_canopy(pos, r, tree.id, alpha);
        }
        TreeType::Cactus => {
            // Saguaro desert cactus
            render_cactus_canopy(pos, r, tree.rotation, tree.scale, alpha);
        }
        TreeType::Palm => {
            // Tropical palm: radial starburst crown with arching feather fronds & coconut core
            render_palm_canopy(pos, r, tree.rotation, tree.scale, alpha);
        }
        TreeType::Oak => {
            // Deciduous oak: billowing multi-lobed organic cloud canopy with rich dappled greens
            render_cloud_canopy(
                pos,
                r,
                tree.id,
                Color::new(0.10, 0.30, 0.14, alpha), // Shadow
                Color::new(0.18, 0.48, 0.22, alpha), // Main
                Color::new(0.28, 0.62, 0.30, alpha), // Highlight
            );
        }
        TreeType::Cypress => {
            // Italian cypress: compact, dense, narrow flame/oval crown
            render_cypress_canopy(pos, r, tree.rotation, alpha);
        }
        TreeType::Sakura => {
            // Flowering cherry blossom: soft vibrant floral cloud canopy of pastel pink and magenta petals
            render_cloud_canopy(
                pos,
                r,
                tree.id,
                Color::new(0.65, 0.22, 0.42, alpha), // Plum magenta shadow
                Color::new(0.96, 0.58, 0.75, alpha), // Soft blossom pink
                Color::new(0.99, 0.85, 0.92, alpha), // Highlight floral white
            );
        }
        TreeType::AutumnMaple => {
            // Seasonal autumn maple: warm fiery crown of golden-amber, orange, and crimson foliage lobes
            render_cloud_canopy(
                pos,
                r,
                tree.id,
                Color::new(0.55, 0.12, 0.08, alpha), // Crimson shadow
                Color::new(0.88, 0.38, 0.08, alpha), // Blazing orange
                Color::new(0.98, 0.72, 0.14, alpha), // Golden amber highlight
            );
        }
    }
}

// -------------------------------------------------------------------------
// CENITAL BOTANICAL PROCEDURAL SHAPERS
// -------------------------------------------------------------------------

/// Draws a Coniferous Pine: 4 concentric star/spoke needle tiers with decreasing radius.
fn render_pine_canopy(pos: Vec2, radius: f32, rotation: f32, alpha: f32) {
    let tiers = [
        (1.00, 8, rotation, Color::new(0.08, 0.25, 0.12, alpha)),
        (0.74, 8, rotation + 0.39, Color::new(0.12, 0.35, 0.16, alpha)),
        (0.50, 7, rotation + 0.78, Color::new(0.18, 0.46, 0.22, alpha)),
        (0.28, 6, rotation + 1.17, Color::new(0.24, 0.56, 0.28, alpha)),
    ];

    for (scale, points, rot, color) in tiers {
        let r_outer = radius * scale;
        let r_inner = r_outer * 0.62;
        let angle_step = std::f32::consts::TAU / (points * 2) as f32;

        for i in 0..(points * 2) {
            let a0 = rot + i as f32 * angle_step;
            let a1 = rot + (i + 1) as f32 * angle_step;

            let r0 = if i % 2 == 0 { r_outer } else { r_inner };
            let r1 = if i % 2 == 0 { r_inner } else { r_outer };

            let p0 = pos + Vec2::new(a0.cos(), a0.sin()) * r0;
            let p1 = pos + Vec2::new(a1.cos(), a1.sin()) * r1;

            draw_triangle(
                macroquad::prelude::Vec2::new(pos.x, pos.y),
                macroquad::prelude::Vec2::new(p0.x, p0.y),
                macroquad::prelude::Vec2::new(p1.x, p1.y),
                color,
            );
        }
    }

    // Central needle apex tip
    draw_circle(pos.x, pos.y, radius * 0.12, Color::new(0.32, 0.65, 0.34, alpha));
}

/// Draws a Tropical Palm: Starburst crown of arching feather fronds radiating from a central core.
fn render_palm_canopy(pos: Vec2, radius: f32, rotation: f32, scale: f32, alpha: f32) {
    let frond_count = 10;
    let angle_step = std::f32::consts::TAU / frond_count as f32;

    let frond_spine_col = Color::new(0.38, 0.78, 0.26, alpha);
    let frond_leaf_col = Color::new(0.22, 0.65, 0.18, alpha);
    let frond_highlight_col = Color::new(0.48, 0.88, 0.30, alpha);

    for i in 0..frond_count {
        let base_angle = rotation + i as f32 * angle_step;
        // Slight natural curvature of each frond
        let tip_angle = base_angle + 0.18;
        let tip = pos + Vec2::new(tip_angle.cos(), tip_angle.sin()) * radius;
        let mid = pos + Vec2::new(base_angle.cos(), base_angle.sin()) * (radius * 0.55);

        // Frond spine line
        draw_line(pos.x, pos.y, mid.x, mid.y, 0.28 * scale, frond_spine_col);
        draw_line(mid.x, mid.y, tip.x, tip.y, 0.18 * scale, frond_spine_col);

        // Feather leaflets along the frond
        let spine_dir = (tip - pos).normalize_or_zero();
        let spine_normal = Vec2::new(-spine_dir.y, spine_dir.x);

        for step in 1..=4 {
            let frac = step as f32 / 5.0;
            let leaflet_origin = pos + (tip - pos) * frac;
            let leaflet_len = radius * 0.28 * (1.0 - (frac - 0.45).abs() * 0.8);

            let l_tip = leaflet_origin + (spine_dir * 0.4 + spine_normal) * leaflet_len;
            let r_tip = leaflet_origin + (spine_dir * 0.4 - spine_normal) * leaflet_len;

            let col = if step % 2 == 0 { frond_highlight_col } else { frond_leaf_col };
            draw_line(leaflet_origin.x, leaflet_origin.y, l_tip.x, l_tip.y, 0.16 * scale, col);
            draw_line(leaflet_origin.x, leaflet_origin.y, r_tip.x, r_tip.y, 0.16 * scale, col);
        }
    }

    // Central crown heart
    draw_circle(pos.x, pos.y, radius * 0.24, Color::new(0.18, 0.52, 0.15, alpha));

    // Coconut cluster in crown core
    let coco_col = Color::new(0.42, 0.28, 0.14, alpha);
    let coco_dist = radius * 0.14;
    for k in 0..4 {
        let ca = rotation + k as f32 * (std::f32::consts::PI * 0.5);
        let cp = pos + Vec2::new(ca.cos(), ca.sin()) * coco_dist;
        draw_circle(cp.x, cp.y, radius * 0.08, coco_col);
    }
}

/// Draws an Organic Cloud Canopy (used by Oak, Sakura, and AutumnMaple):
/// Billowing multi-lobed organic foliage puffs with 3 lighting depth levels.
fn render_cloud_canopy(
    pos: Vec2,
    radius: f32,
    seed: usize,
    shadow_col: Color,
    main_col: Color,
    highlight_col: Color,
) {
    // 1. Under-canopy shadow disk
    draw_circle(pos.x, pos.y, radius * 0.88, shadow_col);

    // 2. Main multi-lobed organic foliage puffs
    let num_lobes = 7;
    let angle_step = std::f32::consts::TAU / num_lobes as f32;

    for i in 0..num_lobes {
        let angle = i as f32 * angle_step + ((seed % 17) as f32 * 0.1);
        let dist = radius * (0.52 + ((seed + i * 7) % 5) as f32 * 0.03);
        let puff_r = radius * (0.38 + ((seed * 3 + i * 11) % 4) as f32 * 0.03);
        let puff_pos = pos + Vec2::new(angle.cos(), angle.sin()) * dist;

        draw_circle(puff_pos.x, puff_pos.y, puff_r, main_col);
    }
    // Main center mass
    draw_circle(pos.x, pos.y, radius * 0.58, main_col);

    // 3. Sunlit highlight puffs (offset towards top-left / sunward)
    let sun_offset = Vec2::new(-0.25, -0.25) * radius;
    for i in 0..4 {
        let angle = (i as f32 * 0.8) - 2.4;
        let dist = radius * 0.40;
        let h_pos = pos + sun_offset + Vec2::new(angle.cos(), angle.sin()) * dist;
        let h_r = radius * 0.32;
        draw_circle(h_pos.x, h_pos.y, h_r, highlight_col);
    }
    draw_circle(pos.x + sun_offset.x * 0.5, pos.y + sun_offset.y * 0.5, radius * 0.35, highlight_col);
}

/// Draws an Italian / Mediterranean Cypress: Compact, dense, narrow flame/oval crown.
fn render_cypress_canopy(pos: Vec2, radius: f32, rotation: f32, alpha: f32) {
    let dir = Vec2::new(rotation.cos(), rotation.sin());
    let _norm = Vec2::new(-dir.y, dir.x);

    let len_r = radius * 1.35;
    let width_r = radius * 0.70;

    let base_col = Color::new(0.06, 0.22, 0.09, alpha);
    let mid_col = Color::new(0.10, 0.32, 0.14, alpha);
    let highlight_col = Color::new(0.18, 0.44, 0.20, alpha);

    // Draw dense overlapping oval along heading
    for step in -3..=3 {
        let t = step as f32 / 3.0;
        let step_pos = pos + dir * (len_r * 0.55 * t);
        let r_t = width_r * (1.0 - t.abs() * 0.35);
        draw_circle(step_pos.x, step_pos.y, r_t, base_col);
    }

    // Mid-tone inner spine
    for step in -2..=2 {
        let t = step as f32 / 2.0;
        let step_pos = pos + dir * (len_r * 0.45 * t);
        let r_t = width_r * 0.70 * (1.0 - t.abs() * 0.30);
        draw_circle(step_pos.x, step_pos.y, r_t, mid_col);
    }

    // Highlighted central ridge
    let tip = pos + dir * (len_r * 0.65);
    let base = pos - dir * (len_r * 0.50);
    draw_line(base.x, base.y, tip.x, tip.y, width_r * 0.45, highlight_col);
    draw_circle(tip.x, tip.y, width_r * 0.22, highlight_col);
}

/// Draws a Snow Pine: Conifer star tiers with cold blue-green needle palette and snow drifts on needle tips.
fn render_snow_pine_canopy(pos: Vec2, radius: f32, rotation: f32, alpha: f32) {
    let tiers = [
        (1.00, 8, rotation, Color::new(0.07, 0.22, 0.18, alpha)),
        (0.74, 8, rotation + 0.39, Color::new(0.11, 0.30, 0.24, alpha)),
        (0.50, 7, rotation + 0.78, Color::new(0.15, 0.40, 0.32, alpha)),
        (0.28, 6, rotation + 1.17, Color::new(0.20, 0.50, 0.40, alpha)),
    ];

    for (scale, points, rot, color) in tiers {
        let r_outer = radius * scale;
        let r_inner = r_outer * 0.62;
        let angle_step = std::f32::consts::TAU / (points * 2) as f32;

        for i in 0..(points * 2) {
            let a0 = rot + i as f32 * angle_step;
            let a1 = rot + (i + 1) as f32 * angle_step;

            let r0 = if i % 2 == 0 { r_outer } else { r_inner };
            let r1 = if i % 2 == 0 { r_inner } else { r_outer };

            let p0 = pos + Vec2::new(a0.cos(), a0.sin()) * r0;
            let p1 = pos + Vec2::new(a1.cos(), a1.sin()) * r1;

            draw_triangle(
                macroquad::prelude::Vec2::new(pos.x, pos.y),
                macroquad::prelude::Vec2::new(p0.x, p0.y),
                macroquad::prelude::Vec2::new(p1.x, p1.y),
                color,
            );
        }
    }

    // Snow drifts on outermost needle tips and apex
    let snow_color = Color::new(0.92, 0.95, 0.98, alpha * 0.95);
    let points = 8;
    let angle_step = std::f32::consts::TAU / points as f32;
    for i in 0..points {
        let a = rotation + i as f32 * angle_step;
        let tip = pos + Vec2::new(a.cos(), a.sin()) * (radius * 0.88);
        draw_circle(tip.x, tip.y, radius * 0.14, snow_color);
    }
    // Crown snow cap
    draw_circle(pos.x, pos.y, radius * 0.22, snow_color);
}

/// Draws a Low Shrub / Bush: Dense multi-lobed organic cloud puffs with deep sage green tones.
fn render_bush_canopy(pos: Vec2, radius: f32, seed: usize, alpha: f32) {
    let shadow_col = Color::new(0.12, 0.28, 0.12, alpha);
    let main_col = Color::new(0.24, 0.44, 0.20, alpha);
    let highlight_col = Color::new(0.36, 0.58, 0.28, alpha);
    render_cloud_canopy(pos, radius, seed, shadow_col, main_col, highlight_col);
}

/// Draws a Desert Saguaro Cactus: Ribbed cylindrical succulent stem with lateral branching arms.
fn render_cactus_canopy(pos: Vec2, radius: f32, rotation: f32, scale: f32, alpha: f32) {
    let dir = Vec2::new(rotation.cos(), rotation.sin());

    let stem_col = Color::new(0.18, 0.46, 0.22, alpha);
    let ridge_col = Color::new(0.26, 0.60, 0.30, alpha);
    let spine_col = Color::new(0.85, 0.88, 0.70, alpha);

    let stem_r = radius * 0.42;
    // Central stem
    draw_circle(pos.x, pos.y, stem_r, stem_col);
    draw_circle(pos.x, pos.y, stem_r * 0.70, ridge_col);

    // Lateral arms (saguaro branches)
    let arm_len = radius * 0.75;
    let arm1_end = pos + dir * arm_len;
    let arm2_end = pos - dir * (arm_len * 0.8);

    draw_line(pos.x, pos.y, arm1_end.x, arm1_end.y, stem_r * 0.85, stem_col);
    draw_line(pos.x, pos.y, arm2_end.x, arm2_end.y, stem_r * 0.75, stem_col);

    draw_circle(arm1_end.x, arm1_end.y, stem_r * 0.42, ridge_col);
    draw_circle(arm2_end.x, arm2_end.y, stem_r * 0.38, ridge_col);

    // Spine accents
    draw_circle(pos.x, pos.y, 0.08 * scale, spine_col);
    draw_circle(arm1_end.x, arm1_end.y, 0.06 * scale, spine_col);
    draw_circle(arm2_end.x, arm2_end.y, 0.06 * scale, spine_col);
}

// =========================================================================
// ROCK RENDERING
// =========================================================================

/// Draws ground drop shadows for all rocks.
pub fn render_rock_shadows_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    let shadow_color = Color::new(0.0, 0.0, 0.0, 0.28);
    for rock in &track.geometry.rocks {
        if !is_rock_in_view(rock, view_bounds) {
            continue;
        }
        let shift = SHADOW_OFFSET * (1.3 * rock.scale);
        let pos = rock.position + shift;
        let r = rock.radius();
        draw_circle(pos.x, pos.y, r * 0.95, shadow_color);
    }
}

/// Renders all rocks (granite, sandstone, slate, snow-capped).
pub fn render_rocks_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    for rock in &track.geometry.rocks {
        if !is_rock_in_view(rock, view_bounds) {
            continue;
        }
        render_rock(rock);
    }
}

/// Renders a single rock with procedural faceted stone geometry.
pub fn render_rock(rock: &Rock) {
    let (base_col, facet_col, highlight_col) = match rock.rock_type {
        RockType::Granite => (
            Color::new(0.44, 0.46, 0.48, 1.0),
            Color::new(0.36, 0.38, 0.40, 1.0),
            Color::new(0.60, 0.62, 0.65, 1.0),
        ),
        RockType::Sandstone => (
            Color::new(0.78, 0.48, 0.28, 1.0),
            Color::new(0.65, 0.38, 0.22, 1.0),
            Color::new(0.90, 0.62, 0.40, 1.0),
        ),
        RockType::Slate => (
            Color::new(0.24, 0.26, 0.30, 1.0),
            Color::new(0.18, 0.20, 0.23, 1.0),
            Color::new(0.35, 0.38, 0.42, 1.0),
        ),
        RockType::SnowCapped => (
            Color::new(0.35, 0.37, 0.40, 1.0),
            Color::new(0.25, 0.27, 0.30, 1.0),
            Color::new(0.94, 0.96, 0.98, 1.0),
        ),
    };

    let r = rock.radius();
    let pos = rock.position;
    let n_verts = 7;
    let angle_step = std::f32::consts::TAU / n_verts as f32;
    let seed = rock.id;

    let mut poly = Vec::with_capacity(n_verts);
    for i in 0..n_verts {
        let a = rock.rotation + i as f32 * angle_step;
        let rad_jitter = 0.82 + (((seed + i * 5) % 7) as f32 * 0.05);
        let p = pos + Vec2::new(a.cos(), a.sin()) * (r * rad_jitter);
        poly.push(p);
    }

    for i in 0..n_verts {
        let next = (i + 1) % n_verts;
        let col = if i % 2 == 0 { base_col } else { facet_col };
        draw_triangle(
            macroquad::prelude::Vec2::new(pos.x, pos.y),
            macroquad::prelude::Vec2::new(poly[i].x, poly[i].y),
            macroquad::prelude::Vec2::new(poly[next].x, poly[next].y),
            col,
        );
    }

    for i in 0..n_verts {
        let next = (i + 1) % n_verts;
        draw_line(poly[i].x, poly[i].y, poly[next].x, poly[next].y, 0.12 * rock.scale, facet_col);
    }

    match rock.rock_type {
        RockType::SnowCapped => {
            draw_circle(pos.x - r * 0.15, pos.y - r * 0.15, r * 0.45, highlight_col);
            draw_circle(pos.x, pos.y, r * 0.30, highlight_col);
        }
        _ => {
            let h_pos = pos + Vec2::new(-0.25, -0.25) * r;
            draw_circle(h_pos.x, h_pos.y, r * 0.28, highlight_col);
        }
    }
}

// =========================================================================
// BUILDING RENDERING
// =========================================================================

/// Draws ground drop shadows for all buildings.
pub fn render_building_shadows_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    let shadow_color = Color::new(0.0, 0.0, 0.0, 0.32);
    let shadow_shift = SHADOW_OFFSET * 2.2;

    for b in &track.geometry.buildings {
        if !is_building_in_view(b, view_bounds) {
            continue;
        }
        let corners = b.corners();
        draw_quad(
            corners[0] + shadow_shift,
            corners[1] + shadow_shift,
            corners[2] + shadow_shift,
            corners[3] + shadow_shift,
            shadow_color,
        );
    }
}

/// Renders all buildings (pit garages, control towers, paddock tents, hall walls, pillars).
pub fn render_buildings_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    for b in &track.geometry.buildings {
        if !is_building_in_view(b, view_bounds) {
            continue;
        }
        render_building(b);
    }
}

/// Renders a single building with architectural style details and fictional graphics.
pub fn render_building(b: &Building) {
    let corners = b.corners();
    let fl = corners[0];
    let fr = corners[1];
    let rr = corners[2];
    let rl = corners[3];

    // Base foundation slab
    draw_quad(fl, fr, rr, rl, Color::new(0.40, 0.42, 0.45, 1.0));

    let roof_c = Color::new(b.roof_color[0], b.roof_color[1], b.roof_color[2], 1.0);

    match b.style {
        BuildingStyle::PitGarage => {
            // Main garage roof
            draw_quad(fl, fr, rr, rl, roof_c);

            // Corrugated roofing panel lines along depth
            let n_ribs = (b.size.x / 2.5).floor().max(2.0) as usize;
            for i in 1..n_ribs {
                let frac = i as f32 / n_ribs as f32;
                let p0 = fl + (fr - fl) * frac;
                let p1 = rl + (rr - rl) * frac;
                draw_line(p0.x, p0.y, p1.x, p1.y, 0.18, Color::new(0.15, 0.15, 0.15, 0.4));
            }

            // Pit apron front fascia line / team header
            draw_line(fl.x, fl.y, fr.x, fr.y, 0.45, Color::new(0.95, 0.95, 0.98, 1.0));
            // Rear wall border
            draw_line(rl.x, rl.y, rr.x, rr.y, 0.35, Color::new(0.20, 0.22, 0.25, 1.0));
        }
        BuildingStyle::ControlTower => {
            // Multi-tier race control tower
            // Lower deck
            draw_quad(fl, fr, rr, rl, roof_c);

            // Upper glass observation deck (inset 18%)
            let inset = 0.18;
            let o_fl = fl + (rr - fl) * inset;
            let o_fr = fr + (rl - fr) * inset;
            let o_rr = rr - (rr - fl) * inset;
            let o_rl = rl - (rl - fr) * inset;

            // Tinted cyan glass canopy
            draw_quad(o_fl, o_fr, o_rr, o_rl, Color::new(0.18, 0.65, 0.85, 0.85));
            // Architectural white frame
            draw_line(o_fl.x, o_fl.y, o_fr.x, o_fr.y, 0.25, Color::new(0.95, 0.95, 0.98, 1.0));
            draw_line(o_fr.x, o_fr.y, o_rr.x, o_rr.y, 0.25, Color::new(0.95, 0.95, 0.98, 1.0));
            draw_line(o_rr.x, o_rr.y, o_rl.x, o_rl.y, 0.25, Color::new(0.95, 0.95, 0.98, 1.0));
            draw_line(o_rl.x, o_rl.y, o_fl.x, o_fl.y, 0.25, Color::new(0.95, 0.95, 0.98, 1.0));

            // Central beacon / antenna
            draw_circle(b.center.x, b.center.y, 0.45, Color::new(0.95, 0.80, 0.10, 1.0));
            draw_circle(b.center.x, b.center.y, 0.20, Color::new(1.0, 0.2, 0.2, 1.0));
        }
        BuildingStyle::PaddockTent => {
            // Hospitality marquee canopy with center ridge line
            draw_quad(fl, fr, rr, rl, roof_c);

            // Center ridge dividing the two gable slopes
            let ridge_start = (fl + rl) * 0.5;
            let ridge_end = (fr + rr) * 0.5;
            draw_line(ridge_start.x, ridge_start.y, ridge_end.x, ridge_end.y, 0.35, Color::new(1.0, 1.0, 1.0, 0.85));

            // Striped awning accents
            let n_stripes = (b.size.x / 1.8).floor().max(2.0) as usize;
            for i in 1..n_stripes {
                let frac = i as f32 / n_stripes as f32;
                let p0 = fl + (fr - fl) * frac;
                let p1 = rl + (rr - rl) * frac;
                let col = if i % 2 == 0 { Color::new(1.0, 1.0, 1.0, 0.5) } else { Color::new(0.1, 0.1, 0.1, 0.2) };
                draw_line(p0.x, p0.y, p1.x, p1.y, 0.25, col);
            }

            // Corner support poles
            draw_circle(fl.x, fl.y, 0.22, Color::new(0.2, 0.2, 0.2, 1.0));
            draw_circle(fr.x, fr.y, 0.22, Color::new(0.2, 0.2, 0.2, 1.0));
            draw_circle(rr.x, rr.y, 0.22, Color::new(0.2, 0.2, 0.2, 1.0));
            draw_circle(rl.x, rl.y, 0.22, Color::new(0.2, 0.2, 0.2, 1.0));
        }
        BuildingStyle::HallWall => {
            // Indoor karting enclosure wall: heavy industrial paneling
            draw_quad(fl, fr, rr, rl, roof_c);

            // High-contrast hazard / racing trim along length
            let border_col = Color::new(0.20, 0.22, 0.26, 1.0);
            draw_line(fl.x, fl.y, fr.x, fr.y, 0.40, border_col);
            draw_line(rl.x, rl.y, rr.x, rr.y, 0.40, border_col);

            // Transverse structural beam ribs
            let n_beams = (b.size.x / 3.0).floor().max(2.0) as usize;
            for i in 1..n_beams {
                let frac = i as f32 / n_beams as f32;
                let p0 = fl + (fr - fl) * frac;
                let p1 = rl + (rr - rl) * frac;
                draw_line(p0.x, p0.y, p1.x, p1.y, 0.22, Color::new(0.85, 0.70, 0.15, 0.75));
            }
        }
        BuildingStyle::Pillar => {
            // Indoor support column: reinforced circular or square pillar
            let min_dim = b.size.x.min(b.size.y);
            let r = min_dim * 0.48;
            draw_circle(b.center.x, b.center.y, r, Color::new(0.32, 0.34, 0.38, 1.0));
            // Warning yellow / black hazard ring
            draw_circle(b.center.x, b.center.y, r * 0.72, Color::new(0.92, 0.78, 0.12, 1.0));
            draw_circle(b.center.x, b.center.y, r * 0.45, Color::new(0.20, 0.22, 0.25, 1.0));
        }
    }
}

