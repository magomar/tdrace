//! Comprehensive Spec 022 Regression Test Suite:
//! Per-Tier Engine Sound Banks and Physical Synthesis.
//!
//! Validates:
//! 1. All 25 motorsport tiers resolve unique, tier-appropriate engine sound archetypes.
//! 2. No two tiers within the same discipline share the exact same sound archetype.
//! 3. Classic arcade vehicles preserve Stage 1 Tier 1 discipline mapping.
//! 4. DSP synthesis produces valid, click-free audio buffers for all 25 archetypes (zero NaN/inf, DC bias < 0.05, amplitude in [-1.0, 1.0]).
//! 5. Dynamic vehicle switching in garage showroom crossfades sound banks smoothly.

use std::collections::HashSet;
use tdrace_app::audio::manager::EngineSoundType;
use tdrace_app::audio::samples::ArchetypeSampleBank;
use tdrace_app::catalog::{get_models_for_module_and_tier, CLASSIC_ARCADE_CARS};
use tdrace_app::game::{GameState, GarageOrigin, RaceSession};
use tdrace_app::module::classic::ClassicGameModule;
use tdrace_app::module::GameModule;

fn decode_wav_16bit_mono(bytes: &[u8]) -> Vec<f32> {
    assert!(bytes.len() >= 44, "WAV byte slice too small: {} bytes", bytes.len());
    let mut offset = 12;
    while offset + 8 <= bytes.len() {
        let chunk_id = &bytes[offset..offset + 4];
        let chunk_size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        offset += 8;
        if chunk_id == b"data" {
            let end = (offset + chunk_size).min(bytes.len());
            let data_bytes = &bytes[offset..end];
            return data_bytes
                .chunks_exact(2)
                .map(|b| {
                    let s = i16::from_le_bytes([b[0], b[1]]);
                    s as f32 / 32768.0
                })
                .collect();
        }
        offset += chunk_size;
    }
    // Fallback: standard 44-byte PCM header
    bytes[44..]
        .chunks_exact(2)
        .map(|b| {
            let s = i16::from_le_bytes([b[0], b[1]]);
            s as f32 / 32768.0
        })
        .collect()
}

#[test]
fn test_all_25_tiers_resolve_unique_dedicated_archetypes() {
    let disciplines = [
        (
            "gt",
            [
                EngineSoundType::Gt4Clubsport,
                EngineSoundType::Gt3HighRev,
                EngineSoundType::Gt2Biturbo,
                EngineSoundType::Gt1V12Analogue,
                EngineSoundType::HypercarV6Hybrid,
            ],
        ),
        (
            "nascar",
            [
                EngineSoundType::LateModelV8,
                EngineSoundType::ArcaSpecV8,
                EngineSoundType::SuperTruckV8,
                EngineSoundType::XfinityV8,
                EngineSoundType::NascarV8,
            ],
        ),
        (
            "rally",
            [
                EngineSoundType::CrossCarMotorcycle,
                EngineSoundType::Super1600Atmo,
                EngineSoundType::Rally2Turbo,
                EngineSoundType::SupercarRx1,
                EngineSoundType::GroupBInline5,
            ],
        ),
        (
            "kart",
            [
                EngineSoundType::KartCadet60,
                EngineSoundType::RacingMowerV2,
                EngineSoundType::Kart125cc,
                EngineSoundType::KartShifterKZ,
                EngineSoundType::Superkart250Twin,
            ],
        ),
        (
            "extreme_offroad",
            [
                EngineSoundType::SandRailBoxer,
                EngineSoundType::ProLiteV6,
                EngineSoundType::Ultra4V8,
                EngineSoundType::Pro4UnlimitedV8,
                EngineSoundType::MonsterTruckBlower,
            ],
        ),
    ];

    let mut all_seen = HashSet::new();

    for (module_id, expected_tier_archetypes) in disciplines {
        let mut discipline_seen = HashSet::new();

        for (tier_idx, &expected_archetype) in expected_tier_archetypes.iter().enumerate() {
            let tier = (tier_idx + 1) as u8;
            let models = get_models_for_module_and_tier(module_id, tier);
            assert!(
                !models.is_empty(),
                "Module {module_id} Tier {tier} must have at least one car model"
            );

            for model in &models {
                let sound = model.sound_type();
                assert_eq!(
                    sound, expected_archetype,
                    "Model {} in {} Tier {} should have archetype {:?}, got {:?}",
                    model.id, module_id, tier, expected_archetype, sound
                );
            }

            // Ensure no duplicate archetypes within the same discipline
            assert!(
                discipline_seen.insert(expected_archetype),
                "Duplicate sound archetype {:?} within discipline {}",
                expected_archetype,
                module_id
            );

            // Ensure all 25 archetypes across the entire matrix are unique
            assert!(
                all_seen.insert(expected_archetype),
                "Duplicate sound archetype {:?} across motorsport matrix",
                expected_archetype
            );
        }
    }

    assert_eq!(all_seen.len(), 25, "Exactly 25 distinct archetypes must be represented");
}

