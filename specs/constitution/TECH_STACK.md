---
type: Tech Stack
title: "Tech Stack & Engineering Architecture"
description: "Approved technical stack, framework runtimes, multi-crate architecture, styling guidelines, and command enforcements."
status: active
tags: [tech-stack, architecture, rust, python, astro, macroquad, kira, sqlite]
---

# Tech Stack & Engineering Architecture 🛠️

The definitive technical specification for **TDRace**, codifying all approved programming languages, runtimes, multi-crate engine architectures, rendering engines, audio pipelines, persistence layers, styling guidelines, and command enforcements.

---

## 💻 Core Technology Matrix

TDRace is built upon a high-performance polyglot architecture consisting of three core pillars:
1. **Rust Core Simulation & Arcade Client**: Microsecond-latency deterministic physics, continuous collision detection, low-overhead 2D rendering, procedural audio, and local database persistence.
2. **Python Gymnasium Reinforcement Learning Subsystem**: High-throughput C-extension bindings, zero-copy NumPy observation buffers, and a pure-CPU software scanline rasterizer for scalable AI training.
3. **TypeScript / Bun / Astro Portals**: Dual-site technical documentation, interactive asset showrooms, and physics reference manuals adhering to Google Open Knowledge Format (OKF v0.2).

---

### 1. Rust Workspace & Engine Crates

The Rust codebase is organized as a Cargo workspace (resolver `2`, edition `2021`, minimum supported `rustc >= 1.80`, tested on `1.97.1`) consisting of 8 decoupled crates:

| Crate | Purpose & Scope | Key Dependencies | Strict Architectural Boundaries |
| :--- | :--- | :--- | :--- |
| [`crates/wheelbase`](../../crates/wheelbase) | Pure deterministic 2D/2.5D vehicle dynamics, Pacejka '96 tire solver, dynamic weight transfer (pitch squat/dive, lateral roll), chassis electronic assists (ABS, TC, counter-steer), explicit differential models (Open, Spool, LSD), jump ramp ballistics, and 12-surface split-$\mu$ sampling. | `glam 0.29`, `serde 1.0`, `serde_json 1.0` | **Zero graphics or audio**. Zero file I/O. Builds for any target including embedded and `wasm32-unknown-unknown`. |
| [`crates/arcade-race-core`](../../crates/arcade-race-core) | Course geometry, Catmull-Rom splines with continuous arc-length parameterization, continuous Separating Axis Theorem (SAT) OBB collision resolution, directional checkpoint gates, sector timing, and 32-beam racing LIDAR. | `wheelbase`, `glam 0.29`, `serde 1.0`, `serde_json 1.0` | **Zero graphics or audio**. Zero file I/O. Pure mathematical course and collision algorithms. |
| [`crates/race-kit`](../../crates/race-kit) | Headless race world orchestrator: fixed 60 Hz stepping order, finish order with real elapsed times, vehicle wrecks, damage, and DNF states, race event bus, and bot AI driver models with orthogonal styles and skill tiers. | `wheelbase`, `arcade-race-core`, `glam 0.29`, `serde 1.0` | **Headless only**. No window, graphics, sound, database, or system clock. Guaranteed `wasm32-unknown-unknown` compatibility. |
| [`crates/race-ui`](../../crates/race-ui) | Reusable 2D top-down racing drawing primitives: track ribbon and surface renderers, predictive follow camera (lookahead, speed-zoom, screen shake trauma), particle emitters (tire smoke, roost, sparks), ring-buffered skidmarks, and telemetry HUD overlays. | `wheelbase`, `arcade-race-core`, `cabinet`, `macroquad 0.4`, `glam 0.29`, `serde 1.0` | **Rendering primitives only**. Decoupled from specific game modes or persistence. |
| [`crates/cabinet`](../../crates/cabinet) | Opinionated 2D arcade game shell: resolution-independent `UiScaler` (1080p reference canvas), TrueType font loader, glassmorphism UI widgets (`Accordion`, `FilterBar`, `TabBar`, `SliderWidget`), 2D orthogonal navigation (`NavGrid2D`), modal screen stack, multi-bus audio mixer, and zero-config UDP LAN multiplayer discovery and lobby. | `macroquad 0.4`, `glam 0.29`, `serde 1.0`, `serde_json 1.0`, `gilrs 0.11`, `chrono 0.4`, `kira 0.12` (native) | **Game-agnostic shell**. Usable across any 2D arcade title. Native audio and gamepad features conditionally compiled. |
| [`crates/tdrace-core`](../../crates/tdrace-core) | Unified engine facade combining `wheelbase` and `arcade-race-core`. Provides backwards-compatible API surfaces and decompresses embedded official circuit presets using DEFLATE. | `wheelbase`, `arcade-race-core`, `glam 0.29`, `serde 1.0`, `serde_json 1.0`, `miniz_oxide 0.8` | Engine facade and embedded preset decompressor. |
| [`crates/tdrace-py`](../../crates/tdrace-py) | High-throughput PyO3 native C-extension (`_tdrace`). Exposes headless simulation stepping with zero-copy NumPy buffers and hosts an ultra-fast CPU software scanline rasterizer. | `tdrace-core`, `pyo3 0.23`, `numpy 0.23`, `glam 0.29`, `serde 1.0`, `rand 0.8` | **RL acceleration bridge**. Pure CPU rasterizer; requires zero GPU or display server (X11/Wayland). |
| [`crates/tdrace-app`](../../crates/tdrace-app) | Main playable arcade game binary across Desktop (Linux, macOS, Windows), Web (WASM), and Mobile. Integrates Macroquad game loop, Kira procedural audio, SQLite database persistence, in-game CAD Track Studio, and career mode runners. | `tdrace-core`, `race-kit`, `race-ui`, `cabinet`, `macroquad 0.4`, `gilrs 0.11`, `kira 0.12`, `rusqlite 0.32`, `chrono 0.4`, `toml 0.8` | **Application root**. Unites all engine layers into the complete standalone game product. |

