---
type: Reference Guide
title: "Vehicle Specifications & Handling Dynamics"
description: "Comprehensive engineering guide to vehicle specifications, the 6-stat telemetry matrix, category vs model physical differentiation, and dynamic engine parameter derivation."
status: active
category: vehicles
tags: [vehicles, specs, telemetry, physics, bop, dynamics, handling]
---

# Vehicle Specifications & Handling Dynamics 🏎️📊

In **TdRace**, vehicle physics operate on a sophisticated two-tier hierarchy: **Macro-Category Archetypes** that define fundamental motorsport disciplines, and **Micro-Model Dynamic Derivations** that impart authentic, real-world handling traits, weight transfers, and Balance of Performance (BoP) characteristics to every individual car.

This guide details the **6-dimensional telemetry standard**, the engineering architecture behind category and model differentiation, and the mathematical formulas implemented in the simulation engine.

---

## 🧭 The Two-Tier Physics Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                      1. CATEGORY MACRO-TIER                            │
│  Baseline physics archetype: Mass envelope, suspension roll/pitch,     │
│  wheelbase geometry, default tires, and module force multipliers.      │
│  (e.g., 125cc Shifter Kart vs GT3 EVO vs Dakar Rally Raid T1+)         │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ derives
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                      2. MODEL MICRO-TIER (`to_car_config`)             │
│  Dynamic individual parameters: Curb weight, engine BHP, drivetrain,   │
│  caliper braking force, Pacejka lateral D, rack turn-in rate,         │
│  yaw moment of inertia, and aerodynamic downforce / drag coefficients. │
│  (e.g., Porsche 718 Cayman GT4 RS vs BMW M4 GT4 vs Alpine A110 GT4)     │
└────────────────────────────────────────────────────────────────────────┘
```

1. **Category Macro-Tier (`CarCategory` & `CarChoice`)**: Establishes structural physics parameters such as baseline sprung mass, center-of-gravity height ($h_{\text{cg}}$), pitch/roll spring rates ($k_{\text{pitch}}$, $k_{\text{roll}}$), wheelbase ($L$), track width ($w$), and base tire slip curves.
2. **Model Micro-Tier (`RealCarModel::to_car_config`)**: Dynamically overrides and scales the category archetype with the vehicle's exact homologated specifications (mass, horsepower, drivetrain bias, brake package, aerodynamic downforce, and normalized telemetry ratings).

---

## 📊 Unified 6-Dimensional Telemetry Standard

Every vehicle across the 25 progression tiers is rated along **6 unified telemetry metrics**, shared between the **In-Game Garage HUD** (`crates/tdrace-app/src/ui/garage.rs`) and the **Web Showroom** (`portals/option-b-showroom/src/components/CarCard.astro`):

| # | Metric | Short Label | Signature Color | Visual Icon | Engine Variable | Physical Role in Simulation |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **1** | **Top Speed** | `Speed` | Neon Cyan (`#22d3ee`) | 🏎️ Speedometer Dial | `cfg.top_speed_mps` | Terminal straight-line velocity ($v_{\text{top}} = v_{\text{km/h}} / 3.6$). |
| **2** | **Acceleration** | `Accel` | Neon Gold (`#facc15`) | ⚡ Launch Sprint Bolt | `cfg.max_engine_force` | Tractive drive force: $F_{\text{engine}} = \text{BHP} \cdot k_{\text{force}}$. |
| **3** | **Cornering Grip** | `Grip` | Neon Green (`#22c55e`) | 🛞 Apex Curve Vector | `cfg.tire.peak_d` | Scales Pacejka Peak factor ($D$) on lateral tire slip curve ($F_y = D \cdot \sin(\dots)$). |
| **4** | **Drift Agility** | `Agility` | Neon Magenta (`#e879f9`) | 🔀 Chicane Slalom Flick | `cfg.steer_speed` & `cfg.inertia` | Modulates steering rack angular velocity ($\omega_{\text{steer}}$) and inverse yaw inertia ($I_z$). |
| **5** | **Braking Force** | `Braking` | Neon Orange (`#fb923c`) | 🛑 Stop Sign Disc | `cfg.max_brake_force` | Total caliper deceleration force ($F_{\text{brake}}$) scaled by mass and hardware rating. |
| **6** | **Downforce** | `Downforce` | Pure White (`#ffffff`) | 🪽 GT Downforce Wing | `cfg.downforce_coefficient` | Dynamic vertical load: $F_{\text{downforce}} = C_{\text{downforce}} \cdot v^2$. |

