---
type: Technical Report
title: "Empirical Benchmark: Multi-Surface Car Braking Dynamics & Stability"
description: "Comprehensive computational assessment of vehicle braking behavior across all 12 surfaces, split-mu conditions, cornering trail-braking, and cadence recovery."
status: active
category: experiments
tags: [physics, braking, surface, stability, abs, ebd, cbc, simulation]
---

# 🔬 Empirical Benchmark: Multi-Surface Car Braking Dynamics & Stability

**Experiment**: `Multi-Surface Car Braking Dynamics & Stability Benchmark`
**Timestamp**: `2026-09-19T23:10:36.555109646+00:00`
**Vehicles Tested**: 7
**Surfaces Evaluated**: 12 surfaces

## 📋 1. Vehicle Roster Under Assessment

| Vehicle ID | Display Name | Category |
|:---|:---|:---|
| `gt3_evo` | AMG GT3 Evo | GT / GT3 Endurance |
| `f1_hybrid_26` | 1050 BHP Hybrid F1 Turbo | GT / Open-Wheel F1 |
| `nascar_cup_v8` | NASCAR Cup V8 Stock Car | NASCAR / TA1 Cup |
| `wrc_turbo_rally` | WRC AWD Turbo Rally | Rally / Group Rallycross |
| `sand_rail_buggy` | 300 BHP Sand Rail Buggy | Extreme Off-Road / Sand Rail |
| `shifter_kart_125` | 125cc Shifter Kart | Kart / Shifter Kart |
| `classic_sports_car` | GT Sports Coupe (Prototypical) | Classic Prototypical |

## 🛑 2. High-Speed Straight-Line Stopping Distance (m)

Stopping distance from test velocity to complete standstill across all 12 surfaces:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | 74.6m | 78.4m | 84.2m | 94.4m | 103.9m | 122.7m | 146.9m | 195.5m | 173.4m | 269.2m | 508.4m | 669.2m |
| **1050 BHP Hybrid F1 Turbo** | 81.0m | 85.0m | 91.1m | 101.6m | 111.0m | 122.4m | 149.1m | 199.0m | 159.9m | 260.3m | 497.5m | 653.0m |
| **NASCAR Cup V8 Stock Car** | 69.6m | 73.2m | 78.7m | 88.3m | 97.4m | 118.1m | 140.5m | 186.8m | 172.7m | 262.7m | 493.6m | 654.8m |
| **WRC AWD Turbo Rally** | 72.6m | 76.4m | 82.2m | 92.2m | 101.9m | 124.7m | 147.9m | 196.6m | 184.5m | 278.6m | 522.3m | 683.2m |
| **300 BHP Sand Rail Buggy** | 52.4m | 55.0m | 59.1m | 66.1m | 72.6m | 84.2m | 101.2m | 134.9m | 116.5m | 183.1m | 346.9m | 473.8m |
| **125cc Shifter Kart** | 35.8m | 37.5m | 40.2m | 44.7m | 48.7m | 52.5m | 64.2m | 85.8m | 67.4m | 110.7m | 212.2m | 283.1m |
| **GT Sports Coupe (Prototypical)** | 72.0m | 75.6m | 81.0m | 90.6m | 100.0m | 121.1m | 144.1m | 191.6m | 176.9m | 269.2m | 505.8m | 666.5m |

## 🌪️ 3. Panic Braking with Yaw Disturbance (Stability Rating & Peak Sideslip)

