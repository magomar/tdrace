---
type: Index
title: "Specifications & Project Knowledge"
description: "Progressive disclosure directory index of Keel constitution and specifications."
status: active
---

# Specifications & Project Knowledge 📋

Table of contents providing progressive disclosure of project constitution and technical specifications.

## 🏛️ Constitution & Governance

| Document | Type | Status | Description |
| :--- | :--- | :--- | :--- |
| [MISSION.md](constitution/MISSION.md) | Project Constitution | `active` | Constitutional governance, coding laws, and architectural boundaries. |
| [TECH_STACK.md](constitution/TECH_STACK.md) | Tech Stack | `active` | Approved technical stack, framework runtimes, styling guidelines, and commands. |
| [ROADMAP.md](constitution/ROADMAP.md) | Product Roadmap | `active` | Living roadmap tracking upcoming developmental milestones and technical debt. |
| [BACKLOG.md](../BACKLOG.md) | Feature Backlog | `active` | Consolidated idea registry for pre-development features and speculative enhancements. |

## 📋 Technical Specifications

| Spec | Title | Type | Status | Description |
| :--- | :--- | :--- | :--- | :--- |
| [001](001_nascar_career_mode.md) | NASCAR & Trans-Am TA1 Career Mode | Feature Spec | `implemented` | 5-tier American stock car and Trans-Am TA1 career progression featuring 17 circuits, stage racing rules, pack drafting dynamics, declarative series presets, and XP unlocks. |
| [002](002_rallycross_and_allterrain_career_mode.md) | Rallycross & All-Terrain World Cup Career Mode | Feature Spec | `implemented` | 5-tier Rallycross and All-Terrain career ladder covering 17 global mixed-surface circuits, World RX tournament format, mandatory Joker lap rules, and declarative series presets. |
| [003](003_extreme_offroad_and_stunt_arenas_career_mode.md) | Extreme Off-Road & Stunt Arenas Career Mode | Feature Spec | `implemented` | 5-tier career progression combining open desert raids, ice drifting, deep mud bogs, supercross stadium whoops, and monster truck freestyle stunt arenas across a 17-venue calendar. |
| [004](004_karting_career_mode.md) | Karting World Cup Career Mode | Feature Spec | `implemented` | 5-tier grassroots karting career ladder from 60cc Cadet karts to 240 km/h 250cc Superkarts and novelty racing lawnmowers across a 17-venue calendar. |
| [005](005_modular_racing_architecture.md) | Modular Motorsport Architecture (wheelbase & arcade-race-core) | Architecture Spec | `implemented` | Extraction of reusable zero-dependency Rust engine crates: wheelbase (Pacejka vehicle physics) and arcade-race-core (SAT collisions, splines, timing). |
| [006](006_road_split_and_branching_tracks.md) | Road Split Segments, Branching Splines & Alternative Circuit Layouts | Architecture Spec | `implemented` | Directed Ribbon Graph (TrackNetwork) architecture enabling branching splines, Rallycross Joker Laps, and multi-layout tracks. |
| [007](007_extreme_offroad_and_stunt_arenas.md) | Extreme Off-Road & Stunt Arenas Module | Feature Spec | `implemented` | High-octane off-road module featuring the 300 BHP Sand Rail Buggy, 15 circuits and open arenas, Mud/Snow physics, and rhythm whoops. |
| [008](008_modality_selection_and_menu_flow.md) | Race Modality Selection & Interface Flow Restructuring | Feature Spec | `implemented` | Dedicated ModalitySelect screen separating Single Player (Quick, Custom, Career, Time Trial) from Multiplayer (Split Screen, LAN, Cloud). |
| [009](009_real_world_car_models_and_garage.md) | Real-World Vehicle Rosters, Physics Balance & Interactive Garage | Feature Spec | `implemented` | Migration from prototypical archetypes to authentic real-world motorsport models across 5 modules and 25 categories, with BoP parity, dual-view 2D rendering, and interactive garage showroom. |
| [010](010_surface_car_interaction_simulation.md) | Systematic Computational Simulation of Surface-Car Dynamics | Architecture Spec | `implemented` | Headless computational simulation harness and benchmarking suite to measure vehicle acceleration, braking, and cornering dynamics across all surface types without graphics. |
| [011](011_game_asset_catalogue_and_physics_reference_portals.md) | Game Asset Catalogue & Technical Reference Portals | Architecture Spec | `implemented` | Dual-site Astro architecture cataloguing 25 car categories, 90 circuits, and simulation physics under a unified Google OKF v0.2 knowledge graph. |
| [012](012_track_scenery_and_decorative_elements.md) | Track Scenery and Decorative Elements (Grades & Trees) | Feature Spec | `implemented` | Decorative track elements featuring tiered concrete grandstands and multi-species trees designed for cenital (top-down) recognition with realistic physical car interactions. |
| [013](013_player_profile_enhancement_and_career_dossier.md) | Player Profile Enhancement & Career Dossier | Feature Spec | `implemented` | Full-screen player profile dossier with integrated driver switcher, multi-level career hierarchy (Global -> Category -> Championship -> Race), stunt points, collision telemetry, and three selectable layout alternatives. |
| [014](014_segment_runoff_terrain_and_virtual_track_boundaries.md) | Segment Runoff Terrain and Virtual Track Boundaries | Feature Spec | `implemented` | Segment-level off-track runoff terrain definition and non-physical virtual boundary barriers for authentic gravel traps, tarmac runoffs, and multi-tier surface zones. |
| [015](015_modality_selector_hub_icons_and_visual_emblems.md) | Modality Selector Hub Visual Iconography & Emblems | Feature Spec | `implemented` | Custom vector iconography and high-DPI emblems for all 12 race modalities across Single Player, Multiplayer, and Options. |
| [016](016_surface_textures_and_environmental_materials.md) | Surface Textures and Environmental Materials | Feature Spec | `implemented` | High-fidelity texture rendering pipeline, dual ribbon/world UV mapping, macro-color modulation, and organic edge transitions across all 12 racing surfaces. |
| [017](017_declarative_championship_format_and_editor.md) | Declarative Series Format and Series Studio Editor (Championships & Cups) | Feature Spec | `implemented` | Human-readable declarative TOML series specification format, file discovery engine, and Series Studio editor for authoring, validating, and testing championships and cups. |
| [018](018_vehicle_telemetry_alignment_and_physics_differentiation.md) | Vehicle Telemetry Alignment and Physics Differentiation | Feature Spec | `implemented` | Unify 6-stat performance telemetry across in-game garage and web showroom with custom vector icons, while dynamically hooking up individual vehicle parameters (braking, grip, agility, and aerodynamics) in the simulation engine. |
| [019](019_surface_tire_marks_and_runoff_terrain_dynamics.md) | Surface Tire Marks and Runoff Terrain Dynamics | Feature Spec | `implemented` | Corrects surface sampling precedence between segment runoff corridors and below-track zones, and establishes an authentic physical tire marking and debris roost pipeline across all 12 surfaces. |
| [020](020_driver_favorite_cars_per_discipline_and_tier.md) | Per-Discipline and Per-Tier Driver Favorite Cars & 12-Pilot Rosters | Feature Spec | `implemented` | Expands motorsport rosters to 12 unique pilots per discipline (72 total) and establishes authentic signature vehicle mappings per discipline and tier across DriverCharacter, modules, and race sessions. |
| [021](021_authentic_openstreetmap_references_and_circuit_provenance.md) | Authentic OpenStreetMap References and Circuit Provenance | Feature Spec | `implemented` | Embeds authentic OpenStreetMap (OSM) relations and ways and reference URLs across all 71 real-world circuits, fixing showroom inaccuracies and establishing persistent provenance across Rust presets, track JSON files, and Astro portals. |
| [022](022_per_tier_engine_sound_banks_and_synthesis.md) | Per-Tier Engine Sound Banks and Physical Synthesis | Feature Spec | `draft` | Expands the motor audio architecture to 25 distinct physical sound archetypes across all 5 tiers of the 5 motorsport disciplines, featuring custom acoustic wave shaping, forced induction, and hybrid whine. |
| [023](023_orthogonal_ai_driving_styles_and_quality_tiers.md) | Orthogonal AI Driving Styles and Quality Tiers | Feature Spec | `implemented` | Decouples AI driving style (tactical personality) from experience and quality (competence, consistency, execution) into orthogonal dimensions across 6 styles and 5 performance tiers. |
| [024](024_crossmodule_ai_character_rosters_and_dynamic_tier_assignment.md) | Cross-Module AI Character Rosters and Dynamic Tier Assignment | Feature Spec | `draft` | Decouples experience tiers from character definitions, aligns 72 drivers to 6 styles, introduces uniform style sampling from the global pool, normal-distribution difficulty tiering, and persistent career rosters with 90% retention and probabilistic tier advancement. |
