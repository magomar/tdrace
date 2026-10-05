---
type: Feature Spec
template: feature
title: "Tactical Cockpit HUD Tire Compound Borders, Bottom Legend, and Animated Wheel Color Decoupling"
description: "Removes intrusive procedural color bands from the in-race animated steered wheels, transfers compound visual identification to the outer tire casing borders in the Tactical Hologram Cockpit Telemetry HUD, and introduces a dedicated compound badge and full nameplate legend at the bottom of the cockpit HUD card."
status: implemented
verified: { by: "human:mario", at: "2026-10-05T15:40:12Z", hash: "a6a9283592dd" }
created: 2026-10-05
generated: { by: agent/antigravity, at: 2026-10-05T14:43:44Z }
depends_on:
  - "074"
  - "079"
---

# Feature Spec: Tactical Cockpit HUD Tire Compound Borders, Bottom Legend, and Animated Wheel Color Decoupling 🛞✨🏎️

A targeted visual refinement and cockpit instrumentation specification for **TDRace**, eliminating artificial procedural color bands glued to the animated wheels in the top-down race view, relocating compound color identification to the 4-corner wheel casing borders in the **Tactical Hologram Cockpit Telemetry HUD**, and presenting a polished compound badge and full nameplate legend at the bottom of the cockpit HUD glass bezel.

---

## 🎯 Executive Summary & Problem Statement

### 1. The World Sprite Visual Artifact
Under [Spec 074](074_decoupled_wheel_geometry_and_data_driven_tire_compounds.md) and [Spec 073](073_realistic_vehicle_sprite_harmonization_and_modular_steered_wheel_articulation.md), modular Ackermann steered wheels were introduced across the vehicle fleet. To indicate tire compound without generating dozens of texture permutations ($N \text{ archetypes} \times M \text{ compounds}$), [`draw_steered_wheel_with_accent`](../crates/tdrace-app/src/render/car.rs) overlaid a procedural line on the outer sidewall rim lip of the steered front wheels using `car.config.wheels[0].compound.id.accent_rgba()`.

In practice, this produced glaring visual defects:
- On open-wheelers like the **Mudlark Cross Car** (`classic_ax_mudlark`) and **Alpine Lynx** (`classic_kart`), prominent orange or red rectangular bars appear pasted onto the outer edges of the tires.
- The bands resemble neon tape or rendering glitches rather than authentic motorsport tire rubber, breaking visual immersion in top-down racing.
- The rear wheels (which are embedded in the chassis sprite or fixed) do not receive this line, creating an asymmetrical and inconsistent look between front and rear axles.

### 2. Relocating Compound Identity to the Cockpit HUD
[Spec 079](079_tactical_hologram_cockpit_telemetry_hud.md) established the **Tactical Hologram Cockpit Telemetry HUD** ([`crates/race-ui/src/hud/chassis_telemetry.rs`](../crates/race-ui/src/hud/chassis_telemetry.rs)) to provide clean, proportional vehicle geometry, powertrain damage, and decoupled tire telemetry.

Currently in the HUD:
- The 4-corner tire assemblies draw both their outer casing border and their inner rubber fill using the tire temperature color (`thermal_col`: cyan/green/gold/red).
- The active tire compound is only represented by a tiny badge pill in the upper-left header bezel (`render_compound_badge`), without displaying the compound's full descriptive name.

### 3. Core Design Objectives
1. **Clean World Vehicle Presentation**: Completely remove the procedural outer lip color band from live in-race steered wheel rendering. Wheels render cleanly with their authentic texture, rim shading, and Ackermann steering rotation.
2. **Compound-Bordered Cockpit Wheels**: In the Tactical Hologram HUD, color the perimeter casing border of all 4 tires with the active compound's FIA accent color (`compound.accent_rgba()`). This provides an immediate, high-contrast visual frame identifying the fitted tire type.
3. **Inner Temperature & Wear Preservation**: The inner rubber fill inside each HUD wheel continues to communicate real-time bulk temperature (Cyan = Cold, Green = Optimal, Gold = Warm, Red = Overheated) and vertically drains with mechanical tread wear.
4. **Bottom HUD Compound Legend**: Add a dedicated compound indicator at the bottom of the HUD card displaying the colored compound badge pill (e.g. `[S]`, `[M]`, `[H]`, `[AT]`, `[INT]`, `[W]`, `[MUD]`, `[ICE]`) alongside the full compound name (e.g. "Soft Slick", "Medium Slick", "Hard Slick", "All-Terrain", etc.) in crisp UI typography.

