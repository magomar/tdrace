use glam::Vec2;
use std::f32::consts::PI;
use tdrace_core::physics::car::Car;
use tdrace_core::track::geometry::{ObstacleShape, SurfaceShape};
use tdrace_core::track::spline::TrackSpline;
use tdrace_core::track::Track;

/// An individual skid mark segment in world coordinates.
#[derive(Debug, Clone, Copy)]
pub struct SkidMark {
    pub start: Vec2,
    pub end: Vec2,
    pub width: f32,
    pub alpha: f32, // 0.0 to 1.0
}

/// A pre-tessellated 2D colored triangle in world space with precomputed AABB bounds (Spec 118).
#[derive(Debug, Clone, Copy)]
pub struct WorldTriangle {
    pub v0: Vec2,
    pub v1: Vec2,
    pub v2: Vec2,
    pub min: Vec2,
    pub max: Vec2,
    pub color: [u8; 3],
}

impl WorldTriangle {
    #[inline]
    pub fn new(v0: Vec2, v1: Vec2, v2: Vec2, color: [u8; 3]) -> Self {
        Self {
            v0,
            v1,
            v2,
            min: v0.min(v1).min(v2),
            max: v0.max(v1).max(v2),
            color,
        }
    }

    #[inline(always)]
    pub fn intersects_aabb(&self, cam_min: Vec2, cam_max: Vec2) -> bool {
        self.max.x >= cam_min.x
            && self.min.x <= cam_max.x
            && self.max.y >= cam_min.y
            && self.min.y <= cam_max.y
    }
}

/// Fast top-down software RGB rasterizer for high-throughput pixel observations.
pub struct FastRasterizer {
    pub track_triangles: Vec<WorldTriangle>,
    pub wall_segments: Vec<(Vec2, Vec2, f32, [u8; 3])>,
    pub sand_triangles: Vec<WorldTriangle>,
    pub default_scale: f32, // pixels per meter
}

impl FastRasterizer {
    /// Builds and pre-tessellates track geometry for ultra-fast rendering.
    pub fn new(track: &Track) -> Self {
        let mut track_triangles = Vec::new();
        let mut wall_segments = Vec::new();
        let mut sand_triangles = Vec::new();

        // 1. Tessellate track spline ribbon into quads/triangles
        Self::tessellate_spline(&track.spline, &mut track_triangles);

        // 2. Tessellate surface zones (sand traps, oil slicks, water puddles, dirt areas)
        for zone in &track.geometry.surface_zones {
            let color = match zone.surface {
                tdrace_core::physics::surface::SurfaceType::PackedSand
                | tdrace_core::physics::surface::SurfaceType::DeepSand => [218, 204, 150],
                tdrace_core::physics::surface::SurfaceType::Dirt => [122, 89, 56],
                tdrace_core::physics::surface::SurfaceType::MudTrack
                | tdrace_core::physics::surface::SurfaceType::DeepMud => [100, 70, 40],
                tdrace_core::physics::surface::SurfaceType::PackedSnow
                | tdrace_core::physics::surface::SurfaceType::DeepSnow => [240, 245, 255],
                tdrace_core::physics::surface::SurfaceType::Water => [46, 148, 224],
                tdrace_core::physics::surface::SurfaceType::Oil => [40, 35, 45],
                tdrace_core::physics::surface::SurfaceType::Curb => [220, 50, 50],
                tdrace_core::physics::surface::SurfaceType::SheetIce => [200, 220, 240],
                tdrace_core::physics::surface::SurfaceType::Concrete => [180, 185, 190],
                _ => [120, 120, 120],
            };
            Self::tessellate_surface_shape(&zone.shape, color, &mut sand_triangles);
        }

        // 3. Walls and barriers
        for wall in track.geometry.all_walls().filter(|w| w.is_physical()) {
            let color = match wall.barrier_type {
                tdrace_core::track::geometry::BarrierType::Concrete => [180, 180, 185],
                tdrace_core::track::geometry::BarrierType::TireWall => [200, 30, 30],
                tdrace_core::track::geometry::BarrierType::Steel => [160, 165, 175],
                tdrace_core::track::geometry::BarrierType::CurbWall => [190, 190, 190],
                tdrace_core::track::geometry::BarrierType::Virtual => [0, 0, 0],
            };
            wall_segments.push((wall.segment.start, wall.segment.end, 0.4, color));
        }

        Self {
            track_triangles,
            wall_segments,
            sand_triangles,
            default_scale: 1.8, // default ~53m x 53m view for 96x96
        }
    }

