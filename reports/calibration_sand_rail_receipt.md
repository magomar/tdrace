# 🏁 Auto-Calibration Receipt: Sand Rail Buggy

* **Target Benchmark**: Extreme Off-Road Sand Rail Regulation
* **Constraint Regime**: SpoolAxle { max_turning_diameter_m: 7.5, min_caster_jacking_unloading_ratio: 0.0 }
* **Generations Evaluated**: 30
* **Total Simulations**: 480
* **Execution Time**: 0.448 s
* **Cost Improvement**: 18.2% (Initial: 20.83 -> Final: 17.04)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 6.17 | **7.20** | meters |
| Inside Rear Unloading | 17.2% | **22.1%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 0.00 | **0.00** | rad·s |

✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!

### ⚙️ Calibrated Vehicle Parameters (Rust Source Representation)

```rust
speed_sensitive_steer_factor: 0.00167,
angular_damping: 135.7,
weight_transfer_lateral: 1.65,
weight_transfer_longitudinal: 1.06,
brake_bias: 0.58,
max_steer_angle: 0.65,
caster_jacking_factor: 0.00,
drive_bias: 0.00,
rear_differential: Spool,
```