---

## 🏎️ Category-Level Macro Differentiation

The 5 motorsport modules span extreme engineering regimes, producing fundamentally distinct handling profiles before model-level tuning is applied:

| Motorsport Module | Curb Weight Range | Power Range | Drive Layouts | Baseline Aero ($C_l$) | Defining Dynamic Trait |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Karting & Micro-Racers** | 110 – 185 kg | 9 – 95 BHP | Direct RWD | $0.00 - 0.50$ | Ultra-low CG, direct 1:1 steering ratio, zero suspension roll, high force-per-BHP ratio ($38.0\text{ N/BHP}$). |
| **Gran Turismo & Endurance** | 972 – 1,430 kg | 360 – 1,000 BHP | RWD / Hybrid AWD | $0.55 - 2.80$ | High aerodynamic loading, carbon-ceramic brakes (25–34 kN), balanced front-to-rear chassis weight transfer. |
| **NASCAR Stock Car Racing** | 1,450 – 1,540 kg | 400 – 850 BHP | Pure RWD | $0.20 - 0.45$ | Heavy steel tubular chassis, high-power naturally aspirated pushrod V8s, high yaw inertia, low agility. |
| **Rallycross & All-Terrain** | 1,150 – 2,000 kg | 140 – 600 BHP | AWD (50/50) / FWD | $0.15 - 0.45$ | Long suspension travel, violent turbo spool, loose-surface drift yaw balance, jump landing dampening. |
| **Extreme Off-Road & Arenas** | 1,200 – 5,400 kg | 185 – 1,500 BHP | 4WD / Locked AWD | $0.00 - 0.15$ | Giant tire diameters, locking differentials, high CG squat/dive, massive unsprung mass. |

---

## ⚖️ Model-Level Micro Differentiation & Balance of Performance (BoP)

While cars within the same class share lap-time parity ($\Delta t \le 0.15\text{ s}$ around benchmark tracks), their physical inputs produce distinctly different driving feels.

### Case Study 1: GT4 Clubsport (Tier 1 GT)
* **Porsche 718 Cayman GT4 RS Clubsport** ($1,415\text{ kg}$, $500\text{ BHP}$):
  * *Setup*: Mid-engine flat-6, high downforce ($C_l\ 0.85$, $C_d\ 0.42$), $25.5\text{ kN}$ steel Brembo brakes.
  * *Handling*: Neutral cornering balance, high apex corner speed, moderate straight-line top speed.
* **BMW M4 GT4** ($1,430\text{ kg}$, $550\text{ BHP}$):
  * *Setup*: Front-engine twin-turbo I6, lower downforce ($C_l\ 0.70$, $C_d\ 0.38$), $26.5\text{ kN}$ AP Racing brakes.
  * *Handling*: Phenomenal straight-line punch, higher top speed, requires deliberate trail-braking to rotate due to higher front-axle yaw inertia.
* **Alpine A110 GT4** ($1,080\text{ kg}$, $360\text{ BHP}$):
  * *Setup*: Ultra-lightweight rear-mid turbo-4, low downforce ($C_l\ 0.55$, $C_d\ 0.35$), $20.0\text{ kN}$ lightweight Brembo brakes.
  * *Handling*: Extreme directional flickability, razor-sharp turn-in agility, lower straight-line acceleration compensated by blistering mid-corner speed.

### Case Study 2: GT2 Supercars (Tier 3 GT)
* **Porsche 911 GT2 RS Clubsport** ($1,390\text{ kg}$, $700\text{ BHP}$):
  * *Setup*: Rear-engine twin-turbo flat-6, massive mechanical rear grip, $29.0\text{ kN}$ PFC brakes, $C_l\ 1.10$.
  * *Handling*: Monstrous exit traction out of slow turns; requires patience on entry to manage rear pendulum weight transfer.
