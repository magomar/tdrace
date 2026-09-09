//! High-Fidelity Engine Audio Sample Bank Curator and Seamless Loop Generator.
//!
//! Generates and manages steady-state sampled engine audio loops across vehicle archetypes:
//! - Idle (~950 RPM steady rumble)
//! - Mid On-Load (~3,600 RPM high manifold intake gulp + combustion bark)
//! - Mid Off-Load (~3,600 RPM deceleration vacuum + overrun burble)
//! - High On-Load (~7,200 RPM screaming top-end power delivery)
//! - High Off-Load (~7,200 RPM engine braking rasp)

use std::path::{Path, PathBuf};

use crate::audio::backend::SoundData;
use crate::audio::dsp::{
    encode_wav_16bit_mono, soft_saturate, waveshape_engine, BiquadBandPass, BiquadLowPass,
    NoiseGenerator, Oscillator, DEFAULT_SAMPLE_RATE,
};
use crate::audio::manager::EngineSoundType;
use crate::audio::sfx::EngineSoundConfig;

/// Standard engine RPM breakpoints for multi-sample loop banks.
pub const IDLE_RPM: f32 = 950.0;
pub const MID_RPM: f32 = 3600.0;
pub const HIGH_RPM: f32 = 7200.0;

/// A single steady-state engine audio sample loop tagged with its exact fundamental RPM and load state.
#[derive(Clone)]
pub struct EngineSamplePoint {
    pub rpm: f32,
    pub is_load: bool,
    pub sound: SoundData,
    pub wav_bytes: Vec<u8>,
}

/// Complete 5-point steady-state sample library for a specific vehicle archetype.
#[derive(Clone)]
pub struct ArchetypeSampleBank {
    pub engine_type: EngineSoundType,
    pub idle: EngineSamplePoint,
    pub mid_on: EngineSamplePoint,
    pub mid_off: EngineSamplePoint,
    pub high_on: EngineSamplePoint,
    pub high_off: EngineSamplePoint,
}

impl ArchetypeSampleBank {
    /// Generates pristine, click-free seamless loop sample points in memory for the given archetype.
    pub fn generate(engine_type: EngineSoundType, sample_rate: u32) -> Self {
        let config = match engine_type {
            EngineSoundType::Generic => EngineSoundConfig::generic(),
            EngineSoundType::SportGT => EngineSoundConfig::sport_gt(),
            EngineSoundType::Kart125cc => EngineSoundConfig::kart_125cc(),
            EngineSoundType::F1V6Turbo => EngineSoundConfig::f1_v6_turbo(),
            EngineSoundType::RallyTurbo => EngineSoundConfig::rally_turbo(),
            EngineSoundType::NascarV8 => EngineSoundConfig::nascar_v8(),
        };

        let idle_wav = generate_steady_engine_loop(sample_rate, IDLE_RPM, false, &config);
        let mid_on_wav = generate_steady_engine_loop(sample_rate, MID_RPM, true, &config);
        let mid_off_wav = generate_steady_engine_loop(sample_rate, MID_RPM, false, &config);
        let high_on_wav = generate_steady_engine_loop(sample_rate, HIGH_RPM, true, &config);
        let high_off_wav = generate_steady_engine_loop(sample_rate, HIGH_RPM, false, &config);

        let idle_snd = SoundData::from_bytes(&idle_wav, true).unwrap_or_else(|_| SoundData::from_bytes(&[], true).unwrap());
        let mid_on_snd = SoundData::from_bytes(&mid_on_wav, true).unwrap_or_else(|_| SoundData::from_bytes(&[], true).unwrap());
        let mid_off_snd = SoundData::from_bytes(&mid_off_wav, true).unwrap_or_else(|_| SoundData::from_bytes(&[], true).unwrap());
        let high_on_snd = SoundData::from_bytes(&high_on_wav, true).unwrap_or_else(|_| SoundData::from_bytes(&[], true).unwrap());
        let high_off_snd = SoundData::from_bytes(&high_off_wav, true).unwrap_or_else(|_| SoundData::from_bytes(&[], true).unwrap());

        Self {
            engine_type,
            idle: EngineSamplePoint {
                rpm: IDLE_RPM,
                is_load: false,
                sound: idle_snd,
                wav_bytes: idle_wav,
            },
            mid_on: EngineSamplePoint {
                rpm: MID_RPM,
                is_load: true,
                sound: mid_on_snd,
                wav_bytes: mid_on_wav,
            },
            mid_off: EngineSamplePoint {
                rpm: MID_RPM,
                is_load: false,
                sound: mid_off_snd,
                wav_bytes: mid_off_wav,
            },
            high_on: EngineSamplePoint {
                rpm: HIGH_RPM,
                is_load: true,
                sound: high_on_snd,
                wav_bytes: high_on_wav,
            },
            high_off: EngineSamplePoint {
                rpm: HIGH_RPM,
                is_load: false,
                sound: high_off_snd,
                wav_bytes: high_off_wav,
            },
        }
    }

