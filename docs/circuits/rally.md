---
type: Asset Catalog
title: "Rallycross & All-Terrain Stages Catalog"
description: "Directory of 17 World RX mixed-surface circuits, Joker Lap branching networks, desert raid stages, and jump ramps."
status: active
category: circuits
tags: [circuits, rally, rallycross, joker-lap, gravel, dirt]
---

# Rallycross & All-Terrain Stages Catalog 🌲🏜️🏁

The **Rallycross & All-Terrain** catalog in [`tracks/rally/`](../../tracks/rally) covers 17 world-renowned mixed-surface venues. These tracks combine asphalt acceleration launches with loose dirt sliding sections, massive jump ramp crests, and mandatory **Joker Lap alternative routes**.

---

## 🔀 The Joker Lap Mechanic

Every official rallycross circuit incorporates a **Joker Lap detour** modeled via the Directed Ribbon Graph (`TrackNetwork`) in [`crates/arcade-race-core/src/track/network.rs`](../../crates/arcade-race-core/src/track/network.rs):
* **Format Rule**: Every driver must take the Joker Lap detour exactly once per race heat.
* **Delta Penalty**: Detour adds approximately $+2.0 \dots +3.5\text{ s}$ to standard lap time.
* **Re-join Hazard**: Exit merges back into the main circuit right before the start/finish straight, demanding strategic timing to avoid traffic merges.

---

## 📋 Rallycross Circuits Roster

| Venue | File Key | Lap Length | Surface Split | Joker Lap Layout | Track Highlights |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Höljes Motorstadion** | `holjes_rx.json` | $1,210\text{ m}$ | $60\%$ Asphalt / $40\%$ Gravel | Turn 5 Wide Sweeper | The "Magic Weekend"; iconic 35-meter jump crest over the velodrome. |
| **Lydden Hill** | `lydden_hill.json` | $1,300\text{ m}$ | $55\%$ Asphalt / $45\%$ Chalk/Dirt | Chessons Drift Outer Loop | The birthplace of rallycross; fast downhill entry into North Bend. |
| **Lånkebanen (Hell RX)** | `hell_rx.json` | $1,019\text{ m}$ | $63\%$ Asphalt / $37\%$ Gravel | Turn 1 Outside Banking | Steep elevation changes; blind crests and sweeping gravel switchbacks. |
| **Lohéac RX** | `loheac_rx.json` | $1,088\text{ m}$ | $66\%$ Asphalt / $34\%$ Gravel | Turn 3 Inside Shortcut | Ultra-wide asphalt launch grid; high-speed braking into tight gravel hairpin. |
| **Estering** | `estering_rx.json` | $952\text{ m}$ | $60\%$ Asphalt / $40\%$ Dirt | Final Turn Cut | Narrow, high-speed straight leading to dramatic first-corner hairpin contacts. |
| **Circuit de Barcelona-Catalunya RX**| `catalunya_rx.json`| $1,125\text{ m}$| $67\%$ Asphalt / $33\%$ Gravel | Stadium Sector Outer Loop | Built within the F1 stadium section; tight kerbs and loose gravel chicane. |
| **Silverstone RX** | `silverstone_rx.json`| $972\text{ m}$ | $50\%$ Asphalt / $50\%$ Gravel | Stowe Corner Detour | Technical gravel infield with high-banked dirt berms. |
| **Bikernieki (Riga RX)** | `riga_rx.json` | $1,294\text{ m}$ | $60\%$ Asphalt / $40\%$ Parallel Gravel| Turn 2 Overpass Loop | Double jump ramps; zero runoff concrete barriers lined through pine forest. |
| **Mettet (Circuit Jules Tacheny)** | `mettet_rx.json` | $1,031\text{ m}$ | $61\%$ Asphalt / $39\%$ Dirt | Turn 6 Inner Apex | Technical asphalt section followed by banked dirt bowl. |
| **Montalegre RX** | `montalegre_rx.json`| $1,046\text{ m}$ | $60\%$ Asphalt / $40\%$ Dirt | First Turn Outside Loop | Altitude circuit in Portuguese mountains; wet weather transforms dirt into deep mud. |
| **Kouvola RX** | `kouvola_rx.json` | $1,060\text{ m}$ | $58\%$ Asphalt / $42\%$ Gravel | Turn 4 Outer Loop | Technical Finnish forest circuit with severe camber shifts. |
| **Nyirád Racing Center** | `nyirad_rx.json` | $1,220\text{ m}$ | $48\%$ Asphalt / $52\%$ Red Clay | Final Turn Sweep | The "Red Cauldron"; deep abrasive bauxite clay that degrades tires rapidly. |
| **Killarney International RX** | `killarney_rx.json` | $1,067\text{ m}$ | $60\%$ Asphalt / $40\%$ Sand/Gravel| Turn 3 Outer Hairpin | Tabletop jump ramp with Table Mountain backdrop; fine coastal sand dust. |
| **Yas Marina RX** | `yas_marina_rx.json` | $1,010\text{ m}$ | $63\%$ Asphalt / $37\%$ Gravel | North Hairpin Sweep | Floodlit night racing venue; high-grip asphalt abruptly meeting fine desert gravel. |
| **Sahara Dunes Raid** | `sahara_dunes.json` | $4,850\text{ m}$ | $100\%$ Sand | Natural Ridge Alternative | Open desert rally raid; high dunes, jump crests, severe sand deceleration traps. |
| **Sahara Oasis Stage** | `sahara.json` | $3,650\text{ m}$ | $70\%$ Dirt / $30\%$ Sand | Palm Oasis Crossing | Mixed desert track with water hazard splashdown zones. |
| **Outlaw Pass Rally** | `outlaw_pass.json` | $3,200\text{ m}$ | $70\%$ Asphalt / $30\%$ Dirt | Canyon Spur Line | Mountain pass with cliff drop-offs and gravel chicane shortcuts. |

---

For vehicle profiles suited for mixed-surface competition, see [Rallycross & All-Terrain Roster](../vehicles/rally_allterrain.md).
