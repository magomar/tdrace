---
type: Technical Report
title: "Empirical Benchmark: Multi-Surface Automotive Dynamics & Balance"
description: "Empirical 1,800-run simulation benchmark, realism review, and cross-surface balance assessment across 30 vehicles and 12 surfaces."
status: active
category: experiments
tags: [physics, surface, benchmark, telemetry, simulation, balance]
---

# 🔬 Empirical Benchmark Report: Surface-Car Dynamics Simulation

**Experiment**: `Full 5-Tier Multi-Module & Classic Surface Dynamics Simulation`
**Timestamp**: `2026-09-24T16:43:28.309334520+00:00`
**Vehicles Tested**: 29
**Surfaces Evaluated**: 15 surfaces

## 📋 1. Vehicle Roster (1 Vehicle Per Category Across Non-Classic Modules)

| Module | Tier | Category | Vehicle ID | Display Name |
|:---|:---:|:---|:---|:---|
| **GT World Challenge** | Tier 1 | Tier 1: GT4 Clubsport | `gt_porsche_718_gt4` | Porsche 718 Cayman GT4 RS Clubsport |
| **GT World Challenge** | Tier 2 | Tier 2: GT3 Evo / FIA GT3 | `gt_porsche_911_gt3r` | Porsche 911 GT3 R (992) |
| **GT World Challenge** | Tier 3 | Tier 3: GT2 Biturbo | `gt_porsche_911_gt2_rs` | Porsche 911 GT2 RS Clubsport |
| **GT World Challenge** | Tier 4 | Tier 4: GT1 Legend | `gt_porsche_911_gt1_98` | Porsche 911 GT1-98 |
| **GT World Challenge** | Tier 5 | Tier 5: Hypercar Prototype | `gt_ferrari_499p` | Ferrari 499P LMH |
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
| **Classic** | Tier 1 | Classic Prototypical | `classic_sports_car` | GT Sports Coupe (Prototypical) |
| **Classic** | Tier 2 | Classic Prototypical | `classic_drift_car` | Tuned Drift Spec (Prototypical) |
| **Classic** | Tier 3 | Classic Prototypical | `classic_kart` | 125cc Shifter Kart (Prototypical) |
| **Classic** | Tier 2 | Classic Prototypical | `classic_rally_car` | AWD Turbo Rally (Prototypical) |

## 🚀 2. Protocol A: Longitudinal Acceleration & Traction

