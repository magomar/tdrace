---
type: Product Roadmap
title: "Product Roadmap & Feature Backlog"
description: "Living roadmap tracking upcoming developmental milestones and technical debt."
status: active
---

# Product Roadmap & Feature Backlog 🚀

A living template tracking upcoming development milestones, technical debt tasks, and the feature backlog of potential future ideas.

---

## 📅 Living Milestones: Q3–Q4 2026

Organize chronological active phases as actionable check-lists. Agents will scan these links to find their targets:

### Phase 1: Motorsport Career Expansion (Priority: High)
- `[x]` **[NASCAR Career Mode](../001_nascar_career_mode.md)**: 5-tier stock car & Trans-Am career progression with stage racing and pack drafting.
- `[x]` **[Rallycross & All-Terrain Career Mode](../002_rallycross_and_allterrain_career_mode.md)**: 5-tier World RX and desert raid career progression with joker lap rules.
- `[x]` **[Extreme Off-Road & Stunt Arenas Career Mode](../003_extreme_offroad_and_stunt_arenas_career_mode.md)**: 5-tier off-road and stunt arena career progression with freestyle scoring.
- `[x]` **[Karting Career Mode](../004_karting_career_mode.md)**: 5-tier grassroots karting, shifter, racing mower, and superkart career progression.

### Phase 2: Vehicle Roster Expansion, AI Driving Styles & Audio (Priority: High)
- `[x]` **[Real-World Vehicle Rosters & Interactive Garage](../009_real_world_car_models_and_garage.md)**: Migration to authentic motorsport models, Balance of Performance (BoP) calibration, dual-view 2D rendering, and interactive showroom screen.
- `[x]` **[Vehicle Telemetry Alignment and Physics Differentiation](../018_vehicle_telemetry_alignment_and_physics_differentiation.md)**: 6-metric telemetry standardization with custom vector iconography and intra-category physics parameter differentiation.
- `[x]` **[Per-Discipline and Per-Tier Driver Favorite Cars & 12-Pilot Rosters](../020_driver_favorite_cars_per_discipline_and_tier.md)**: Expands motorsport rosters to 12 unique pilots per discipline (72 total) and establishes authentic signature vehicle mappings per discipline and tier.
- `[x]` **[Orthogonal AI Driving Styles and Quality Tiers](../023_orthogonal_ai_driving_styles_and_quality_tiers.md)**: Decouples driving style (tactical personality) from experience and quality (competence, consistency, execution) into orthogonal dimensions across 6 styles and 5 performance tiers.
- `[x]` **[Cross-Module AI Character Rosters and Dynamic Tier Assignment](../024_crossmodule_ai_character_rosters_and_dynamic_tier_assignment.md)**: Decouples experience tiers from character definitions, aligns 72 drivers across 6 styles, introduces uniform style sampling from the global pool, normal-distribution difficulty tiering for casual races, and persistent career rosters with 90% retention and probabilistic tier advancement.
- `[x]` **[Per-Tier Engine Sound Banks and Physical Synthesis](../022_per_tier_engine_sound_banks_and_synthesis.md)**: Expands motor audio synthesis to 25 authentic physical archetypes across all 5 tiers of the 5 motorsport disciplines with turbo flutter, blower whine, and hybrid motor acoustics.
- `[x]` **[Floating Bot Names and Dynamic Proximity Nameplates](../029_floating_bot_names_and_dynamic_proximity_nameplates.md)**: In-race dynamic overhead floating bot nameplates with proximity culling, distance-based alpha fading, anti-crowding deconfliction, and Alt key toggle.

### Phase 3: Physics Simulation Harness & Technical Asset Portals (Priority: High)
- `[x]` **[Systematic Computational Simulation of Surface-Car Dynamics](../010_surface_car_interaction_simulation.md)**: Headless computational simulation harness and benchmarking suite to measure vehicle acceleration, braking, and cornering dynamics across all surface types without graphics.
- `[x]` **[Game Asset Catalogue & Technical Reference Portals](../011_game_asset_catalogue_and_physics_reference_portals.md)**: Dual-site Astro architecture cataloguing 25 car categories, 90 circuits, and simulation physics under a unified Google OKF v0.2 knowledge graph.

