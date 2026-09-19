---
type: Physics Reference
title: "Empirical Benchmark Report: Surface-Car Dynamics Simulation"
description: "Empirical benchmark report for full 5-tier multi-module and classic surface dynamics simulation across 11 surface types."
status: active
category: experiments
tags: [physics, surface, benchmark, telemetry, simulation]
---

# 🔬 Empirical Benchmark Report: Surface-Car Dynamics Simulation

**Experiment**: `Full 5-Tier Multi-Module & Classic Surface Dynamics Simulation`
**Timestamp**: `2026-09-19T17:56:14.481608323+00:00`
**Vehicles Tested**: 30
**Surfaces Evaluated**: 11 surfaces

## 📋 1. Vehicle Roster (1 Vehicle Per Category Across Non-Classic Modules)

| Module | Tier | Category | Vehicle ID | Display Name |
|:---|:---:|:---|:---|:---|
| **GT / F1** | Tier 1 | Tier 1: GT4 Clubsport | `gt_porsche_718_gt4` | Porsche 718 Cayman GT4 RS Clubsport |
| **GT / F1** | Tier 2 | Tier 2: GT3 Evo / FIA GT3 | `gt_porsche_911_gt3r` | Porsche 911 GT3 R (992) |
| **GT / F1** | Tier 3 | Tier 3: GT2 Biturbo | `gt_porsche_911_gt2_rs` | Porsche 911 GT2 RS Clubsport |
| **GT / F1** | Tier 4 | Tier 4: GT1 Legend | `gt_porsche_911_gt1_98` | Porsche 911 GT1-98 |
| **GT / F1** | Tier 5 | Tier 5: Hypercar Prototype | `gt_ferrari_499p` | Ferrari 499P LMH |
| **NASCAR** | Tier 1 | Tier 1: Street Stock V8 | `nascar_monte_carlo_ss` | Chevrolet Monte Carlo Street Stock |
| **NASCAR** | Tier 2 | Tier 2: Late Model Stock | `nascar_super_late_model` | Super Late Model Camaro |
| **NASCAR** | Tier 3 | Tier 3: ARCA Menards Series | `nascar_arca_chevy_ss` | Chevrolet SS ARCA Spec |
| **NASCAR** | Tier 4 | Tier 4: Craftsman Truck | `nascar_silverado_truck` | Chevrolet Silverado RST Truck |
| **NASCAR** | Tier 5 | Tier 5: Trans-Am TA1 | `nascar_corvette_ta1` | Chevrolet Corvette C7 TA1 |
| **Rally** | Tier 1 | Tier 1: Rally Junior FWD | `rally_peugeot_208_rally4` | Peugeot 208 Rally4 |
| **Rally** | Tier 2 | Tier 2: WRC / RX Supercar | `rally_hyundai_i20_rx` | Hyundai i20 RX Supercar |
| **Rally** | Tier 3 | Tier 3: Group B Beast | `rally_audi_sport_quattro_s1` | Audi Sport Quattro S1 E2 |
| **Rally** | Tier 4 | Tier 4: Rally Raid T1+ | `rally_toyota_hilux_t1_plus` | Toyota GR DKR Hilux T1+ |
| **Rally** | Tier 5 | Tier 5: Stadium Super Truck | `rally_sst_super_truck` | Stadium Super Truck V8 |
| **Extreme Off-Road** | Tier 1 | Tier 1: Sand Rail Buggy | `offroad_sand_rail_buggy` | Can-Am Maverick R Trophy Spec |
| **Extreme Off-Road** | Tier 2 | Tier 2: Trophy Truck 4x4 | `offroad_baja_trophy_truck` | Geiser Bros AWD Trophy Truck |
| **Extreme Off-Road** | Tier 3 | Tier 3: Arctic Ice Racer | `offroad_subaru_ice_racer` | Subaru WRX STI Ice Racer |
| **Extreme Off-Road** | Tier 4 | Tier 4: Mud Bogger V8 | `offroad_mega_mud_truck` | Mega Truck V8 Mud Slinger |
| **Extreme Off-Road** | Tier 5 | Tier 5: Crusher Monster Truck | `offroad_grave_crusher` | Grave Digger Spec Monster Jam |
| **Kart** | Tier 1 | Tier 1: Cadet Kart 60cc | `kart_crg_hero_60` | CRG Hero 60cc Cadet |
| **Kart** | Tier 2 | Tier 2: Senior Kart 100cc OK | `kart_tony_kart_racer_ok` | Tony Kart Racer 401 RR OK |
| **Kart** | Tier 3 | Tier 3: Shifter Kart 125cc KZ2 | `kart_birel_art_kz2` | Birel ART KZ2 125cc Shifter |
| **Kart** | Tier 4 | Tier 4: Racing Lawnmower | `kart_honda_mean_mower` | Honda Mean Mower V2 Tuned |
| **Kart** | Tier 5 | Tier 5: Superkart 250cc GP | `kart_anderson_cs250` | Anderson CS250 Twin GP |
| **GT / F1** | Tier 5 | Tier 5: Open-Wheel F1 | `f1_hybrid_26` | 1050 BHP Hybrid F1 Turbo |
| **Classic** | Tier 1 | Classic Prototypical | `classic_sports_car` | GT Sports Coupe (Prototypical) |
| **Classic** | Tier 2 | Classic Prototypical | `classic_drift_car` | Tuned Drift Spec (Prototypical) |
| **Classic** | Tier 3 | Classic Prototypical | `classic_kart` | 125cc Shifter Kart (Prototypical) |
| **Classic** | Tier 2 | Classic Prototypical | `classic_rally_car` | AWD Turbo Rally (Prototypical) |