Elapsed time to 100 km/h ($t_{100}$ in seconds) across surfaces (*DNC* = did not reach 100 km/h):

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Grass | Packed Sand | Deep Sand | Mud Track | Deep Mud | Packed Snow | Deep Snow | Sheet Ice | Water | Oil |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Porsche 718 Cayman GT4 RS Clubsport** | 5.66s | 5.99s | 6.54s | 7.47s | 8.68s | *DNC* (71k) | 11.05s | *DNC* (1k) | 12.16s | *DNC* (65k) | 13.34s | *DNC* (40k) | *DNC* (31k) | *DNC* (66k) | *DNC* (45k) |
| **Porsche 911 GT3 R (992)** | 5.60s | 5.92s | 6.49s | 7.42s | 8.66s | *DNC* (68k) | 11.27s | *DNC* (1k) | 12.59s | *DNC* (58k) | 13.49s | *DNC* (37k) | *DNC* (31k) | *DNC* (63k) | *DNC* (44k) |
| **Porsche 911 GT2 RS Clubsport** | 5.63s | 5.96s | 6.52s | 7.43s | 8.67s | *DNC* (70k) | 11.10s | *DNC* (1k) | 12.27s | *DNC* (63k) | 13.37s | *DNC* (39k) | *DNC* (31k) | *DNC* (65k) | *DNC* (45k) |
| **Porsche 911 GT1-98** | 5.51s | 5.83s | 6.39s | 7.31s | 8.55s | *DNC* (67k) | 11.22s | *DNC* (1k) | 12.62s | *DNC* (56k) | 13.38s | *DNC* (36k) | *DNC* (31k) | *DNC* (62k) | *DNC* (44k) |
| **Ferrari 499P LMH** | 6.47s | 6.85s | 7.50s | 8.58s | 10.05s | *DNC* (58k) | 13.33s | *DNC* (1k) | 15.11s | *DNC* (51k) | 15.81s | *DNC* (32k) | *DNC* (26k) | *DNC* (55k) | *DNC* (38k) |
| **Chevrolet Monte Carlo Street Stock** | 5.37s | 5.69s | 6.23s | 7.15s | 8.33s | *DNC* (81k) | 10.64s | *DNC* (2k) | 11.81s | *DNC* (68k) | 13.01s | *DNC* (46k) | *DNC* (32k) | *DNC* (68k) | *DNC* (46k) |
| **Super Late Model Camaro** | 5.34s | 5.67s | 6.21s | 7.12s | 8.31s | *DNC* (80k) | 10.63s | *DNC* (2k) | 11.82s | *DNC* (67k) | 12.98s | *DNC* (45k) | *DNC* (32k) | *DNC* (67k) | *DNC* (46k) |
| **Chevrolet SS ARCA Spec** | 5.31s | 5.63s | 6.17s | 7.07s | 8.22s | *DNC* (84k) | 10.39s | *DNC* (2k) | 11.46s | *DNC* (72k) | 12.75s | *DNC* (47k) | *DNC* (32k) | *DNC* (69k) | *DNC* (47k) |
| **Chevrolet Silverado RST Truck** | 5.32s | 5.64s | 6.18s | 7.09s | 8.25s | *DNC* (83k) | 10.47s | *DNC* (2k) | 11.58s | *DNC* (70k) | 12.83s | *DNC* (46k) | *DNC* (32k) | *DNC* (69k) | *DNC* (46k) |
| **Chevrolet Corvette C7 TA1** | 5.28s | 5.61s | 6.14s | 7.05s | 8.21s | *DNC* (82k) | 10.45s | *DNC* (2k) | 11.57s | *DNC* (69k) | 12.78s | *DNC* (46k) | *DNC* (32k) | *DNC* (68k) | *DNC* (46k) |
| **Peugeot 208 Rally4** | 8.93s | 8.95s | 9.07s | 9.05s | 9.71s | *DNC* (58k) | 11.79s | *DNC* (2k) | 12.47s | *DNC* (71k) | 13.42s | *DNC* (48k) | *DNC* (52k) | *DNC* (70k) | *DNC* (49k) |
| **Hyundai i20 RX Supercar** | 5.44s | 5.45s | 5.50s | 5.49s | 5.72s | 11.49s | 6.20s | *DNC* (95k) | 6.23s | 9.18s | 6.09s | 11.89s | 14.64s | 14.12s | *DNC* (98k) |
| **Audi Sport Quattro S1 E2** | 3.38s | 3.46s | 3.58s | 3.76s | 4.05s | 6.60s | 4.68s | 12.31s | 5.08s | 8.63s | 6.00s | 12.67s | 10.32s | 14.55s | *DNC* (97k) |
| **Toyota GR DKR Hilux T1+** | 7.50s | 7.52s | 7.60s | 7.57s | 8.02s | *DNC* (98k) | 8.38s | 22.17s | 8.47s | 11.81s | 7.95s | 15.41s | 12.15s | 13.97s | *DNC* (99k) |
| **Stadium Super Truck V8** | 5.33s | 5.69s | 6.30s | 7.31s | 8.68s | *DNC* (53k) | 10.78s | *DNC* (26k) | 12.22s | *DNC* (73k) | 14.10s | *DNC* (27k) | 20.37s | *DNC* (58k) | *DNC* (42k) |
| **Can-Am Maverick R Trophy Spec** | 3.54s | 3.81s | 4.27s | 5.06s | 6.02s | *DNC* (85k) | 7.80s | *DNC* (54k) | 9.50s | *DNC* (65k) | 10.36s | *DNC* (48k) | *DNC* (56k) | *DNC* (70k) | *DNC* (54k) |
| **Geiser Bros AWD Trophy Truck** | 5.38s | 5.48s | 5.67s | 5.91s | 6.34s | 12.99s | 6.72s | 13.09s | 6.99s | 9.69s | 7.15s | 13.08s | 14.66s | 13.89s | *DNC* (99k) |
| **Subaru WRX STI Ice Racer** | 4.03s | 4.03s | 4.05s | 4.18s | 4.45s | 6.92s | 4.88s | 11.62s | 5.08s | 8.31s | 6.02s | 12.10s | 4.45s | 14.28s | *DNC* (98k) |
| **Mega Truck V8 Mud Slinger** | 6.54s | 6.56s | 6.63s | 6.70s | 7.27s | 24.42s | 7.91s | 23.87s | 7.94s | 10.74s | 8.42s | 18.19s | 18.55s | 13.93s | *DNC* (99k) |
| **Grave Digger Spec Monster Jam** | 7.57s | 7.59s | 7.69s | 7.66s | 8.18s | *DNC* (62k) | 8.22s | *DNC* (89k) | 8.17s | 13.45s | 9.07s | *DNC* (74k) | 19.22s | *DNC* (94k) | *DNC* (59k) |
| **CRG Hero 60cc Cadet** | *DNC* (79k) | *DNC* (79k) | *DNC* (79k) | *DNC* (79k) | *DNC* (77k) | *DNC* (17k) | *DNC* (58k) | *DNC* (1k) | *DNC* (51k) | *DNC* (18k) | *DNC* (65k) | *DNC* (11k) | *DNC* (20k) | *DNC* (31k) | *DNC* (32k) |
| **Tony Kart Racer 401 RR OK** | 6.68s | 7.27s | 8.56s | 11.28s | *DNC* (95k) | *DNC* (25k) | *DNC* (65k) | *DNC* (1k) | *DNC* (58k) | *DNC* (20k) | *DNC* (73k) | *DNC* (13k) | *DNC* (20k) | *DNC* (34k) | *DNC* (34k) |
| **Birel ART KZ2 125cc Shifter** | 6.37s | 6.88s | 7.95s | 9.99s | 16.99s | *DNC* (26k) | *DNC* (70k) | *DNC* (1k) | *DNC* (62k) | *DNC* (22k) | *DNC* (78k) | *DNC* (13k) | *DNC* (21k) | *DNC* (36k) | *DNC* (35k) |
| **Honda Mean Mower V2 Tuned** | 7.04s | 7.69s | 9.23s | 12.99s | *DNC* (92k) | *DNC* (25k) | *DNC* (63k) | *DNC* (1k) | *DNC* (56k) | *DNC* (20k) | *DNC* (71k) | *DNC* (12k) | *DNC* (20k) | *DNC* (33k) | *DNC* (33k) |
| **Anderson CS250 Twin GP** | 5.13s | 5.48s | 6.12s | 7.22s | 9.05s | *DNC* (30k) | *DNC* (96k) | *DNC* (1k) | *DNC* (82k) | *DNC* (26k) | 18.66s | *DNC* (15k) | *DNC* (22k) | *DNC* (42k) | *DNC* (38k) |
| **GT Sports Coupe (Prototypical)** | 5.41s | 5.76s | 6.35s | 7.33s | 8.66s | *DNC* (54k) | 11.42s | *DNC* (1k) | 12.76s | *DNC* (52k) | 13.77s | *DNC* (28k) | *DNC* (29k) | *DNC* (60k) | *DNC* (42k) |
| **Tuned Drift Spec (Prototypical)** | 5.44s | 5.79s | 6.39s | 7.38s | 8.72s | *DNC* (53k) | 11.57s | *DNC* (1k) | 12.98s | *DNC* (51k) | 13.92s | *DNC* (27k) | *DNC* (29k) | *DNC* (59k) | *DNC* (42k) |
| **125cc Shifter Kart (Prototypical)** | 5.82s | 6.24s | 7.02s | 8.36s | 10.72s | *DNC* (31k) | *DNC* (88k) | *DNC* (1k) | *DNC* (78k) | *DNC* (27k) | *DNC* (95k) | *DNC* (15k) | *DNC* (22k) | *DNC* (43k) | *DNC* (38k) |
| **AWD Turbo Rally (Prototypical)** | 4.03s | 4.04s | 4.07s | 4.17s | 4.44s | 6.89s | 4.83s | 11.24s | 5.03s | 8.11s | 5.98s | 11.77s | 10.28s | 14.04s | *DNC* (99k) |

