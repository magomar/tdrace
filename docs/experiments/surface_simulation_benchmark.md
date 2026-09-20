---
type: Physics Reference
title: "Empirical Benchmark: Multi-Surface Automotive Dynamics"
description: "Empirical benchmark report on vehicle slip, acceleration, and terminal velocity across 11 surfaces."
status: active
category: experiments
tags: [physics, surface, benchmark, telemetry, simulation]
---

# 🔬 Empirical Benchmark Report: Surface-Car Dynamics Simulation

**Experiment**: `Multi-Surface Automotive Dynamics Benchmark (Non-Classic Modules)`
**Timestamp**: `2026-09-19T04:56:37.554397323+00:00`
**Vehicles Tested**: 12
**Surfaces Evaluated**: 11 surfaces

## 📋 1. Vehicle Roster (1 Vehicle Per Category Across Non-Classic Modules)

| Module | Category | Vehicle ID | Display Name |
|:---|:---|:---|:---|
| **GT** | GT4 | `gt4_clubsport` | 420 BHP GT4 Clubsport |
| **GT** | GT3 | `gt3_evo` | 600 BHP GT3 Evo Racer |
| **GT** | GT2 | `gt2_biturbo` | 707 BHP GT2 Biturbo Sprint |
| **GT** | GT1 | `gt1_legend` | 650 BHP GT1 Le Mans Legend |
| **GT** | LMH Hypercar | `hypercar_prototype` | 800 BHP LMH Hypercar Prototype |
| **NASCAR** | NASCAR Cup | `nascar_cup_v8` | NASCAR Cup Next-Gen V8 |
| **NASCAR** | Trans-Am TA1 | `trans_am_ta1` | Trans-Am TA1 Spaceframe V8 |
| **Rally** | Modern WRC | `wrc_turbo_rally` | Apex WRC Turbo AWD |
| **Rally** | Historic Group B | `group_b_beast` | Quattro Group B Spec |
| **Kart** | Sprint Shifter Kart | `shifter_kart_125` | 125cc Shifter Kart Super Sprint |
| **Extreme Off-Road** | Sand Rail Buggy | `sand_rail_buggy` | 300 BHP Sand Rail Buggy |

## 🚀 2. Protocol A: Longitudinal Acceleration & Traction

Elapsed time to 100 km/h ($t_{100}$ in seconds) across surfaces (*DNC* = did not reach 100 km/h):

| Vehicle | Asphalt | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **420 BHP GT4 Clubsport** | 5.66s | 6.55s | 7.48s | 8.72s | 16.14s | *DNC* (70k) | 21.87s | *DNC* (1k) | *DNC* (65k) | *DNC* (45k) | *DNC* (31k) |
| **600 BHP GT3 Evo Racer** | 5.55s | 6.43s | 7.35s | 8.59s | 16.82s | *DNC* (68k) | 22.16s | *DNC* (1k) | *DNC* (63k) | *DNC* (44k) | *DNC* (31k) |
| **707 BHP GT2 Biturbo Sprint** | 5.62s | 6.50s | 7.42s | 8.65s | 16.14s | *DNC* (69k) | 21.80s | *DNC* (1k) | *DNC* (64k) | *DNC* (45k) | *DNC* (31k) |
| **650 BHP GT1 Le Mans Legend** | 5.47s | 6.33s | 7.24s | 8.46s | 16.54s | *DNC* (68k) | 21.81s | *DNC* (1k) | *DNC* (63k) | *DNC* (45k) | *DNC* (31k) |
| **800 BHP LMH Hypercar Prototype** | 6.40s | 7.42s | 8.51s | 9.99s | 23.15s | *DNC* (57k) | *DNC* (94k) | *DNC* (1k) | *DNC* (54k) | *DNC* (38k) | *DNC* (26k) |
| **NASCAR Cup Next-Gen V8** | 5.25s | 6.11s | 7.01s | 8.16s | 14.85s | *DNC* (82k) | 20.56s | *DNC* (2k) | *DNC* (68k) | *DNC* (46k) | *DNC* (32k) |
| **Trans-Am TA1 Spaceframe V8** | 5.25s | 6.11s | 7.01s | 8.16s | 14.85s | *DNC* (82k) | 20.56s | *DNC* (2k) | *DNC* (68k) | *DNC* (46k) | *DNC* (32k) |
| **Apex WRC Turbo AWD** | 3.71s | 3.86s | 4.06s | 4.31s | 5.80s | 6.62s | 8.73s | 11.18s | 14.15s | *DNC* (98k) | *DNC* (66k) |
| **Quattro Group B Spec** | 2.87s | 3.26s | 3.68s | 4.13s | 5.86s | 6.66s | 8.79s | 11.72s | 14.43s | *DNC* (97k) | *DNC* (66k) |
| **125cc Shifter Kart Super Sprint** | 5.82s | 7.02s | 8.36s | 10.72s | *DNC* (62k) | *DNC* (31k) | *DNC* (70k) | *DNC* (1k) | *DNC* (43k) | *DNC* (38k) | *DNC* (27k) |
| **300 BHP Sand Rail Buggy** | 3.48s | 4.18s | 4.94s | 5.86s | 12.91s | *DNC* (93k) | 17.30s | *DNC* (41k) | *DNC* (75k) | *DNC* (55k) | *DNC* (38k) |

