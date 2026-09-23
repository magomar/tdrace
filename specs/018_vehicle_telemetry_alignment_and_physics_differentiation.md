---
type: Feature Spec
template: feature
title: "Vehicle Telemetry Alignment and Physics Differentiation"
description: "Unify 6-stat performance telemetry across in-game garage and web showroom with custom vector icons, while dynamically hooking up individual vehicle parameters (braking, grip, agility, and aerodynamics) in the simulation engine."
status: implemented
created: 2026-09-22
generated: { by: agent/antigravity, at: 2026-09-22T17:45:00Z }
---

# Feature Spec: Vehicle Telemetry Alignment and Physics Differentiation 🏎️📊

A comprehensive unification and physics differentiation initiative that harmonizes the **6-dimensional performance telemetry system** (`Speed`, `Acceleration`, `Lateral Grip`, `Drift Agility`, `Braking Force`, `Aerodynamics`) across both the **in-game HUD / garage (`crates/tdrace-app/src/ui/garage.rs`)** and the **Web Showroom portal (`portals/option-b-showroom`)**. In parallel, it transitions the game engine from relying solely on category archetype defaults to dynamically deriving individual vehicle physical characteristics (`max_brake_force`, `tire.peak_d`, `steer_speed`, `inertia`, and `downforce_coefficient`) directly from each car's homologated real-world specifications.

---

## 🗺️ User Flow & Interface Design

### 1. Unified 6-Dimensional Telemetry Standard

Every vehicle in TdRace is characterized across 6 distinct telemetry vectors, aligned in terminology, iconography, and color hierarchy:

| # | Telemetry Metric | Unified Label | Signature Color | Icon Concept | Physics Manifestation |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **1** | **Top Speed** | `Speed` | Neon Cyan (`#22d3ee` / `#3b82f6`) | 🏎️ **Analog Speedometer**: Round dial with perimeter ticks, center pivot, and redline needle. | Governs terminal straight-line velocity ($v_{\text{top}}$) in m/s. |
| **2** | **Acceleration** | `Accel` | Neon Gold (`#facc15` / `#06b6d4`) | ⚡ **Launch Sprint Bolt**: Angular dynamic lightning sprint icon. | Governs engine tractive force ($F_{\text{engine}} = \text{BHP} \cdot k$) and power-to-weight ratio. |
| **3** | **Lateral Grip** | `Grip` | Neon Green (`#22c55e` / `#10b981`) | 🛞 **Apex Curve Vector**: Cornering arc with apex target dot and lateral adhesion vector. | Scales Pacejka Peak factor multiplier ($D$) on lateral tire slip curve ($F_y = D \cdot \sin(\dots)$). |
| **4** | **Drift Agility** | `Agility` | Neon Magenta (`#e879f9` / `#a855f7`) | 🔀 **Chicane Slalom Flick**: S-curve chicane demonstrating rapid directional weight transfer. | Scales steering rack rate (`steer_speed`) and inverse yaw inertia ($I_z$). |
| **5** | **Braking Force** | `Braking` | Neon Orange / Red (`#fb923c` / `#ef4444`) | 🛑 **Stop Sign**: Solid red disc with clean horizontal white bar. | Scales total caliper deceleration force (`max_brake_force`) based on brake package. |
| **6** | **Aerodynamics** | `Aero` | Pure White / Amber (`#ffffff` / `#f59e0b`) | 🪽 **GT Downforce Wing**: High-efficiency rear wing foil with downward vertical pressure vectors. | Injects dynamic aerodynamic vertical loading ($F_{\text{downforce}} = 0.5 \cdot C_l \cdot A \cdot \rho \cdot v^2$). |

---

### 2. Web Showroom Single-Row Inline Card Layout

In [`portals/option-b-showroom/src/components/CarCard.astro`](../portals/option-b-showroom/src/components/CarCard.astro), the previous 2-line layout (label stacked above bar) is refactored into a high-density, horizontal single-row telemetry HUD. This incorporates all 6 performance metrics within the identical vertical envelope (~125px):

