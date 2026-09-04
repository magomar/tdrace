//! Space Arena Prototype - Standalone 2D Arcade Game powered by Cabinet.
//! Demonstrates multi-game UI/UX consistency, 2D navigation, Juice FX, and modal screen stack.

use cabinet::audio::AudioMixer;
use cabinet::fx::{HitStop, ScreenFlash, ScreenShake};
use cabinet::input::{DigitalInputFilter, GamepadConfig, GamepadManager};
use cabinet::profile::ProfileManager;
use cabinet::records::{RecordDatabase, RecordMetric};
use cabinet::state::{
    ArcadeSettingsModal, CabinetContext, CabinetScreen, ScreenAction, ScreenStack,
    UniversalPauseModal,
};
use cabinet::ui::{draw_stat_bar, CabinetTheme, Fonts, Palette, UiScaler};
use glam::Vec2;
use macroquad::color::Color;
use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
use macroquad::math::vec2;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_triangle};
use macroquad::window::{clear_background, next_frame, screen_height, screen_width, Conf};

/// Laser projectile fired by ships.
#[derive(Debug, Clone)]
pub struct Laser {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub color: Color,
}

/// Floating space debris / asteroid target.
#[derive(Debug, Clone)]
pub struct Asteroid {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
    pub health: f32,
    pub max_health: f32,
}

/// Active space arena simulation state.
pub struct SpaceArenaGame {
    pub ship_pos: Vec2,
    pub ship_vel: Vec2,
    pub ship_heading: f32,
    pub ship_shields: f32,
    pub max_shields: f32,
    pub score: u32,
    pub wave: u32,
    pub lasers: Vec<Laser>,
    pub asteroids: Vec<Asteroid>,
    pub shake: ScreenShake,
    pub flash: ScreenFlash,
    pub hitstop: HitStop,
    pub filter: DigitalInputFilter,
    pub profile_manager: ProfileManager,
    pub record_db: RecordDatabase,
    pub audio: AudioMixer,
    pub fire_cooldown: f32,
}

impl Default for SpaceArenaGame {
    fn default() -> Self {
        Self::new()
    }
}

impl SpaceArenaGame {
    pub fn new() -> Self {
        let mut record_db = RecordDatabase::new();
        record_db.get_or_create("space_arena_highscores", RecordMetric::HighestScore, 10);

        let mut game = Self {
            ship_pos: Vec2::new(640.0, 360.0),
            ship_vel: Vec2::ZERO,
            ship_heading: 0.0,
            ship_shields: 100.0,
            max_shields: 100.0,
            score: 0,
            wave: 1,
            lasers: Vec::new(),
            asteroids: Vec::new(),
            shake: ScreenShake::new(18.0, 2.0),
            flash: ScreenFlash::new(),
            hitstop: HitStop::new(),
            filter: DigitalInputFilter::default(),
            profile_manager: ProfileManager::new(),
            record_db,
            audio: AudioMixer::new(),
            fire_cooldown: 0.0,
        };
        game.spawn_wave(1);
        game
    }

