---
type: Feature Spec
template: feature
title: "Track Studio Scenery Manipulation: Interactive Translation, Rotation, and Selection"
description: "Refactors Track Studio CAD editor to support selecting, translating, rotating, scaling, duplicating, and inspecting Buildings, Trees, Grandstands, and Rocks with interactive CAD gizmos, unified multi-spline selection across splits and pit lanes, and Inspector controls."
status: draft
created: 2026-10-23
generated: { by: agent/antigravity, at: 2026-10-23T12:50:22Z }
depends_on:
  - "012"
---

# Feature Spec: Track Studio Scenery Manipulation: Interactive Translation, Rotation, and Selection 🏗️🌲

When authoring, adjusting, or debugging circuits in Track Studio (`crates/tdrace-app/src/editor/`), authors can currently select and manipulate Waypoints, SurfaceZones, Obstacles, JumpRamps, Checkpoints, GridSlots, and PitBoxes. However, all decorative and structural scenery elements—specifically **Buildings**, **Grandstands**, **Trees**, and **Rocks**—are currently completely unselectable in the 2D CAD viewport. When scenery items are placed procedurally or imported from real-world OpenStreetMap survey data (such as the grandstand crossing the East Course turn in Utah Motorsport UMC RX), circuit designers have no interactive mechanism to select, move, rotate, scale, or inspect them, forcing manual edits to raw track JSON files.

This specification refactors the Track Studio entity selection system, hit-testing pipeline, interactive CAD manipulation gizmos, and right-hand Inspector panel to provide full first-class interactive control over all scenery elements.

---

## 🗺️ User Flow & Interface Design

### 1. Viewport Selection & Hit-Testing
- **Click-to-Select**:
  - Left-clicking on any scenery item in the 2D viewport selects it and opens its dedicated Inspector panel:
    - **Building**: Hit-tested against the oriented bounding box (OBB) defined by `center`, `size`, and `angle` (with a minimum $2.0\text{ m}$ grab margin).
    - **Grandstand**: Hit-tested against the OBB defined by `center`, `length`, `depth`, and `angle`.
    - **Tree**: Hit-tested against the radial canopy circle `(point - cursor).length() <= canopy_radius * scale`.
    - **Rock**: Hit-tested against the radial boulder collision boundary.
  - When overlapping elements exist (e.g. tree beside a building), clicking cycles selection through stacked entities at that point.
- **Selection Visual Feedback**:
  - Selected scenery items display a high-contrast CAD highlight outline:
    - **Buildings & Grandstands**: Accent bounding rectangle with corner resizing nodes, facing direction arrow indicator, and center translation crosshair.
    - **Trees & Rocks**: Dashed accent boundary ring with center pivot dot and radial scaling handle.
- **Multi-Selection (`Shift + Click` & Drag-Box)**:
  - Holding `Shift` while clicking toggles entity inclusion in multi-selection.
  - Dragging a selection marquee box captures scenery entities whose bounding boxes intersect the selection rectangle.

### 2. Interactive CAD Transformation Gizmos
- **Direct Translation**:
  - Clicking and dragging the body of any selected scenery item translates its `position` or `center`.
  - Translation snaps according to the active `SnapSettings` (`Off`, `1.0m`, `2.5m`, `5.0m`, `10.0m`).
  - Multi-selected entities translate rigidly together with preserved relative offsets.
- **Rotation Gizmo & Quick-Rotate (`R` Key)**:
  - **Rotation Handle**: Selected Buildings, Grandstands, and Rocks display an offset rotation handle node positioned along the entity's forward normal. Dragging this handle rotates the entity continuously about its center pivot.
  - **Keyboard Rotation**: Pressing `R` rotates selected entities clockwise by $+15^\circ$; pressing `Shift + R` rotates counter-clockwise by $-15^\circ$. Holding `Ctrl` snaps rotation to $45^\circ$ or $90^\circ$ increments.
  - **Tree Radial Scaling Handle**: Trees feature a radial scale handle at the canopy rim allowing intuitive dragging to scale canopy and trunk radius between $0.5\times$ and $3.0\times$.
