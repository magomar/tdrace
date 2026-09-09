# Motor Sound Improvement Analysis: Procedural Synthesis vs. Real Sampled Audio

## 1. Executive Summary & Root Cause Diagnosis

In **TDRace**, the engine audio is currently produced by an in-house procedural synthesis engine (`crates/tdrace-app/src/audio/sfx.rs` & `manager.rs`). While the synthesis code models physical combustion pulses, cylinder counts, crankshaft imbalance, formants, and wave-shaping across 6 vehicle archetypes (`Generic`, `SportGT`, `Kart125cc`, `F1V6Turbo`, `RallyTurbo`, `NascarV8`), **the resulting in-game audio sounds like an organ chord or accordion rather than a continuous, revving internal combustion engine.**

### The Technical Root Causes

1. **Audio Driver Playback Limitation (`macroquad::audio` / `quad-snd`)**:
   `macroquad` relies on `quad-snd` for audio output. Its playback interface (`PlaySoundParams`) only supports:
   ```rust
   pub struct PlaySoundParams {
       pub looped: bool,
       pub volume: f32,
   }
   ```
   Crucially, **there is no API or internal mixer capability for dynamic pitch shifting, playback speed modulation, or real-time resampling.**

2. **The 28-Band Discrete Harmonic Workaround**:
   Because pitch cannot be adjusted during playback, TDRace pre-synthesizes **28 discrete harmonic frequency bands** (from 850 RPM to 9,300 RPM) for each archetype ($6 \times 28 = 168$ pre-rendered WAV buffers generated at startup).

3. **The "Organ Chord" Crossfade Artifact**:
   When accelerating from 2,000 RPM to 3,000 RPM, the game cannot pitch-bend a sound upward. Instead, it identifies the two nearest fixed-pitch bands and crossfades their volumes using equal-power sine/cosine weights:
   - Band $A$ (e.g., 2,050 RPM = 102.5 Hz) remains at a fixed frequency while fading down.
   - Band $B$ (e.g., 2,350 RPM = 117.5 Hz) remains at a fixed frequency while fading up.
   - The human ear detects two distinct musical pitches sounding simultaneously (an interval of ~2.4 semitones), producing acoustic beating and phasing rather than an accelerating vehicle.

4. **Startup Latency & Memory Footprint**:
   Pre-rendering 168 separate WAV buffers consumes noticeable CPU cycles on startup and keeps tens of megabytes of decompressed PCM buffers resident in memory, even though only one vehicle is driven at a time.

---

## 2. Industry Paradigms for Racing Engine Audio

Modern racing titles (from *Gran Turismo*, *Assetto Corsa*, and *Forza* to arcade classics like *Screamer* and *OutRun 2*) utilize three main techniques:

```
                      ┌──────────────────────────────────────────────┐
                      │             Engine Audio Paradigms           │
                      └──────────────────────┬───────────────────────┘
                                             │
             ┌───────────────────────────────┴───────────────────────────────┐
             ▼                                                               ▼
   【Sample-Based Engine】                                         【Pure Procedural Engine】
   • Multi-RPM Dynamic Resampling (Standard)                        • Real-time Phase Accumulator
   • Granular Single-Cycle Slicing (High-End)                       • Physical Waveguide / Modal Modeling
   • Hybrid: Sampled Core + Procedural Layers                       • Zero asset size, but synthetic
```

### A. Multi-Sample Loop Bank with Dynamic Pitch Matching (Industry Standard)
- **Asset Layout**: Steady-state chassis dyno recordings captured at 3–5 representative RPM breakpoints (e.g., 1,000 RPM Idle, 3,000 RPM Low, 5,500 RPM Mid, 8,500 RPM Redline).
  - **On-Throttle (Drive / Accel)**: High manifold pressure, wide-open intake gulp, sharp combustion clatter.
  - **Off-Throttle (Coast / Decel)**: High intake vacuum, subdued combustion, exhaust overrun burble, engine braking rasp.
