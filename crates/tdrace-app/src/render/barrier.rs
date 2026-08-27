use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_triangle};
use glam::Vec2;
use tdrace_core::track::geometry::{BarrierType, Obstacle, ObstacleShape, WallBarrier};
use tdrace_core::track::Track;

use super::color::Palette;
use super::track::draw_quad;

/// Global 2.5D visual light / shadow offset in meters.
pub const SHADOW_OFFSET: Vec2 = Vec2::new(0.35, 0.45);

/// Renders all boundary wall barriers and static obstacles with 2.5D drop shadows.
pub fn render_barriers_and_obstacles(track: &Track) {
    // Pass 1: Draw all 2.5D drop shadows
    for wall in &track.geometry.inner_walls {
        render_wall_shadow(wall);
    }
    for wall in &track.geometry.outer_walls {
        render_wall_shadow(wall);
    }
    for obs in &track.geometry.obstacles {
        render_obstacle_shadow(obs);
    }

    // Pass 2: Draw barrier and obstacle geometry
    for wall in &track.geometry.inner_walls {
        render_wall_body(wall);
    }
    for wall in &track.geometry.outer_walls {
        render_wall_body(wall);
    }
    for obs in &track.geometry.obstacles {
        render_obstacle_body(obs);
    }
}

/// Draws 2.5D shadow cast by a wall barrier.
fn render_wall_shadow(wall: &WallBarrier) {
    let p0 = wall.segment.start;
    let p1 = wall.segment.end;
    let s0 = p0 + SHADOW_OFFSET;
    let s1 = p1 + SHADOW_OFFSET;

    let thickness = match wall.barrier_type {
        BarrierType::Concrete => 0.70,
        BarrierType::Armco => 0.50,
        BarrierType::TireWall => 0.85,
        BarrierType::CurbWall => 0.40,
    };

    draw_line(s0.x, s0.y, s1.x, s1.y, thickness, Palette::SHADOW);
}

/// Draws the actual barrier geometry based on its barrier type.
fn render_wall_body(wall: &WallBarrier) {
    let p0 = wall.segment.start;
    let p1 = wall.segment.end;
    let dir = p1 - p0;
    let len = dir.length();
    if len < 0.05 {
        return;
    }
    let norm = wall.segment.normal();

    match wall.barrier_type {
        BarrierType::Armco => {
            // Metallic beam with support posts
            let half_w = 0.20;
            let a = p0 + norm * half_w;
            let b = p1 + norm * half_w;
            let c = p1 - norm * half_w;
            let d = p0 - norm * half_w;
            draw_quad(a, b, c, d, Palette::ARMCO_RAIL);

            // Center metallic groove
            draw_line(p0.x, p0.y, p1.x, p1.y, 0.08, Palette::ARMCO_POST);

            // Support posts every ~2.5m
            let post_count = (len / 2.5).max(1.0) as usize;
            for i in 0..=post_count {
                let t = i as f32 / post_count as f32;
                let post_pos = p0 + dir * t;
                draw_circle(post_pos.x, post_pos.y, 0.22, Palette::ARMCO_POST);
            }
        }
        BarrierType::Concrete => {
            // Thick solid concrete barrier with bevel highlight
            let half_w = 0.32;
            let a = p0 + norm * half_w;
            let b = p1 + norm * half_w;
            let c = p1 - norm * half_w;
            let d = p0 - norm * half_w;
            draw_quad(a, b, c, d, Palette::CONCRETE_WALL);

            // Top highlight bevel line
            let top0 = p0 + norm * (half_w * 0.4);
            let top1 = p1 + norm * (half_w * 0.4);
            draw_line(top0.x, top0.y, top1.x, top1.y, 0.15, Palette::CONCRETE_TOP);
        }
        BarrierType::TireWall => {
            // Stacked tire circles along the wall segment
            let tire_radius = 0.45;
            let tire_step = tire_radius * 1.75;
            let count = (len / tire_step).max(1.0) as usize;

            for i in 0..=count {
                let t = i as f32 / count as f32;
                let center = p0 + dir * t;
                draw_circle(center.x, center.y, tire_radius, Palette::TIRE_WALL);
                draw_circle_lines(center.x, center.y, tire_radius, 0.08, Color::new(0.08, 0.08, 0.10, 1.0));
                draw_circle(center.x, center.y, tire_radius * 0.45, Palette::TIRE_RIM);
            }
        }
        BarrierType::CurbWall => {
            let half_w = 0.15;
            let a = p0 + norm * half_w;
            let b = p1 + norm * half_w;
            let c = p1 - norm * half_w;
            let d = p0 - norm * half_w;
            draw_quad(a, b, c, d, Palette::CURB_RED);
        }
    }
}