    /// Subdirectory name under assets/audio/engines/
    pub fn slug(engine_type: EngineSoundType) -> &'static str {
        match engine_type {
            EngineSoundType::Generic => "generic",
            EngineSoundType::SportGT => "sport_gt",
            EngineSoundType::Kart125cc => "kart_125cc",
            EngineSoundType::F1V6Turbo => "f1_v6_turbo",
            EngineSoundType::RallyTurbo => "rally_turbo",
            EngineSoundType::NascarV8 => "nascar_v8",
        }
    }

    /// Saves all 5 steady loops to disk under the specified base assets directory.
    pub fn save_to_dir(&self, base_dir: &Path) -> std::io::Result<()> {
        let dir = base_dir.join(Self::slug(self.engine_type));
        std::fs::create_dir_all(&dir)?;

        std::fs::write(dir.join("idle.wav"), &self.idle.wav_bytes)?;
        std::fs::write(dir.join("mid_on.wav"), &self.mid_on.wav_bytes)?;
        std::fs::write(dir.join("mid_off.wav"), &self.mid_off.wav_bytes)?;
        std::fs::write(dir.join("high_on.wav"), &self.high_on.wav_bytes)?;
        std::fs::write(dir.join("high_off.wav"), &self.high_off.wav_bytes)?;
        Ok(())
    }

    /// Loads loops from disk if available, otherwise generates them dynamically and caches to disk.
    pub fn load_or_generate(engine_type: EngineSoundType, base_dir: &Path) -> Self {
        let dir = base_dir.join(Self::slug(engine_type));
        let idle_path = dir.join("idle.wav");
        let mid_on_path = dir.join("mid_on.wav");
        let mid_off_path = dir.join("mid_off.wav");
        let high_on_path = dir.join("high_on.wav");
        let high_off_path = dir.join("high_off.wav");

        let read_loop = |path: &PathBuf, rpm: f32, is_load: bool| -> Option<EngineSamplePoint> {
            let bytes = std::fs::read(path).ok()?;
            let sound = SoundData::from_bytes(&bytes, true).ok()?;
            Some(EngineSamplePoint {
                rpm,
                is_load,
                sound,
                wav_bytes: bytes,
            })
        };

        if let (Some(idle), Some(mid_on), Some(mid_off), Some(high_on), Some(high_off)) = (
            read_loop(&idle_path, IDLE_RPM, false),
            read_loop(&mid_on_path, MID_RPM, true),
            read_loop(&mid_off_path, MID_RPM, false),
            read_loop(&high_on_path, HIGH_RPM, true),
            read_loop(&high_off_path, HIGH_RPM, false),
        ) {
            Self {
                engine_type,
                idle,
                mid_on,
                mid_off,
                high_on,
                high_off,
            }
        } else {
            let bank = Self::generate(engine_type, DEFAULT_SAMPLE_RATE);
            let _ = bank.save_to_dir(base_dir);
            bank
        }
    }
}

