# Specification: Road Split Segments, Branching Splines & Alternative Circuit Layouts

**Document Status:** PROPOSED  
**Author:** Antigravity Pairing Assistant & Engineering Team  
**Date:** September 15, 2026  
**Primary Target Crates:**  
1. `crates/arcade-race-core` (Track Network Topology, Splines, Checkpoint Tracking & Perception)  
2. `crates/tdrace-app` (Track Editor Tooling, Multi-Route Rendering & Bot Navigation)  
3. `crates/wheelbase` (Surface Sampling across Divergent Ribbons)  

---

## 1. Executive Summary & Motivation

### 1.1 Current Architecture Limitations
Currently, `arcade-race-core` models every racing circuit as a single linear sequence of waypoints defined by `TrackSpline` (`Vec<TrackWaypoint>`), which can be either a closed cyclic loop (`closed: true`) or an open point-to-point path.

While this works well for standard closed-loop circuits, it imposes strict architectural limitations:
- **No Branching or Splits**: It is impossible to model road bifurcations, forks, or alternate paths within the same circuit.
- **No Alternative Layouts within One Track**: Real-world venues (e.g. Silverstone GP vs National vs International, or Nürburgring GP vs Nordschleife vs Combined 24h) must currently be duplicated as separate, disconnected track files, even when they share $70\%+$ of their asphalt ribbon, pit lanes, and infrastructure.
- **No Joker Laps for Rallycross**: Rallycross tracks (such as Holjes, Hell, and Lydden Hill) require an alternate, mandatory longer/shorter Joker Lap detour that splits off from the main line and merges back later.
- **Disconnected Pit Lanes**: Pit lanes currently cannot be modeled as continuous branching road splines that peel off from the main straight and rejoin down the road.

### 1.2 Objectives
This specification introduces:
1. **The Road Split Segment (`RoadSplitSegment` / `SplitJunction`)**: A road element that introduces a clean physical bifurcation (1 entry $\to$ $N$ branch sockets).
2. **Branching Splines**: The capability to attach, edit, and extend independent Catmull-Rom splines from any branch socket using standard track editor tools.
3. **Directed Track Ribbon Graph (`TrackNetwork`)**: A robust topological graph connecting road segments via junctions (splits, merges, and endpoints).
4. **Alternative Track Configurations (`TrackLayout`)**: First-class support for defining named route variants (e.g., "Grand Prix Course", "Club Course", "Joker Lap Route") within a single track definition.
5. **Multi-Route Checkpoint, Timing & AI Integration**: Seamless lap tracking, sector timing, and AI navigation across branching circuits.

---

## 2. Mathematical & Topological Domain Model

### 2.1 The Directed Ribbon Graph (`TrackNetwork`)
To allow splines to branch and merge naturally while preserving smooth Catmull-Rom interpolation, circuits transition from a single isolated spline to a **Directed Ribbon Graph**:

```
                          ┌────────────────────────┐
                          │   RoadSegment A (Main) │
                          │   (Pre-Split Trunk)    │
                          └───────────┬────────────┘
                                      │
                                      ▼
                          ┌────────────────────────┐
                          │   RoadJunction::Split  │
                          │   (Branch Sockets 0,1) │
                          └─────┬────────────┬─────┘
                                │            │
                    Socket 0 ┌──┘            └──┐ Socket 1
                             ▼                  ▼
             ┌─────────────────────────┐      ┌─────────────────────────┐
             │  RoadSegment B (Main)   │      │  RoadSegment C (Joker)  │
             │  (Primary Racing Line)  │      │  (Alternative Detour)   │
             └─────────────┬───────────┘      └────────────┬────────────┘
                           │                               │
                           └──────────────┐ ┌──────────────┘
                                          ▼ ▼
                              ┌────────────────────────┐
                              │  RoadJunction::Merge   │
                              │   (Convergence Point)  │
                              └───────────┬────────────┘
                                          │
                                          ▼
                              ┌────────────────────────┐
                              │  RoadSegment D (Return)│
                              └────────────────────────┘
```

### 2.2 Core Rust Data Structures

