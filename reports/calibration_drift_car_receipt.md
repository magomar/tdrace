# 🏁 Auto-Calibration Receipt: Drift Machine

* **Target Benchmark**: SRO GT3 Homologation Window
* **Constraint Regime**: SalisburyRwd { min_power_coast_delta: 0.15, max_preload_nm: 120.0, max_yaw_acceleration_rad_s2: 3.5 }
* **Generations Evaluated**: 30
* **Total Simulations**: 480
* **Execution Time**: 0.434 s
* **Cost Improvement**: 0.6% (Initial: 35.30 -> Final: 35.09)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 5.09 | **5.12** | meters |
| Inside Rear Unloading | 4.2% | **4.6%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 1.01 | **0.90** | rad·s |

✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!

### ⚙️ Calibrated Vehicle Parameters (Rust Source Representation)

```rust
speed_sensitive_steer_factor: 0.00371,
angular_damping: 113.6,
weight_transfer_lateral: 1.05,
weight_transfer_longitudinal: 0.71,
brake_bias: 0.56,
max_steer_angle: 0.78,
caster_jacking_factor: 0.00,
drive_bias: 0.00,
rear_differential: LimitedSlip { power_lock: 0.47669387, coast_lock: 0.16373028, preload_nm: 61.98726 },
```