## 🚀 2. Protocol A: Longitudinal Acceleration & Traction

Elapsed time to 100 km/h ($t_{100}$ in seconds) across surfaces (*DNC* = did not reach 100 km/h):

| Vehicle | Asphalt | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Porsche 718 Cayman GT4 RS Clubsport** | 5.66s | 6.55s | 7.48s | 8.72s | 16.14s | *DNC* (70k) | 21.87s | *DNC* (1k) | *DNC* (65k) | *DNC* (45k) | *DNC* (31k) |
| **Porsche 911 GT3 R (992)** | 5.55s | 6.43s | 7.35s | 8.59s | 16.86s | *DNC* (67k) | 22.18s | *DNC* (1k) | *DNC* (63k) | *DNC* (44k) | *DNC* (31k) |
| **Porsche 911 GT2 RS Clubsport** | 5.62s | 6.50s | 7.42s | 8.65s | 16.14s | *DNC* (69k) | 21.80s | *DNC* (1k) | *DNC* (64k) | *DNC* (45k) | *DNC* (31k) |
| **Porsche 911 GT1-98** | 5.44s | 6.31s | 7.22s | 8.45s | 17.47s | *DNC* (66k) | 22.35s | *DNC* (1k) | *DNC* (62k) | *DNC* (44k) | *DNC* (31k) |
| **Ferrari 499P LMH** | 6.40s | 7.42s | 8.51s | 9.99s | 23.15s | *DNC* (57k) | *DNC* (94k) | *DNC* (1k) | *DNC* (54k) | *DNC* (38k) | *DNC* (26k) |
| **Chevrolet Monte Carlo Street Stock** | 5.35s | 6.11s | 7.00s | 8.14s | 14.45s | *DNC* (84k) | 20.24s | *DNC* (2k) | *DNC* (69k) | *DNC* (47k) | *DNC* (32k) |
| **Super Late Model Camaro** | 5.25s | 6.11s | 7.01s | 8.17s | 14.96s | *DNC* (81k) | 20.64s | *DNC* (2k) | *DNC* (68k) | *DNC* (46k) | *DNC* (32k) |
| **Chevrolet SS ARCA Spec** | 5.26s | 6.11s | 7.00s | 8.14s | 14.45s | *DNC* (84k) | 20.24s | *DNC* (2k) | *DNC* (69k) | *DNC* (47k) | *DNC* (32k) |
| **Chevrolet Silverado RST Truck** | 5.26s | 6.11s | 7.00s | 8.14s | 14.37s | *DNC* (84k) | 20.17s | *DNC* (2k) | *DNC* (69k) | *DNC* (47k) | *DNC* (32k) |
| **Chevrolet Corvette C7 TA1** | 5.25s | 6.11s | 7.01s | 8.16s | 14.85s | *DNC* (82k) | 20.56s | *DNC* (2k) | *DNC* (68k) | *DNC* (46k) | *DNC* (32k) |
| **Peugeot 208 Rally4** | 8.91s | 9.05s | 9.02s | 9.68s | 15.28s | *DNC* (58k) | 20.26s | *DNC* (2k) | *DNC* (72k) | *DNC* (50k) | *DNC* (34k) |
| **Hyundai i20 RX Supercar** | 5.43s | 5.48s | 5.47s | 5.69s | 6.77s | 11.19s | 8.57s | *DNC* (65k) | 13.84s | *DNC* (100k) | *DNC* (68k) |
| **Audi Sport Quattro S1 E2** | 3.38s | 3.58s | 3.75s | 4.05s | 5.72s | 6.52s | 8.60s | 11.15s | 13.99s | *DNC* (99k) | *DNC* (67k) |
| **Toyota GR DKR Hilux T1+** | 7.44s | 7.53s | 7.51s | 7.93s | 9.92s | 24.29s | 8.48s | *DNC* (3k) | 13.44s | 24.47s | *DNC* (69k) |
| **Stadium Super Truck V8** | 5.22s | 6.14s | 7.11s | 8.38s | 15.86s | *DNC* (56k) | 22.10s | *DNC* (1k) | *DNC* (62k) | *DNC* (43k) | *DNC* (30k) |
| **Can-Am Maverick R Trophy Spec** | 3.48s | 4.18s | 4.94s | 5.86s | 12.91s | *DNC* (93k) | 17.30s | *DNC* (41k) | *DNC* (75k) | *DNC* (55k) | *DNC* (38k) |
| **Geiser Bros AWD Trophy Truck** | 5.35s | 5.64s | 5.87s | 6.30s | 7.98s | 12.69s | 8.52s | *DNC* (45k) | 13.49s | 24.56s | *DNC* (69k) |
| **Subaru WRX STI Ice Racer** | 4.01s | 4.04s | 4.16s | 4.43s | 5.68s | 6.82s | 8.57s | 12.57s | 13.86s | *DNC* (100k) | *DNC* (68k) |
| **Mega Truck V8 Mud Slinger** | 6.49s | 6.57s | 6.64s | 7.19s | 9.47s | 22.74s | 9.66s | *DNC* (2k) | 13.39s | 24.41s | *DNC* (69k) |
| **Grave Digger Spec Monster Jam** | 7.53s | 7.64s | 7.61s | 8.12s | 10.40s | *DNC* (64k) | 14.02s | *DNC* (2k) | *DNC* (99k) | *DNC* (60k) | *DNC* (41k) |
| **CRG Hero 60cc Cadet** | *DNC* (81k) | *DNC* (80k) | *DNC* (80k) | *DNC* (79k) | *DNC* (49k) | *DNC* (19k) | *DNC* (57k) | *DNC* (1k) | *DNC* (36k) | *DNC* (35k) | *DNC* (26k) |
| **Tony Kart Racer 401 RR OK** | 6.02s | 7.33s | 8.87s | 11.99s | *DNC* (56k) | *DNC* (29k) | *DNC* (64k) | *DNC* (1k) | *DNC* (40k) | *DNC* (36k) | *DNC* (27k) |
| **Birel ART KZ2 125cc Shifter** | 5.85s | 7.05s | 8.41s | 10.84s | *DNC* (61k) | *DNC* (31k) | *DNC* (69k) | *DNC* (1k) | *DNC* (42k) | *DNC* (38k) | *DNC* (27k) |
| **Honda Mean Mower V2 Tuned** | 5.76s | 6.91s | 8.17s | 10.32s | *DNC* (65k) | *DNC* (32k) | *DNC* (72k) | *DNC* (1k) | *DNC* (44k) | *DNC* (38k) | *DNC* (28k) |
| **Anderson CS250 Twin GP** | 5.70s | 6.82s | 8.03s | 10.04s | *DNC* (68k) | *DNC* (33k) | *DNC* (75k) | *DNC* (1k) | *DNC* (45k) | *DNC* (39k) | *DNC* (28k) |
| **1050 BHP Hybrid F1 Turbo** | 5.73s | 6.65s | 7.61s | 8.94s | *DNC* (99k) | *DNC* (63k) | *DNC* (99k) | *DNC* (2k) | *DNC* (57k) | *DNC* (42k) | *DNC* (29k) |
| **GT Sports Coupe (Prototypical)** | 5.41s | 6.35s | 7.33s | 8.66s | 17.21s | *DNC* (54k) | 23.16s | *DNC* (1k) | *DNC* (60k) | *DNC* (42k) | *DNC* (29k) |
| **Tuned Drift Spec (Prototypical)** | 5.44s | 6.39s | 7.38s | 8.72s | 17.77s | *DNC* (53k) | 23.62s | *DNC* (1k) | *DNC* (59k) | *DNC* (42k) | *DNC* (29k) |
| **125cc Shifter Kart (Prototypical)** | 5.82s | 7.02s | 8.36s | 10.72s | *DNC* (62k) | *DNC* (31k) | *DNC* (70k) | *DNC* (1k) | *DNC* (43k) | *DNC* (38k) | *DNC* (27k) |
| **AWD Turbo Rally (Prototypical)** | 4.03s | 4.07s | 4.17s | 4.44s | 5.73s | 6.89s | 8.62s | 13.47s | 14.04s | *DNC* (99k) | *DNC* (67k) |