## 🛑 3. Protocol B: Emergency Braking Distance (100 → 0 km/h)

Stopping distance ($d_{\text{stop}}$ in meters) from an initial speed of 100 km/h:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Grass | Packed Sand | Deep Sand | Mud Track | Deep Mud | Packed Snow | Deep Snow | Sheet Ice | Water | Oil |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Porsche 718 Cayman GT4 RS Clubsport** | 38.0m | 40.0m | 43.1m | 48.5m | 53.8m | 80.9m | 59.7m | 109.6m | 63.2m | 84.1m | 77.6m | 119.0m | 424.5m | 158.4m | 294.0m |
| **Porsche 911 GT3 R (992)** | 36.7m | 38.6m | 41.6m | 46.7m | 51.8m | 76.6m | 56.9m | 99.7m | 59.9m | 77.2m | 74.1m | 108.8m | 392.9m | 147.2m | 274.6m |
| **Porsche 911 GT2 RS Clubsport** | 37.5m | 39.5m | 42.5m | 47.8m | 53.1m | 79.3m | 58.7m | 106.0m | 62.0m | 81.6m | 76.3m | 115.3m | 413.4m | 154.4m | 287.0m |
| **Porsche 911 GT1-98** | 35.6m | 37.4m | 40.3m | 45.3m | 50.1m | 73.8m | 54.9m | 94.8m | 57.8m | 73.6m | 71.6m | 103.6m | 375.4m | 140.9m | 263.4m |
| **Ferrari 499P LMH** | 41.1m | 43.2m | 46.5m | 52.2m | 57.8m | 84.5m | 63.1m | 107.1m | 66.3m | 83.4m | 82.4m | 117.2m | 420.7m | 160.4m | 300.2m |
| **Chevrolet Monte Carlo Street Stock** | 38.3m | 40.3m | 43.4m | 48.9m | 54.2m | 80.6m | 59.8m | 106.7m | 63.1m | 82.3m | 77.7m | 116.2m | 417.1m | 156.2m | 290.8m |
| **Super Late Model Camaro** | 38.0m | 39.9m | 43.0m | 48.4m | 53.6m | 79.7m | 59.1m | 104.9m | 62.3m | 81.0m | 76.9m | 114.3m | 411.1m | 153.9m | 286.8m |
| **Chevrolet SS ARCA Spec** | 38.1m | 40.1m | 43.2m | 48.7m | 54.0m | 81.0m | 59.8m | 109.2m | 63.2m | 84.0m | 77.7m | 118.7m | 423.9m | 158.3m | 294.0m |
| **Chevrolet Silverado RST Truck** | 38.0m | 40.0m | 43.1m | 48.5m | 53.8m | 80.5m | 59.5m | 107.5m | 62.9m | 82.8m | 77.4m | 116.9m | 418.8m | 156.5m | 291.1m |
| **Chevrolet Corvette C7 TA1** | 37.6m | 39.5m | 42.6m | 47.9m | 53.1m | 79.2m | 58.6m | 105.2m | 61.9m | 81.1m | 76.3m | 114.6m | 411.3m | 153.7m | 286.0m |
| **Peugeot 208 Rally4** | 38.5m | 40.5m | 43.5m | 48.8m | 54.0m | 80.2m | 59.5m | 105.4m | 62.7m | 81.5m | 77.5m | 114.9m | 290.2m | 154.9m | 288.6m |
| **Hyundai i20 RX Supercar** | 37.8m | 39.7m | 42.6m | 47.9m | 53.1m | 79.1m | 58.6m | 104.4m | 61.8m | 80.6m | 76.2m | 113.7m | 179.0m | 153.0m | 284.9m |
| **Audi Sport Quattro S1 E2** | 37.2m | 39.0m | 41.9m | 47.1m | 52.1m | 76.6m | 57.1m | 98.3m | 60.0m | 76.3m | 74.4m | 107.5m | 126.9m | 146.2m | 273.2m |
| **Toyota GR DKR Hilux T1+** | 38.7m | 40.6m | 43.7m | 49.1m | 54.5m | 81.8m | 60.4m | 110.3m | 63.8m | 84.8m | 78.5m | 119.9m | 155.5m | 159.8m | 296.8m |
| **Stadium Super Truck V8** | 38.4m | 40.3m | 43.4m | 48.7m | 53.9m | 79.5m | 59.2m | 102.4m | 62.2m | 79.5m | 77.1m | 111.9m | 115.9m | 152.0m | 283.9m |
| **Can-Am Maverick R Trophy Spec** | 37.3m | 39.2m | 42.1m | 47.1m | 51.8m | 72.9m | 55.4m | 85.4m | 57.6m | 67.5m | 72.7m | 94.3m | 254.4m | 133.1m | 251.6m |
| **Geiser Bros AWD Trophy Truck** | 38.6m | 40.6m | 43.8m | 49.3m | 54.7m | 82.3m | 60.7m | 111.8m | 64.2m | 85.8m | 78.8m | 121.4m | 186.1m | 161.3m | 299.4m |
| **Subaru WRX STI Ice Racer** | 37.9m | 39.8m | 42.8m | 48.1m | 53.3m | 79.1m | 58.7m | 103.7m | 61.8m | 80.2m | 76.4m | 113.1m | 57.7m | 152.5m | 284.2m |
| **Mega Truck V8 Mud Slinger** | 38.7m | 40.7m | 43.9m | 49.4m | 54.8m | 82.5m | 60.8m | 111.8m | 64.4m | 85.8m | 79.0m | 121.4m | 230.3m | 161.5m | 299.8m |
| **Grave Digger Spec Monster Jam** | 38.9m | 41.0m | 44.2m | 49.8m | 55.4m | 84.4m | 61.9m | 119.0m | 65.7m | 90.5m | 80.2m | 128.6m | 159.6m | 168.2m | 310.8m |
| **CRG Hero 60cc Cadet** | 32.8m | 34.3m | 36.3m | 39.9m | 42.5m | 49.8m | 40.8m | 44.8m | 40.4m | 36.9m | 55.1m | 50.6m | 218.1m | 78.8m | 154.0m |
| **Tony Kart Racer 401 RR OK** | 33.7m | 35.2m | 37.5m | 41.3m | 44.4m | 53.9m | 43.6m | 50.7m | 43.5m | 41.5m | 58.5m | 57.1m | 246.1m | 87.5m | 170.1m |
| **Birel ART KZ2 125cc Shifter** | 34.0m | 35.7m | 38.0m | 42.0m | 45.3m | 56.1m | 45.0m | 54.1m | 45.2m | 44.2m | 60.2m | 60.8m | 262.2m | 92.3m | 178.9m |
| **Honda Mean Mower V2 Tuned** | 34.1m | 35.7m | 37.9m | 41.8m | 44.7m | 53.7m | 43.6m | 49.8m | 43.4m | 40.9m | 58.7m | 56.2m | 242.1m | 86.5m | 168.4m |
| **Anderson CS250 Twin GP** | 30.7m | 32.2m | 34.4m | 38.4m | 41.9m | 56.2m | 43.6m | 60.3m | 44.7m | 48.4m | 57.7m | 67.2m | 290.5m | 98.1m | 187.5m |
| **GT Sports Coupe (Prototypical)** | 38.0m | 39.9m | 42.9m | 48.1m | 53.3m | 79.5m | 58.8m | 105.4m | 62.1m | 81.3m | 76.5m | 114.8m | 412.1m | 154.1m | 286.7m |
| **Tuned Drift Spec (Prototypical)** | 37.9m | 39.9m | 42.9m | 48.3m | 53.5m | 79.5m | 59.0m | 104.6m | 62.2m | 80.8m | 76.7m | 114.0m | 410.2m | 153.6m | 286.1m |
| **125cc Shifter Kart (Prototypical)** | 35.8m | 37.5m | 40.2m | 44.7m | 48.7m | 64.2m | 50.2m | 67.4m | 51.2m | 54.3m | 66.5m | 75.2m | 325.0m | 110.7m | 212.2m |
| **AWD Turbo Rally (Prototypical)** | 37.9m | 39.8m | 42.8m | 48.0m | 53.2m | 79.3m | 58.7m | 105.2m | 62.0m | 81.2m | 76.4m | 114.6m | 130.5m | 153.8m | 286.3m |

