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

mod preset_feel {
    use tdrace_app::input::{DigitalInputFilter, SteeringProfile};
    use tdrace_core::physics::car::{Car, CarControls};
    use tdrace_core::physics::config::{CarConfig, PlayerHandling};
    use tdrace_core::physics::surface::SurfaceType;
    use tdrace_core::Vec2;

    const DT: f32 = 1.0 / 120.0;

    /// A keyboard driver on `profile`: steer key held from t = 0, W held or a modulated
    /// speed-holding throttle. Returns per-step (yaw rate, sideslip).
    fn drive(profile: SteeringProfile, speed: f32, hold_w: bool, seconds: f32) -> Vec<(f32, f32)> {
        let keys = profile.to_config();
        let mut cfg = CarConfig::sports_car();
        cfg.player = PlayerHandling::human(keys.steer_authority, keys.traction_help);
        let mut car = Car::new(cfg);
        car.set_velocity(Vec2::new(speed, 0.0));
        let mut filter = DigitalInputFilter::new(keys);
        let mut out = Vec::new();
        for _ in 0..(seconds / DT) as usize {
            // W held, or a driver who modulates throttle to hold the entry speed.
            let raw_throttle = if hold_w { 1.0 } else { ((speed - car.state.speed) * 0.5).clamp(0.0, 1.0) };
            let (steer, throttle, brake) = filter.update(1.0, raw_throttle, 0.0, DT);
            car.step(&CarControls::new(throttle, steer, brake, false), SurfaceType::Asphalt, DT);
            out.push((car.state.angular_velocity.abs(), car.state.sideslip_angle.abs()));
        }
        out
    }

    /// Scenario: Presets feel different in a measurable order (turn-in time)
    ///
    /// Given the 4 presets
    /// When a steer key is pressed at 25 m/s
    /// Then the time to 90% of steady yaw is ordered Smooth > Balanced > Sharp > Raw, each adjacent gap >= 15%
    #[test]
    fn test_turn_in_time_is_ordered_by_preset() {
        let times = SteeringProfile::PRESETS.map(|p| {
            let trace = drive(p, 25.0, false, 2.0);
            let steady = trace[trace.len() - 60..].iter().map(|s| s.0).sum::<f32>() / 60.0;
            let idx = trace.iter().position(|s| s.0 >= 0.9 * steady).unwrap_or(trace.len());
            idx as f32 * DT * 1000.0
        });
        println!("time to 90% steady yaw @25 m/s (ms): Smooth {:.0} Balanced {:.0} Sharp {:.0} Raw {:.0}", times[0], times[1], times[2], times[3]);
        for k in 0..3 {
            assert!(times[k + 1] <= times[k] * 0.85, "preset {} must turn in >= 15% faster than {}: {times:?}", k + 1, k);
        }
    }

    /// Scenario: Presets feel different in a measurable order (authority)
    ///
    /// Given the 4 presets
    /// When the steer key is held at 45 m/s
    /// Then the peak yaw is ordered Smooth < Balanced < Sharp < Raw, each adjacent gap >= 8%
    #[test]
    fn test_peak_yaw_at_speed_is_ordered_by_preset() {
        let peaks = SteeringProfile::PRESETS.map(|p| drive(p, 45.0, false, 2.0).iter().map(|s| s.0).fold(0.0f32, f32::max));
        println!("peak yaw @45 m/s (rad/s): Smooth {:.3} Balanced {:.3} Sharp {:.3} Raw {:.3}", peaks[0], peaks[1], peaks[2], peaks[3]);
        for k in 0..3 {
            assert!(peaks[k + 1] >= peaks[k] * 1.08, "preset {} must reach >= 8% more peak yaw than {}: {peaks:?}", k + 1, k);
        }
    }

    /// Scenario: Safe presets do not spin on a held key
    ///
    /// Given Smooth or Balanced, at 45 m/s with W held
    /// When full steer is held for 2 s
    /// Then the peak body sideslip stays below 0.25 rad
    #[test]
    fn test_safe_presets_do_not_spin_on_held_key() {
        for p in [SteeringProfile::Smooth, SteeringProfile::Balanced] {
            let peak = drive(p, 45.0, true, 2.0).iter().map(|s| s.1).fold(0.0f32, f32::max);
            println!("{p:?}: peak sideslip {peak:.3} rad (W + full steer held @45 m/s)");
            assert!(peak < 0.25, "{p:?} spun: peak sideslip {peak:.3} rad");
        }
    }
}

/// Scenario: The model is numerically stable (catalog part)
///
/// Given every catalog car (real and classic arcade), as a bot and as a Raw-preset human
/// When 20 s of random inputs run, including full throttle, full brake, handbrake and reverse
/// Then no state becomes non-finite, |omega_wheel| stays <= 550 rad/s and speed stays bounded
#[test]
fn test_catalog_random_input_fuzz_is_numerically_stable() {
    use tdrace_app::catalog::{ALL_REAL_CARS, CLASSIC_ARCADE_CARS};
    use tdrace_core::physics::car::{Car, CarControls};
    use tdrace_core::physics::config::PlayerHandling;
    use tdrace_core::physics::surface::SurfaceType;

    let surfaces = [SurfaceType::Asphalt, SurfaceType::Gravel, SurfaceType::Grass, SurfaceType::SheetIce, SurfaceType::DeepSand];
    for model in ALL_REAL_CARS.iter().chain(CLASSIC_ARCADE_CARS.iter()) {
        for human in [false, true] {
            let mut cfg = model.to_car_config();
            if human {
                cfg.player = PlayerHandling::human(1.15, 0.0);
            }
            let top = cfg.top_speed_mps;
            let mut car = Car::new(cfg);
            let mut state: u64 = 0x2545_F491_4F6C_DD1D ^ model.id.len() as u64;
            let mut rng = move || {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                ((state >> 33) as f32) / (u32::MAX >> 1) as f32
            };
            let (mut ctrl, mut surf) = (CarControls::default(), SurfaceType::Asphalt);
            for step in 0..(20 * 120) {
                if step % 30 == 0 {
                    ctrl = CarControls {
                        throttle: if rng() > 0.3 { 1.0 } else { rng() },
                        steer: rng() * 2.0 - 1.0,
                        brake: if rng() > 0.8 { rng() } else { 0.0 },
                        handbrake: rng() > 0.9,
                        reverse: car.state.speed < 1.0 && rng() > 0.8,
                    };
                    surf = surfaces[(rng() * surfaces.len() as f32) as usize % surfaces.len()];
                }
                car.step(&ctrl, surf, 1.0 / 120.0);
                let s = &car.state;
                assert!(
                    s.position.is_finite() && s.velocity.is_finite() && s.angular_velocity.is_finite(),
                    "{} (human={human}) non-finite state at step {step}",
                    model.id
                );
                assert!(s.wheel_assemblies.iter().all(|w| w.angular_velocity.is_finite() && w.angular_velocity.abs() <= 550.0));
                assert!(s.speed < top * 1.3 + 5.0, "{} (human={human}) runaway speed {:.1} m/s", model.id, s.speed);
            }
        }
    }
}