```text
┌────────────────────────────────────────────────────────────────────────┐
│  GT4  Tier 1 • RWD                        PORSCHE 718 CAYMAN GT4 RS    │
├────────────────────────────────────────────────────────────────────────┤
│  [Speedometer] Speed     ██████████████████████████░░░░░░░░░░     78%  │
│  [Launch Bolt] Accel     ████████████████████████████░░░░░░░░     82%  │
│  [Apex Vector] Grip      █████████████████████████████░░░░░░░     85%  │
│  [Slalom S-Curv] Agility  ██████████████████████░░░░░░░░░░░░░░     70%  │
│  [Stop Sign]   Braking   ████████████████████████████░░░░░░░░   25.5k  │
│  [Aero Wing]   Downforce █████████████████████░░░░░░░░░░░░░░░    0.85  │
├────────────────────────────────────────────────────────────────────────┤
│  "Mid-engine benchmark of customer racing with screaming flat-6."      │
└────────────────────────────────────────────────────────────────────────┘
```

#### Micro-Interactions & Tooltips
- **Left Column**: Fixed width `w-24`, containing the SVG icon (`w-4 h-4`) and abbreviated label (`text-slate-400 font-mono text-[11px]`).
- **Interactive Tooltip**: Hovering over the icon/label or stat row displays a floating badge revealing exact engineering units (e.g. `🛑 Braking: Steel 380mm 6-Piston (25.5 kN)` or `🪽 Downforce: Cl 0.85 / Cd 0.42`).
- **Center Column**: Full flex width progress bar (`h-1.5 bg-slate-800 rounded-full`) with vibrant colored fill.
- **Right Column**: Fixed width `w-10 text-right font-mono text-[11px] font-bold text-slate-200`.

---

## ⚙️ Backend Models & API Endpoints

### 1. Data Models & Schemas (`okf.ts` & `vehicles.json`)
The vehicle catalog schema in [`portals/shared/schemas/okf.ts`](../portals/shared/schemas/okf.ts) defines the type contract for vehicle stats:
```typescript
export const vehicleSchema = z.object({
  id: z.string(),
  name: z.string(),
  // ...
  brakes_kn: z.number(),
  stats: z.object({
    speed: z.number(),
    acceleration: z.number(),
    grip: z.number(),
    agility: z.number(),
    braking: z.number().optional(),
    downforce: z.number(),
  }),
  summary: z.string(),
});
```

### 2. Physics Engine Integration & Dynamic Parameter Modeling
Currently, [`crates/tdrace-app/src/catalog/mod.rs:to_car_config()`](../crates/tdrace-app/src/catalog/mod.rs) only overrides `mass`, `top_speed_mps`, `max_engine_force`, and `drive_bias`. The remaining chassis dynamics inherit uniform category defaults. 

Under Spec 018, `to_car_config()` dynamically modulates 5 additional parameters per car:

#### A. Braking Deceleration (`max_brake_force`)
$$F_{\text{brake}} = F_{\text{base\_brake}} \cdot \left(\frac{m_{\text{car}}}{m_{\text{base}}}\right) \cdot (0.80 + 0.40 \cdot \text{stats.4})$$
- Cars equipped with top-tier carbon-ceramic or multi-piston Brembo packages (`stats.4 >= 0.85`, e.g. Toyota GR Supra GT4 or Porsche GT3 R) brake substantially later into turns.
- Heavier vehicles with entry-level calipers experience realistic longer braking zones and require disciplined deceleration planning.

#### B. Pacejka Lateral Tire Grip (`tire.peak_d`)
$$D_{\text{lateral}} = D_{\text{base}} \cdot (0.85 + 0.30 \cdot \text{stats.2})$$
- Directly modulates Hans B. Pacejka's peak lateral force factor ($D$).
- High-grip cornering specialists hold higher mid-corner speeds ($\Delta v \approx 3\text{--}6\text{ km/h}$) before tire breakaway occurs.

#### C. Directional Agility & Steering Rack Speed (`steer_speed`)
$$\omega_{\text{steer}} = \omega_{\text{base\_steer}} \cdot (0.80 + 0.40 \cdot \text{stats.3})$$
- High-agility vehicles (e.g. lightweight sprint karts and mid-engine sports cars) track steering wheel and gamepad inputs with razor-sharp rapidity.
- Heavy stock cars and trucks exhibit authentic steering inertia.

#### D. Yaw Moment of Inertia ($I_z$)
$$I_z = I_{\text{base}} \cdot \left(\frac{m_{\text{car}}}{m_{\text{base}}}\right) \cdot (1.15 - 0.30 \cdot \text{stats.3})$$
- Centralized-mass, high-agility vehicles rotate eagerly around their vertical yaw axis with minimal rotational sluggishness, rewarding trail-braking flick techniques.

