---
type: Project Constitution
title: "Project Mission & Agent Constitution"
description: "Constitutional governance, coding laws, and architectural boundaries for TDRace."
status: active
tags: [constitution, governance, mission, architecture]
---

# Project Mission & Agent Constitution 📜

The primary constitutional document for **TDRace**, establishing the project mission, target audience, core operational laws, and architectural boundaries for developers and AI coding agents.

---

## 🎯 Primary Mission

**TDRace** is a high-performance, deterministic top-down 2D/2.5D arcade motorsport racing game, reusable racing engine platform, and ultra-fast Gymnasium Reinforcement Learning environment. Inspired by seminal arcade classics such as *Super Sprint* (1986), *Slicks 'N Slide* (1993), and *GeneRally* (2002), TDRace fuses nostalgic 2.5D arcade aesthetics with modern vehicle dynamics (Pacejka tire model, dynamic weight transfer, split-$\mu$ surface friction, and explicit differential coupling). The project is engineered around three foundational pillars:

1. **Human Competitive Arcade Motorsport**: Deliver an accessible yet mechanically deep racing game featuring responsive handling, authentic Balance of Performance (BoP) vehicle calibration across 25 categories and 6 disciplines (GT/Endurance, NASCAR Stock Car, Rallycross, Extreme Off-Road, Karting, and Autocross), an interactive in-game CAD Track Studio with 60 Hz instant playtesting, comprehensive career progression with XP and credits economies, local LAN multiplayer, full gamepad/keyboard parity with progressive steering filters, and tactile retro arcade juice (CRT scanlines, particle roost, skidmarks, and procedural motor acoustics with Doppler shift).
2. **High-Throughput Reinforcement Learning & Simulation**: Provide a lightning-fast, deterministic simulation benchmark that shatters traditional RL bottleneck limits. TDRace delivers **4,000,000+ steps/second** in pure Rust, **64,000+ steps/second** in Python vectorized Gymnasium environments (over **180× faster** than Box2D `CarRacing-v3`), and **11,000+ FPS** RGB pixel observations via an ultra-lightweight CPU software scanline rasterizer that operates with zero GPU or display server (X11/Wayland) dependencies.
3. **Modular Reusable Racing Platform**: Serve as an unbundled, publish-ready multi-crate platform (`wheelbase`, `arcade-race-core`, `race-kit`, `race-ui`, `cabinet`) designed to power multiple distinct racing titles (such as the ancient chariot racing game `tdchariots`, point-to-point rally raid stages, and two-wheeled motorsport engines) from a shared deterministic foundation.

### Target Audience
- **Arcade Racing Gamers & Hotlappers**: Players seeking immediate, responsive pick-up-and-play racing with deep cornering physics, time-trial shadow cars, and diverse career disciplines.
- **Circuit Designers & Modders**: Creators utilizing the in-game vector CAD Track Studio to design, validate, and share custom circuits.
- **Machine Learning & Robotics Researchers**: AI practitioners training autonomous driving agents, multi-agent competitive policies, and vision-based RL models at datacenter scale without GPU rendering bottlenecks.
- **Motorsport Game Developers**: Engineers seeking clean, modular, zero-dependency Rust crates for deterministic 2D vehicle dynamics, continuous SAT collision resolution, and arcade UI shell systems.

### Core Problems Resolved
- **Slow, GPU-Bound RL Benchmarks**: Eliminates the heavy CPU/GPU overhead of legacy environments by implementing headless Rust stepping and CPU software rasterization.
- **Floaty, Non-Physical Arcade Physics**: Replaces simplistic kinematic vehicle motion with a deterministic 60 Hz 4-wheel Pacejka '96 slip model, dynamic load transfer (acceleration squat, braking dive, lateral roll), and 12-surface split-$\mu$ traction matrices.
- **Monolithic Game Codebases**: Avoids monolithic bloat by enforcing a clean, layered multi-crate boundary where physics and course geometry have zero dependencies on rendering or audio engines.

