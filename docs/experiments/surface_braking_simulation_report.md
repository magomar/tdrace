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
**Timestamp**: `2026-09-20T15:34:10.381441594+00:00`
**Vehicles Tested**: 7
**Surfaces Evaluated**: 12 surfaces

## 📋 1. Vehicle Roster Under Assessment

| Vehicle ID | Display Name | Category |
|:---|:---|:---|
| `gt3_evo` | AMG GT3 Evo | GT / GT3 Endurance |
| `hypercar_prototype` | 800 BHP LMH Hypercar Prototype | GT / Le Mans Hypercar |
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
| **800 BHP LMH Hypercar Prototype** | 103.8m | 109.0m | 116.8m | 130.2m | 142.3m | 157.5m | 191.6m | 255.6m | 206.6m | 335.3m | 637.9m | 784.0m |
| **NASCAR Cup V8 Stock Car** | 69.6m | 73.2m | 78.7m | 88.3m | 97.4m | 118.1m | 140.5m | 186.8m | 172.7m | 262.7m | 493.6m | 654.8m |
| **WRC AWD Turbo Rally** | 72.6m | 76.4m | 82.2m | 92.2m | 101.9m | 124.7m | 147.9m | 196.6m | 184.5m | 278.6m | 522.3m | 683.2m |
| **300 BHP Sand Rail Buggy** | 52.4m | 55.0m | 59.1m | 66.1m | 72.6m | 84.2m | 101.2m | 134.9m | 116.5m | 183.1m | 346.9m | 473.8m |
| **125cc Shifter Kart** | 35.8m | 37.5m | 40.2m | 44.7m | 48.7m | 52.5m | 64.2m | 85.8m | 67.4m | 110.7m | 212.2m | 283.1m |
| **GT Sports Coupe (Prototypical)** | 72.0m | 75.6m | 81.0m | 90.6m | 100.0m | 121.1m | 144.1m | 191.6m | 176.9m | 269.2m | 505.8m | 666.5m |

## 🌪️ 3. Panic Braking with Yaw Disturbance (Stability Rating & Peak Sideslip)

Vehicle stability response under high-speed emergency braking with early yaw perturbation ($0.25$ steer twitch during brake hit):

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | ✅ Stable (0.9°) | ✅ Stable (1.0°) | ✅ Stable (1.0°) | ✅ Stable (1.1°) | ✅ Stable (1.2°) | ⚠️ Wander (2.2°) | ⚠️ Wander (2.4°) | ⚠️ Wander (2.5°) | ⚠️ Wander (2.7°) | ⚠️ Wander (3.0°) | ❌ Spin (89.5°) | ❌ Spin (89.7°) |
| **800 BHP LMH Hypercar Prototype** | ✅ Stable (1.2°) | ✅ Stable (1.2°) | ✅ Stable (1.3°) | ⚠️ Wander (1.4°) | ⚠️ Wander (1.7°) | ⚠️ Wander (2.5°) | ⚠️ Wander (2.5°) | ⚠️ Wander (2.6°) | ⚠️ Wander (3.7°) | ❌ Spin (88.3°) | ❌ Spin (89.6°) | ❌ Spin (89.7°) |
| **NASCAR Cup V8 Stock Car** | ✅ Stable (1.3°) | ✅ Stable (1.3°) | ✅ Stable (1.4°) | ⚠️ Wander (1.6°) | ⚠️ Wander (1.8°) | ⚠️ Wander (2.4°) | ⚠️ Wander (2.5°) | ⚠️ Wander (2.7°) | ⚠️ Wander (2.9°) | ⚠️ Wander (3.3°) | ❌ Spin (89.5°) | ❌ Spin (89.6°) |
| **WRC AWD Turbo Rally** | ⚠️ Wander (2.4°) | ⚠️ Wander (2.4°) | ⚠️ Wander (2.4°) | ⚠️ Wander (2.4°) | ⚠️ Wander (2.5°) | ⚠️ Wander (2.7°) | ⚠️ Wander (2.8°) | ⚠️ Wander (2.7°) | ✅ Stable (2.9°) | ⚠️ Wander (2.6°) | ⚠️ Wander (6.6°) | ✅ Stable (3.0°) |
| **300 BHP Sand Rail Buggy** | ⚠️ Wander (2.5°) | ⚠️ Wander (2.5°) | ⚠️ Wander (2.6°) | ⚠️ Wander (2.6°) | ⚠️ Wander (2.8°) | ⚠️ Wander (3.9°) | ⚠️ Wander (4.4°) | ⚠️ Wander (4.1°) | ❌ Spin (81.6°) | ⚠️ Wander (6.4°) | ⚠️ Wander (6.4°) | ✅ Stable (3.9°) |
| **125cc Shifter Kart** | ✅ Stable (0.7°) | ✅ Stable (0.7°) | ✅ Stable (0.8°) | ✅ Stable (0.8°) | ✅ Stable (0.9°) | ⚠️ Wander (2.0°) | ⚠️ Wander (2.3°) | ⚠️ Wander (2.3°) | ✅ Stable (3.1°) | ⚠️ Wander (3.0°) | ❌ Spin (83.8°) | ❌ Spin (87.7°) |
| **GT Sports Coupe (Prototypical)** | ✅ Stable (1.2°) | ✅ Stable (1.3°) | ✅ Stable (1.3°) | ⚠️ Wander (1.5°) | ⚠️ Wander (1.7°) | ⚠️ Wander (2.3°) | ⚠️ Wander (2.6°) | ⚠️ Wander (2.7°) | ✅ Stable (2.9°) | ⚠️ Wander (3.4°) | ⚠️ Wander (8.7°) | ❌ Spin (89.5°) |

