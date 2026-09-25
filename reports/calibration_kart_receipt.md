# 🏁 Auto-Calibration Receipt: Sprint Kart

* **Target Benchmark**: FIA Karting Sprint Regulation
* **Constraint Regime**: SpoolAxle { max_turning_diameter_m: 2.6, min_caster_jacking_unloading_ratio: 0.8 }
* **Generations Evaluated**: 35
* **Total Simulations**: 560
* **Execution Time**: 0.518 s
* **Cost Improvement**: 6.3% (Initial: 22.27 -> Final: 20.87)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 2.13 | **2.52** | meters |
| Inside Rear Unloading | 95.0% | **95.0%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 0.00 | **0.00** | rad·s |

✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!
