---
type: Asset Catalog
title: "Rallycross & All-Terrain Stages Catalog"
description: "Directory of 15 World RX mixed-surface circuits, Joker Lap branching networks, and jump ramps."
status: active
category: circuits
tags: [circuits, rally, rallycross, joker-lap, gravel, dirt]
---

# Rallycross & All-Terrain Stages Catalog 🌲🏜️🏁

The **Rallycross & All-Terrain** catalog in [`tracks/rally/`](../../tracks/rally) covers 15 world-renowned mixed-surface venues. These tracks combine asphalt acceleration launches with loose dirt sliding sections, massive jump ramp crests, and mandatory **Joker Lap alternative routes**.

---

## 🔀 The Joker Lap Mechanic

Every official rallycross circuit incorporates a **Joker Lap detour** modeled via the Directed Ribbon Graph (`TrackNetwork`) in [`crates/arcade-race-core/src/track/network.rs`](../../crates/arcade-race-core/src/track/network.rs):
* **Format Rule**: Every driver must take the Joker Lap detour exactly once per race heat.
* **Delta Penalty**: Detour adds approximately $+2.0 \dots +3.5\text{ s}$ to standard lap time.
* **Re-join Hazard**: Exit merges back into the main circuit right before the start/finish straight, demanding strategic timing to avoid traffic merges.

---

## 📋 Rallycross Circuits Roster (17 Circuits)

### Tier 1: Nordic & British Grassroots Rallycross (5 Starter Circuits)
| Venue | File Key | Lap Length | Surface Split | Joker Lap Layout | Track Highlights |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Höljes Motorstadion** | `holjes_rx.json` | $1,210\text{ m}$ | $60\%$ Asphalt / $40\%$ Gravel | Turn 5 Wide Sweeper | The "Magic Weekend"; iconic 35-meter jump crest over the velodrome. |
| **Lydden Hill** | `lydden_hill.json` | $1,300\text{ m}$ | $55\%$ Asphalt / $45\%$ Chalk/Dirt | Chessons Drift Outer Loop | The birthplace of rallycross; fast downhill entry into North Bend. |
| **Mettet (Circuit Jules Tacheny)** | `mettet_rx.json` | $1,031\text{ m}$ | $61\%$ Asphalt / $39\%$ Dirt | Turn 6 Inner Apex | Technical asphalt section followed by banked dirt bowl. |
| **Dreux RX (Circuit de l'Ouest Parisien)** | `dreux_rx.json` | $1,048\text{ m}$ | $62\%$ Asphalt / $38\%$ Dirt | Turn 4 Inner Hook | French Championship venue with high-speed sweeping tarmac and loose dirt hairpins. |
| **Blyton Park RX** | `blyton_rx.json` | $1,180\text{ m}$ | $58\%$ Asphalt / $42\%$ Gravel | Airfield Outer Loop | British classic on former RAF airfield with flowing flat curves and jump ramp. |

### Tier 2: Scandinavian & Continental Classics (3 Circuits)
| Venue | File Key | Lap Length | Surface Split | Joker Lap Layout | Track Highlights |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Lånkebanen (Hell RX)** | `hell_rx.json` | $1,019\text{ m}$ | $63\%$ Asphalt / $37\%$ Gravel | Turn 1 Outside Banking | Steep elevation changes; blind crests and sweeping gravel switchbacks. |
| **Lohéac RX** | `loheac_rx.json` | $1,088\text{ m}$ | $66\%$ Asphalt / $34\%$ Gravel | Turn 3 Inside Shortcut | Ultra-wide asphalt launch grid; high-speed braking into tight gravel hairpin. |
| **Silverstone RX** | `silverstone_rx.json`| $972\text{ m}$ | $50\%$ Asphalt / $50\%$ Gravel | Stowe Corner Detour | Technical gravel infield with high-banked dirt berms. |

### Tier 3: High-Octane European Supercar Tour (3 Circuits)
| Venue | File Key | Lap Length | Surface Split | Joker Lap Layout | Track Highlights |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Estering** | `estering_rx.json` | $952\text{ m}$ | $60\%$ Asphalt / $40\%$ Dirt | Final Turn Cut | Narrow, high-speed straight leading to dramatic first-corner hairpin contacts. |
| **Montalegre RX** | `montalegre_rx.json`| $1,046\text{ m}$ | $60\%$ Asphalt / $40\%$ Dirt | First Turn Outside Loop | Altitude circuit in Portuguese mountains; wet weather transforms dirt into deep mud. |
| **Bikernieki (Riga RX)** | `riga_rx.json` | $1,294\text{ m}$ | $60\%$ Asphalt / $40\%$ Parallel Gravel| Turn 2 Overpass Loop | Double jump ramps; zero runoff concrete barriers lined through pine forest. |

### Tier 4: Red Cauldron & Forest Stages (3 Circuits)
| Venue | File Key | Lap Length | Surface Split | Joker Lap Layout | Track Highlights |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Nyirád Racing Center** | `nyirad_rx.json` | $1,220\text{ m}$ | $48\%$ Asphalt / $52\%$ Red Clay | Final Turn Sweep | The "Red Cauldron"; deep abrasive bauxite clay that degrades tires rapidly. |
| **Kouvola RX** | `kouvola_rx.json` | $1,060\text{ m}$ | $58\%$ Asphalt / $42\%$ Gravel | Turn 4 Outer Loop | Technical Finnish forest circuit with severe camber shifts. |
| **Killarney International RX** | `killarney_rx.json` | $1,067\text{ m}$ | $60\%$ Asphalt / $40\%$ Sand/Gravel| Turn 3 Outer Hairpin | Tabletop jump ramp with Table Mountain backdrop; fine coastal sand dust. |

### Tier 5: World Championship Grand Finale (3 Circuits)
| Venue | File Key | Lap Length | Surface Split | Joker Lap Layout | Track Highlights |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Circuit de Barcelona-Catalunya RX**| `catalunya_rx.json`| $1,125\text{ m}$| $67\%$ Asphalt / $33\%$ Gravel | Stadium Sector Outer Loop | Built within the F1 stadium section; tight kerbs and loose gravel chicane. |
| **Yas Marina RX** | `yas_marina_rx.json` | $1,010\text{ m}$ | $63\%$ Asphalt / $37\%$ Gravel | North Hairpin Sweep | Floodlit night racing venue; high-grip asphalt abruptly meeting fine desert gravel. |
| **Circuit des Ducs (Essay RX)** | `essay_rx.json` | $936\text{ m}$ | $65\%$ Asphalt / $35\%$ Dirt | La Butte Dirt Jump | Historic French rallycross proving ground in Normandy with high-speed launch, technical hairpins, and wooded perimeter. |

---

For vehicle profiles suited for mixed-surface competition, see [Rallycross & All-Terrain Roster](../vehicles/rally_allterrain.md).