- **Dynamic Pitch-Matching Algorithm**:
  When the vehicle is at an arbitrary $RPM_{\text{current}}$, the two nearest sample loops ($A$ at $RPM_A$ and $B$ at $RPM_B$) are **both dynamically pitch-shifted to the exact current target frequency**:
  $$\text{pitch}_A = \frac{RPM_{\text{current}}}{RPM_A}, \quad \text{pitch}_B = \frac{RPM_{\text{current}}}{RPM_B}$$
  Because both sample streams play at the **exact same fundamental frequency**, there is **zero musical beating or chord dissonance**. The crossfade only smoothly morphs the acoustic *timbre* and harmonic balance as the revs climb.
- **Load Crossfade**: Linearly blends between the On-Throttle and Off-Throttle pitch-matched pairs according to driver throttle demand and engine load.

### B. Granular Engine Synthesis (Gran Turismo / AngeTheGreat / Turn10)
- Real audio recordings are parsed and sliced into individual combustion cycles or single cylinder pulses (grains).
- At runtime, grains are scheduled and triggered at the instantaneous firing frequency:
  $$f_{\text{firing}} = \frac{RPM \times \text{cylinders}}{120} \quad (\text{for 4-stroke})$$
- Eliminates pitch-shifting artifacts ("chipmunk effect") and supports an infinite RPM range from single cycle recordings. Requires grain overlap-add (OLA) DSP and careful phase alignment.

### C. Hybrid Synthesis (Sampled Combustion + Procedural FX)
- Uses real sampled loops for the core combustion drone, where mic authenticity matters most.
- Stacks lightweight real-time procedural generators for dynamic secondary effects:
  - Gearbox straight-cut transmission whine (speed-proportional tone).
  - Turbocharger compressor spool whistle + blow-off valve (BOV) hiss.
  - Exhaust overrun burble, crackles, and ignition-cut rev limiter bangs.

### D. Continuous-Phase Procedural Physical Modeling
- Retains pure code synthesis (zero WAV assets), but replaces pre-rendered buffers with an active real-time audio callback thread.
- Phase is accumulated continuously each frame: $\Delta \phi = \frac{f(RPM)}{f_s}$.
- Solves the discrete-band beating issue, but requires high-order non-linear physical modeling to sound gritty and organic rather than synthesizer-like.

---

## 3. Options Comparison Matrix

| Option | Realism & Punch | CPU / Memory | Asset Size | Engineering Complexity | Cross-Platform Parity |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Option A: Real Sampled Loops + Dynamic Pitch (`kira`)** | **⭐⭐⭐⭐⭐** (Authentic dyno recordings) | Low (~2–4% CPU, ~10 MB RAM) | ~2–6 MB OGG/WAV | Medium | Native: Excellent<br>WASM: WebAudio bridge |
| **Option B: Hybrid (Real Sample Core + Procedural Layers)** | **⭐⭐⭐⭐⭐** (Rich, highly dynamic) | Low–Moderate | ~2–4 MB OGG/WAV | Medium–High | Native: Excellent<br>WASM: WebAudio bridge |
| **Option C: Streamed Continuous Procedural Synth (`cpal`)** | **⭐⭐⭐** (Smooth revs, but synthetic) | Moderate (streaming thread) | **0 MB** | Medium | Native: Good<br>WASM: AudioWorklet required |
| **Option D: Dense Sample Banks in Current `macroquad::audio`** | **⭐⭐⭐** (Better texture, still chords) | High startup time & RAM (~150 MB) | Pre-rendered in RAM | Low | Native & WASM out-of-the-box |

---

## 4. Deep Dive: Option A (Real Sampled Audio with Dynamic Pitch-Shifting)

### Architectural Approach
To make real sampled engine audio sing without chord beating, TDRace requires an audio engine that supports **dynamic playback rate / pitch modification**.