---

### 2. Client, Graphics, Audio & Input Technologies

| Technology | Approved Version | Purpose & Architectural Role | Target Scope |
| :--- | :--- | :--- | :--- |
| **`macroquad`** | `0.4.13` | Cross-platform 2D graphics engine built on `miniquad`. Handles window creation, OpenGL 2.1/3.3/ES, WebGL 1.0/2.0, Metal rendering contexts, text rendering, and low-level input events. | Desktop, Web (WASM), Mobile |
| **`glam`** | `0.29.2` | High-performance, SIMD-aligned linear algebra library (`Vec2`, `Mat2`, `Quat`) with full `serde` serialization support. | All workspace crates |
| **`kira`** | `0.12.2` | Expressive procedural game audio library. Implements multi-cylinder engine sound synthesis, exhaust pops, Doppler pitch modulation, spatial proximity panning, and multi-bus mixing (`Master`, `Music`, `Sfx`, `Ui`). | Native desktop only (`cfg(not(target_arch = "wasm32"))`) |
| **`gilrs`** | `0.11.1` | Cross-platform gamepad input library supporting raw analog triggers, joysticks, D-pads, hot-plugging, and custom deadzones via `gamepad_profile.json`. | Native desktop only (`cfg(not(target_arch = "wasm32"))`) |
| **`rusqlite`** | `0.32.1` (`features = ["bundled"]`) | Embedded SQLite database engine for local Hall of Fame leaderboards, player profiles, race history telemetry, and career progression in `tdrace_records.db`. | Native desktop only (`cfg(not(target_arch = "wasm32"))`) |
| **`chrono`** | `0.4.40` | High-precision time and date handling, UTC timestamps, and race lap time string formatting. | Native desktop, headless |
| **`miniz_oxide`** | `0.8.5` | Pure Rust DEFLATE compression and decompression library. Used at build time to embed compressed official circuit catalogs into the binary. | `tdrace-core` |
| **`toml`** | `0.8.20` | TOML parser and serializer for declarative championship and tournament formats (`series/*.toml`) and global game settings (`config.toml`). | `tdrace-app` |
| **`serde` / `serde_json`** | `1.0.218` / `1.0.140` | Zero-allocation serialization and deserialization for JSON circuit definitions (`tracks/**/*.json`), LAN network datagrams, and telemetry export. | All workspace crates |

