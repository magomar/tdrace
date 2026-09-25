# 🏁 Auto-Calibration Receipt: Cup Stock Car

* **Target Benchmark**: Trans-Am TA1 Homologation Spec
* **Constraint Regime**: SpoolAxle { max_turning_diameter_m: 11.2, min_caster_jacking_unloading_ratio: 0.0 }
* **Generations Evaluated**: 35
* **Total Simulations**: 560
* **Execution Time**: 0.520 s
* **Cost Improvement**: 1.6% (Initial: 28.08 -> Final: 27.64)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 10.91 | **11.06** | meters |
| Inside Rear Unloading | 0.3% | **0.4%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 0.00 | **0.00** | rad·s |

✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!
