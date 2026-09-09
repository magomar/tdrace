//! Dynamic Auxiliary Engine Audio FX.
//!
//! Adds mechanical race car excitement layered on top of core combustion:
//! - Straight-cut dog-ring transmission whine (speed-proportional pitch & volume)
//! - Turbocharger compressor spool whistle & atmospheric blow-off valve (BOV) hiss
//! - High-RPM exhaust overrun burble & backfire crackles
//! - 15 Hz ignition-cut rev limiter bounce at redline

use std::time::Duration;

use crate::audio::backend::{ActiveSoundHandle, AudioBackend, SoundData};
use crate::audio::dsp::{
    encode_wav_16bit_mono, soft_saturate, BiquadBandPass, BiquadLowPass, NoiseGenerator,
    Oscillator, DEFAULT_SAMPLE_RATE,
};
use crate::audio::manager::EngineSoundType;

/// Generates a looping straight-cut racing transmission gear whine WAV (~0.25s).
pub fn generate_transmission_whine_loop(sample_rate: u32) -> Vec<u8> {
    let duration = 0.25;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];

    // Reference fundamental: 400 Hz
    let base_hz = 400.0;

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        // Tooth engagement harmonic mix: fundamental + 2nd + 3rd harmonic
        let tooth = Oscillator::sine(t * base_hz) * 0.65
            + Oscillator::sine(t * (base_hz * 2.0)) * 0.25
            + Oscillator::triangle(t * (base_hz * 3.0)) * 0.10;

        *sample = soft_saturate(tooth, 1.1) * 0.80;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates a looping turbocharger compressor spool whistle WAV (~0.25s).
pub fn generate_turbo_spool_loop(sample_rate: u32) -> Vec<u8> {
    let duration = 0.25;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut bp_filter = BiquadBandPass::new(sample_rate, 1800.0, 3.5);
    let mut noise_gen = NoiseGenerator::new(0x700b059001);

    let base_hz = 1200.0;

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let whistle = Oscillator::sine(t * base_hz) * 0.70 + Oscillator::sine(t * (base_hz * 2.0)) * 0.15;
        let air_rush = bp_filter.process(noise_gen.next_sample()) * 0.15;

        *sample = soft_saturate(whistle + air_rush, 1.1) * 0.75;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates a crisp pneumatic blow-off valve (BOV) pressure dump (~0.16s).
pub fn generate_blow_off_valve_sound(sample_rate: u32) -> Vec<u8> {
    let duration = 0.16;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut bp_filter = BiquadBandPass::new(sample_rate, 2800.0, 1.8);
    let mut lp_filter = BiquadLowPass::new(sample_rate, 4500.0, 1.2);
    let mut noise_gen = NoiseGenerator::new(0xb00570ff);

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = if t < 0.008 {
            t / 0.008
        } else {
            (1.0 - (t - 0.008) / (duration - 0.008)).max(0.0).powi(2)
        };

        // Pneumatic pressure hiss + flutter oscillation (24 Hz wastegate flutter)
        let flutter = 1.0 + (t * 24.0 * std::f32::consts::TAU).sin() * 0.35 * (1.0 - t / duration);
        let raw = noise_gen.next_sample() * flutter;
        let filtered = lp_filter.process(bp_filter.process(raw));

        *sample = soft_saturate(filtered * env * 1.8, 1.2) * 0.85;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Generates a sharp exhaust backfire crackle pop (~0.05s).
pub fn generate_exhaust_crackle_sound(sample_rate: u32) -> Vec<u8> {
    let duration = 0.05;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0f32; total_samples];
    let mut lp_filter = BiquadLowPass::new(sample_rate, 1200.0, 1.4);
    let mut noise_gen = NoiseGenerator::new(0xcacacaca);

    for (i, sample) in samples.iter_mut().enumerate().take(total_samples) {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).max(0.0).powi(3);

        let pitch = 180.0 * (-t * 60.0).exp() + 40.0;
        let crack = Oscillator::square(t * pitch, 0.4) * 0.50 + Oscillator::sine(t * pitch) * 0.30;
        let noise = noise_gen.next_sample() * (-t * 80.0).exp() * 0.50;
        let filtered = lp_filter.process(crack + noise);

        *sample = soft_saturate(filtered * env, 1.4) * 0.90;
    }

    encode_wav_16bit_mono(&samples, sample_rate)
}

/// Coordinator for auxiliary engine acoustics.
pub struct AuxiliaryAudioLayer {
    pub whine_sound: SoundData,
    pub spool_sound: SoundData,
    pub bov_sound: SoundData,
    pub crackle_sound: SoundData,

    pub whine_voice: ActiveSoundHandle,
    pub spool_voice: ActiveSoundHandle,

    pub turbo_spool_level: f32,
    pub prev_throttle: f32,
    pub overrun_timer: f32,
    pub limiter_phase: f32,
}

