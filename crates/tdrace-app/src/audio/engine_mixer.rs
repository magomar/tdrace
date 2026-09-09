//! Multi-Sample Dynamic Pitch-Matching and Load-Blending Engine Audio Mixer.
//!
//! Eliminates harmonic chord beating by dynamically pitch-matching adjacent bounding
//! sample loops to the exact instantaneous target RPM fundamental before blending,
//! combined with equal-power On-Load vs Off-Load throttle interpolation.

use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use crate::audio::backend::{ActiveSoundHandle, AudioBackend};
use crate::audio::samples::{ArchetypeSampleBank, HIGH_RPM, IDLE_RPM, MID_RPM};

/// Active audio voices for a playing engine sound bank.
pub struct ActiveVoiceGroup {
    pub idle: ActiveSoundHandle,
    pub mid_on: ActiveSoundHandle,
    pub mid_off: ActiveSoundHandle,
    pub high_on: ActiveSoundHandle,
    pub high_off: ActiveSoundHandle,
    pub is_active: bool,
}

impl ActiveVoiceGroup {
    pub fn empty() -> Self {
        Self {
            idle: ActiveSoundHandle::new_empty(),
            mid_on: ActiveSoundHandle::new_empty(),
            mid_off: ActiveSoundHandle::new_empty(),
            high_on: ActiveSoundHandle::new_empty(),
            high_off: ActiveSoundHandle::new_empty(),
            is_active: false,
        }
    }

    /// Stops all active voices with a smooth fade-out.
    pub fn stop(&mut self, fade_duration: Duration) {
        self.idle.stop(fade_duration);
        self.mid_on.stop(fade_duration);
        self.mid_off.stop(fade_duration);
        self.high_on.stop(fade_duration);
        self.high_off.stop(fade_duration);
        self.is_active = false;
    }
}

/// Dynamic pitch-matching multi-sample engine mixer.
pub struct EngineAudioMixer {
    pub bank: ArchetypeSampleBank,
    pub voices: ActiveVoiceGroup,
    pub current_rpm: f32,
    pub current_throttle: f32,
}

impl EngineAudioMixer {
    pub fn new(bank: ArchetypeSampleBank) -> Self {
        Self {
            bank,
            voices: ActiveVoiceGroup::empty(),
            current_rpm: IDLE_RPM,
            current_throttle: 0.0,
        }
    }

    /// Starts all 5 continuous looping voices at zero initial volume if not already running.
    pub fn start_voices(&mut self, backend: &mut AudioBackend) {
        if self.voices.is_active {
            return;
        }

        self.voices.idle = backend.play(&self.bank.idle.sound, 0.0, 1.0);
        self.voices.mid_on = backend.play(&self.bank.mid_on.sound, 0.0, 1.0);
        self.voices.mid_off = backend.play(&self.bank.mid_off.sound, 0.0, 1.0);
        self.voices.high_on = backend.play(&self.bank.high_on.sound, 0.0, 1.0);
        self.voices.high_off = backend.play(&self.bank.high_off.sound, 0.0, 1.0);
        self.voices.is_active = true;
    }

    /// Replaces active sample bank with a new vehicle archetype.
    pub fn set_bank(&mut self, new_bank: ArchetypeSampleBank, backend: &mut AudioBackend) {
        self.voices.stop(Duration::from_millis(25));
        self.bank = new_bank;
        self.start_voices(backend);
    }