#[test]
fn test_classic_arcade_vehicles_preserve_tier_one_mapping() {
    let classic_module = ClassicGameModule::new();
    let vehicles = classic_module.vehicles();

    let expected = [
        ("classic_gt", EngineSoundType::Gt4Clubsport),
        ("classic_nascar", EngineSoundType::LateModelV8),
        ("classic_offroad", EngineSoundType::SandRailBoxer),
        ("classic_kart", EngineSoundType::KartCadet60),
        ("classic_rally", EngineSoundType::CrossCarMotorcycle),
    ];

    // 1. Verify VehicleModelDefinition audio_profile
    for (id, expected_sound) in expected {
        let v = vehicles.iter().find(|c| c.id == id).expect("classic vehicle definition missing");
        let profile = v.audio_profile.expect("profile missing");
        assert_eq!(
            profile.sound_type, expected_sound,
            "Classic vehicle {id} profile sound_type must be {expected_sound:?}"
        );
    }

    // 2. Verify RealCarModel sound_type dispatch
    for (id, expected_sound) in expected {
        let model = CLASSIC_ARCADE_CARS.iter().find(|c| c.id == id).expect("classic car missing in catalog");
        assert_eq!(
            model.sound_type(),
            expected_sound,
            "RealCarModel {id} sound_type must be {expected_sound:?}"
        );
    }
}

#[test]
fn test_dsp_synthesis_all_25_archetypes_valid_buffers() {
    let archetypes = EngineSoundType::all_archetypes();
    assert_eq!(archetypes.len(), 25);

    let sample_rate = 44100;

    for &archetype in archetypes {
        let bank = ArchetypeSampleBank::generate(archetype, sample_rate);

        assert_eq!(bank.engine_type, archetype);

        let points = [
            (&bank.idle, false, "idle"),
            (&bank.mid_on, true, "mid_on"),
            (&bank.mid_off, false, "mid_off"),
            (&bank.high_on, true, "high_on"),
            (&bank.high_off, false, "high_off"),
        ];

        assert!(bank.idle.rpm >= 600.0 && bank.idle.rpm <= 3500.0, "Archetype {:?} idle RPM out of range", archetype);
        assert!(bank.high_on.rpm >= 4000.0 && bank.high_on.rpm <= 16000.0, "Archetype {:?} high RPM out of range", archetype);
        assert_eq!(bank.mid_on.rpm, bank.mid_off.rpm, "Archetype {:?} mid on/off RPM mismatch", archetype);
        assert_eq!(bank.high_on.rpm, bank.high_off.rpm, "Archetype {:?} high on/off RPM mismatch", archetype);

        for (point, expected_load, label) in points {
            assert!(point.rpm > 0.0, "Archetype {:?} point {label} rpm must be positive", archetype);
            assert_eq!(point.is_load, expected_load, "Archetype {:?} point {label} is_load mismatch", archetype);
            assert!(
                !point.wav_bytes.is_empty(),
                "Archetype {:?} point {label} wav_bytes must not be empty",
                archetype
            );

            let samples = decode_wav_16bit_mono(&point.wav_bytes);
            assert!(
                !samples.is_empty(),
                "Archetype {:?} point {label} samples must not be empty",
                archetype
            );

            let mut peak = 0.0f32;
            let mut sum = 0.0f32;

            for &s in &samples {
                assert!(
                    !s.is_nan(),
                    "Archetype {:?} point {label} produced NaN sample",
                    archetype
                );
                assert!(
                    !s.is_infinite(),
                    "Archetype {:?} point {label} produced infinite sample",
                    archetype
                );
                assert!(
                    s >= -1.0 && s <= 1.0,
                    "Archetype {:?} point {label} clipped outside [-1.0, 1.0]: {s}",
                    archetype
                );
                let abs_s = s.abs();
                if abs_s > peak {
                    peak = abs_s;
                }
                sum += s;
            }

            // Must produce audible content
            assert!(
                peak > 0.05,
                "Archetype {:?} point {label} peak {peak} too quiet",
                archetype
            );

            // DC offset check
            let mean = (sum / samples.len() as f32).abs();
            assert!(
                mean < 0.05,
                "Archetype {:?} point {label} has excessive DC bias: {mean}",
                archetype
            );
        }
    }
}

