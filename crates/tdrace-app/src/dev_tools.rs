use tdrace_core::track::validation::{validate_track, TrackValidationError, ValidationSeverity};
use tdrace_core::track::Track;


/// Serializes a track to formatted JSON string suitable for preset files or sharing.
pub fn export_track_to_json(track: &Track) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(track)
}

/// Validates all official circuits in the embedded catalog, returning any errors or warnings.
pub fn validate_all_official_presets() -> Vec<(String, Vec<TrackValidationError>)> {
    let mut results = Vec::new();
    for circuit in tdrace_core::catalog::circuits() {
        let track = match circuit.load() {
            Ok(track) => track,
            Err(e) => {
                results.push((
                    format!("{}:{}", circuit.module, circuit.id),
                    vec![TrackValidationError {
                        severity: ValidationSeverity::Error,
                        code: "LOAD_FAILED",
                        message: e.to_string(),
                        details: None,
                        entity_index: None,
                    }],
                ));
                continue;
            }
        };
        let errors_or_warnings: Vec<_> = validate_track(&track)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error || d.severity == ValidationSeverity::Warning)
            .collect();
        if !errors_or_warnings.is_empty() {
            results.push((format!("{}:{}", circuit.module, circuit.id), errors_or_warnings));
        }
    }

    results
}

/// Per-frame subsystem timing breakdown in milliseconds.
#[derive(Debug, Clone, Copy, Default)]
pub struct FrameTimings {
    pub frame_ms: f64,
    pub update_ms: f64,
    pub substeps: u32,
    pub track_ms: f64,
    pub skidmarks_ms: f64,
    pub scenery_ms: f64,
    pub cars_ms: f64,
    pub particles_ms: f64,
    pub hud_ms: f64,
    pub total_render_ms: f64,
}

impl FrameTimings {
    pub fn clear_render(&mut self) {
        self.track_ms = 0.0;
        self.skidmarks_ms = 0.0;
        self.scenery_ms = 0.0;
        self.cars_ms = 0.0;
        self.particles_ms = 0.0;
        self.hud_ms = 0.0;
        self.total_render_ms = 0.0;
    }
}

/// 1-second rolling window statistics displayed on the profiler HUD and logged to stdout.
#[derive(Debug, Clone, Copy, Default)]
pub struct RollingProfileStats {
    pub fps: f64,
    pub avg_frame_ms: f64,
    pub max_frame_ms: f64,
    pub avg_update_ms: f64,
    pub max_update_ms: f64,
    pub avg_substeps: f64,
    pub max_substeps: u32,
    pub avg_render_ms: f64,
    pub avg_track_ms: f64,
    pub avg_skidmarks_ms: f64,
    pub avg_scenery_ms: f64,
    pub avg_cars_ms: f64,
    pub avg_particles_ms: f64,
    pub avg_hud_ms: f64,
    pub skidmark_segments: usize,
    pub skidmark_chunks: usize,
    pub skidmark_cap: usize,
    pub active_particles: usize,
    pub topdown_tex_cache: usize,
    pub lateral_tex_cache: usize,
    pub rss_mb: f64,
}

/// Real-time frame & subsystem profiler activated via `--profile`, `TDRACE_PROFILE=1`, or `F7`.
#[derive(Debug, Clone)]
pub struct RuntimeProfiler {
    pub enabled: bool,
    pub current: FrameTimings,
    pub stats: RollingProfileStats,
    last_frame_instant: Option<std::time::Instant>,
    window_start: std::time::Instant,
    window_frames: u32,
    sum_frame_ms: f64,
    max_frame_ms: f64,
    sum_update_ms: f64,
    max_update_ms: f64,
    sum_substeps: u32,
    max_substeps: u32,
    sum_render_ms: f64,
    sum_track_ms: f64,
    sum_skidmarks_ms: f64,
    sum_scenery_ms: f64,
    sum_cars_ms: f64,
    sum_particles_ms: f64,
    sum_hud_ms: f64,
}

