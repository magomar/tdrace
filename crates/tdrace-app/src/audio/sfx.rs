//! Arcade Sound Effects Generator.
//!
//! Synthesizes clean, high-quality, pleasant arcade sound effects in 16-bit 44.1kHz PCM WAV:
//! - Crisp tire drift chirps (classic arcade racer squeaks)
//! - Solid low-end impact thuds (wall & car collision hits)
//! - Crystal clear countdown beeps & "GO!" chime
//! - Celebratory lap arpeggio, sector ping & victory fanfare
//! - Snappy UI navigation clicks

use crate::audio::dsp::{
    encode_wav_16bit_mono, soft_saturate, waveshape_engine, BiquadBandPass, BiquadLowPass,
    NoiseGenerator, Oscillator,
};


pub use cabinet::audio::sfx::{
    generate_car_hit_sound, generate_countdown_high, generate_countdown_low, generate_lap_chime,
    generate_race_finish, generate_sector_ping, generate_skid_sound, generate_ui_move,
    generate_ui_select, generate_wall_crash_sound, SoundCue,
};


/// Physical layout and firing interval distribution across cylinders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CylinderLayout {
    /// Uniformly distributed firing intervals (Inline-4, Inline-6, Flat-plane V8, V10, V12)
    EvenlySpaced,
    /// Crossplane 90° V8 with uneven 90-180-270-90 collector pulse spacing (NASCAR, AMG V8, LS V8)
    CrossplaneV8,
    /// Opposed-cylinder Boxer Flat-6 with split-bank acoustic rasp (Porsche 911 / Cayman)
    BoxerFlat6,
    /// Opposed-cylinder Boxer Flat-4 with syncopated rumble (Sand Rail, Subaru EJ)
    BoxerFlat4,
    /// 5-Cylinder 144° syncopated firing order (Audi Group B Quattro)
    Inline5,
    /// Uneven 270°-450° 4-stroke V-Twin thumper (Racing Mower)
    VTwin4Stroke,
}

impl Default for CylinderLayout {
    fn default() -> Self {
        Self::EvenlySpaced
    }
}

impl CylinderLayout {
    /// Computes the normalized cycle phase offset in `[0.0, 1.0)` for a given cylinder.
    #[inline(always)]
    pub const fn phase_offset(self, cylinder_idx: usize, cylinder_count: usize) -> f32 {
        match self {
            Self::CrossplaneV8 if cylinder_count == 8 => {
                const OFFSETS: [f32; 8] = [0.0, 0.125, 0.375, 0.50, 0.25, 0.625, 0.75, 0.875];
                OFFSETS[cylinder_idx % 8]
            }
            Self::BoxerFlat6 if cylinder_count == 6 => {
                const OFFSETS: [f32; 6] = [0.0, 0.1667, 0.50, 0.6667, 0.3333, 0.8333];
                OFFSETS[cylinder_idx % 6]
            }
            Self::BoxerFlat4 if cylinder_count == 4 => {
                const OFFSETS: [f32; 4] = [0.0, 0.25, 0.625, 0.875];
                OFFSETS[cylinder_idx % 4]
            }
            Self::Inline5 if cylinder_count == 5 => {
                const OFFSETS: [f32; 5] = [0.0, 0.40, 0.20, 0.80, 0.60];
                OFFSETS[cylinder_idx % 5]
            }
            Self::VTwin4Stroke if cylinder_count == 2 => {
                const OFFSETS: [f32; 2] = [0.0, 0.375];
                OFFSETS[cylinder_idx % 2]
            }
            _ => {
                if cylinder_count == 0 {
                    0.0
                } else {
                    (cylinder_idx as f32) / (cylinder_count as f32)
                }
            }
        }
    }
}

