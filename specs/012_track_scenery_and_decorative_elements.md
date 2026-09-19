---
type: Feature Spec
template: feature
title: "Track Scenery and Decorative Elements (Grades & Trees)"
description: "Decorative track elements featuring tiered concrete grandstands and multi-species trees designed for cenital (top-down) recognition with realistic physical car interactions."
status: implemented
created: 2026-09-19
generated: { by: agent/antigravity, at: 2026-09-19T20:36:00Z }
---
# Feature Spec: Track Scenery & Decorative Elements (Grades & Trees) 🌲🏟️

A comprehensive architectural, simulation, and rendering specification introducing first-class decorative scenery elements—specifically tiered concrete grandstands ("grades" / bleachers) and multi-variety trees—engineered for high aesthetic fidelity in **cenital (top-down / bird's-eye) view** and realistic **physical interactions with cars**.

---

## 🗺️ User Flow & Interface Design

### 1. In-Game Cenital (Top-Down) Visual Aesthetics
In-game cameras dynamically display high-fidelity track environment props:
- **Stepped Concrete Grandstands**: Rendered with top-down directional lighting, cast shadows, colorful spectator seating blocks, access stairways, and front concrete safety barriers.
- **Top-Down Tree Silhouettes**: Rendered with sun drop shadows, visible central trunk stubs, species-specific leaf/frond silhouettes, and dynamic proximity alpha fading ($60\%$ opacity) when cars pass under the canopy so drivers maintain continuous visual track of their vehicles.

### 2. Track Editor & Prop Placement Flow
The in-game track editor integrates scenery elements into the placement toolset:
- Select and drop grandstands along straightaways and corner runoffs with customizable length, depth, and tier counts.
- Scatter tree groves across in-field and out-field zones with species selection (`Pine`, `Palm`, `Oak`, `Cypress`, `Sakura`, `AutumnMaple`) and randomized scale/rotation jitter.

---

## ⚙️ Backend Models & API Endpoints

### 1. Core Scenery Entities (`crates/arcade-race-core/src/track/scenery.rs`)
- **`TreeType`**:
  - `Pine`: Conifer with multi-tiered radial star-spoked needle clusters.
  - `Palm`: Tropical radial starburst crown with arching fronds and gold center.
  - `Oak`: Deciduous broadleaf multi-lobed organic cloud canopy.
  - `Cypress`: Compact columnar flame/oval silhouette.
  - `Sakura`: Soft flowering cloud canopy of pastel pink and magenta petals.
  - `AutumnMaple`: Warm fiery crown of golden-amber, orange, and crimson foliage.
- **`Tree`**:
  - Contains `id`, `position`, `tree_type`, `scale`, `rotation`, `elevation`.
  - Computes `trunk_radius()` ($0.25 - 0.50\,\text{m}$) and `canopy_radius()` ($1.8 - 5.0\,\text{m}$).
  - Generates rigid trunk circle obstacle with wood physical parameters (`restitution = 0.25`, `friction = 0.55`).
- **`GrandstandStyle`**:
  - `OpenBleachers`, `CoveredStadium`, `HillsideBleachers`.
- **`Grandstand`**:
  - Contains `id`, `center`, `length`, `depth`, `angle`, `tiers`, `style`, `seat_color`, `elevation`.
  - Generates concrete oriented box obstacle (`restitution = 0.65`, `friction = 0.32`).
  - Provides concrete ground surface footprint (`SurfaceType::Concrete`).

### 2. Surface & Physical Interaction Modeling
- **`SurfaceType::Concrete` (`crates/wheelbase/src/surface.rs`)**:
  - Friction coefficient: $\mu = 0.95$.
  - Rolling resistance: $C_{\text{rr}} = 1.05$.
  - Drag multiplier: $C_{\text{drag}} = 1.00$.
- **Canopy Soft Brush Interaction (`crates/tdrace-app/src/game/mod.rs`)**:
  - When $r_{\text{trunk}} < d \le r_{\text{canopy}}$, applies soft deceleration: $\vec{a}_{\text{drag}} = -c_{\text{drag}} \cdot \vec{v}$ ($1.5 - 3.5\,\text{m/s}^2$).
  - Emits leaf/needle roost particles in the car's wake.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

### 1. Memory Safety & Bounded Geometry
- Deserialization of scenery elements adheres to strict numeric sanity bounds (e.g. scale clamped to $[0.2, 5.0]$, grandstand length clamped to $[5.0, 300.0]\,\text{m}$).
- SAT collision loops enforce finite intersection tests with early broad-phase bounding box rejection.

### 2. Deterministic Physics Invariance
- Dual-zone tree interactions and grandstand collision impulses run strictly on 60 Hz deterministic fixed-step updates, guaranteeing zero divergence across split-screen, ghost replays, or headless simulations.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- Command to verify wheelbase surfaces: `cargo test -p wheelbase`
- Command to verify scenery models and SAT collision: `cargo test -p arcade-race-core`
- Command to verify game integration and physics: `cargo test -p tdrace-core`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Physical collision with concrete grandstand**
  - [x] **Given** a vehicle driving toward a trackside concrete grandstand
  - [x] **When** the vehicle impacts the grandstand boundary wall
  - [x] **Then** the collision must be resolved using concrete restitution ($0.65$) and sliding friction ($0.32$)
  - [x] **And** vehicle speed must be arrested without tunneling or NaN positions
  - [x] **And** wall scraping deceleration must apply $9.0\,\text{m/s}^2$ resistance when sliding along the structure

- **Scenario: Dual-zone physical interaction with a tree**
  - [x] **Given** a tree positioned at $(x, y)$ with trunk radius $r_{\text{trunk}} = 0.4\,\text{m}$ and canopy radius $r_{\text{canopy}} = 3.5\,\text{m}$
  - [x] **When** a vehicle drives within the canopy ($d = 2.0\,\text{m}$) without striking the trunk
  - [x] **Then** the vehicle must pass through without a hard rigid body bounce
  - [x] **And** soft foliage drag deceleration must be applied to the car
  - [x] **And** leaf/needle roost particles must be emitted in the vehicle's wake
  - [x] **And** the canopy must fade to semi-transparent opacity ($0.60$) while the car is underneath
  - [x] **When** the vehicle strikes the central trunk ($d \le 0.4\,\text{m}$)
  - [x] **Then** a rigid circle collision must occur with wood restitution ($0.25$) and bark friction ($0.55$)

- **Scenario: Cenital (top-down) visual differentiation**
  - [x] **Given** the 6 tree species rendered on screen from a top-down camera
  - [x] **Then** Pine must exhibit concentric star-spoked needle tiers
  - [x] **And** Palm must exhibit arching radiating fronds with central crown
  - [x] **And** Oak must exhibit multi-lobed organic green cloud foliage
  - [x] **And** Cypress must exhibit a tight columnar oval crown
  - [x] **And** Sakura must render with distinct pink/magenta floral tones
  - [x] **And** Autumn Maple must render with golden-orange autumn tones

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Crates
- `crates/wheelbase/src/surface.rs` -> Concrete surface type and physical parameters.
- `crates/arcade-race-core/src/track/scenery.rs` -> Data structures for `Tree`, `TreeType`, `Grandstand`, `GrandstandStyle`.
- `crates/arcade-race-core/src/track/geometry.rs` -> `TrackGeometry` integration.
- `crates/tdrace-app/src/render/scenery.rs` -> Top-down cenital scenery rendering.
- `crates/tdrace-app/src/game/mod.rs` -> Physics step integration and canopy drag.

### Beads Epic Mapping
- Tracked via Beads Epic: `tdrace-qktf` ("Fulfill Spec 012: Track Scenery & Decorative Elements").
- Child task: `tdrace-track-decorative-elements-dnhp`.
