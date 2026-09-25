# 🏁 Auto-Calibration Receipt: Cup Stock Car

* **Target Benchmark**: Trans-Am TA1 Homologation Spec
* **Constraint Regime**: SpoolAxle { max_turning_diameter_m: 11.2, min_caster_jacking_unloading_ratio: 0.0 }
* **Generations Evaluated**: 30
* **Total Simulations**: 480
* **Execution Time**: 0.442 s
* **Cost Improvement**: 2.0% (Initial: 28.23 -> Final: 27.66)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 10.77 | **11.04** | meters |
| Inside Rear Unloading | 0.3% | **0.1%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 0.00 | **0.00** | rad·s |

✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!

### ⚙️ Calibrated Vehicle Parameters (Rust Source Representation)

```rust
speed_sensitive_steer_factor: 0.00145,
angular_damping: 148.0,
weight_transfer_lateral: 0.81,
weight_transfer_longitudinal: 0.99,
brake_bias: 0.57,
max_steer_angle: 0.47,
caster_jacking_factor: 0.00,
drive_bias: 0.00,
rear_differential: Spool,
```