impl Default for RuntimeProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeProfiler {
    pub fn new() -> Self {
        let enabled = std::env::var("TDRACE_PROFILE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        Self {
            enabled,
            current: FrameTimings::default(),
            stats: RollingProfileStats {
                rss_mb: read_process_rss_mb(),
                ..Default::default()
            },
            last_frame_instant: None,
            window_start: std::time::Instant::now(),
            window_frames: 0,
            sum_frame_ms: 0.0,
            max_frame_ms: 0.0,
            sum_update_ms: 0.0,
            max_update_ms: 0.0,
            sum_substeps: 0,
            max_substeps: 0,
            sum_render_ms: 0.0,
            sum_track_ms: 0.0,
            sum_skidmarks_ms: 0.0,
            sum_scenery_ms: 0.0,
            sum_cars_ms: 0.0,
            sum_particles_ms: 0.0,
            sum_hud_ms: 0.0,
        }
    }

    /// Marks the start of a new frame and records wall-clock delta since the previous frame.
    pub fn begin_frame(&mut self) {
        let now = std::time::Instant::now();
        if let Some(prev) = self.last_frame_instant {
            self.current.frame_ms = now.duration_since(prev).as_secs_f64() * 1000.0;
        }
        self.last_frame_instant = Some(now);
        self.current.update_ms = 0.0;
        self.current.substeps = 0;
        self.current.clear_render();
    }

    /// Finishes the current frame, updates the 1-second rolling window, and logs a `[PROFILE]` line every second.
    pub fn end_frame(
        &mut self,
        session_time: f32,
        skidmark_segments: usize,
        skidmark_chunks: usize,
        skidmark_cap: usize,
        active_particles: usize,
        tex_cache: (usize, usize),
    ) {
        if !self.enabled {
            return;
        }

        self.window_frames += 1;
        let f_ms = self.current.frame_ms;
        self.sum_frame_ms += f_ms;
        if f_ms > self.max_frame_ms {
            self.max_frame_ms = f_ms;
        }
        self.sum_update_ms += self.current.update_ms;
        if self.current.update_ms > self.max_update_ms {
            self.max_update_ms = self.current.update_ms;
        }
        self.sum_substeps += self.current.substeps;
        if self.current.substeps > self.max_substeps {
            self.max_substeps = self.current.substeps;
        }
        self.sum_render_ms += self.current.total_render_ms;
        self.sum_track_ms += self.current.track_ms;
        self.sum_skidmarks_ms += self.current.skidmarks_ms;
        self.sum_scenery_ms += self.current.scenery_ms;
        self.sum_cars_ms += self.current.cars_ms;
        self.sum_particles_ms += self.current.particles_ms;
        self.sum_hud_ms += self.current.hud_ms;

        let elapsed = self.window_start.elapsed().as_secs_f64();
        if elapsed >= 1.0 && self.window_frames > 0 {
            let n = self.window_frames as f64;
            let rss_mb = read_process_rss_mb();
            self.stats = RollingProfileStats {
                fps: n / elapsed,
                avg_frame_ms: self.sum_frame_ms / n,
                max_frame_ms: self.max_frame_ms,
                avg_update_ms: self.sum_update_ms / n,
                max_update_ms: self.max_update_ms,
                avg_substeps: self.sum_substeps as f64 / n,
                max_substeps: self.max_substeps,
                avg_render_ms: self.sum_render_ms / n,
                avg_track_ms: self.sum_track_ms / n,
                avg_skidmarks_ms: self.sum_skidmarks_ms / n,
                avg_scenery_ms: self.sum_scenery_ms / n,
                avg_cars_ms: self.sum_cars_ms / n,
                avg_particles_ms: self.sum_particles_ms / n,
                avg_hud_ms: self.sum_hud_ms / n,
                skidmark_segments,
                skidmark_chunks,
                skidmark_cap,
                active_particles,
                topdown_tex_cache: tex_cache.0,
                lateral_tex_cache: tex_cache.1,
                rss_mb,
            };

            let build_mode = if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" };
            println!(
                "[PROFILE {build_mode}] t={session_time:6.1}s | FPS={:5.1} | frame={:5.2}ms (max {:5.2}ms) | upd={:5.2}ms (steps avg {:.1}/max {}) | render={:5.2}ms [track={:4.2}ms skid={:5.2}ms scen={:4.2}ms cars={:4.2}ms part={:4.2}ms hud={:4.2}ms] | skids={}/{}K ({} chunks) parts={} tex={}+{} | RSS={:.1}MB",
                self.stats.fps,
                self.stats.avg_frame_ms,
                self.stats.max_frame_ms,
                self.stats.avg_update_ms,
                self.stats.avg_substeps,
                self.stats.max_substeps,
                self.stats.avg_render_ms,
                self.stats.avg_track_ms,
                self.stats.avg_skidmarks_ms,
                self.stats.avg_scenery_ms,
                self.stats.avg_cars_ms,
                self.stats.avg_particles_ms,
                self.stats.avg_hud_ms,
                self.stats.skidmark_segments,
                self.stats.skidmark_cap / 1024,
                self.stats.skidmark_chunks,
                self.stats.active_particles,
                self.stats.topdown_tex_cache,
                self.stats.lateral_tex_cache,
                self.stats.rss_mb,
            );

            self.window_start = std::time::Instant::now();
            self.window_frames = 0;
            self.sum_frame_ms = 0.0;
            self.max_frame_ms = 0.0;
            self.sum_update_ms = 0.0;
            self.max_update_ms = 0.0;
            self.sum_substeps = 0;
            self.max_substeps = 0;
            self.sum_render_ms = 0.0;
            self.sum_track_ms = 0.0;
            self.sum_skidmarks_ms = 0.0;
            self.sum_scenery_ms = 0.0;
            self.sum_cars_ms = 0.0;
            self.sum_particles_ms = 0.0;
            self.sum_hud_ms = 0.0;
        } else {
            self.stats.skidmark_segments = skidmark_segments;
            self.stats.skidmark_chunks = skidmark_chunks;
            self.stats.skidmark_cap = skidmark_cap;
            self.stats.active_particles = active_particles;
            self.stats.topdown_tex_cache = tex_cache.0;
            self.stats.lateral_tex_cache = tex_cache.1;
        }
    }

