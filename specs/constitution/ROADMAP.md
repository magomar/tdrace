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
- `[x]` **[NASCAR Career Mode](../001_nascar_career_mode.md)**: 5-tier stock car & Trans-Am career progression with stage racing and pack drafting.
- `[x]` **[Rallycross & All-Terrain Career Mode](../002_rallycross_and_allterrain_career_mode.md)**: 5-tier World RX and desert raid career progression with joker lap rules.
- `[x]` **[Extreme Off-Road & Stunt Arenas Career Mode](../003_extreme_offroad_and_stunt_arenas_career_mode.md)**: 5-tier off-road and stunt arena career progression with freestyle scoring.
- `[x]` **[Karting Career Mode](../004_karting_career_mode.md)**: 5-tier grassroots karting, shifter, racing mower, and superkart career progression.

### Phase 2: Vehicle Roster Expansion & Garage Showroom (Priority: Medium)
- `[x]` **[Real-World Vehicle Rosters & Interactive Garage](../009_real_world_car_models_and_garage.md)**: Migration to authentic motorsport models, Balance of Performance (BoP) calibration, dual-view 2D rendering, and interactive showroom screen.
- `[x]` **[Vehicle Telemetry Alignment and Physics Differentiation](../018_vehicle_telemetry_alignment_and_physics_differentiation.md)**: 6-metric telemetry standardization with custom vector iconography and intra-category physics parameter differentiation.
- `[x]` **[Per-Discipline and Per-Tier Driver Favorite Cars & 12-Pilot Rosters](../020_driver_favorite_cars_per_discipline_and_tier.md)**: Expands motorsport rosters to 12 unique pilots per discipline (72 total) and establishes authentic signature vehicle mappings per discipline and tier.
- `[ ]` **[Per-Tier Engine Sound Banks and Physical Synthesis](../022_per_tier_engine_sound_banks_and_synthesis.md)**: Expands motor audio synthesis to 25 authentic physical archetypes across all 5 tiers of the 5 motorsport disciplines with turbo flutter, blower whine, and hybrid motor acoustics.

### Phase 3: Game Asset Catalogue & Reference Portals (Priority: High)
- `[x]` **[Game Asset Catalogue & Technical Reference Portals](../011_game_asset_catalogue_and_physics_reference_portals.md)**: Dual-site Astro architecture cataloguing 25 car categories, 90 circuits, and simulation physics under a unified Google OKF v0.2 knowledge graph.

### Phase 4: Track Scenery & World Building (Priority: High)
- `[x]` **[Track Scenery and Decorative Elements](../012_track_scenery_and_decorative_elements.md)**: Tiered concrete grandstands and multi-species trees with cenital recognition and physical car interactions.
- `[x]` **[Segment Runoff Terrain & Virtual Track Boundaries](../014_segment_runoff_terrain_and_virtual_track_boundaries.md)**: Track-segment runoff terrain properties (e.g. gravel margins, tarmac runoff) and non-physical virtual boundary barriers for fluid off-track surface transitions.
- `[ ]` **[Surface Textures & Environmental Materials](../016_surface_textures_and_environmental_materials.md)**: High-fidelity tileable texture mapping, dual spline/world UV pipeline, macro-modulation, and organic terrain transitions across all 12 racing surfaces.
- `[/]` **[Surface Tire Marks and Runoff Terrain Dynamics](../019_surface_tire_marks_and_runoff_terrain_dynamics.md)**: Surface query precedence fix for segment runoff corridors over below-track zones, multi-surface tire marking and deformable rutting pipeline, and tire dirt contamination.
- `[x]` **[Authentic OpenStreetMap References and Circuit Provenance](../021_authentic_openstreetmap_references_and_circuit_provenance.md)**: Restore authentic OpenStreetMap and Wikipedia provenance URLs for 71 real circuits across GT, Kart, Rally, NASCAR, and Extreme Off-Road, ensuring non-null propagation to presets, tracks/*.json, and circuits.json.

### Phase 5: Tournament & Career Engine (Priority: High)
- `[ ]` **[Declarative Championship Format & In-Game Championship Editor](../017_declarative_championship_format_and_editor.md)**: Declarative TOML championship format, discovery engine, and interactive developer Championship Editor studio.

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
