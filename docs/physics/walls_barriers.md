---
type: Physics Reference
title: "Walls & Barrier Collision Mechanics"
description: "Collision response, restitution coefficients, tangential friction, spark emission, and SAT OBB detection for 4 barrier types."
status: active
category: physics
tags: [physics, collision, walls, barriers, sat]
---

# Walls & Barrier Collision Mechanics 🛡️💥

Collisions between vehicles and perimeter track barriers in **TdRace** are resolved using Separating Axis Theorem (SAT) Oriented Bounding Box (OBB) tests implemented in [`crates/arcade-race-core/src/collision/wall.rs`](../../crates/arcade-race-core/src/collision/wall.rs). Collision responses use physical impulse resolution parameterized across 4 distinct barrier materials.

---

## 1. 4 Barrier Material Profiles

| Barrier Type | Restitution ($e$) | Tangential Friction | Kinetic Energy Dissipation | Spark Emission | Use Case & Circuit Placement |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **`Concrete`** | `0.25` | `0.35` | Moderate | High | Permanent circuit perimeter walls, street circuits, and tunnel portals. |
| **`Steel` (Armco)** | `0.40` | `0.20` | Low (Bouncy) | Intense | Highway guardrails, oval outer rings, and classic road circuits. Low friction allows wall-riding glancing slides. |
| **`TireWall`** | `0.12` | `0.65` | Very High (Dead stop) | None | High-speed braking runoff zones and hairpin apex traps. Heavily absorbs kinetic energy and retards vehicle speed. |
| **`CurbWall`** | `0.30` | `0.40` | Moderate | Low | Concrete-backed apex borders and urban chicane barriers. |

---

## 2. Physical Impulse Resolution

Upon contact detection at collision point $\mathbf{p}$ with wall contact normal $\mathbf{n}$:

### 2.1 Normal Collision Impulse ($J_n$)
The normal velocity component $v_n = \mathbf{v}_{\text{rel}} \cdot \mathbf{n}$ is reversed according to the material coefficient of restitution $e$:

$$J_n = -\frac{(1 + e) \cdot (\mathbf{v}_{\text{rel}} \cdot \mathbf{n})}{\frac{1}{m} + \frac{(\mathbf{r} \times \mathbf{n})^2}{I_z}}$$

Where:
* $\mathbf{r} = \mathbf{p} - \mathbf{p}_{\text{cg}}$ is the vector from the vehicle mass center to the impact point.
* $I_z$ is the vehicle yaw moment of inertia.
* $\mathbf{v}_{\text{rel}}$ is the velocity of the vehicle body at point $\mathbf{p}$.

### 2.2 Tangential Friction Impulse ($J_t$)
Coulomb friction along the tangent vector $\mathbf{t} \perp \mathbf{n}$ resists scraping motion:

$$|J_t| \le \mu_{\text{barrier}} \cdot |J_n|$$

### 2.3 Post-Collision Angular Velocity Update
Impacts offset from the vehicle centerline generate rotational torque:

$$\Delta \omega = \frac{\mathbf{r} \times (J_n \mathbf{n} + J_t \mathbf{t})}{I_z}$$

This produces authentic spin-outs when glancing barriers at high angles of attack.

---

## 3. Spark & Debris Particle Generation

Glancing impacts with `Concrete` and `Steel` barriers generate directional sparks:
* Emitted along the tangential velocity vector $\mathbf{t}$.
* Emission density scales with collision normal momentum $|J_n| \cdot v_{\text{scrape}}$.
* Accompanied by metal scraping audio frequencies and momentary camera trauma shake.

For details on vehicle mass and inertia parameters, see [Vehicle Dynamics](vehicle_dynamics.md).
