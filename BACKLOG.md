---
type: Feature Backlog
title: "TdRace & Cabinet Platform — Project Backlog"
description: "Consolidated idea registry for pre-development features and speculative enhancements."
status: active
---

# TdRace & Cabinet Platform — Project Backlog

This backlog records prospective features, architectural improvements, and future milestone ideas across the **TdRace** game and the **Cabinet** arcade platform crate.

---

## 1. `cabinet` Arcade Platform: Standalone Readiness & Packaging

The `cabinet` crate (`crates/cabinet`) is now an opinionated, batteries-included 2D arcade game shell (resolution scaling, retro typography, themes, orthogonal 2D navigation, Juice FX, CRT post-processing, screen transitions, action-based input mapper, zero-dependency DSP, and Kira audio backend).

### 1.1 Documentation & Publishing Prep
- **Developer Guide & Architecture Overview**: Create a comprehensive `crates/cabinet/README.md` detailing crate architecture, initialization flow, screen lifecycle, and integration examples.
- **Cargo.toml Publishing Metadata**: Add crate categories, keywords (`game-dev`, `arcade`, `retro`, `macroquad`, `kira`), repository link, documentation badges, and verify crates.io packaging readiness.
- **WebAssembly Build Pipeline**: Standalone WebAssembly build script and demo harness (`web/build_cabinet_demos.sh`) to preview Cabinet examples in-browser.

### 1.2 Multi-Genre Prototypes
- **Second Prototype Mini-Game (`brick_breaker.rs` or `topdown_shooter.rs`)**:
  - Implement a second example alongside `space_arena.rs` in `crates/cabinet/examples/`.
  - Validate that screen stack, juice FX, floating text, CRT shaders, and audio synthesis generalize seamlessly across genres without racing-specific assumptions.

---

## 2. Core Racing, Tracks & Physics Enhancements (`tdrace-app` & `tdrace-core`)

### 2.1 Track Roster & OSM Circuit Builder
- **New Real-World Circuits**: Ingest OpenStreetMap track survey data and FIA homologation dimensions for iconic tracks (e.g. Nürburgring GP, Spa-Francorchamps, Yas Marina, Brands Hatch).
- **Rallycross Stadium Venues**: Design dedicated mixed-surface arenas featuring gravel switchbacks, jump ramps, and split joker laps.
- **Road Split Segments & Branching Splines (Alternative Circuits & Joker Laps)**: See detailed specification in [`docs/spec_road_split_and_branching_tracks.md`](docs/spec_road_split_and_branching_tracks.md) for topological Directed Ribbon Graph (`TrackNetwork`), $C^1$ spline stitching at branch sockets, multi-route progress tracking, and editor branch extension workflows.
- **Dynamic Track Grip & Rubbering-In**: Surface grip evolution where the racing line rubbers in over laps, while off-line sections collect marbles and lose traction.

### 2.2 Advanced AI Opponents
- **Dynamic Overtaking & Slipstream Drafting**:
  - Teach AI cars to utilize aerodynamic slipstreams and execute late-braking moves down straights.
  - Implement defensive positioning lines when an opponent is alongside.
- **Dynamic Difficulty Adjustment (DDA)**: Adaptive rubber-banding or skill-tier tuning based on player pace to keep arcade races competitive.
- **Grid AI Personalities**: Driver traits with varying aggression, apex discipline, and wet/loose surface confidence.

### 2.3 Championship & Career Mode
- **Multi-Race Championship Cups**: Structured tournament ladders (e.g. 4-race series) with FIA-style points systems (25-18-15-12...), podium sequences, and season trophies.
- **Persistent Career Milestones**: Wire career statistics into `cabinet::profile::PlayerProfile` and unlockable liveries/car badges.

### 2.4 Ghost Car & Time Trial Challenges
- **Compact Telemetry Recording**: Store player hot-lap waypoints, velocity vectors, and steering angles as lightweight JSON/binary recordings.
- **Ghost Car Playback**: Render a semi-transparent ghost vehicle during Time Trial sessions to assist with apex optimization and racing line analysis.

### 2.5 Vehicle Roster & Archetype Expansions (Real-World Migration & Interactive Garage)
- **Real-World Models & Interactive Garage Specification**: See [`docs/spec_real_world_car_models_and_garage.md`](docs/spec_real_world_car_models_and_garage.md) for the complete roadmap migrating prototypical cars to authentic models across all 11 categories (Porsche Cayman GT4 / 911 GT3 R, BMW M4 GT4 / GT3, Ferrari 296 GT3, etc.), Balance of Performance (BoP) calibration, dual-view graphics pipeline (accurate top-down + high-detail 2D lateral view), and the Interactive Garage Showroom.
- **Novelty Archetypes & Experimental Classes**: See [`docs/vehicle_roster_expansion_ideas.md`](docs/vehicle_roster_expansion_ideas.md) for full physics levers, surface interactions, visual archetypes, and parameter tables.
- **Off-Road & Bashing**:
  - *Baja Sand Buggy*: Lightweight rear-engine RWD buggy with dirt/sand immunity and high jump compliance.
  - *Stadium Super Truck (SST)*: Long-travel suspension truck cornering aggressively on 3 wheels with bouncy ramp landings.
  - *Monster Truck ("Crusher V8")*: High-mass 4WD giant with oversized tires bulldozing straight through grass/gravel shortcuts and pushing opponents aside.
