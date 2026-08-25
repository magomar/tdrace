# 🏎️ TDRace

<div align="center">

**High-Performance Top-Down 2D Arcade Racing Game & Gymnasium Reinforcement Learning Environment**

[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10%2B-blue?logo=python)](https://www.python.org/)
[![Gymnasium](https://img.shields.io/badge/Gymnasium-1.0%2B-brightgreen?logo=openai)](https://gymnasium.farama.org/)
[![Platforms](https://img.shields.io/badge/Platforms-Linux%20|%20macOS%20|%20Windows%20|%20Web%20|%20Mobile-blueviolet)]()
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-lightgrey)]()

</div>

---

## 📖 Overview

**TDRace** is a high-performance top-down 2D arcade racing game inspired by arcade classics such as *Super Sprint* (1986), *Slicks 'N Slide* (1993), and *GeneRally* (2002).

Built in pure **Rust** with native **Python / Gymnasium bindings**, TDRace combines nostalgic 2.5D arcade aesthetics with modern vehicle dynamics and a lightning-fast simulation engine designed from the ground up for both human competitive play and deep reinforcement learning (RL) training.

---

## ✨ Features

### 🏎️ Advanced Arcade Vehicle Physics (`crates/tdrace-core`)
- **4-Wheel Pacejka Tire Dynamics**: Non-linear lateral and longitudinal slip calculations adapted for responsive arcade drifting.
- **Dynamic Weight Transfer**: Realistic pitch (acceleration squat & braking dive) and lateral roll affecting per-wheel normal loads.
- **Surface Friction Split-$\mu$**: Per-wheel surface sampling across 6 terrain types: `Asphalt`, `Curb`, `Grass`, `Sand`, `Oil`, and `Ice`.
- **Ackermann Steering**: Speed-sensitive steering angle attenuation and counter-steer drift assist.
- **Deterministic Math**: Pure fixed-timestep stepping ($60\text{ Hz}$) yielding bit-identical physics reproduction.

### 🏁 Track Geometry & Collision Solver
- **Catmull-Rom Spline Track Ribbons**: Continuous arc-length parameterization, curvature calculation, and curb zones.
- **Zero-Tunneling SAT Solver**: Continuous Separating Axis Theorem (SAT) Oriented Bounding Box (OBB) collision resolution for high-speed multi-car pileups and wall impacts.
- **Directional Checkpoint Gates**: Anti-cheat sequence validation, wrong-way detection, sector timing, and pit lane support.
- **4 Built-in Track Presets**:
  1. *Classic Grand Prix* (flowing corners, chicanes, curbs, pit lane)
  2. *Oval Speedway* (high-speed banked perimeter walls)
  3. *Drift Park* (technical hairpins, sand traps, oil slicks)
  4. *Kart Arena* (tight chicanes and curbing)

### 🤖 Gymnasium RL Interface (`python/tdrace`, `crates/tdrace-py`)
- **Ultra-Fast Throughput**: **64,000+ steps/sec** in Python vector environments (**180x faster** than standard `CarRacing-v3`).
- **Software RGB Scanline Rasterizer**: Ultra-lightweight CPU renderer producing $(96, 96, 3)$ pixel observations at **11,000+ FPS**.
- **Multi-Agent Simulation**: Parallel stepping of up to 16 cars simultaneously with mutual collision dynamics.
- **Registered Environments**:
  - `TDRace-v0` (Continuous action space: `[steer, throttle, brake]`)
  - `TDRace-Continuous-v0` (Continuous actions with 32-beam LIDAR + vehicle telemetry observations)
  - `TDRace-Discrete-v0` (Discrete action space: 5 actions)
  - `TDRace-Drift-v0` (Continuous action space with drift scoring rewards)
  - `TDRace-Pixels-v0` (Top-down RGB pixel observations)
  - `TDRace-MultiAgent-v0` (Multi-car grid racing)

### 🎮 Arcade Application & Visuals (`crates/tdrace-app`)
- **2.5D Visual Styling**: Drop shadows, rumble curbs, driver helmets, and steerable articulated wheels.
- **Particle & Skidmark System**: Ring-buffered skidmarks, tire smoke, gravel roost, and collision sparks.
- **Smooth Predictive Camera**: Dynamic speed-zoom, velocity lookahead, screen shake trauma, and static track overview toggle.
- **Autonomous Bot AI Drivers**: Realistic racing lines, cornering envelopes ($v = \sqrt{v_{\text{apex}}^2 + 2ad}$), and collision evasion profiles (*Pro*, *Aggressive*, *Club*, *Rookie*).
- **Zero-Lag Multi-Touch Controls**: Virtual joystick, split steering buttons, and pedals for touchscreens.
- **Deterministic Replay & Ghost Car**: `.tdr` binary format with $1\times$–$8\times$ playback, scrubbing, and personal best ghost telemetry.

---

## 🕹️ Controls

### Keyboard
| Key | Action | Description |
| :---: | :---: | :--- |
| **`Q`** | **Throttle** | Accelerate forward |
| **`A`** | **Brake** | 4-wheel service braking |
| **`Space`** | **Handbrake** | Rear-wheel lock for power sliding and drifting |
| **`Z`** | **Reverse** | Reverse gear throttle |
| **`O`** | **Steer Left** | Steer front wheels and chassis left |
| **`P`** | **Steer Right** | Steer front wheels and chassis right |
| **`Tab` / `C`** | **Camera** | Toggle between dynamic follow camera and full-track overview |
| **`R`** | **Restart** | Instantly reset the current race session |
| **`Esc`** | **Pause** | Pause/resume session |
| **`M`** | **Menu** | Return to main track & vehicle selection menu |
| **`F1`–`F5`** | **Debug** | Toggle LIDAR beams, Checkpoints, OBBs, AI Paths, and Telemetry |

*(Arrow keys `Up`/`Down`/`Left`/`Right` are also supported as secondary driving controls).*

### Gamepad
| Button / Input | Action | Description |
| :---: | :---: | :--- |
| **`RT`** | **Throttle** | Progressive analog throttle |
| **`LT`** | **Brake** | Progressive analog brake |
| **`A` / `RB`** | **Handbrake** | Rear-wheel lock for power sliding and drifting |
| **`Left Stick` / `D-Pad`** | **Steer** | Proportional analog steering |
| **`Y` / `LB`** | **Reverse** | Reverse gear throttle |
| **`Start`** | **Pause** | Pause/resume session |
| **`R3` / `Select`** | **Assists** | Cycle driving assist profile (Arcade, Sport, Pro) |
| **`L3`** | **Camera** | Toggle follow and overview camera |

---

## 🚀 Quickstart

### Prerequisites
- **Rust** (2021 edition or newer): [rustup.rs](https://rustup.rs/)
- **Python 3.10+** (for Gymnasium RL environment)
- **uv** (recommended) or `python3-venv`

### 1. Setup Environment
```bash
# Clone the repository
git clone git@github.com:magomar/tdrace.git
cd tdrace

# Initialize virtualenv, install dependencies, and build PyO3 extension
make setup
```

### 2. Launch the Desktop Game
```bash
# Run in optimized release mode
make run

# Or in debug mode
make run-dev
```

### 3. Play in Web Browser (WebAssembly)
```bash
# Builds WASM binary and serves at http://localhost:8080
make serve-web
```

---

## 🧠 Gymnasium Python RL Usage

```python
import gymnasium as gym
import tdrace

# 1. Vector Observation Environment (32-beam LIDAR + vehicle telemetry)
env = gym.make("TDRace-v0", track_name="classic_grand_prix")
obs, info = env.reset(seed=42)

for _ in range(1000):
    # Action: [steer (-1 to 1), throttle (0 to 1), brake (0 to 1)]
    action = env.action_space.sample()
    obs, reward, terminated, truncated, info = env.step(action)
    
    if terminated or truncated:
        obs, info = env.reset()

env.close()

# 2. Top-Down Pixel Observation Environment (96x96 RGB)
pixel_env = gym.make("TDRace-Pixels-v0", image_size=(96, 96))
obs, info = pixel_env.reset()
print("Pixel observation shape:", obs.shape)  # (96, 96, 3)
```

---

## 📊 Performance Benchmarks

| Benchmark Metric | Gymnasium `CarRacing-v3` (Box2D) | **TDRace-v0 (Rust + PyO3)** | Speedup |
| :--- | :---: | :---: | :---: |
| **Pure Rust Step Throughput** | N/A | **4,020,000 steps/sec** | — |
| **Python Single-Env Throughput** | ~1,250 steps/sec | **28,400 steps/sec** | **22.7x** |
| **Python Vectorized Throughput** | ~357 steps/sec | **64,350 steps/sec** | **180.3x** |
| **Software RGB Pixel Rasterizer** | ~350 FPS (OpenGL required) | **11,180 FPS (Pure CPU)** | **31.9x** |
| **OBB SAT Collision Checks** | ~1.2M checks/sec | **22,400,000 checks/sec** | **18.6x** |
| **LIDAR Raycasting Throughput** | N/A | **1,110,000 rays/sec** | — |

To run the benchmarks locally:
```bash
make bench
```

---

## 📱 Cross-Platform Builds

- **WebAssembly**: `./web/build_web.sh` (produces a standalone 660KB WASM bundle in `web/dist/`)
- **Android**: `./mobile/android/build_android.sh` (builds `arm64-v8a` / `x86_64` `.so` and Gradle project)
- **iOS**: `./mobile/ios/build_ios.sh` (builds universal static framework for iOS Device and Simulator)
- **Universal Build Script**: `./scripts/build_all.sh`

---

## 📂 Project Architecture

```
tdrace/
├── crates/
│   ├── tdrace-core/       # Core vehicle physics, splines, checkpoints, and SAT collisions
│   ├── tdrace-py/         # PyO3 C-extension, zero-copy buffers, and fast scanline rasterizer
│   └── tdrace-app/        # Macroquad 2D arcade renderer, camera, bot AI, HUD, and touch input
├── python/
│   └── tdrace/            # Gymnasium 1.0+ environment definitions and wrappers
├── benchmarks/            # Throughput benchmarking suite (TDRace vs CarRacing-v3)
├── mobile/
│   ├── android/           # Android NDK manifest and Gradle build setup
│   └── ios/               # iOS Xcode project setup and build script
├── web/                   # HTML5 canvas shell and WebAssembly compiler pipeline
├── tests/
│   └── python/            # Gymnasium compliance, adversarial, and determinism tests
└── Makefile               # Developer workflow shortcuts
```

---

## 🧪 Testing

```bash
# Run all Rust & Python test suites
make test

# Run Rust unit, integration, and physics tests (87 tests)
make test-rust

# Run Python Gymnasium compliance tests (54 tests)
make test-python
```

---

## 📜 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