## 🛑 3. Protocol B: Emergency Braking Distance (100 → 0 km/h)

Stopping distance ($d_{\text{stop}}$ in meters) from an initial speed of 100 km/h:

| Vehicle | Asphalt | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Porsche 718 Cayman GT4 RS Clubsport** | 37.7m | 42.7m | 48.1m | 53.3m | 67.6m | 79.5m | 105.5m | 105.6m | 154.2m | 286.9m | 412.6m |
| **Porsche 911 GT3 R (992)** | 36.1m | 40.9m | 46.0m | 51.0m | 63.9m | 75.3m | 100.0m | 97.8m | 144.6m | 269.9m | 385.9m |
| **Porsche 911 GT2 RS Clubsport** | 37.2m | 42.1m | 47.4m | 52.5m | 66.5m | 78.2m | 103.8m | 103.5m | 151.5m | 282.1m | 405.5m |
| **Porsche 911 GT1-98** | 34.8m | 39.4m | 44.2m | 48.9m | 60.7m | 71.8m | 95.4m | 91.7m | 136.8m | 255.8m | 363.9m |
| **Ferrari 499P LMH** | 40.0m | 45.3m | 50.9m | 56.2m | 68.7m | 81.6m | 108.4m | 101.8m | 153.7m | 288.2m | 403.8m |
| **Chevrolet Monte Carlo Street Stock** | 37.4m | 42.4m | 47.8m | 53.0m | 67.6m | 79.3m | 105.3m | 106.5m | 154.7m | 287.5m | 414.6m |
| **Super Late Model Camaro** | 37.1m | 42.1m | 47.3m | 52.4m | 66.4m | 78.1m | 103.6m | 103.1m | 151.1m | 281.3m | 404.1m |
| **Chevrolet SS ARCA Spec** | 37.4m | 42.4m | 47.8m | 53.0m | 67.6m | 79.3m | 105.3m | 106.5m | 154.7m | 287.5m | 414.6m |
| **Chevrolet Silverado RST Truck** | 37.5m | 42.5m | 47.8m | 53.1m | 67.8m | 79.6m | 105.6m | 107.1m | 155.4m | 288.6m | 416.5m |
| **Chevrolet Corvette C7 TA1** | 37.2m | 42.1m | 47.4m | 52.6m | 66.6m | 78.3m | 103.9m | 103.8m | 151.8m | 282.5m | 406.2m |
| **Peugeot 208 Rally4** | 37.7m | 42.7m | 48.1m | 53.3m | 67.7m | 79.5m | 105.5m | 105.8m | 154.4m | 287.3m | 413.2m |
| **Hyundai i20 RX Supercar** | 37.9m | 42.9m | 48.3m | 53.6m | 68.6m | 80.4m | 106.7m | 108.3m | 157.0m | 291.7m | 420.8m |
| **Audi Sport Quattro S1 E2** | 37.7m | 42.7m | 48.1m | 53.3m | 67.7m | 79.6m | 105.6m | 106.0m | 154.6m | 287.6m | 413.7m |
| **Toyota GR DKR Hilux T1+** | 46.2m | 48.0m | 49.9m | 54.5m | 71.0m | 82.9m | 109.9m | 115.7m | 164.4m | 304.1m | 441.6m |
| **Stadium Super Truck V8** | 39.6m | 43.1m | 48.5m | 53.8m | 69.1m | 80.9m | 107.4m | 109.8m | 158.6m | 294.3m | 425.1m |
| **Can-Am Maverick R Trophy Spec** | 37.2m | 42.0m | 47.1m | 52.0m | 62.5m | 74.5m | 99.1m | 90.5m | 138.6m | 260.7m | 364.8m |
| **Geiser Bros AWD Trophy Truck** | 50.8m | 50.5m | 52.8m | 55.0m | 71.7m | 83.6m | 110.9m | 117.0m | 166.1m | 307.2m | 445.8m |
| **Subaru WRX STI Ice Racer** | 37.9m | 42.9m | 48.3m | 53.6m | 68.5m | 80.3m | 106.6m | 108.0m | 156.7m | 291.2m | 419.9m |
| **Mega Truck V8 Mud Slinger** | 55.9m | 55.5m | 55.7m | 56.8m | 72.2m | 84.1m | 111.5m | 118.8m | 167.8m | 309.9m | 450.4m |
| **Grave Digger Spec Monster Jam** | 79.0m | 78.2m | 78.4m | 74.8m | 75.4m | 85.6m | 113.4m | 124.2m | 172.8m | 318.1m | 464.2m |
| **CRG Hero 60cc Cadet** | 34.0m | 37.9m | 41.9m | 45.1m | 44.9m | 55.9m | 74.9m | 53.7m | 91.8m | 178.0m | 231.0m |
| **Tony Kart Racer 401 RR OK** | 35.1m | 39.3m | 43.6m | 47.2m | 49.2m | 60.7m | 81.2m | 61.3m | 102.5m | 197.4m | 260.2m |
| **Birel ART KZ2 125cc Shifter** | 35.7m | 40.0m | 44.6m | 48.5m | 52.1m | 63.8m | 85.2m | 66.6m | 109.7m | 210.3m | 280.2m |
| **Honda Mean Mower V2 Tuned** | 36.1m | 40.5m | 45.2m | 49.3m | 54.0m | 65.8m | 87.9m | 70.4m | 114.6m | 219.2m | 294.2m |
| **Anderson CS250 Twin GP** | 36.3m | 40.9m | 45.6m | 49.8m | 55.3m | 67.2m | 89.7m | 73.1m | 118.1m | 225.3m | 304.1m |
| **1050 BHP Hybrid F1 Turbo** | 32.9m | 37.2m | 41.7m | 46.0m | 55.9m | 66.5m | 88.4m | 81.8m | 124.4m | 233.8m | 328.5m |
| **GT Sports Coupe (Prototypical)** | 37.7m | 42.7m | 48.1m | 53.3m | 67.6m | 79.5m | 105.5m | 105.4m | 154.1m | 286.7m | 412.1m |
| **Tuned Drift Spec (Prototypical)** | 37.9m | 42.9m | 48.3m | 53.5m | 67.6m | 79.5m | 105.6m | 104.6m | 153.6m | 286.1m | 410.2m |
| **125cc Shifter Kart (Prototypical)** | 35.8m | 40.2m | 44.7m | 48.7m | 52.5m | 64.2m | 85.8m | 67.4m | 110.7m | 212.2m | 283.1m |
| **AWD Turbo Rally (Prototypical)** | 37.6m | 42.7m | 48.0m | 53.2m | 67.5m | 79.3m | 105.3m | 105.2m | 153.8m | 286.3m | 411.5m |