### Expected Standards of Polish
- **Bit-Identical 60 Hz Determinism**: Strict fixed-timestep ($dt = 1/60\text{ s}$) stepping ensuring identical replays, ghost cars, and reproducible RL rollouts across runs.
- **Instant Responsiveness & High Frame Rates**: Rock-solid 60 FPS performance across desktop Linux, macOS, Windows, WebAssembly browsers, and mobile devices.
- **Holistic Audio-Visual Juice**: Procedural multi-cylinder engine sound synthesis with dynamic spatial Doppler shifts, reactive camera lookahead with speed-zoom and screen shake trauma, and crisp resolution-independent typography.

---

## 📜 Core Laws of Development

All developers and AI coding agents must unconditionally adhere to the following principles:

### 1. Hybrid Pragmatic Keel Methodology (Spec-Driven Development)
- **Major Features & Architectural Changes**: Must be plan-driven. A formal specification (or modification to an existing spec) must be drafted in `specs/` and approved before code changes.
- **Minor Fixes & Local Adjustments**: Bug fixes, style tweaks, doc typos, and local test additions can be updated directly in code without requiring a formal spec update.
- **Strict Major Boundaries**: A formal specification MUST be created or modified if a change:
  1. Adds, renames, or removes a database field, collection, or SQLite schema.
  2. Modifies an engine API or FFI signature (query parameters, Rust crate public traits, or PyO3 export methods).
  3. Adds or removes an external dependency, crate, or library.
  4. Aligns with a milestone checklist item in `ROADMAP.md`.
- **File Naming & Casing Standard**:
  - **UPPERCASE** is strictly used for high-priority constitutional and policy sheets (e.g., `MISSION.md`, `TECH_STACK.md`, `ROADMAP.md`, `BACKLOG.md`).
  - **lowercase_snake_case** with 3-digit prefix is strictly used for active, numbered specifications (e.g., `001_nascar_career_mode.md`, `056_racekit_headless_race_world.md`).
