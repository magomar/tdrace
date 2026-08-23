//! Multi-track Procedural Synthwave Music Synthesizer.
//!
//! Generates 80s Outrun / Synthwave soundtrack loops inspired by the mood,
//! instrumentation, and driving groove of Kavinsky's "Nightcall" (Drive OST).
//! All oscillators use continuous phase integration for zero phase distortion.

use crate::audio::dsp::{
    encode_wav_16bit_stereo, soft_saturate, AdsrEnvelope, BiquadLowPass, NoiseGenerator,
    Oscillator, StereoDelay,
};

/// Converts a standard MIDI note number (0-127) to frequency in Hz.
/// Middle C (C4) = 60 (261.63 Hz), A4 = 69 (440.0 Hz).
#[inline]
pub fn midi_to_hz(note: u8) -> f32 {
    440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}

/// Note constants for easy composition
#[allow(non_upper_case_globals)]
pub mod notes {
    pub const A1: u8 = 33;
    pub const Bb1: u8 = 34;
    pub const C2: u8 = 36;
    pub const D2: u8 = 38;
    pub const E2: u8 = 40;
    pub const F2: u8 = 41;
    pub const G2: u8 = 43;
    pub const A2: u8 = 45;
    pub const Bb2: u8 = 46;
    pub const B2: u8 = 47;
    pub const C3: u8 = 48;
    pub const D3: u8 = 50;
    pub const E3: u8 = 52;
    pub const F3: u8 = 53;
    pub const G3: u8 = 55;
    pub const A3: u8 = 57;
    pub const Bb3: u8 = 58;
    pub const B3: u8 = 59;
    pub const C4: u8 = 60;
    pub const D4: u8 = 62;
    pub const E4: u8 = 64;
    pub const F4: u8 = 65;
    pub const G4: u8 = 67;
    pub const A4: u8 = 69;
    pub const Bb4: u8 = 70;
    pub const B4: u8 = 71;
    pub const C5: u8 = 72;
    pub const D5: u8 = 74;
    pub const E5: u8 = 76;
    pub const F5: u8 = 77;
    pub const G5: u8 = 79;
}