    /// Renders the live performance profiler HUD overlay on screen (toggled via `--profile` or `F7`).
    pub fn render_overlay(&self, fonts: &crate::ui::font::Fonts, session_time: f32) {
        if !self.enabled {
            return;
        }

        use macroquad::color::Color;
        use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
        use crate::render::color::Palette;

        let s = &self.stats;
        let x = 16.0;
        let y = 76.0;
        let w = 384.0;
        let line_h = 16.5;
        let num_lines = 11usize;
        let h = 28.0 + num_lines as f32 * line_h + 10.0;

        draw_rectangle(x, y, w, h, Color::new(0.03, 0.04, 0.07, 0.90));
        let border_col = if s.avg_frame_ms > 18.0 || s.max_substeps > 2 {
            Palette::RED
        } else if s.avg_frame_ms > 14.0 {
            Palette::YELLOW
        } else {
            Palette::NEON_CYAN
        };
        draw_rectangle_lines(x, y, w, h, 1.5, border_col);

        let build_tag = if cfg!(debug_assertions) {
            "DEBUG (opt=0)"
        } else {
            "RELEASE (opt=3)"
        };
        let header = format!("=== PERFORMANCE PROFILER [F7] ({build_tag}) ===");
        fonts.draw_ui_bold(&header, x + 10.0, y + 18.0, 13.0, border_col);

        let fps_col = if s.fps < 50.0 && s.fps > 0.0 {
            Palette::RED
        } else if s.fps < 58.0 && s.fps > 0.0 {
            Palette::YELLOW
        } else {
            Palette::NEON_GREEN
        };

        let mut ry = y + 36.0;
        let l1 = format!(
            "FPS: {:5.1}  |  Frame: {:5.2} ms (max {:5.2} ms)  t={:.1}s",
            s.fps, s.avg_frame_ms, s.max_frame_ms, session_time
        );
        fonts.draw_ui_bold(&l1, x + 10.0, ry, 13.0, fps_col);
        ry += line_h;

        let step_col = if s.max_substeps > 2 {
            Palette::RED
        } else {
            Color::new(0.88, 0.92, 0.96, 1.0)
        };
        let l2 = format!(
            "Update (120Hz): {:5.2} ms (max {:5.2} ms) | Substeps: {:.1} (max {})",
            s.avg_update_ms, s.max_update_ms, s.avg_substeps, s.max_substeps
        );
        fonts.draw_ui_bold(&l2, x + 10.0, ry, 12.5, step_col);
        ry += line_h;

        let rows = [
            (
                format!("Render Total:   {:5.2} ms (budget 16.67 ms)", s.avg_render_ms),
                Color::new(0.85, 0.90, 0.98, 1.0),
            ),
            (
                format!("  |- Track:     {:5.2} ms (spline & curbs)", s.avg_track_ms),
                Color::new(0.78, 0.84, 0.92, 1.0),
            ),
            (
                format!(
                    "  |- Skidmarks: {:5.2} ms ({}/{}K segs, {} chunks)",
                    s.avg_skidmarks_ms,
                    s.skidmark_segments,
                    s.skidmark_cap / 1024,
                    s.skidmark_chunks
                ),
                if s.avg_skidmarks_ms > 4.0 {
                    Palette::NEON_ORANGE
                } else {
                    Color::new(0.78, 0.84, 0.92, 1.0)
                },
            ),
            (
                format!("  |- Scenery:   {:5.2} ms (trees, walls, stands)", s.avg_scenery_ms),
                Color::new(0.78, 0.84, 0.92, 1.0),
            ),
            (
                format!("  |- Vehicles:  {:5.2} ms (cars, shadows, aids)", s.avg_cars_ms),
                Color::new(0.78, 0.84, 0.92, 1.0),
            ),
            (
                format!(
                    "  |- Particles: {:5.2} ms ({} active)",
                    s.avg_particles_ms, s.active_particles
                ),
                Color::new(0.78, 0.84, 0.92, 1.0),
            ),
            (
                format!("  +- HUD / UI:  {:5.2} ms", s.avg_hud_ms),
                Color::new(0.78, 0.84, 0.92, 1.0),
            ),
            (
                format!(
                    "Tex Cache: {} topdown + {} lateral  |  RSS: {:.1} MB",
                    s.topdown_tex_cache, s.lateral_tex_cache, s.rss_mb
                ),
                Palette::NEON_CYAN,
            ),
        ];

        for (text, col) in &rows {
            fonts.draw_ui_bold(text, x + 10.0, ry, 12.5, *col);
            ry += line_h;
        }
    }
}