## 🔄 4. Protocol C: Steady-State Skidpad Cornering Limit ($R = 30\text{m}$)

Peak lateral acceleration ($a_{y,\max}$ in $g$) on constant-radius skidpad:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Grass | Packed Sand | Deep Sand | Mud Track | Deep Mud | Packed Snow | Deep Snow | Sheet Ice | Water | Oil |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Porsche 718 Cayman GT4 RS Clubsport** | 1.00g | 0.95g | 0.87g | 0.77g | 0.69g | 0.40g | 0.60g | 0.09g | 0.56g | 0.37g | 0.46g | 0.24g | 0.06g | 0.19g | 0.09g |
| **Porsche 911 GT3 R (992)** | 1.02g | 0.97g | 0.89g | 0.79g | 0.70g | 0.31g | 0.60g | 0.09g | 0.56g | 0.27g | 0.47g | 0.17g | 0.06g | 0.17g | 0.08g |
| **Porsche 911 GT2 RS Clubsport** | 1.01g | 0.95g | 0.88g | 0.78g | 0.69g | 0.31g | 0.59g | 0.09g | 0.55g | 0.27g | 0.46g | 0.16g | 0.05g | 0.17g | 0.08g |
| **Porsche 911 GT1-98** | 1.05g | 0.99g | 0.91g | 0.80g | 0.71g | 0.26g | 0.61g | 0.09g | 0.56g | 0.23g | 0.47g | 0.12g | 0.05g | 0.16g | 0.08g |
| **Ferrari 499P LMH** | 0.89g | 0.84g | 0.78g | 0.68g | 0.60g | 0.25g | 0.51g | 0.09g | 0.48g | 0.22g | 0.40g | 0.13g | 0.04g | 0.13g | 0.07g |
| **Chevrolet Monte Carlo Street Stock** | 0.98g | 0.93g | 0.86g | 0.76g | 0.68g | 0.34g | 0.59g | 0.09g | 0.55g | 0.30g | 0.46g | 0.18g | 0.05g | 0.17g | 0.08g |
| **Super Late Model Camaro** | 0.99g | 0.94g | 0.87g | 0.77g | 0.68g | 0.32g | 0.59g | 0.09g | 0.55g | 0.28g | 0.46g | 0.17g | 0.05g | 0.17g | 0.08g |
| **Chevrolet SS ARCA Spec** | 0.99g | 0.94g | 0.87g | 0.77g | 0.68g | 0.32g | 0.59g | 0.09g | 0.55g | 0.28g | 0.46g | 0.17g | 0.05g | 0.17g | 0.08g |
| **Chevrolet Silverado RST Truck** | 0.99g | 0.94g | 0.87g | 0.77g | 0.68g | 0.32g | 0.59g | 0.09g | 0.55g | 0.28g | 0.46g | 0.17g | 0.05g | 0.17g | 0.08g |
| **Chevrolet Corvette C7 TA1** | 1.00g | 0.95g | 0.88g | 0.77g | 0.69g | 0.32g | 0.59g | 0.09g | 0.55g | 0.28g | 0.46g | 0.16g | 0.05g | 0.17g | 0.07g |
| **Peugeot 208 Rally4** | 0.97g | 0.92g | 0.85g | 0.75g | 0.67g | 0.35g | 0.60g | 0.08g | 0.56g | 0.32g | 0.46g | 0.21g | 0.10g | 0.19g | 0.10g |
| **Hyundai i20 RX Supercar** | 0.98g | 0.93g | 0.86g | 0.76g | 0.68g | 0.42g | 0.60g | 0.28g | 0.56g | 0.38g | 0.47g | 0.27g | 0.19g | 0.21g | 0.11g |
| **Audi Sport Quattro S1 E2** | 0.99g | 0.94g | 0.87g | 0.77g | 0.68g | 0.43g | 0.61g | 0.28g | 0.57g | 0.38g | 0.47g | 0.27g | 0.27g | 0.21g | 0.11g |
| **Toyota GR DKR Hilux T1+** | 0.96g | 0.92g | 0.85g | 0.75g | 0.67g | 0.42g | 0.60g | 0.28g | 0.56g | 0.38g | 0.47g | 0.27g | 0.23g | 0.21g | 0.11g |
| **Stadium Super Truck V8** | 0.96g | 0.91g | 0.84g | 0.74g | 0.65g | 0.31g | 0.57g | 0.18g | 0.53g | 0.35g | 0.44g | 0.17g | 0.29g | 0.17g | 0.08g |
| **Can-Am Maverick R Trophy Spec** | 0.91g | 0.86g | 0.80g | 0.71g | 0.63g | 0.30g | 0.56g | 0.19g | 0.52g | 0.30g | 0.44g | 0.16g | 0.08g | 0.17g | 0.08g |
| **Geiser Bros AWD Trophy Truck** | 0.92g | 0.88g | 0.81g | 0.72g | 0.65g | 0.41g | 0.58g | 0.28g | 0.54g | 0.38g | 0.45g | 0.26g | 0.19g | 0.21g | 0.10g |
| **Subaru WRX STI Ice Racer** | 0.97g | 0.92g | 0.86g | 0.76g | 0.68g | 0.42g | 0.60g | 0.28g | 0.56g | 0.38g | 0.47g | 0.27g | 0.63g | 0.21g | 0.11g |
| **Mega Truck V8 Mud Slinger** | 0.92g | 0.87g | 0.81g | 0.72g | 0.65g | 0.41g | 0.58g | 0.28g | 0.54g | 0.38g | 0.45g | 0.26g | 0.15g | 0.21g | 0.10g |
| **Grave Digger Spec Monster Jam** | 0.91g | 0.87g | 0.81g | 0.72g | 0.64g | 0.29g | 0.57g | 0.24g | 0.53g | 0.36g | 0.44g | 0.21g | 0.22g | 0.18g | 0.09g |
| **CRG Hero 60cc Cadet** | 0.95g | 0.90g | 0.84g | 0.76g | 0.68g | 0.09g | 0.51g | 0.08g | 0.42g | 0.08g | 0.46g | 0.05g | 0.04g | 0.17g | 0.10g |
| **Tony Kart Racer 401 RR OK** | 0.96g | 0.91g | 0.85g | 0.75g | 0.67g | 0.15g | 0.60g | 0.09g | 0.52g | 0.10g | 0.44g | 0.05g | 0.04g | 0.17g | 0.08g |
| **Birel ART KZ2 125cc Shifter** | 0.97g | 0.92g | 0.85g | 0.75g | 0.65g | 0.17g | 0.57g | 0.08g | 0.53g | 0.11g | 0.43g | 0.05g | 0.04g | 0.16g | 0.08g |
| **Honda Mean Mower V2 Tuned** | 0.95g | 0.90g | 0.83g | 0.73g | 0.64g | 0.14g | 0.57g | 0.10g | 0.50g | 0.09g | 0.42g | 0.05g | 0.04g | 0.17g | 0.08g |
| **Anderson CS250 Twin GP** | 1.14g | 1.08g | 0.98g | 0.85g | 0.74g | 0.31g | 0.60g | 0.05g | 0.54g | 0.16g | 0.47g | 0.07g | 0.04g | 0.15g | 0.07g |
| **GT Sports Coupe (Prototypical)** | 0.97g | 0.91g | 0.84g | 0.74g | 0.66g | 0.32g | 0.57g | 0.15g | 0.53g | 0.28g | 0.44g | 0.18g | 0.05g | 0.17g | 0.08g |
| **Tuned Drift Spec (Prototypical)** | 0.96g | 0.91g | 0.84g | 0.74g | 0.65g | 0.28g | 0.55g | 0.15g | 0.51g | 0.26g | 0.43g | 0.16g | 0.05g | 0.15g | 0.07g |
| **125cc Shifter Kart (Prototypical)** | 0.96g | 0.91g | 0.84g | 0.74g | 0.65g | 0.27g | 0.54g | 0.08g | 0.49g | 0.18g | 0.43g | 0.08g | 0.04g | 0.15g | 0.07g |
| **AWD Turbo Rally (Prototypical)** | 0.96g | 0.92g | 0.85g | 0.75g | 0.67g | 0.42g | 0.60g | 0.28g | 0.56g | 0.38g | 0.46g | 0.26g | 0.27g | 0.21g | 0.11g |

