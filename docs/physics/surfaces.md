---
type: Physics Reference
title: "Surface Physics & Environmental Friction Matrix"
description: "Comprehensive 12-surface simulation matrix: friction coefficients (mu), rolling resistance, viscous drag, particle roost, and split-mu per-wheel sampling."
status: active
category: physics
tags: [physics, surfaces, friction, terrain, wheelbase]
---

# Surface Physics & Environmental Friction Matrix 🏁🌍

In **TdRace**, track and off-track terrain interactions are computed on an independent per-wheel basis via the split-$\mu$ surface sampler in [`crates/wheelbase/src/surface.rs`](../../crates/wheelbase/src/surface.rs). Each wheel samples the local terrain beneath its tire contact patch, allowing vehicles to experience half-on / half-off split-friction when clipping kerbs, sliding onto grass runoffs, or splashing through standing puddles.

---

## 1. 12-Surface Comparative Parameter Matrix

| Surface Type | Friction ($\mu$) | Rolling Resistance Multiplier | Surface Drag Multiplier | Tire Smoke | Debris Roost | Water Splash | Layer Default | Primary Role & Track Character |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| **`Asphalt`** | **`1.00`** | `1.0×` | `1.00×` | Yes | No | No | `BelowTrack` | Standard dry tarmac; optimal grip baseline, full tire smoke on heavy slip. |
| **`Concrete`** | **`0.95`** | `1.05×` | `1.00×` | Yes | No | No | `BelowTrack` | Poured solid pavement; high grip with low rolling drag for grandstands, pit aprons, and stadium bowls. |
| **`Curb`** | **`0.88`** | `1.3×` | `1.05×` | Yes | No | No | `BelowTrack` | Apex kerb / rumble strip; subtle haptic rumble, high grip with mild drag. |
| **`Dirt`** | **`0.78`** | `1.2×` | `1.10×` | No | Yes | No | `BelowTrack` | Compacted clay / gravel rally track; predictable sliding and drift control. |
| **`Gravel`** | **`0.70`** | `2.5×` | `1.25×` | No | Yes | No | `BelowTrack` | Loose stone rally stage / runoff; moderate grip with heavy stone roost. |
| **`Mud`** | **`0.52`** | `6.5×` | `3.20×` | No | Yes | No | `AboveTrack` | Viscous mud bog; heavy deceleration drag, low lateral bite, brown roost. |
| **`Grass`** | **`0.45`** | `18.0×` | `2.20×` | No | Yes | No | `BelowTrack` | Standard off-track runoff; heavy rolling resistance penalizing corner cuts. |
| **`Snow`** | **`0.34`** | `3.0×` | `1.60×` | No | Yes | No | `AboveTrack` | Packed/powder snow; slippery winter rallying, white roost plumes. |
| **`Sand`** | **`0.30`** | `30.0×` | `4.50×` | No | Yes | No | `BelowTrack` | Deep gravel / sand trap; severe vehicle deceleration trap, sand rooster tails. |
| **`Water`** | **`0.22`** | `3.5×` | `2.00×` | No | No | Yes | `AboveTrack` | Standing puddle hazard; hydroplaning risk, aqua spray plumes. |
| **`Oil`** | **`0.12`** | `0.8×` | `0.95×` | No | No | No | `AboveTrack` | Oil slick hazard; extreme spin hazard, breaks rear traction instantly. |
| **`Ice`** | **`0.08`** | `0.4×` | `0.90×` | No | No | No | `AboveTrack` | Frozen lake; near-zero traction, near-frictionless gliding with no braking. |

---

## 2. Dynamic Physical Effects

### 2.1 Split-$\mu$ Per-Wheel Sampling
Rather than applying a single surface modifier to the car center of mass, each wheel calculates independent contact patch friction:

$$F_{y, i} = \mu_{\text{surface}, i} \cdot F_{z, i} \cdot \text{Pacejka}(\alpha_i)$$

$$F_{x, i} = \mu_{\text{surface}, i} \cdot F_{z, i} \cdot \text{Longitudinal}(\kappa_i)$$

When right-side tires touch `Grass` ($\mu = 0.45$) while left-side tires remain on `Asphalt` ($\mu = 1.00$), an asymmetric yaw moment $\tau_{\text{split}} = (F_{x, \text{left}} - F_{x, \text{right}}) \cdot \frac{w}{2}$ immediately pulls the car toward the higher-grip side, requiring active driver counter-steering.

### 2.2 Viscous Surface Drag & Resistance
Soft or deep surfaces (`Sand`, `Mud`, `Grass`, `Water`) exert retarding resistive forces directly proportional to vehicle velocity $v$:

$$F_{\text{retard}} = k_{\text{rolling}} \cdot m \cdot g \cdot C_{\text{rolling\_mult}} + k_{\text{viscous}} \cdot C_{\text{drag\_mult}} \cdot v^2$$

This creates genuine sand traps that safely arrest high-speed off-track excursions without requiring unnatural artificial speed clamps.

---

## 3. Visual FX & Audio Cues

* **Skidmarks**: Drawn to a persistent canvas texture whenever tire slip exceeds `skid_threshold` ($0.10$). Renders black rubber marks on `Asphalt`/`Curb`, darker brown grooves on `Dirt`, and compressed trails on `Snow`.
* **Particle Plumes**:
  * `Asphalt` / `Curb`: Dense white smoke particles.
  * `Dirt` / `Gravel`: High-velocity brown and grey roost particles hurled in wheel rotation direction.
  * `Water`: Transparent aqua spray plumes accompanied by hydroplaning audio wash.
  * `Mud`: Heavy thick brown clumps with slow dissipation.

For details on wall collisions, see [Walls & Barriers](walls_barriers.md).
