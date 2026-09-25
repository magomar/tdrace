# 🏁 Auto-Calibration Receipt: Sprint Kart

* **Target Benchmark**: FIA Karting Sprint Regulation
* **Constraint Regime**: SpoolAxle { max_turning_diameter_m: 2.6, min_caster_jacking_unloading_ratio: 0.8 }
* **Generations Evaluated**: 30
* **Total Simulations**: 480
* **Execution Time**: 0.446 s
* **Cost Improvement**: 4.5% (Initial: 22.45 -> Final: 21.43)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 2.26 | **2.54** | meters |
| Inside Rear Unloading | 95.0% | **95.0%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 0.00 | **0.00** | rad·s |

✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!

### ⚙️ Calibrated Vehicle Parameters (Rust Source Representation)

```rust
speed_sensitive_steer_factor: 0.00141,
angular_damping: 26.9,
weight_transfer_lateral: 0.83,
weight_transfer_longitudinal: 1.03,
brake_bias: 0.54,
max_steer_angle: 0.65,
caster_jacking_factor: 1.73,
drive_bias: 0.00,
rear_differential: Spool,
```
