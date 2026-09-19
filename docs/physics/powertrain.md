---
type: Physics Reference
title: "Powertrain, Drivetrain & Future Sub-Models"
description: "Engine tractive forces, drive layout biases (FWD/AWD/RWD), brake distribution, and roadmap for multi-gear transmissions and thermal tire degradation."
status: active
category: physics
tags: [physics, engine, powertrain, drivetrain, future-models]
---

# Powertrain, Drivetrain & Future Sub-Models ⚙️🏎️

Vehicle acceleration, braking distribution, and drivetrain configurations in **TdRace** are controlled through the powertrain sub-models in [`crates/wheelbase/src/car.rs`](../../crates/wheelbase/src/car.rs).

---

## 1. Drivetrain Layouts & Torque Distribution

The `drive_bias` parameter specifies how longitudinal engine force $F_{\text{engine}}$ is split between the front and rear axles:

$$F_{\text{drive, front}} = \text{drive\_bias} \cdot F_{\text{engine}} \cdot \text{throttle}$$

$$F_{\text{drive, rear}} = (1.0 - \text{drive\_bias}) \cdot F_{\text{engine}} \cdot \text{throttle}$$

| Drivetrain Layout | `drive_bias` | Handling Character | Applied Vehicle Categories |
| :--- | :---: | :--- | :--- |
| **Rear-Wheel Drive (RWD)** | `0.0` | High launch traction; prone to power oversteer on corner exits; responsive to throttle steering. | GT3/GT2/GT4, NASCAR Cup, Trans-Am TA1, Drift Cars, Formula / Superkart. |
| **All-Wheel Drive (AWD)** | `0.5` | Maximum launch traction on loose surfaces; high mid-corner stability; neutral drift control. | WRC Supercars, Rallycross Supercars, Raid T1+, Mud Boggers, Sand Rails. |
| **Front-Wheel Drive (FWD)** | `1.0` | Natural terminal understeer; forgiving for novice drivers; high directional pull out of hairpins. | Rally Jr FWD, Grassroots Hot Hatches, Cadet racers. |

---

## 2. Braking System & Brake Bias

Total braking force $F_{\text{brake}}$ is apportioned to avoid hazardous rear axle lockups:

$$F_{\text{brake, front}} = \text{brake\_bias} \cdot F_{\text{brake, max}} \cdot \text{brake}$$

$$F_{\text{brake, rear}} = (1.0 - \text{brake\_bias}) \cdot F_{\text{brake, max}} \cdot \text{brake}$$

Standard competition vehicles set `brake_bias` between $0.58 \dots 0.65$ (front-biased) to counterbalance forward load transfer under heavy deceleration.

---

## 3. Engine Braking & Speed Governors

When throttle is released, an opposing engine retarding force simulates powertrain parasitic drag and manifold vacuum:

$$F_{\text{engine\_braking}} = \text{engine\_braking\_coefficient} \cdot m \cdot g$$

Top speed is governed by equilibrium between maximum engine tractive force and aerodynamic/viscous drag:

$$v_{\text{terminal}} = \min\left(\text{top\_speed\_mps}, \sqrt{\frac{F_{\text{engine}} - C_{\text{rolling}} m g}{C_{\text{drag}}}}\right)$$

---

## 4. Roadmap: Modular Engine & Thermal Tire Sub-Models

Future releases of `crates/wheelbase` will introduce modular plug-in simulation components:

### 4.1 Non-Linear Torque-RPM Power Curves
Replacement of constant thrust envelopes with authentic engine dyno curves:

$$\tau_{\text{engine}}(\text{RPM}) = \tau_{\text{peak}} \cdot f(\text{RPM}) \cdot \eta_{\text{gear}}$$

Including discrete gear ratios ($1 \dots 7$), clutch slip, and redline rev-limiters.

### 4.2 Multi-Compound Thermal Tire Degradation
Dynamic tire carcass and tread temperature modeling:

$$\frac{dT_{\text{tire}}}{dt} = \dot{Q}_{\text{friction}} + \dot{Q}_{\text{flex}} - \dot{Q}_{\text{cooling}}$$

Peak grip coefficient $D(T)$ optimized within a specific thermal window ($80^\circ\text{C} \dots 105^\circ\text{C}$) with progressive blistering/graining when overheated.

To inspect car categories utilizing these drivetrains, explore the [Vehicle Roster](../vehicles/index.md).