    fn tessellate_spline(spline: &TrackSpline, out: &mut Vec<WorldTriangle>) {
        let total_len = spline.total_length();
        if total_len < 10.0 {
            return;
        }

        let step_size = 2.0; // sample every 2 meters
        let num_steps = (total_len / step_size).ceil() as usize;
        let ds = total_len / num_steps as f32;

        let asphalt_color = [60, 60, 65];
        let concrete_color = [180, 185, 190];
        let dirt_color = [122, 89, 56];
        let curb_red = [220, 45, 45];
        let curb_white = [240, 240, 240];
        let line_white = [255, 255, 255];

        for i in 0..num_steps {
            let s0 = i as f32 * ds;
            let s1 = if i + 1 == num_steps { total_len } else { (i + 1) as f32 * ds };

            let samp0 = spline.sample_at_distance(s0);
            let samp1 = spline.sample_at_distance(s1);

            let p0 = samp0.point;
            let n0 = samp0.normal;
            let w0 = samp0.width * 0.5;
            let c0 = if samp0.left_curb || samp0.right_curb {
                TrackSpline::DEFAULT_CURB_WIDTH
            } else {
                0.0
            };

            let p1 = samp1.point;
            let n1 = samp1.normal;
            let w1 = samp1.width * 0.5;
            let c1 = if samp1.left_curb || samp1.right_curb {
                TrackSpline::DEFAULT_CURB_WIDTH
            } else {
                0.0
            };

            // Track edges
            let l_track0 = p0 - n0 * w0;
            let r_track0 = p0 + n0 * w0;
            let l_track1 = p1 - n1 * w1;
            let r_track1 = p1 + n1 * w1;

            // Curb outer edges
            let l_curb0 = p0 - n0 * (w0 + c0);
            let r_curb0 = p0 + n0 * (w0 + c0);
            let l_curb1 = p1 - n1 * (w1 + c1);
            let r_curb1 = p1 + n1 * (w1 + c1);

            let road_color = match samp0.surface {
                tdrace_core::physics::surface::SurfaceType::Dirt => dirt_color,
                tdrace_core::physics::surface::SurfaceType::Concrete => concrete_color,
                _ => asphalt_color,
            };

            // 1. Road quad (2 triangles)
            out.push(WorldTriangle::new(l_track0, r_track0, l_track1, road_color));
            out.push(WorldTriangle::new(r_track0, r_track1, l_track1, road_color));

            // 2. Curbs (alternating red/white stripes)
            let curb_color = if (i / 2) % 2 == 0 { curb_red } else { curb_white };
            if c0 > 0.1 || c1 > 0.1 {
                // Left curb
                if samp0.left_curb || samp1.left_curb {
                    out.push(WorldTriangle::new(l_curb0, l_track0, l_curb1, curb_color));
                    out.push(WorldTriangle::new(l_track0, l_track1, l_curb1, curb_color));
                }
                // Right curb
                if samp0.right_curb || samp1.right_curb {
                    out.push(WorldTriangle::new(r_track0, r_curb0, r_track1, curb_color));
                    out.push(WorldTriangle::new(r_curb0, r_curb1, r_track1, curb_color));
                }
            }

            // 3. Center dashed line (every other segment on asphalt)
            if samp0.surface != tdrace_core::physics::surface::SurfaceType::Dirt && i % 3 == 0 {
                let lw0 = 0.15;
                let c_l0 = p0 - n0 * lw0;
                let c_r0 = p0 + n0 * lw0;
                let c_l1 = p1 - n1 * lw0;
                let c_r1 = p1 + n1 * lw0;
                out.push(WorldTriangle::new(c_l0, c_r0, c_l1, line_white));
                out.push(WorldTriangle::new(c_r0, c_r1, c_l1, line_white));
            }
        }
    }