## 🔀 5. Protocol D: Transient Step-Steer & Slalom Stability (80 km/h)

Dynamic stability classification and recovery status under rapid lateral excitation:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Grass | Packed Sand | Deep Sand | Mud Track | Deep Mud | Packed Snow | Deep Snow | Sheet Ice | Water | Oil |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Porsche 718 Cayman GT4 RS Clubsport** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift |
| **Porsche 911 GT3 R (992)** | Stable | Stable | Stable | Stable | Stable | Drift | Stable | Drift | Stable | Drift | Stable | Drift | Drift | Drift | Drift |
| **Porsche 911 GT2 RS Clubsport** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Stable | Drift | Stable | Drift | Drift | Drift | Drift |
| **Porsche 911 GT1-98** | Spun | Spun | Spun | Spun | Spun | Spun | Spun | Spun | Spun | Spun | Spun | Spun | Spun | Spun | Spun |
| **Ferrari 499P LMH** | Stable | Stable | Stable | Stable | Stable | Drift | Drift | Drift | Drift | Drift | Stable | Drift | Drift | Drift | Drift |
| **Chevrolet Monte Carlo Street Stock** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Drift | Drift | Stable | Drift | Stable | Drift | Drift |
| **Super Late Model Camaro** | Stable | Stable | Stable | Stable | Stable | Drift | Drift | Drift | Drift | Drift | Stable | Drift | Drift | Drift | Drift |
| **Chevrolet SS ARCA Spec** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Drift | Drift | Stable | Drift | Stable | Drift | Drift |
| **Chevrolet Silverado RST Truck** | Stable | Stable | Stable | Stable | Stable | Drift | Drift | Drift | Drift | Drift | Stable | Drift | Stable | Drift | Drift |
| **Chevrolet Corvette C7 TA1** | Stable | Stable | Stable | Stable | Stable | Drift | Drift | Drift | Drift | Drift | Drift | Drift | Drift | Drift | Drift |
| **Peugeot 208 Rally4** | Stable | Stable | Stable | Stable | Stable | Drift | Stable | Drift | Stable | Stable | Stable | Stable | Stable | Stable | Stable |
| **Hyundai i20 RX Supercar** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable |
| **Audi Sport Quattro S1 E2** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable |
| **Toyota GR DKR Hilux T1+** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable |
| **Stadium Super Truck V8** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Stable | Drift | Stable | Drift | Stable | Drift | Drift |
| **Can-Am Maverick R Trophy Spec** | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Drift | Drift | Drift | Drift | Drift | Drift | Drift | Drift |
| **Geiser Bros AWD Trophy Truck** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Stable | Drift | Drift |
| **Subaru WRX STI Ice Racer** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable |
| **Mega Truck V8 Mud Slinger** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Stable | Drift | Stable |
| **Grave Digger Spec Monster Jam** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Stable | Stable | Stable | Stable | Drift | Drift | Drift |
| **CRG Hero 60cc Cadet** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift |
| **Tony Kart Racer 401 RR OK** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Drift | Drift | Drift | Drift | Drift |
| **Birel ART KZ2 125cc Shifter** | Stable | Stable | Stable | Stable | Stable | Drift | Drift | Stable | Drift | Drift | Drift | Drift | Stable | Drift | Drift |
| **Honda Mean Mower V2 Tuned** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Drift | Drift | Drift | Drift | Drift | Drift |
| **Anderson CS250 Twin GP** | Stable | Stable | Stable | Stable | Stable | Drift | Stable | Stable | Drift | Drift | Stable | Drift | Drift | Drift | Drift |
| **GT Sports Coupe (Prototypical)** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Drift | Stable | Drift | Stable |
| **Tuned Drift Spec (Prototypical)** | Stable | Stable | Stable | Drift | Drift | Drift | Drift | Stable | Drift | Drift | Drift | Drift | Stable | Drift | Drift |
| **125cc Shifter Kart (Prototypical)** | Stable | Stable | Stable | Stable | Stable | Drift | Stable | Stable | Drift | Drift | Drift | Drift | Drift | Drift | Drift |
| **AWD Turbo Rally (Prototypical)** | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable | Stable |