    /// Computes dynamic playback rates and equal-power volume weights for all 5 voices.
    /// Returns: `[(rate_idle, vol_idle), (rate_mid_on, vol_mid_on), (rate_mid_off, vol_mid_off), (rate_high_on, vol_high_on), (rate_high_off, vol_high_off)]`
    pub fn compute_voice_parameters(
        rpm: f32,
        throttle: f32,
        master_vol: f32,
    ) -> [(f32, f32); 5] {
        let clamped_rpm = rpm.clamp(600.0, 9500.0);
        let clamped_throttle = throttle.clamp(0.0, 1.0);

        // Equal-power throttle load blend: on-throttle vs off-throttle
        let load_angle = clamped_throttle * FRAC_PI_2;
        let on_weight = load_angle.sin();
        let off_weight = load_angle.cos();

        // Throttle volume factor: wide open intake roar vs idling/coasting
        let load_gain = 0.55 + 0.45 * clamped_throttle;
        let effective_vol = master_vol * load_gain;

        if clamped_rpm <= MID_RPM {
            // Low to Mid RPM region: interpolate between IDLE and MID
            let u = ((clamped_rpm - IDLE_RPM) / (MID_RPM - IDLE_RPM)).clamp(0.0, 1.0);
            let rpm_angle = u * FRAC_PI_2;
            let idle_w = rpm_angle.cos();
            let mid_w = rpm_angle.sin();

            // Dynamic Pitch Matching: both samples scale to the exact target RPM
            let rate_idle = (clamped_rpm / IDLE_RPM).clamp(0.5, 2.5);
            let rate_mid = (clamped_rpm / MID_RPM).clamp(0.5, 2.5);

            let vol_idle = effective_vol * idle_w;
            let vol_mid_on = effective_vol * mid_w * on_weight;
            let vol_mid_off = effective_vol * mid_w * off_weight;

            [
                (rate_idle, vol_idle),
                (rate_mid, vol_mid_on),
                (rate_mid, vol_mid_off),
                (1.0, 0.0), // high_on idle
                (1.0, 0.0), // high_off idle
            ]
        } else {
            // Mid to High/Redline RPM region: interpolate between MID and HIGH
            let u = ((clamped_rpm - MID_RPM) / (HIGH_RPM - MID_RPM)).clamp(0.0, 1.0);
            let rpm_angle = u * FRAC_PI_2;
            let mid_w = rpm_angle.cos();
            let high_w = rpm_angle.sin();

            // Dynamic Pitch Matching: both samples scale to the exact target RPM
            let rate_mid = (clamped_rpm / MID_RPM).clamp(0.5, 2.5);
            let rate_high = (clamped_rpm / HIGH_RPM).clamp(0.5, 2.5);

            let vol_mid_on = effective_vol * mid_w * on_weight;
            let vol_mid_off = effective_vol * mid_w * off_weight;
            let vol_high_on = effective_vol * high_w * on_weight;
            let vol_high_off = effective_vol * high_w * off_weight;

            [
                (1.0, 0.0), // idle closed
                (rate_mid, vol_mid_on),
                (rate_mid, vol_mid_off),
                (rate_high, vol_high_on),
                (rate_high, vol_high_off),
            ]
        }
    }

    /// Updates dynamic playback rate and volume across all active engine voices with sub-frame tweening.
    pub fn update(
        &mut self,
        rpm: f32,
        throttle: f32,
        master_vol: f32,
        backend: &mut AudioBackend,
    ) {
        if !self.voices.is_active {
            self.start_voices(backend);
        }

        self.current_rpm = rpm;
        self.current_throttle = throttle;

        let params = Self::compute_voice_parameters(rpm, throttle, master_vol);
        let tween_dur = Duration::from_millis(8);

        self.voices.idle.set_playback_rate(params[0].0, tween_dur);
        self.voices.idle.set_volume(params[0].1, tween_dur);

        self.voices.mid_on.set_playback_rate(params[1].0, tween_dur);
        self.voices.mid_on.set_volume(params[1].1, tween_dur);

        self.voices.mid_off.set_playback_rate(params[2].0, tween_dur);
        self.voices.mid_off.set_volume(params[2].1, tween_dur);

        self.voices.high_on.set_playback_rate(params[3].0, tween_dur);
        self.voices.high_on.set_volume(params[3].1, tween_dur);

        self.voices.high_off.set_playback_rate(params[4].0, tween_dur);
        self.voices.high_off.set_volume(params[4].1, tween_dur);
    }

    /// Stops playback and mutes all voices immediately.
    pub fn stop(&mut self) {
        self.voices.stop(Duration::from_millis(20));
    }
}