    fn tessellate_surface_shape(shape: &SurfaceShape, color: [u8; 3], out: &mut Vec<WorldTriangle>) {
        match shape {
            SurfaceShape::OrientedBox { center, half_extents, angle } => {
                let cos_r = angle.cos();
                let sin_r = angle.sin();
                let u = Vec2::new(cos_r, sin_r) * half_extents.x;
                let v = Vec2::new(-sin_r, cos_r) * half_extents.y;
                let p0 = *center - u - v;
                let p1 = *center + u - v;
                let p2 = *center + u + v;
                let p3 = *center - u + v;
                out.push(WorldTriangle::new(p0, p1, p2, color));
                out.push(WorldTriangle::new(p0, p2, p3, color));
            }
            SurfaceShape::Aabb { min, max } => {
                let p0 = *min;
                let p1 = Vec2::new(max.x, min.y);
                let p2 = *max;
                let p3 = Vec2::new(min.x, max.y);
                out.push(WorldTriangle::new(p0, p1, p2, color));
                out.push(WorldTriangle::new(p0, p2, p3, color));
            }
            SurfaceShape::Polygon { vertices } => {
                if vertices.len() >= 3 {
                    let v0 = vertices[0];
                    for i in 1..vertices.len() - 1 {
                        out.push(WorldTriangle::new(
                            v0,
                            vertices[i],
                            vertices[i + 1],
                            color,
                        ));
                    }
                }
            }
            SurfaceShape::Circle { center, radius } => {
                let segments = 16;
                for i in 0..segments {
                    let a0 = (i as f32 / segments as f32) * 2.0 * PI;
                    let a1 = ((i + 1) as f32 / segments as f32) * 2.0 * PI;
                    let p0 = *center;
                    let p1 = *center + Vec2::new(a0.cos(), a0.sin()) * *radius;
                    let p2 = *center + Vec2::new(a1.cos(), a1.sin()) * *radius;
                    out.push(WorldTriangle::new(p0, p1, p2, color));
                }
            }
        }
    }