#### Audio Backend: `kira`
- **Why `kira`**:
  - Purpose-built for game audio in Rust.
  - First-class support for `sound_handle.set_playback_rate(factor, Tween::default())` and `sound_handle.set_volume(vol, Tween::default())`.
  - Non-blocking audio thread with lock-free communication channels.
  - Built-in spatial panning, tweening, and sub-buses for clean volume control.
  - Clean separation: Macroquad continues to handle 2D graphics, windowing, and input; `kira` manages audio output.

### Asset Structure & Loop Specifications
Each vehicle archetype requires 4 steady-state seamless audio loops encoded in 44.1 kHz 16-bit mono WAV or compressed OGG:

```
assets/audio/engines/<archetype>/
├── idle.ogg          (~1,000 RPM steady loop, ~1.5s duration)
├── mid_on.ogg        (~3,500 RPM on-load steady loop)
├── mid_off.ogg       (~3,500 RPM off-load / engine-braking loop)
├── high_on.ogg       (~7,000 RPM on-load steady loop)
├── high_off.ogg      (~7,000 RPM off-load steady loop)
└── shift_pop.ogg     (Optional one-shot tailpipe backfire crack)
```

### Runtime Engine Mixer Math
On every game update frame ($dt$):
1. **Pitch Calculation**:
   For the lower and upper bounding sample loops:
   $$\text{pitch}_{\text{low}} = \frac{RPM_{\text{current}}}{RPM_{\text{low}}}, \quad \text{pitch}_{\text{high}} = \frac{RPM_{\text{current}}}{RPM_{\text{high}}}$$
2. **Frequency Crossfade**:
   Compute fractional interpolation $u = \frac{RPM_{\text{current}} - RPM_{\text{low}}}{RPM_{\text{high}} - RPM_{\text{low}}}$.
3. **Throttle / Load Balance**:
   Blend on-load and off-load samples:
   $$\text{vol}_{\text{on}} = \text{load}, \quad \text{vol}_{\text{off}} = 1.0 - \text{load}$$
4. **Final Gain Assignment**:
   Update the 4 active looping voices with zero-latency smoothing tweens (5–10 ms).

### Multi-Layer Acoustic Soundscape
In addition to the core combustion loops, the new system introduces:
- **Transmission Straight-Cut Gear Whine**: Pitch directly proportional to wheel speed; prominent in racing GT and Rally cars.
- **Turbocharger Spool & Blow-Off Valve (BOV)**: Whistle pitch follows RPM/boost; releasing throttle triggers a crisp pressure release ("pssshh").
- **Exhaust Overrun Pops & Flames**: Randomized detonation one-shots on aggressive downshifts or lift-off from $>6,000$ RPM.
- **Rev Limiter Ignition Cut**: Rapid 15 Hz amplitude gating at redline rather than a flat plateau.

---

## 5. Licensing & Sample Acquisition Strategy

High-fidelity engine recordings can be sourced cleanly and legally:
1. **Soniss GDC Audio Archives (Royalty-Free / Commercial Use)**:
   Annual professional game audio releases containing isolated multi-mic dyno passes for Ford V8s, Porsche GT3s, Subaru WRCs, and 2-stroke karts.
2. **Freesound.org (CC0 / Public Domain)**:
   Extensive collection of isolated steady-state engine dyno runs with clearly measured RPM labels.
3. **AngeTheGreat Engine Simulator (Open-Source Synthesizer)**:
   Enables deterministic synthesis and export of clean combustion audio without wind or background noise, customized to specific cylinder layouts and firing orders.

---

## 6. Migration & Backward Compatibility Strategy

- **Graceful Fallback**: The existing procedural engine remains functional as a fallback if sample assets are missing or during headless test runs.
- **Platform Separation**: Desktop targets (Linux, macOS, Windows) use `kira` with native `cpal`. Web targets can either use `kira`'s WebAudio backend or an extended miniquad audio plugin that exposes `playbackRate`.