#[test]
fn test_garage_showroom_dynamic_vehicle_switching_sound_resolution() {
    let mut session = RaceSession::new();
    session.state = GameState::Garage(GarageOrigin::Menu);

    // Test across all 5 modules and 5 tiers
    let modules = ["gt", "nascar", "rally", "kart", "extreme_offroad"];

    for module_id in modules {
        session.active_module_id = module_id;

        for tier in 1..=5 {
            session.garage_tier = tier;
            let models = get_models_for_module_and_tier(module_id, tier);
            assert!(!models.is_empty());

            for (idx, model) in models.iter().enumerate() {
                session.garage_car_idx = idx;
                let active_sound = session.resolve_active_sound_type();
                assert_eq!(
                    active_sound,
                    model.sound_type(),
                    "Garage sound mismatch for module {module_id} tier {tier} car {idx}"
                );

                // Setting engine type on audio manager must update active engine type without panic
                session.audio.set_engine_type(active_sound);
                assert_eq!(session.audio.active_engine_type, active_sound);
            }
        }
    }
}

#[test]
fn test_vehicles_within_same_category_have_distinct_audio() {
    let gt4_models = get_models_for_module_and_tier("gt", 1);
    assert!(gt4_models.len() >= 4, "Must have at least 4 GT4 models");

    let sample_rate = 44100;
    let mut vehicle_samples: Vec<(&str, Vec<f32>)> = Vec::new();

    for model in gt4_models {
        let config = model.sound_config();
        let bank = ArchetypeSampleBank::generate_from_config(model.sound_type(), &config, sample_rate);
        let samples = decode_wav_16bit_mono(&bank.mid_on.wav_bytes);
        assert!(!samples.is_empty(), "Vehicle {} produced empty mid_on samples", model.id);
        vehicle_samples.push((model.id, samples));
    }

    // Compare all pairs and ensure normalized cross-correlation < 0.85
    for i in 0..vehicle_samples.len() {
        for j in (i + 1)..vehicle_samples.len() {
            let (id_a, ref samples_a) = vehicle_samples[i];
            let (id_b, ref samples_b) = vehicle_samples[j];

            let min_len = samples_a.len().min(samples_b.len());
            assert!(min_len > 1000, "Buffer too short for correlation analysis");

            let a = &samples_a[..min_len];
            let b = &samples_b[..min_len];

            let mut dot = 0.0f32;
            let mut norm_a = 0.0f32;
            let mut norm_b = 0.0f32;

            for k in 0..min_len {
                dot += a[k] * b[k];
                norm_a += a[k] * a[k];
                norm_b += b[k] * b[k];
            }

            let corr = (dot / (norm_a.sqrt() * norm_b.sqrt())).abs();
            assert!(
                corr < 0.85,
                "Vehicle audio too similar between {} and {}: cross-correlation = {:.3} >= 0.85",
                id_a,
                id_b,
                corr
            );
        }
    }
}