/// Reads current process Resident Set Size (RSS) in MiB from `/proc/self/statm` on Linux.
pub fn read_process_rss_mb() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
            if let Some(res_pages) = statm.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok()) {
                return (res_pages as f64 * 4096.0) / (1024.0 * 1024.0);
            }
        }
        0.0
    }
    #[cfg(not(target_os = "linux"))]
    {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dev_tools_export_to_json() {
        let gp = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let json = export_track_to_json(&gp).expect("Must serialize track to JSON");
        assert!(json.contains("Coastal Grand Prix"));
    }

    #[test]
    fn test_runtime_profiler_rolling_stats() {
        let mut profiler = RuntimeProfiler::new();
        profiler.enabled = true;
        profiler.window_start = std::time::Instant::now() - std::time::Duration::from_millis(1100);
        profiler.begin_frame();
        profiler.current.update_ms = 2.4;
        profiler.current.substeps = 2;
        profiler.current.track_ms = 1.5;
        profiler.current.skidmarks_ms = 0.8;
        profiler.current.scenery_ms = 0.6;
        profiler.current.cars_ms = 0.5;
        profiler.current.particles_ms = 0.3;
        profiler.current.hud_ms = 0.4;
        profiler.current.total_render_ms = 4.1;
        profiler.end_frame(1.0, 1400, 6, 262_144, 40, (12, 8));

        let stats = &profiler.stats;
        assert!((stats.avg_update_ms - 2.4).abs() < 1e-5);
        assert!((stats.avg_skidmarks_ms - 0.8).abs() < 1e-5);
        assert_eq!(stats.skidmark_segments, 1400);
        assert_eq!(stats.skidmark_chunks, 6);
        assert_eq!(stats.skidmark_cap, 262_144);
        assert_eq!(stats.active_particles, 40);
        assert_eq!(stats.topdown_tex_cache, 12);
        assert_eq!(stats.lateral_tex_cache, 8);
        #[cfg(target_os = "linux")]
        assert!(stats.rss_mb > 0.0);
    }
}