## 🔄 4. Protocol C: Steady-State Skidpad Cornering Limit ($R = 30\text{m}$)

Peak lateral acceleration ($a_{y,\max}$ in $g$) on constant-radius skidpad:

| Vehicle | Asphalt | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Porsche 718 Cayman GT4 RS Clubsport** | 0.16g | 0.16g | 0.16g | 0.16g | 0.15g | 0.12g | 0.16g | 0.08g | 0.15g | 0.11g | 0.07g |
| **Porsche 911 GT3 R (992)** | 0.16g | 0.16g | 0.16g | 0.16g | 0.15g | 0.12g | 0.16g | 0.09g | 0.15g | 0.11g | 0.07g |
| **Porsche 911 GT2 RS Clubsport** | 0.16g | 0.16g | 0.16g | 0.16g | 0.15g | 0.13g | 0.16g | 0.09g | 0.15g | 0.11g | 0.07g |
| **Porsche 911 GT1-98** | 0.16g | 0.16g | 0.16g | 0.15g | 0.15g | 0.12g | 0.15g | 0.09g | 0.15g | 0.11g | 0.07g |
| **Ferrari 499P LMH** | 0.16g | 0.16g | 0.16g | 0.16g | 0.15g | 0.13g | 0.16g | 0.10g | 0.15g | 0.09g | 0.06g |
| **Chevrolet Monte Carlo Street Stock** | 0.14g | 0.14g | 0.14g | 0.14g | 0.13g | 0.10g | 0.13g | 0.07g | 0.13g | 0.11g | 0.07g |
| **Super Late Model Camaro** | 0.14g | 0.14g | 0.14g | 0.14g | 0.13g | 0.11g | 0.14g | 0.08g | 0.13g | 0.11g | 0.07g |
| **Chevrolet SS ARCA Spec** | 0.14g | 0.14g | 0.14g | 0.14g | 0.13g | 0.11g | 0.14g | 0.08g | 0.13g | 0.11g | 0.07g |
| **Chevrolet Silverado RST Truck** | 0.14g | 0.14g | 0.14g | 0.14g | 0.13g | 0.11g | 0.14g | 0.08g | 0.13g | 0.11g | 0.07g |
| **Chevrolet Corvette C7 TA1** | 0.14g | 0.14g | 0.14g | 0.14g | 0.13g | 0.12g | 0.14g | 0.10g | 0.13g | 0.11g | 0.07g |
| **Peugeot 208 Rally4** | 0.47g | 0.45g | 0.44g | 0.40g | 0.33g | 0.25g | 0.29g | 0.11g | 0.20g | 0.11g | 0.07g |
| **Hyundai i20 RX Supercar** | 0.49g | 0.47g | 0.46g | 0.42g | 0.34g | 0.27g | 0.28g | 0.18g | 0.20g | 0.12g | 0.08g |
| **Audi Sport Quattro S1 E2** | 0.50g | 0.48g | 0.46g | 0.43g | 0.35g | 0.29g | 0.28g | 0.21g | 0.20g | 0.11g | 0.08g |
| **Toyota GR DKR Hilux T1+** | 0.51g | 0.49g | 0.48g | 0.44g | 0.35g | 0.25g | 0.30g | 0.06g | 0.20g | 0.12g | 0.08g |
| **Stadium Super Truck V8** | 0.50g | 0.48g | 0.46g | 0.43g | 0.33g | 0.22g | 0.26g | 0.08g | 0.17g | 0.10g | 0.07g |
| **Can-Am Maverick R Trophy Spec** | 0.60g | 0.56g | 0.51g | 0.46g | 0.32g | 0.19g | 0.25g | 0.06g | 0.16g | 0.10g | 0.07g |
| **Geiser Bros AWD Trophy Truck** | 0.68g | 0.65g | 0.62g | 0.58g | 0.45g | 0.37g | 0.31g | 0.26g | 0.21g | 0.12g | 0.08g |
| **Subaru WRX STI Ice Racer** | 0.50g | 0.48g | 0.46g | 0.43g | 0.35g | 0.29g | 0.29g | 0.21g | 0.20g | 0.12g | 0.08g |
| **Mega Truck V8 Mud Slinger** | 0.67g | 0.65g | 0.61g | 0.57g | 0.44g | 0.36g | 0.31g | 0.06g | 0.21g | 0.12g | 0.08g |
| **Grave Digger Spec Monster Jam** | 0.66g | 0.62g | 0.58g | 0.51g | 0.35g | 0.17g | 0.27g | 0.05g | 0.18g | 0.11g | 0.07g |
| **CRG Hero 60cc Cadet** | 0.63g | 0.54g | 0.47g | 0.41g | 0.28g | 0.16g | 0.22g | 0.05g | 0.14g | 0.09g | 0.06g |
| **Tony Kart Racer 401 RR OK** | 0.69g | 0.59g | 0.51g | 0.45g | 0.32g | 0.20g | 0.24g | 0.07g | 0.15g | 0.10g | 0.06g |
| **Birel ART KZ2 125cc Shifter** | 0.72g | 0.61g | 0.53g | 0.47g | 0.34g | 0.22g | 0.25g | 0.07g | 0.16g | 0.10g | 0.06g |
| **Honda Mean Mower V2 Tuned** | 0.75g | 0.63g | 0.54g | 0.48g | 0.34g | 0.22g | 0.25g | 0.08g | 0.16g | 0.10g | 0.07g |
| **Anderson CS250 Twin GP** | 0.74g | 0.63g | 0.55g | 0.49g | 0.36g | 0.23g | 0.26g | 0.08g | 0.16g | 0.10g | 0.07g |
| **1050 BHP Hybrid F1 Turbo** | 0.09g | 0.09g | 0.09g | 0.09g | 0.08g | 0.08g | 0.08g | 0.05g | 0.07g | 0.07g | 0.06g |
| **GT Sports Coupe (Prototypical)** | 0.49g | 0.46g | 0.45g | 0.41g | 0.32g | 0.22g | 0.26g | 0.08g | 0.17g | 0.10g | 0.07g |
| **Tuned Drift Spec (Prototypical)** | 0.61g | 0.56g | 0.52g | 0.47g | 0.34g | 0.22g | 0.26g | 0.08g | 0.17g | 0.10g | 0.07g |
| **125cc Shifter Kart (Prototypical)** | 0.72g | 0.61g | 0.53g | 0.47g | 0.34g | 0.22g | 0.25g | 0.08g | 0.16g | 0.10g | 0.06g |
| **AWD Turbo Rally (Prototypical)** | 0.49g | 0.47g | 0.45g | 0.42g | 0.34g | 0.28g | 0.28g | 0.20g | 0.20g | 0.11g | 0.08g |

