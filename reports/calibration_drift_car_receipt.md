# 🏁 Auto-Calibration Receipt: Drift Machine

* **Target Benchmark**: SRO GT3 Homologation Window
* **Constraint Regime**: SalisburyRwd { min_power_coast_delta: 0.15, max_preload_nm: 120.0, max_yaw_acceleration_rad_s2: 3.5 }
* **Generations Evaluated**: 25
* **Total Simulations**: 400
* **Execution Time**: 0.379 s
* **Cost Improvement**: 0.9% (Initial: 34.97 -> Final: 34.64)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 5.13 | **5.64** | meters |
| Inside Rear Unloading | 4.2% | **4.6%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 1.00 | **0.97** | rad·s |

✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!
