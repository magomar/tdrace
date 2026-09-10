use macroquad::color::Color;
use macroquad::shapes::{draw_circle_lines, draw_rectangle};

/// Types of arcade screen transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionType {
    /// Smooth fade through solid color (default black).
    #[default]
    Fade,
    /// Expanding/contracting circular iris wipe.
    IrisWipe,
    /// Dual horizontal curtains closing/opening from edges to center.
    CurtainWipe,
    /// Alternating left/right interlaced scanline band wipe.
    ScanlineWipe,
}

/// Lifecycle phase of an active transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    /// Transition is covering the old screen.
    Covering,
    /// Transition is fully covering the screen (screen swap happens here).
    Holding,
    /// Transition is uncovering the new screen.
    Uncovering,
    /// Transition has concluded.
    Complete,
}

/// Configuration settings for a screen transition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionConfig {
    /// Visual style of transition.
    pub kind: TransitionType,
    /// Duration in seconds to cover the outgoing screen (default: 0.22s).
    pub duration_cover: f32,
    /// Duration in seconds to hold at full cover (default: 0.04s).
    pub duration_hold: f32,
    /// Duration in seconds to uncover the incoming screen (default: 0.22s).
    pub duration_uncover: f32,
    /// Matte color for the transition (default: solid black).
    pub color: Color,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            kind: TransitionType::Fade,
            duration_cover: 0.22,
            duration_hold: 0.04,
            duration_uncover: 0.22,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

/// Active arcade screen transition controller.
#[derive(Debug, Clone)]
pub struct ScreenTransition {
    pub config: TransitionConfig,
    pub phase: TransitionPhase,
    pub elapsed: f32,
}

impl Default for ScreenTransition {
    fn default() -> Self {
        Self::new(TransitionConfig::default())
    }
}

impl ScreenTransition {
    /// Creates a new transition from configuration.
    pub fn new(config: TransitionConfig) -> Self {
        Self {
            config,
            phase: TransitionPhase::Covering,
            elapsed: 0.0,
        }
    }

    /// Helper for a standard fade transition.
    pub fn fade(duration: f32) -> Self {
        let half = (duration * 0.48).max(0.02);
        let hold = (duration * 0.04).max(0.01);
        Self::new(TransitionConfig {
            kind: TransitionType::Fade,
            duration_cover: half,
            duration_hold: hold,
            duration_uncover: half,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        })
    }

    /// Helper for an iris circle wipe transition.
    pub fn iris(duration: f32) -> Self {
        let half = (duration * 0.48).max(0.02);
        let hold = (duration * 0.04).max(0.01);
        Self::new(TransitionConfig {
            kind: TransitionType::IrisWipe,
            duration_cover: half,
            duration_hold: hold,
            duration_uncover: half,
            color: Color::new(0.02, 0.02, 0.04, 1.0),
        })
    }

    /// Helper for a dual curtain wipe transition.
    pub fn curtain(duration: f32) -> Self {
        let half = (duration * 0.48).max(0.02);
        let hold = (duration * 0.04).max(0.01);
        Self::new(TransitionConfig {
            kind: TransitionType::CurtainWipe,
            duration_cover: half,
            duration_hold: hold,
            duration_uncover: half,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        })
    }

    /// Helper for an interlaced scanline band wipe transition.
    pub fn scanline(duration: f32) -> Self {
        let half = (duration * 0.48).max(0.02);
        let hold = (duration * 0.04).max(0.01);
        Self::new(TransitionConfig {
            kind: TransitionType::ScanlineWipe,
            duration_cover: half,
            duration_hold: hold,
            duration_uncover: half,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        })
    }