/// Draws shadow for obstacles (tire stacks, bollards).
fn render_obstacle_shadow(obs: &Obstacle) {
    match &obs.shape {
        ObstacleShape::Circle { center, radius } => {
            let s_pos = *center + SHADOW_OFFSET;
            draw_circle(s_pos.x, s_pos.y, *radius, Palette::SHADOW);
        }
        ObstacleShape::Box { center, half_extents, angle } => {
            let fwd = Vec2::new(angle.cos(), angle.sin()) * half_extents.x;
            let right = Vec2::new(-angle.sin(), angle.cos()) * half_extents.y;
            let c = *center + SHADOW_OFFSET;
            draw_quad(
                c - fwd - right,
                c + fwd - right,
                c + fwd + right,
                c - fwd + right,
                Palette::SHADOW,
            );
        }
        ObstacleShape::Polygon { vertices } => {
            if vertices.len() >= 3 {
                let v0 = vertices[0] + SHADOW_OFFSET;
                for i in 1..vertices.len() - 1 {
                    let v1 = vertices[i] + SHADOW_OFFSET;
                    let v2 = vertices[i + 1] + SHADOW_OFFSET;
                    draw_triangle(
                        macroquad::prelude::Vec2::new(v0.x, v0.y),
                        macroquad::prelude::Vec2::new(v1.x, v1.y),
                        macroquad::prelude::Vec2::new(v2.x, v2.y),
                        Palette::SHADOW,
                    );
                }
            }
        }
    }
}

/// Draws obstacle geometry.
fn render_obstacle_body(obs: &Obstacle) {
    match &obs.shape {
        ObstacleShape::Circle { center, radius } => {
            // Apex tire bundle / bollard
            draw_circle(center.x, center.y, *radius, Palette::TIRE_WALL);
            draw_circle_lines(center.x, center.y, *radius, 0.12, Color::new(0.95, 0.85, 0.1, 1.0));
            draw_circle(center.x, center.y, *radius * 0.45, Color::new(0.95, 0.85, 0.1, 1.0));
        }
        ObstacleShape::Box { center, half_extents, angle } => {
            let fwd = Vec2::new(angle.cos(), angle.sin()) * half_extents.x;
            let right = Vec2::new(-angle.sin(), angle.cos()) * half_extents.y;
            draw_quad(
                *center - fwd - right,
                *center + fwd - right,
                *center + fwd + right,
                *center - fwd + right,
                Palette::CONCRETE_WALL,
            );
        }
        ObstacleShape::Polygon { vertices } => {
            if vertices.len() >= 3 {
                let v0 = vertices[0];
                for i in 1..vertices.len() - 1 {
                    let v1 = vertices[i];
                    let v2 = vertices[i + 1];
                    draw_triangle(
                        macroquad::prelude::Vec2::new(v0.x, v0.y),
                        macroquad::prelude::Vec2::new(v1.x, v1.y),
                        macroquad::prelude::Vec2::new(v2.x, v2.y),
                        Palette::CONCRETE_WALL,
                    );
                }
                for i in 0..vertices.len() {
                    let v1 = vertices[i];
                    let v2 = vertices[(i + 1) % vertices.len()];
                    draw_line(v1.x, v1.y, v2.x, v2.y, 0.25, Color::new(0.95, 0.85, 0.1, 1.0));
                }
            }
        }
    }
}