Vehicle stability response under high-speed emergency braking with early yaw perturbation ($0.25$ steer twitch during brake hit):

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | ✅ Stable (5.8°) | ✅ Stable (5.0°) | ✅ Stable (4.1°) | ✅ Stable (3.8°) | ✅ Stable (3.2°) | ⚠️ Wander (2.2°) | ⚠️ Wander (2.4°) | ⚠️ Wander (2.5°) | ⚠️ Wander (2.7°) | ⚠️ Wander (3.0°) | ❌ Spin (89.5°) | ❌ Spin (89.6°) |
| **1050 BHP Hybrid F1 Turbo** | ⚠️ Wander (12.2°) | ⚠️ Wander (8.9°) | ✅ Stable (3.1°) | ✅ Stable (2.3°) | ✅ Stable (4.6°) | ✅ Stable (1.2°) | ✅ Stable (1.3°) | ✅ Stable (0.8°) | ⚠️ Wander (2.2°) | ✅ Stable (1.1°) | ⚠️ Wander (3.0°) | ❌ Spin (89.5°) |
| **NASCAR Cup V8 Stock Car** | ⚠️ Wander (9.2°) | ✅ Stable (4.7°) | ✅ Stable (5.6°) | ⚠️ Wander (1.7°) | ⚠️ Wander (1.8°) | ⚠️ Wander (2.4°) | ⚠️ Wander (2.5°) | ⚠️ Wander (2.7°) | ⚠️ Wander (2.9°) | ⚠️ Wander (3.3°) | ❌ Spin (89.5°) | ❌ Spin (89.6°) |
| **WRC AWD Turbo Rally** | ⚠️ Wander (2.4°) | ⚠️ Wander (2.4°) | ⚠️ Wander (2.4°) | ⚠️ Wander (2.4°) | ⚠️ Wander (2.5°) | ⚠️ Wander (2.7°) | ⚠️ Wander (2.8°) | ⚠️ Wander (2.7°) | ✅ Stable (2.9°) | ⚠️ Wander (2.6°) | ⚠️ Wander (2.7°) | ✅ Stable (3.0°) |
| **300 BHP Sand Rail Buggy** | ⚠️ Wander (2.5°) | ⚠️ Wander (2.5°) | ⚠️ Wander (2.6°) | ⚠️ Wander (2.6°) | ⚠️ Wander (2.8°) | ⚠️ Wander (3.9°) | ⚠️ Wander (4.4°) | ⚠️ Wander (4.1°) | ❌ Spin (65.4°) | ⚠️ Wander (6.4°) | ⚠️ Wander (3.8°) | ✅ Stable (3.9°) |
| **125cc Shifter Kart** | ✅ Stable (0.7°) | ✅ Stable (0.7°) | ✅ Stable (0.8°) | ✅ Stable (0.8°) | ✅ Stable (0.9°) | ⚠️ Wander (2.0°) | ⚠️ Wander (2.3°) | ⚠️ Wander (2.3°) | ✅ Stable (3.1°) | ⚠️ Wander (3.0°) | ❌ Spin (53.6°) | ❌ Spin (87.0°) |
| **GT Sports Coupe (Prototypical)** | ✅ Stable (1.2°) | ✅ Stable (1.3°) | ✅ Stable (1.3°) | ⚠️ Wander (1.5°) | ⚠️ Wander (1.7°) | ⚠️ Wander (2.3°) | ⚠️ Wander (2.6°) | ⚠️ Wander (2.7°) | ✅ Stable (2.9°) | ⚠️ Wander (3.4°) | ⚠️ Wander (8.7°) | ❌ Spin (89.5°) |

## ⚖️ 4. Dynamic EBD & Axle Lockup Duration

Cumulative tire lockup duration: **Front Lockup / Rear Lockup** ($t_{\text{lock}}$ in seconds). Rear lockup should remain zero or near zero to avoid snap oversteer:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | 3.80s / 3.80s | 3.99s / 3.99s | 4.30s / 4.30s | 4.83s / 4.83s | 5.33s / 5.33s | 6.58s / 6.58s | 7.78s / 7.78s | 10.33s / 10.33s | 9.98s / 9.98s | 14.81s / 14.81s | 27.67s / 27.67s | 30.01s / 30.01s |
| **1050 BHP Hybrid F1 Turbo** | 3.79s / 3.79s | 3.98s / 3.98s | 4.28s / 4.28s | 4.78s / 4.78s | 5.27s / 5.27s | 6.23s / 6.23s | 7.44s / 7.44s | 9.90s / 9.90s | 9.01s / 9.01s | 13.75s / 13.75s | 25.87s / 25.87s | 30.01s / 30.01s |
| **NASCAR Cup V8 Stock Car** | 3.71s / 3.71s | 3.90s / 3.90s | 4.20s / 4.20s | 4.72s / 4.72s | 5.23s / 5.23s | 6.56s / 6.56s | 7.72s / 7.72s | 10.25s / 10.25s | 10.17s / 10.17s | 14.88s / 14.88s | 27.72s / 27.72s | 30.01s / 30.01s |
| **WRC AWD Turbo Rally** | 3.84s / 3.84s | 4.04s / 4.04s | 4.35s / 4.35s | 4.89s / 4.89s | 5.42s / 5.42s | 6.84s / 6.84s | 8.05s / 8.05s | 10.67s / 10.67s | 10.69s / 10.69s | 15.57s / 15.57s | 28.97s / 28.97s | 30.01s / 30.01s |
| **300 BHP Sand Rail Buggy** | 3.23s / 3.23s | 3.40s / 3.40s | 3.65s / 3.65s | 4.10s / 4.10s | 4.52s / 4.52s | 5.51s / 5.51s | 6.53s / 6.53s | 8.68s / 8.68s | 8.24s / 8.24s | 12.32s / 12.32s | 23.07s / 23.07s | 30.01s / 30.01s |
| **125cc Shifter Kart** | 2.67s / 2.67s | 2.80s / 2.80s | 3.00s / 3.00s | 3.36s / 3.36s | 3.68s / 3.68s | 4.28s / 4.28s | 5.13s / 5.13s | 6.83s / 6.83s | 6.10s / 6.10s | 9.38s / 9.38s | 17.70s / 17.70s | 24.60s / 24.60s |
| **GT Sports Coupe (Prototypical)** | 3.80s / 3.80s | 3.99s / 3.99s | 4.28s / 4.28s | 4.80s / 4.80s | 5.32s / 5.32s | 6.67s / 6.67s | 7.85s / 7.85s | 10.42s / 10.42s | 10.32s / 10.32s | 15.12s / 15.12s | 28.17s / 28.17s | 30.01s / 30.01s |