/// Generates a perfectly seamless, integer-cycle steady engine audio loop.
pub fn generate_steady_engine_loop(
    sample_rate: u32,
    rpm: f32,
    on_throttle: bool,
    config: &EngineSoundConfig,
) -> Vec<u8> {
    let crank_hz = (rpm / 60.0).max(5.0);
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
    let target_duration = 0.45;
    let num_cycles = (target_duration * cycle_hz).round().max(1.0);
    let duration = num_cycles / cycle_hz;
    let total_samples = (duration * sample_rate as f32).round() as usize;

    // Filters: Master low-pass + exhaust pipe formants + intake noise filter
    let cutoff = if on_throttle {
        (firing_hz * 6.5).clamp(600.0, 8000.0)
    } else {
        (firing_hz * 4.0).clamp(400.0, 5000.0)
    };
    let mut lp_filter = BiquadLowPass::new(sample_rate, cutoff, 0.95);
    let mut bp_formant1 = BiquadBandPass::new(sample_rate, config.formant_f1_hz, config.formant_q);
    let mut bp_formant2 = BiquadBandPass::new(sample_rate, config.formant_f2_hz, config.formant_q);
    let mut bp_intake = BiquadBandPass::new(
        sample_rate,
        (600.0 + firing_hz * 1.5).clamp(300.0, 3500.0),
        1.4,
    );

    let mut noise_gen = NoiseGenerator::new(0x53414d504c455321);

    // Overlap tail length for seamless crossfading
    let xfade_len = 256.min(total_samples / 4);
    let pre_roll = (0.05 * sample_rate as f32).round() as usize;
    let gen_len = pre_roll + total_samples + xfade_len;

    let load_scale = if on_throttle { 1.0 } else { 0.35 };
    let sat_drive = if on_throttle {
        config.saturation_drive
    } else {
        (config.saturation_drive * 0.85).max(1.0)
    };

    let mut samples = vec![0.0f32; gen_len];
    for (i, sample) in samples.iter_mut().enumerate() {
        let t = i as f32 / sample_rate as f32;

        // 1. Physical Cylinder Combustion Pressure Pulses
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
        combustion *= if on_throttle { 1.0 } else { 0.65 };

        // 2. Crankshaft Sub-Harmonics & Idle/Overrun Lumping
        let crank_phase = t * crank_hz;
        let sub = (Oscillator::sine(crank_phase * 0.5) * 0.70 + Oscillator::saw(crank_phase) * 0.30)
            * config.crank_lumpiness;

        // 3. Intake Induction Air Rush (Wide open on throttle, muffled on decel)
        let intake_noise = bp_intake.process(noise_gen.next_sample());
        let intake_mod = (Oscillator::saw(t * firing_hz) * 0.5 + 0.5)
            * intake_noise
            * config.intake_growl_intensity
            * load_scale;

        // 4. Turbo Compressor Spool Whine (On throttle under boost)
        let turbo = if config.turbo_whine_level > 0.001 && on_throttle {
            let turbo_freq = (crank_hz * 6.5).clamp(800.0, 4800.0);
            Oscillator::sine(t * turbo_freq) * config.turbo_whine_level * 0.10
        } else {
            0.0
        };

        // 5. Mechanical Valvetrain & Cam Chatter (Prominent on decel)
        let valvetrain = if config.mechanical_buzz > 0.001 {
            let buzz_gain = if on_throttle {
                config.mechanical_buzz * 0.12
            } else {
                config.mechanical_buzz * 0.22
            };
            Oscillator::saw(t * (firing_hz * 2.0)) * buzz_gain
        } else {
            0.0
        };

        // Raw acoustic mixture
        let raw = combustion + sub + intake_mod + turbo + valvetrain;

        // 6. Dual-Stage Exhaust Formant Acoustic Resonators
        let form1 = bp_formant1.process(raw);
        let form2 = bp_formant2.process(raw);
        let shaped_exhaust = raw * 0.70 + form1 * 0.35 + form2 * 0.25;

        // 7. Master Low-Pass & Non-Linear Wave-Shaping
        let filtered = lp_filter.process(shaped_exhaust);
        let saturated = waveshape_engine(filtered, sat_drive, 0.30);

        *sample = soft_saturate(saturated * 2.5, 1.0) * 0.90;
    }

    // Extract steady-state loop buffer and overlap tail
    let mut pcm_samples = samples[pre_roll..(pre_roll + total_samples)].to_vec();
    let overlap_tail = &samples[(pre_roll + total_samples)..(pre_roll + total_samples + xfade_len)];

    // Equal-power crossfade overlap tail for seamless wrapping
    for k in 0..xfade_len {
        let w = k as f32 / xfade_len as f32;
        pcm_samples[k] = pcm_samples[k] * w + overlap_tail[k] * (1.0 - w);
    }

    // DC-blocking zero-mean centering
    let mean: f32 = pcm_samples.iter().sum::<f32>() / pcm_samples.len().max(1) as f32;
    for s in &mut pcm_samples {
        *s -= mean;
    }

    encode_wav_16bit_mono(&pcm_samples, sample_rate)
}