    /// Renders the complete scene to an RGB buffer of shape (height, width, 3).
    pub fn render(
        &self,
        target_car: &Car,
        opponents: &[Car],
        obstacles: &[tdrace_core::track::geometry::Obstacle],
        skid_marks: &[SkidMark],
        width: usize,
        height: usize,
        follow_car: bool,
        scale: Option<f32>,
        buffer: &mut [u8],
    ) {
        assert_eq!(buffer.len(), width * height * 3);

        // 1. Fill background with Grass green
        let grass_color = [88, 160, 88];
        for chunk in buffer.chunks_exact_mut(3) {
            chunk.copy_from_slice(&grass_color);
        }

        let scale = scale.unwrap_or(self.default_scale);
        let center = target_car.state.position;
        let car_angle = target_car.state.angle;

        // Camera transform parameters
        let (cam_pos, rot_cos, rot_sin) = if follow_car {
            // Target car points UP (heading aligned with -Y screen / 90 deg)
            let rot = -car_angle - PI * 0.5;
            (center, rot.cos(), rot.sin())
        } else {
            (center, 1.0, 0.0)
        };

        let half_w = width as f32 * 0.5;
        let half_h = height as f32 * 0.5;

        // World-space camera AABB for culling off-screen triangles and walls before projection (Spec 118)
        let inv_scale = 1.0 / scale.max(1e-4);
        let (cam_min, cam_max) = if follow_car {
            let r = (half_w * half_w + half_h * half_h).sqrt() * inv_scale + 2.0;
            (cam_pos - Vec2::splat(r), cam_pos + Vec2::splat(r))
        } else {
            let ext = Vec2::new(half_w * inv_scale + 2.0, half_h * inv_scale + 2.0);
            (cam_pos - ext, cam_pos + ext)
        };

        // World to Screen coordinate transform closure
        let world_to_screen = |wp: Vec2| -> Vec2 {
            let dx = wp.x - cam_pos.x;
            let dy = wp.y - cam_pos.y;
            let rx = dx * rot_cos - dy * rot_sin;
            let ry = dx * rot_sin + dy * rot_cos;
            Vec2::new(half_w + rx * scale, half_h - ry * scale)
        };

        // 2. Render surface zones (sand traps)
        for tri in &self.sand_triangles {
            if !tri.intersects_aabb(cam_min, cam_max) {
                continue;
            }
            Self::draw_screen_triangle(
                world_to_screen(tri.v0),
                world_to_screen(tri.v1),
                world_to_screen(tri.v2),
                tri.color,
                width,
                height,
                buffer,
            );
        }

        // 3. Render track road & curbs
        for tri in &self.track_triangles {
            if !tri.intersects_aabb(cam_min, cam_max) {
                continue;
            }
            Self::draw_screen_triangle(
                world_to_screen(tri.v0),
                world_to_screen(tri.v1),
                world_to_screen(tri.v2),
                tri.color,
                width,
                height,
                buffer,
            );
        }

        // 4. Render skid marks
        let skid_color = [35, 35, 40];
        for skid in skid_marks {
            let s_min = skid.start.min(skid.end);
            let s_max = skid.start.max(skid.end);
            if s_max.x < cam_min.x || s_min.x > cam_max.x || s_max.y < cam_min.y || s_min.y > cam_max.y {
                continue;
            }
            let s0 = world_to_screen(skid.start);
            let s1 = world_to_screen(skid.end);
            Self::draw_thick_line(s0, s1, (skid.width * scale).max(1.0), skid_color, width, height, buffer);
        }

        // 5. Render walls and barriers
        for &(start, end, thick, color) in &self.wall_segments {
            let w_min = start.min(end);
            let w_max = start.max(end);
            if w_max.x < cam_min.x || w_min.x > cam_max.x || w_max.y < cam_min.y || w_min.y > cam_max.y {
                continue;
            }
            let s0 = world_to_screen(start);
            let s1 = world_to_screen(end);
            Self::draw_thick_line(s0, s1, (thick * scale).max(1.5), color, width, height, buffer);
        }

        // 6. Render static obstacles
        for obs in obstacles {
            match &obs.shape {
                ObstacleShape::Circle { center, radius } => {
                    let sc = world_to_screen(*center);
                    let sr = radius * scale;
                    Self::draw_circle(sc, sr, [220, 90, 30], width, height, buffer);
                }
                ObstacleShape::Box { center, half_extents, angle } => {
                    let cos_r = angle.cos();
                    let sin_r = angle.sin();
                    let u = Vec2::new(cos_r, sin_r) * half_extents.x;
                    let v = Vec2::new(-sin_r, cos_r) * half_extents.y;
                    let p0 = world_to_screen(*center - u - v);
                    let p1 = world_to_screen(*center + u - v);
                    let p2 = world_to_screen(*center + u + v);
                    let p3 = world_to_screen(*center - u + v);
                    Self::draw_screen_triangle(p0, p1, p2, [220, 90, 30], width, height, buffer);
                    Self::draw_screen_triangle(p0, p2, p3, [220, 90, 30], width, height, buffer);
                }
                ObstacleShape::Polygon { vertices } => {
                    if vertices.len() >= 3 {
                        let p0 = world_to_screen(vertices[0]);
                        for i in 1..vertices.len() - 1 {
                            let p1 = world_to_screen(vertices[i]);
                            let p2 = world_to_screen(vertices[i + 1]);
                            Self::draw_screen_triangle(p0, p1, p2, [220, 90, 30], width, height, buffer);
                        }
                    }
                }
            }
        }

        // 7. Render opponent cars (skipping target_car if opponents slice is &self.cars)
        let opp_colors = [
            [35, 115, 230], // Blue
            [235, 130, 25], // Orange
            [40, 190, 75],  // Green
            [165, 55, 210], // Purple
            [230, 215, 35], // Yellow
            [210, 50, 130], // Pink
        ];

        let mut opp_idx = 0usize;
        for opp in opponents {
            if std::ptr::eq(opp, target_car) {
                continue;
            }
            let color = opp_colors[opp_idx % opp_colors.len()];
            opp_idx += 1;
            Self::draw_car(opp, color, &world_to_screen, width, height, buffer);
        }

        // 8. Render target player car (Bright Red)
        Self::draw_car(target_car, [230, 30, 30], &world_to_screen, width, height, buffer);
    }