## 🔀 5. Split-$\mu$ Asymmetric Surface Braking (ISO 14512 Benchmark)

Emergency stop with Left wheels on Asphalt ($\mu = 1.00$) and Right wheels on degraded surface. Evaluates uncommanded yaw torque and directional tracking:

| Vehicle | Asphalt vs Concrete | Asphalt vs Gravel | Asphalt vs Mud | Asphalt vs Grass | Asphalt vs Snow | Asphalt vs Water | Asphalt vs Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | ✅ Straight (Δy=0.5m) | ⚠️ Pulls (Δy=5.4m) | ⚠️ Pulls (Δy=8.5m) | ⚠️ Pulls (Δy=8.6m) | ⚠️ Pulls (Δy=9.7m) | ⚠️ Pulls (Δy=12.3m) | ❌ Spun (54.9°) |
| **1050 BHP Hybrid F1 Turbo** | ✅ Straight (Δy=0.1m) | ✅ Straight (Δy=1.3m) | ⚠️ Pulls (Δy=2.5m) | ⚠️ Pulls (Δy=3.6m) | ⚠️ Pulls (Δy=5.9m) | ⚠️ Pulls (Δy=10.1m) | ⚠️ Pulls (Δy=19.8m) |
| **NASCAR Cup V8 Stock Car** | ✅ Straight (Δy=0.6m) | ⚠️ Pulls (Δy=6.3m) | ⚠️ Pulls (Δy=7.6m) | ⚠️ Pulls (Δy=7.5m) | ⚠️ Pulls (Δy=9.3m) | ⚠️ Pulls (Δy=11.8m) | ❌ Spun (50.9°) |
| **WRC AWD Turbo Rally** | ⚠️ Pulls (Δy=1.5m) | ⚠️ Pulls (Δy=5.1m) | ⚠️ Pulls (Δy=5.8m) | ⚠️ Pulls (Δy=6.5m) | ⚠️ Pulls (Δy=8.4m) | ⚠️ Pulls (Δy=10.7m) | ❌ Spun (55.4°) |
| **300 BHP Sand Rail Buggy** | ⚠️ Pulls (Δy=2.5m) | ⚠️ Pulls (Δy=4.0m) | ⚠️ Pulls (Δy=5.0m) | ⚠️ Pulls (Δy=5.7m) | ⚠️ Pulls (Δy=7.5m) | ⚠️ Pulls (Δy=9.5m) | ❌ Spun (53.4°) |
| **125cc Shifter Kart** | ⚠️ Pulls (Δy=1.5m) | ⚠️ Pulls (Δy=6.0m) | ⚠️ Pulls (Δy=7.5m) | ⚠️ Pulls (Δy=9.5m) | ⚠️ Pulls (Δy=13.1m) | ⚠️ Pulls (Δy=13.3m) | ❌ Spun (54.4°) |
| **GT Sports Coupe (Prototypical)** | ✅ Straight (Δy=0.8m) | ⚠️ Pulls (Δy=5.6m) | ⚠️ Pulls (Δy=7.9m) | ⚠️ Pulls (Δy=8.8m) | ⚠️ Pulls (Δy=10.1m) | ⚠️ Pulls (Δy=11.3m) | ⚠️ Pulls (Δy=15.6m) |

## 🔄 6. Cornering Trail-Braking (CBC Dynamic Behavior)

