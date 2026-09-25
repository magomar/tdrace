---
type: Technical Report
title: "Explicit Drivetrain Differential Models and Axle Coupling"
description: "Implementation, mathematical validation, and calibration report for explicit drivetrain differential models (Spool, Salisbury Limited-Slip, Open) across the TdRace simulation fleet (Spec 034)."
status: active
category: architecture
spec: "specs/034_explicit_drivetrain_differential_models_and_axle_coupling.md"
epic: "tdrace-0ptm"
---

# ⚙️ Explicit Drivetrain Differential Models & Axle Coupling Report

> **Specification Receipt**: Fulfills [**Architecture Spec 034**](file:///home/mario/workspace/games/tdrace/specs/034_explicit_drivetrain_differential_models_and_axle_coupling.md) under Beads Epic `tdrace-0ptm`.

* **Execution Date**: 2026-09-25
* **Target Engine**: Pure-Rust [`crates/wheelbase`](file:///home/mario/workspace/games/tdrace/crates/wheelbase) at 60 Hz deterministic stepping ($dt = 0.0167\text{ s}$)
* **Beads Task**: `tdrace-0ptm` (Claimed & Verified)
* **Fleet Coverage**: All 6 Modalities (Kart, GT World Challenge, NASCAR, Rallycross / Group B, Extreme Off-Road, Classic Arcade)

---

## 🎯 1. Executive Summary & Physical Realism

Prior to Spec 034, all driven wheels received torque split evenly or via a simplified caster-jacking heuristic, while wheel rotational degrees of freedom operated independently. While numerically simple, this failed to reflect real-world vehicle dynamics:
1. **Kart Inside Wheel Spin**: Real go-karts have a single solid rear axle (no differential). Without cross-axle locking, cornering or aggressive steer could cause rapid single-wheel slip and power loss.
2. **Open Differential Torque Limiting**: In real vehicles with open differentials, if one tire loses traction (e.g. on ice, mud, or unweighted during a curb hop), both tires produce only the traction of the slipping wheel, halving drive power.
3. **Limited-Slip Differential (LSD) Traction Transfer**: Purpose-built GT, touring, and rally cars use multi-plate clutch packs with asymmetric ramp angles to lock under throttle or coast, transferring drive torque to the gripping tire.

Spec 034 introduced **explicit, physically grounded differential models** with cross-axle torque distribution and rotational coupling directly integrated into the 60 Hz deterministic physics pipeline.

```
                    ┌─────────────────────────────────┐
                    │      Total Engine / Drive       │
                    │         Torque Request          │
                    └────────────────┬────────────────┘
                                     │
                                     ▼
                    ┌─────────────────────────────────┐
                    │       Drive Bias Splitter       │
                    │   (FWD / RWD / AWD Center Tq)   │
                    └───────┬─────────────────┬───────┘
                            │                 │
                Front Axle  │                 │  Rear Axle
                            ▼                 ▼
          ┌────────────────────────┐   ┌────────────────────────┐
          │   Front Differential   │   │   Rear Differential    │
          │  [Open / LSD / Spool]  │   │  [Open / LSD / Spool]  │
          └─────┬────────────┬─────┘   └─────┬────────────┬─────┘
                │            │               │            │
                ▼            ▼               ▼            ▼
             Wheel 0      Wheel 1         Wheel 2      Wheel 3
              (FL)         (FR)            (RL)         (RR)
```

---

## 📐 2. Mathematical Formulations & Implementations

### 2.1 Spool Differential (`DifferentialType::Spool`)

A solid live axle or welded differential. The left and right shafts are mechanically locked to turn at the same angular rate.

* **Rotational Coupling**:
  $$\omega_{\text{locked}} = \frac{I_L \cdot \omega_L + I_R \cdot \omega_R}{I_L + I_R}$$
  $$\omega_L \leftarrow \omega_{\text{locked}}, \quad \omega_R \leftarrow \omega_{\text{locked}}$$

* **Drive Force Split**:
  Drive torque is transferred to whichever wheel has normal load and ground grip.
  $$G_L = \max(0, \mu_L \cdot F_{z, L}), \quad G_R = \max(0, \mu_R \cdot F_{z, R})$$
  $$F_{\text{drive}, L} = F_{\text{axle}} \cdot \frac{G_L}{G_L + G_R}, \quad F_{\text{drive}, R} = F_{\text{axle}} \cdot \frac{G_R}{G_L + G_R}$$
  When an inside wheel lifts off the pavement under caster jacking ($F_{z, L} \to 0$), 100% of the drive force automatically shifts to the loaded outside wheel with zero inside wheelspin!

### 2.2 Salisbury Limited-Slip Differential (`DifferentialType::LimitedSlip`)

Models multi-plate clutch pack differentials with asymmetric ramp angles for power (acceleration) and coast (deceleration / engine braking), plus preloaded Belleville springs.

* **Clutch Locking Capacity**:
  $$T_{\text{lock}} = T_{\text{preload}} + K_{\text{ramp}} \cdot |T_{\text{axle}}|$$
  $$F_{\text{clutch, max}} = \min\left( \frac{T_{\text{lock}}}{R_{\text{tire}}}, \frac{|F_{\text{axle}}|}{2} \right)$$
  Where $K_{\text{ramp}} = \text{power\_lock}$ under positive drive thrust, and $K_{\text{ramp}} = \text{coast\_lock}$ under overrun / deceleration.

* **Kinematic Differentiation vs Slip**:
  To prevent artificial torque steer when rolling cleanly without slip through turns, the geometric kinematic speed difference across the axle is subtracted:
  $$\Delta \omega_{\text{kin}} = - \omega_{\text{yaw}} \cdot \frac{\text{track\_width}}{R_{\text{tire}}}$$
  $$\Delta \omega_{\text{slip}} = (\omega_L - \omega_R) - \Delta \omega_{\text{kin}}$$

* **Discrete-Stable Torque Transfer**:
  To eliminate discrete Euler chatter / limit-cycle flip-flops across 60 Hz timesteps:
  $$F_{\text{max\_stable}} = \frac{I_{\text{wheel}} \cdot |\Delta \omega_{\text{slip}}|}{2 \cdot R_{\text{tire}} \cdot \Delta t}$$
  $$\Delta F = \min\left( F_{\text{clutch, max}}, F_{\text{max\_stable}} \right) \cdot \operatorname{sgn}(\Delta \omega_{\text{slip}} \cdot \operatorname{sgn}(F_{\text{axle}}))$$
  $$F_{\text{drive}, L} = \frac{F_{\text{axle}}}{2} - \Delta F, \quad F_{\text{drive}, R} = \frac{F_{\text{axle}}}{2} + \Delta F$$

### 2.3 Open Differential (`DifferentialType::Open`)

Conventional epicyclic bevel gear differential.
* Wheels rotate completely independently: $\omega_L$ and $\omega_R$ are unconstrained.
* Equal torque delivery:
  $$F_{\text{drive}, L} = \frac{F_{\text{axle}}}{2}, \quad F_{\text{drive}, R} = \frac{F_{\text{axle}}}{2}$$
* If one wheel is on zero-traction ice, reaction torque is zero, limiting forward acceleration on both sides.

---

## 🏎️ 3. Fleet Configuration & Calibration Matrix

| Modality | Vehicle Preset | Front Differential | Rear Differential | Physical Motorsport Justification |
| :--- | :--- | :--- | :--- | :--- |
| **Kart** | Classic Kart / Cadet / Superkart | `Open` (Unpowered) | `Spool` | Authentic 30–50mm solid chromoly axle; caster jacking lifts inside rear wheel |
| **GT** | Sports Car / GT3 / GT4 / GT2 | `Open` (Unpowered) | `LimitedSlip` (40% power, 25% coast, 45 Nm) | Multi-plate Salisbury clutch LSD for traction out of corners and trail-braking stability |
| **GT (Hypercar)** | LMP1 / LMH AWD Hybrid | `LimitedSlip` (30% power, 20% coast) | `LimitedSlip` (50% power, 35% coast) | Front electric motor LSD and rear internal combustion transaxle LSD |
| **NASCAR** | Stock Car TA1 / Cup / ARCA | `Open` (Unpowered) | `Spool` | 9-inch Ford locked rear spool axle for high-speed banked oval traction |
| **Off-Road** | Sand Rail / Buggy | `Open` (Unpowered) | `Spool` | Spool axle prevents one-wheel sand entrapment in dunes |
| **Off-Road** | Trophy Truck / Mega Truck | `Open` or `LSD` | `Spool` | Solid live axle rear with spool locker for heavy terrain and jumps |
| **Rally** | Rallycross / Group B AWD | `LimitedSlip` (45% power, 30% coast) | `LimitedSlip` (55% power, 35% coast) | Mechanical plated LSD front & rear for mixed loose gravel/ice/asphalt |
| **Drift** | Drift Car | `Open` (Unpowered) | `Spool` (Welded Diff) | 100% locked welded rear diff for predictable oversteer initiation |

---

## 🧪 4. Empirical Test Verification Receipts

### 4.1 Unit & Integration Test Suite Results

```bash
cargo test --test differential_dynamics_tests
```
```
running 5 tests
test test_open_differential_equalizes_torque_and_limits_power_on_split_mu ... ok
test test_kart_spool_with_caster_jacking_maintains_drive_and_turning_circle ... ok
test test_spool_differential_locks_wheel_rotational_velocities ... ok
test test_lsd_transfers_torque_to_gripping_wheel_proportional_to_locking_factor ... ok
test test_spool_differential_transfers_100_percent_torque_when_one_wheel_unloaded ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 4.2 Full Workspace Health Confirmation

* **`wheelbase` unit tests**: 47 passed; 0 failed
* **`wheelbase` decoupled tire tests**: 9 passed; 0 failed
* **`wheelbase` differential dynamics tests**: 5 passed; 0 failed
* **Full workspace tests (`cargo test`)**: 100% passed across all crates (`wheelbase`, `arcade_race_core`, `cabinet`, `tdrace-app`, `tdrace_core`)
* **Full fleet turning benchmark (`cargo run --bin turning_benchmark`)**: 85/85 vehicle simulations passed with zero regressions. All karts strictly satisfy $D < 2.60\text{ m}$.

---

## 🌟 5. Conclusion & Operational Impact

The introduction of explicit differential models resolves the fundamental physical discrepancies identified during kart driving evaluations:
1. Karts no longer suffer from artificial inside-wheel slip or loss of drive force when turning.
2. The entire vehicle fleet across all tiers and modalities now exhibits authentic, real-world differential behavior tailored to specific motorsport disciplines.
3. Simulation performance remains entirely deterministic with zero runtime heap allocations, executing in sub-millisecond time.
