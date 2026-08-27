//! Arcade Sound Effects Generator.
//!
//! Synthesizes clean, high-quality, pleasant arcade sound effects in 16-bit 44.1kHz PCM WAV:
//! - Crisp tire drift chirps (classic arcade racer squeaks)
//! - Solid low-end impact thuds (wall & car collision hits)
//! - Crystal clear countdown beeps & "GO!" chime
//! - Celebratory lap arpeggio, sector ping & victory fanfare
//! - Snappy UI navigation clicks

use crate::audio::dsp::{
    encode_wav_16bit_mono, encode_wav_16bit_stereo, soft_saturate, BiquadLowPass,
    Oscillator,
};

/// Generates a short, crisp arcade tire chirp/squeak (~0.09s).
/// High-pass filtered clean chirp without harsh digital noise.
pub fn generate_skid_sound(sample_rate: u32) -> Vec<u8> {
    let duration = 0.09;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut filter = BiquadLowPass::new(sample_rate, 2200.0, 1.0);

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = if t < 0.005 {
            t / 0.005
        } else {
            (1.0 - (t - 0.005) / (duration - 0.005)).max(0.0).powi(2)
        };

        // Smooth downward arcade chirp from 950 Hz to 600 Hz
        let freq = 950.0 - (t / duration) * 350.0;
        let tone = Oscillator::sine(t * freq) * 0.75 + Oscillator::triangle(t * (freq * 0.5)) * 0.25;
        let filtered = filter.process(tone);

        *sample = soft_saturate(filtered * env, 1.1) * 0.70;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates a solid, punchy arcade wall impact thud (~0.18s).
/// Warm sub-bass punch with quick decay, free of harsh distortion.
pub fn generate_wall_crash_sound(sample_rate: u32) -> Vec<u8> {
    let duration = 0.18;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut filter = BiquadLowPass::new(sample_rate, 500.0, 1.2);

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).max(0.0).powi(3);

        // Low punchy pitch drop from 115 Hz to 35 Hz
        let pitch = 115.0 * (-t * 26.0).exp() + 35.0;
        let thump = Oscillator::sine(t * pitch) * 0.85 + Oscillator::triangle(t * (pitch * 0.5)) * 0.25;
        let filtered = filter.process(thump);

        *sample = soft_saturate(filtered * env, 1.2) * 0.85;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates a crisp arcade car-to-car impact tap (~0.12s).
pub fn generate_car_hit_sound(sample_rate: u32) -> Vec<u8> {
    let duration = 0.12;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut filter = BiquadLowPass::new(sample_rate, 800.0, 1.0);

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).max(0.0).powi(2);

        let pitch = 160.0 * (-t * 32.0).exp() + 55.0;
        let body = (Oscillator::sine(t * pitch) * 0.75 + Oscillator::triangle(t * pitch)) * 0.5;
        let filtered = filter.process(body);

        *sample = soft_saturate(filtered * env, 1.1) * 0.75;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates countdown low tone round beep (440 Hz A4, 0.14s).
pub fn generate_countdown_low(sample_rate: u32) -> Vec<u8> {
    let duration = 0.14;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = if t < 0.004 {
            t / 0.004
        } else {
            (1.0 - (t - 0.004) / (duration - 0.004)).max(0.0).powi(2)
        };

        let tone = Oscillator::sine(t * 440.0) * 0.85 + Oscillator::triangle(t * 880.0) * 0.15;
        *sample = tone * env * 0.75;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates countdown "GO!" bright arcade chime (880 Hz + 1320 Hz, 0.32s).
pub fn generate_countdown_high(sample_rate: u32) -> Vec<u8> {
    let duration = 0.32;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = if t < 0.004 {
            t / 0.004
        } else {
            (1.0 - (t - 0.004) / (duration - 0.004)).max(0.0).powi(2)
        };

        let bell = Oscillator::sine(t * 880.0) * 0.60
            + Oscillator::sine(t * 1320.0) * 0.40;
        *sample = bell * env * 0.85;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates celebratory ascending lap completion arpeggio (C5 -> E5 -> G5 -> C6, ~0.36s).
pub fn generate_lap_chime(sample_rate: u32) -> Vec<u8> {
    let duration = 0.36;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let note_hz = [523.25, 659.25, 783.99, 1046.50]; // C5, E5, G5, C6
    let step_sec = duration / 4.0;

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let note_idx = ((t / step_sec) as usize).min(3);
        let t_note = t % step_sec;
        let env = (1.0 - t_note / step_sec).max(0.0).powi(2);

        let hz = note_hz[note_idx];
        let tone = Oscillator::sine(t * hz) * 0.80 + Oscillator::triangle(t * (hz * 2.0)) * 0.20;
        *sample = tone * env * 0.70;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates crisp dual-tone sector split ping (987.77 Hz B5 + 1479.98 Hz F#6, ~0.16s).
pub fn generate_sector_ping(sample_rate: u32) -> Vec<u8> {
    let duration = 0.16;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).max(0.0).powi(2);
        let tone1 = Oscillator::sine(t * 987.77) * 0.55;
        let tone2 = Oscillator::sine(t * 1479.98) * 0.45;
        *sample = (tone1 + tone2) * env * 0.65;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates snappy 80s UI navigation select blip (880 Hz swept to 1320 Hz, 0.04s).
pub fn generate_ui_select(sample_rate: u32) -> Vec<u8> {
    let duration = 0.04;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).max(0.0);
        let freq = 880.0 + (t / duration) * 440.0;
        let tone = Oscillator::sine(t * freq);
        *sample = tone * env * 0.55;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates UI move tick (660 Hz, 0.02s).
pub fn generate_ui_move(sample_rate: u32) -> Vec<u8> {
    let duration = 0.02;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).max(0.0).powi(2);
        let tone = Oscillator::sine(t * 660.0);
        *sample = tone * env * 0.40;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates victorious race finish celebration fanfare (~1.0s stereo).
pub fn generate_race_finish(sample_rate: u32) -> Vec<u8> {
    let duration = 1.0;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut left_samples = vec![0.0f32; total_samples];
    let mut right_samples = vec![0.0f32; total_samples];

    // Chords: F (0.0 - 0.3s) -> G (0.3 - 0.6s) -> Cmaj (0.6 - 1.0s)
    let chords = [
        [349.23, 440.0, 523.25], // F
        [392.0, 493.88, 587.33], // G
        [523.25, 659.25, 783.99], // C
    ];

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let chord_idx = ((t / 0.3) as usize).min(2);
        let chord_t = if chord_idx < 2 { t % 0.3 } else { t - 0.6 };
        let env = (1.0 - chord_t / if chord_idx == 2 { 0.4 } else { 0.3 }).max(0.0).powi(2);

        let chord = chords[chord_idx];
        let mut chord_l = 0.0f32;
        let mut chord_r = 0.0f32;
        for (idx, &hz) in chord.iter().enumerate() {
            let v = (Oscillator::sine(t * hz) * 0.70 + Oscillator::triangle(t * (hz * 2.0)) * 0.30) * 0.22;
            let pan = (idx as f32 / 2.0) * 0.4 - 0.2;
            chord_l += v * (1.0 - pan);
            chord_r += v * (1.0 + pan);
        }

        left_samples[i] = chord_l * env * 0.80;
        right_samples[i] = chord_r * env * 0.80;
    }

    let mut frames = Vec::with_capacity(total_samples);
    for i in 0..total_samples {
        frames.push((left_samples[i], right_samples[i]));
    }

    encode_wav_16bit_stereo(&frames, sample_rate)
}

/// Generates an integer-cycle seamless looping engine harmonic sound for a specific RPM band frequency.
pub fn generate_engine_rpm_band(sample_rate: u32, base_hz: f32) -> Vec<u8> {
    // Integer crank cycles so every oscillator multiple (0.5x, 1x, 2x, 3x) loops seamlessly
    let crank_hz = base_hz * 0.5;
    let target_duration = 0.5;
    let num_cycles = (target_duration * crank_hz).round().max(1.0);
    let duration = num_cycles / crank_hz;
    let total_samples = (duration * sample_rate as f32).round() as usize;

    // Low-pass opens with RPM: muffled idle, bright redline scream
    let cutoff = (base_hz * 6.0).clamp(300.0, 6000.0);
    let mut filter = BiquadLowPass::new(sample_rate, cutoff, 0.95);

    // Pre-roll settles the stateful filter into steady state so the loop wraps cleanly
    let pre_roll = (0.05 * sample_rate as f32).round() as usize;
    let gen_len = pre_roll + total_samples;

    let mut samples = vec![0.0f32; gen_len];
    for (i, sample) in samples.iter_mut().enumerate() {
        let t = i as f32 / sample_rate as f32;
        let phase = t * base_hz;

        // Sub-octave crank rotation (0.5x): deep lumpy idle rumble
        let sub =
            Oscillator::sine(phase * 0.5) * 0.32 + Oscillator::square(phase * 0.5, 0.5) * 0.10;

        // Firing frequency (1x): rich saw body plus 2nd-order intake harmonic
        let fire = Oscillator::saw(phase) * 0.62 + Oscillator::sine(phase * 2.0) * 0.28;

        // 3rd-order exhaust bark for top-end aggression
        let bark = Oscillator::saw(phase * 3.0) * 0.22;

        let raw = sub + fire + bark;
        *sample = soft_saturate(filter.process(raw), 1.2) * 0.85;
    }

    encode_wav_16bit_mono(&samples[pre_roll..], sample_rate)
}

/// Generates a gear upshift exhaust backfire pop & turbo blow-off (~0.05s).
pub fn generate_gear_shift_pop(sample_rate: u32) -> Vec<u8> {
    let duration = 0.05;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut filter = BiquadLowPass::new(sample_rate, 1200.0, 1.4);

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).max(0.0).powi(3);

        let pitch = 180.0 * (-t * 45.0).exp() + 45.0;
        let crack = Oscillator::square(t * pitch, 0.35) * 0.65 + Oscillator::sine(t * pitch) * 0.35;
        let filtered = filter.process(crack);

        *sample = soft_saturate(filtered * env, 1.4) * 0.85;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Legacy single engine sound generator (using Idle band frequency)
pub fn generate_engine_sound(sample_rate: u32) -> Vec<u8> {
    generate_engine_rpm_band(sample_rate, 65.0)
}

/// Placeholder generator for curb rumble
pub fn generate_curb_rumble_sound(sample_rate: u32) -> Vec<u8> {
    let samples = vec![0.0f32; 100];
    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Placeholder generator for off-road rumble
pub fn generate_offroad_sound(sample_rate: u32) -> Vec<u8> {
    let samples = vec![0.0f32; 100];
    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates an arcade jump launch aerodynamic whoosh and pitch rise (~0.18s).
pub fn generate_jump_launch_sound(sample_rate: u32) -> Vec<u8> {
    let duration = 0.18;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut filter = BiquadLowPass::new(sample_rate, 1800.0, 1.1);

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = if t < 0.02 {
            t / 0.02
        } else {
            (1.0 - (t - 0.02) / (duration - 0.02)).max(0.0).powi(2)
        };

        let pitch = 180.0 + (t / duration).powi(2) * 320.0;
        let tone = Oscillator::sine(t * pitch) * 0.60 + Oscillator::saw(t * (pitch * 0.5)) * 0.40;
        let filtered = filter.process(tone);

        *sample = soft_saturate(filtered * env, 1.2) * 0.80;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates a solid suspension compression landing impact thud (~0.20s).
pub fn generate_landing_sound(sample_rate: u32) -> Vec<u8> {
    let duration = 0.20;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut filter = BiquadLowPass::new(sample_rate, 450.0, 1.3);

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).max(0.0).powi(3);

        let pitch = 95.0 * (-t * 22.0).exp() + 30.0;
        let thump = Oscillator::sine(t * pitch) * 0.80 + Oscillator::triangle(t * (pitch * 0.5)) * 0.30;
        let filtered = filter.process(thump);

        *sample = soft_saturate(filtered * env, 1.3) * 0.90;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates a viscous water splash and aquaplaning spray sound (~0.24s).
pub fn generate_water_splash_sound(sample_rate: u32) -> Vec<u8> {
    let duration = 0.24;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut filter = BiquadLowPass::new(sample_rate, 2200.0, 1.2);
    let mut rng_state = 987654321u64;

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).max(0.0).powi(2);

        // Pseudo-random white noise for fluid water spray hiss
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let noise = ((rng_state >> 32) as i32 as f32) / 2147483648.0;

        // Sub-bass water displacement plunge
        let sub_pitch = 110.0 * (-t * 18.0).exp() + 35.0;
        let sub_thump = Oscillator::sine(t * sub_pitch) * 0.70;

        let raw = noise * 0.65 + sub_thump;
        let filtered = filter.process(raw);

        *sample = soft_saturate(filtered * env, 1.2) * 0.85;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::dsp::DEFAULT_SAMPLE_RATE;

    #[test]
    fn test_all_sfx_generators_produce_valid_wav_headers() {
        let sfx_list = vec![
            ("engine", generate_engine_sound(DEFAULT_SAMPLE_RATE)),
            ("shift_pop", generate_gear_shift_pop(DEFAULT_SAMPLE_RATE)),
            ("rpm_band_55", generate_engine_rpm_band(DEFAULT_SAMPLE_RATE, 55.0)),
            ("rpm_band_420", generate_engine_rpm_band(DEFAULT_SAMPLE_RATE, 420.0)),
            ("skid", generate_skid_sound(DEFAULT_SAMPLE_RATE)),
            ("wall_crash", generate_wall_crash_sound(DEFAULT_SAMPLE_RATE)),
            ("car_hit", generate_car_hit_sound(DEFAULT_SAMPLE_RATE)),
            ("curb", generate_curb_rumble_sound(DEFAULT_SAMPLE_RATE)),
            ("offroad", generate_offroad_sound(DEFAULT_SAMPLE_RATE)),
            ("cd_low", generate_countdown_low(DEFAULT_SAMPLE_RATE)),
            ("cd_high", generate_countdown_high(DEFAULT_SAMPLE_RATE)),
            ("lap", generate_lap_chime(DEFAULT_SAMPLE_RATE)),
            ("sector", generate_sector_ping(DEFAULT_SAMPLE_RATE)),
            ("ui_select", generate_ui_select(DEFAULT_SAMPLE_RATE)),
            ("ui_move", generate_ui_move(DEFAULT_SAMPLE_RATE)),
            ("finish", generate_race_finish(DEFAULT_SAMPLE_RATE)),
        ];

        for (name, wav) in sfx_list {
            assert_eq!(&wav[0..4], b"RIFF", "SFX {} invalid RIFF header", name);
            assert_eq!(&wav[8..12], b"WAVE", "SFX {} invalid WAVE header", name);
            assert!(wav.len() > 44, "SFX {} has no payload", name);
        }
    }

    /// Decodes the PCM payload of a mono 16-bit WAV into normalized f32 samples.
    fn decode_mono_pcm(wav: &[u8]) -> Vec<f32> {
        wav[44..]
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
            .collect()
    }

    /// Goertzel magnitude of `freq` over the sample buffer (dependency-free spectral probe).
    fn goertzel(samples: &[f32], sample_rate: u32, freq: f32) -> f32 {
        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate as f32;
        let coeff = 2.0 * omega.cos();
        let (mut s1, mut s2) = (0.0f32, 0.0f32);
        for &x in samples {
            let s = x + coeff * s1 - s2;
            s2 = s1;
            s1 = s;
        }
        (s1 * s1 + s2 * s2 - coeff * s1 * s2).sqrt()
    }

    /// Regression: engine bands must be real pitched tones, not the DC-clipped
    /// garbage caused by unwrapped triangle phases (historically dc=+0.78).
    #[test]
    fn test_engine_rpm_band_is_pitched_tone_not_dc() {
        for &base_hz in &[42.5f32, 150.0, 465.0] {
            let x = decode_mono_pcm(&generate_engine_rpm_band(DEFAULT_SAMPLE_RATE, base_hz));

            // No DC offset / saturation clipping
            let mean: f32 = x.iter().sum::<f32>() / x.len() as f32;
            assert!(
                mean.abs() < 0.05,
                "band {base_hz} Hz has DC offset {mean} - oscillator phase bug"
            );

            // Audible level
            let rms = (x.iter().map(|s| s * s).sum::<f32>() / x.len() as f32).sqrt();
            assert!(
                (0.15..0.85).contains(&rms),
                "band {base_hz} Hz RMS {rms} out of audible range"
            );

            // Pitch must sit at the firing frequency, not at a detuned bin
            let fire_power = goertzel(&x, DEFAULT_SAMPLE_RATE, base_hz);
            let off_power = goertzel(&x, DEFAULT_SAMPLE_RATE, base_hz * 1.37);
            assert!(
                fire_power > off_power * 3.0,
                "band {base_hz} Hz lacks firing-frequency energy (fire={fire_power}, off={off_power})"
            );
        }
    }

    /// Engine band loops must wrap seamlessly: last and first samples nearly equal.
    #[test]
    fn test_engine_rpm_band_loop_is_seamless() {
        for &base_hz in &[42.5f32, 305.0] {
            let x = decode_mono_pcm(&generate_engine_rpm_band(DEFAULT_SAMPLE_RATE, base_hz));
            let seam = (x[x.len() - 1] - x[0]).abs();
            assert!(seam < 0.08, "band {base_hz} Hz loop seam click {seam}");
        }
    }
}