    pub fn spawn_wave(&mut self, wave: u32) {
        self.wave = wave;
        self.asteroids.clear();
        let count = 4 + wave * 2;
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let dist = 280.0 + (i % 3) as f32 * 60.0;
            let pos = Vec2::new(640.0 + angle.cos() * dist, 360.0 + angle.sin() * dist);
            let vel = Vec2::new(-angle.sin() * 45.0, angle.cos() * 45.0);
            self.asteroids.push(Asteroid {
                pos,
                vel,
                radius: 24.0,
                health: 30.0,
                max_health: 30.0,
            });
        }
    }

    pub fn fire_laser(&mut self) {
        if self.fire_cooldown <= 0.0 {
            let dir = Vec2::new(self.ship_heading.cos(), self.ship_heading.sin());
            let tip = self.ship_pos + dir * 20.0;
            self.lasers.push(Laser {
                pos: tip,
                vel: dir * 550.0 + self.ship_vel * 0.3,
                life: 1.2,
                color: Palette::NEON_CYAN,
            });
            self.fire_cooldown = 0.18;
            self.shake.add_trauma(0.06);
        }
    }

    pub fn step_physics(&mut self, dt: f32) {
        // Update lasers
        for laser in &mut self.lasers {
            laser.pos += laser.vel * dt;
            laser.life -= dt;
        }
        self.lasers.retain(|l| l.life > 0.0);

        // Update asteroids
        for ast in &mut self.asteroids {
            ast.pos += ast.vel * dt;
            // Screen wrap
            if ast.pos.x < 0.0 { ast.pos.x += 1280.0; }
            if ast.pos.x > 1280.0 { ast.pos.x -= 1280.0; }
            if ast.pos.y < 0.0 { ast.pos.y += 720.0; }
            if ast.pos.y > 720.0 { ast.pos.y -= 720.0; }
        }

        // Collision: Laser vs Asteroid
        let mut hit_indices = Vec::new();
        for (l_idx, laser) in self.lasers.iter().enumerate() {
            for ast in self.asteroids.iter_mut() {
                if laser.pos.distance(ast.pos) <= ast.radius {
                    ast.health -= 15.0;
                    hit_indices.push(l_idx);
                    if ast.health <= 0.0 {
                        self.score += 100;
                        self.shake.add_trauma(0.25);
                        self.flash.trigger(Color::new(1.0, 0.85, 0.20, 0.45), 0.08);
                        self.hitstop.freeze(0.04);
                    }
                    break;
                }
            }
        }
        for idx in hit_indices.into_iter().rev() {
            if idx < self.lasers.len() {
                self.lasers.remove(idx);
            }
        }
        self.asteroids.retain(|a| a.health > 0.0);

        if self.asteroids.is_empty() {
            self.score += 500;
            self.flash.trigger(Palette::NEON_GREEN, 0.20);
            self.spawn_wave(self.wave + 1);
        }

        // Ship movement
        self.ship_pos += self.ship_vel * dt;
        self.ship_vel *= 0.985; // Space drag
        if self.ship_pos.x < 0.0 { self.ship_pos.x += 1280.0; }
        if self.ship_pos.x > 1280.0 { self.ship_pos.x -= 1280.0; }
        if self.ship_pos.y < 0.0 { self.ship_pos.y += 720.0; }
        if self.ship_pos.y > 720.0 { self.ship_pos.y -= 720.0; }
    }
}

impl CabinetScreen for SpaceArenaGame {
    fn name(&self) -> &str {
        "SpaceArenaGame"
    }

    fn update(&mut self, ctx: &mut CabinetContext) -> ScreenAction {
        // Pause trigger
        if is_key_pressed(KeyCode::Escape) || ctx.gamepad.btn_start_pressed {
            return ScreenAction::Push(Box::new(UniversalPauseModal::new("SPACE ARENA PAUSED")));
        }

        // Settings modal trigger
        if is_key_pressed(KeyCode::O) || ctx.gamepad.btn_back_pressed {
            return ScreenAction::Push(Box::new(ArcadeSettingsModal::new(
                &self.audio.settings,
                &GamepadConfig::default(),
            )));
        }

        let effective_dt = self.hitstop.step(ctx.dt);
        self.shake.update(ctx.dt);
        self.flash.update(ctx.dt);

        if self.fire_cooldown > 0.0 {
            self.fire_cooldown -= ctx.dt;
        }

        // Turn inputs
        let mut steer_input = 0.0;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            steer_input -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            steer_input += 1.0;
        }
        if ctx.gamepad.steer.abs() > 0.15 {
            steer_input = ctx.gamepad.steer;
        }