    fn draw_car<F>(
        car: &Car,
        body_color: [u8; 3],
        w2s: &F,
        width: usize,
        height: usize,
        buffer: &mut [u8],
    ) where
        F: Fn(Vec2) -> Vec2,
    {
        let pos = car.state.position;
        let fwd = car.forward_vector();
        let right = car.right_vector();

        let geom_offset = car.config.chassis.geometric_center_offset_from_cg(car.config.cg_to_front, car.config.cg_to_rear);
        let center = pos + fwd * geom_offset;
        let half_l = car.config.chassis.half_length(car.config.wheelbase);
        let half_w = car.config.chassis.half_width();

        // 4 Car chassis corners
        let c_fl = w2s(center + fwd * half_l - right * half_w);
        let c_fr = w2s(center + fwd * half_l + right * half_w);
        let c_rr = w2s(center - fwd * half_l + right * half_w);
        let c_rl = w2s(center - fwd * half_l - right * half_w);

        // Draw car body quad
        Self::draw_screen_triangle(c_fl, c_fr, c_rr, body_color, width, height, buffer);
        Self::draw_screen_triangle(c_fl, c_rr, c_rl, body_color, width, height, buffer);

        // Draw roof / windshield quad
        let roof_l = half_l * 0.45;
        let roof_w = half_w * 0.70;
        let r_fl = w2s(center + fwd * roof_l - right * roof_w);
        let r_fr = w2s(center + fwd * roof_l + right * roof_w);
        let r_rr = w2s(center - fwd * roof_l + right * roof_w);
        let r_rl = w2s(center - fwd * roof_l - right * roof_w);
        let roof_color = [35, 40, 50];
        Self::draw_screen_triangle(r_fl, r_fr, r_rr, roof_color, width, height, buffer);
        Self::draw_screen_triangle(r_fl, r_rr, r_rl, roof_color, width, height, buffer);

        // Draw 4 Wheels (black rectangles)
        let wheel_positions = car.wheel_positions_world();
        let wheel_l = 0.45;
        let wheel_w = 0.22;
        let wheel_color = [15, 15, 15];

        for (i, &w_pos) in wheel_positions.iter().enumerate() {
            let w_angle = car.state.angle + if i < 2 { car.state.steer_angle } else { 0.0 };
            let w_fwd = Vec2::new(w_angle.cos(), w_angle.sin());
            let w_right = Vec2::new(w_angle.sin(), -w_angle.cos());

            let p0 = w2s(w_pos + w_fwd * wheel_l - w_right * wheel_w);
            let p1 = w2s(w_pos + w_fwd * wheel_l + w_right * wheel_w);
            let p2 = w2s(w_pos - w_fwd * wheel_l + w_right * wheel_w);
            let p3 = w2s(w_pos - w_fwd * wheel_l - w_right * wheel_w);

            Self::draw_screen_triangle(p0, p1, p2, wheel_color, width, height, buffer);
            Self::draw_screen_triangle(p0, p2, p3, wheel_color, width, height, buffer);
        }

        // Headlights (yellow) & Taillights (red)
        let h_l = w2s(pos + fwd * (half_l * 0.95) - right * (half_w * 0.7));
        let h_r = w2s(pos + fwd * (half_l * 0.95) + right * (half_w * 0.7));
        let t_l = w2s(pos - fwd * (half_l * 0.95) - right * (half_w * 0.7));
        let t_r = w2s(pos - fwd * (half_l * 0.95) + right * (half_w * 0.7));

        Self::draw_point(h_l, [255, 250, 160], width, height, buffer);
        Self::draw_point(h_r, [255, 250, 160], width, height, buffer);
        Self::draw_point(t_l, [255, 30, 30], width, height, buffer);
        Self::draw_point(t_r, [255, 30, 30], width, height, buffer);
    }

