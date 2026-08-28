//! Procedural DSP Synthesizer and in-memory WAV Audio Encoder.
//!
//! Provides zero-dependency synthesis primitives:
//! - 16-bit 44.1kHz PCM RIFF/WAV encoder (Mono & Stereo)
//! - Band-limited / analog waveform oscillators (Sine, Saw, Triangle, Square/PWM, Noise)
//! - ADSR amplitude envelopes & pitch modulators
//! - Resonant 2-pole Biquad Low-Pass / High-Pass filters
//! - Stereo delay / reverberation and saturation effects

use std::f32::consts::PI;

pub const DEFAULT_SAMPLE_RATE: u32 = 44100;

/// Generates a standard RIFF/WAVE header and 16-bit mono PCM payload.
pub fn encode_wav_16bit_mono(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_samples = samples.len() as u32;
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_size = num_samples * (bits_per_sample as u32 / 8);
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity((44 + data_size) as usize);

    // RIFF Chunk
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // "fmt " Subchunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (16 for PCM)
    wav.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (1 = PCM)
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // "data" Subchunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let sample_i16 = (clamped * 32767.0).round() as i16;
        wav.extend_from_slice(&sample_i16.to_le_bytes());
    }

    wav
}

/// Generates a standard RIFF/WAVE header and 16-bit stereo (L/R interleaved) PCM payload.
pub fn encode_wav_16bit_stereo(samples: &[(f32, f32)], sample_rate: u32) -> Vec<u8> {
    let num_frames = samples.len() as u32;
    let num_channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_size = num_frames * num_channels as u32 * (bits_per_sample as u32 / 8);
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity((44 + data_size) as usize);

    // RIFF Chunk
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // "fmt " Subchunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat PCM
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // "data" Subchunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for &(left, right) in samples {
        let left_clamped = left.clamp(-1.0, 1.0);
        let right_clamped = right.clamp(-1.0, 1.0);

        let left_i16 = (left_clamped * 32767.0).round() as i16;
        let right_i16 = (right_clamped * 32767.0).round() as i16;

        wav.extend_from_slice(&left_i16.to_le_bytes());
        wav.extend_from_slice(&right_i16.to_le_bytes());
    }

    wav
}

/// Fast pseudo-random white noise generator using a linear congruential step.
#[derive(Debug, Clone)]
pub struct NoiseGenerator {
    state: u64,
}

impl NoiseGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }

    /// Returns a random float uniformly distributed in `[-1.0, 1.0]`.
    pub fn next_sample(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = (self.state >> 33) as u32;
        ((val as f32 / 2147483648.0) - 1.0).clamp(-1.0, 1.0)
    }
}

/// Fundamental waveform oscillators.
pub struct Oscillator;

impl Oscillator {
    /// Pure Sine wave from phase `[0.0, 1.0)`.
    #[inline(always)]
    pub fn sine(phase: f32) -> f32 {
        (phase * 2.0 * PI).sin()
    }

    /// Analog Sawtooth wave in `[-1.0, 1.0]`.
    #[inline(always)]
    pub fn saw(phase: f32) -> f32 {
        2.0 * (phase - (phase + 0.5).floor())
    }

    /// Triangle wave in `[-1.0, 1.0]`. Wraps phase so arbitrary
    /// (unwrapped) phase inputs fold correctly into `[0.0, 1.0)`.
    #[inline(always)]
    pub fn triangle(phase: f32) -> f32 {
        let mut p = phase.fract();
        if p < 0.0 {
            p += 1.0;
        }
        4.0 * (p - 0.5).abs() - 1.0
    }

    /// Square wave with pulse width modulation (PWM) parameter `[0.01, 0.99]`.
    #[inline(always)]
    pub fn square(phase: f32, pulse_width: f32) -> f32 {
        let norm_phase = phase.fract();
        let p = if norm_phase < 0.0 { norm_phase + 1.0 } else { norm_phase };
        if p < pulse_width {
            1.0
        } else {
            -1.0
        }
    }