/// Physical configuration parameters for procedural engine synthesis.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EngineSoundConfig {
    /// Number of engine cylinders (e.g. 1 for Kart, 4 for Rally, 6 for F1, 8 for GT V8)
    pub cylinder_count: usize,
    /// Whether the engine operates on a 2-stroke cycle (true for Kart) vs 4-stroke cycle
    pub is_two_stroke: bool,
    /// Physical cylinder spatial layout and firing sequence
    pub cylinder_layout: CylinderLayout,
    /// Typical engine idle speed in RPM
    pub idle_rpm: f32,
    /// Maximum operational redline speed in RPM
    pub redline_rpm: f32,
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
    /// Whether this engine has turbocharger blow-off flutter / wastegate chuff
    pub has_turbo_flutter: bool,
    /// Whether this engine has Roots gear-driven blower supercharger whine
    pub has_blower_whine: bool,
    /// Whether this engine has electric MGU-K hybrid inverter whine
    pub has_hybrid_whine: bool,
    /// Whether this engine has aggressive anti-lag overrun pops and backfires
    pub has_anti_lag_pops: bool,
}

impl Default for EngineSoundConfig {
    fn default() -> Self {
        Self::generic()
    }
}

impl EngineSoundConfig {
    /// Builder: updates cylinder count and layout.
    pub const fn with_cylinders(mut self, count: usize, layout: CylinderLayout) -> Self {
        self.cylinder_count = count;
        self.cylinder_layout = layout;
        self
    }

    /// Builder: updates idle and redline RPM bounds.
    pub const fn with_rpms(mut self, idle: f32, redline: f32) -> Self {
        self.idle_rpm = idle;
        self.redline_rpm = redline;
        self
    }

    /// Builder: updates exhaust pipe formant resonant frequencies.
    pub const fn with_formants(mut self, f1: f32, f2: f32) -> Self {
        self.formant_f1_hz = f1;
        self.formant_f2_hz = f2;
        self
    }

    /// Builder: updates turbo spool level and blow-off flutter flag.
    pub const fn with_turbo(mut self, whine: f32, flutter: bool) -> Self {
        self.turbo_whine_level = whine;
        self.has_turbo_flutter = flutter;
        self
    }

    /// Builder: updates intake induction growl intensity.
    pub const fn with_induction(mut self, growl: f32) -> Self {
        self.intake_growl_intensity = growl;
        self
    }

    /// Builder: updates crank sub-harmonic lumpiness.
    pub const fn with_lumpiness(mut self, lumpiness: f32) -> Self {
        self.crank_lumpiness = lumpiness;
        self
    }

    /// Builder: updates valvetrain & mechanical buzz intensity.
    pub const fn with_buzz(mut self, buzz: f32) -> Self {
        self.mechanical_buzz = buzz;
        self
    }

    /// Builder: updates supercharger blower whine flag.
    pub const fn with_blower(mut self, blower: bool) -> Self {
        self.has_blower_whine = blower;
        self
    }

    /// Builder: updates electric hybrid inverter whine flag.
    pub const fn with_hybrid(mut self, hybrid: bool) -> Self {
        self.has_hybrid_whine = hybrid;
        self
    }

    /// Builder: updates anti-lag overrun backfire flag.
    pub const fn with_anti_lag(mut self, anti_lag: bool) -> Self {
        self.has_anti_lag_pops = anti_lag;
        self
    }