## 🍃 6. Protocol E: Passive Coast-Down Distance (120 → 0 km/h)

Distance rolled ($d_{\text{coast}}$ in meters) under purely aerodynamic and rolling resistance:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Grass | Packed Sand | Deep Sand | Mud Track | Deep Mud | Packed Snow | Deep Snow | Sheet Ice | Water | Oil |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Porsche 718 Cayman GT4 RS Clubsport** | 1355m | 1345m | 1283m | 1289m | 1001m | 191m | 525m | 148m | 513m | 202m | 1028m | 242m | 1503m | 702m | 1409m |
| **Porsche 911 GT3 R (992)** | 1208m | 1199m | 1137m | 1139m | 860m | 174m | 448m | 131m | 432m | 173m | 879m | 210m | 1355m | 586m | 1262m |
| **Porsche 911 GT2 RS Clubsport** | 1304m | 1294m | 1232m | 1236m | 951m | 185m | 497m | 142m | 483m | 191m | 975m | 230m | 1452m | 659m | 1358m |
| **Porsche 911 GT1-98** | 1148m | 1138m | 1076m | 1078m | 800m | 165m | 417m | 123m | 400m | 162m | 818m | 196m | 1296m | 542m | 1202m |
| **Ferrari 499P LMH** | 1182m | 1174m | 1117m | 1117m | 871m | 186m | 462m | 138m | 442m | 179m | 880m | 218m | 1316m | 596m | 1232m |
| **Chevrolet Monte Carlo Street Stock** | 1303m | 1294m | 1236m | 1238m | 978m | 200m | 520m | 143m | 502m | 201m | 994m | 243m | 1439m | 682m | 1353m |
| **Super Late Model Camaro** | 1280m | 1272m | 1213m | 1215m | 955m | 196m | 507m | 140m | 488m | 196m | 970m | 237m | 1417m | 662m | 1331m |
| **Chevrolet SS ARCA Spec** | 1359m | 1350m | 1291m | 1295m | 1028m | 204m | 549m | 147m | 534m | 211m | 1048m | 255m | 1497m | 729m | 1410m |
| **Chevrolet Silverado RST Truck** | 1328m | 1319m | 1260m | 1263m | 999m | 201m | 532m | 144m | 515m | 205m | 1017m | 248m | 1466m | 702m | 1379m |
| **Chevrolet Corvette C7 TA1** | 1300m | 1291m | 1232m | 1235m | 970m | 196m | 514m | 140m | 497m | 198m | 987m | 240m | 1439m | 676m | 1351m |
| **Peugeot 208 Rally4** | 1256m | 1247m | 1182m | 1187m | 892m | 175m | 463m | 141m | 460m | 184m | 917m | 215m | 1407m | 609m | 1312m |
| **Hyundai i20 RX Supercar** | 1256m | 1246m | 1181m | 1186m | 887m | 172m | 517m | 139m | 498m | 204m | 913m | 213m | 1410m | 606m | 1312m |
| **Audi Sport Quattro S1 E2** | 1162m | 1152m | 1088m | 1091m | 800m | 163m | 495m | 129m | 470m | 199m | 821m | 195m | 1314m | 542m | 1218m |
| **Toyota GR DKR Hilux T1+** | 1340m | 1329m | 1264m | 1271m | 968m | 182m | 728m | 188m | 683m | 297m | 999m | 231m | 1493m | 672m | 1395m |
| **Stadium Super Truck V8** | 1201m | 1192m | 1129m | 1131m | 843m | 171m | 641m | 183m | 587m | 263m | 864m | 206m | 1350m | 572m | 1256m |
| **Can-Am Maverick R Trophy Spec** | 905m | 895m | 827m | 829m | 562m | 128m | 452m | 150m | 332m | 144m | 568m | 142m | 1062m | 376m | 964m |
| **Geiser Bros AWD Trophy Truck** | 1321m | 1309m | 1233m | 1246m | 884m | 156m | 734m | 197m | 640m | 277m | 932m | 205m | 1500m | 608m | 1385m |
| **Subaru WRX STI Ice Racer** | 1240m | 1230m | 1165m | 1170m | 874m | 172m | 493m | 138m | 475m | 193m | 899m | 211m | 1393m | 596m | 1296m |
| **Mega Truck V8 Mud Slinger** | 1319m | 1307m | 1231m | 1244m | 883m | 157m | 651m | 161m | 803m | 388m | 930m | 205m | 1497m | 607m | 1383m |
| **Grave Digger Spec Monster Jam** | 1447m | 1433m | 1356m | 1374m | 996m | 164m | 930m | 249m | 942m | 446m | 1057m | 225m | 1630m | 696m | 1511m |
| **CRG Hero 60cc Cadet** | 373m | 367m | 333m | 330m | 239m | 74m | 132m | 52m | 119m | 53m | 232m | 66m | 487m | 156m | 412m |
| **Tony Kart Racer 401 RR OK** | 440m | 433m | 392m | 389m | 278m | 83m | 153m | 60m | 139m | 61m | 271m | 76m | 566m | 183m | 486m |
| **Birel ART KZ2 125cc Shifter** | 482m | 474m | 428m | 426m | 302m | 87m | 165m | 64m | 151m | 66m | 295m | 82m | 612m | 199m | 531m |
| **Honda Mean Mower V2 Tuned** | 427m | 421m | 381m | 378m | 272m | 82m | 149m | 59m | 136m | 60m | 264m | 75m | 551m | 178m | 472m |
| **Anderson CS250 Twin GP** | 596m | 586m | 526m | 527m | 359m | 90m | 193m | 73m | 180m | 76m | 357m | 94m | 743m | 239m | 651m |
| **GT Sports Coupe (Prototypical)** | 1272m | 1262m | 1196m | 1202m | 901m | 174m | 466m | 141m | 453m | 179m | 929m | 216m | 1426m | 618m | 1328m |
| **Tuned Drift Spec (Prototypical)** | 1252m | 1242m | 1177m | 1182m | 885m | 173m | 459m | 139m | 444m | 177m | 910m | 213m | 1404m | 604m | 1307m |
| **125cc Shifter Kart (Prototypical)** | 654m | 645m | 583m | 583m | 402m | 105m | 217m | 82m | 201m | 86m | 397m | 106m | 796m | 266m | 708m |
| **AWD Turbo Rally (Prototypical)** | 1271m | 1261m | 1195m | 1201m | 900m | 173m | 562m | 140m | 539m | 225m | 928m | 216m | 1426m | 617m | 1327m |