---

### 3. Python & Reinforcement Learning Subsystem

| Technology | Approved Version | Purpose & Architectural Role | Target Scope |
| :--- | :--- | :--- | :--- |
| **Python** | `>= 3.10` (active `.venv` on `3.13.13`) | Primary runtime for reinforcement learning, training harnesses, benchmark scripts, and asset generation pipelines. | RL, Tooling |
| **`pyo3`** | `0.23.5` | Rust-to-Python FFI binding generator. Exposes `PyEngine` and `RewardConfig` as the high-speed native C-extension module `tdrace._tdrace`. | `crates/tdrace-py` |
| **`maturin`** | `>= 1.5, < 2.0` | PEP 517 build backend and packaging tool for building and developing PyO3 native extensions into the active virtual environment. | Build system |
| **`gymnasium`** | `>= 1.0.0` | Standard Reinforcement Learning API specification. Registers official environments: `TDRace-v0`, `Continuous`, `Discrete`, `Drift`, `Pixels`, and `MultiAgent`. | `python/tdrace` |
| **`numpy`** | `>= 1.24.0` | Zero-copy vector and image tensor observation array bridge between Rust and Python. | `python/tdrace` |
| **Software Scanline Rasterizer** | In-House Pure Rust | Ultra-lightweight CPU scanline rasterizer in `tdrace-py` producing $(96, 96, 3)$ RGB observations at **11,000+ FPS** without GPU, OpenGL, or X11/Wayland dependencies. | RL training |
| **`uv`** | `>= 0.11` | Ultra-fast Python package installer, environment manager, and virtualenv orchestrator (`.venv`). | Development workflow |
| **`pytest`** | `>= 8.0` | Test suite runner for Gymnasium API compliance, adversarial action space boundaries, and multi-agent determinism tests (`tests/python`). | Quality assurance |

---

### 4. Web Portals & Documentation Stack

| Technology | Approved Version | Purpose & Architectural Role | Target Scope |
| :--- | :--- | :--- | :--- |
| **`bun`** | `1.3.14+` | All-in-one JavaScript/TypeScript runtime, bundler, and workspace package manager powering the documentation portals. | `portals/` |
| **`astro`** | `5.4.2` | High-performance content-driven static site generator with Island Architecture. Powers both technical portals. | `portals/` |
| **`@astrojs/starlight`** | `0.32.0` | Documentation framework powering **Option A: Technical Reference Manual** (`portals/option-a-starlight`) on port 4321. | Wiki & Specs |
| **`tailwindcss`** | `3.4.17` (`@astrojs/tailwind 5.1.5`) | Utility-first CSS framework styling **Option B: Motorsport Showroom & Physics Lab** (`portals/showroom`) on port 4322. | Showroom portal |
| **`katex`** | `0.16.21` (`remark-math 6.0`, `rehype-katex 7.0`) | High-speed mathematical rendering for Pacejka tire curves, weight transfer formulas, and Balance of Performance (BoP) equations. | Portals |
| **`zod`** | `3.24.2` | TypeScript schema declaration and validation library for circuit datasets and vehicle specifications. | Portals shared data |

---

### 5. Networking & Local LAN Protocol

| Parameter | Specification | Details |
| :--- | :--- | :--- |
| **Transport Protocol** | UDP Datagrams | Raw `std::net::UdpSocket` for minimal latency without TCP head-of-line blocking. |
| **LAN Beacon Discovery** | UDP Port `34567` | Host broadcasts `LanBeacon` packets at 2 Hz; clients scan subnet with `LanBeaconScanner`. |
| **Lobby & Game Port** | UDP Port `34568` | Direct UDP socket connection for lobby slot assignment, car selection, and ready checks. |
| **State Synchronization** | Owner-Authoritative | Each player computes and transmits their own `NetCarState` (position, velocity, angle, steering, yaw rate). Host acts as session clock referee and relay. |
| **Packet Formats** | Compact Binary | `Packet` enum with fixed magic bytes `[0x54, 0x44, 0x52, 0x4E]` (`TDRN`), versioned protocol, and CRC validation. |
| **Smoothing & Compensation** | Dead-Reckoning & LERP | Remote opponent positions are smoothly interpolated using linear and spherical interpolation with velocity extrapolation. |