    /// Resolves the default synthesis configuration for a given engine sound archetype.
    pub const fn from_sound_type(sound_type: crate::audio::manager::EngineSoundType) -> Self {
        use crate::audio::manager::EngineSoundType;
        match sound_type {
            EngineSoundType::Generic => Self::generic(),
            EngineSoundType::Gt4Clubsport => Self::gt4_clubsport(),
            EngineSoundType::Gt3HighRev => Self::gt3_high_rev(),
            EngineSoundType::Gt2Biturbo => Self::gt2_biturbo(),
            EngineSoundType::Gt1V12Analogue => Self::gt1_v12_analogue(),
            EngineSoundType::HypercarV6Hybrid => Self::hypercar_v6_hybrid(),
            EngineSoundType::LateModelV8 => Self::late_model_v8(),
            EngineSoundType::ArcaSpecV8 => Self::arca_spec_v8(),
            EngineSoundType::SuperTruckV8 => Self::super_truck_v8(),
            EngineSoundType::XfinityV8 => Self::xfinity_v8(),
            EngineSoundType::NascarV8 => Self::nascar_v8(),
            EngineSoundType::CrossCarMotorcycle => Self::cross_car_motorcycle(),
            EngineSoundType::Super1600Atmo => Self::super1600_atmo(),
            EngineSoundType::Rally2Turbo => Self::rally2_turbo(),
            EngineSoundType::SupercarRx1 => Self::supercar_rx1(),
            EngineSoundType::GroupBInline5 => Self::group_b_inline5(),
            EngineSoundType::KartCadet60 => Self::kart_cadet_60(),
            EngineSoundType::RacingMowerV2 => Self::racing_mower_v2(),
            EngineSoundType::Kart125cc => Self::kart_125cc(),
            EngineSoundType::KartShifterKZ => Self::kart_shifter_kz(),
            EngineSoundType::Superkart250Twin => Self::superkart_250_twin(),
            EngineSoundType::SandRailBoxer => Self::sand_rail_boxer(),
            EngineSoundType::ProLiteV6 => Self::pro_lite_v6(),
            EngineSoundType::Ultra4V8 => Self::ultra4_v8(),
            EngineSoundType::Pro4UnlimitedV8 => Self::pro4_unlimited_v8(),
            EngineSoundType::MonsterTruckBlower => Self::monster_truck_blower(),
            EngineSoundType::SportGT => Self::sport_gt(),
            EngineSoundType::RallyTurbo => Self::rally_turbo(),
        }
    }

    /// Generic / Balanced 6-Cylinder 4-Stroke Sports Engine (Fallback Default)
    pub const fn generic() -> Self {
        Self {
            cylinder_count: 6,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 850.0,
            redline_rpm: 8000.0,
            crank_lumpiness: 0.28,
            combustion_asymmetry: 0.40,
            intake_growl_intensity: 0.22,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.14,
            formant_f1_hz: 200.0,
            formant_f2_hz: 1800.0,
            formant_q: 1.8,
            saturation_drive: 1.20,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: false,
        }
    }

    // =========================================================================
    // 1. Gran Turismo & Endurance GT (T1–T5)
    // =========================================================================

    /// T1: GT4 Clubsport (production-based 6-cyl sport exhaust, clean mechanical rasp)
    pub const fn gt4_clubsport() -> Self {
        Self {
            cylinder_count: 6,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::BoxerFlat6,
            idle_rpm: 900.0,
            redline_rpm: 7800.0,
            crank_lumpiness: 0.25,
            combustion_asymmetry: 0.42,
            intake_growl_intensity: 0.28,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.16,
            formant_f1_hz: 180.0,
            formant_f2_hz: 1750.0,
            formant_q: 2.0,
            saturation_drive: 1.25,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: false,
        }
    }

