---
type: Asset Catalog
title: "Vehicle Roster & Motorsport Categories Catalog"
description: "Master catalog of 80 authentic real-world car models across 25 progression categories in 5 motorsport modules, vehicle specs, and Balance of Performance (BoP) calibration."
status: active
category: vehicles
tags: [vehicles, catalog, roster, bop, garage]
---

# Vehicle Roster & Motorsport Categories Catalog 🏎️🏆

**TdRace** features **80 authentic motorsport car models** across 25 distinct progression categories structured in 5 core motorsport modules, representing 44 real-world manufacturers. Every category is mathematically balanced using intra-class Balance of Performance (BoP) equations to ensure close racing while honoring authentic mechanical individuality.

---

## 🏛️ The 5 Motorsport Modules

| Module | Document Reference | Tiers & Progression | Mechanical Character |
| :--- | :--- | :--- | :--- |
| **Gran Turismo & Endurance** | [gt_endurance.md](gt_endurance.md) | **Tier 1**: GT4<br>**Tier 2**: GT3<br>**Tier 3**: GT2<br>**Tier 4**: GT1<br>**Tier 5**: LMH / LMDh Hypercar | High downforce, carbon-ceramic brakes, slick racing tires, balanced RWD setups. |
| **NASCAR Stock Car Racing** | [stock_car.md](stock_car.md) | **Tier 1**: Street Stock<br>**Tier 2**: Late Model<br>**Tier 3**: ARCA Menards<br>**Tier 4**: Craftsman Super Truck<br>**Tier 5**: Trans-Am TA1 & Cup | Heavy steel tubular chassis, high-power naturally aspirated pushrod V8s, minimal downforce, asymmetric oval setups. |
| **Rallycross & All-Terrain** | [rally_allterrain.md](rally_allterrain.md) | **Tier 1**: Rally Jr FWD<br>**Tier 2**: WRC / World RX Supercar<br>**Tier 3**: Group B Legends<br>**Tier 4**: Dakar Rally Raid T1+<br>**Tier 5**: Stadium Super Truck (SST) | Rapid-acceleration AWD turbos, extreme suspension travel, loose-surface drift balance, 2.5D jump landing dampening. |
| **Extreme Off-Road & Arenas** | [offroad_stunt.md](offroad_stunt.md) | **Tier 1**: Sand Rail Buggy<br>**Tier 2**: Trophy Truck Spec<br>**Tier 3**: Arctic Ice Drifter<br>**Tier 4**: Mud Bogger 4x4<br>**Tier 5**: Monster Truck Freestyle | Enormous tire diameters, locked differentials, extreme ground clearance, low tire pressures for mud and sand bogs. |
| **Karting & Micro-Racers** | [karting.md](karting.md) | **Tier 1**: 60cc Cadet Kart<br>**Tier 2**: 100cc OK Junior<br>**Tier 3**: 125cc KZ2 Shifter<br>**Tier 4**: Racing Lawnmower Pro<br>**Tier 5**: 250cc Superkart GP | Zero suspension (flexible chassis frame roll), ultra-low CG, direct 1:1 steering ratio, explosive power-to-weight ratio. |

---

## ⚖️ Balance of Performance (BoP) Framework

Within every multi-car category, vehicles are tuned to match a normalized class lap-time window ($\Delta t \le 0.15\text{ s}$) around the benchmark circuit while offering distinct handling tradeoffs:

$$\text{Lap Pace Index} = w_1 \cdot \frac{F_{\text{engine}}}{m} + w_2 \cdot D_{\text{tire}} + w_3 \cdot C_{\text{downforce}} - w_4 \cdot C_{\text{drag}}$$

* **Lightweight Cornering Specialists**: Lower mass ($m$), lower top speed, higher cornering apex grip ($D$).
* **Power & Straight-Line Brawlers**: Higher horsepower ($F_{\text{engine}}$), higher mass, higher top speed ($v_{\text{top}}$).

Select a module above to view detailed car specifications and telemetry profiles, or explore the [Circuits Directory](../circuits/index.md).
