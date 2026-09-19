---
type: Physics Reference
title: "Pacejka '96 Tire Model & Lateral Slip Curves"
description: "Mathematical formulation of the Pacejka Magic Formula, slip angle calculation, grip saturation, progressive breakaway, and drift slide friction."
status: active
category: physics
tags: [physics, tires, pacejka, slip-angle, drift]
---

# Pacejka '96 Tire Model & Lateral Slip Curves 🛞📐

Tire forces in **TdRace** are computed using a modified **Pacejka '96 "Magic Formula"** semi-empirical model implemented in [`crates/wheelbase/src/tire.rs`](../../crates/wheelbase/src/tire.rs). Unlike simplified linear friction models, Pacejka captures non-linear grip saturation, progressive breakaway past the limit of adhesion, and realistic transition into dynamic drift slides.

---

## 1. The Magic Formula Formulation

The lateral cornering force $F_y$ developed by a tire is a function of its **slip angle** $\alpha$ (the angle between the tire's pointing direction and its actual travel vector) and normal load $F_z$:

$$F_y(\alpha) = \mu_{\text{surface}} \cdot F_z \cdot D \cdot \sin\left(C \cdot \arctan\left(B \cdot \alpha - E \cdot (B \cdot \alpha - \arctan(B \cdot \alpha))\right)\right)$$

### Parameter Definitions

| Parameter | Field Name | Typical Range | Physical Significance |
| :--- | :--- | :---: | :--- |
| **$B$** | `stiffness_b` | $6.0 \dots 14.0$ | **Stiffness factor**: Dictates the initial cornering stiffness (linear response slope $B \cdot C \cdot D$) at small slip angles. Higher values yield razor-sharp steering turn-in. |
| **$C$** | `shape_c` | $1.2 \dots 1.7$ | **Shape factor**: Controls the transition curve and asymptotic limits. |
| **$D$** | `peak_d` | $0.9 \dots 1.3$ | **Peak friction coefficient**: The maximum normalized traction coefficient achievable before tire breakaway. |
| **$E$** | `curvature_e` | $-0.4 \dots 0.2$ | **Curvature factor**: Governs the sharpness of the peak and the subsequent post-limit grip decay. |

---

## 2. Slip Angle ($\alpha$) Calculation

For each wheel with forward speed $v_x$ and lateral speed $v_y$:

$$\alpha = \arctan\left(\frac{v_y}{|v_x| + \epsilon}\right) - \delta_{\text{steer}}$$

Where $\delta_{\text{steer}}$ is the steered angle (non-zero for front wheels) and $\epsilon = 0.1\text{ m/s}$ avoids division by zero at standstill.

```
       Travel Vector (v)
             ↗
            /
           /  ) Slip Angle (α)
          /  /
         /  /
  [====TIRE====] -> Wheel Heading
```

---

## 3. Grip Curve Progression

```
Lateral Force (Fy)
     ▲
Peak │            ╭─────────╮   <- Peak Grip (D)
Grip │          ╭─╯         ╰─╮
     │        ╭─╯             ╰────────────── Asymptotic Slide Friction
     │       ╭╯                                (drift_slide_friction)
     │      ╭╯
     │     ╭╯
     │    ╭╯
   0 └────┴─────────────────────────────► Slip Angle (α)
         0°        6°~10°          35°
       Linear     Grip Peak     Full Slide
       Region    (Limit Point)
```

1. **Linear Region ($\alpha < 4^\circ$)**: Tire deflection is elastic; cornering force increases proportionally with slip angle ($F_y \approx B \cdot C \cdot D \cdot \alpha$).
2. **Grip Peak ($\alpha \approx 6^\circ \dots 10^\circ$)**: Tire reaches maximum traction limit $D$. Drivers operating in this window achieve optimal lap times.
3. **Breakaway & Slide ($\alpha > 15^\circ$)**: Tread shears across the road surface. Force drops toward `drift_slide_friction` ($0.75 \dots 0.92\times$ of peak), allowing controllable power sliding and counter-steering.

---

## 4. Special Dynamics & Driver Assists

### 4.1 Handbrake Slip Reduction
Engaging the e-brake locks rear wheels and drops lateral friction by `handbrake_lateral_friction_multiplier` ($0.30 \dots 0.45\times$), intentionally breaking rear traction to induce rapid yaw rotation for hairpins and rally turns.

### 4.2 Traction Control System (TCS)
Monitors longitudinal wheel slip ratio $\kappa = \frac{r \cdot \omega - v}{v}$. When $\kappa > \text{tcs\_slip\_threshold}$, engine drive force is cut proportionally by `tcs_strength` to maintain forward traction.

### 4.3 Electronic Stability Control (ESC)
Compares intended driver yaw rate with actual angular velocity $\dot{\psi}$. If yaw divergence exceeds `esc_yaw_threshold`, asymmetric braking impulses restore directional alignment.

---

## 5. Tire Configuration Archetypes

Sample presets from [`crates/wheelbase/src/config.rs`](../../crates/wheelbase/src/config.rs):

```rust
// Drift Car: Progressive breakaway, high slide control
TireConfig {
    stiffness_b: 9.5,
    shape_c: 1.45,
    peak_d: 1.00,
    curvature_e: -0.15,
    drift_slide_friction: 0.92,
    handbrake_lateral_friction_multiplier: 0.30,
}

// Formula / Kart: Ultra-sharp turn-in, unforgiving cliff past peak
TireConfig {
    stiffness_b: 13.5,
    shape_c: 1.65,
    peak_d: 1.25,
    curvature_e: 0.10,
    drift_slide_friction: 0.72,
    handbrake_lateral_friction_multiplier: 0.40,
}
```

For environmental friction scaling, see [Surface Specifications](surfaces.md).