## 🍃 5. Protocol E: Passive Coast-Down Distance (120 → 0 km/h)

Distance rolled ($d_{\text{coast}}$ in meters) under purely aerodynamic and rolling resistance:

| Vehicle | Asphalt | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Porsche 718 Cayman GT4 RS Clubsport** | 1289m | 1218m | 1222m | 939m | 368m | 185m | 776m | 141m | 649m | 1343m | 1436m |
| **Porsche 911 GT3 R (992)** | 1188m | 1116m | 1119m | 839m | 325m | 170m | 684m | 128m | 570m | 1243m | 1337m |
| **Porsche 911 GT2 RS Clubsport** | 1267m | 1195m | 1199m | 916m | 358m | 180m | 754m | 138m | 630m | 1322m | 1416m |
| **Porsche 911 GT1-98** | 1112m | 1040m | 1042m | 764m | 295m | 159m | 620m | 118m | 516m | 1167m | 1261m |
| **Ferrari 499P LMH** | 1120m | 1056m | 1054m | 811m | 316m | 177m | 661m | 130m | 550m | 1170m | 1254m |
| **Chevrolet Monte Carlo Street Stock** | 1330m | 1261m | 1265m | 996m | 397m | 198m | 837m | 143m | 700m | 1381m | 1470m |
| **Super Late Model Camaro** | 1274m | 1206m | 1208m | 943m | 372m | 191m | 783m | 137m | 654m | 1326m | 1414m |
| **Chevrolet SS ARCA Spec** | 1330m | 1261m | 1265m | 996m | 397m | 198m | 837m | 143m | 700m | 1381m | 1470m |
| **Chevrolet Silverado RST Truck** | 1340m | 1271m | 1276m | 1006m | 402m | 199m | 847m | 144m | 709m | 1392m | 1481m |
| **Chevrolet Corvette C7 TA1** | 1285m | 1216m | 1219m | 953m | 377m | 193m | 793m | 138m | 662m | 1336m | 1425m |
| **Peugeot 208 Rally4** | 1280m | 1204m | 1210m | 909m | 353m | 174m | 745m | 141m | 623m | 1336m | 1435m |
| **Hyundai i20 RX Supercar** | 1322m | 1246m | 1253m | 948m | 370m | 178m | 782m | 146m | 655m | 1379m | 1478m |
| **Audi Sport Quattro S1 E2** | 1283m | 1207m | 1213m | 911m | 354m | 174m | 748m | 142m | 626m | 1339m | 1438m |
| **Toyota GR DKR Hilux T1+** | 1448m | 1371m | 1382m | 1065m | 424m | 188m | 901m | 159m | 759m | 1505m | 1607m |
| **Stadium Super Truck V8** | 1347m | 1271m | 1278m | 971m | 380m | 180m | 804m | 148m | 675m | 1404m | 1504m |
| **Can-Am Maverick R Trophy Spec** | 988m | 907m | 911m | 617m | 239m | 133m | 500m | 116m | 416m | 1048m | 1151m |
| **Geiser Bros AWD Trophy Truck** | 1418m | 1327m | 1344m | 968m | 375m | 161m | 798m | 161m | 674m | 1483m | 1602m |
| **Subaru WRX STI Ice Racer** | 1318m | 1241m | 1248m | 943m | 368m | 177m | 778m | 145m | 652m | 1374m | 1473m |
| **Mega Truck V8 Mud Slinger** | 1450m | 1358m | 1377m | 996m | 386m | 163m | 825m | 165m | 697m | 1514m | 1634m |
| **Grave Digger Spec Monster Jam** | 1551m | 1458m | 1481m | 1087m | 426m | 168m | 915m | 175m | 778m | 1615m | 1739m |
| **CRG Hero 60cc Cadet** | 477m | 424m | 422m | 300m | 116m | 87m | 240m | 64m | 197m | 526m | 608m |
| **Tony Kart Racer 401 RR OK** | 573m | 508m | 508m | 354m | 137m | 97m | 284m | 74m | 234m | 626m | 710m |
| **Birel ART KZ2 125cc Shifter** | 643m | 573m | 573m | 395m | 153m | 104m | 317m | 81m | 262m | 697m | 784m |
| **Honda Mean Mower V2 Tuned** | 694m | 621m | 622m | 426m | 165m | 109m | 342m | 87m | 282m | 749m | 838m |
| **Anderson CS250 Twin GP** | 730m | 657m | 658m | 448m | 174m | 113m | 361m | 90m | 298m | 786m | 877m |
| **1050 BHP Hybrid F1 Turbo** | 996m | 934m | 931m | 696m | 270m | 159m | 562m | 103m | 466m | 1046m | 1127m |
| **GT Sports Coupe (Prototypical)** | 1272m | 1196m | 1202m | 901m | 350m | 174m | 739m | 141m | 618m | 1328m | 1426m |
| **Tuned Drift Spec (Prototypical)** | 1252m | 1177m | 1182m | 885m | 343m | 173m | 723m | 139m | 604m | 1307m | 1404m |
| **125cc Shifter Kart (Prototypical)** | 654m | 583m | 583m | 402m | 156m | 105m | 323m | 82m | 266m | 708m | 796m |
| **AWD Turbo Rally (Prototypical)** | 1271m | 1195m | 1201m | 900m | 350m | 173m | 738m | 140m | 617m | 1327m | 1426m |