## 🛑 3. Protocol B: Emergency Braking Distance (100 → 0 km/h)

Stopping distance ($d_{\text{stop}}$ in meters) from an initial speed of 100 km/h:

| Vehicle | Asphalt | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **420 BHP GT4 Clubsport** | 37.7m | 42.7m | 48.1m | 53.3m | 67.6m | 79.5m | 105.5m | 105.6m | 154.2m | 286.9m | 412.6m |
| **600 BHP GT3 Evo Racer** | 36.2m | 41.0m | 46.1m | 51.0m | 63.9m | 75.4m | 100.1m | 98.0m | 144.8m | 270.2m | 386.5m |
| **707 BHP GT2 Biturbo Sprint** | 37.2m | 42.1m | 47.4m | 52.5m | 66.5m | 78.2m | 103.8m | 103.5m | 151.5m | 282.1m | 405.5m |
| **650 BHP GT1 Le Mans Legend** | 35.3m | 40.0m | 45.0m | 49.9m | 62.5m | 73.7m | 97.9m | 95.7m | 141.5m | 264.1m | 377.8m |
| **800 BHP LMH Hypercar Prototype** | 40.0m | 45.3m | 50.9m | 56.2m | 68.7m | 81.6m | 108.4m | 101.8m | 153.7m | 288.2m | 403.8m |
| **NASCAR Cup Next-Gen V8** | 37.2m | 42.1m | 47.4m | 52.6m | 66.6m | 78.3m | 103.9m | 103.8m | 151.8m | 282.5m | 406.2m |
| **Trans-Am TA1 Spaceframe V8** | 37.2m | 42.1m | 47.4m | 52.6m | 66.6m | 78.3m | 103.9m | 103.8m | 151.8m | 282.5m | 406.2m |
| **Apex WRC Turbo AWD** | 38.4m | 43.6m | 49.0m | 54.4m | 69.3m | 81.4m | 108.0m | 109.1m | 158.6m | 294.8m | 423.8m |
| **Quattro Group B Spec** | 38.1m | 43.1m | 48.5m | 53.7m | 67.8m | 79.8m | 105.9m | 104.8m | 154.0m | 287.0m | 410.9m |
| **125cc Shifter Kart Super Sprint** | 35.8m | 40.2m | 44.7m | 48.7m | 52.5m | 64.2m | 85.8m | 67.4m | 110.7m | 212.2m | 283.1m |
| **300 BHP Sand Rail Buggy** | 37.2m | 42.0m | 47.1m | 52.0m | 62.5m | 74.5m | 99.1m | 90.5m | 138.6m | 260.7m | 364.8m |

## 🔄 4. Protocol C: Steady-State Skidpad Cornering Limit ($R = 30\text{m}$)

Peak lateral acceleration ($a_{y,\max}$ in $g$) on constant-radius skidpad:

| Vehicle | Asphalt | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **420 BHP GT4 Clubsport** | 0.16g | 0.16g | 0.16g | 0.16g | 0.14g | 0.11g | 0.15g | 0.07g | 0.15g | 0.11g | 0.07g |
| **600 BHP GT3 Evo Racer** | 0.16g | 0.16g | 0.16g | 0.16g | 0.15g | 0.12g | 0.16g | 0.08g | 0.15g | 0.11g | 0.07g |
| **707 BHP GT2 Biturbo Sprint** | 0.16g | 0.16g | 0.16g | 0.16g | 0.15g | 0.12g | 0.16g | 0.09g | 0.15g | 0.11g | 0.07g |
| **650 BHP GT1 Le Mans Legend** | 0.16g | 0.16g | 0.16g | 0.16g | 0.15g | 0.12g | 0.15g | 0.09g | 0.15g | 0.11g | 0.07g |
| **800 BHP LMH Hypercar Prototype** | 0.16g | 0.16g | 0.16g | 0.16g | 0.15g | 0.13g | 0.16g | 0.10g | 0.15g | 0.09g | 0.06g |
| **NASCAR Cup Next-Gen V8** | 0.14g | 0.14g | 0.14g | 0.14g | 0.13g | 0.11g | 0.14g | 0.09g | 0.13g | 0.11g | 0.07g |
| **Trans-Am TA1 Spaceframe V8** | 0.14g | 0.14g | 0.14g | 0.14g | 0.13g | 0.11g | 0.14g | 0.09g | 0.13g | 0.11g | 0.07g |
| **Apex WRC Turbo AWD** | 0.59g | 0.55g | 0.53g | 0.49g | 0.39g | 0.31g | 0.29g | 0.22g | 0.20g | 0.11g | 0.07g |
| **Quattro Group B Spec** | 0.58g | 0.55g | 0.52g | 0.49g | 0.39g | 0.33g | 0.29g | 0.23g | 0.20g | 0.11g | 0.07g |
| **125cc Shifter Kart Super Sprint** | 0.72g | 0.61g | 0.53g | 0.47g | 0.34g | 0.22g | 0.25g | 0.08g | 0.16g | 0.10g | 0.06g |
| **300 BHP Sand Rail Buggy** | 0.61g | 0.57g | 0.52g | 0.46g | 0.33g | 0.20g | 0.25g | 0.07g | 0.16g | 0.10g | 0.07g |

