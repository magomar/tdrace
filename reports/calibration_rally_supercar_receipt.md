# 🏁 Auto-Calibration Receipt: Rally Supercar

* **Target Benchmark**: World RX Supercar Technical Regulations
* **Constraint Regime**: MultiLsdAwd { drive_bias_range: (0.35, 0.65), strict_torque_conservation: true }
* **Generations Evaluated**: 30
* **Total Simulations**: 480
* **Execution Time**: 0.476 s
* **Cost Improvement**: 1.4% (Initial: 32.54 -> Final: 32.07)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 6.07 | **6.08** | meters |
| Inside Rear Unloading | 3.8% | **5.2%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 5.29 | **2.88** | rad·s |

⚠️ **Active Constraint Boundaries**:
- *Front Drive Bias Below Range*: deviation = 0.350

### ⚙️ Calibrated Vehicle Parameters (Rust Source Representation)

```rust
speed_sensitive_steer_factor: 0.00103,
angular_damping: 126.4,
weight_transfer_lateral: 1.24,
weight_transfer_longitudinal: 0.70,
brake_bias: 0.62,
max_steer_angle: 0.68,
caster_jacking_factor: 0.00,
drive_bias: 0.35,
rear_differential: LimitedSlip { power_lock: 0.7, coast_lock: 0.5, preload_nm: 85.0 },
```