### Phase 4: Track Scenery, Surface Dynamics & Terrain Physics (Priority: High)
- `[x]` **[Track Scenery and Decorative Elements](../012_track_scenery_and_decorative_elements.md)**: Tiered concrete grandstands and multi-species trees with cenital recognition and physical car interactions.
- `[x]` **[Segment Runoff Terrain & Virtual Track Boundaries](../014_segment_runoff_terrain_and_virtual_track_boundaries.md)**: Track-segment runoff terrain properties (e.g. gravel margins, tarmac runoff) and non-physical virtual boundary barriers for fluid off-track surface transitions.
- `[x]` **[Surface Textures & Environmental Materials](../016_surface_textures_and_environmental_materials.md)**: High-fidelity tileable texture mapping, dual spline/world UV pipeline, macro-modulation, and organic terrain transitions across all 12 racing surfaces.
- `[x]` **[Surface Tire Marks and Runoff Terrain Dynamics](../019_surface_tire_marks_and_runoff_terrain_dynamics.md)**: Surface query precedence fix for segment runoff corridors over below-track zones, multi-surface tire marking and deformable rutting pipeline, and tire dirt contamination.
- `[x]` **[Authentic OpenStreetMap References and Circuit Provenance](../021_authentic_openstreetmap_references_and_circuit_provenance.md)**: Restore authentic OpenStreetMap and Wikipedia provenance URLs for 71 real circuits across GT, Kart, Rally, NASCAR, and Extreme Off-Road, ensuring non-null propagation to presets, tracks/*.json, and circuits.json.
- `[ ]` **[Terrain Surface Bifurcation, Vehicle Terrain Interaction, and Category-Tier Gating](../025_terrain_surface_bifurcation_and_category_tier_gating.md)**: Bifurcates sand into packed dune ribbons vs deep arrestor traps, defines vehicle-terrain interaction coefficients (sand flotation, mud paddles, ice studs), and establishes simulation-backed category and tier gating.

### Phase 5: Tournament Engine, Career Progression & Player Dossier (Priority: High)
- `[x]` **[Modality Selector Hub Visual Iconography & Emblems](../015_modality_selector_hub_icons_and_visual_emblems.md)**: Custom vector iconography and high-DPI emblems for all 12 race modalities across Single Player, Multiplayer, and Options.
- `[x]` **[Player Profile Enhancement & Career Dossier](../013_player_profile_enhancement_and_career_dossier.md)**: Full-screen player profile dossier with integrated driver switcher, multi-level career hierarchy (Global -> Category -> Championship -> Race), stunt points, collision telemetry, and three selectable layout alternatives.
- `[x]` **[Declarative Championship Format & In-Game Championship Editor](../017_declarative_championship_format_and_editor.md)**: Declarative TOML championship format, discovery engine, and interactive developer Championship Editor studio.
- `[x]` **[Championship Trophy Badges & Player Profile Trophy Cabinet](../027_championship_trophy_badges_and_player_profile_cabinet.md)**: Comprehensive vector iconography system for championship podium badges (Gold, Silver, Bronze) with tier stars (1-5), modality-specific motorsport DNA across 5 disciplines, and an interactive Trophy Cabinet UI within the Player Profile.
- `[x]` **[Career System Wiki Reference and Progression Mechanics](../030_career_system_wiki_reference_and_progression_mechanics.md)**: Comprehensive technical reference and public wiki documentation of the 5-tier career system, championship scoring matrices, XP economy, car acquisition, and circuit calendars.

### Phase 6: Next-Gen Vehicle Dynamics & Pre-Baked Visual Kinematics (Priority: Medium)
- `[x]` **[Top-Down Pre-Baked Vehicle Wheel Steering Animations](../026_topdown_wheel_steering_animations.md)**: Multi-layer sprite decomposition and Ackermann wheel steering animation architecture for pre-baked high-resolution 2D top-down vehicles with proof-of-concept on classic_kart.
- `[x]` **[Decoupled Physical Tire Component & Per-Wheel Dynamics Architecture](../028_decoupled_tire_physics_and_wheel_components.md)**: Decouples vehicle tires into independent physical wheel assemblies with per-axle Pacejka curves, rotational inertia integration, wheel spin/lockup dynamics, and thermal wear modeling in wheelbase.
- `[x]` **[Modality-Realistic Vehicle Lighting Architecture](../031_modality_realistic_vehicle_lighting.md)**: Modality-governed vehicle lighting pipeline enforcing zero lights on Karts and NASCAR, full DRL/brake/projector illumination on GT and Rally, and multi-pod roof lightbars on Extreme Off-Road.
- `[x]` **[Kart Handling, Caster Jacking & Agile Turning Dynamics](../032_kart_handling_caster_jacking_and_agile_turning_dynamics.md)**: Authentic karting physics architecture introducing mechanical caster-jacking inside-rear wheel unloading, high-G sticky slick compound balance, extended 42° steering lock, and agile rear-biased weight distribution in wheelbase.
- `[x]` **[Cross-Modality Vehicle Turning Capabilities and Real-World Benchmark Analysis](../033_crossmodality_vehicle_turning_capabilities_and_benchmark_analysis.md)**: Comprehensive engineering analysis architecture evaluating turning circles, cornering limits, yaw agility, and understeer/oversteer dynamics across all 6 motorsport modalities, 25 performance tiers, and 85 vehicles compared with real-world counterparts.
- `[x]` **[Explicit Drivetrain Differential Models and Axle Coupling](../034_explicit_drivetrain_differential_models_and_axle_coupling.md)**: Explicit drivetrain differential models (Spool, LimitedSlip, Open), dynamic cross-axle torque distribution, and rotational velocity coupling across front, rear, and AWD powertrains.
- `[ ]` **[Automated Simulation Harness Parameter Optimization and Constrained Vehicle Calibration](../035_automated_simulation_harness_parameter_optimization_and_constrained_calibration.md)**: Algorithmic parameter optimization and constrained calibration architecture utilizing the deterministic headless simulation harness to systematically fit vehicle dynamics under physical drivetrain, chassis, and homologation constraints.


---

## 🛠 Technical Debt & Maintenance

Living list of security audits, performance profiling targets, or general workspace cleanup routines:
- **Performance**: Simulation throughput benchmarks ($\ge 4.0\text{M steps/sec}$) and collision checks ($\ge 22.0\text{M checks/sec}$).
- **Hygiene**: Spec-Driven Development alignment via `keel doctor` and `keel validate`.

---

## 🗄️ Combined Feature Backlog

Unscheduled explorations, long-term visions, and community feature requests (refer to [BACKLOG.md](../../BACKLOG.md) for complete idea registry):

### 1. Multiplayer Networking & Online Lobbies
- **Authoritative Relay & Netcode**: Client-server architecture with dead-reckoning prediction, delta compression, and rollback collision arbitration.
- **Matchmaking & Lobbies**: Dedicated room lobbies with synchronized car selection and spectator director slots.

### 2. Procedural Circuit Generation & Dynamic Grip
- **Terrain Synthesis**: Seed-based procedural track ribbon generation with elevation contours and surface hazards.
- **Dynamic Track Grip & Rubbering-In**: Racing line rubbers in over consecutive laps, while off-line sections collect marbles and lose grip.

### 3. Arcade Damage Modelling & Pitstop Repairs
- **Vehicle Durability & Health Bar**: 0–100% HP damage model from barrier and vehicle SAT collisions with progressive handling penalties (minor scrapes, engine smoke, limp-mode, spinout).
- **Tactical Pitlane Service**: Branching pit lane spline with rapid 2–3s arcade pitstop countdowns and floating repair popups.

### 4. Dedicated Stunt & Acrobatic Competition Mode
- **Acrobatic Rulesets**: Gymkhana precision drift slaloms, donut clipping zones, and freestyle arena trick attack with combo multipliers.

### 5. Ecosystem Spin-Off Titles
- **`tdbikes`**: 2-wheel lean angle physics, camber thrust, wheelies/stoppies, and highside/lowside crash states.
- **`tdopenworld`**: Tilemap NavMesh surface sampler, urban driving, and mission/heist mechanics powered by `wheelbase::Car`.

---

## ✅ Completed Milestones

Chronological log of past successfully deployed features and versions:
- `[x]` **[Modular Motorsport Architecture](../005_modular_racing_architecture.md)**: Extracted standalone crates `crates/wheelbase` and `crates/arcade-race-core` (2026-09-13).
- `[x]` **[Road Split & Branching Tracks](../006_road_split_and_branching_tracks.md)**: Implemented Directed Ribbon Graph (`TrackNetwork`), Joker Laps, and multi-layout tracks (2026-09-15).
- `[x]` **[Extreme Off-Road & Stunt Arenas Module](../007_extreme_offroad_and_stunt_arenas.md)**: 300 BHP Sand Rail Buggy, 15 venues, Mud/Snow physics, and open arenas (2026-09-17).
- `[x]` **[Race Modality Selection & Interface Flow](../008_modality_selection_and_menu_flow.md)**: Dedicated Single Player vs Multiplayer modality selection screen (2026-09-18).
