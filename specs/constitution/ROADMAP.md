---
type: Product Roadmap
title: "Product Roadmap & Feature Backlog"
description: "Living roadmap tracking upcoming developmental milestones and technical debt."
status: active
---

# Product Roadmap & Feature Backlog 🚀

A living template tracking upcoming development milestones, technical debt tasks, and the feature backlog of potential future ideas.

---

## 📅 Living Milestones: [Current Phase, e.g. Q2 2026]

Organize chronological active phases as actionable check-lists. Agents will scan these links to find their targets:

### Phase 1: Motorsport Career Expansion (Priority: High)
- `[ ]` **[NASCAR Career Mode](../001_nascar_career_mode.md)**: 5-tier stock car & Trans-Am career progression with stage racing and pack drafting.
- `[ ]` **[Rallycross & All-Terrain Career Mode](../002_rallycross_and_allterrain_career_mode.md)**: 5-tier World RX and desert raid career progression with joker lap rules.
- `[ ]` **[Extreme Off-Road & Stunt Arenas Career Mode](../003_extreme_offroad_and_stunt_arenas_career_mode.md)**: 5-tier off-road and stunt arena career progression with freestyle scoring.
- `[ ]` **[Karting Career Mode](../004_karting_career_mode.md)**: 5-tier grassroots karting, shifter, racing mower, and superkart career progression.

### Phase 2: Vehicle Roster Expansion & Garage Showroom (Priority: Medium)
- `[ ]` **[Real-World Vehicle Rosters & Interactive Garage](../009_real_world_car_models_and_garage.md)**: Migration to authentic motorsport models, Balance of Performance (BoP) calibration, dual-view 2D rendering, and interactive showroom screen.

---

## 🛠 Technical Debt & Maintenance

Living list of security audits, performance profiling targets, or general workspace cleanup routines:
- **Performance**: Simulation throughput benchmarks ($\ge 4.0\text{M steps/sec}$) and collision checks ($\ge 22.0\text{M checks/sec}$).
- **Hygiene**: Spec-Driven Development alignment via `keel doctor` and `keel validate`.

---

## 🗄️ Combined Feature Backlog

Unscheduled explorations, long-term visions, and community feature requests:

### 1. Multiplayer Networking & Online Lobbies
- **LAN & Cloud Matchmaking**: Peer-to-peer and dedicated room matchmaking for cross-platform multiplayer.

### 2. Procedural Circuit Generation
- **Terrain Synthesis**: Seed-based procedural track ribbon generation with elevation contours and surface hazards.

---

## ✅ Completed Milestones

Chronological log of past successfully deployed features and versions:
- `[x]` **[Modular Motorsport Architecture](../005_modular_racing_architecture.md)**: Extracted standalone crates `crates/wheelbase` and `crates/arcade-race-core` (2026-09-13).
- `[x]` **[Road Split & Branching Tracks](../006_road_split_and_branching_tracks.md)**: Implemented Directed Ribbon Graph (`TrackNetwork`), Joker Laps, and multi-layout tracks (2026-09-15).
- `[x]` **[Extreme Off-Road & Stunt Arenas Module](../007_extreme_offroad_and_stunt_arenas.md)**: 300 BHP Sand Rail Buggy, 15 venues, Mud/Snow physics, and open arenas (2026-09-17).
- `[x]` **[Race Modality Selection & Interface Flow](../008_modality_selection_and_menu_flow.md)**: Dedicated Single Player vs Multiplayer modality selection screen (2026-09-18).
