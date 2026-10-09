---
type: Feature Spec
template: feature
title: "Autocross and Rallycross Launch Chutes and Templated Track Components"
description: "Introduces authentic Autocross and Rallycross walled launch chutes and packed starting grids as modular templated track components via TrackNetwork, providing Run-Once Lap 1 launch progression, Track Studio CAD stamp authoring, and global retrofitting across all 17 Autocross and 23 Rallycross circuits."
status: in_progress
receipt: "docs/receipts/spec-103-receipt.md"
verified: { by: human:mario, at: 2026-10-08T21:13:11Z, hash: "b50753032b35" }
created: 2026-10-08
generated: { by: agent/antigravity, at: 2026-10-08T23:00:00Z }
depends_on:
  - "006"
  - "042"
  - "050"
  - "051"
  - "081"
  - "086"
---

# Feature Spec 103: Autocross and Rallycross Launch Chutes and Templated Track Components 🏁🏗️

In real-world **FIA Autocross** and **Rallycross (World RX)**, vehicles do not start in an elongated, two-by-two staggered queue along the main circuit straightaway as in GT or Formula 1. Instead, competitors assemble in a dedicated, enclosed **launch chute / start pad** separate from the main circuit loop—such as the high-grip concrete launch pad at **Matschenberg Offroad Arena** (Upper Lusatia, Germany) or the famous launch lanes at **Höljes** and **Nová Paka**. Competitors launch 4-to-5 abreast off high-traction concrete or tarmac, accelerate down the walled funnel, and merge aggressively into Turn 1 of the dirt raceway. Subsequent laps bypass the start chute entirely.

This specification formalizes:
1. **The Modular Launch Chute Track Component**: An open-ended spur spline segment terminating in a rigid rear barrier, enclosed by side walls, supporting high-density packed grid spawn poses (e.g. 5-3 or 3-2-3 rows), and merging into the main circuit via a dedicated `RoadJunction::Merge`.
2. **Track Studio Templated Component Stamp**: An interactive CAD tool in the in-game Track Studio enabling creators to stamp, position, and snap pre-fabricated launch chutes directly onto circuit splines.
3. **Run-Once Lap 1 Route Progression**: Enhancing `TrackLayout` with an `entry_segment` property ensuring vehicles launch down the chute once, merge onto the track, and complete all subsequent laps exclusively on the primary circuit loop.
4. **Comprehensive AX & RX Circuit Rollout**: Retrofitting authentic launch chute geometry across all 17 official European Autocross circuits and 23 Rallycross circuits.

---

## 🗺️ User Flow & Interface Design

### 1. Circuit Start Topologies

#### Previous Circuit Start (Standard Monolithic Grid)
All vehicles spawned directly on the primary circuit ribbon in a two-by-two staggered column, often stretching 100+ meters down the main straight. Cars immediately crossed the Start/Finish line upon rolling forward.

```
═════════════════════════════════════════════════════════════════════
   [12] [11] ... [4] [3] [2] [1] | S/F Line | ===> Turn 1
═════════════════════════════════════════════════════════════════════
```

#### Proposed Modular Launch Chute Topology
Cars spawn off-circuit on a dedicated, high-grip pad (`Concrete` or `Asphalt`) bounded by end and side barriers. Vehicles drag-race down the launch chute and merge into the dirt track loop. Subsequent laps bypass the launch chute entirely via an un-diverted main ribbon.

```
 [ Walled Terminal ]
 ┌─────────────────┐
 │ [ 1 ] [ 2 ] [ 3 ] [ 4 ] [ 5 ] │ <- Row 1 (5 Abreast)
 │     [ 6 ] [ 7 ] [ 8 ]         │ <- Row 2 (3 Abreast)
 │                               │
 │   Paved Launch Pad (Spline)   │ <- High-grip asphalt/concrete pad
 └────────┬───────────────┬──────┘
          │ Armco Barrier │
          ▼               ▼
══════════╪═══════════════╪═══════════════════════════════════════════
 Dirt Main Circuit Ribbon  \ Merge Junction (Socket)
                            \═══► Turn 1 Funnel (Compacted Dirt/Clay)
═════════════════════════════════════════════════════════════════════
```