* **KTM X-Bow GT2** ($1,048\text{ kg}$, $600\text{ BHP}$):
  * *Setup*: Carbon monocoque, mid-engine Audi 2.5L turbo, $24.5\text{ kN}$ monobloc brakes, $C_l\ 1.05$.
  * *Handling*: Rapid transitional response in chicanes, high lateral tire load capacity, forgiving slip angles.
* **Brabham BT62** ($972\text{ kg}$, $700\text{ BHP}$):
  * *Setup*: Lightweight track weapon, naturally aspirated 5.4L V8, extreme aero ($C_l\ 1.50$, $C_d\ 0.48$), $31.0\text{ kN}$ carbon-on-carbon brakes.
  * *Handling*: Immense high-speed braking and downforce grip, demands aggressive entry speeds to generate aerodynamic tire load.

### Case Study 3: LMH & LMDh Hypercars (Tier 5 GT)
* **Ferrari 499P LMH** ($1,030\text{ kg}$, $680\text{ BHP}$ ICE + Hybrid AWD):
  * *Setup*: 3.0L twin-turbo V6, front axle electric motor, $C_l\ 2.40$, $C_d\ 0.48$, $34.0\text{ kN}$ Brembo carbon-carbon.
  * *Handling*: Maximum aerodynamic grip through high-speed sweeps, all-wheel exit drive out of hairpins.
* **Aston Martin Valkyrie AMR Pro** ($1,000\text{ kg}$, $1,000\text{ BHP}$ pure RWD):
  * *Setup*: 6.5L naturally aspirated Cosworth V12 screaming to 11,100 RPM, $C_l\ 2.80$, $C_d\ 0.42$, $385\text{ km/h}$ top speed.
  * *Handling*: Immense raw horsepower and downforce suction; lightning-fast steering rack response and brutal high-speed stability.

---

## ⚙️ Simulation Engine Derivation (`to_car_config`)

In [`crates/tdrace-app/src/catalog/mod.rs`](../../crates/tdrace-app/src/catalog/mod.rs), the `RealCarModel::to_car_config(&self)` function translates catalog specifications into the active physics engine parameters:

### 1. Curb Mass & Straight-Line Velocity
$$\text{cfg.mass} = m_{\text{car}}$$
$$\text{cfg.top\_speed\_mps} = \frac{v_{\text{top\_kmh}}}{3.6}$$

### 2. Engine Tractive Force
$$\text{cfg.max\_engine\_force} = \text{BHP} \cdot k_{\text{force\_per\_bhp}}$$

Where $k_{\text{force\_per\_bhp}}$ reflects powertrain efficiency and vehicle scale:
* **Karting**: $38.0\text{ N/BHP}$ (ultra-light direct gear ratio drive)
* **Extreme Off-Road (Tier 4–5)**: $14.0\text{ N/BHP}$ (massive crawler reduction gears and tire inertia)
* **All Other Categories**: $17.5\text{ N/BHP}$ (standard competition racing transmissions)

### 3. Drivetrain Torque Bias
$$\text{cfg.drive\_bias} = \begin{cases} 1.0 & \text{for FWD (Front-Wheel Drive)} \\ 0.5 & \text{for AWD / 4WD (All-Wheel Drive)} \\ 0.0 & \text{for RWD (Rear-Wheel Drive)} \end{cases}$$

### 4. Braking Deceleration Force
$$F_{\text{brake}} = F_{\text{base\_brake}} \cdot \left(\frac{m_{\text{car}}}{m_{\text{base}}}\right) \cdot \text{clamp}(0.80 + 0.40 \cdot \text{stats.braking},\, 0.5,\, 1.5)$$

* **Mass Scaling**: Heavier cars naturally require proportionally higher brake torque to achieve standard deceleration rates.
* **Hardware Multiplier**: Calibrated from $0.80$ to $1.20$ based on brake rotor diameter, caliper piston count, and friction compound (steel vs carbon-ceramic).

### 5. Pacejka Lateral Tire Adhesion
$$D_{\text{lateral}} = \text{clamp}(D_{\text{base}} \cdot (0.85 + 0.30 \cdot \text{stats.grip}),\, 0.5,\, 1.8)$$

* Scales the Peak Factor ($D$) in the Pacejka '96 Magic Formula:
$$F_y(\alpha) = \mu \cdot F_z \cdot D \cdot \sin\left(C \cdot \arctan\left(B\alpha - E(B\alpha - \arctan(B\alpha))\right)\right)$$
* Ensures high-grip models pull higher lateral $g$-forces before breaking into a slide.