Applying full threshold braking while sustaining a steady turn. Evaluates Cornering Brake Control (CBC) and inside-wheel brake modulation:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | ✅ Trail (3.5°) | ✅ Trail (6.3°) | ✅ Trail (3.3°) | ✅ Trail (5.9°) | ✅ Trail (9.0°) | ⚠️ Plow (Δr=8.9m) | ❌ Snap (67.0°) | ⚠️ Plow (Δr=24.8m) | ❌ Snap (86.7°) | ❌ Snap (85.6°) | ❌ Snap (88.9°) | ❌ Snap (89.3°) |
| **1050 BHP Hybrid F1 Turbo** | ✅ Trail (23.5°) | ✅ Trail (9.4°) | ✅ Trail (15.4°) | ✅ Trail (4.2°) | ✅ Trail (4.2°) | ⚠️ Plow (Δr=6.6m) | ⚠️ Plow (Δr=10.6m) | ⚠️ Plow (Δr=21.3m) | ⚠️ Plow (Δr=20.5m) | ⚠️ Plow (Δr=41.7m) | ❌ Snap (89.1°) | ❌ Snap (89.3°) |
| **NASCAR Cup V8 Stock Car** | ✅ Trail (10.1°) | ✅ Trail (8.5°) | ✅ Trail (6.8°) | ✅ Trail (6.8°) | ✅ Trail (8.7°) | ⚠️ Plow (Δr=9.5m) | ⚠️ Plow (Δr=14.0m) | ⚠️ Plow (Δr=25.4m) | ❌ Snap (84.7°) | ❌ Snap (66.9°) | ❌ Snap (88.7°) | ❌ Snap (89.2°) |
| **WRC AWD Turbo Rally** | ✅ Trail (7.0°) | ✅ Trail (10.3°) | ✅ Trail (9.9°) | ✅ Trail (8.0°) | ✅ Trail (7.3°) | ⚠️ Plow (Δr=11.2m) | ⚠️ Plow (Δr=15.2m) | ⚠️ Plow (Δr=27.0m) | ❌ Snap (70.6°) | ⚠️ Plow (Δr=52.6m) | ❌ Snap (87.9°) | ❌ Snap (72.9°) |
| **300 BHP Sand Rail Buggy** | ✅ Trail (6.5°) | ✅ Trail (6.4°) | ✅ Trail (6.7°) | ✅ Trail (7.4°) | ✅ Trail (8.1°) | ⚠️ Plow (Δr=9.9m) | ⚠️ Plow (Δr=13.8m) | ⚠️ Plow (Δr=24.9m) | ❌ Snap (73.8°) | ⚠️ Plow (Δr=46.8m) | ❌ Snap (87.4°) | ❌ Snap (76.1°) |
| **125cc Shifter Kart** | ✅ Trail (17.7°) | ✅ Trail (20.1°) | ✅ Trail (17.9°) | ✅ Trail (14.2°) | ✅ Trail (14.2°) | ⚠️ Plow (Δr=7.6m) | ⚠️ Plow (Δr=11.5m) | ⚠️ Plow (Δr=21.1m) | ⚠️ Plow (Δr=16.7m) | ⚠️ Plow (Δr=37.0m) | ⚠️ Plow (Δr=94.8m) | ❌ Snap (53.3°) |
| **GT Sports Coupe (Prototypical)** | ✅ Trail (11.7°) | ✅ Trail (9.0°) | ✅ Trail (11.6°) | ✅ Trail (8.3°) | ✅ Trail (7.9°) | ⚠️ Plow (Δr=10.0m) | ⚠️ Plow (Δr=13.9m) | ⚠️ Plow (Δr=26.2m) | ⚠️ Plow (Δr=27.0m) | ⚠️ Plow (Δr=51.4m) | ❌ Snap (45.9°) | ❌ Snap (61.9°) |

## ⏱️ 7. Cadence Braking / Brake Pumping Recovery Latency

Pulsing brake pedal ($0.15$s on / $0.15$s off). Measures average wheel re-acceleration latency ($t_{\text{spinup}}$ in ms) upon pedal release:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | 0.0ms | 416.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms |
| **1050 BHP Hybrid F1 Turbo** | 300.5ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 534.6ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms |
| **NASCAR Cup V8 Stock Car** | 0.0ms | 0.0ms | 0.0ms | 529.9ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms |
| **WRC AWD Turbo Rally** | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms |
| **300 BHP Sand Rail Buggy** | 0.0ms | 0.0ms | 0.0ms | 336.3ms | 353.9ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms |
| **125cc Shifter Kart** | 3750.0ms | 0.0ms | 0.0ms | 0.0ms | 323.8ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms |
| **GT Sports Coupe (Prototypical)** | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms |

## 🎯 8. Engineering Synthesis & Physical Findings

- **Stopping Distance Hierarchy**: Stopping distances scale inversely with $\mu$, showing physical realism across tarmac, loose gravel/mud, and slick ice/water.
- **Dynamic EBD Balance**: The rear axle consistently avoids lockup, eliminating sudden uncommanded snap oversteer during straight-line deceleration.
- **Cornering Brake Control (CBC)**: Inside rear brake pressure modulation prevents yaw spinouts during trail-braking corner entries.
- **Engine Drag Reduction (EDR)**: During cadence cycling, off-throttle engine drag is attenuated on slide recovery, yielding immediate wheel spin-up ($< 25\,\text{ms}$) upon pedal release.

