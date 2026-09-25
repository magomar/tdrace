# 🏁 Auto-Calibration Receipt: Sports Car

* **Target Benchmark**: SRO GT3 Homologation Window
* **Constraint Regime**: SalisburyRwd { min_power_coast_delta: 0.15, max_preload_nm: 120.0, max_yaw_acceleration_rad_s2: 3.5 }
* **Generations Evaluated**: 30
* **Total Simulations**: 480
* **Execution Time**: 0.451 s
* **Cost Improvement**: 0.4% (Initial: 34.83 -> Final: 34.70)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 6.18 | **6.21** | meters |
| Inside Rear Unloading | 1.6% | **2.4%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 0.67 | **0.65** | rad·s |

✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!

### ⚙️ Calibrated Vehicle Parameters (Rust Source Representation)

```rust
speed_sensitive_steer_factor: 0.00456,
angular_damping: 120.3,
weight_transfer_lateral: 1.12,
weight_transfer_longitudinal: 0.70,
brake_bias: 0.56,
max_steer_angle: 0.68,
caster_jacking_factor: 0.00,
drive_bias: 0.00,
rear_differential: LimitedSlip { power_lock: 0.55590105, coast_lock: 0.33140507, preload_nm: 65.40405 },
```
