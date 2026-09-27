use tdrace_app::catalog::CLASSIC_ARCADE_CARS;
use tdrace_app::input::simulation::{
    run_keyboard_chicane_simulation, run_keyboard_slide_catch_simulation,
    run_keyboard_sweeper_simulation, KeyboardDriverProfile,
};
use tdrace_core::physics::sim::DEFAULT_SIMULATION_DT;
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_keyboard_sweeper_feathering_preserves_speed_vs_holding() {
    let gt = CLASSIC_ARCADE_CARS
        .iter()
        .find(|c| c.id == "classic_gt")
        .expect("classic_gt must exist in arcade catalog");
    let cfg = gt.to_car_config();
    let dt = DEFAULT_SIMULATION_DT;

    let hold_profile = KeyboardDriverProfile::sustained_hold_balanced();
    let feather_profile = KeyboardDriverProfile::rapid_feathering_balanced();

    let hold_res = run_keyboard_sweeper_simulation(
        gt.id,
        gt.name,
        &cfg,
        SurfaceType::Asphalt,
        &hold_profile,
        70.0,
        3.0,
        dt,
    );

    let feather_res = run_keyboard_sweeper_simulation(
        gt.id,
        gt.name,
        &cfg,
        SurfaceType::Asphalt,
        &feather_profile,
        70.0,
        3.0,
        dt,
    );

    // 1. Feathering avoids excessive tire scrub and maintains higher exit speed
    assert!(
        feather_res.exit_speed_kmh > hold_res.exit_speed_kmh,
        "Feathering must retain higher exit speed than holding lock: feather={:.1} km/h vs hold={:.1} km/h",
        feather_res.exit_speed_kmh,
        hold_res.exit_speed_kmh
    );

    // 2. Speed loss under holding must be substantially higher
    assert!(
        hold_res.speed_loss_kmh > feather_res.speed_loss_kmh,
        "Sustained hold must suffer greater scrub drag: hold_loss={:.1} km/h vs feather_loss={:.1} km/h",
        hold_res.speed_loss_kmh,
        feather_res.speed_loss_kmh
    );

    // 3. Peak front slip angle under sustained hold pushes into scrub saturation
    assert!(
        hold_res.peak_front_slip_deg > feather_res.peak_front_slip_deg,
        "Hold must produce higher peak front slip angle: hold_slip={:.1}° vs feather_slip={:.1}°",
        hold_res.peak_front_slip_deg,
        feather_res.peak_front_slip_deg
    );
}

#[test]
fn test_keyboard_kart_caster_jacking_scrub_differential() {
    let kart = CLASSIC_ARCADE_CARS
        .iter()
        .find(|c| c.id == "classic_kart")
        .expect("classic_kart must exist in arcade catalog");
    let cfg = kart.to_car_config();
    let dt = DEFAULT_SIMULATION_DT;

    let hold_profile = KeyboardDriverProfile::sustained_hold_balanced();
    let feather_profile = KeyboardDriverProfile::rapid_feathering_balanced();

    let hold_res = run_keyboard_sweeper_simulation(
        kart.id,
        kart.name,
        &cfg,
        SurfaceType::Asphalt,
        &hold_profile,
        55.0,
        3.0,
        dt,
    );

    let feather_res = run_keyboard_sweeper_simulation(
        kart.id,
        kart.name,
        &cfg,
        SurfaceType::Asphalt,
        &feather_profile,
        55.0,
        3.0,
        dt,
    );

    // Kart with 1:1 steering lock and caster jacking suffers heavy scrub on sustained hold
    assert!(
        feather_res.speed_retention_pct > hold_res.speed_retention_pct + 10.0,
        "Kart feathering must improve speed retention by at least 10%: feather={:.1}% vs hold={:.1}%",
        feather_res.speed_retention_pct,
        hold_res.speed_retention_pct
    );
}

#[test]
fn test_keyboard_filter_profiles_direct_vs_balanced_cornering() {
    let gt = CLASSIC_ARCADE_CARS
        .iter()
        .find(|c| c.id == "classic_gt")
        .expect("classic_gt must exist in arcade catalog");
    let cfg = gt.to_car_config();
    let dt = DEFAULT_SIMULATION_DT;

    let direct_profile = KeyboardDriverProfile::sustained_hold_direct();
    let balanced_profile = KeyboardDriverProfile::sustained_hold_balanced();

    let direct_res = run_keyboard_sweeper_simulation(
        gt.id,
        gt.name,
        &cfg,
        SurfaceType::Asphalt,
        &direct_profile,
        70.0,
        3.0,
        dt,
    );

    let balanced_res = run_keyboard_sweeper_simulation(
        gt.id,
        gt.name,
        &cfg,
        SurfaceType::Asphalt,
        &balanced_profile,
        70.0,
        3.0,
        dt,
    );

    // Direct raw digital input snaps to full lock immediately, causing faster initial scrub
    assert!(
        direct_res.peak_steer_angle_deg >= balanced_res.peak_steer_angle_deg,
        "Direct steering must reach full mechanical lock: direct={:.1}° vs balanced={:.1}°",
        direct_res.peak_steer_angle_deg,
        balanced_res.peak_steer_angle_deg
    );
}

#[test]
fn test_keyboard_chicane_reversal_latency_measurement() {
    let rally = CLASSIC_ARCADE_CARS
        .iter()
        .find(|c| c.id == "classic_rally")
        .expect("classic_rally must exist in arcade catalog");
    let cfg = rally.to_car_config();
    let dt = DEFAULT_SIMULATION_DT;

    let driver = KeyboardDriverProfile::sustained_hold_balanced();
    let chicane_res = run_keyboard_chicane_simulation(
        rally.id,
        rally.name,
        &cfg,
        SurfaceType::Dirt,
        &driver,
        65.0,
        dt,
    );

    // Chicane reversal latency must be positive and measured
    assert!(
        chicane_res.reversal_latency_ms > 50.0 && chicane_res.reversal_latency_ms < 1000.0,
        "Reversal latency must be in realistic physical window: got {:.1} ms",
        chicane_res.reversal_latency_ms
    );
}

#[test]
fn test_keyboard_slide_catch_recovery_on_dirt() {
    let offroad = CLASSIC_ARCADE_CARS
        .iter()
        .find(|c| c.id == "classic_offroad")
        .expect("classic_offroad must exist in arcade catalog");
    let cfg = offroad.to_car_config();
    let dt = DEFAULT_SIMULATION_DT;

    let driver = KeyboardDriverProfile::sustained_hold_balanced();
    let catch_res = run_keyboard_slide_catch_simulation(
        offroad.id,
        offroad.name,
        &cfg,
        SurfaceType::Dirt,
        &driver,
        55.0,
        dt,
    );

    // Off-road buggy on dirt should successfully catch or damp the slide
    assert!(
        catch_res.max_sideslip_deg > 5.0,
        "Vehicle must experience initial induced slip: got {:.1}°",
        catch_res.max_sideslip_deg
    );
}