impl AuxiliaryAudioLayer {
    pub fn new(backend: &mut AudioBackend) -> Self {
        let sample_rate = DEFAULT_SAMPLE_RATE;
        let whine_wav = generate_transmission_whine_loop(sample_rate);
        let spool_wav = generate_turbo_spool_loop(sample_rate);
        let bov_wav = generate_blow_off_valve_sound(sample_rate);
        let crackle_wav = generate_exhaust_crackle_sound(sample_rate);

        let whine_sound = SoundData::from_bytes(&whine_wav, true).unwrap();
        let spool_sound = SoundData::from_bytes(&spool_wav, true).unwrap();
        let bov_sound = SoundData::from_bytes(&bov_wav, false).unwrap();
        let crackle_sound = SoundData::from_bytes(&crackle_wav, false).unwrap();

        let whine_voice = backend.play(&whine_sound, 0.0, 1.0);
        let spool_voice = backend.play(&spool_sound, 0.0, 1.0);

        Self {
            whine_sound,
            spool_sound,
            bov_sound,
            crackle_sound,
            whine_voice,
            spool_voice,
            turbo_spool_level: 0.0,
            prev_throttle: 0.0,
            overrun_timer: 0.0,
            limiter_phase: 0.0,
        }
    }

    /// Updates auxiliary acoustic layers.
    /// Returns: `limiter_volume_mod` [0.35..1.0] for the core engine combustion mixer.
    pub fn update(
        &mut self,
        speed_abs: f32,
        gear: usize,
        rpm: f32,
        throttle: f32,
        engine_type: EngineSoundType,
        dt: f32,
        master_vol: f32,
        backend: &mut AudioBackend,
    ) -> f32 {
        let has_turbo = matches!(engine_type, EngineSoundType::F1V6Turbo | EngineSoundType::RallyTurbo);
        let tween_dur = Duration::from_millis(10);

        // 1. Straight-Cut Transmission Gear Whine
        let has_whine = !matches!(engine_type, EngineSoundType::Kart125cc);
        if has_whine && speed_abs > 1.0 && gear > 0 {
            // Whine frequency scales with vehicle speed: 400 Hz base -> 2,200 Hz top speed
            let target_freq = (400.0 + speed_abs * 35.0).clamp(400.0, 2600.0);
            let whine_rate = (target_freq / 400.0).clamp(0.5, 3.5);
            let gear_factor = 0.4 + 0.15 * (gear as f32).min(5.0);
            let speed_factor = (speed_abs / 45.0).clamp(0.0, 1.0);
            let whine_vol = master_vol * speed_factor * gear_factor * 0.22;

            self.whine_voice.set_playback_rate(whine_rate, tween_dur);
            self.whine_voice.set_volume(whine_vol, tween_dur);
        } else {
            self.whine_voice.set_volume(0.0, tween_dur);
        }

        // 2. Turbocharger Spool & Blow-Off Valve (BOV)
        if has_turbo {
            let target_spool = if throttle > 0.15 && rpm > 2800.0 {
                let rpm_factor = ((rpm - 2800.0) / 4500.0).clamp(0.0, 1.0);
                throttle * rpm_factor
            } else {
                0.0
            };

            let spool_speed = if target_spool > self.turbo_spool_level { 3.5 } else { 2.0 };
            self.turbo_spool_level += (target_spool - self.turbo_spool_level) * (dt * spool_speed).min(1.0);

            // Turbo spool pitch and volume
            let spool_rate = (0.7 + self.turbo_spool_level * 1.6).clamp(0.5, 2.5);
            let spool_vol = master_vol * self.turbo_spool_level * 0.24;
            self.spool_voice.set_playback_rate(spool_rate, tween_dur);
            self.spool_voice.set_volume(spool_vol, tween_dur);

            // Blow-Off Valve trigger on sudden throttle lift-off under boost
            let throttle_drop = self.prev_throttle - throttle;
            if throttle_drop > 0.35 && self.turbo_spool_level > 0.40 {
                let bov_gain = master_vol * (self.turbo_spool_level * 0.85).clamp(0.3, 0.9);
                backend.play(&self.bov_sound, bov_gain, 1.0);
                self.turbo_spool_level *= 0.25; // Pressure dumped
            }
        } else {
            self.spool_voice.set_volume(0.0, tween_dur);
            self.turbo_spool_level = 0.0;
        }
        self.prev_throttle = throttle;

        // 3. High-RPM Overrun Pops & Crackles
        let allows_crackle = matches!(
            engine_type,
            EngineSoundType::NascarV8 | EngineSoundType::RallyTurbo | EngineSoundType::SportGT
        );
        if allows_crackle && throttle < 0.08 && rpm > 5600.0 {
            self.overrun_timer += dt;
            if self.overrun_timer > 0.09 {
                self.overrun_timer = 0.0;
                let pop_gain = master_vol * 0.55;
                backend.play(&self.crackle_sound, pop_gain, 0.95 + (rpm * 0.0001).fract() * 0.1);
            }
        } else {
            self.overrun_timer = 0.0;
        }

        // 4. Rev Limiter Ignition Cut (15 Hz oscillation at redline)
        let max_rpm = 9300.0;
        if rpm >= max_rpm - 300.0 && throttle > 0.5 {
            self.limiter_phase = (self.limiter_phase + dt * 15.0).fract();
            if self.limiter_phase < 0.5 {
                1.0
            } else {
                0.35 // Hard ignition cut phase
            }
        } else {
            self.limiter_phase = 0.0;
            1.0
        }
    }

    /// Stops all continuous auxiliary audio loops.
    pub fn stop(&mut self) {
        self.whine_voice.stop(Duration::from_millis(15));
        self.spool_voice.stop(Duration::from_millis(15));
        self.turbo_spool_level = 0.0;
    }
}