### 2. Track Studio CAD Component Workflow
- In the in-game Track Studio CAD editor ([`crates/tdrace-app/src/editor/`](../crates/tdrace-app/src/editor/)), creators can access the Component Palette and click `[ + INSERT LAUNCH CHUTE ]`.
- Clicking any waypoint on the active circuit snaps a templated launch pad spur segment (35m–50m length) with auto-generated end walls, side barriers, and packed grid positions connected via a merge junction.
- The inspector provides controls for launch pad width (14.0m–18.0m), grid pattern (`AutocrossFiveThree`, `RallycrossThreeTwoThree`), and surface type (`Concrete` or `Asphalt`).

---

## ⚙️ Backend Models & API Endpoints

### 1. Launch Chute Specification (`LaunchChuteConfig`)
Added to [`crates/arcade-race-core/src/track/network.rs`](../crates/arcade-race-core/src/track/network.rs):

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LaunchChuteConfig {
    /// Identifier of the spur road segment forming the launch pad.
    pub segment_id: SegmentId,
    /// Upstream terminal boundary with protective end wall.
    pub terminal_barrier: WallBarrier,
    /// Side barrier walls flanking the launch chute.
    pub side_barriers: Vec<WallBarrier>,
    /// Junction ID where the launch chute converges into the primary circuit.
    pub merge_junction_id: JunctionId,
    /// Packed starting grid spawn configurations.
    pub grid_slots: Vec<SpawnPose>,
    /// Launch pad surface (typically Concrete or Asphalt for launch traction).
    pub surface: SurfaceType,
    /// Width of the launch pad (meters), accommodating multi-car rows (typically 14.0–18.0m).
    pub pad_width: f32,
}
```

### 2. Track Layout Run-Once Entry Support
In [`crates/arcade-race-core/src/track/network.rs`](../crates/arcade-race-core/src/track/network.rs), `TrackLayout` is enhanced to differentiate between the **Lap 1 Start Sequence** (traversing the launch chute once) and the **Cyclic Racing Lap Sequence** (traversed on laps 2+):

```rust
pub struct TrackLayout {
    pub id: String,
    pub display_name: String,
    /// Optional run-once lead-in segment traversed exclusively at the start of the race.
    pub entry_segment: Option<SegmentId>,
    /// Cyclic loop segments traversed during full racing laps.
    pub segment_sequence: Vec<SegmentId>,
    pub is_closed: bool,
    pub total_lap_length: f32,
    pub start_finish_segment: SegmentId,
    pub checkpoint_ids: Vec<usize>,
}
```

### 3. Packed Grid Formation Generator
In [`crates/arcade-race-core/src/track/presets.rs`](../crates/arcade-race-core/src/track/presets.rs):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackedGridPattern {
    /// 5 cars on Row 1, 3 cars on Row 2 (Classic FIA Autocross 8-car sprint).
    AutocrossFiveThree,
    /// 3 cars on Row 1, 2 cars on Row 2, 3 cars on Row 3 (FIA Rallycross 3-2-3 staggered).
    RallycrossThreeTwoThree,
    /// 4 cars per row uniform grid.
    UniformFourAcross,
}

pub fn generate_packed_launch_grid(
    pad_center: Vec2,
    pad_tangent: Vec2,
    pad_width: f32,
    pattern: PackedGridPattern,
    row_spacing: f32,
) -> Vec<SpawnPose>;
```

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

1. **Anti-Cheat & Progress Boundary Enforcement**:
   - Vehicles launching from the launch chute must strictly cross through the merge junction gate. Skipping or jumping across boundary barriers immediately invalidates the lap.
   - On subsequent laps (Laps 2+), the merge junction enforces one-way ingress into the circuit. Any attempt by a vehicle to turn backwards into the launch chute triggers instant wrong-way warning and invalidation.
2. **Determinism & Physics Stability**:
   - Boundary collisions against the rear terminal wall and flanking Armco barriers must execute with continuous SAT resolution, preventing penetration or corner-catching during high-pack launches.
