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
| [Pacejka '96 & Tyre Dynamics](http://localhost:4322/technical/tyres) | Codex Reference | `active` | Interactive Pacejka slip curves, thermal degradation, optimal temperature windows, and 4-compound telemetry in the TDRace Codex. |
| [Surface Physics Matrix](http://localhost:4322/technical/surfaces) | Codex Reference | `active` | 15-surface simulation matrix: dynamic friction ($\mu$), rolling resistance, drag, and 8×15 compound affinity heatmap in the TDRace Codex. |
| [Walls & Barriers](physics/walls_barriers.md) | Physics Reference | `active` | 4 perimeter barrier profiles: Concrete, Steel Armco, Tire Wall, Curb Wall restitution coefficients and impulse resolution. |
| [Powertrain & Drivetrain](physics/powertrain.md) | Physics Reference | `active` | Engine tractive forces, drivetrain layouts (FWD/AWD/RWD), brake distribution, and roadmap for multi-gear transmissions and thermal tire degradation. |

---

## 🏎️ Vehicle Catalog & Motorsport Disciplines

Vehicle catalogs, chassis geometry, suspension archetypes, homologated specifications, and telemetry ratings are maintained in the live **[TDRace Codex](http://localhost:4322/showroom/cars)** on port 4322, generated directly from engine source definitions.

| Portal Section | Type | Status | Description |
| :--- | :--- | :---: | :--- |
| [Codex Car Catalogue](http://localhost:4322/showroom/cars) | Interactive Codex | `active` | Complete 80+ homologated car catalogue across all 7 disciplines with 6-axis telemetry radars, power curves, and 3D specifications. |
| [Vehicle Compare Tool](http://localhost:4322/showroom/compare) | Interactive Codex | `active` | Side-by-side comparative telemetry analysis for 2 to 4 competitor vehicles. |
| [Motorsport Disciplines](http://localhost:4322/racing/disciplines) | Interactive Codex | `active` | Overview of all 7 motorsport disciplines, homologation tiers, licence requirements, and series presets. |

---

## 🏁 World Circuit & Track Directory

Meticulously mapped vector geometry circuits and procedural arenas catalogued across 6 racing modalities.

| Document | Type | Status | Description |
| :--- | :--- | :---: | :--- |
| [Circuits Directory Overview](circuits/index.md) | Asset Catalog | `active` | Overview of all 96 circuits, spline representations, surface zones, and barrier definitions. |
| [Classic Heritage Circuits](circuits/classic.md) | Asset Catalog | `active` | 10 foundational tracks: Classic Grand Prix, Drift Park, Figure Eight crossovers, and jump ramp raceways. |
| [Stock cars & Ovals](circuits/nascar.md) | Asset Catalog | `active` | 17 speedways: Daytona, Talladega, Bristol, Martinsville, Charlotte, Darlington, and Eldora clay dirt oval. |
| [Rallycross Stages](circuits/rally.md) | Asset Catalog | `active` | 17 World RX venues: Höljes, Lydden Hill, Hell RX, Lohéac, Essay RX, and Joker Lap branching networks. |
| [Karting Circuits & Arenas](circuits/kart.md) | Asset Catalog | `active` | 17 international karting venues: South Garda (Lonato), Genk, Sarno, Valencia, Campillos, and PFI bridge crossover. |
| [All terrain Arenas](circuits/offroad.md) | Asset Catalog | `active` | 17 extreme arenas: sand dune raids, deep mud bog pits, arctic ice lakes, and stadium whoops. |
| [GT World Challenge & Endurance Road Courses](circuits/f1_gt.md) | Asset Catalog | `active` | 18 FIA Grade 1 world championship circuits: Spa-Francorchamps, Monza, Silverstone, Le Mans, Bathurst, MadRing, and Suzuka. |

---

## 🏆 Career Progression & Economy Reference

Comprehensive specifications covering 5-tier motorsport careers, tournament scoring, experience points (XP), tier promotion gates, and vehicle acquisition.

| Document | Type | Status | Description |
| :--- | :--- | :---: | :--- |
| [Career Progression & Points Systems](career/index.md) | Reference Guide | `active` | 5-tier career architecture, tournament scoring matrices (FIA, MotoGP, Classic, NASCAR), distance-based XP formulas, tier promotion criteria, and vehicle acquisition economy. |

---

## 🛠️ Architecture & Engineering Reference

Technical design documents, audio engine architectures, and performance profiles.

| Document | Type | Status | Description |
| :--- | :--- | :---: | :--- |
| [AI Driving Archetypes & Characters](engineering/ai_driving_archetypes_and_characters.md) | Architecture Spec | `active` | Orthogonal driving styles (6 styles) and quality tiers (5 tiers), physics control models, and 72 hand-tuned driver profiles across 6 modules. |
| [Screen Architecture](engineering/screens_and_navigation.md) | Architecture Spec | `active` | UI screens, state machines, transition triggers, and navigation schemas. |
| [UI & Screen Terminology](engineering/terminology.md) | Architecture Spec | `active` | Formal UI component hierarchy, Cabinet platform primitives, and canonical screen catalog. |
| [Motor Sound Synthesis](engineering/motor_sound_improvement.md) | Architecture Spec | `active` | Procedural motor sound synthesis, pitch modulation, exhaust pop harmonics, and Kira integration. |
| [Circuit Rendering Performance](engineering/circuit_rendering_performance.md) | Architecture Spec | `active` | OpenGL batching, Catmull-Rom spline tessellation, and memory footprint analysis. |
| [Circuit Building & Environment Analysis](engineering/circuit_building_analysis.md) | Architecture Spec | `active` | As-built analysis of track splines, runoff, walls, scenery, rendering and the OSM import pipeline, with defects and realism gaps. |
| [Circuit Realism Knowledge Base](engineering/circuit_realism_knowledge_base.md) | Architecture Spec | `active` | OSM tagging and Overpass queries, runoff derivation, FIA design rules at game scale, grandstand/pit/scenery placement, visual design, and improvement plan. |
| [Steering Responsiveness & Controls](engineering/high_speed_steering_responsiveness_and_double_attenuation.md) | Architecture Spec | `active` | Root cause analysis and resolution of high-speed steering double-attenuation across input filter and wheelbase physics. |
| [Surface & Wall Legacy Spec](engineering/surface_and_wall_legacy_spec.md) | Architecture Spec | `active` | Historical specifications for 12 surface types and 4 barrier collision models. |
| [Vehicle Roster Ideas](engineering/vehicle_roster_expansion_ideas.md) | Feature Spec | `active` | Exploratory design notes on prospective vehicle variants and class archetypes. |
| [LAN Race Netcode](engineering/lan_netcode.md) | Architecture Spec | `active` | Owner-authoritative LAN cars, host relay and referee, shared race clock, interpolation, reliable control channel, packet formats and tuning constants. |
| [Contributing Guide](engineering/contributing.md) | Architecture Spec | `active` | Repository setup, cross-platform symlink instructions, and development workflow. |

---

## 🔬 Empirical Simulation & Dynamic Benchmarks

Headless physical simulation batteries, empirical surface degradation reports, and vehicle balance assessments.

| Document | Type | Status | Description |
| :--- | :--- | :---: | :--- |
| [Surface Simulation Technical Report](experiments/surface_simulation_technical_report.md) | Technical Report | `active` | In-depth realism and balance evaluation across 30 vehicles, 12 surfaces, and 5 dynamic test protocols. |
| [Full Surface Simulation Telemetry](experiments/full_surface_simulation_report.md) | Technical Report | `active` | Comprehensive 1,800-run empirical dataset covering all 5 vehicle tiers, classic prototypes, and 12 surfaces. |
| [Category Dynamics Benchmark](experiments/surface_simulation_benchmark.md) | Physics Reference | `active` | Baseline empirical evaluation of 12 representative vehicle archetypes across non-classic modules. |