- **Kart & Micro-Racer Variants**:
  - *250cc Superkart*: Aerodynamic pod fairings, 240+ km/h top speed, and razor-sharp 3.5G lateral grip for full-sized GP circuits.
  - *Racing Lawnmower*: Narrow-track, high-CG single-cylinder tractor with lift-off oversteer and bumpy curb hopping.
  - *Drift Trike / Slick Kart*: Low-friction rear slide rings for effortless continuous pendulum drifting and 360° entries.
- **Sport, GT & Classic Variants**:
  - *Trackday Ultralight*: Featherweight exoskeleton (~520 kg) prioritizing mechanical grip and trail-braking agility.
  - *Group 5 "Super Silhouette" Turbo*: Late 70s DRM/IMSA monster with extreme box flares, turbo boost lag, and overrun flames.
  - *Electric Hypercar (AWD)*: Instant 0-100 km/h acceleration (<1.9s) with heavy battery mass requiring disciplined braking points.
  - *Classic Muscle Cruiser*: 427 Big Block V8 with dramatic pitch/squat weight transfer and lazy, controllable power slides.
- **Novelty & Party Specials**:
  - *Tuned Kei Micro-Van*: High-CG body roll, front-heavy brake dive, and slipstream drafting dependence.
  - *European Racing Super Truck*: 5-ton 1200 BHP cab-over semi-truck acting as an unstoppable moving fortress.
- **Implementation Status**: Specification completed in [`docs/spec_real_world_car_models_and_garage.md`](docs/spec_real_world_car_models_and_garage.md); scheduled for phased catalog, rendering, and garage screen implementation.

### 2.6 Arcade Damage Modelling & Pitstop Repairs
- **Vehicle Durability & Health Bar**:
  - Track vehicle structural integrity (0–100% HP) in simulation state (`wheelbase` / `tdrace-app`).
  - Damage calculated from normal impact velocities ($v_{\text{rel}} \cdot \mathbf{n}$) against course barriers and during car-to-car SAT collisions.
- **Progressive Arcade Degradation**:
  - *Minor Damage (>25%)*: Visual collision sparks, surface scrapes, and subtle chassis rattling audio.
  - *Moderate Damage (>50%)*: Light exhaust/engine smoke particles, subtle top-speed handicap (-10%), and slight steering pull.
  - *Critical Damage (>75%)*: Dark engine smoke and flame particles, flashing red HUD warning alarm, and limp-mode acceleration penalty (-25%).
  - *Totalled / Wrecked (100%)*: Exaggerated arcade spinout / explosion into debris, followed by a short respawn countdown and fresh car reset.
- **Pit Lane & Pitstop Repairs**:
  - Dedicated branching pit lane spline or designated drive-through service bay along the start/finish straight.
  - Rapid arcade pit stop sequence: coming to a halt in the pit box triggers a 2–3 second pit crew service countdown with air-ratchet wrench audio, welding sparkle bursts, and floating "+REPAIRED" juice popups.
  - High-stakes tactical trade-off: pitting costs ~3–5s track position delta versus racing with degraded top-speed or risking a full wreck penalty.
  - AI awareness: AI drivers monitor their health telemetry and autonomously divert into the pit lane when damage exceeds a critical threshold (>65%).

### 2.7 Arcade Power-Ups & Track Pickups
- **Track Spawners & Pickups**:
  - Rotating holographic 2D crates / glowing pads placed along track ribbons with timed respawn intervals.
  - Game mode ruleset toggle: selectable between "Pure Racing" and "Arcade Action / Battle" modes.
- **Power-Up Roster**:
  - *Boost / Nitro Surge*: Immediate forward propulsion burst with blue exhaust flames, camera FOV punch, screen shake, and top-speed overrun.
  - *Kinetic Shield*: Temporary energy bubble deflecting opponent rams, wall impacts, and hazard effects.
  - *Oil Slick / Hazard Drop*: Deployed behind vehicle to trigger immediate zero-friction spinout for trailing opponents.
  - *Pulse Blast / EMP Wave*: Radial shockwave knocking surrounding vehicles outward and briefly cutting throttle/steering.
  - *Field Repair Wrench*: Emergency instant repair (+50% HP) on the fly without entering the pit lane.
  - *Super Grip (Sticky Tires)*: Temporary traction multiplier granting immunity to off-track grass/gravel slowdowns.