- **Duplication & Deletion**:
  - Pressing `Ctrl + D` duplicates the selected scenery element with an offset of $+5.0\text{ m}$ in X and Y.
  - Pressing `Delete` or `Backspace` deletes the selected scenery item(s) with full undo/redo support.

### 3. Dedicated Scenery Inspector Panels
When a scenery element is selected, the right-hand Inspector displays specialized attribute controls:

#### Building Inspector
- **Position**: `X` and `Y` numeric input fields / steppers.
- **Dimensions**: `Width` (meters) and `Length` (meters).
- **Height**: Structural height in meters ($2.0\text{ m}..30.0\text{ m}$).
- **Rotation**: Heading angle in degrees ($0.0^\circ..359.9^\circ$) with $90^\circ$ orthogonal step buttons.
- **Style**: Cycler supporting `IndustrialWarehouse`, `PitGarage`, `ControlTower`, `Clubhouse`, and `StorageShed`.
- **Roof Color**: Palette swatch selector with RGB custom color picker.
- **Elevation**: Base ground elevation offset in meters.

#### Grandstand Inspector
- **Position**: `X` and `Y` coordinates.
- **Length & Depth**: Seating span length ($10\text{ m}..250\text{ m}$) and depth ($6\text{ m}..30\text{ m}$).
- **Tiers**: Spectator row tier count ($2..30$ tiers).
- **Rotation**: Facing orientation angle in degrees (ensuring spectator seats face the track).
- **Style**: Cycler supporting `OpenBleachers`, `CoveredStadium`, and `ConcreteTerrace`.
- **Seat Color**: Palette swatch picker for stadium seat coloring.
- **Elevation**: Base elevation offset in meters.

#### Tree Inspector
- **Position**: `X` and `Y` coordinates.
- **Species**: `TreeType` cycler (`Pine`, `Oak`, `Palm`, `Cypress`, `Birch`, `Bush`).
- **Scale**: Scale slider and numeric multiplier ($0.5\times..3.0\times$).
- **Elevation**: Base elevation offset in meters.

#### Rock Inspector
- **Position**: `X` and `Y` coordinates.
- **Rock Type**: `RockType` cycler (`Granite`, `Sandstone`, `Limestone`, `Boulder`, `PebbleCluster`).
- **Scale**: Scale multiplier ($0.5\times..3.0\times$).
- **Rotation**: Angular orientation in degrees.
- **Elevation**: Base elevation offset in meters.

### 4. Scenery Placement Tool (`EditorToolType::Scenery`)
- Adds a **Scenery Stamp Tool** to the Track Studio left toolbar:
  - Sub-mode buttons: `[Tree]`, `[Building]`, `[Grandstand]`, `[Rock]`.
  - Clicking on the canvas stamps a new instance of the active scenery prop with default parameters, immediately selecting it for adjustment.

### 5. Unified Multi-Spline Selection (`Ctrl + A` across all track branches)
- When selecting all splines in Track Studio (via `EditorToolType::RoadSpline`, `EditorToolType::RoadSplit`, `EditorToolType::LaunchChute`, or `select_all_splines`), selection is not limited to the main trunk spline: it automatically includes all split segments, joker laps, launch chutes (`branch_waypoints` in `TrackNetwork`), and pit lane splines (`state.track.pit_lane` / `pit_box_area`).
- Similarly, in `EditorToolType::Select` (global select all), all spline branches and pit lanes are selected alongside all other track entities.
- Translating or batch manipulating multi-selected splines moves main waypoints, branch waypoints, and the pit lane together rigidly.

---

## ⚙️ Backend Models & API Endpoints