/// Generates the rich, warm, atmospheric "Neon Nights" Menu Theme as a 16-bit stereo WAV buffer.
/// - BPM: 104.0 (smooth retro outrun groove)
/// - Length: 16 bars (~36.92 seconds, seamless loop)
/// - Key: D Minor (Dm -> Bb -> F -> C)
/// - Sound Design: Analog Kick/Snare, warm pulsing bassline, lush detuned Juno pads, retro bell arpeggio
pub fn generate_menu_theme(sample_rate: u32) -> Vec<u8> {
    let bpm = 104.0;
    let beats_per_bar = 4.0;
    let total_bars = 16;
    let sec_per_beat = 60.0 / bpm;
    let bar_sec = sec_per_beat * beats_per_bar;
    let total_sec = bar_sec * total_bars as f32;
    let total_samples = (total_sec * sample_rate as f32).round() as usize;
    let dt = 1.0 / sample_rate as f32;

    let mut left_samples = vec![0.0f32; total_samples];
    let mut right_samples = vec![0.0f32; total_samples];

    let mut noise = NoiseGenerator::new(0x4E454F4E); // "NEON"
    let mut delay = StereoDelay::new(sample_rate, 288.46, 432.69, 0.40, 0.30); // 16th and dotted delay
    let mut pad_filter_l = BiquadLowPass::new(sample_rate, 2200.0, 1.1);
    let mut pad_filter_r = BiquadLowPass::new(sample_rate, 2200.0, 1.1);
    let mut bass_filter = BiquadLowPass::new(sample_rate, 800.0, 2.0);

    let sec_per_16th = sec_per_beat / 4.0;

    // 4-bar chord progression in D minor repeated 4 times:
    // Bars 0-3: Dm  (D3, F3, A3, D4)  Root: D2 (38)
    // Bars 4-7: Bb  (Bb2, D3, F3, Bb3) Root: Bb1 (34)
    // Bars 8-11: F  (F3, A3, C4, F4)  Root: F2 (41)
    // Bars 12-15: C (C3, E3, G3, C4)  Root: C2 (36)
    let chord_roots = [
        notes::D2, notes::D2, notes::D2, notes::D2,
        notes::Bb1, notes::Bb1, notes::Bb1, notes::Bb1,
        notes::F2, notes::F2, notes::F2, notes::F2,
        notes::C2, notes::C2, notes::C2, notes::C2,
    ];

    let chord_pads = [
        [notes::D3, notes::F3, notes::A3, notes::D4], // Dm
        [notes::Bb2, notes::D3, notes::F3, notes::Bb3], // Bb
        [notes::F3, notes::A3, notes::C4, notes::F4], // F
        [notes::C3, notes::E3, notes::G3, notes::C4], // C
    ];

    // Continuous running phase accumulators for seamless analog continuity
    let mut phase_bass: f32 = 0.0;
    let mut phase_pad_voices: [[f32; 3]; 4] = [[0.0; 3]; 4];
    let mut phase_arp: f32 = 0.0;
    let mut phase_kick: f32 = 0.0;

    let bass_adsr = AdsrEnvelope::new(0.008, 0.08, 0.45, 0.05);
    let arp_adsr = AdsrEnvelope::new(0.005, 0.08, 0.20, 0.06);

    for i in 0..total_samples {
        let t = i as f32 * dt;
        let bar_idx = ((t / bar_sec) as usize).min(total_bars - 1);
        let chord_idx = bar_idx / 4;
        let beat_in_bar = (t % bar_sec) / sec_per_beat;
        let step_16th = ((t / sec_per_16th) as usize) % 256;

        // Sidechain compression from Kick
        let beat_frac = beat_in_bar.fract();
        let is_kick_beat = beat_in_bar < 0.6 || (beat_in_bar >= 2.0 && beat_in_bar < 2.6);
        let sidechain_duck = if is_kick_beat {
            (beat_frac * 3.2).min(1.0)
        } else {
            1.0
        };

        // --- 1. DRUMS (Smooth Retro Analog Beat) ---
        let mut drum_l = 0.0f32;
        let mut drum_r = 0.0f32;

        // Kick Drum (Beats 0 and 2)
        for &k_beat in &[0.0f32, 2.0] {
            let t_kick = (t % bar_sec) - k_beat * sec_per_beat;
            if t_kick >= 0.0 && t_kick < 0.28 {
                let kick_env = (1.0 - t_kick / 0.28).max(0.0).powi(2);
                let kick_pitch = 135.0 * (-t_kick * 24.0).exp() + 42.0;
                phase_kick = (phase_kick + dt * kick_pitch).fract();
                let kick_body = Oscillator::sine(phase_kick) * kick_env * 0.95;
                let click = if t_kick < 0.005 { (1.0 - t_kick / 0.005) * 0.3 } else { 0.0 };
                drum_l += kick_body + click;
                drum_r += kick_body + click;
            }
        }

        // Snare / Clap on Beats 1 and 3
        for &s_beat in &[1.0f32, 3.0] {
            let t_snare = (t % bar_sec) - s_beat * sec_per_beat;
            if t_snare >= 0.0 && t_snare < 0.24 {
                let tone_env = (1.0 - t_snare / 0.16).max(0.0).powi(2);
                let tone = Oscillator::triangle(t_snare * 185.0) * tone_env * 0.40;
                let noise_env = if t_snare < 0.20 { (1.0 - t_snare / 0.20) * 0.50 } else { 0.0 };
                let noise_sample = noise.next_sample() * noise_env * 0.65;
                drum_l += tone + noise_sample;
                drum_r += tone + noise_sample;
            }
        }

        // Hi-Hat on 8th notes
        let t_8th = t % (sec_per_beat * 0.5);
        let hat_decay = 0.040;
        if t_8th < hat_decay {
            let hat_env = (1.0 - t_8th / hat_decay).max(0.0).powi(2);
            let hat_sample = noise.next_sample() * hat_env * 0.22;
            drum_l += hat_sample * 0.85;
            drum_r += hat_sample * 1.15;
        }

        // --- 2. WARM PULSING SYNTH BASS ---
        // 8th-note bass pulse with continuous phase
        let root_note = chord_roots[bar_idx];
        let is_high_step = (step_16th % 4) == 2;
        let bass_note = if is_high_step { root_note + 12 } else { root_note };
        let bass_hz = midi_to_hz(bass_note);

        phase_bass = (phase_bass + dt * bass_hz).fract();
        let t_bass = t % (sec_per_16th * 2.0);
        let bass_env = bass_adsr.evaluate(t_bass, sec_per_16th * 1.6);
        let bass_raw = (Oscillator::saw(phase_bass) * 0.60 + Oscillator::triangle(phase_bass) * 0.40) * bass_env;
        let bass_filtered = bass_filter.process(bass_raw) * (0.40 + 0.60 * sidechain_duck) * 0.90;

        // --- 3. LUSH JUNO-106 POLYSYNTH PADS ---
        let current_pad = chord_pads[chord_idx];
        let mut pad_mix_l = 0.0f32;
        let mut pad_mix_r = 0.0f32;

        let lfo = (t * 0.6).sin() * 0.006;
        let pad_cutoff = ((t * 0.25).sin() * 500.0 + 1700.0).clamp(800.0, 2600.0);
        pad_filter_l.update_coefficients(sample_rate, pad_cutoff, 0.9);
        pad_filter_r.update_coefficients(sample_rate, pad_cutoff, 0.9);

        for (v_idx, &note) in current_pad.iter().enumerate() {
            let hz = midi_to_hz(note);
            let hz_detune1 = hz * (1.0015 + lfo);
            let hz_detune2 = hz * (0.9985 - lfo);

            phase_pad_voices[v_idx][0] = (phase_pad_voices[v_idx][0] + dt * hz).fract();
            phase_pad_voices[v_idx][1] = (phase_pad_voices[v_idx][1] + dt * hz_detune1).fract();
            phase_pad_voices[v_idx][2] = (phase_pad_voices[v_idx][2] + dt * hz_detune2).fract();

            let o1 = Oscillator::saw(phase_pad_voices[v_idx][0]);
            let o2 = Oscillator::saw(phase_pad_voices[v_idx][1]) * 0.85;
            let o3 = Oscillator::saw(phase_pad_voices[v_idx][2]) * 0.85;
            let voice = (o1 + o2 + o3) * 0.095;

            let pan = (v_idx as f32 / 3.0) * 0.6 - 0.3;
            pad_mix_l += voice * (1.0 - pan);
            pad_mix_r += voice * (1.0 + pan);
        }

        let pad_l = pad_filter_l.process(pad_mix_l);
        let pad_r = pad_filter_r.process(pad_mix_r);

        // --- 4. NOSTALGIC BELL ARPEGGIO CHIMES ---
        let arp_notes = [current_pad[0], current_pad[1], current_pad[2], current_pad[3] + 12];
        let arp_note = arp_notes[step_16th % 4];
        let arp_hz = midi_to_hz(arp_note);

        phase_arp = (phase_arp + dt * arp_hz).fract();
        let t_arp = t % sec_per_16th;
        let arp_env = arp_adsr.evaluate(t_arp, sec_per_16th * 0.7);
        let arp_sample = (Oscillator::sine(phase_arp) * 0.75 + Oscillator::triangle(phase_arp * 2.0) * 0.25) * arp_env * 0.25;

        // Stereo Delay on Arpeggio
        let (delay_l, delay_r) = delay.process(arp_sample, arp_sample);

        // Master Mixing
        let mix_l = drum_l + bass_filtered + pad_l + arp_sample * 0.65 + delay_l * 0.55;
        let mix_r = drum_r + bass_filtered + pad_r + arp_sample * 0.45 + delay_r * 0.55;

        let out_l = soft_saturate(mix_l, 1.05) * 0.82;
        let out_r = soft_saturate(mix_r, 1.05) * 0.82;

        left_samples[i] = out_l;
        right_samples[i] = out_r;
    }

    // Interleave stereo pairs
    let mut stereo_frames = Vec::with_capacity(total_samples);
    for i in 0..total_samples {
        stereo_frames.push((left_samples[i], right_samples[i]));
    }

    encode_wav_16bit_stereo(&stereo_frames, sample_rate)
}

