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

### 2.5 Vehicle Roster & Archetype Expansions (Buggies, Trucks, Superkarts, Novelty Specials)
- **Detailed Design Specification**: See [`docs/vehicle_roster_expansion_ideas.md`](file:///home/mario/workspace/games/tdrace/docs/vehicle_roster_expansion_ideas.md) for full physics levers, surface interactions, visual archetypes, and parameter tables.
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
- **Implementation Status**: TBD selection of prioritized vehicles for future milestone integration into `crates/wheelbase` and `crates/tdrace-app`.

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