3. **Track Validation Invariants**:
   - `validate_track()` verifies that:
     - The launch chute segment terminates in a closed wall barrier with no open vertex gaps.
     - The merge junction maintains $C^1$ tangent alignment with the receiving track ribbon.
     - All 8 grid spawn poses lie strictly within the drivable launch pad width.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- `cargo test -p arcade-race-core --test launch_chute_tests`
- `cargo test -p tdrace-app --test track_editor_launch_chute_tests`
- `cargo test -p tdrace-core --test ax_rx_circuit_launch_chute_validation`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Packed Launch Grid Formation Generation**
  - [x] **Given** an Autocross launch pad spline with a width of 16.0 meters and tangent vector `(1.0, 0.0)`
  - [x] **When** `generate_packed_launch_grid` is called with pattern `AutocrossFiveThree`
  - [x] **Then** 8 spawn poses should be generated
  - [x] **And** poses 0 through 4 should occupy Row 1 with identical longitudinal distances and spaced lateral offsets
  - [x] **And** poses 5 through 7 should occupy Row 2 offset backwards by `row_spacing` (4.0m)

- **Scenario: Lap 1 Execution from Launch Chute into Main Circuit**
  - [x] **Given** a circuit loaded with `entry_segment` pointing to the launch chute
  - [x] **When** vehicles spawn at race start and green lights activate
  - [x] **Then** vehicle initial positions should be located within the launch chute segment
  - [x] **And** passing through the merge junction should smoothly transition vehicles onto the main dirt ribbon without wall snagging
  - [x] **And** crossing the primary Start/Finish line on the main ribbon should register the completion of Lap 1

- **Scenario: Subsequent Laps Exclude the Launch Chute**
  - [x] **Given** a vehicle navigating Lap 2 or higher on the main circuit loop
  - [x] **When** the vehicle approaches the merge junction zone
  - [x] **Then** the racing line and track boundaries should keep the car on the main circuit ribbon
  - [x] **And** AI bots should not divert backwards or turn into the launch chute

- **Scenario: Track Studio Launch Chute Insertion**
  - [x] **Given** the Track Studio editor active on a user-authored or official circuit
  - [x] **When** the user clicks `[ + INSERT LAUNCH CHUTE ]` and selects a target waypoint
  - [x] **Then** a templated launch spur segment, walled end cap, and 8 packed grid slots should be appended
  - [x] **And** saving the circuit should preserve the `LaunchChuteConfig` and valid `TrackNetwork` topology

- **Scenario: Global AX and RX Circuit Validation**
  - [x] **Given** the 17 official Autocross circuits and 23 official Rallycross circuits
  - [x] **When** `validate_track()` is executed on every circuit definition
  - [x] **Then** all 40 circuits should contain a valid `LaunchChuteConfig`
  - [x] **And** zero boundary wall gaps or invalid spawn poses should be detected

---

## 🔗 Traceability & Codebase Mapping

### Core Geometry & Track Network Layer
- [`crates/arcade-race-core/src/track/network.rs`](../crates/arcade-race-core/src/track/network.rs): `LaunchChuteConfig`, `TrackLayout::entry_segment`, merge validation.
- [`crates/arcade-race-core/src/track/presets.rs`](../crates/arcade-race-core/src/track/presets.rs): `generate_packed_launch_grid`, `PackedGridPattern`.

### Headless Simulation & Timing Layer
- [`crates/race-kit/src/world.rs`](../crates/race-kit/src/world.rs): Lap 1 chute-to-trunk progress detection and finish-line checkpoint traversal.
- [`crates/race-kit/src/ai/mod.rs`](../crates/race-kit/src/ai/mod.rs): AI launchpad drag-race throttle pacing and merge awareness.

### Application & Editor Layer
- [`crates/tdrace-app/src/editor/tools.rs`](../crates/tdrace-app/src/editor/tools.rs): Templated launch chute stamp insertion tool.
- [`crates/tdrace-app/src/editor/ui.rs`](../crates/tdrace-app/src/editor/ui.rs): Launch chute inspector properties and button actions.
- [`tracks/autocross/matschenberg_ax.json`](../tracks/autocross/matschenberg_ax.json): Reference circuit launch chute integration.