    /// Fast scanline half-space rasterizer for a screen triangle with incremental edge stepping (Spec 118).
    #[inline]
    fn draw_screen_triangle(
        v0: Vec2,
        v1: Vec2,
        v2: Vec2,
        color: [u8; 3],
        width: usize,
        height: usize,
        buffer: &mut [u8],
    ) {
        let min_x = v0.x.min(v1.x).min(v2.x);
        let max_x = v0.x.max(v1.x).max(v2.x);
        let min_y = v0.y.min(v1.y).min(v2.y);
        let max_y = v0.y.max(v1.y).max(v2.y);

        let w_f = width as f32;
        let h_f = height as f32;

        // Frustum culling
        if max_x < 0.0 || min_x >= w_f || max_y < 0.0 || min_y >= h_f {
            return;
        }

        let x0 = min_x.floor().max(0.0) as usize;
        let x1 = (max_x.ceil().min(w_f - 1.0) as usize).min(width - 1);
        let y0 = min_y.floor().max(0.0) as usize;
        let y1 = (max_y.ceil().min(h_f - 1.0) as usize).min(height - 1);

        // Edge function determinant
        let area = (v1.x - v0.x) * (v2.y - v0.y) - (v1.y - v0.y) * (v2.x - v0.x);
        if area.abs() < 1e-4 {
            return;
        }

        let (p0, p1, p2) = if area < 0.0 { (v0, v2, v1) } else { (v0, v1, v2) };

        let dx01 = p1.x - p0.x;
        let dy01 = p1.y - p0.y;
        let dx12 = p2.x - p1.x;
        let dy12 = p2.y - p1.y;
        let dx20 = p0.x - p2.x;
        let dy20 = p0.y - p2.y;

        let px0 = x0 as f32 + 0.5;
        let py0 = y0 as f32 + 0.5;
        let mut e01_row = dx01 * (py0 - p0.y) - dy01 * (px0 - p0.x);
        let mut e12_row = dx12 * (py0 - p1.y) - dy12 * (px0 - p1.x);
        let mut e20_row = dx20 * (py0 - p2.y) - dy20 * (px0 - p2.x);

        for y in y0..=y1 {
            let row_offset = y * width * 3;
            let mut e01 = e01_row;
            let mut e12 = e12_row;
            let mut e20 = e20_row;

            for x in x0..=x1 {
                if e01 >= -1e-5 && e12 >= -1e-5 && e20 >= -1e-5 {
                    let idx = row_offset + x * 3;
                    buffer[idx] = color[0];
                    buffer[idx + 1] = color[1];
                    buffer[idx + 2] = color[2];
                }
                e01 -= dy01;
                e12 -= dy12;
                e20 -= dy20;
            }
            e01_row += dx01;
            e12_row += dx12;
            e20_row += dx20;
        }
    }