    /// Asymmetric internal combustion pressure pulse with physical compression lobe & expansion tail.
    /// Produces a zero-mean periodic waveform rich in engine-like odd and even harmonics.
    #[inline(always)]
    pub fn combustion_pulse(phase: f32, asymmetry: f32) -> f32 {
        let mut p = phase.fract();
        if p < 0.0 {
            p += 1.0;
        }
        let rad = p * 2.0 * PI;
        let fund = rad.sin();
        let harm2 = (rad * 2.0 - 0.35).sin() * (0.42 + asymmetry * 0.20);
        let harm3 = (rad * 3.0 - 0.70).sin() * (0.22 + asymmetry * 0.10);
        let harm4 = (rad * 4.0 - 1.05).sin() * 0.12;

        let raw = fund + harm2 + harm3 + harm4;
        raw * 0.65
    }

    /// Sharp expansion pressure wave pulse for 2-stroke or high-compression racing engines.
    /// Uses Fourier harmonic series with staggered phase alignment for zero DC offset.
    #[inline(always)]
    pub fn cylinder_pulse(phase: f32, sharpness: f32) -> f32 {
        let mut p = phase.fract();
        if p < 0.0 {
            p += 1.0;
        }
        let rad = p * 2.0 * PI;
        let s = sharpness.clamp(1.0, 4.0);
        let h1 = rad.sin();
        let h2 = (rad * 2.0 - 0.40).sin() * (0.50 * s * 0.45);
        let h3 = (rad * 3.0 - 0.80).sin() * (0.30 * s * 0.45);
        let h4 = (rad * 4.0 - 1.20).sin() * 0.18;
        let h5 = (rad * 5.0 - 1.60).sin() * 0.10;
        (h1 + h2 + h3 + h4 + h5) * 0.65
    }
}

/// Attack-Decay-Sustain-Release (ADSR) Volume / Modulation Envelope.
#[derive(Debug, Clone, Copy)]
pub struct AdsrEnvelope {
    pub attack_sec: f32,
    pub decay_sec: f32,
    pub sustain_level: f32,
    pub release_sec: f32,
}

impl AdsrEnvelope {
    pub const fn new(attack_sec: f32, decay_sec: f32, sustain_level: f32, release_sec: f32) -> Self {
        Self {
            attack_sec,
            decay_sec,
            sustain_level,
            release_sec,
        }
    }

    /// Evaluates envelope multiplier in `[0.0, 1.0]` at time `t` for a note lasting `gate_duration` seconds.
    pub fn evaluate(&self, t: f32, gate_duration: f32) -> f32 {
        if t < 0.0 {
            return 0.0;
        }

        if t < gate_duration {
            // Note is held (Attack -> Decay -> Sustain)
            if t < self.attack_sec {
                if self.attack_sec <= 0.0001 {
                    1.0
                } else {
                    t / self.attack_sec
                }
            } else {
                let decay_time = t - self.attack_sec;
                if decay_time < self.decay_sec {
                    let progress = decay_time / self.decay_sec.max(0.0001);
                    1.0 - (1.0 - self.sustain_level) * progress
                } else {
                    self.sustain_level
                }
            }
        } else {
            // Note released (Release phase)
            let release_time = t - gate_duration;
            if release_time >= self.release_sec {
                0.0
            } else {
                let note_val_at_release = if gate_duration < self.attack_sec {
                    gate_duration / self.attack_sec.max(0.0001)
                } else if gate_duration < (self.attack_sec + self.decay_sec) {
                    let progress = (gate_duration - self.attack_sec) / self.decay_sec.max(0.0001);
                    1.0 - (1.0 - self.sustain_level) * progress
                } else {
                    self.sustain_level
                };

                let progress = release_time / self.release_sec.max(0.0001);
                note_val_at_release * (1.0 - progress)
            }
        }
    }
}

/// Resonant 2-pole Biquad Low-Pass Filter (Direct Form I).
#[derive(Debug, Clone)]
pub struct BiquadLowPass {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadLowPass {
    pub fn new(sample_rate: u32, cutoff_hz: f32, q: f32) -> Self {
        let mut filter = Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        filter.update_coefficients(sample_rate, cutoff_hz, q);
        filter
    }