## 🍃 5. Protocol E: Passive Coast-Down Distance (120 → 0 km/h)

Distance rolled ($d_{\text{coast}}$ in meters) under purely aerodynamic and rolling resistance:

| Vehicle | Asphalt | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **420 BHP GT4 Clubsport** | 1289m | 1218m | 1222m | 939m | 368m | 185m | 776m | 141m | 649m | 1343m | 1436m |
| **600 BHP GT3 Evo Racer** | 1191m | 1119m | 1122m | 841m | 326m | 170m | 686m | 128m | 572m | 1246m | 1340m |
| **707 BHP GT2 Biturbo Sprint** | 1267m | 1195m | 1199m | 916m | 358m | 180m | 754m | 138m | 630m | 1322m | 1416m |
| **650 BHP GT1 Le Mans Legend** | 1172m | 1099m | 1102m | 820m | 317m | 165m | 667m | 125m | 557m | 1228m | 1323m |
| **800 BHP LMH Hypercar Prototype** | 1120m | 1056m | 1054m | 811m | 316m | 177m | 661m | 130m | 550m | 1170m | 1254m |
| **NASCAR Cup Next-Gen V8** | 1285m | 1216m | 1219m | 953m | 377m | 193m | 793m | 138m | 662m | 1336m | 1425m |
| **Trans-Am TA1 Spaceframe V8** | 1285m | 1216m | 1219m | 953m | 377m | 193m | 793m | 138m | 662m | 1336m | 1425m |
| **Apex WRC Turbo AWD** | 1313m | 1237m | 1244m | 944m | 369m | 179m | 779m | 146m | 653m | 1369m | 1467m |
| **Quattro Group B Spec** | 1244m | 1170m | 1174m | 881m | 342m | 173m | 720m | 139m | 602m | 1300m | 1397m |
| **125cc Shifter Kart Super Sprint** | 654m | 583m | 583m | 402m | 156m | 105m | 323m | 82m | 266m | 708m | 796m |
| **300 BHP Sand Rail Buggy** | 988m | 907m | 911m | 617m | 239m | 133m | 500m | 116m | 416m | 1048m | 1151m |

## 📊 6. Cross-Surface Adhesion & Degradation Index (vs Asphalt 100%)

Mean stopping distance degradation factor relative to baseline dry Asphalt ($1.00\times$):

| Surface Type | Nominal Friction ($\mu$) | Rolling Resistance ($C_{\text{rr}}$) | Mean Stopping Mult | Mean Skidpad Grip Mult |
|:---|:---:|:---:|:---:|:---:|
| **Asphalt** | 1.00 | 1.0x | 1.00x | 0.99x (99%) |
| **Curb** | 0.88 | 1.3x | 1.13x | 0.96x (96%) |
| **Dirt** | 0.78 | 1.2x | 1.27x | 0.94x (94%) |
| **Gravel** | 0.70 | 2.5x | 1.41x | 0.90x (90%) |
| **Mud** | 0.52 | 6.5x | 1.74x | 0.80x (80%) |
| **Grass** | 0.45 | 18.0x | 2.06x | 0.65x (65%) |
| **Snow** | 0.34 | 3.0x | 2.73x | 0.78x (78%) |
| **Sand** | 0.30 | 30.0x | 2.63x | 0.45x (45%) |
| **Water** | 0.22 | 3.5x | 3.91x | 0.70x (70%) |
| **Oil** | 0.12 | 0.8x | 7.32x | 0.53x (53%) |
| **Ice** | 0.08 | 0.4x | 10.39x | 0.36x (36%) |