    #[inline]
    fn draw_thick_line(
        p0: Vec2,
        p1: Vec2,
        thickness: f32,
        color: [u8; 3],
        width: usize,
        height: usize,
        buffer: &mut [u8],
    ) {
        let dir = p1 - p0;
        let len = dir.length();
        if len < 1e-3 {
            return;
        }
        let norm = Vec2::new(-dir.y, dir.x) / len * (thickness * 0.5);

        let v0 = p0 - norm;
        let v1 = p0 + norm;
        let v2 = p1 + norm;
        let v3 = p1 - norm;

        Self::draw_screen_triangle(v0, v1, v2, color, width, height, buffer);
        Self::draw_screen_triangle(v0, v2, v3, color, width, height, buffer);
    }

    #[inline]
    fn draw_circle(
        center: Vec2,
        radius: f32,
        color: [u8; 3],
        width: usize,
        height: usize,
        buffer: &mut [u8],
    ) {
        let min_x = (center.x - radius).floor().max(0.0) as usize;
        let max_x = (center.x + radius).ceil().min(width as f32 - 1.0) as usize;
        let min_y = (center.y - radius).floor().max(0.0) as usize;
        let max_y = (center.y + radius).ceil().min(height as f32 - 1.0) as usize;
        let r2 = radius * radius;

        for y in min_y..=max_y {
            let dy = y as f32 + 0.5 - center.y;
            let dy2 = dy * dy;
            let row_offset = y * width * 3;
            for x in min_x..=max_x {
                let dx = x as f32 + 0.5 - center.x;
                if dx * dx + dy2 <= r2 {
                    let idx = row_offset + x * 3;
                    buffer[idx] = color[0];
                    buffer[idx + 1] = color[1];
                    buffer[idx + 2] = color[2];
                }
            }
        }
    }

    #[inline]
    fn draw_point(p: Vec2, color: [u8; 3], width: usize, height: usize, buffer: &mut [u8]) {
        let x = p.x.round() as isize;
        let y = p.y.round() as isize;
        if x >= 0 && x < width as isize && y >= 0 && y < height as isize {
            let idx = (y as usize * width + x as usize) * 3;
            buffer[idx] = color[0];
            buffer[idx + 1] = color[1];
            buffer[idx + 2] = color[2];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tdrace_core::physics::config::CarConfig;
    use tdrace_core::track::{create_prototypical_track, RaceDirection, TrackShape};

    #[test]
    fn test_spec_118_fast_rasterizer_culling_and_scanline() {
        let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
        let rasterizer = FastRasterizer::new(&track);
        let spawn = track.grid_positions[0];
        let car0 = Car::new(CarConfig::sports_car()).with_pose(spawn.position, spawn.angle);
        let car1 = Car::new(CarConfig::sports_car()).with_pose(spawn.position + Vec2::new(6.0, 2.0), spawn.angle);
        let cars = vec![car0.clone(), car1.clone()];
        let opponents_only = vec![car1];

        for follow_car in [true, false] {
            let mut buf_all = vec![0u8; 96 * 96 * 3];
            let mut buf_opps = vec![0u8; 96 * 96 * 3];
            rasterizer.render(&cars[0], &cars, &track.geometry.obstacles, &[], 96, 96, follow_car, None, &mut buf_all);
            rasterizer.render(&car0, &opponents_only, &track.geometry.obstacles, &[], 96, 96, follow_car, None, &mut buf_opps);
            assert_eq!(buf_all, buf_opps, "Zero-copy &self.cars must render identically to filtered opponents slice");
            assert!(buf_all.chunks_exact(3).any(|px| px == [60, 60, 65]), "Asphalt road pixels must be rendered");
            assert!(buf_all.chunks_exact(3).any(|px| px == [230, 30, 30]), "Target car pixels must be rendered");
        }
    }
}