---

## 🗺️ User Flow & Interface Design

### 1. In-Race World View
- When driving any vehicle with modular steered wheels (e.g., `classic_kart`, `classic_ax_mudlark`, `classic_gt`, `classic_nascar`, `classic_offroad`, `classic_rally`):
  - Front wheels articulate dynamically with Ackermann steering angles.
  - Zero synthetic color stripes, tape bands, or neon lines are drawn on the tires.
  - Tires present uniform, realistic black racing rubber and wheel rims matching the vehicle archetype.

### 2. Tactical Hologram Cockpit Telemetry HUD
```
┌──────────────────────────────────────────────────┐
│ [S] (badge)                        KINEMATICS ── │  <- Header Bezel
│                                                  │
│         [FL Tire]              [FR Tire]         │  <- Compound-colored outer borders
│         ┌───────┐              ┌───────┐         │     Inner fill: Temp + Wear drain
│         │ ~~~~~ │              │ ~~~~~ │         │
│         └───────┘              └───────┘         │
│             \                      /             │
│              === Chassis Skeleton ===            │
│             /                      \             │
│         [RL Tire]              [RR Tire]         │
│         ┌───────┐              ┌───────┐         │
│         │ ~~~~~ │              │ ~~~~~ │         │
│         └───────┘              └───────┘         │
│                                                  │
│         [S]  Soft Slick                          │  <- New Bottom Compound Legend
└──────────────────────────────────────────────────┘
```

#### Detailed HUD Components:
1. **4 Outboard Tires (`draw_integrated_tire`)**:
   - **Outer Casing Border**: Drawn with line thickness `scaler.s(2.2)` using `compound.accent_rgba()`.
     - `SoftSlick`: Vivid Red (`#F22626`)
     - `MediumSlick`: Golden Yellow (`#F2D91A`)
     - `HardSlick`: Bright White (`#E6E6E6`)
     - `AllTerrain`: Earthy Orange (`#F28C1A`)
     - `IntermediateWet`: Emerald Green (`#26CC33`)
     - `MonsoonWet`: Deep Blue (`#1A80F2`)
     - `ExtremeMud`: Clay Brown (`#8C5926`)
     - `StuddedIce`: Arctic Cyan (`#99E6FF`)
   - **Inner Rubber Fill**: Retains dynamic temperature coloration (`tire_temp_to_color(temp_celsius)`) and drains vertically downward according to remaining tread ratio `(1.0 - wear)`.
   - **Midpoint Tick Marks**: Left and right sidewall ticks remain at 50% height.

2. **Bottom Bezel Compound Legend**:
   - Positioned horizontally centered at the bottom of the card (`y + box_h - scaler.s(18.0)`).
   - Displays the compact FIA badge pill:
     - Background: dark translucent fill `Color::new(col.r * 0.22, col.g * 0.22, col.b * 0.22, 0.90)`.
     - Outline: `col` with `scaler.s(1.2)` thickness.
     - Text: Bold acronym (`S`, `M`, `H`, `AT`, `INT`, `W`, `MUD`, `ICE`) in `col`.
   - Displays the full compound title adjacent to the pill:
     - Font: `scaler.font_s(11.5)`.
     - Color: `Palette::WHITE` / `Color::new(0.92, 0.94, 0.98, 1.0)`.
     - Text: `compound.name()` (e.g. "Soft Slick", "All-Terrain", "Monsoon Wet").

---

## ⚙️ Backend Models & API Endpoints

### 1. Presentation Wheel API Signature Cleanup
In [`crates/tdrace-app/src/render/car.rs`](../crates/tdrace-app/src/render/car.rs):
- `draw_steered_wheel_with_accent` deprecates the outer lip line drawing or becomes an alias for `draw_steered_wheel`.
- In-race vehicle top-down chassis and wheel rendering loop ceases passing procedural color accents to wheels.

### 2. HUD Tire Assembly Interface
In [`crates/race-ui/src/hud/chassis_telemetry.rs`](../crates/race-ui/src/hud/chassis_telemetry.rs):
```rust
fn draw_integrated_tire(
    center: Vec2,
    wheel_w: f32,
    wheel_h: f32,
    temp_celsius: f32,
    wear: f32,
    rotation_rad: f32,
    border_col: Color,
    scaler: &UiScaler,
);
```
- Parameters:
  - `border_col`: `Color` derived from `car.state.wheel_assemblies[i].config.compound.id.accent_rgba()`.
  - `temp_celsius`: Feeds the inner rubber gradient and thermal state.
  - `wear`: Controls the vertical drain ratio of the inner rubber.