- **Issue Tracking via Beads (`br` / `bvr`)**: Engineering tasks are decomposed into discrete, tracked units in `.beads/` rather than loose markdown files. Use `bvr --robot-next --format toon` to rank work and `br` to claim and close issues. Commits implementing tasks should follow [Conventional Commits](https://www.conventionalcommits.org/) format with the associated task ID suffix (e.g., `feat(physics): decouple tire slip ratio (tdrace-ab12)`).

### 2. Simplicity First (KISS/DRY)
- Write the minimum amount of code required to resolve the problem. No speculative abstractions, unused generic parameters, or unrequested features.
- Avoid introducing redundant external packages or library dependencies when the standard library or existing workspace crates already provide the necessary primitives.
- If a simpler approach exists, advocate for it before implementing complex architectures.

### 3. Surgical Changes & Codebase Integrity
- Touch only what is required to satisfy the goal. Never format, refactor, or "improve" adjacent code, comments, or imports that are unrelated to the current task.
- Match existing code conventions, naming schemes, and indentation style exactly.
- Remove all imports, variables, or functions that YOUR changes render unused. Do not remove pre-existing dead code unless explicitly requested.

### 4. Goal-Driven Verification & Invariant Enforcement
- Enforce the Plan-Build-Verify loop: know your testing and validation criteria before writing any code.
- **Strict Determinism Invariant**: Physics calculations must strictly execute with fixed $\Delta t = 1/60\text{ s}$. Never introduce frame-rate dependent or variable-time integration into simulation code.
- **Zero-Warning Compilation**: Ensure all target quality checks (`cargo clippy --workspace`, `cargo test --workspace --exclude tdrace-py`, `pytest tests/python`, `bun run build:all`) compile and pass with zero warnings prior to merge.

---

## 🤝 Codebase Architecture & Ownership

The repository is organized into distinct, decoupled layers with strict dependency boundaries:

```
tdrace/
├── crates/
│   ├── wheelbase/         # Layer 0: Pure deterministic 2D/2.5D vehicle dynamics & Pacejka tire solver (Zero graphics/audio)
│   ├── arcade-race-core/  # Layer 0: Spline geometry, continuous SAT OBB collisions, sector gates, 32-beam LIDAR
│   ├── race-kit/          # Layer 1: Headless race world orchestrator, rules/DNF, lap timing, bot AI driver models
│   ├── race-ui/           # Layer 2: Drawing primitives, follow camera, particle emitters, skidmarks, HUD widgets
│   ├── cabinet/           # Layer 2: Arcade game shell, UiScaler, NavGrid2D, modal stack, audio mixer, LAN netcode
│   ├── tdrace-core/       # Layer 1: Unified facade unifying wheelbase and arcade-race-core, compressed preset loaders
│   ├── tdrace-py/         # Layer 1: PyO3 native extension, zero-copy buffers, CPU software scanline rasterizer
│   └── tdrace-app/        # Layer 3: Playable game application, Macroquad loop, Kira audio, SQLite persistence, Track Studio
├── python/
│   └── tdrace/            # Gymnasium 1.0+ environment definitions (Vector, Continuous, Discrete, Pixels, Multi-Agent)
├── portals/
│   ├── option-a-starlight/# Astro + Starlight Technical Reference Manual & OKF Knowledge Base (port 4321)
│   └── showroom/          # Astro + Tailwind CSS Motorsport Showroom & Physics Lab (port 4322)
├── tracks/                # Single-source-of-truth JSON circuit definitions organized by motorsport module
├── series/                # Declarative TOML championship and tournament season definitions
├── docs/                  # Master Google OKF v0.2 Knowledge Base (physics, vehicles, circuits, engineering)
├── specs/                 # Formal Keel specifications (constitution, feature contracts, API blueprints)
└── .beads/                # Beads issue tracker database and durable task graph
```

### Layer Responsibilities & Strict Boundaries

1. **Physics & Core Geometry Layer (`wheelbase`, `arcade-race-core`)**:
   - Must have **zero dependencies** on graphics engines (`macroquad`), audio (`kira`), windowing, or disk I/O.
   - Responsible for pure mathematical modeling: Pacejka '96 tire equations, dynamic pitch/roll weight transfer, 12-surface friction coefficients, Catmull-Rom arc-length parameterization, continuous SAT collision response, and raycast LIDAR.
2. **Headless Race Simulation Layer (`race-kit`, `tdrace-core`)**:
   - Manages race rules, sector splits, finish line detection, DNF states, and bot AI tactical decision-making without rendering.
   - Guaranteed to compile for `wasm32-unknown-unknown` without audio or windowing overhead.
3. **Shell & Presentation Layer (`cabinet`, `race-ui`)**:
   - `cabinet`: Implements a resolution-independent 1080p virtual canvas (`UiScaler`), glassmorphism UI widgets (`Accordion`, `FilterBar`, `TabBar`), 2D orthogonal navigation (`NavGrid2D`) with seamless gamepad/keyboard parity, modal screen stacks, profile management, and LAN multiplayer discovery.
   - `race-ui`: Implements top-down track rendering, predictive follow cameras with speed-zoom, particle systems (tire smoke, roost, sparks), skidmark buffers, and telemetry HUD overlays.
4. **Application Layer (`tdrace-app`)**:
   - Single point of integration uniting the game loop, Macroquad rendering, Kira procedural audio synthesis, SQLite persistence (`tdrace_records.db`), Track Studio CAD editor, and career mode dispatch.
5. **Reinforcement Learning Bridge (`tdrace-py`, `python/tdrace`)**:
   - High-throughput PyO3 bindings wrapping the simulation engine with zero-copy NumPy buffers.
   - Pure-CPU software RGB scanline rasterizer producing $(96, 96, 3)$ pixel observations at 11,000+ FPS without an active display or GPU.
6. **Documentation & Portals Layer (`portals/`, `docs/`)**:
   - Dual-site Astro portal ecosystem maintaining public technical manuals and asset showrooms synchronized with the codebase via automated ingestion scripts.