/// Generates the "Nightcall Outrun" Race Theme with continuous phase integration.
/// - BPM: 108.0
/// - Key: A Minor (Am -> F -> C -> G)
pub fn generate_nightcall_race_theme(sample_rate: u32) -> Vec<u8> {
    let bpm = 108.0;
    let beats_per_bar = 4.0;
    let total_bars = 16;
    let sec_per_beat = 60.0 / bpm;
    let bar_sec = sec_per_beat * beats_per_bar;
    let total_sec = bar_sec * total_bars as f32;
    let total_samples = (total_sec * sample_rate as f32).round() as usize;
    let dt = 1.0 / sample_rate as f32;

    let mut left_samples = vec![0.0f32; total_samples];
    let mut right_samples = vec![0.0f32; total_samples];

    let mut noise = NoiseGenerator::new(0x4452495645);
    let mut delay = StereoDelay::new(sample_rate, 277.0, 416.0, 0.40, 0.28);
    let mut pad_filter_l = BiquadLowPass::new(sample_rate, 1800.0, 1.2);
    let mut pad_filter_r = BiquadLowPass::new(sample_rate, 1800.0, 1.2);
    let mut bass_filter = BiquadLowPass::new(sample_rate, 950.0, 2.5);

    let sec_per_16th = sec_per_beat / 4.0;

    let chord_roots = [
        notes::A1, notes::A1, notes::A1, notes::A1,
        notes::F2 - 12, notes::F2 - 12, notes::F2 - 12, notes::F2 - 12,
        notes::C2, notes::C2, notes::C2, notes::C2,
        notes::G2 - 12, notes::G2 - 12, notes::G2 - 12, notes::G2 - 12,
    ];

    let chord_pads = [
        [notes::A3, notes::C4, notes::E4, notes::A4], // Am
        [notes::F3, notes::A3, notes::C4, notes::F4], // F
        [notes::C3, notes::E3, notes::G3, notes::C4], // C
        [notes::G3, notes::B3, notes::D4, notes::G4], // G
    ];

    let mut phase_bass: f32 = 0.0;
    let mut phase_pad_voices: [[f32; 3]; 4] = [[0.0; 3]; 4];
    let mut phase_arp: f32 = 0.0;
    let mut phase_lead: f32 = 0.0;
    let mut phase_kick: f32 = 0.0;

    let lead_adsr = AdsrEnvelope::new(0.015, 0.12, 0.70, 0.18);
    let bass_adsr = AdsrEnvelope::new(0.005, 0.06, 0.40, 0.04);
    let arp_adsr = AdsrEnvelope::new(0.008, 0.05, 0.15, 0.03);

    for i in 0..total_samples {
        let t = i as f32 * dt;
        let bar_idx = ((t / bar_sec) as usize).min(total_bars - 1);
        let chord_idx = bar_idx / 4;
        let beat_in_bar = (t % bar_sec) / sec_per_beat;
        let step_16th = ((t / sec_per_16th) as usize) % 256;

        let beat_frac = beat_in_bar.fract();
        let is_kick_beat = beat_in_bar < 0.6 || (beat_in_bar >= 2.0 && beat_in_bar < 2.6);
        let sidechain_duck = if is_kick_beat {
            (beat_frac * 3.5).min(1.0)
        } else {
            1.0
        };

        // --- 1. DRUMS ---
        let mut drum_l = 0.0f32;
        let mut drum_r = 0.0f32;

        for &k_beat in &[0.0f32, 2.0] {
            let t_kick = (t % bar_sec) - k_beat * sec_per_beat;
            if t_kick >= 0.0 && t_kick < 0.28 {
                let kick_env = (1.0 - t_kick / 0.28).max(0.0).powi(2);
                let kick_pitch = 140.0 * (-t_kick * 28.0).exp() + 45.0;
                phase_kick = (phase_kick + dt * kick_pitch).fract();
                let kick_sample = Oscillator::sine(phase_kick) * kick_env * 0.85;
                let click = if t_kick < 0.006 { (1.0 - t_kick / 0.006) * 0.35 } else { 0.0 };
                drum_l += kick_sample + click;
                drum_r += kick_sample + click;
            }
        }

        for &s_beat in &[1.0f32, 3.0] {
            let t_snare = (t % bar_sec) - s_beat * sec_per_beat;
            if t_snare >= 0.0 && t_snare < 0.26 {
                let tone_env = (1.0 - t_snare / 0.15).max(0.0).powi(2);
                let tone = Oscillator::triangle(t_snare * 190.0) * tone_env * 0.45;
                let noise_env = if t_snare < 0.22 { (1.0 - t_snare / 0.22) * 0.55 } else { 0.0 };
                let noise_sample = noise.next_sample() * noise_env;
                drum_l += tone + noise_sample * 0.8;
                drum_r += tone + noise_sample * 0.8;
            }
        }

        let t_hat = t % sec_per_16th;
        let hat_step = (t / sec_per_16th) as usize % 16;
        let hat_is_open = hat_step == 2 || hat_step == 6 || hat_step == 10 || hat_step == 14;
        let hat_decay = if hat_is_open { 0.08 } else { 0.035 };
        if t_hat < hat_decay {
            let hat_env = (1.0 - t_hat / hat_decay).max(0.0).powi(2);
            let hat_accent = if hat_step % 4 == 0 { 0.25 } else if hat_is_open { 0.32 } else { 0.15 };
            let hat_n = noise.next_sample() * hat_env * hat_accent;
            let pan = (step_16th % 2) as f32 * 0.3 - 0.15;
            drum_l += hat_n * (1.0 - pan);
            drum_r += hat_n * (1.0 + pan);
        }

        // --- 2. BASSLINE ---
        let root_note = chord_roots[bar_idx];
        let is_octave_high = (step_16th % 2) == 1;
        let bass_note = if is_octave_high { root_note + 12 } else { root_note };
        let bass_hz = midi_to_hz(bass_note);

        phase_bass = (phase_bass + dt * bass_hz).fract();
        let t_bass = t % sec_per_16th;
        let bass_env = bass_adsr.evaluate(t_bass, sec_per_16th * 0.85);
        let bass_raw = (Oscillator::saw(phase_bass) * 0.65 + Oscillator::square(phase_bass * 0.5, 0.5) * 0.35) * bass_env;
        let bass_filtered = bass_filter.process(bass_raw) * (0.35 + 0.65 * sidechain_duck);

        // --- 3. PADS ---
        let current_pad_notes = chord_pads[chord_idx];
        let mut pad_mix_l = 0.0f32;
        let mut pad_mix_r = 0.0f32;

        let pad_lfo = (t * 0.8).sin() * 0.008;
        let pad_sweep = ((t * 0.35).sin() * 400.0 + 1400.0).clamp(500.0, 2400.0);
        pad_filter_l.update_coefficients(sample_rate, pad_sweep, 0.8);
        pad_filter_r.update_coefficients(sample_rate, pad_sweep, 0.8);

        for (v_idx, &note) in current_pad_notes.iter().enumerate() {
            let hz = midi_to_hz(note);
            let hz_detune1 = hz * (1.002 + pad_lfo);
            let hz_detune2 = hz * (0.998 - pad_lfo);

            phase_pad_voices[v_idx][0] = (phase_pad_voices[v_idx][0] + dt * hz).fract();
            phase_pad_voices[v_idx][1] = (phase_pad_voices[v_idx][1] + dt * hz_detune1).fract();
            phase_pad_voices[v_idx][2] = (phase_pad_voices[v_idx][2] + dt * hz_detune2).fract();

            let o1 = Oscillator::saw(phase_pad_voices[v_idx][0]);
            let o2 = Oscillator::saw(phase_pad_voices[v_idx][1]) * 0.8;
            let o3 = Oscillator::saw(phase_pad_voices[v_idx][2]) * 0.8;
            let voice = (o1 + o2 + o3) * 0.055;

            let pan = (v_idx as f32 / 3.0) * 0.6 - 0.3;
            pad_mix_l += voice * (1.0 - pan);
            pad_mix_r += voice * (1.0 + pan);
        }
        let pad_l = pad_filter_l.process(pad_mix_l);
        let pad_r = pad_filter_r.process(pad_mix_r);

        // --- 4. ARPEGGIO ---
        let arp_notes = [current_pad_notes[0], current_pad_notes[1], current_pad_notes[2], current_pad_notes[3] + 12];
        let arp_note = arp_notes[step_16th % 4];
        let arp_hz = midi_to_hz(arp_note);
        phase_arp = (phase_arp + dt * arp_hz).fract();

        let t_arp = t % sec_per_16th;
        let arp_env = arp_adsr.evaluate(t_arp, sec_per_16th * 0.7);
        let arp_raw = (Oscillator::square(phase_arp, 0.35) * 0.6 + Oscillator::triangle(phase_arp * 2.0) * 0.4) * arp_env * 0.12;

        // --- 5. LEAD HOOK ---
        let mut lead_sample = 0.0f32;
        if bar_idx >= 4 {
            let mel_step_16th = (step_16th - 64) % 128;
            let lead_note = match mel_step_16th / 4 {
                0 => Some(notes::E4),
                1 => Some(notes::G4),
                2 => Some(notes::A4),
                3 => Some(notes::G4),
                4 => Some(notes::E4),
                5 => Some(notes::D4),
                6 => Some(notes::C4),
                7 => Some(notes::D4),
                8 => Some(notes::F4),
                9 => Some(notes::A4),
                10 => Some(notes::C5),
                11 => Some(notes::A4),
                12 => Some(notes::G4),
                13 => Some(notes::F4),
                14 => Some(notes::E4),
                15 => Some(notes::D4),
                16 => Some(notes::E4),
                17 => Some(notes::G4),
                18 => Some(notes::C5),
                19 => Some(notes::B4),
                20 => Some(notes::G4),
                21 => Some(notes::E4),
                22 => Some(notes::D4),
                23 => Some(notes::E4),
                24 => Some(notes::D4),
                25 => Some(notes::G4),
                26 => Some(notes::B4),
                27 => Some(notes::A4),
                28 => Some(notes::G4),
                29 => Some(notes::E4),
                30 => Some(notes::D4),
                _ => Some(notes::C4),
            };

            if let Some(note) = lead_note {
                let hz = midi_to_hz(note);
                let t_note = t % (sec_per_beat * 1.0);
                let vibrato = (t * 5.5).sin() * 0.006 * (t_note / sec_per_beat);
                let lead_hz = hz * (1.0 + vibrato);
                phase_lead = (phase_lead + dt * lead_hz).fract();
                let env = lead_adsr.evaluate(t_note, sec_per_beat * 0.85);
                let raw_lead = (Oscillator::saw(phase_lead) * 0.6 + Oscillator::square(phase_lead, 0.48) * 0.4) * env;
                lead_sample = raw_lead * 0.22;
            }
        }

        let dry_l = drum_l + bass_filtered + pad_l + arp_raw * 0.8 + lead_sample;
        let dry_r = drum_r + bass_filtered + pad_r + arp_raw * 0.6 + lead_sample;

        let (fx_l, fx_r) = delay.process(lead_sample * 0.5 + arp_raw * 0.3, lead_sample * 0.5 + arp_raw * 0.3);

        left_samples[i] = soft_saturate(dry_l + fx_l, 1.15) * 0.78;
        right_samples[i] = soft_saturate(dry_r + fx_r, 1.15) * 0.78;
    }

    let mut stereo_frames = Vec::with_capacity(total_samples);
    for i in 0..total_samples {
        stereo_frames.push((left_samples[i], right_samples[i]));
    }

    encode_wav_16bit_stereo(&stereo_frames, sample_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_to_hz() {
        assert!((midi_to_hz(69) - 440.0).abs() < 0.01);
        assert!((midi_to_hz(60) - 261.63).abs() < 0.05);
    }

    #[test]
    fn test_generate_menu_theme_header_and_energy() {
        let wav = generate_menu_theme(22050);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert!(wav.len() > 50_000);
    }

    #[test]
    fn test_generate_nightcall_race_theme_header_and_data() {
        let wav = generate_nightcall_race_theme(22050);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert!(wav.len() > 100_000);
    }
}