### 6. Steering Rack Agility & Yaw Inertia
$$\omega_{\text{steer}} = \text{clamp}(\omega_{\text{base\_steer}} \cdot (0.80 + 0.40 \cdot \text{stats.agility}),\, 2.0,\, 15.0)$$
$$I_z = \max\left(I_{\text{base}} \cdot \left(\frac{m_{\text{car}}}{m_{\text{base}}}\right) \cdot (1.15 - 0.30 \cdot \text{stats.agility}),\, 10.0\right)$$

* **Steering Rack Speed ($\omega_{\text{steer}}$)**: Governs how quickly the front wheels rotate in response to driver steering input (rad/s).
* **Yaw Moment of Inertia ($I_z$)**: Determines rotational resistance to changing heading angle. High-agility, centralized-mass cars feature lower $I_z$, allowing rapid chicane transitions.

### 7. Aerodynamic Downforce & Drag
Homologated aero strings (e.g., `"Cl 0.85 / Cd 0.42"`) are parsed dynamically:
$$\text{cfg.downforce\_coefficient} = C_l \cdot 0.76$$
$$\text{cfg.air\_drag\_coefficient} = C_d$$

* **Dynamic Downforce Loading**:
$$F_{\text{downforce}} = \text{cfg.downforce\_coefficient} \cdot v^2$$
* Vertically presses tires into the track at high velocities, amplifying normal load $F_z$ and lateral cornering grip without increasing inertial mass $m$.

---

## 🎮 Interface & Telemetry Parity

The telemetry pipeline guarantees mathematical and visual alignment across all surfaces:

```
┌─────────────────────────────────┐       ┌─────────────────────────────────┐
│       IN-GAME GARAGE HUD        │       │       WEB SHOWROOM PORTAL       │
│   crates/tdrace-app/src/ui/     │       │     portals/option-b-showroom   │
│   garage.rs:draw_garage_hud()   │       │     src/components/CarCard.astro│
├─────────────────────────────────┤       ├─────────────────────────────────┤
│ Speed     [████████░░░░]  78%   │       │ [🏎️] Speed     [████████░░░░] 78%│
│ Accel     [█████████░░░]  82%   │       │ [⚡] Accel     [█████████░░░] 82%│
│ Grip      [██████████░░]  85%   │  ===  │ [🛞] Grip      [██████████░░] 85%│
│ Agility   [███████░░░░░]  70%   │       │ [🔀] Agility   [███████░░░░░] 70%│
│ Braking   [████████░░░░] 25.5k  │       │ [🛑] Braking   [████████░░░░] 25k│
│ Downforce [██████░░░░░░]  0.85  │       │ [🪽] Downforce [██████░░░░░░]0.85│
└─────────────────────────────────┘       └─────────────────────────────────┘
```

Both interfaces display:
1. **Identical Metric Ordering**: Speed $\rightarrow$ Accel $\rightarrow$ Grip $\rightarrow$ Agility $\rightarrow$ Braking $\rightarrow$ Downforce.
2. **Unified Color Spectrum**: Cyan, Gold, Green, Magenta, Orange/Red, and Pure White.
3. **Hover Tooltips**: Displaying engineering ground truths (0–100 times, brake caliper specs, aerodynamic lift/drag coefficients, and exact engine horsepower).

---

## 🔗 Related Documentation & References

* **Motorsport Modules**:
  * [Gran Turismo & Endurance GT Roster](gt_endurance.md)
  * [NASCAR Stock Car & Trans-Am Roster](stock_car.md)
  * [Rallycross & All-Terrain Roster](rally_allterrain.md)
  * [Extreme Off-Road & Stunt Arenas Roster](offroad_stunt.md)
  * [Karting & Micro-Racers Roster](karting.md)
* **Simulation Physics Engine**:
  * [Pacejka '96 Tire Slip Model](../physics/tire_pacejka.md)
  * [Vehicle Dynamics & Chassis Weight Transfer](../physics/vehicle_dynamics.md)
  * [Powertrain & Drivetrain Engineering](../physics/powertrain.md)
  * [Surface Friction & Split-$\mu$ Sampling](../physics/surfaces.md)