## 📊 6. Cross-Surface Adhesion & Degradation Index (vs Asphalt 100%)

Mean stopping distance degradation factor relative to baseline dry Asphalt ($1.00\times$):

| Surface Type | Nominal Friction ($\mu$) | Rolling Resistance ($C_{\text{rr}}$) | Mean Stopping Mult | Mean Skidpad Grip Mult |
|:---|:---:|:---:|:---:|:---:|
| **Asphalt** | 1.00 | 1.0x | 1.00x | 1.00x (100%) |
| **Curb** | 0.88 | 1.3x | 1.11x | 0.94x (94%) |
| **Dirt** | 0.78 | 1.2x | 1.23x | 0.90x (90%) |
| **Gravel** | 0.70 | 2.5x | 1.35x | 0.85x (85%) |
| **Mud** | 0.52 | 6.5x | 1.65x | 0.71x (71%) |
| **Grass** | 0.45 | 18.0x | 1.95x | 0.55x (55%) |
| **Snow** | 0.34 | 3.0x | 2.59x | 0.64x (64%) |
| **Sand** | 0.30 | 30.0x | 2.48x | 0.33x (33%) |
| **Water** | 0.22 | 3.5x | 3.70x | 0.53x (53%) |
| **Oil** | 0.12 | 0.8x | 6.92x | 0.38x (38%) |
| **Ice** | 0.08 | 0.4x | 9.81x | 0.26x (26%) |

