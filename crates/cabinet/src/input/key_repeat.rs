use serde::{Deserialize, Serialize};

/// Hold-acceleration key repeat processor for steppers, counters, and cyclers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyRepeat {
    /// Initial delay before auto-repeat begins (e.g. 0.35s).
    pub delay_sec: f32,
    /// Repeat frequency (ticks per second) when held (e.g. 10.0).
    pub rate_per_sec: f32,
    /// Total elapsed holding time.
    pub elapsed_sec: f32,
    /// Internal accumulator for sub-tick increments.
    pub repeat_accumulator: f32,
    /// Whether the key is currently held.
    pub holding: bool,
}

impl Default for KeyRepeat {
    fn default() -> Self {
        Self::new(0.35, 12.0)
    }
}

impl KeyRepeat {
    pub fn new(delay_sec: f32, rate_per_sec: f32) -> Self {
        Self {
            delay_sec: delay_sec.max(0.05),
            rate_per_sec: rate_per_sec.max(0.5),
            elapsed_sec: 0.0,
            repeat_accumulator: 0.0,
            holding: false,
        }
    }

    /// Resets the key repeat state when released.
    pub fn reset(&mut self) {
        self.elapsed_sec = 0.0;
        self.repeat_accumulator = 0.0;
        self.holding = false;
    }

    /// Updates holding state with time delta `dt`.
    /// Returns the number of triggers (0 on idle/delay, 1 on initial press or per interval) fired this tick.
    pub fn update(&mut self, is_down: bool, dt: f32) -> usize {
        if !is_down {
            self.reset();
            return 0;
        }

        if !self.holding {
            // First frame pressed
            self.holding = true;
            self.elapsed_sec = 0.0;
            self.repeat_accumulator = 0.0;
            return 1;
        }

        self.elapsed_sec += dt;
        if self.elapsed_sec >= self.delay_sec {
            let interval = 1.0 / self.rate_per_sec;
            self.repeat_accumulator += dt;
            let mut ticks = 0;
            while self.repeat_accumulator >= interval {
                self.repeat_accumulator -= interval;
                ticks += 1;
            }
            ticks
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_repeat_lifecycle() {
        let mut kr = KeyRepeat::new(0.3, 10.0); // 0.3s delay, 10 ticks/sec (0.1s interval)

        // Initial press triggers 1
        assert_eq!(kr.update(true, 0.016), 1);
        assert!(kr.holding);

        // During delay period (0.016 to 0.29s), returns 0
        assert_eq!(kr.update(true, 0.1), 0);
        assert_eq!(kr.update(true, 0.1), 0);

        // Passes 0.3s threshold -> starts auto-repeating
        assert_eq!(kr.update(true, 0.1), 1);

        // Release resets state
        assert_eq!(kr.update(false, 0.016), 0);
        assert!(!kr.holding);

        // Re-press triggers immediate 1
        assert_eq!(kr.update(true, 0.016), 1);
    }
}