#### E. Aerodynamic Downforce & Drag Coefficients
Parsed directly from `self.aero_downforce` (format `"Cl X.XX / Cd Y.YY"`):
- `cfg.downforce_coefficient = parsed_cl * 0.76` (yielding authentic vertical suction scaling with $v^2$).
- `cfg.air_drag_coefficient = parsed_cd`.
- High-downforce cars (e.g. Hypercars and GT3s) remain glued in high-speed 200+ km/h sweepers, while low-downforce variants enjoy superior top-end straight-line speed.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

### 1. Deterministic Physics Verification & Simulation Safety
- Parameter derivation in `to_car_config()` must be 100% deterministic (IEEE 754 arithmetic without undefined floating point states or NaN).
- All multipliers clamp dynamically within safe physical bounds (e.g. `steer_speed.clamp(2.0, 15.0)`, `tire.peak_d.clamp(0.5, 1.8)`), preventing stability blowups or teleportation glitches in explicit Euler integration.

### 2. Data Integrity & Schema Validation
- Missing optional attributes in `vehicles.json` gracefully default to category standard baselines, avoiding client runtime crashes during offline or standalone showroom viewing.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
1. **Catalog Unit Tests**:
   `cargo test -p tdrace-app --lib catalog`
   Validates that `to_car_config()` produces non-zero, properly bounded, and differentiated physics configs across intra-category vehicle models.
2. **Showroom Web Portal Compilation**:
   `bun run build` (within `portals/option-b-showroom`)
   Verifies zero TypeScript errors, clean CSS generation, and valid HTML output.
3. **Keel Spec Suite Validation**:
   `keel validate`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Showroom displays 6 unified telemetry metrics**
  - [x] **Given** the user browses the Showroom vehicle catalog on any category card
  - [x] **When** the performance telemetry section renders
  - [x] **Then** exactly 6 horizontal single-row stats appear: Speed, Accel, Grip, Agility, Braking, and Downforce
  - [x] **And** each row displays its distinct icon, label, colored progress bar, and value
  - [x] **And** the total vertical card height does not exceed pre-redesign dimensions

- **Scenario: Showroom tooltips reveal detailed engineering specs**
  - [x] **Given** the user hovers over any telemetry stat row or icon
  - [x] **When** the cursor hovers on the element
  - [x] **Then** a high-contrast floating tooltip appears detailing exact physical metrics (km/h, 0-100s, kN braking, downforce $C_l$)

- **Scenario: Intra-category physics differentiation**
  - [x] **Given** two vehicles from the same competition class (e.g. Toyota GR Supra GT4 vs BMW M4 GT4)
  - [x] **When** `to_car_config()` derives their physical simulation parameters
  - [x] **Then** the lighter vehicle (Toyota) has higher `steer_speed`, lower `inertia`, higher `max_brake_force`, and higher `tire.peak_d`
  - [x] **And** the heavier vehicle (BMW) produces higher straight-line engine tractive force but requires earlier braking

---

## 🔗 Traceability & Codebase Mapping

### Modified Files
- `[x]` [`portals/option-b-showroom/src/components/CarCard.astro`](../portals/option-b-showroom/src/components/CarCard.astro) -> 6-row inline telemetry layout with SVG icons, tooltips, and compact layout.
- `[x]` [`portals/shared/schemas/okf.ts`](../portals/shared/schemas/okf.ts) -> Optional `braking` schema attribute.
- `[x]` [`scripts/generate_asset_data.py`](../scripts/generate_asset_data.py) -> Export `braking` stat to `vehicles.json`.
- `[x]` [`portals/shared/data/vehicles.json`](../portals/shared/data/vehicles.json) -> Synchronized vehicle stats.
- `[x]` [`crates/tdrace-app/src/catalog/mod.rs`](../crates/tdrace-app/src/catalog/mod.rs) -> Differentiated `to_car_config()` derivation and unit tests.
- `[x]` [`crates/tdrace-app/src/ui/garage.rs`](../crates/tdrace-app/src/ui/garage.rs) -> Telemetry HUD alignment.
- `[x]` [`specs/index.md`](index.md) -> Index registration of Spec 018.
- `[x]` [`specs/constitution/ROADMAP.md`](constitution/ROADMAP.md) -> Milestone registration.