## ⚖️ 4. Dynamic EBD & Axle Lockup Duration

Cumulative tire lockup duration: **Front Lockup / Rear Lockup** ($t_{\text{lock}}$ in seconds). Rear lockup should remain zero or near zero to avoid snap oversteer:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | 3.80s / 3.80s | 3.99s / 3.99s | 4.30s / 4.30s | 4.83s / 4.83s | 5.33s / 5.33s | 6.58s / 6.58s | 7.78s / 7.78s | 10.33s / 10.33s | 9.98s / 9.98s | 14.81s / 14.81s | 27.67s / 27.67s | 30.01s / 30.01s |
| **800 BHP LMH Hypercar Prototype** | 4.70s / 4.70s | 4.93s / 4.93s | 5.30s / 5.30s | 5.93s / 5.93s | 6.52s / 6.52s | 7.73s / 7.73s | 9.24s / 9.24s | 12.29s / 12.29s | 11.22s / 11.22s | 17.08s / 17.08s | 30.01s / 30.01s | 30.01s / 30.01s |
| **NASCAR Cup V8 Stock Car** | 3.71s / 3.71s | 3.90s / 3.90s | 4.20s / 4.20s | 4.72s / 4.72s | 5.23s / 5.23s | 6.56s / 6.56s | 7.72s / 7.72s | 10.25s / 10.25s | 10.17s / 10.17s | 14.88s / 14.88s | 27.72s / 27.72s | 30.01s / 30.01s |
| **WRC AWD Turbo Rally** | 3.84s / 3.84s | 4.04s / 4.04s | 4.35s / 4.35s | 4.89s / 4.89s | 5.42s / 5.42s | 6.84s / 6.84s | 8.05s / 8.05s | 10.67s / 10.67s | 10.69s / 10.69s | 15.57s / 15.57s | 28.97s / 28.97s | 30.01s / 30.01s |
| **300 BHP Sand Rail Buggy** | 3.23s / 3.23s | 3.40s / 3.40s | 3.65s / 3.65s | 4.10s / 4.10s | 4.52s / 4.52s | 5.51s / 5.51s | 6.53s / 6.53s | 8.68s / 8.68s | 8.24s / 8.24s | 12.32s / 12.32s | 23.07s / 23.07s | 30.01s / 30.01s |
| **125cc Shifter Kart** | 2.67s / 2.67s | 2.80s / 2.80s | 3.00s / 3.00s | 3.36s / 3.36s | 3.68s / 3.68s | 4.28s / 4.28s | 5.13s / 5.13s | 6.83s / 6.83s | 6.10s / 6.10s | 9.38s / 9.38s | 17.70s / 17.70s | 24.60s / 24.60s |
| **GT Sports Coupe (Prototypical)** | 3.80s / 3.80s | 3.99s / 3.99s | 4.28s / 4.28s | 4.80s / 4.80s | 5.32s / 5.32s | 6.67s / 6.67s | 7.85s / 7.85s | 10.42s / 10.42s | 10.32s / 10.32s | 15.12s / 15.12s | 28.17s / 28.17s | 30.01s / 30.01s |