    /// T2: GT3 Evo (flat-plane screaming V8 / 9,000 RPM howl, straight-cut gear whine)
    pub const fn gt3_high_rev() -> Self {
        Self {
            cylinder_count: 8,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 1000.0,
            redline_rpm: 9000.0,
            crank_lumpiness: 0.18,
            combustion_asymmetry: 0.60,
            intake_growl_intensity: 0.38,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.24,
            formant_f1_hz: 220.0,
            formant_f2_hz: 2200.0,
            formant_q: 2.5,
            saturation_drive: 1.35,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T3: GT2 Biturbo (700+ BHP forced induction, deep twin-turbo spool & wastegate chuff)
    pub const fn gt2_biturbo() -> Self {
        Self {
            cylinder_count: 6,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::BoxerFlat6,
            idle_rpm: 900.0,
            redline_rpm: 7500.0,
            crank_lumpiness: 0.30,
            combustion_asymmetry: 0.48,
            intake_growl_intensity: 0.42,
            turbo_whine_level: 0.35,
            mechanical_buzz: 0.18,
            formant_f1_hz: 150.0,
            formant_f2_hz: 1600.0,
            formant_q: 2.1,
            saturation_drive: 1.38,
            has_turbo_flutter: true,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T4: GT1 Legend (raw 90s screaming 6.0L V12, deafening high-pitched wail, pure analog)
    pub const fn gt1_v12_analogue() -> Self {
        Self {
            cylinder_count: 12,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 1000.0,
            redline_rpm: 8500.0,
            crank_lumpiness: 0.12,
            combustion_asymmetry: 0.52,
            intake_growl_intensity: 0.45,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.28,
            formant_f1_hz: 320.0,
            formant_f2_hz: 2800.0,
            formant_q: 3.0,
            saturation_drive: 1.42,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T5: LMH Hypercar (high-strung twin-turbo V6 + electric MGU-K motor inverter whine)
    pub const fn hypercar_v6_hybrid() -> Self {
        Self {
            cylinder_count: 6,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 1200.0,
            redline_rpm: 9000.0,
            crank_lumpiness: 0.22,
            combustion_asymmetry: 0.50,
            intake_growl_intensity: 0.36,
            turbo_whine_level: 0.26,
            mechanical_buzz: 0.20,
            formant_f1_hz: 240.0,
            formant_f2_hz: 2400.0,
            formant_q: 2.6,
            saturation_drive: 1.32,
            has_turbo_flutter: true,
            has_blower_whine: false,
            has_hybrid_whine: true,
            has_anti_lag_pops: false,
        }
    }

    // =========================================================================
    // 2. NASCAR & Stock Car Racing (T1–T5)
    // =========================================================================

    /// T1: Street Stock (small-block 350 cu in V8, deep low-end idle cam lope, iron-block rumble)
    pub const fn late_model_v8() -> Self {
        Self {
            cylinder_count: 8,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::CrossplaneV8,
            idle_rpm: 850.0,
            redline_rpm: 6800.0,
            crank_lumpiness: 0.48,
            combustion_asymmetry: 0.50,
            intake_growl_intensity: 0.34,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.20,
            formant_f1_hz: 110.0,
            formant_f2_hz: 1200.0,
            formant_q: 2.2,
            saturation_drive: 1.32,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T2: ARCA Spec Racer (spec 396 cu in V8, raspy side-exit collector bark, pushrod clatter)
    pub const fn arca_spec_v8() -> Self {
        Self {
            cylinder_count: 8,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::CrossplaneV8,
            idle_rpm: 900.0,
            redline_rpm: 7200.0,
            crank_lumpiness: 0.46,
            combustion_asymmetry: 0.52,
            intake_growl_intensity: 0.38,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.25,
            formant_f1_hz: 115.0,
            formant_f2_hz: 1250.0,
            formant_q: 2.3,
            saturation_drive: 1.35,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T3: Craftsman Trucks (pushrod 358 cu in V8, booming pickup bed acoustic resonance)
    pub const fn super_truck_v8() -> Self {
        Self {
            cylinder_count: 8,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::CrossplaneV8,
            idle_rpm: 950.0,
            redline_rpm: 7500.0,
            crank_lumpiness: 0.49,
            combustion_asymmetry: 0.54,
            intake_growl_intensity: 0.40,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.24,
            formant_f1_hz: 105.0,
            formant_f2_hz: 1180.0,
            formant_q: 2.5,
            saturation_drive: 1.38,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T4: Xfinity Series (high-compression 358 cu in V8, screaming crossover X-pipe exhaust howl)
    pub const fn xfinity_v8() -> Self {
        Self {
            cylinder_count: 8,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::CrossplaneV8,
            idle_rpm: 1000.0,
            redline_rpm: 8500.0,
            crank_lumpiness: 0.45,
            combustion_asymmetry: 0.58,
            intake_growl_intensity: 0.42,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.26,
            formant_f1_hz: 130.0,
            formant_f2_hz: 1400.0,
            formant_q: 2.4,
            saturation_drive: 1.40,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T5: Cup Next-Gen / TA1 (850 BHP pushrod V8, open boom-tube split side pipes, thunderous roar)
    pub const fn nascar_v8() -> Self {
        Self {
            cylinder_count: 8,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::CrossplaneV8,
            idle_rpm: 1100.0,
            redline_rpm: 9200.0,
            crank_lumpiness: 0.50,
            combustion_asymmetry: 0.56,
            intake_growl_intensity: 0.42,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.26,
            formant_f1_hz: 120.0,
            formant_f2_hz: 1300.0,
            formant_q: 2.4,
            saturation_drive: 1.40,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    // =========================================================================
    // 3. Rallycross & All-Terrain (T1–T5)
    // =========================================================================

    /// T1: CrossCar Junior (750cc 4-cyl motorcycle superbike engine, ultra-fast rev acceleration)
    pub const fn cross_car_motorcycle() -> Self {
        Self {
            cylinder_count: 4,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 1500.0,
            redline_rpm: 13500.0,
            crank_lumpiness: 0.15,
            combustion_asymmetry: 0.62,
            intake_growl_intensity: 0.32,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.35,
            formant_f1_hz: 380.0,
            formant_f2_hz: 2900.0,
            formant_q: 2.7,
            saturation_drive: 1.36,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: false,
        }
    }

    /// T2: Super1600 FWD (screaming high-compression 1.6L intake bark, carbon airbox roar)
    pub const fn super1600_atmo() -> Self {
        Self {
            cylinder_count: 4,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 1100.0,
            redline_rpm: 9000.0,
            crank_lumpiness: 0.26,
            combustion_asymmetry: 0.56,
            intake_growl_intensity: 0.44,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.24,
            formant_f1_hz: 240.0,
            formant_f2_hz: 2300.0,
            formant_q: 2.2,
            saturation_drive: 1.32,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: false,
        }
    }

    /// T3: Rally2 / R5 AWD (1.6L turbo + 32mm restrictor chuff, responsive anti-lag system pops)
    pub const fn rally2_turbo() -> Self {
        Self {
            cylinder_count: 4,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 1000.0,
            redline_rpm: 7500.0,
            crank_lumpiness: 0.32,
            combustion_asymmetry: 0.50,
            intake_growl_intensity: 0.38,
            turbo_whine_level: 0.24,
            mechanical_buzz: 0.20,
            formant_f1_hz: 180.0,
            formant_f2_hz: 2000.0,
            formant_q: 2.0,
            saturation_drive: 1.30,
            has_turbo_flutter: true,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T4: Supercar RX1 (600 BHP 2.0L turbo, violent machine-gun 2-step anti-lag firecrackers)
    pub const fn supercar_rx1() -> Self {
        Self {
            cylinder_count: 4,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 1200.0,
            redline_rpm: 8500.0,
            crank_lumpiness: 0.38,
            combustion_asymmetry: 0.58,
            intake_growl_intensity: 0.46,
            turbo_whine_level: 0.34,
            mechanical_buzz: 0.22,
            formant_f1_hz: 195.0,
            formant_f2_hz: 2150.0,
            formant_q: 2.2,
            saturation_drive: 1.44,
            has_turbo_flutter: true,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T5: Group B Beast (2.1L turbo inline-5 off-beat syncopated warble, aggressive wastegate screech)
    pub const fn group_b_inline5() -> Self {
        Self {
            cylinder_count: 5,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::Inline5,
            idle_rpm: 1100.0,
            redline_rpm: 8200.0,
            crank_lumpiness: 0.42,
            combustion_asymmetry: 0.55,
            intake_growl_intensity: 0.48,
            turbo_whine_level: 0.38,
            mechanical_buzz: 0.25,
            formant_f1_hz: 165.0,
            formant_f2_hz: 1950.0,
            formant_q: 2.3,
            saturation_drive: 1.45,
            has_turbo_flutter: true,
            has_blower_whine: true,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    // =========================================================================
    // 4. Grassroots Karting (T1–T5)
    // =========================================================================

    /// T1: 60cc Cadet (60cc 2-stroke single, gentle ring-a-ding centrifugal clutch buzz)
    pub const fn kart_cadet_60() -> Self {
        Self {
            cylinder_count: 1,
            is_two_stroke: true,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 2500.0,
            redline_rpm: 11000.0,
            crank_lumpiness: 0.06,
            combustion_asymmetry: 0.60,
            intake_growl_intensity: 0.14,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.36,
            formant_f1_hz: 680.0,
            formant_f2_hz: 2400.0,
            formant_q: 2.5,
            saturation_drive: 1.25,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: false,
        }
    }

    /// T2: Racing Mower (4-stroke V-Twin thumper, straight-pipe unbaffled mower raspy chug)
    pub const fn racing_mower_v2() -> Self {
        Self {
            cylinder_count: 2,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::VTwin4Stroke,
            idle_rpm: 800.0,
            redline_rpm: 4200.0,
            crank_lumpiness: 0.52,
            combustion_asymmetry: 0.64,
            intake_growl_intensity: 0.35,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.28,
            formant_f1_hz: 130.0,
            formant_f2_hz: 1100.0,
            formant_q: 2.1,
            saturation_drive: 1.34,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: false,
        }
    }

    /// T3: 125cc Rotax MAX (125cc 2-stroke single, crisp expansion chamber bite)
    pub const fn kart_125cc() -> Self {
        Self {
            cylinder_count: 1,
            is_two_stroke: true,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 2800.0,
            redline_rpm: 14000.0,
            crank_lumpiness: 0.08,
            combustion_asymmetry: 0.65,
            intake_growl_intensity: 0.18,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.42,
            formant_f1_hz: 750.0,
            formant_f2_hz: 2600.0,
            formant_q: 2.8,
            saturation_drive: 1.38,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: false,
        }
    }

    /// T4: 125cc KZ Shifter (125cc 6-speed shifter, metallic 2-stroke sting, ignition-cut bangs)
    pub const fn kart_shifter_kz() -> Self {
        Self {
            cylinder_count: 1,
            is_two_stroke: true,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 2900.0,
            redline_rpm: 15500.0,
            crank_lumpiness: 0.09,
            combustion_asymmetry: 0.68,
            intake_growl_intensity: 0.22,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.46,
            formant_f1_hz: 820.0,
            formant_f2_hz: 2850.0,
            formant_q: 3.0,
            saturation_drive: 1.40,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T5: 250cc Superkart (250cc twin-cylinder 2-stroke, banshee wail at 240 km/h)
    pub const fn superkart_250_twin() -> Self {
        Self {
            cylinder_count: 2,
            is_two_stroke: true,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 3000.0,
            redline_rpm: 14500.0,
            crank_lumpiness: 0.10,
            combustion_asymmetry: 0.70,
            intake_growl_intensity: 0.26,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.48,
            formant_f1_hz: 900.0,
            formant_f2_hz: 3100.0,
            formant_q: 3.2,
            saturation_drive: 1.44,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    // =========================================================================
    // 5. Extreme Off-Road (T1–T5)
    // =========================================================================

    /// T1: Pro Buggy / Sand Rail (2.5L turbo flat-4 boxer, off-beat thrum, turbo flutter, blow-off hiss)
    pub const fn sand_rail_boxer() -> Self {
        Self {
            cylinder_count: 4,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::BoxerFlat4,
            idle_rpm: 950.0,
            redline_rpm: 7000.0,
            crank_lumpiness: 0.44,
            combustion_asymmetry: 0.54,
            intake_growl_intensity: 0.36,
            turbo_whine_level: 0.28,
            mechanical_buzz: 0.22,
            formant_f1_hz: 160.0,
            formant_f2_hz: 1850.0,
            formant_q: 2.2,
            saturation_drive: 1.35,
            has_turbo_flutter: true,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T2: Pro Lite (4.0L race V6, naturally aspirated desert rasp, sharp throttle response)
    pub const fn pro_lite_v6() -> Self {
        Self {
            cylinder_count: 6,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::EvenlySpaced,
            idle_rpm: 950.0,
            redline_rpm: 7600.0,
            crank_lumpiness: 0.32,
            combustion_asymmetry: 0.46,
            intake_growl_intensity: 0.36,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.20,
            formant_f1_hz: 190.0,
            formant_f2_hz: 1900.0,
            formant_q: 2.1,
            saturation_drive: 1.30,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: false,
        }
    }

    /// T3: Ultra4 Bouncer (7.0L big block LS V8, uncorked zoomie headers, low-end torque chop)
    pub const fn ultra4_v8() -> Self {
        Self {
            cylinder_count: 8,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::CrossplaneV8,
            idle_rpm: 850.0,
            redline_rpm: 6500.0,
            crank_lumpiness: 0.54,
            combustion_asymmetry: 0.58,
            intake_growl_intensity: 0.44,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.28,
            formant_f1_hz: 110.0,
            formant_f2_hz: 1350.0,
            formant_q: 2.3,
            saturation_drive: 1.42,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T4: Pro4 Stadium Truck (900 BHP 4WD race V8, extreme short-course screamer)
    pub const fn pro4_unlimited_v8() -> Self {
        Self {
            cylinder_count: 8,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::CrossplaneV8,
            idle_rpm: 1050.0,
            redline_rpm: 8400.0,
            crank_lumpiness: 0.46,
            combustion_asymmetry: 0.58,
            intake_growl_intensity: 0.44,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.26,
            formant_f1_hz: 135.0,
            formant_f2_hz: 1450.0,
            formant_q: 2.4,
            saturation_drive: 1.40,
            has_turbo_flutter: false,
            has_blower_whine: false,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    /// T5: Monster Truck (1,500 BHP methanol V8, screaming Roots supercharger blower whine)
    pub const fn monster_truck_blower() -> Self {
        Self {
            cylinder_count: 8,
            is_two_stroke: false,
            cylinder_layout: CylinderLayout::CrossplaneV8,
            idle_rpm: 1000.0,
            redline_rpm: 7200.0,
            crank_lumpiness: 0.56,
            combustion_asymmetry: 0.62,
            intake_growl_intensity: 0.50,
            turbo_whine_level: 0.0,
            mechanical_buzz: 0.30,
            formant_f1_hz: 100.0,
            formant_f2_hz: 1250.0,
            formant_q: 2.6,
            saturation_drive: 1.48,
            has_turbo_flutter: false,
            has_blower_whine: true,
            has_hybrid_whine: false,
            has_anti_lag_pops: true,
        }
    }

    // =========================================================================
    // Legacy Aliases
    // =========================================================================

    /// High-Displacement Crossplane V8 Touring GT Muscle Engine (Legacy alias for GT)
    pub const fn sport_gt() -> Self {
        Self::gt4_clubsport()
    }

    /// 4-Cylinder WRC Turbo Anti-Lag Rally Engine (Legacy alias for Rally2Turbo)
    pub const fn rally_turbo() -> Self {
        Self::rally2_turbo()
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

        // 1. Physical Cylinder Combustion Pressure Pulses across engine cycle (staggered at cylinder layout offsets)
        let mut combustion = 0.0f32;
        for c in 0..config.cylinder_count {
            let phase_offset = config.cylinder_layout.phase_offset(c, config.cylinder_count);
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


/// Generates aggressive 4-cylinder WRC turbo rally engine sound loop band.
pub fn generate_rally_turbo_rpm_band(sample_rate: u32, base_hz: f32) -> Vec<u8> {
    generate_custom_engine_rpm_band(sample_rate, base_hz, &EngineSoundConfig::rally_turbo())
}

/// Generates roaring 5.9L Pushrod V8 stock car engine sound loop band.
pub fn generate_nascar_v8_rpm_band(sample_rate: u32, base_hz: f32) -> Vec<u8> {
    generate_custom_engine_rpm_band(sample_rate, base_hz, &EngineSoundConfig::nascar_v8())
}

/// Generates raspy 2.5L Turbo Flat-4 boxer sand rail engine sound loop band.
pub fn generate_sand_rail_boxer_rpm_band(sample_rate: u32, base_hz: f32) -> Vec<u8> {
    generate_custom_engine_rpm_band(sample_rate, base_hz, &EngineSoundConfig::sand_rail_boxer())
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

/// Generates a neutral sound buffer for jump launch (no arcade pitch sweep).
pub fn generate_jump_launch_sound(sample_rate: u32) -> Vec<u8> {
    let duration = 0.08;
    let total_samples = (duration * sample_rate as f32).round() as usize;
    let samples = vec![0.0f32; total_samples];
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
            ("rally", generate_rally_turbo_rpm_band(DEFAULT_SAMPLE_RATE, base_hz)),
            ("nascar", generate_nascar_v8_rpm_band(DEFAULT_SAMPLE_RATE, base_hz)),
            ("sand_rail", generate_sand_rail_boxer_rpm_band(DEFAULT_SAMPLE_RATE, base_hz)),
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