---

### 6. Persistence & File Formats

- **SQLite Database (`tdrace_records.db`)**: Local relational database with auto-migrated schemas:
  - `hall_of_fame`: High-score track lap records and overall times indexed by `(track_id, total_time ASC)`.
  - `player_profiles`: Multi-slot player identities, aliases, country codes, custom color schemes, credits, and lifetime earnings.
  - `race_history`: Per-race telemetry logs, finish positions, stunt points, collisions, and lap times.
  - `profile_module_progress`: Per-discipline career XP, levels, unlocked cars, visited tracks, and trophy counts.
  - `profile_championship_awards`: Career championship standings, podium positions, and points.
- **Circuit Geometry Catalog (`tracks/**/*.json`)**: Single-source-of-truth JSON format containing Catmull-Rom spline waypoints, width attributes, jump ramps, hazard surface zones, barriers, and sector timing gates.
- **Championship Season Presets (`series/**/*.toml`)**: Declarative TOML format specifying series tiers, scoring matrices (FIA, NASCAR, Classic), circuit calendars, and vehicle eligibility criteria.
- **Binary Replay Stream (`.tdr`)**: Deterministic 60 Hz vehicle telemetry recording format supporting variable-rate playback ($1\times$ to $8\times$), timeline scrubbing, and ghost car benchmarking.
- **Controller Mappings (`gamepad_profile.json`)**: Configurable button assignments, trigger sensitivity, and stick deadzones for `gilrs`.

---

### 7. Target Runtimes & Cross-Compilation Matrix

| Platform Target | Architecture | Compilation Toolchain | Output Artifact |
| :--- | :--- | :--- | :--- |
| **Linux Desktop** | `x86_64-unknown-linux-gnu` | `cargo build --release -p tdrace-app` | Native ELF executable (`target/release/tdrace-app`) |
| **Windows Desktop** | `x86_64-pc-windows-gnu` / `msvc` | `./scripts/package_windows.sh release` | Standalone zip bundle with `tdrace-app.exe`, assets, tracks, and launcher |
| **WebAssembly** | `wasm32-unknown-unknown` | `./web/build_web.sh` (wasm-bindgen) | 660KB WASM bundle served via HTML5 canvas shell (`web/dist/`) |
| **Android Mobile** | `arm64-v8a`, `x86_64` | `./mobile/android/build_android.sh` (NDK + Gradle) | Android APK with native activity and touch controls |
| **iOS Mobile** | `aarch64-apple-ios`, `x86_64-apple-ios` | `./mobile/ios/build_ios.sh` (Xcode) | Universal static framework for iOS Device and Simulator |

---

## 🎨 Design & Styling Principles (Cabinet Arcade System)

The user interface and visual presentation follow the **Cabinet Arcade System**, an interaction design system optimized for high-DPI displays, retro arcade authenticity, and full controller navigation.

### 1. Resolution-Independent Virtual Canvas (`UiScaler`)
- **Reference Resolution**: Fixed 1920×1080 (16:9) virtual design coordinates.
- **Aspect Ratio Preservation**: Automatic pillarboxing and letterboxing preventing UI stretching across arbitrary physical monitor aspect ratios (21:9 ultra-wide, 16:10, 4:3).
- **Scale Profiles**: User-configurable scaling modes (`Auto`, `1080p`, `720p`, `4k`) managed through `DisplayResolution` in `crates/cabinet/src/ui/scaler.rs`.

### 2. Dark Arcade Color Tokens

