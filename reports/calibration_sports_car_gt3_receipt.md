# 🏁 Auto-Calibration Receipt: Sports Car

* **Target Benchmark**: SRO GT3 Homologation Window
* **Constraint Regime**: SalisburyRwd { min_power_coast_delta: 0.15, max_preload_nm: 120.0, max_yaw_acceleration_rad_s2: 3.5 }
* **Generations Evaluated**: 35
* **Total Simulations**: 560
* **Execution Time**: 0.503 s
* **Cost Improvement**: 0.9% (Initial: 34.80 -> Final: 34.49)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 6.21 | **6.66** | meters |
| Inside Rear Unloading | 1.6% | **2.4%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 0.67 | **0.78** | rad·s |

✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!
