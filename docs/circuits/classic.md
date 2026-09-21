---
type: Asset Catalog
title: "Classic Heritage Circuit Catalog"
description: "Directory of 10 foundational TdRace tracks: Classic Grand Prix, Drift Park, Figure Eight crossovers, and jump ramp raceways."
status: active
category: circuits
tags: [circuits, classic, tracks, grand-prix, drift-park]
---

# Classic Heritage Circuit Catalog 🏎️🏛️

The **Classic Heritage** tracks form the foundational proving grounds of **TdRace**. Located in [`tracks/classic/`](../../tracks/classic), these 10 venues showcase foundational mechanics: asphalt apex clipping, high-risk intersection crossings, dirt sliding, and 2.5D jump ramps.

> [!NOTE]
> Classic circuits are decoupled from specific modern motorsport modules (GT, Karting, Rallycross). They are available under the **Classic Motorsport** category and can be driven by any classic vehicle preset or imported into the Track Studio.

---

## 📋 Track Roster & Specifications

| Circuit | File Key | Discipline Category | Lap Length | Turns | Primary Surface | Special Hazards & Features |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **Classic Grand Prix** | `classic_grand_prix.json` | `GT` | $903\text{ m}$ | 12 | `Asphalt` | FIA Grade curb teeth, chicane, sweeping curves, hairpin sand trap, pit lane. |
| **Drift Park** | `drift_park.json` | `GT` | $1,420\text{ m}$ | 9 | `Asphalt` | Tight hairpin switchbacks, extended radius clipping zones, polished tarmac. |
| **Figure Eight** | `figure_eight.json` | `GT` | $1,850\text{ m}$ | 8 | `Asphalt` | 2.5D elevation overpass / underpass bridge crossing. |
| **Oval Speedway** | `oval_speedway.json` | `NASCAR` | $1,200\text{ m}$ | 4 | `Asphalt` | Banked tri-oval turns, slipstream drafting battles, steel Armco barriers. |
| **Dirty Oval Speedway** | `dirty_oval_speedway.json` | `NASCAR` | $800\text{ m}$ | 4 | `Dirt` | Compacted clay surface, heavy rooster plumes, cushion sliding line. |
| **Oasis Rally** | `oasis_rally.json` | `RALLY` | $2,450\text{ m}$ | 11 | `Dirt` / `Sand` | Desert trail circling water hazard; deep sand deceleration traps. |
| **Classic Rallycross** | `classic_rallycross.json` | `RALLY` | $1,148\text{ m}$ | 6 | `Asphalt` / `Dirt` | Dynamic mixed-surface rallycross circuit with launch straights, dirt hairpins & tabletop jumps. |
| **Dirt Figure Eight** | `dirt_figure_eight.json` | `OFFROAD` | $801\text{ m}$ | 8 | `Dirt` | Flat at-grade crossover intersection, high risk of head-on collisions. |
| **Ramp Raceway** | `ramp_raceway.json` | `OFFROAD` | $2,100\text{ m}$ | 10 | `Asphalt` / `Dirt` | Triple elevated jump ramps over sand pits; aerodynamic vehicle pitch control. |
| **Kart Arena** | `kart_arena.json` | `KART` | $950\text{ m}$ | 14 | `Asphalt` | Compact indoor sprint layout with tire wall chicanes. |

---

## 🔍 Featured Circuit: Classic Grand Prix
* **File Location**: [`tracks/classic/classic_grand_prix.json`](../../tracks/classic/classic_grand_prix.json)
* **Surface Composition**: $85\%$ Asphalt, $10\%$ Kerb, $5\%$ Grass runoff.
* **Telemetry Hotspots**:
  * **Turn 1 (Castrol Hairpin)**: Heavy braking from $280\text{ km/h}$ down to $90\text{ km/h}$; tests forward weight transfer and ABS.
  * **Turns 4-6 (The Esses)**: Rapid lateral weight transfer switchbacks; rewards stiff anti-roll bars.
  * **Final Turn (Apex Sweeper)**: High-speed parabolic sweeper where aerodynamic downforce dominates.

For classic vehicle profiles (GT Sports Coupe, Tuned Drift Spec, 125cc Shifter Kart, AWD Turbo Rally) and motorsport rosters, see the [Vehicles Overview](../vehicles/index.md).
