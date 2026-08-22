use tdrace_core::physics::{Car, CarConfig, CarControls, SurfaceType};

// Simple deterministic PRNG (Linear Congruential Generator) for reproducible input generation
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = (self.state >> 32) as u32;
        (val as f32) / (u32::MAX as f32)
    }
}

#[test]
fn test_exact_bit_identical_determinism() {
    let dt = 1.0 / 60.0;
    let total_steps = 10_000;

    // Generate deterministic control and surface stream
    let mut rng = SimpleRng::new(0xDEADBEEF_CAFE1234);
    let mut control_stream = Vec::with_capacity(total_steps);
    let mut surface_stream = Vec::with_capacity(total_steps);

    let surfaces = [
        SurfaceType::Asphalt,
        SurfaceType::Curb,
        SurfaceType::Grass,
        SurfaceType::Sand,
        SurfaceType::Oil,
        SurfaceType::Ice,
    ];

    for _ in 0..total_steps {
        let throttle = rng.next_f32();
        let steer = rng.next_f32() * 2.0 - 1.0;
        let brake = if rng.next_f32() > 0.85 { rng.next_f32() } else { 0.0 };
        let handbrake = rng.next_f32() > 0.92;
        let reverse = rng.next_f32() > 0.98;

        let ctrl = CarControls {
            throttle,
            steer,
            brake,
            handbrake,
            reverse,
        };
        let surf_idx = (rng.next_f32() * surfaces.len() as f32) as usize % surfaces.len();
        control_stream.push(ctrl);
        surface_stream.push(surfaces[surf_idx]);
    }

    // Run Car 1
    let mut car1 = Car::new(CarConfig::sports_car());
    for i in 0..total_steps {
        car1.step(&control_stream[i], surface_stream[i], dt);
    }

    // Run Car 2 with identical inputs
    let mut car2 = Car::new(CarConfig::sports_car());
    for i in 0..total_steps {
        car2.step(&control_stream[i], surface_stream[i], dt);
    }

    // Compare states bit-for-bit
    assert_eq!(
        car1.state.position.x.to_bits(),
        car2.state.position.x.to_bits(),
        "Position X bits mismatch"
    );
    assert_eq!(
        car1.state.position.y.to_bits(),
        car2.state.position.y.to_bits(),
        "Position Y bits mismatch"
    );
    assert_eq!(
        car1.state.velocity.x.to_bits(),
        car2.state.velocity.x.to_bits(),
        "Velocity X bits mismatch"
    );
    assert_eq!(
        car1.state.velocity.y.to_bits(),
        car2.state.velocity.y.to_bits(),
        "Velocity Y bits mismatch"
    );
    assert_eq!(
        car1.state.angle.to_bits(),
        car2.state.angle.to_bits(),
        "Angle bits mismatch"
    );
    assert_eq!(
        car1.state.angular_velocity.to_bits(),
        car2.state.angular_velocity.to_bits(),
        "Angular velocity bits mismatch"
    );
    assert_eq!(
        car1.state.steer_angle.to_bits(),
        car2.state.steer_angle.to_bits(),
        "Steer angle bits mismatch"
    );
    assert_eq!(
        car1.state.drift_score.to_bits(),
        car2.state.drift_score.to_bits(),
        "Drift score bits mismatch"
    );

    for w in 0..4 {
        assert_eq!(
            car1.state.wheels[w].normal_load.to_bits(),
            car2.state.wheels[w].normal_load.to_bits()
        );
        assert_eq!(
            car1.state.wheels[w].lateral_force.to_bits(),
            car2.state.wheels[w].lateral_force.to_bits()
        );
        assert_eq!(
            car1.state.wheels[w].longitudinal_force.to_bits(),
            car2.state.wheels[w].longitudinal_force.to_bits()
        );
        assert_eq!(
            car1.state.wheels[w].slip_angle.to_bits(),
            car2.state.wheels[w].slip_angle.to_bits()
        );
    }

    println!(
        "✅ Verified bit-identical determinism across {} steps! Final pos: {:?}",
        total_steps, car1.state.position
    );
}

#[test]
fn test_save_restore_rewind_determinism() {
    let dt = 1.0 / 60.0;
    let mut rng = SimpleRng::new(0x1337BEEF);
    let mut controls = Vec::new();
    for _ in 0..1000 {
        controls.push(CarControls::new(
            rng.next_f32(),
            rng.next_f32() * 2.0 - 1.0,
            if rng.next_f32() > 0.8 { rng.next_f32() } else { 0.0 },
            rng.next_f32() > 0.9,
        ));
    }

    // Step 1: Run reference simulation for 1000 steps
    let mut ref_car = Car::new(CarConfig::drift_car());
    let mut checkpoint_500 = None;
    for (i, ctrl) in controls.iter().enumerate() {
        if i == 500 {
            checkpoint_500 = Some(ref_car.state().clone());
        }
        ref_car.step(ctrl, SurfaceType::Asphalt, dt);
    }
    let ref_final_state = ref_car.state().clone();

    // Step 2: Run new car for 500 steps, mutate it completely, then restore checkpoint and continue
    let mut test_car = Car::new(CarConfig::drift_car());
    for ctrl in &controls[0..500] {
        test_car.step(ctrl, SurfaceType::Asphalt, dt);
    }
    // Mutate state with garbage
    for _ in 0..200 {
        test_car.step(&CarControls::new(1.0, 1.0, 1.0, true), SurfaceType::Sand, dt);
    }

    // Restore checkpoint at 500
    test_car.set_state(checkpoint_500.unwrap());

    // Continue stepping from 500 to 1000
    for ctrl in &controls[500..1000] {
        test_car.step(ctrl, SurfaceType::Asphalt, dt);
    }

    assert_eq!(
        test_car.state(),
        &ref_final_state,
        "Restored simulation state diverged from reference simulation"
    );
    println!("✅ Save/Restore rewind determinism verified!");
}

#[test]
fn test_json_state_serialization() {
    let mut car = Car::new(CarConfig::sports_car());
    for _ in 0..50 {
        car.step(&CarControls::new(0.8, 0.4, 0.0, true), SurfaceType::Asphalt, 1.0 / 60.0);
    }

    let json_str = serde_json::to_string(&car.state).expect("Failed to serialize CarState");
    assert!(!json_str.is_empty());

    let deserialized: tdrace_core::physics::CarState =
        serde_json::from_str(&json_str).expect("Failed to deserialize CarState");

    assert_eq!(car.state.position, deserialized.position);
    assert_eq!(car.state.velocity, deserialized.velocity);
    assert_eq!(car.state.angle, deserialized.angle);
    assert_eq!(car.state.angular_velocity, deserialized.angular_velocity);
    assert_eq!(car.state.is_drifting, deserialized.is_drifting);
}
