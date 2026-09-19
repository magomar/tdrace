---
type: Physics Reference
title: "Vehicle Dynamics & Chassis Weight Transfer"
description: "Mathematical modeling of 2D/2.5D kinematics, longitudinal pitch, lateral body roll, center of gravity, downforce, and aerodynamic drag."
status: active
category: physics
tags: [physics, dynamics, weight-transfer, aerodynamics]
---

# Vehicle Dynamics & Chassis Weight Transfer 🏎️⚖️

Vehicle motion in **TdRace** is governed by rigid-body planar dynamics integrated with dynamic 2.5D weight transfer and aerodynamic load scaling. These calculations determine normal load ($F_z$) per tire, which directly scales peak tire grip.

---

## 1. Chassis Geometry & Center of Gravity (CG)

The vehicle chassis is modeled with distinct front and rear axle offsets from the Center of Gravity:

| Symbol | Parameter Name | Unit | Physical Role |
| :--- | :--- | :--- | :--- |
| $m$ | `mass` | $\text{kg}$ | Total vehicle mass including driver and fluids. |
| $I_z$ | `inertia` | $\text{kg}\cdot\text{m}^2$ | Yaw polar moment of inertia; resists rotational acceleration. |
| $L$ | `wheelbase` | $\text{m}$ | Total axle span: $L = a + b$. |
| $a$ | `cg_to_front` | $\text{m}$ | Distance from CG to front steering axle. |
| $b$ | `cg_to_rear` | $\text{m}$ | Distance from CG to rear drive axle. |
| $w$ | `track_width` | $\text{m}$ | Lateral distance between left and right wheels. |
| $h_{\text{cg}}$ | `cg_height` | $\text{m}$ | Height of mass center above ground plane; governs roll/pitch torque. |

### Static Normal Axle Loads
Under static resting conditions with gravity $g = 9.81\text{ m/s}^2$:

$$F_{z,\text{front, static}} = m \cdot g \cdot \frac{b}{a + b}$$

$$F_{z,\text{rear, static}} = m \cdot g \cdot \frac{a}{a + b}$$

---

## 2. Dynamic Weight Transfer

When accelerating, braking, or cornering, inertial forces act at the CG height ($h_{\text{cg}}$), shifting normal load between wheels:

### 2.1 Longitudinal Weight Transfer (Pitch: Squat & Dive)
With longitudinal vehicle acceleration $a_x$:

$$\Delta F_{z,\text{long}} = m \cdot a_x \cdot \frac{h_{\text{cg}}}{L} \cdot k_{\text{weight\_transfer\_longitudinal}}$$

* **Acceleration ($a_x > 0$)**: Rear tires gain normal load ($\Delta F_z > 0$), front tires lose load. Enhances RWD launch grip; induces understeer.
* **Braking ($a_x < 0$)**: Front tires gain normal load (dive), rear tires unload. Increases front turning bite; induces trail-braking oversteer.

### 2.2 Lateral Weight Transfer (Roll: Body Lean)
With lateral cornering acceleration $a_y$:

$$\Delta F_{z,\text{lat}} = m \cdot a_y \cdot \frac{h_{\text{cg}}}{w} \cdot k_{\text{weight\_transfer\_lateral}}$$

Outside wheels in a turn gain vertical load while inside wheels unload. Due to tire load sensitivity (diminishing return of friction with increased vertical load), excessive body roll reduces total axle cornering capacity.

---

## 3. Aerodynamics: Downforce & Drag

Aerodynamic forces scale quadratically with longitudinal speed $v$:

### 3.1 Aerodynamic Downforce
Downward aerodynamic load adds vertical tire pressure without adding inertial mass:

$$F_{\text{downforce}} = C_{\text{downforce}} \cdot v^2$$

This additional load is partitioned between front and rear axles according to vehicle aero-balance, enabling high-speed cornering stability for downforce-heavy classes (e.g. FIA GT3, Hypercars, Superkarts).

### 3.2 Aerodynamic Drag
Opposing drag retards longitudinal acceleration:

$$F_{\text{drag}} = C_{\text{air\_drag}} \cdot v^2 + C_{\text{rolling\_resistance}} \cdot m \cdot g$$

Terminal top speed $v_{\text{top}}$ is reached when engine tractive thrust equals total aerodynamic and rolling resistance:

$$F_{\text{engine, max}} = C_{\text{air\_drag}} \cdot v_{\text{top}}^2 + C_{\text{rolling\_resistance}} \cdot m \cdot g$$

---

## 4. Summary of Configuration Parameters

These levers are defined per vehicle class in [`crates/wheelbase/src/config.rs`](../../crates/wheelbase/src/config.rs):

```rust
pub struct CarConfig {
    pub mass: f32,                          // e.g. 1265.0 kg (GT3)
    pub inertia: f32,                       // e.g. 1850.0 kg·m²
    pub wheelbase: f32,                     // e.g. 2.65 m
    pub track_width: f32,                   // e.g. 1.68 m
    pub cg_to_front: f32,                   // e.g. 1.25 m
    pub cg_to_rear: f32,                    // e.g. 1.40 m
    pub cg_height: f32,                     // e.g. 0.32 m
    pub weight_transfer_longitudinal: f32, // 1.0 = full pitch physics
    pub weight_transfer_lateral: f32,      // 1.0 = full body roll
    pub downforce_coefficient: f32,        // e.g. 2.20
    pub air_drag_coefficient: f32,         // e.g. 0.42
}
```

For tire grip details and how $F_z$ determines slip force, see [Pacejka Tire Model](tire_pacejka.md).
