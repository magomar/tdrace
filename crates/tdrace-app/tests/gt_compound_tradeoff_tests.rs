//! Spec 098: the slick compound trades grip for wear on a GT car.

use glam::Vec2;
use tdrace_app::module::gt::GtWorldChallengeModule;
use tdrace_app::profile::GT_GARAGE_COMPOUNDS;
use tdrace_core::physics::sim::{run_protocol_c, SimulationRunner, DEFAULT_SIMULATION_DT};
use tdrace_core::physics::surface::CompoundId;
use tdrace_core::physics::{CarConfig, CarControls, SurfaceType};

const SLICKS: [CompoundId; 3] = [CompoundId::SoftSlick, CompoundId::MediumSlick, CompoundId::HardSlick];

fn gt3_on(compound: CompoundId) -> CarConfig {
    let mut cfg = GtWorldChallengeModule::car_gt3_evo();
    cfg.set_slick_compound(compound);
    cfg
}

/// Mean tread wear of the four wheels after a 60 s sustained power drift on asphalt:
/// a handbrake kick at the start breaks the rear loose, then a held steer and a speed-holding
/// throttle keep the car sliding past the peak slip angle.
fn drift_wear_60s(compound: CompoundId) -> f32 {
    let cfg = gt3_on(compound);
    let v0 = 80.0 / 3.6;
    let mut runner = SimulationRunner::new(cfg, DEFAULT_SIMULATION_DT)
        .with_telemetry_capture(false)
        .with_state(Vec2::ZERO, 0.0, Vec2::new(v0, 0.0));
    runner.run_for(60.0, SurfaceType::Asphalt, |t, car| {
        let v_err = v0 - car.state().speed;
        CarControls {
            throttle: (0.6 + v_err * 0.3).clamp(0.2, 1.0),
            steer: 0.7,
            brake: 0.0,
            handbrake: t < 0.4,
            reverse: false,
        }
    });
    let wear: f32 = runner.car.state().wheel_assemblies.iter().map(|w| w.wear).sum();
    wear / 4.0
}

/// Peak lateral acceleration (g) of the skidpad protocol (R = 30 m) on asphalt.
fn skidpad_peak_g(compound: CompoundId) -> f32 {
    run_protocol_c(&gt3_on(compound), SurfaceType::Asphalt, 30.0, DEFAULT_SIMULATION_DT).peak_lateral_accel_g
}

#[test]
fn the_garage_slicks_are_the_three_compounds_of_the_spec() {
    assert_eq!(GT_GARAGE_COMPOUNDS, SLICKS);
}

#[test]
fn peak_lateral_grip_orders_soft_medium_hard() {
    let [soft, medium, hard] = SLICKS.map(skidpad_peak_g);
    println!("skidpad peak lateral g: soft {soft:.3}, medium {medium:.3}, hard {hard:.3}");
    assert!(soft > medium, "Soft {soft:.3} g must beat Medium {medium:.3} g");
    assert!(medium > hard, "Medium {medium:.3} g must beat Hard {hard:.3} g");
}

#[test]
fn drift_tread_wear_orders_soft_medium_hard() {
    let [soft, medium, hard] = SLICKS.map(drift_wear_60s);
    println!("60 s drift tread wear: soft {soft:.3}, medium {medium:.3}, hard {hard:.3}");
    assert!(hard > 0.02, "the drift must wear the Hard tyre ({hard:.3}), or the ordering means nothing");
    assert!(soft < 1.0, "a fully worn Soft tyre ({soft:.3}) would hide the ordering");
    assert!(soft > medium, "Soft {soft:.3} must wear more than Medium {medium:.3}");
    assert!(medium > hard, "Medium {medium:.3} must wear more than Hard {hard:.3}");
}

#[test]
fn medium_is_todays_behaviour() {
    // Fitting Medium through the garage path leaves the GT3 exactly as its preset.
    let preset = GtWorldChallengeModule::car_gt3_evo();
    let medium = gt3_on(CompoundId::MediumSlick);
    assert_eq!(medium.tire, preset.tire);
    assert_eq!(medium.wheels, preset.wheels);
}
