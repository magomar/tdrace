use macroquad::color::Color;
use serde::{Deserialize, Serialize};

use crate::audio::sfx::SoundCue;
use crate::audio::CabinetAudioSink;
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Event emitted during the countdown sequence lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountDownEvent {
    /// Countdown step ticked (e.g. 3, 2, 1).
    Tick(u8),
    /// Reached GO phase.
    Go,
    /// Sequence fully concluded.
    Finished,
}

/// Race start countdown sequence (e.g. 3 → 2 → 1 → GO) with animated scale/fade and audio cues.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountDown {
    pub remaining: u8,
    pub total: u8,
    pub scale: f32,
    pub alpha: f32,
    pub elapsed_in_step: f32,
    pub step_duration: f32,
    pub is_finished: bool,
}

impl CountDown {
    /// Creates a new countdown starting at `total` seconds (typically 3).
    pub fn new(total: u8) -> Self {
        Self {
            remaining: total,
            total,
            scale: 1.6,
            alpha: 1.0,
            elapsed_in_step: 0.0,
            step_duration: 1.0,
            is_finished: false,
        }
    }

    /// Display string for the current countdown step.
    pub fn text(&self) -> &'static str {
        match self.remaining {
            3 => "3",
            2 => "2",
            1 => "1",
            0 => "GO!",
            _ => "",
        }
    }

    /// Color for the current step.
    pub fn step_color(&self) -> Color {
        match self.remaining {
            3 => Palette::RED,
            2 => Palette::NEON_ORANGE,
            1 => Palette::NEON_GOLD,
            0 => Palette::NEON_GREEN,
            _ => Palette::WHITE,
        }
    }

    /// Advances the countdown animation by `dt` seconds, returning an event on transition.
    pub fn update(&mut self, dt: f32) -> Option<CountDownEvent> {
        if self.is_finished {
            return None;
        }

        self.elapsed_in_step += dt;

        // Animate scale from 1.6 down towards 1.0 within the step
        let progress = (self.elapsed_in_step / self.step_duration).clamp(0.0, 1.0);
        self.scale = 1.6 - 0.6 * progress;
        self.alpha = 1.0 - 0.25 * progress;

        if self.elapsed_in_step >= self.step_duration {
            self.elapsed_in_step = 0.0;
            self.scale = 1.6;
            self.alpha = 1.0;

            if self.remaining > 0 {
                self.remaining -= 1;
                if self.remaining == 0 {
                    Some(CountDownEvent::Go)
                } else {
                    Some(CountDownEvent::Tick(self.remaining))
                }
            } else {
                self.is_finished = true;
                Some(CountDownEvent::Finished)
            }
        } else {
            None
        }
    }

    /// Updates countdown and plays procedural audio cues through the audio sink.
    pub fn update_with_audio(
        &mut self,
        dt: f32,
        audio: &impl CabinetAudioSink,
    ) -> Option<CountDownEvent> {
        let event = self.update(dt);
        match event {
            Some(CountDownEvent::Tick(_)) => {
                audio.play_cue(SoundCue::CountdownLow);
            }
            Some(CountDownEvent::Go) => {
                audio.play_cue(SoundCue::CountdownHigh);
            }
            _ => {}
        }
        event
    }

    /// Renders the countdown text at the specified center position with animated scale and alpha.
    pub fn draw(&self, scaler: &UiScaler, fonts: &Fonts, center_x: f32, center_y: f32) {
        if self.is_finished {
            return;
        }
        let txt = self.text();
        if txt.is_empty() {
            return;
        }

        let base_font_size = if self.remaining == 0 { 64.0 } else { 72.0 };
        let font_size = scaler.font_s(base_font_size * self.scale);
        let mut color = self.step_color();
        color.a *= self.alpha;

        fonts.draw_display_centered(txt, center_x, center_y, font_size, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSink {
        cues: std::cell::RefCell<Vec<SoundCue>>,
    }

    impl MockSink {
        fn new() -> Self {
            Self {
                cues: std::cell::RefCell::new(Vec::new()),
            }
        }
    }

    impl CabinetAudioSink for MockSink {
        fn play_cue(&self, cue: SoundCue) {
            self.cues.borrow_mut().push(cue);
        }
    }

    #[test]
    fn test_countdown_sequence_and_audio() {
        let mut cd = CountDown::new(3);
        let sink = MockSink::new();

        assert_eq!(cd.text(), "3");
        assert_eq!(cd.remaining, 3);
        assert!(!cd.is_finished);

        // Advance 1 second -> ticks to 2
        let evt = cd.update_with_audio(1.0, &sink);
        assert_eq!(evt, Some(CountDownEvent::Tick(2)));
        assert_eq!(cd.text(), "2");
        assert_eq!(sink.cues.borrow().last(), Some(&SoundCue::CountdownLow));

        // Advance 1 second -> ticks to 1
        let evt = cd.update_with_audio(1.0, &sink);
        assert_eq!(evt, Some(CountDownEvent::Tick(1)));
        assert_eq!(cd.text(), "1");
        assert_eq!(sink.cues.borrow().last(), Some(&SoundCue::CountdownLow));

        // Advance 1 second -> triggers GO
        let evt = cd.update_with_audio(1.0, &sink);
        assert_eq!(evt, Some(CountDownEvent::Go));
        assert_eq!(cd.text(), "GO!");
        assert_eq!(sink.cues.borrow().last(), Some(&SoundCue::CountdownHigh));

        // Advance 1 second -> finishes
        let evt = cd.update_with_audio(1.0, &sink);
        assert_eq!(evt, Some(CountDownEvent::Finished));
        assert!(cd.is_finished);
    }
}