## 📊 7. Cross-Surface Adhesion & Degradation Index (vs Asphalt 100%)

Mean stopping distance degradation factor relative to baseline dry Asphalt ($1.00\times$):

| Surface Type | Nominal Friction ($\mu$) | Rolling Resistance ($C_{\text{rr}}$) | Mean Stopping Mult | Mean Skidpad Grip Mult |
|:---|:---:|:---:|:---:|:---:|
| **Asphalt** | 1.00 | 1.0x | 1.00x | 1.00x (100%) |
| **Concrete** | 0.95 | 1.0x | 1.05x | 0.95x (95%) |
| **Curb** | 0.88 | 1.3x | 1.13x | 0.88x (88%) |
| **Dirt** | 0.78 | 1.2x | 1.26x | 0.78x (78%) |
| **Gravel** | 0.70 | 2.5x | 1.39x | 0.69x (69%) |
| **Grass** | 0.45 | 18.0x | 2.01x | 0.32x (32%) |
| **Packed Sand** | 0.62 | 5.2x | 1.51x | 0.60x (60%) |
| **Deep Sand** | 0.30 | 30.0x | 2.53x | 0.16x (16%) |
| **Mud Track** | 0.58 | 5.0x | 1.58x | 0.55x (55%) |
| **Deep Mud** | 0.40 | 14.0x | 1.97x | 0.29x (29%) |
| **Packed Snow** | 0.48 | 2.2x | 1.97x | 0.46x (46%) |
| **Deep Snow** | 0.28 | 12.0x | 2.77x | 0.18x (18%) |
| **Sheet Ice** | 0.08 | 0.4x | 7.82x | 0.12x (12%) |
| **Water** | 0.22 | 3.5x | 3.78x | 0.18x (18%) |
| **Oil** | 0.12 | 0.8x | 7.09x | 0.09x (9%) |