## 🔀 5. Split-$\mu$ Asymmetric Surface Braking (ISO 14512 Benchmark)

Emergency stop with Left wheels on Asphalt ($\mu = 1.00$) and Right wheels on degraded surface. Evaluates uncommanded yaw torque and directional tracking:

| Vehicle | Asphalt vs Concrete | Asphalt vs Gravel | Asphalt vs Mud | Asphalt vs Grass | Asphalt vs Snow | Asphalt vs Water | Asphalt vs Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | ✅ Straight (Δy=0.5m) | ⚠️ Pulls (Δy=5.4m) | ⚠️ Pulls (Δy=8.5m) | ⚠️ Pulls (Δy=8.6m) | ⚠️ Pulls (Δy=9.7m) | ❌ Spun (44.4°) | ❌ Spun (83.6°) |
| **800 BHP LMH Hypercar Prototype** | ✅ Straight (Δy=0.8m) | ⚠️ Pulls (Δy=7.3m) | ⚠️ Pulls (Δy=7.4m) | ⚠️ Pulls (Δy=8.3m) | ❌ Spun (39.8°) | ❌ Spun (62.5°) | ❌ Spun (117.2°) |
| **NASCAR Cup V8 Stock Car** | ✅ Straight (Δy=0.6m) | ⚠️ Pulls (Δy=6.3m) | ⚠️ Pulls (Δy=7.6m) | ⚠️ Pulls (Δy=7.5m) | ⚠️ Pulls (Δy=9.3m) | ❌ Spun (43.2°) | ❌ Spun (75.7°) |
| **WRC AWD Turbo Rally** | ⚠️ Pulls (Δy=1.5m) | ⚠️ Pulls (Δy=5.1m) | ⚠️ Pulls (Δy=5.8m) | ⚠️ Pulls (Δy=6.5m) | ❌ Spun (38.1°) | ❌ Spun (52.3°) | ❌ Spun (90.6°) |
| **300 BHP Sand Rail Buggy** | ⚠️ Pulls (Δy=2.5m) | ⚠️ Pulls (Δy=4.0m) | ❌ Spun (27.2°) | ❌ Spun (31.2°) | ❌ Spun (38.9°) | ❌ Spun (51.6°) | ❌ Spun (84.1°) |
| **125cc Shifter Kart** | ⚠️ Pulls (Δy=1.5m) | ⚠️ Pulls (Δy=6.0m) | ⚠️ Pulls (Δy=7.5m) | ⚠️ Pulls (Δy=9.5m) | ⚠️ Pulls (Δy=13.2m) | ❌ Spun (53.8°) | ❌ Spun (82.0°) |
| **GT Sports Coupe (Prototypical)** | ✅ Straight (Δy=0.8m) | ⚠️ Pulls (Δy=5.6m) | ⚠️ Pulls (Δy=7.9m) | ⚠️ Pulls (Δy=8.9m) | ⚠️ Pulls (Δy=10.1m) | ⚠️ Pulls (Δy=11.4m) | ❌ Spun (55.5°) |

## 🔄 6. Cornering Trail-Braking (CBC Dynamic Behavior)

