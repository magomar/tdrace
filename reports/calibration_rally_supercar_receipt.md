# 🏁 Auto-Calibration Receipt: Rally Supercar

* **Target Benchmark**: World RX Supercar Technical Regulations
* **Constraint Regime**: MultiLsdAwd { drive_bias_range: (0.35, 0.65), strict_torque_conservation: true }
* **Generations Evaluated**: 35
* **Total Simulations**: 560
* **Execution Time**: 0.557 s
* **Cost Improvement**: 2.3% (Initial: 32.48 -> Final: 31.72)

## 📊 Dynamic Metrics Telemetry Comparison

| Metric | Baseline | Optimized Calibrated | Unit |
| :--- | :---: | :---: | :---: |
| Turning Circle Diameter | 6.10 | **6.53** | meters |
| Inside Rear Unloading | 3.8% | **3.3%** | load ratio |
| Trail-Braking Peak Yaw Accel | 0.00 | **0.00** | rad/s² |
| Max Body Sideslip | 0.00° | **0.00°** | degrees |
| Differential Slip Integral | 5.22 | **2.30** | rad·s |

⚠️ **Active Constraint Boundaries**:
- *Front Drive Bias Below Range*: deviation = 0.350
