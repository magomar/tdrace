---
type: Index
title: "TdRace Knowledge Base & Asset Catalog"
description: "Master progressive disclosure index of game assets, circuit directories, simulation physics, and engineering references."
status: active
okf_version: "0.2"
category: index
tags: [index, knowledge-base, assets, physics, circuits]
---

# TdRace Knowledge Base & Asset Catalog 🏁📚

Welcome to the central, single-source-of-truth knowledge graph for **TdRace** (Top-Down Racing). This repository contains technical reference manuals, mathematical models, asset catalogs, and architectural specifications structured according to **Google's Open Knowledge Format (OKF v0.2)**.

---

## 🔬 Simulation Physics Reference

Comprehensive mathematical modeling of planar and 2.5D motorsport physics implemented in [`crates/wheelbase`](../crates/wheelbase) and [`crates/arcade-race-core`](../crates/arcade-race-core).

| Document | Type | Status | Description |
| :--- | :--- | :---: | :--- |
| [Overview](physics/index.md) | Physics Reference | `active` | Architectural overview of vehicle physics, integration pipelines, and 60 Hz update cycles. |
| [Vehicle Dynamics](physics/vehicle_dynamics.md) | Physics Reference | `active` | Planar kinematics, center of gravity (CG), longitudinal squat/dive, lateral body roll, downforce, and aerodynamic drag. |
| [Pacejka '96 Tire Model](physics/tire_pacejka.md) | Physics Reference | `active` | Pacejka Magic Formula coefficients ($B, C, D, E$), slip angle calculation, grip saturation, and drift slide friction. |
| [Surface Physics Matrix](physics/surfaces.md) | Physics Reference | `active` | 11-surface simulation matrix: friction coefficients ($\mu$), rolling resistance, viscous drag, and per-wheel split-$\mu$ sampling. |
| [Walls & Barriers](physics/walls_barriers.md) | Physics Reference | `active` | 4 perimeter barrier profiles: Concrete, Steel Armco, Tire Wall, Curb Wall restitution coefficients and impulse resolution. |
| [Powertrain & Drivetrain](physics/powertrain.md) | Physics Reference | `active` | Engine tractive forces, drivetrain layouts (FWD/AWD/RWD), brake distribution, and roadmap for multi-gear transmissions and thermal tire degradation. |

---

## 🏎️ Vehicle Roster & Motorsport Modules

25 progression categories across 5 motorsport disciplines, balanced using Balance of Performance (BoP) equations.

| Document | Type | Status | Description |
| :--- | :--- | :---: | :--- |
| [Vehicle Catalog Overview](vehicles/index.md) | Asset Catalog | `active` | Master overview of 5 motorsport modules, 25 progression categories, and Balance of Performance calibration. |
| [Gran Turismo & Endurance](vehicles/gt_endurance.md) | Asset Catalog | `active` | GT4, GT3 EVO, GT2, GT1 Legends, and Le Mans LMH/LMDh Hypercars. |
| [NASCAR Stock Car & Trans-Am](vehicles/stock_car.md) | Asset Catalog | `active` | Street Stock, Late Model, ARCA Menards, Craftsman Super Truck, and Trans-Am TA1 / Cup. |
| [Rallycross & All-Terrain](vehicles/rally_allterrain.md) | Asset Catalog | `active` | Rally Jr FWD, World RX Supercars, Group B Legends, Dakar Raid T1+, and Stadium Super Trucks. |
| [Extreme Off-Road & Stunts](vehicles/offroad_stunt.md) | Asset Catalog | `active` | Sand Rail Buggies, Trophy Trucks, Arctic Ice Drifters, Mud Boggers, and Freestyle Monster Trucks. |
| [Karting & Micro-Racers](vehicles/karting.md) | Asset Catalog | `active` | 60cc Cadet karts, 100cc OK Junior, 125cc KZ2 Shifters, Racing Lawnmowers, and 240 km/h 250cc Superkarts. |

---

## 🏁 World Circuit & Track Directory

90 vector geometry tracks catalogued across 6 racing modalities.

| Document | Type | Status | Description |
| :--- | :--- | :---: | :--- |
| [Circuits Directory Overview](circuits/index.md) | Asset Catalog | `active` | Overview of all 90 tracks, spline representations, surface zones, and barrier definitions. |
| [Classic Heritage Circuits](circuits/classic.md) | Asset Catalog | `active` | 10 foundational tracks: Classic Grand Prix, Drift Park, Figure Eight crossovers, and jump ramp raceways. |
| [NASCAR & Stock Car Ovals](circuits/nascar.md) | Asset Catalog | `active` | 12 speedways: Daytona, Talladega, Bristol, Martinsville, Charlotte, Darlington, and Eldora clay dirt oval. |
| [Rallycross Stages](circuits/rally.md) | Asset Catalog | `active` | 17 World RX venues: Höljes, Lydden Hill, Hell RX, Lohéac, Joker Lap branching networks, and Sahara sand raids. |
| [Karting Circuits & Arenas](circuits/kart.md) | Asset Catalog | `active` | 15 international karting venues: South Garda (Lonato), Genk, Sarno, Wackersdorf, and PFI bridge crossover. |
| [Extreme Off-Road Arenas](circuits/offroad.md) | Asset Catalog | `active` | 15 extreme arenas: sand dune raids, deep mud bog pits, arctic ice lakes, and stadium whoops. |
| [Formula 1 & GT Road Courses](circuits/f1_gt.md) | Asset Catalog | `active` | 19 FIA Grade 1 world championship circuits: Spa-Francorchamps, Monza, Silverstone, Le Mans, Bathurst, and Suzuka. |

---

## 🛠️ Architecture & Engineering Reference

Technical design documents, audio engine architectures, and performance profiles.

| Document | Type | Status | Description |
| :--- | :--- | :---: | :--- |
| [Screen Architecture](engineering/screens_and_navigation.md) | Architecture Spec | `active` | UI screens, state machines, transition triggers, and navigation schemas. |
| [Motor Sound Synthesis](engineering/motor_sound_improvement.md) | Architecture Spec | `active` | Procedural motor sound synthesis, pitch modulation, exhaust pop harmonics, and Kira integration. |
| [Circuit Rendering Performance](engineering/circuit_rendering_performance.md) | Architecture Spec | `active` | OpenGL batching, Catmull-Rom spline tessellation, and memory footprint analysis. |
| [Surface & Wall Legacy Spec](engineering/surface_and_wall_legacy_spec.md) | Architecture Spec | `active` | Historical specifications for 11 surface types and 4 barrier collision models. |
| [Vehicle Roster Ideas](engineering/vehicle_roster_expansion_ideas.md) | Feature Spec | `active` | Exploratory design notes on prospective vehicle variants and class archetypes. |