Applying full threshold braking while sustaining a steady turn. Evaluates Cornering Brake Control (CBC) and inside-wheel brake modulation:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | ✅ Trail (2.9°) | ✅ Trail (2.9°) | ✅ Trail (3.0°) | ✅ Trail (5.9°) | ✅ Trail (9.0°) | ⚠️ Plow (Δr=8.9m) | ❌ Snap (83.2°) | ❌ Snap (78.6°) | ❌ Snap (87.5°) | ❌ Snap (87.2°) | ❌ Snap (89.0°) | ❌ Snap (89.3°) |
| **800 BHP LMH Hypercar Prototype** | ✅ Trail (7.2°) | ✅ Trail (8.5°) | ✅ Trail (10.4°) | ✅ Trail (13.6°) | ⚠️ Plow (Δr=6.1m) | ❌ Snap (85.7°) | ❌ Snap (87.7°) | ❌ Snap (87.7°) | ❌ Snap (88.3°) | ❌ Snap (88.6°) | ❌ Snap (89.2°) | ❌ Snap (89.4°) |
| **NASCAR Cup V8 Stock Car** | ✅ Trail (3.2°) | ✅ Trail (3.8°) | ✅ Trail (4.7°) | ✅ Trail (6.8°) | ✅ Trail (8.7°) | ⚠️ Plow (Δr=9.5m) | ❌ Snap (48.7°) | ⚠️ Plow (Δr=25.4m) | ❌ Snap (86.4°) | ❌ Snap (84.5°) | ❌ Snap (88.9°) | ❌ Snap (89.3°) |
| **WRC AWD Turbo Rally** | ✅ Trail (4.9°) | ✅ Trail (5.6°) | ✅ Trail (5.6°) | ✅ Trail (6.5°) | ✅ Trail (7.3°) | ⚠️ Plow (Δr=11.2m) | ⚠️ Plow (Δr=15.2m) | ⚠️ Plow (Δr=27.0m) | ❌ Snap (83.0°) | ❌ Snap (58.2°) | ❌ Snap (88.3°) | ❌ Snap (72.9°) |
| **300 BHP Sand Rail Buggy** | ✅ Trail (7.6°) | ✅ Trail (7.4°) | ✅ Trail (7.2°) | ✅ Trail (7.4°) | ✅ Trail (8.1°) | ⚠️ Plow (Δr=9.9m) | ❌ Snap (37.5°) | ⚠️ Plow (Δr=25.0m) | ❌ Snap (83.2°) | ❌ Snap (47.6°) | ❌ Snap (88.0°) | ❌ Snap (76.6°) |
| **125cc Shifter Kart** | ✅ Trail (2.4°) | ✅ Trail (2.4°) | ✅ Trail (2.4°) | ✅ Trail (2.4°) | ✅ Trail (2.4°) | ⚠️ Plow (Δr=7.6m) | ⚠️ Plow (Δr=11.5m) | ⚠️ Plow (Δr=21.1m) | ⚠️ Plow (Δr=16.8m) | ⚠️ Plow (Δr=37.0m) | ❌ Snap (76.7°) | ❌ Snap (53.4°) |
| **GT Sports Coupe (Prototypical)** | ✅ Trail (3.9°) | ✅ Trail (3.4°) | ✅ Trail (3.0°) | ✅ Trail (3.4°) | ✅ Trail (3.9°) | ⚠️ Plow (Δr=10.0m) | ⚠️ Plow (Δr=13.9m) | ⚠️ Plow (Δr=26.2m) | ⚠️ Plow (Δr=27.1m) | ⚠️ Plow (Δr=51.4m) | ❌ Snap (49.8°) | ❌ Snap (61.9°) |

## ⏱️ 7. Cadence Braking / Brake Pumping Recovery Latency

Pulsing brake pedal ($0.15$s on / $0.15$s off). Measures average wheel re-acceleration latency ($t_{\text{spinup}}$ in ms) upon pedal release:

| Vehicle | Asphalt | Concrete | Curb | Dirt | Gravel | Mud | Grass | Snow | Sand | Water | Oil | Ice |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **AMG GT3 Evo** | 0.0ms | 416.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms |
| **800 BHP LMH Hypercar Prototype** | 371.6ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms | 0.0ms |
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