```
Obsidian Deep Background: #0A0F1D (0x0A0F1D)   -- Main canvas background
Panel Surface Elevation:  #161F30 (0x161F30)   -- Dialog cards, sidebars, modal bodies
Elevated Card Container:  #1E2B42 (0x1E2B42)   -- Track/car cards, roster item slots
Structural Border Rim:    #2A3B5C (0x2A3B5C)   -- Panel dividers, input borders
Focus Highlight Rim:      #3D547F (0x3D547F)   -- Hover and inactive focus borders

Electric Cyan (Action):   #00E5FF (0x00E5FF)   -- Active selection focus, primary buttons, tachometer
Neon Amber (Reward):      #FFB300 (0xFFB300)   -- Trophies, warnings, drift multiplier points
Velocity Red (Alert):     #FF3D00 (0xFF3D00)   -- Braking markers, redline revs, collision alerts
Apex Green (Success):     #00E676 (0x00E676)   -- Checkpoints, start lights, positive split times
Nitro Purple (Special):   #AA00FF (0xAA00FF)   -- Boost indicators, experimental race modes

Text Primary White:       #FFFFFF (0xFFFFFF)   -- Screen titles, primary data values
Text Secondary Slate:     #94A3B8 (0x94A3B8)   -- Field labels, descriptions, sub-headings
Text Muted Caption:       #64748B (0x64748B)   -- Inactive options, breadcrumbs, copyright text
```

### 3. Typography & Numerical Layouts
- **Proportional Sans-Serif**: Clean modern typography for headers, screen titles, category selectors, and modal prompts.
- **Monospaced Tabular Numerals**: Strict requirement for lap times (`01:23.456`), sector delta splits (`+0.142s`), speedometers (`284 km/h`), and RPM gauges to eliminate character-width jittering during high-speed updates.

### 4. Retro Arcade Juice & FX Primitives
- **Dynamic CRT Scanlines**: GPU shader overlay with selectable density (`Disabled`, `Light`, `Medium`, `Heavy`) and subtle edge vignette darkening.
- **Trauma-Decay Screen Shake**: Non-linear camera trauma decay ($\text{shake} = \text{trauma}^2 \times \text{max\_offset}$) triggered by wall collisions, barrier scrapes, and jump ramp landings.
- **Directional Particle Emitters**: Pre-allocated ring buffers generating dynamic tire smoke puffs, gravel roost pebbles, sand dust clouds, and metallic collision sparks.
- **4-Wheel Skidmark Buffers**: Continuous circular vertex buffers laying persistent rubber skidmarks on tarmac and surface ruts on dirt and mud.
- **Floating Combat Text**: Animated floating text overlays for stunt points, sector delta times, and wrong-way indicators.

### 5. Input Paradigms & Controller Navigation
- **2D Orthogonal Spatial Navigation (`NavGrid2D`)**: All interactive screens, tabs, accordions, and dialogs are traversed orthogonally via D-Pad, arrow keys, or analog stick with zero mouse requirement.
- **Full Controller Parity**: Every feature, option, track selection, and garage tuning menu must be 100% operable using a standard gamepad.
- **Digital Keyboard Steering Smoothing (`DigitalInputFilter`)**: Progressive input filtering with 5 selectable profiles:
  1. *Balanced* (140 ms rise time, 1.0 authority, 1.3 center precision)
  2. *Smooth* (200 ms rise time, progressive hold bleed for high-speed stability)
  3. *Agile* (90 ms rise time, high initial snap for tight chicanes)
  4. *Direct* (50 ms rise time, raw responsive control)
  5. *Raw* (0 ms instantaneous digital input for purists)

---

## 🚀 Execution & Command Enforcements

Developers and AI coding agents must use the following codified commands:

### 1. Game Launch & Execution

```bash
# Launch default game in release mode (Classic Grand Prix)
make run

# Launch game with specific motorsport module directly
make run-gt         # GT World Challenge & Endurance Road Courses
make run-nascar     # NASCAR Cup Series & Oval Speedways
make run-rally      # Rallycross World Cup & Mixed-Terrain Stages
make run-kart       # Karting World Cup & Shifter Karts
make run-classic    # Classic Heritage Arcade Tracks

# Launch in debug mode with backtraces enabled
make dev            # or: make run-dev

# Pass custom arguments through Make
make run -- --track spa_francorchamps --car gt3_evo
```