### 3. Bottom Compound Legend Layout Contract
In [`render_cockpit_chassis_telemetry`](../crates/race-ui/src/hud/chassis_telemetry.rs):
- Reads active compound from `car.state.wheel_assemblies[0].config.compound.id`.
- Queries `compound.badge_code()` and `compound.name()`.
- Calculates proportional horizontal centering:
  $$\text{total\_w} = w_{\text{pill}} + \text{gap} + w_{\text{name}}$$
  $$x_{\text{legend}} = x + (w_{\text{box}} - \text{total\_w}) \cdot 0.5$$
  $$y_{\text{legend}} = y + h_{\text{box}} - \text{scaler.s}(19.0)$$

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

1. **Deterministic Presentation Isolation**: The Tactical Hologram HUD and vehicle wheel rendering pipeline are strictly presentation-layer systems located in `crates/race-ui` and `crates/tdrace-app`. No UI rendering logic or compound color calculations mutate deterministic simulation state in `wheelbase`.
2. **Headless Simulation Boundary**: Core simulation crates (`wheelbase`, `arcade-race-core`, `race-kit`) remain completely free of graphics dependencies (`macroquad`, OpenGL, shaders).
3. **Allocation Guarantees & Real-Time Performance**: All layout calculations in `render_cockpit_chassis_telemetry` and `draw_integrated_tire` are stack-allocated, non-blocking floating point operations executing within the 60–120 FPS game loop without heap pressure.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
1. **App Compilation & Render Tests**:
   ```bash
   cargo test -p tdrace-app test_render -- --nocapture
   ```
2. **Race UI HUD Regression Suite**:
   ```bash
   cargo test -p race-ui -- --nocapture
   ```
3. **Compound Color & Name Contract Consistency**:
   ```bash
   cargo test -p wheelbase test_compound -- --nocapture
   ```

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: In-race steered wheels render without color bands**
  - [x] **Given** a vehicle with modular steered wheels (`classic_kart`, `classic_ax_mudlark`, or `classic_gt`)
  - [x] **When** driving on track and turning the steering wheel at full lock
  - [x] **Then** the steered front wheels articulate dynamically to match Ackermann angles
  - [x] **And** no colored bands, neon lines, or procedural stripes appear on the outer tire rim lips

- **Scenario: Cockpit HUD wheels show tire compound colored border**
  - [x] **Given** the Tactical Hologram Cockpit Telemetry HUD is active during a race
  - [x] **When** observing the 4 corner wheel assemblies in the HUD
  - [x] **Then** the outer perimeter casing border of all 4 tires is colored with the active tire compound's accent color (e.g. Red for SoftSlick, Orange for AllTerrain)
  - [x] **And** the inner rubber fill displays the live thermal temperature gradient draining with tread wear

- **Scenario: Cockpit HUD displays compound badge and name at the bottom**
  - [x] **Given** any vehicle equipped with a specific tire compound (e.g., Kart with Soft Slick, Mudlark with All-Terrain)
  - [x] **When** viewing the Tactical Hologram Cockpit Telemetry HUD card
  - [x] **Then** the bottom of the card displays a colored badge pill with the compound code (e.g. `[S]`, `[AT]`)
  - [x] **And** immediately beside the pill, the full compound name is rendered (e.g. "Soft Slick", "All-Terrain") in clear white typography

---

## 🔗 Traceability & Codebase Mapping

### Modified Files
- `[x]` [`crates/tdrace-app/src/render/car.rs`](../crates/tdrace-app/src/render/car.rs) -> Removes outer sidewall color highlight lines from steered wheel rendering.
- `[x]` [`crates/race-ui/src/hud/chassis_telemetry.rs`](../crates/race-ui/src/hud/chassis_telemetry.rs) -> Colors HUD tire borders with compound accent color and adds the bottom compound legend.
- `[x]` [`crates/race-ui/src/hud/widgets.rs`](../crates/race-ui/src/hud/widgets.rs) -> Implements render_compound_legend widget.

### Verification Assertions
- Verification receipts will be recorded via `keel receipt 089 --cmd "cargo test -p race-ui -p tdrace-app"`.
