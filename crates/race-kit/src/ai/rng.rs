//! Seeded random numbers for bots. Moved from `tdrace-app/src/ai/driver.rs`.

/// Deterministic linear congruential generator for reproducible roster sampling.
#[derive(Debug, Clone)]
pub struct LcgRng(pub u64);

impl LcgRng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(1442695040888963407))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }

    pub fn gen_range(&mut self, range: std::ops::Range<u32>) -> u32 {
        let len = range.end.saturating_sub(range.start);
        if len == 0 {
            return range.start;
        }
        range.start + (self.next_u32() % len)
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = (self.next_u64() >> 33) as usize % (i + 1);
            slice.swap(i, j);
        }
    }
}