    /// Returns whether the transition is actively animating.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.phase != TransitionPhase::Complete
    }

    /// Returns whether the transition has reached the hold/swap point.
    #[inline]
    pub fn is_holding(&self) -> bool {
        self.phase == TransitionPhase::Holding
    }

    /// Returns whether the transition has concluded.
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.phase == TransitionPhase::Complete
    }

    /// Normalized coverage of the screen (0.0 = completely clear, 1.0 = completely covered).
    pub fn coverage(&self) -> f32 {
        match self.phase {
            TransitionPhase::Covering => {
                if self.config.duration_cover <= 0.0 {
                    1.0
                } else {
                    (self.elapsed / self.config.duration_cover).clamp(0.0, 1.0)
                }
            }
            TransitionPhase::Holding => 1.0,
            TransitionPhase::Uncovering => {
                if self.config.duration_uncover <= 0.0 {
                    0.0
                } else {
                    1.0 - (self.elapsed / self.config.duration_uncover).clamp(0.0, 1.0)
                }
            }
            TransitionPhase::Complete => 0.0,
        }
    }

    /// Steps transition animation by `dt`. Returns `true` on the frame it enters `Holding`
    /// (indicating screens should be swapped).
    pub fn update(&mut self, mut dt: f32) -> bool {
        if self.phase == TransitionPhase::Complete {
            return false;
        }

        let mut just_hit_hold = false;

        while dt > 0.0 && self.phase != TransitionPhase::Complete {
            match self.phase {
                TransitionPhase::Covering => {
                    let needed = (self.config.duration_cover - self.elapsed).max(0.0);
                    if dt >= needed {
                        dt -= needed;
                        self.phase = TransitionPhase::Holding;
                        self.elapsed = 0.0;
                        just_hit_hold = true;
                    } else {
                        self.elapsed += dt;
                        dt = 0.0;
                    }
                }
                TransitionPhase::Holding => {
                    let needed = (self.config.duration_hold - self.elapsed).max(0.0);
                    if dt >= needed {
                        dt -= needed;
                        self.phase = TransitionPhase::Uncovering;
                        self.elapsed = 0.0;
                    } else {
                        self.elapsed += dt;
                        dt = 0.0;
                    }
                }
                TransitionPhase::Uncovering => {
                    let needed = (self.config.duration_uncover - self.elapsed).max(0.0);
                    if dt >= needed {
                        dt -= needed;
                        self.phase = TransitionPhase::Complete;
                        self.elapsed = 0.0;
                    } else {
                        self.elapsed += dt;
                        dt = 0.0;
                    }
                }
                TransitionPhase::Complete => {
                    break;
                }
            }
        }

        just_hit_hold
    }

    /// Renders transition matte geometry over the given bounds.
    pub fn render(&self, x: f32, y: f32, width: f32, height: f32) {
        if !self.is_active() || width <= 0.0 || height <= 0.0 {
            return;
        }

        let cov = self.coverage();
        if cov <= 0.001 {
            return;
        }

        let base_col = self.config.color;

        match self.config.kind {
            TransitionType::Fade => {
                let col = Color::new(base_col.r, base_col.g, base_col.b, base_col.a * cov);
                draw_rectangle(x, y, width, height, col);
            }
            TransitionType::IrisWipe => {
                if cov >= 0.999 {
                    draw_rectangle(x, y, width, height, base_col);
                    return;
                }

                let cx = x + width * 0.5;
                let cy = y + height * 0.5;
                let max_r = (width * width + height * height).sqrt() * 0.5;
                let open_r = max_r * (1.0 - cov);

                // Thick stroke circle lines to block outside circle
                let thickness = (max_r - open_r).max(1.0);
                draw_circle_lines(cx, cy, open_r + thickness * 0.5, thickness, base_col);

                // Fill screen edges if necessary
                let top_edge = cy - open_r;
                let btm_edge = cy + open_r;
                let lft_edge = cx - open_r;
                let rgt_edge = cx + open_r;

                if top_edge > y {
                    draw_rectangle(x, y, width, top_edge - y, base_col);
                }
                if btm_edge < y + height {
                    draw_rectangle(x, btm_edge, width, y + height - btm_edge, base_col);
                }
                if lft_edge > x {
                    draw_rectangle(x, y, lft_edge - x, height, base_col);
                }
                if rgt_edge < x + width {
                    draw_rectangle(rgt_edge, y, x + width - rgt_edge, height, base_col);
                }
            }
            TransitionType::CurtainWipe => {
                let curtain_w = (width * 0.5) * cov;
                draw_rectangle(x, y, curtain_w, height, base_col);
                draw_rectangle(x + width - curtain_w, y, curtain_w, height, base_col);
            }
            TransitionType::ScanlineWipe => {
                let bands = 16;
                let band_h = height / bands as f32;
                for i in 0..bands {
                    let band_y = y + i as f32 * band_h;
                    // Staggered coverage: alternating bands move left-to-right or right-to-left
                    let stagger = (i as f32 / bands as f32) * 0.25;
                    let band_cov = ((cov - stagger) / 0.75).clamp(0.0, 1.0);
                    let w = width * band_cov;

                    if i % 2 == 0 {
                        // From left
                        draw_rectangle(x, band_y, w, band_h + 1.0, base_col);
                    } else {
                        // From right
                        draw_rectangle(x + width - w, band_y, w, band_h + 1.0, base_col);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_lifecycle_and_coverage() {
        let mut trans = ScreenTransition::new(TransitionConfig {
            kind: TransitionType::Fade,
            duration_cover: 0.10,
            duration_hold: 0.05,
            duration_uncover: 0.10,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        });

        assert_eq!(trans.phase, TransitionPhase::Covering);
        assert_eq!(trans.coverage(), 0.0);
        assert!(trans.is_active());

        // Step halfway through covering
        let hit = trans.update(0.05);
        assert!(!hit);
        assert!((trans.coverage() - 0.5).abs() < 1e-4);

        // Step to completion of covering -> hit holding
        let hit = trans.update(0.05);
        assert!(hit);
        assert_eq!(trans.phase, TransitionPhase::Holding);
        assert_eq!(trans.coverage(), 1.0);
        assert!(trans.is_holding());

        // Step through holding -> uncovering
        let hit = trans.update(0.05);
        assert!(!hit);
        assert_eq!(trans.phase, TransitionPhase::Uncovering);

        // Step halfway through uncovering
        let hit = trans.update(0.05);
        assert!(!hit);
        assert!((trans.coverage() - 0.5).abs() < 1e-4);

        // Step to completion
        let hit = trans.update(0.05);
        assert!(!hit);
        assert_eq!(trans.phase, TransitionPhase::Complete);
        assert_eq!(trans.coverage(), 0.0);
        assert!(!trans.is_active());
        assert!(trans.is_complete());
    }

    #[test]
    fn test_transition_presets() {
        let fade = ScreenTransition::fade(0.4);
        assert_eq!(fade.config.kind, TransitionType::Fade);

        let iris = ScreenTransition::iris(0.4);
        assert_eq!(iris.config.kind, TransitionType::IrisWipe);

        let curtain = ScreenTransition::curtain(0.4);
        assert_eq!(curtain.config.kind, TransitionType::CurtainWipe);

        let scanline = ScreenTransition::scanline(0.4);
        assert_eq!(scanline.config.kind, TransitionType::ScanlineWipe);
    }
}