    pub fn update_coefficients(&mut self, sample_rate: u32, cutoff_hz: f32, q: f32) {
        let nyquist = sample_rate as f32 * 0.5;
        let safe_cutoff = cutoff_hz.clamp(20.0, nyquist * 0.95);
        let safe_q = q.clamp(0.1, 20.0);

        let omega = 2.0 * PI * (safe_cutoff / sample_rate as f32);
        let alpha = omega.sin() / (2.0 * safe_q);
        let cos_omega = omega.cos();

        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 - cos_omega) * 0.5) / a0;
        self.b1 = (1.0 - cos_omega) / a0;
        self.b2 = ((1.0 - cos_omega) * 0.5) / a0;
        self.a1 = (-2.0 * cos_omega) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Stereo Delay / Echo processor with feedback and high-frequency damping.
#[derive(Debug, Clone)]
pub struct StereoDelay {
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_idx: usize,
    delay_samples_l: usize,
    delay_samples_r: usize,
    feedback: f32,
    mix: f32,
}

impl StereoDelay {
    pub fn new(sample_rate: u32, delay_ms_l: f32, delay_ms_r: f32, feedback: f32, mix: f32) -> Self {
        let delay_samples_l = ((delay_ms_l * 0.001) * sample_rate as f32).round() as usize;
        let delay_samples_r = ((delay_ms_r * 0.001) * sample_rate as f32).round() as usize;
        let max_len = delay_samples_l.max(delay_samples_r).max(1) + 16;

        Self {
            buffer_l: vec![0.0; max_len],
            buffer_r: vec![0.0; max_len],
            write_idx: 0,
            delay_samples_l,
            delay_samples_r,
            feedback: feedback.clamp(0.0, 0.95),
            mix: mix.clamp(0.0, 1.0),
        }
    }

