//! Keyboard handling presets and settings plumbing (Spec 042).

use tdrace_app::config::{GameConfig, InputConfig};
use tdrace_app::game::RaceSession;
use tdrace_app::input::SteeringProfile;

/// Scenario: Old settings files still load
///
/// Given a settings file saved with `steering_profile = "agile"` and the old 10 fields
/// When the game loads it
/// Then the Sharp preset is active and no error is raised
#[test]
fn test_legacy_settings_file_loads_as_sharp() {
    let legacy = r#"
[input]
steering_profile = "agile"
speed_sensitive_enabled = true
steer_rise_rate = 10.0
steer_return_rate = 16.0
steer_exponent = 1.15
speed_sensitive_factor = 0.003
min_speed_steer_limit = 0.8
hold_bleed_rate = 6.0
throttle_rise_rate = 11.0
brake_rise_rate = 8.0
"#;
    let cfg: GameConfig = toml::from_str(legacy).expect("legacy settings must load");
    assert_eq!(cfg.input.steering_profile, SteeringProfile::Sharp);
    let keys = cfg.input.to_filter_config();
    assert_eq!(keys, SteeringProfile::Sharp.to_config());

    let direct: InputConfig = toml::from_str("steering_profile = \"direct\"").unwrap();
    assert_eq!(direct.to_filter_config().profile, SteeringProfile::Sharp);
}

#[test]
fn test_input_config_round_trips_through_filter_config() {
    let mut keys = SteeringProfile::Smooth.to_config();
    keys.traction_help = 0.3;
    keys.profile = keys.matching_profile();
    let persisted = InputConfig::from_filter_config(&keys);
    let text = toml::to_string(&persisted).unwrap();
    let loaded: InputConfig = toml::from_str(&text).unwrap();
    assert_eq!(loaded.to_filter_config(), keys);
    assert_eq!(keys.profile, SteeringProfile::Custom);
}

/// Scenario: Settings apply to the live car in both directions
///
/// Given a race running with Raw
/// When the player switches to Smooth and back to Raw
/// Then the player car's steering authority and traction help match the active preset each time
#[test]
fn test_settings_apply_to_live_player_car_both_directions() {
    let mut session = RaceSession::new();
    session.init_race();
    let idx = session.player_car_index();

    for profile in [SteeringProfile::Raw, SteeringProfile::Smooth, SteeringProfile::Raw] {
        session.input.set_steering_profile(profile);
        session.apply_player_handling();
        let keys = profile.to_config();
        let handling = session.cars[idx].config.player;
        assert!(handling.grip_aware_steering, "human car must use grip-aware steering");
        assert!((handling.steer_overslip - keys.steer_authority).abs() < 1e-6, "{profile:?} authority");
        assert!((handling.traction_help - keys.traction_help).abs() < 1e-6, "{profile:?} traction help");
    }

    // Bots keep the default (linear) handling
    for (i, car) in session.cars.iter().enumerate() {
        if i != idx {
            assert!(!car.config.player.grip_aware_steering, "bot {i} must not get player aids");
        }
    }
}