### 1. Editor Selection State (`crates/tdrace-app/src/editor/state.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    None,
    Waypoint(usize),
    MultipleWaypoints(Vec<usize>),
    BranchWaypoint(SegmentId, usize),
    MultipleBranchWaypoints(Vec<(SegmentId, usize)>),
    SurfaceZone(usize),
    Obstacle(usize),
    JumpRamp(usize),
    Checkpoint(usize),
    GridSlot(usize),
    PitBox,
    Building(usize),
    Tree(usize),
    Grandstand(usize),
    Rock(usize),
    Multi {
        waypoints: Vec<usize>,
        branch_waypoints: Vec<(SegmentId, usize)>,
        surface_zones: Vec<usize>,
        obstacles: Vec<usize>,
        jump_ramps: Vec<usize>,
        checkpoints: Vec<usize>,
        grid_slots: Vec<usize>,
        buildings: Vec<usize>,
        trees: Vec<usize>,
        grandstands: Vec<usize>,
        rocks: Vec<usize>,
    },
}
```

### 2. Scenery Hit-Testing Pipeline (`crates/tdrace-app/src/editor/input.rs`)
```rust
impl EditorState {
    /// Tests for scenery element intersection at a given world-space coordinate.
    pub fn find_scenery_at_point(&self, point: Vec2, margin: f32) -> Option<Selection> {
        // 1. Buildings (OBB collision test)
        for b in &self.track.geometry.buildings {
            if is_point_in_oriented_rect(point, b.center, b.size + Vec2::splat(margin * 2.0), b.angle) {
                return Some(Selection::Building(b.id));
            }
        }
        // 2. Grandstands (OBB collision test)
        for g in &self.track.geometry.grandstands {
            let size = Vec2::new(g.length, g.depth) + Vec2::splat(margin * 2.0);
            if is_point_in_oriented_rect(point, g.center, size, g.angle) {
                return Some(Selection::Grandstand(g.id));
            }
        }
        // 3. Trees (Canopy radial distance test)
        for t in &self.track.geometry.trees {
            let radius = t.canopy_radius() + margin;
            if t.position.distance(point) <= radius {
                return Some(Selection::Tree(t.id));
            }
        }
        // 4. Rocks (Radial boundary distance test)
        for r in &self.track.geometry.rocks {
            let radius = r.bounding_radius() + margin;
            if r.position.distance(point) <= radius {
                return Some(Selection::Rock(r.id));
            }
        }
        None
    }
}
```

### 3. Command History & Mutation Operations (`crates/tdrace-app/src/editor/commands.rs`)
- Full undo/redo integration for scenery mutations:
  - `MoveSceneryCommand`: Records previous and updated positions for Buildings, Trees, Grandstands, and Rocks.
  - `RotateSceneryCommand`: Records previous and updated angles.
  - `ScaleSceneryCommand`: Records previous and updated scale factors.
  - `AddSceneryCommand` / `DeleteSceneryCommand`: Records creation and deletion of scenery instances with preserved unique IDs.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

Track Studio is a standalone local CAD authoring tool embedded within the client executable:
- **Dev Mode Authorization**:
  - Overwriting official catalog circuit files (`tracks/<module>/<slug>.json`) requires Dev Mode activation (`--dev` CLI flag or Dev Mode toggle in settings). In non-dev mode, saving official tracks is restricted to exporting as custom user tracks in the user data directory (`~/.local/share/tdrace/custom_tracks/`).
- **Data Integrity & Deserialization Safety**:
  - All scenery properties (angles, dimensions, scales, positions) are clamped to validated ranges ($0.5\text{ m}..500\text{ m}$) during JSON deserialization to prevent memory exhaustion or divide-by-zero panics in physics and spatial partition structures.

---

## 🛡️ Validation & Guardrails

- **Automatic Clearance Validation (`ERR_SCENERY_ON_TRACK`)**:
  - Translating or rotating any building, grandstand, rock, or tree so that its physical collision footprint intersects the drivable track spline or branch segments triggers an immediate real-time visual warning highlight in the Track Studio viewport.
  - Prevents accidental obstructions (such as the historical Utah UMC RX grandstand crossing bug).

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- Command to run Track Studio unit and integration tests:
  `cargo test -p tdrace-app --test track_editor_tests`
- Command to verify scenery collision and validation integrity:
  `cargo test -p arcade-race-core test_scenery`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario 1: Selecting and translating a building**
  - [ ] **Given** Track Studio is open with an official or custom circuit containing buildings
  - [ ] **When** the author left-clicks inside the bounding box of a building
  - [ ] **Then** the building is selected, displaying an accent outline and center translation crosshair
  - [ ] **When** the author drags the building across the canvas
  - [ ] **Then** the building's center coordinates update smoothly with grid snapping, and the change is registered in the undo/redo history

- **Scenario 2: Rotating a grandstand away from the track**
  - [ ] **Given** a grandstand selected in Track Studio (e.g. Utah UMC RX grandstand 1)
  - [ ] **When** the author drags the circular rotation gizmo handle or presses the `R` key
  - [ ] **Then** the grandstand rotates about its center pivot, seating orientation aligns smoothly, and the numeric angle updates in the Inspector

- **Scenario 3: Tree selection, species change, and scaling**
  - [ ] **Given** a tree selected in Track Studio
  - [ ] **When** the author selects a different species from the `TreeType` cycler and drags the canopy scale handle
  - [ ] **Then** the tree's visual canopy and physical collision radius update in real time

- **Scenario 4: Deleting and duplicating scenery**
  - [ ] **Given** one or more scenery items selected in the viewport
  - [ ] **When** the author presses `Ctrl + D`
  - [ ] **Then** duplicates of the selected items are instantiated at an offset, with unique IDs assigned
  - [ ] **When** the author presses `Delete`
  - [ ] **Then** the selected scenery items are removed from `track.geometry`, and pressing `Ctrl + Z` restores them

- **Scenario 5: Real-time track clearance warning**
  - [ ] **Given** a building or rock selected in Track Studio
  - [ ] **When** the author drags the item across the perimeter barrier onto the active track ribbon
  - [ ] **Then** the editor renders a pulsing amber warning border indicating `ERR_SCENERY_ON_TRACK`, warning the author before saving

- **Scenario 6: Unified Multi-Spline Selection Across Splits and Pit Lanes**
  - [ ] **Given** a track with a main spline, split/branch segments, and a pit lane
  - [ ] **When** the author activates the Road Spline, Road Split, or Launch Chute tool and triggers Select All (`Ctrl + A`)
  - [ ] **Then** all waypoints from the main spline, all waypoints from network branch segments, and the pit lane are simultaneously selected in a unified multi-selection, and can be translated, manipulated, or batched together

---

## 🔗 Traceability & Codebase Mapping

### Modified Files
- `[ ]` `crates/tdrace-app/src/editor/state.rs`: Extends `Selection` enum with `Building`, `Tree`, `Grandstand`, `Rock` and multi-selection fields.
- `[ ]` `crates/tdrace-app/src/editor/input.rs`: Implements scenery hit-testing, mouse dragging, rotation handle interaction, and keybindings.
- `[ ]` `crates/tdrace-app/src/editor/render.rs`: Implements selection wireframe boxes, corner handles, rotation arrows, and warning highlights.
- `[ ]` `crates/tdrace-app/src/editor/inspector.rs`: Adds dedicated Inspector UI sections for Buildings, Trees, Grandstands, and Rocks.
- `[ ]` `crates/tdrace-app/src/editor/tools.rs`: Adds Scenery stamp tool to the tool palette.
- `[ ]` `crates/tdrace-app/src/editor/commands.rs`: Implements undoable commands for scenery translation, rotation, scaling, and lifecycle.
- `[ ]` `crates/tdrace-app/tests/track_editor_tests.rs`: Comprehensive test suite verifying scenery selection, gizmo manipulation, duplication, and undo/redo.
- `[ ]` `specs/constitution/ROADMAP.md`: Links Spec 138 in Phase 4.
- `[ ]` `specs/index.md`: Registers Spec 138 in the progressive specification index.