```rust
use glam::Vec2;
use serde::{Deserialize, Serialize};
use wheelbase::SurfaceType;
use crate::track::geometry::{BarrierType, LineSegment, WallBarrier};
use crate::track::spline::{SplineSample, TrackWaypoint};

/// A unique handle identifying a road segment within a track network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SegmentId(pub u32);

/// A unique handle identifying a junction (split/merge/terminal) node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JunctionId(pub u32);

/// Identifies a specific connection socket on a junction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SocketId {
    pub junction_id: JunctionId,
    pub socket_index: usize,
}

/// Geometric and physical boundary conditions at a segment connection port.
/// Guarantees C¹ positional and tangent continuity across connected splines.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SplineSocket {
    /// 2D world coordinates of the centerline connection point.
    pub point: Vec2,
    /// Outgoing tangent unit vector (direction of travel away from socket).
    pub tangent: Vec2,
    /// Inward-pointing normal vector (left normal).
    pub normal: Vec2,
    /// Asphalt track width at the connection boundary (meters).
    pub width: f32,
    /// Road surface elevation above ground (meters).
    pub elevation: f32,
    /// Cross-slope banking angle (degrees).
    pub bank_angle: f32,
    /// Default surface type at junction boundary.
    pub surface: SurfaceType,
}

/// Type of topological junction connecting road segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JunctionKind {
    /// 1 incoming trunk splitting into 2 or more outgoing branches.
    Split {
        /// Entry socket receiving traffic from the ingress segment.
        ingress_socket: SplineSocket,
        /// Outgoing branch sockets (index 0 = Branch A / default, 1 = Branch B / alternate, etc.).
        egress_sockets: Vec<SplineSocket>,
        /// Geometric configuration of the bifurcation wedge / gore area.
        gore_config: GoreConfig,
    },
    /// 2 or more incoming branches merging into 1 outgoing trunk.
    Merge {
        /// Incoming branch sockets delivering traffic to the merge.
        ingress_sockets: Vec<SplineSocket>,
        /// Outgoing socket continuing traffic down the unified trunk.
        egress_socket: SplineSocket,
        /// Geometric merge taper configuration.
        merge_config: MergeConfig,
    },
    /// Open-ended terminal (for point-to-point rally/hillclimb stages).
    Terminal {
        socket: SplineSocket,
    },
}

/// Node representing a connection between road segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadJunction {
    pub id: JunctionId,
    pub name: String,
    pub kind: JunctionKind,
}

/// A continuous spline ribbon forming an edge in the Track Network graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadSegment {
    pub id: SegmentId,
    pub name: String,
    /// Waypoints defining this segment's centerline and cross-section.
    pub waypoints: Vec<TrackWaypoint>,
    /// Dense, uniformly resampled arc-length spline samples.
    pub samples: Vec<SplineSample>,
    /// Total arc-length distance of this segment in meters.
    pub length: f32,
    /// Connection to upstream junction (entry). None if start of circuit.
    pub entry_junction: Option<SocketId>,
    /// Connection to downstream junction (exit). None if terminal/open.
    pub exit_junction: Option<SocketId>,
    /// Segment-specific wall barriers (left and right).
    pub walls: Vec<WallBarrier>,
}
```

### 2.3 Alternative Track Layouts (`TrackLayout`)
A circuit file can host multiple named layout configurations by specifying valid sequential walks through the `TrackNetwork` graph:

```rust
/// A named circuit layout formed by an ordered sequence of connected road segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackLayout {
    /// Identifier (e.g. "grand_prix", "joker_lap", "national", "club").
    pub id: String,
    /// Human-readable title displayed in menus.
    pub display_name: String,
    /// Ordered sequence of segment IDs traversed in this layout.
    pub segment_sequence: Vec<SegmentId>,
    /// Whether this route forms a closed loop back to the starting segment.
    pub is_closed: bool,
    /// Total nominal lap distance in meters.
    pub total_lap_length: f32,
    /// Segment ID containing the primary Start/Finish line.
    pub start_finish_segment: SegmentId,
    /// Checkpoints active along this specific route.
    pub checkpoint_ids: Vec<usize>,
}

/// Complete track network representing multi-branch circuits and layouts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackNetwork {
    pub junctions: Vec<RoadJunction>,
    pub segments: Vec<RoadSegment>,
    pub layouts: Vec<TrackLayout>,
    pub default_layout_id: String,
}
```

---

## 3. Geometry, Bifurcation Mechanics & Meshing

### 3.1 The Split Transition Zone & Gore Wedge
When a single road of width $W_{\text{trunk}}$ splits into two branches of widths $W_1$ and $W_2$, the road geometry must smoothly transition without creating visual pinching, geometric overlaps, or collision seams:

