//! Arcade Sound Effects Generator.
//!
//! Synthesizes clean, high-quality, pleasant arcade sound effects in 16-bit 44.1kHz PCM WAV:
//! - Crisp tire drift chirps (classic arcade racer squeaks)
//! - Solid low-end impact thuds (wall & car collision hits)
//! - Crystal clear countdown beeps & "GO!" chime
//! - Celebratory lap arpeggio, sector ping & victory fanfare
//! - Snappy UI navigation clicks

use crate::audio::dsp::{
    encode_wav_16bit_mono, encode_wav_16bit_stereo, soft_saturate, waveshape_engine,
    BiquadBandPass, BiquadLowPass, NoiseGenerator, Oscillator,
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

/// Physical configuration parameters for procedural engine synthesis.
#[derive(Debug, Clone, Copy)]
pub struct EngineSoundConfig {
    /// Number of engine cylinders (e.g. 1 for Kart, 4 for Rally, 6 for F1, 8 for GT V8)
    pub cylinder_count: usize,
    /// Whether the engine operates on a 2-stroke cycle (true for Kart) vs 4-stroke cycle
    pub is_two_stroke: bool,
    /// Crankshaft sub-harmonic / imbalance intensity (e.g. high for crossplane V8 rumble)
    pub crank_lumpiness: f32,
    /// Combustion pulse asymmetry / shape parameter [0.0..1.0]
    pub combustion_asymmetry: f32,
    /// Intake air rush & induction roar intensity [0.0..1.0]
    pub intake_growl_intensity: f32,
    /// Turbocharger compressor spool whine intensity [0.0..1.0]
    pub turbo_whine_level: f32,
    /// Valvetrain & mechanical friction chatter intensity [0.0..1.0]
    pub mechanical_buzz: f32,
    /// Primary exhaust pipe chamber resonance frequency (Hz)
    pub formant_f1_hz: f32,
    /// Secondary metallic exhaust tailpipe rasp resonance frequency (Hz)
    pub formant_f2_hz: f32,
    /// Resonance Q-factor for exhaust formants
    pub formant_q: f32,
    /// Soft-saturation drive
    pub saturation_drive: f32,
}

impl Default for EngineSoundConfig {
    fn default() -> Self {
        Self::generic()
    }
}

impl EngineSoundConfig {
    /// Generic / Balanced 6-Cylinder 4-Stroke Sports Engine (Fallback Default)
    pub const fn generic() -> Self {
        Self {
            cylinder_count: 6,
            is_two_stroke: false,
            crank_lumpiness: 0.28,
            combustion_asymmetry: 0.40,
            intake_growl_intensity: 0.22,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.14,
            formant_f1_hz: 200.0,
            formant_f2_hz: 1800.0,
            formant_q: 1.8,
            saturation_drive: 1.20,
        }
    }

    /// High-Displacement Crossplane V8 Touring GT Muscle Engine
    pub const fn sport_gt() -> Self {
        Self {
            cylinder_count: 8,
            is_two_stroke: false,
            crank_lumpiness: 0.46, // Heavy crossplane V8 idle lumping
            combustion_asymmetry: 0.55,
            intake_growl_intensity: 0.35, // Throaty widebody induction roar
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.16,
            formant_f1_hz: 140.0, // Deep sub-bass chamber
            formant_f2_hz: 1450.0, // Low-pitch exhaust growl
            formant_q: 2.2,
            saturation_drive: 1.35,
        }
    }

    /// Screaming 125cc 2-Stroke Sprint Go-Kart Engine
    pub const fn kart_125cc() -> Self {
        Self {
            cylinder_count: 1,
            is_two_stroke: true, // 2-Stroke: fires every single crank revolution
            crank_lumpiness: 0.08,
            combustion_asymmetry: 0.65,
            intake_growl_intensity: 0.18,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.42, // Prominent metallic ring-a-ding buzz
            formant_f1_hz: 750.0, // Tuned expansion chamber pipe ring
            formant_f2_hz: 2600.0, // Sharp 2-stroke bite
            formant_q: 2.8,
            saturation_drive: 1.38,
        }
    }

    /// High-Revving Formula 1 V6 Turbo Hybrid Power Unit
    pub const fn f1_v6_turbo() -> Self {
        Self {
            cylinder_count: 6,
            is_two_stroke: false,
            crank_lumpiness: 0.18, // High-RPM racing balance
            combustion_asymmetry: 0.50,
            intake_growl_intensity: 0.28,
            turbo_whine_level: 0.15, // Subtle compressor spool whistle
            mechanical_buzz: 0.22,
            formant_f1_hz: 300.0,
            formant_f2_hz: 2400.0, // Metallic high-pitch scream
            formant_q: 2.4,
            saturation_drive: 1.35,
        }
    }

    /// 4-Cylinder WRC Turbo Anti-Lag Rally Engine
    pub const fn rally_turbo() -> Self {
        Self {
            cylinder_count: 4,
            is_two_stroke: false,
            crank_lumpiness: 0.32,
            combustion_asymmetry: 0.50,
            intake_growl_intensity: 0.38, // Aggressive induction gulp
            turbo_whine_level: 0.24, // Wastegate / spool
            mechanical_buzz: 0.20,
            formant_f1_hz: 180.0,
            formant_f2_hz: 2000.0, // Snappy 4-cyl exhaust rasp
            formant_q: 2.0,
            saturation_drive: 1.30,
        }
    }
}

/// Generates an integer-cycle seamless looping engine harmonic sound for a specific configuration and RPM frequency.
pub fn generate_custom_engine_rpm_band(sample_rate: u32, base_hz: f32, config: &EngineSoundConfig) -> Vec<u8> {
    // base_hz represents 6-cylinder reference (RPM / 20).
    // Effective RPM = base_hz * 20.0.
    let rpm = (base_hz * 20.0).max(500.0);
    let crank_hz = rpm / 60.0;
    let cycle_hz = if config.is_two_stroke {
        crank_hz
    } else {
        crank_hz * 0.5
    };
    let firing_hz = if config.is_two_stroke {
        (rpm * config.cylinder_count as f32) / 60.0
    } else {
        (rpm * config.cylinder_count as f32) / 120.0
    };

    // Calculate integer cycle duration for seamless click-free looping
    let target_duration = 0.5;
    let num_cycles = (target_duration * cycle_hz).round().max(1.0);
    let duration = num_cycles / cycle_hz;
    let total_samples = (duration * sample_rate as f32).round() as usize;

    // Filters: Master low-pass + exhaust pipe formants + intake noise filter
    let cutoff = (firing_hz * 6.0).clamp(400.0, 7500.0);
    let mut lp_filter = BiquadLowPass::new(sample_rate, cutoff, 0.95);
    let mut bp_formant1 = BiquadBandPass::new(sample_rate, config.formant_f1_hz, config.formant_q);
    let mut bp_formant2 = BiquadBandPass::new(sample_rate, config.formant_f2_hz, config.formant_q);
    let mut bp_intake = BiquadBandPass::new(sample_rate, (600.0 + firing_hz * 1.5).clamp(300.0, 3500.0), 1.4);

    let mut noise_gen = NoiseGenerator::new(0x4d595f454e47494e);

    // Overlap tail length for seamless crossfading
    let xfade_len = 256.min(total_samples / 4);

    // Pre-roll settles stateful biquad filters into steady-state periodic regime
    let pre_roll = (0.05 * sample_rate as f32).round() as usize;
    let gen_len = pre_roll + total_samples + xfade_len;

    let mut samples = vec![0.0f32; gen_len];
    for (i, sample) in samples.iter_mut().enumerate() {
        let t = i as f32 / sample_rate as f32;

        // 1. Physical Cylinder Combustion Pressure Pulses across engine cycle (staggered at 1/N cycle offsets)
        let mut combustion = 0.0f32;
        for c in 0..config.cylinder_count {
            let phase_offset = c as f32 / config.cylinder_count as f32;
            let cyl_phase = t * cycle_hz + phase_offset;
            if config.is_two_stroke {
                combustion += Oscillator::cylinder_pulse(cyl_phase, 2.0);
            } else {
                combustion += Oscillator::combustion_pulse(cyl_phase, config.combustion_asymmetry);
            }
        }

        // 2. Crankshaft Sub-Harmonic & Imbalance (Mechanical Body & Idle Lumping)
        let crank_phase = t * crank_hz;
        let sub = (Oscillator::sine(crank_phase * 0.5) * 0.70 + Oscillator::saw(crank_phase) * 0.30)
            * config.crank_lumpiness;

        // 3. Intake Induction Air Rush (Noise modulated at firing rate)
        let intake_noise = bp_intake.process(noise_gen.next_sample());
        let intake_mod = (Oscillator::saw(t * firing_hz) * 0.5 + 0.5) * intake_noise * config.intake_growl_intensity;

        // 4. Turbo Spool Whine (Smooth subtle harmonic whine if equipped)
        let turbo = if config.turbo_whine_level > 0.001 {
            let turbo_freq = (crank_hz * 6.0).clamp(600.0, 4500.0);
            Oscillator::sine(t * turbo_freq) * config.turbo_whine_level * 0.08
        } else {
            0.0
        };

        // 5. Valvetrain & Mechanical Harmonics (Natural Sawtooth harmonics)
        let valvetrain = if config.mechanical_buzz > 0.001 {
            Oscillator::saw(t * (firing_hz * 2.0)) * config.mechanical_buzz * 0.12
        } else {
            0.0
        };

        // Raw engine acoustic mixture
        let raw = combustion + sub + intake_mod + turbo + valvetrain;

        // 6. Dual-Stage Exhaust Formant Acoustic Resonators
        let form1 = bp_formant1.process(raw);
        let form2 = bp_formant2.process(raw);
        let shaped_exhaust = raw * 0.70 + form1 * 0.35 + form2 * 0.25;

        // 7. Master Low-Pass & Non-Linear Wave-Shaping
        let filtered = lp_filter.process(shaped_exhaust);
        let saturated = waveshape_engine(filtered, config.saturation_drive, 0.30);

        *sample = soft_saturate(saturated * 2.6, 1.0) * 0.90;
    }

    // Extract steady-state loop buffer and continuous overlap tail
    let mut pcm_samples = samples[pre_roll..(pre_roll + total_samples)].to_vec();
    let overlap_tail = &samples[(pre_roll + total_samples)..(pre_roll + total_samples + xfade_len)];

    // Crossfade overlap tail into start of loop for continuous seamless wrapping
    for k in 0..xfade_len {
        let w = k as f32 / xfade_len as f32;
        pcm_samples[k] = pcm_samples[k] * w + overlap_tail[k] * (1.0 - w);
    }

    // DC-blocking / zero-mean centering for clean audio hardware playback
    let mean: f32 = pcm_samples.iter().sum::<f32>() / pcm_samples.len().max(1) as f32;
    for s in &mut pcm_samples {
        *s -= mean;
    }

    encode_wav_16bit_mono(&pcm_samples, sample_rate)
}

/// Generates generic balanced engine sound loop band (fallback default).
pub fn generate_generic_engine_rpm_band(sample_rate: u32, base_hz: f32) -> Vec<u8> {
    generate_custom_engine_rpm_band(sample_rate, base_hz, &EngineSoundConfig::generic())
}

/// Generates deep crossplane V8 touring GT engine sound loop band.
pub fn generate_sport_gt_rpm_band(sample_rate: u32, base_hz: f32) -> Vec<u8> {
    generate_custom_engine_rpm_band(sample_rate, base_hz, &EngineSoundConfig::sport_gt())
}

/// Generates high-pitch 125cc 2-stroke kart engine sound loop band.
pub fn generate_kart_125cc_rpm_band(sample_rate: u32, base_hz: f32) -> Vec<u8> {
    generate_custom_engine_rpm_band(sample_rate, base_hz, &EngineSoundConfig::kart_125cc())
}

/// Generates screaming Formula 1 V6 turbo hybrid engine sound loop band.
pub fn generate_f1_v6_rpm_band(sample_rate: u32, base_hz: f32) -> Vec<u8> {
    generate_custom_engine_rpm_band(sample_rate, base_hz, &EngineSoundConfig::f1_v6_turbo())
}

/// Generates aggressive 4-cylinder WRC turbo rally engine sound loop band.
pub fn generate_rally_turbo_rpm_band(sample_rate: u32, base_hz: f32) -> Vec<u8> {
    generate_custom_engine_rpm_band(sample_rate, base_hz, &EngineSoundConfig::rally_turbo())
}

/// Legacy / Standard engine RPM band generator (aliases to generic procedural engine band).
pub fn generate_engine_rpm_band(sample_rate: u32, base_hz: f32) -> Vec<u8> {
    generate_generic_engine_rpm_band(sample_rate, base_hz)
}

/// Generates a gear upshift exhaust backfire pop & turbo blow-off (~0.06s).
pub fn generate_gear_shift_pop(sample_rate: u32) -> Vec<u8> {
    let duration = 0.06;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut lp_filter = BiquadLowPass::new(sample_rate, 1400.0, 1.4);
    let mut bp_filter = BiquadBandPass::new(sample_rate, 320.0, 2.5);
    let mut noise_gen = NoiseGenerator::new(0xdeadbeef12345678);

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).max(0.0).powi(3);

        // Sharp transient crack + resonant exhaust chamber echo + sub-thud
        let pitch = 220.0 * (-t * 50.0).exp() + 45.0;
        let crack = Oscillator::square(t * pitch, 0.35) * 0.50 + Oscillator::sine(t * pitch) * 0.30;
        let pop_noise = noise_gen.next_sample() * (-t * 70.0).exp() * 0.45;
        let resonant = bp_filter.process(crack + pop_noise);
        let filtered = lp_filter.process(crack + resonant * 0.60 + pop_noise * 0.40);

        *sample = soft_saturate(filtered * env, 1.5) * 0.90;
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

    #[test]
    fn test_all_engine_presets_produce_valid_audio() {
        let base_hz = 150.0;
        let presets = [
            ("generic", generate_generic_engine_rpm_band(DEFAULT_SAMPLE_RATE, base_hz)),
            ("sport_gt", generate_sport_gt_rpm_band(DEFAULT_SAMPLE_RATE, base_hz)),
            ("kart", generate_kart_125cc_rpm_band(DEFAULT_SAMPLE_RATE, base_hz)),
            ("f1", generate_f1_v6_rpm_band(DEFAULT_SAMPLE_RATE, base_hz)),
            ("rally", generate_rally_turbo_rpm_band(DEFAULT_SAMPLE_RATE, base_hz)),
        ];

        for (name, wav) in presets {
            assert_eq!(&wav[0..4], b"RIFF", "Preset {name} missing RIFF");
            assert_eq!(&wav[8..12], b"WAVE", "Preset {name} missing WAVE");
            let x = decode_mono_pcm(&wav);
            let mean: f32 = x.iter().sum::<f32>() / x.len() as f32;
            assert!(mean.abs() < 0.05, "Preset {name} has DC offset {mean}");
            let rms = (x.iter().map(|s| s * s).sum::<f32>() / x.len() as f32).sqrt();
            assert!((0.15..0.90).contains(&rms), "Preset {name} RMS {rms} out of range");
            let seam = (x[x.len() - 1] - x[0]).abs();
            assert!(seam < 0.08, "Preset {name} seam click {seam}");
        }
    }
}