        // Thrust inputs
        let mut thrust_input = 0.0;
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            thrust_input = 1.0;
        }
        if ctx.gamepad.throttle > 0.1 {
            thrust_input = ctx.gamepad.throttle;
        }

        let (filtered_steer, filtered_thrust, _) =
            self.filter.update(steer_input, thrust_input, 0.0, 0.0, ctx.dt);

        self.ship_heading += filtered_steer * 4.2 * ctx.dt;
        let thrust_dir = Vec2::new(self.ship_heading.cos(), self.ship_heading.sin());
        self.ship_vel += thrust_dir * filtered_thrust * 380.0 * ctx.dt;

        // Shoot inputs
        if is_key_down(KeyCode::Space) || ctx.gamepad.btn_a_pressed {
            self.fire_laser();
        }

        self.step_physics(effective_dt);
        ScreenAction::None
    }

    fn draw(&self, ctx: &CabinetContext) {
        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let scaler = ctx.scaler;
        let fonts = ctx.fonts;

        // Space arena deep void background
        clear_background(Color::new(0.03, 0.04, 0.07, 1.0));

        let (shake_offset, _) = self.shake.sample_shake();
        let ox = shake_offset.x * scaler.scale;
        let oy = shake_offset.y * scaler.scale;

        let scale_x = sw / 1280.0;
        let scale_y = sh / 720.0;

        // Draw asteroids
        for ast in &self.asteroids {
            let ax = ast.pos.x * scale_x + ox;
            let ay = ast.pos.y * scale_y + oy;
            let ar = ast.radius * scaler.scale;
            draw_circle(ax, ay, ar, Color::new(0.35, 0.40, 0.48, 0.90));
            draw_circle_lines(ax, ay, ar, 1.6 * scaler.scale, Palette::UI_CARD_BORDER_GLOW);
        }

        // Draw lasers
        for laser in &self.lasers {
            let lx = laser.pos.x * scale_x + ox;
            let ly = laser.pos.y * scale_y + oy;
            draw_circle(lx, ly, 3.5 * scaler.scale, laser.color);
        }

        // Draw Player Spaceship
        let sx = self.ship_pos.x * scale_x + ox;
        let sy = self.ship_pos.y * scale_y + oy;
        let dir = Vec2::new(self.ship_heading.cos(), self.ship_heading.sin());
        let normal = Vec2::new(-dir.y, dir.x);

        let p1 = Vec2::new(sx, sy) + dir * (18.0 * scaler.scale);
        let p2 = Vec2::new(sx, sy) - dir * (12.0 * scaler.scale) + normal * (10.0 * scaler.scale);
        let p3 = Vec2::new(sx, sy) - dir * (12.0 * scaler.scale) - normal * (10.0 * scaler.scale);

        let active_profile = self.profile_manager.active_profile();
        let ship_color = active_profile.color_scheme.primary;
        let accent_color = active_profile.color_scheme.accent;

        draw_triangle(vec2(p1.x, p1.y), vec2(p2.x, p2.y), vec2(p3.x, p3.y), ship_color);
        draw_line(p1.x, p1.y, p2.x, p2.y, 2.0 * scaler.scale, accent_color);
        draw_line(p2.x, p2.y, p3.x, p3.y, 2.0 * scaler.scale, accent_color);
        draw_line(p3.x, p3.y, p1.x, p1.y, 2.0 * scaler.scale, accent_color);

        // Flash overlay
        self.flash.draw(sw, sh);

        // HUD Overlay using Cabinet UI Scaler and Widgets
        scaler.draw_glass_card(
            scaler.s(20.0),
            scaler.s(20.0),
            scaler.s(320.0),
            scaler.s(90.0),
            Palette::UI_CARD_BG,
            Palette::NEON_CYAN,
            1.5,
        );

        fonts.draw_display(
            &format!("PILOT: {}", active_profile.name),
            scaler.s(32.0),
            scaler.s(45.0),
            scaler.font_s(16.0),
            Palette::WHITE,
        );

        draw_stat_bar(
            scaler,
            fonts,
            scaler.s(32.0),
            scaler.s(58.0),
            scaler.s(290.0),
            "SHIELDS",
            self.ship_shields / self.max_shields,
            Palette::NEON_CYAN,
        );

        fonts.draw_display(
            &format!("SCORE: {:06}", self.score),
            scaler.s(32.0),
            scaler.s(96.0),
            scaler.font_s(15.0),
            Palette::NEON_GOLD,
        );

        fonts.draw_display_centered(
            &format!("WAVE {}", self.wave),
            sw * 0.5,
            scaler.s(48.0),
            scaler.font_s(26.0),
            Palette::NEON_CYAN,
        );

        fonts.draw_ui_bold_centered(
            "[ESC] PAUSE | [O] SETTINGS",
            sw * 0.5,
            scaler.s(72.0),
            scaler.font_s(11.5),
            Palette::UI_TEXT_MUTED,
        );
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Space Arena - Powered by Cabinet".to_string(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let root_game = Box::new(SpaceArenaGame::new());
    let mut stack = ScreenStack::new(root_game);
    let mut gamepad = GamepadManager::new();
    let theme = CabinetTheme::cyberpunk_neon();
    let fonts = Fonts::load_embedded();

    loop {
        let sw = screen_width();
        let sh = screen_height();
        let scaler = UiScaler::new(sw, sh);
        let dt = macroquad::time::get_frame_time().min(0.1);

        gamepad.update();

        let mut ctx = CabinetContext {
            scaler: &scaler,
            fonts: &fonts,
            theme: &theme,
            gamepad: &gamepad.snapshot,
            dt,
        };

        if let Some(action) = stack.update(&mut ctx) {
            if matches!(action, ScreenAction::Quit) {
                break;
            }
        }

        stack.draw(&ctx);
        next_frame().await;
    }
}
