# 🔬 Cross-Drivetrain & Differential Dynamic Calibration Analysis

## 🎯 Executive Summary & Architectural Overview

Following the implementation of **Spec 034 (Explicit Drivetrain Differential Models)** and **Spec 035 (Automated Parameter Optimization & Constrained Calibration)**, we executed systematic automated calibration runs across all representative motorsport modalities and drivetrains in **TdRace**:

1. **Sprint Kart** (Modality: *Karting*, Weight: 165 kg) — **100% Solid Spool Axle + Mechanical Caster-Jacking**
2. **GT Sports Car** (Modality: *GT / Road*, Weight: 1,120 kg) — **Salisbury Asymmetric Multi-Plate RWD LSD**
3. **Drift Machine** (Modality: *Drift*, Weight: 980 kg) — **Salisbury 2-Way High-Lock RWD LSD**
4. **Sand Rail Buggy** (Modality: *Extreme Off-Road*, Weight: 590 kg) — **100% Solid Spool Axle + High Flotation Paddle Tires**
5. **Rally Supercar** (Modality: *Rally / Rallycross*, Weight: 1,230 kg) — **Multi-LSD Torque-Conserving AWD**
6. **Cup Stock Car TA1** (Modality: *NASCAR / Trans-Am*, Weight: 1,450 kg) — **High-Inertia Solid Spool Live Axle**

---

## ❓ Addressing Drivetrain & Differential Questions

### 1. Do Road and Extreme Off-Road share the same spool mechanism?
**No. Road vehicles do not use a spool at all, and off-road spools operate under fundamentally different surface dynamics than karts.**

| Attribute | Sprint Kart | GT Road Car | Extreme Off-Road Buggy (`sand_rail`) |
| :--- | :---: | :---: | :---: |
| **Differential Type** | **Solid Spool Axle** (no diff) | **Salisbury Limited-Slip (LSD)** | **Solid Locked Spool** |
| **Operating Surface** | High-grip asphalt ($\mu \approx 1.0\text{--}1.2$) | High-grip asphalt ($\mu \approx 1.0\text{--}1.2$) | Sand / Gravel / Dirt ($\mu \approx 0.4\text{--}0.7$) |
| **Turning Mechanism** | **Chassis flex & caster-jacking** lifts inside rear tire off pavement. | **Clutch slip** allows left and right wheels to rotate at different speeds. | **Low surface shear** allows outer/inner tire slip without bogging. |
| **Why not an Open Diff?** | Inapplicable (solid axle tube). | An open diff spins away power on the unloaded inside wheel. | An open diff in sand causes "one-wheel peel", instantly digging a hole and stranding the buggy. |
| **Why not a Spool on Road?** | N/A (Standard karting regulation). | Severe low-speed push/understeer, tire scrub, and drivetrain windup on asphalt. | Perfect for sand: both paddle tires deliver 100% drive torque simultaneously. |

### 2. Are the calibrations of one valid for the others?
**Definitely not.** Calibrations cannot be shared across these vehicles due to three physical laws:

1. **Mass and Inertia Scaling ($F = m \cdot a$ and $\tau = I_z \cdot \ddot{\psi}$):**
   - Kart polar inertia: $95\text{ kg}\cdot\text{m}^2$ (weight $165\text{ kg}$).
   - Sand Rail polar inertia: $720\text{ kg}\cdot\text{m}^2$ (weight $590\text{ kg}$).
   - GT3 Sports Car polar inertia: $1,750\text{ kg}\cdot\text{m}^2$ (weight $1,120\text{ kg}$).
   - Stock Car polar inertia: $2,150\text{ kg}\cdot\text{m}^2$ (weight $1,450\text{ kg}$).
   *Applying kart steering rates or damping to a 1,450kg car creates immediate computational divergence or unrealistic twitchiness; conversely, applying GT damping to a kart destroys its agile, razor-sharp response.*

2. **Wheelbase & Geometric Turning Limit:**
   - Kart wheelbase: $1.04\text{ m} \implies$ geometric turning diameter $D \approx 2.50\text{ m}$.
   - Buggy wheelbase: $2.40\text{ m} \implies$ turning diameter $D \approx 7.20\text{ m}$.
   - GT3 / TA1 wheelbase: $2.55\text{--}2.74\text{ m} \implies$ turning diameter $D \approx 10.9\text{--}11.2\text{ m}$.
   *Forcing a 2.5m constraint on a buggy or stock car is mathematically infeasible.*

3. **Caster-Jacking Necessity:**
   - On a kart, because a spool *must* turn on high-traction asphalt, front kingpin caster-jacking is mandatory ($\text{caster\_jacking\_factor} \approx 0.8\text{--}1.2$) to transfer vertical load from the inside rear to the outside front.
   - On a Sand Rail or Stock Car, suspension is independent double A-arm or live multi-link with long travel ($\text{caster\_jacking\_factor} = 0.0$). A buggy relies on tire slip in soft sand and throttle steering (power sliding), not chassis twist.

---

## 📊 Automated Calibration Results Across Drivetrains

| Vehicle | Drivetrain Constraint | Target Benchmark | Initial Cost | Final Cost | Cost Δ | Calibrated Turning Circle | Unloading / Preload |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **Sprint Kart** | `SpoolAxle` ($D \le 2.6\text{m}, U \ge 80\%$) | FIA Karting Sprint | 22.27 | **20.87** | **-6.3%** | **2.52 m** | **95.0%** unloading |
| **Sports Car** | `SalisburyRwd` ($\Delta \ge 0.15, \ddot{\psi} \le 3.5$) | SRO GT3 Homologation | 34.80 | **34.49** | **-0.9%** | **6.66 m** | **60 Nm** preload |
| **Drift Machine** | `SalisburyRwd` (High Lock 2-way) | Drift / SRO GT3 | 34.97 | **34.64** | **-0.9%** | **5.64 m** | **120 Nm** preload |
| **Sand Rail Buggy** | `SpoolAxle` ($D \le 7.5\text{m}, U = 0\%$) | Extreme Off-Road Spec | 20.43 | **16.85** | **-17.5%** | **7.19 m** | **0.0%** caster jack |
| **Rally Supercar** | `MultiLsdAwd` (Drive bias $35\text{--}65\%$) | World RX Supercar | 32.48 | **31.72** | **-2.3%** | **6.53 m** | **-56%** slip loss |
| **Cup Stock Car** | `SpoolAxle` ($D \le 11.2\text{m}, U = 0\%$) | Trans-Am TA1 Spec | 28.08 | **27.64** | **-1.6%** | **11.06 m** | **0.0%** caster jack |

---

## 🏁 Summary of Verified Receipts
Individual audit receipts for each optimization run have been generated under `reports/`:
- [`reports/calibration_kart_receipt.md`](reports/calibration_kart_receipt.md)
- [`reports/calibration_sports_car_gt3_receipt.md`](reports/calibration_sports_car_gt3_receipt.md)
- [`reports/calibration_drift_car_receipt.md`](reports/calibration_drift_car_receipt.md)
- [`reports/calibration_sand_rail_receipt.md`](reports/calibration_sand_rail_receipt.md)
- [`reports/calibration_rally_supercar_receipt.md`](reports/calibration_rally_supercar_receipt.md)
- [`reports/calibration_stock_car_receipt.md`](reports/calibration_stock_car_receipt.md)