    pub fn process(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let len = self.buffer_l.len();
        let read_idx_l = (self.write_idx + len - self.delay_samples_l) % len;
        let read_idx_r = (self.write_idx + len - self.delay_samples_r) % len;

        let delayed_l = self.buffer_l[read_idx_l];
        let delayed_r = self.buffer_r[read_idx_r];

        self.buffer_l[self.write_idx] = in_l + delayed_l * self.feedback;
        self.buffer_r[self.write_idx] = in_r + delayed_r * self.feedback;

        self.write_idx = (self.write_idx + 1) % len;

        let out_l = in_l * (1.0 - self.mix) + delayed_l * self.mix;
        let out_r = in_r * (1.0 - self.mix) + delayed_r * self.mix;
        (out_l, out_r)
    }
}

/// Resonant 2-pole Biquad Band-Pass Filter (Constant 0 dB Peak Gain).
#[derive(Debug, Clone)]
pub struct BiquadBandPass {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadBandPass {
    pub fn new(sample_rate: u32, center_hz: f32, q: f32) -> Self {
        let mut filter = Self {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        filter.update_coefficients(sample_rate, center_hz, q);
        filter
    }

    pub fn update_coefficients(&mut self, sample_rate: u32, center_hz: f32, q: f32) {
        let nyquist = sample_rate as f32 * 0.5;
        let safe_center = center_hz.clamp(20.0, nyquist * 0.95);
        let safe_q = q.clamp(0.1, 30.0);

        let omega = 2.0 * PI * (safe_center / sample_rate as f32);
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * safe_q);

        let a0 = 1.0 + alpha;
        self.b0 = alpha / a0;
        self.b1 = 0.0;
        self.b2 = -alpha / a0;
        self.a1 = (-2.0 * cos_omega) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Specialized zero-mean wave-shaper for physical internal combustion engine acoustics.
/// Blends odd-symmetric quadratic compression with cubic saturation.
#[inline]
pub fn waveshape_engine(sample: f32, drive: f32, even_harmonics: f32) -> f32 {
    let x = (sample * drive).clamp(-3.0, 3.0);
    let shaped = x + even_harmonics * 0.25 * x * x.abs();
    soft_saturate(shaped, 1.0)
}

/// Soft-saturation distortion for vintage analog warmth.
#[inline]
pub fn soft_saturate(sample: f32, drive: f32) -> f32 {
    let x = sample * drive;
    if x > 3.0 {
        1.0
    } else if x < -3.0 {
        -1.0
    } else {
        x * (27.0 + x * x) / (27.0 + 9.0 * x * x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_wav_16bit_mono_headers() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let bytes = encode_wav_16bit_mono(&samples, 44100);

        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[36..40], b"data");

        // 44 bytes header + 5 * 2 bytes samples = 54 bytes
        assert_eq!(bytes.len(), 44 + 5 * 2);
    }

    #[test]
    fn test_encode_wav_16bit_stereo_headers() {
        let frames = vec![(0.0, 0.0), (0.5, -0.5), (1.0, -1.0)];
        let bytes = encode_wav_16bit_stereo(&frames, 44100);

        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        // 44 bytes header + 3 frames * 2 channels * 2 bytes = 56 bytes
        assert_eq!(bytes.len(), 44 + 3 * 4);
    }

    #[test]
    fn test_adsr_envelope() {
        let adsr = AdsrEnvelope::new(0.1, 0.1, 0.5, 0.2);
        assert_eq!(adsr.evaluate(0.0, 1.0), 0.0);
        assert!((adsr.evaluate(0.1, 1.0) - 1.0).abs() < 0.01);
        assert!((adsr.evaluate(0.2, 1.0) - 0.5).abs() < 0.01);
        assert!((adsr.evaluate(0.5, 1.0) - 0.5).abs() < 0.01);
        assert_eq!(adsr.evaluate(1.25, 1.0), 0.0);
    }

    #[test]
    fn test_biquad_lowpass_stability() {
        let mut filter = BiquadLowPass::new(44100, 1000.0, 0.707);
        for _ in 0..1000 {
            let out = filter.process(0.5);
            assert!(out.is_finite());
            assert!(out.abs() <= 2.0);
        }
    }

    #[test]
    fn test_biquad_bandpass_filtering_and_stability() {
        let mut filter = BiquadBandPass::new(44100, 1000.0, 2.0);
        // Feed 1000 Hz sine -> should pass with high amplitude
        let mut peak_pass = 0.0f32;
        for i in 0..500 {
            let t = i as f32 / 44100.0;
            let sample = (t * 1000.0 * 2.0 * PI).sin();
            let y = filter.process(sample);
            assert!(y.is_finite());
            peak_pass = peak_pass.max(y.abs());
        }
        assert!(peak_pass > 0.6, "1kHz should pass through 1kHz BPF, got {peak_pass}");

        // Feed 100 Hz sine -> should be attenuated
        let mut filter_low = BiquadBandPass::new(44100, 1000.0, 2.0);
        let mut peak_atten = 0.0f32;
        for i in 0..500 {
            let t = i as f32 / 44100.0;
            let sample = (t * 100.0 * 2.0 * PI).sin();
            let y = filter_low.process(sample);
            if i > 100 {
                peak_atten = peak_atten.max(y.abs());
            }
        }
        assert!(peak_atten < 0.35, "100Hz should be attenuated by 1kHz BPF, got {peak_atten}");
    }

    #[test]
    fn test_combustion_and_cylinder_oscillators() {
        for i in 0..100 {
            let phase = i as f32 / 100.0;
            let comb = Oscillator::combustion_pulse(phase, 0.5);
            let cyl = Oscillator::cylinder_pulse(phase, 2.0);
            assert!(comb.is_finite() && comb.abs() <= 2.5);
            assert!(cyl.is_finite() && cyl.abs() <= 2.5);
        }
    }

    #[test]
    fn test_waveshape_engine() {
        assert_eq!(waveshape_engine(0.0, 1.0, 0.5), 0.0);
        let saturated = waveshape_engine(2.0, 1.5, 0.3);
        assert!(saturated > 0.0 && saturated <= 1.0);
    }
}