- **HUD, Input & Audio Wiring**:
  - Item inventory slot on HUD featuring a roulette roll animation upon box collection.
  - Dedicated trigger action mapped through `cabinet::input::InputMap` (Keyboard Space, Gamepad B/Right Trigger, Mobile touch button).

### 2.8 Networked Multiplayer Racing
- **Client-Server Architecture**: Dedicated lightweight authoritative game relay / room host using UDP or WebSockets (with WebRTC data channels for browser builds).
- **Lobby & Matchmaking System**:
  - Module-specific open and private lobby rooms with customizable rulesets (laps, collision modes, vehicle restrictions).
  - Synchronized car and livery selection phase before green light countdown.
  - Spectator camera slots and race director replay feeds.
- **State Synchronization & Netcode**:
  - High-frequency delta compression of vehicle telemetry (position, velocity, orientation, steering, throttle/brake inputs).
  - Client-side prediction and dead-reckoning interpolation to mask latency spikes.
  - Server-arbitrated collision resolution with rollback compensation to prevent rubber-banding on door-to-door passes.

### 2.9 Classic Arcade Tournaments & Retro Career Ladder
- **Heritage Arcade Ladder**: Dedicated tournament cup system for the all-in-one Classic Game Module.
- **Multi-Class Grand Prix**: Progression series spanning Grassroots Clubman, Tuning Drift Spec, 125cc Karting, and Supercar Pro Tour.
- **Retro High-Score Coin-Op Style**: Timed checkpoint stages with classic timer countdown ("Time Extended!") and nostalgic leaderboard ceremonies.

---

## 3. Audio & Soundtrack Expansions

### 3.1 Dynamic Procedural Music Stems
- **Interactive Music Layering**: Adapt the procedural synthwave engine (`synthwave.rs`) to dynamically add or drop musical stems (e.g. heavy drums, arpeggio leads, bass drops) based on race state (final lap, high-speed drafting, pit stops, or low countdown).
- **Multiple Procedural Music Tracks**: Introduce 3 distinct 80s outrun / synthwave procedural compositions (e.g. "Neon Velocity", "Midnight Overpass", "Cyber Circuit").

### 3.2 Spatial & Environmental Audio
- **3D / Panned Spatial Audio**: Stereo panning and distance attenuation for nearby opponent engines, tire skids, and wall scrapes relative to player car position.
- **Doppler Effect**: Pitch-shifting on fast-closing and receding opponent vehicles during high-speed passes.
- **Surface-Specific Audio Textures**: Procedurally synthesize gravel spray, puddle splashing, and curb rumble thuds.

---

## 4. Active Tracking & Current Milestone

The current active milestone is tracked in Beads:
- **Epic**: `tdrace-downstream-cabinet-integration-uqq` (Downstream Cabinet Platform Integration in `tdrace-app`)
  - `tdrace-downstream-cabinet-integration-uqq.1`: Rebindable Racing Controls with `cabinet::input::InputMap`
  - `tdrace-downstream-cabinet-integration-uqq.2`: Arcade Screen Transitions across Game Menus and Racing View
  - `tdrace-downstream-cabinet-integration-uqq.3`: CRT & Retro Scanline Post-Processing in Racing Viewport
  - `tdrace-downstream-cabinet-integration-uqq.4`: Floating Split-Time and Combo Popups in Racing HUD

---

## 5. Ecosystem Spin-Off Titles (Powered by `wheelbase` & `arcade-race-core`)

### 5.1 `tdbikes` (Top-Down 2D Superbike & Motocross Racer)
- **Single-Track Physics Engine**: 2-wheel vehicle dynamics with dynamic lean angle kinematics ($\tan \phi = \frac{v^2}{Rg}$) and camber thrust from rounded tire cross-sections.
- **Pitch Dynamics (Wheelies & Stoppies)**: Full front-wheel lift under heavy acceleration and rear-wheel lift under hard braking.
- **Crash Mechanics**: Dynamic lowside (loss of grip while banked) and highside (violent traction regain snap) crash state machines.
- **Ecosystem Reuse**: Powered by `wheelbase::Motorbike` + `arcade-race-core` (Catmull-Rom track splines, directional timing gates, SAT collisions) + `cabinet` (arcade shell, UI, and audio).

### 5.2 `tdopenworld` (Top-Down 2D Urban Vehicular / Heist Action)
- **Tilemap Surface Sampler**: Decouple vehicle physics from circuits by implementing `SurfaceSampler` over 2D tilemaps / NavMeshes (streets, alleys, sidewalks, grassy parks, construction sand).
- **Urban Driving Mechanics**: Responsive arcade drifting, 180° handbrake reverse flips, vehicle-to-vehicle ramming, and destructible prop collisions.
- **Ecosystem Reuse**: Powered by `wheelbase::Car` (Pacejka 4-wheel chassis, weight transfer, assists) + `cabinet` (arcade shell, UI, gamepad mapper), completely free of circuit spline constraints.