### 2. Rust Workspace Compilation & Testing

```bash
# Build entire workspace in debug mode
cargo build --workspace

# Build entire workspace in optimized release mode (LTO enabled)
cargo build --workspace --release

# Run Rust unit, integration, and physics tests across all crates (excluding PyO3)
make test-rust
# or: cargo test --workspace --exclude tdrace-py

# Run targeted benchmark suites (Wheelbase physics & Arcade SAT collisions)
make bench-rust
# or: cargo bench -p wheelbase -p arcade-race-core

# Run workspace-wide clippy linter with pedantic checks
cargo clippy --workspace --all-targets -- -D warnings
```

### 3. Python & Gymnasium RL Environment

```bash
# Initialize virtualenv (.venv) and compile PyO3 extension with maturin
make setup-python

# Run Gymnasium compliance and determinism tests (54 tests)
make test-python
# or: .venv/bin/pytest tests/python

# Run Gymnasium throughput benchmarks vs CarRacing-v3
make bench-python
# or: .venv/bin/python benchmarks/gym_benchmark.py
```

### 4. Cross-Platform Builds & Packaging

```bash
# Compile and serve WebAssembly build locally at http://localhost:8080
make serve-web

# Build standalone Windows zip distribution with assets and launcher scripts
make package-windows

# Build Android APK and native libraries
make build-android

# Build iOS static framework
make build-ios
```

### 5. Documentation & Technical Portals

```bash
# Launch Option A: Astro + Starlight Technical Reference Manual (http://localhost:4321)
make wiki

# Launch Option B: Custom Motorsport Showroom & Physics Lab (http://localhost:4322)
make showroom

# Re-ingest track geometry and vehicle specs into JSON portal datasets
make ingest-assets

# Verify Open Knowledge Framework (OKF v0.2) link integrity and cross-references
make verify-okf

# Rebuild both static web portals for production deployment
make build-portals
```

### 6. SDD Governance & Task Tracking

```bash
# Run comprehensive Keel diagnostic health check
keel doctor

# Validate Keel specifications, frontmatter, and acceptance criteria
keel validate

# Show next prioritized unblocked task via Beads
bvr --robot-next --format toon

# Claim a Beads task
br update <issue-id> --claim

# Record durable verification receipt for a completed specification
keel receipt <spec-number> --cmd "<test-command>"
```

---

## 🛡️ Coding Laws & Architectural Invariants

1. **Fixed-Timestep Simulation Invariant ($dt = 1/60\text{ s}$)**:
   - All vehicle integration, Pacejka slip calculations, weight transfer, SAT collision impulses, and raycasting must execute strictly at $60\text{ Hz}$.
   - Never use variable render delta times (`get_frame_time()`) inside simulation or physics code.
2. **Zero-Graphics Boundary on Core Physics**:
   - `crates/wheelbase` and `crates/arcade-race-core` must remain zero-dependency mathematical libraries. Never import `macroquad`, `kira`, or windowing symbols into these crates.
3. **Headless WebAssembly Compatibility**:
   - `crates/race-kit` must compile cleanly for `wasm32-unknown-unknown` without audio, windowing, or database features.
4. **Conditional Compilation for Native Features**:
   - Any dependency on `kira` or `rusqlite` must be gated behind `#[cfg(not(target_arch = "wasm32"))]` to preserve WebAssembly target builds.
5. **Zero Heap Allocations in Hot Simulation Loops**:
   - Collision detection ticks, Pacejka calculations, and raycast queries must operate with zero dynamic heap allocations per frame. Pre-allocate scratch buffers, use stack arrays, or utilize ring buffers.
6. **Bit-Identical Determinism**:
   - Given an identical initial state, seed, and input sequence, simulation rollouts must produce bit-identical telemetry outputs across runs.
7. **Strict Schema Validation & Serialization Safety**:
   - All external JSON and TOML structures must parse into strongly-typed Rust structs with `serde`. Never use untyped dynamic mappings (`serde_json::Value`) in production gameplay code.
8. **Dependency Auditing**:
   - Never add third-party crates or npm packages without an approved specification and explicit architectural justification.