```
                  ┌───────────────────────────────────────────────────────────── Branch 1 (Left)
                  │                                                           /
                  │               /──────────────────────────────────────────/
                  │              /              (Gore Paved Area)
Entry Trunk ──────┤             /      ▲
                  │            /       │ Gore Nose Wedge / Impact Attenuator
                  │           /        ▼
                  │          /───────────────────────────────────────────────\
                  │                                                           \
                  └───────────────────────────────────────────────────────────── Branch 2 (Right)
                  |<─ L_trans ─>|
```

1. **Transition Length ($L_{\text{trans}}$)**:  
   The trunk centerline approaches the split bifurcation over an expansion distance $L_{\text{trans}} \ge \frac{W_1 + W_2}{2 \tan(\theta_{\text{max}})}$, where $\theta_{\text{max}} \le 18^\circ$ ensures high-speed stability without jarring lateral redirection.
2. **Width Envelope Expansion**:  
   Between the transition start ($s_0$) and the bifurcation apex ($s_{\text{split}}$), the drivable surface width widens smoothly using cubic Hermite interpolation:
   $$W(u) = W_{\text{trunk}} + (W_1 + W_2 + W_{\text{gore}}) \cdot (3u^2 - 2u^3), \quad u \in [0, 1]$$
3. **The Gore Area (`GoreConfig`)**:
   - The triangular paved runoff between diverging branches.
   - Surface Type: Standard Asphalt or Low-Friction Painted Runoff with chevron hash markings.
   - Inner Nose Apex: Equipped with an energy-absorbing crash attenuator (`BarrierType::TireWall` or deformable safety barrier) with rounded collision radius ($r \ge 1.0\,\text{m}$) to prevent single-vertex snagging when cars brush the apex.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoreConfig {
    /// Apex coordinate where the two inner edges diverge.
    pub apex_point: Vec2,
    /// Divergence angle between left and right branch centerlines in degrees.
    pub divergence_angle: f32,
    /// Length of the painted gore triangle in meters.
    pub gore_length: f32,
    /// Barrier installed at the gore apex (e.g. TireWall attenuator).
    pub nose_barrier: WallBarrier,
    /// Surface markings applied to the gore triangle (e.g. PaintedChevrons).
    pub has_chevrons: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MergeConfig {
    /// Point where incoming inner edges converge.
    pub convergence_point: Vec2,
    /// Merge angle in degrees.
    pub merge_angle: f32,
    /// Length of the zipper merge acceleration zone in meters.
    pub merge_length: f32,
}
```

### 3.2 Continuity Guarantees ($C^1$ Spline Stitching)
When extending a branch from a split socket:
- **First Waypoint Constraint**: Waypoint $0$ of the new branch spline is locked to `socket.point`.
- **First Waypoint Normal & Tangent**: Derived directly from `socket.tangent` and `socket.normal`.
- **Catmull-Rom Ghost Waypoint**: To ensure $C^1$ first-derivative smoothness at the branch seam, Catmull-Rom evaluation requires a preceding "ghost" point $P_{-1}$. This is synthesized analytically:
  $$P_{-1} = \text{socket.point} - \text{socket.tangent} \cdot \Delta d_{\text{step}}$$
  This prevents any kink, angular discontinuity, or derivative spike in lateral acceleration as cars transition from the split junction onto the extended branch.

---

## 4. Race Timing, Checkpoints & Lap Progression

### 4.1 Checkpoint Topology Across Splits
In a circuit with alternative routes, checkpoints are organized into **Shared Checkpoints** and **Branch Checkpoints**:

```
                       [CP 0: Start/Finish]
                                │
                                ▼
                       [CP 1: Pre-Split]
                                │
                        ┌───────┴───────┐
                     (Split)         (Split)
                        │               │
                        ▼               ▼
               [CP 2A: Main Line]     [CP 2B: Joker Lap]
                        │               │
                        └───────┬───────┘
                                ▼
                             (Merge)
                                │
                                ▼
                       [CP 3: Post-Merge]
```

### 4.2 Route Tracking in `TrackProgressTracker`
The progression tracker (`crates/arcade-race-core/src/track/checkpoint.rs`) is extended to track the **active route context**:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiRouteProgressTracker {
    /// Currently assigned or actively detected layout.
    pub active_layout_id: String,
    /// Current segment the car is traversing.
    pub current_segment_id: SegmentId,
    /// Index of next expected checkpoint in the active layout's sequence.
    pub next_checkpoint_index: usize,
    /// Number of joker laps completed by this driver.
    pub joker_laps_completed: u32,
    /// Whether the car is currently in a branch detour.
    pub is_in_branch: bool,
    /// Normalized progress along the currently active layout [0.0, 1.0).
    pub layout_progress: f32,
}
```

#### Dynamic Branch Detection
When a car approaches a split junction without a pre-locked layout (e.g., in a race with free Joker Lap choice):
1. The tracker evaluates trajectory proximity and gate crossing against both candidate branch checkpoints (`CP 2A` and `CP 2B`).
2. Crossing either gate automatically selects that branch's route branch for the remainder of that sector.
3. If `CP 2B` (Joker) is crossed:
   - The lap is flagged as a Joker Lap.
   - `joker_laps_completed` increments upon crossing the finish line.
   - The HUD displays `"JOKER LAP TAKEN"`.

---

## 5. AI Navigation & Decision Making (`BotAiDriver`)

### 5.1 AI Route Planning
In single-line tracks, `BotAiDriver` simply queries `spline.sample_at_distance((curr_dist + lookahead) % total_length)`. With branching circuits, the AI must handle **Branch Lookahead & Strategic Route Selection**:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BotRouteStrategy {
    /// Always follow a fixed, pre-selected track layout.
    FixedLayout(String),
    /// Rallycross: take the standard line except on designated joker lap(s).
    RallycrossJoker {
        /// Target lap number on which the bot plans to take the joker lap.
        planned_joker_lap: u32,
        /// Adaptive trigger: take joker early if stuck behind slower traffic.
        adaptive_traffic_undercut: bool,
    },
    /// Dynamic fork selection (e.g. choose path with fewer opponent cars).
    DynamicTrafficAvoidance,
}
```

### 5.2 Lookahead Horizon Across Junctions
When `curr_dist + lookahead_dist > current_segment.length`:
1. The AI queries the exit junction of the current segment.
2. It consults its `BotRouteStrategy` to determine the target egress socket.
3. The remaining lookahead distance $\Delta d = (\text{curr\_dist} + \text{lookahead}) - \text{segment.length}$ is smoothly projected onto the chosen downstream segment:
   $$\text{target\_sample} = \text{downstream\_segment.sample\_at\_distance}(\Delta d)$$
4. This produces unbroken, jitter-free steering target vectors across splits and merges.

---

## 6. Track Editor UX & Tooling Workflow (`tdrace-app`)

### 6.1 Editor Palette Integration
In `crates/tdrace-app/src/editor/tools.rs`, the tool palette is enhanced with **Branching Spline Tools**:

```
[1] Select & Move
[2] Road Spline (Main)
[3] Split Road Segment [NEW]
[4] Surfaces & Hazards
[5] Jump Ramp
[6] Obstacle Prop
[7] Checkpoint Gate
[8] Starting Grid
[9] Layout Manager [NEW]
```

### 6.2 Step-by-Step Editor Workflow

#### Step 1: Inserting a Road Split
1. Select the **Split Road Segment** tool (`[3]`).
2. Hover over any existing waypoint or segment spline line.
3. Click to insert a `RoadJunction::Split`:
   - The selected point becomes the bifurcation apex.
   - The editor prompts for the number of branches (default: 2; Branch A and Branch B).
   - Visual branch socket gizmos (cyan rings with directional tangent arrows) are rendered at the divergence boundary.

#### Step 2: Extending a Branch by Adding Splines
1. Click on **Branch Socket 1** (or any existing branch socket).
2. The active editing mode switches to **Extend Branch**:
   - The camera centers on the branch socket.
   - The socket’s initial position and tangent vector automatically seed the first spline segment.
3. **Right-Click in world space** to lay down sequential waypoints (`WP 1`, `WP 2`, `WP 3`...):
   - Waypoints inherit all standard spline properties: adjustable track width, left/right curbs, banked angles, elevations, and perimeter walls.
   - Catmull-Rom curvature updates dynamically in real-time as new waypoints are added.

#### Step 3: Rejoining the Track (Merge Junction)
1. When the branch route is complete, hover the final waypoint near an existing road segment or waypoint.
2. A **Snap & Merge** magnetic indicator appears (yellow circular target).
3. Click the target to create a `RoadJunction::Merge`:
   - The branch is cleanly stitched into the destination segment.
   - The inner and outer boundary walls automatically compute corner chamfers and curb transitions.

#### Step 4: Configuring Alternative Track Layouts (`Layout Manager`)
1. Open the **Layout Manager Modal** (`[9]`).
2. Click **"New Layout"** $\to$ Name: `"Joker Lap Route"`.
3. In the 2D CAD editor, click the segments that compose the route:
   `[Segment A] -> [Branch C (Joker)] -> [Segment D]`.
4. Click **"Validate Layout"**:
   - Automated check confirms $C^1$ tangent continuity at all junctions.
   - Validates that the route forms a closed loop and includes a Start/Finish checkpoint.
5. Save track. The JSON file is written with the complete `TrackNetwork`.

---

## 7. Serialization Schema & Backward Compatibility

### 7.1 Backward-Compatible `Track` Schema
To preserve existing JSON tracks (`tracks/classic/*.json`, `tracks/f1/*.json`, `tracks/rally/*.json`) without breaking existing parsers, the top-level `Track` struct in `arcade-race-core` supports both legacy and graph formats:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: TrackCategory,
    
    /// Legacy single-spline representation (preserved for backward compatibility).
    pub spline: TrackSpline,
    
    /// Optional advanced multi-branch track network.
    /// When present, runtime engines utilize the network graph;
    /// when absent, the single `spline` is automatically promoted to a 1-segment cyclic network.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<TrackNetwork>,
    
    pub geometry: TrackGeometry,
    pub checkpoints: Vec<Checkpoint>,
    pub grid_positions: Vec<SpawnPose>,
    pub default_surface: SurfaceType,
    pub pit_box_area: Option<SurfaceShape>,
    #[serde(default = "default_laps_fallback")]
    pub default_laps: u32,
    #[serde(default)]
    pub predefined_car: Option<String>,
    #[serde(default)]
    pub module_id: Option<String>,
    #[serde(default)]
    pub modules: Vec<String>,
}
```

### 7.2 Automatic In-Memory Promotion
When loading a legacy track with `network: None`, `Track::ensure_network()` constructs an equivalent single-segment `TrackNetwork`:
- 1 cyclic `RoadSegment` containing `track.spline.waypoints`.
- 1 default `TrackLayout` named `"main"` traversing that single segment.
- Zero breaking changes for existing unit tests, AI models, and rendering pipelines.

---

## 8. Implementation Roadmap & Milestones

| Milestone | Target Scope | Key Deliverables |
| :--- | :--- | :--- |
| **Phase 1: Core Graph & Math** | `arcade-race-core` | `TrackNetwork`, `RoadSegment`, `RoadJunction`, `SplineSocket`, $C^1$ tangent stitching, and legacy track auto-promotion. |
| **Phase 2: Editor Sockets & Spline Extension** | `tdrace-app::editor` | `RoadSplit` tool, branch socket gizmos, appending waypoints to branches, and branch merge snapping. |
| **Phase 3: Multi-Route Checkpoint & Timing** | `arcade-race-core` | `MultiRouteProgressTracker`, branch checkpoint validation, and Joker Lap detection. |
| **Phase 4: AI & Layout Selection** | `tdrace-app` | AI lookahead across split junctions, Rallycross Joker lap AI tactics, and in-game Layout Selection UI. |

---

## 9. Verification & Acceptance Criteria

### 9.1 Mathematical & Geometric Verification
- **$C^1$ Continuity Test**: Verify that the tangent angle difference $\Delta \theta$ across any `SplineSocket` between connecting segments is $< 10^{-4}$ radians.
- **Envelope Widening Test**: Verify that the cross-sectional width of the transition zone smoothly matches $W_{\text{trunk}} \to W_1 + W_2 + W_{\text{gore}}$ without negative derivative reversals.
- **Apex Attenuator SAT Test**: Verify that vehicles impacting the triangular gore apex collide with the attenuator barrier without penetrating between branch segments.

### 9.2 Editor UX Verification
- **Branch Extension**: A user can click a branch socket on a split segment, add 5 new waypoints, and verify that the branch renders with full curbs, walls, and proper normal offsets.
- **Circuit Layout Validation**: The editor validates and highlights alternative layouts in distinct palette colors (e.g., Gold for Main GP, Cyan for Joker Lap).

### 9.3 Simulation & Gameplay Verification
- **Multi-Car Branch Test**: Run a 4-car race where 2 bots take the Main Line and 2 bots take the Joker Lap branch. Verify that all cars complete their laps without desync, lap counter glitches, or wrong-way false alarms.
- **Backward Compatibility**: All existing tracks (`classic_grand_prix.json`, `rally/outlaw_pass.json`, `f1/*.json`) load and pass `validate_track()` with zero errors.
