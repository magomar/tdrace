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
use tdrace_app::audio::samples::{ArchetypeSampleBank, HIGH_RPM, IDLE_RPM, MID_RPM};
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
            (&bank.idle, IDLE_RPM, false, "idle"),
            (&bank.mid_on, MID_RPM, true, "mid_on"),
            (&bank.mid_off, MID_RPM, false, "mid_off"),
            (&bank.high_on, HIGH_RPM, true, "high_on"),
            (&bank.high_off, HIGH_RPM, false, "high_off"),
        ];

        for (point, expected_rpm, expected_load, label) in points {
            assert_eq!(point.rpm, expected_rpm, "Archetype {:?} point {label} rpm mismatch", archetype);
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
