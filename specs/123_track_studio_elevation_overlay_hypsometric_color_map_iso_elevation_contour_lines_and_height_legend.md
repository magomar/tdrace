---
type: Feature Spec
template: feature
title: "Track Studio Elevation Overlay: Hypsometric Color Map, Iso-Elevation Contour Lines, and Height Legend"
description: "Adds an interactive elevation visualization overlay to Track Studio toggled with the E key, rendering a translucent hypsometric color-map ribbon, 0.5m/2.0m topographic iso-elevation contour lines with embankment skirts, per-waypoint height/grade labels, and a screen-space elevation legend."
status: implemented
created: 2026-10-18
generated: { by: agent/antigravity, at: 2026-10-18T17:48:30Z }
verified: { by: human:mario, at: 2026-10-18T18:38:48Z, hash: "34139db32906" }
depends_on:
  - "086"
---

# Feature Spec: Track Studio Elevation Overlay: Hypsometric Color Map, Iso-Elevation Contour Lines, and Height Legend 🏔️

When designing or debugging circuits in the Track Studio CAD editor—such as verifying whether wall suppression gaps, bridge clearances (`elevation >= 0.6 m`), downhill braking zones, or rallycross jump transitions are caused by elevation changes—authors currently have no direct visual representation of elevation across the 2D top-down viewport. Waypoint elevation (`wp.elevation`) is only visible one node at a time by clicking each waypoint and reading the Inspector stepper, making it difficult to judge overall terrain relief, longitudinal grade slopes, banked cross-slopes, or flat vs. elevated sections at a glance.

This specification introduces a toggleable **Track Studio Elevation Overlay** (`E` key) combining a translucent hypsometric color-map overlay across all track ribbons and ramps, topographic iso-elevation contour lines ($0.5\text{ m}$ minor / $2.0\text{ m}$ major index contours) with lateral embankment skirts, screen-space waypoint height and grade badges, and a compact corner elevation legend.

---

## 🗺️ User Flow & Interface Design

### 1. Activation & Overlay Modes
- **Primary Toggle (`E` Key)**:
  - Pressing `E` in Track Studio (when no text input or modal dialog is active) toggles the Elevation Overlay between `Off` and `ColorAndContours` (combined hypsometric color map + topographic contour lines + height legend).
- **Sub-Mode Cycling (`Shift + E`)**:
  - Pressing `Shift + E` cycles through the four `ElevationOverlayMode` variants on `ToolSettings`:
    1. `Off`: Standard Track Studio surface texture view (default).
    2. `ColorAndContours`: Translucent hypsometric color-map ribbon + $0.5\text{ m} / 2.0\text{ m}$ iso-elevation contour lines + waypoint height labels + legend card.
    3. `ColorMapOnly`: Translucent hypsometric color-map ribbon + waypoint height labels + legend card (without contour lines).
    4. `ContoursOnly`: Topographic iso-elevation contour lines over native road textures + waypoint height labels + legend card.
- **Status Bar & Tooltip Discoverability**:
  - The Track Studio top/bottom status bar displays `[E] Elevation: OFF / ALL / COLOR / CONTOURS` so authors can discover and monitor the active overlay state at any time.

### 2. Hypsometric Color Map Ribbon Overlay
- **Circuit Elevation Range Resolution**:
  - Computes $(z_{\min}, z_{\max})$ across all `SplineSample`s of `track.spline`, `track.network` branch segments, `track.pit_lane`, and the peak surface heights of `track.geometry.jump_ramps`.
  - **Flat-Circuit Reference Scale**: When a circuit is completely flat ($z_{\max} - z_{\min} < 0.1\text{ m}$, e.g. all waypoints at $0.0\text{ m}$), the color scale anchors to $[0.0\text{ m}, 10.0\text{ m}]$ so that $0.0\text{ m}$ renders uniformly in the ground-baseline color rather than stretching numerical noise across the palette. When $z_{\max} - z_{\min} \ge 0.1\text{ m}$, the scale spans $[z_{\min}, \max(z_{\max}, z_{\min} + 2.0\text{ m})]$.
- **5-Stop Perceptual Hypsometric Palette (`hypsometric_color(t: f32)`)**:
  - Normalized height $t \in [0.0, 1.0]$ maps smoothly via piecewise-linear RGB interpolation across five high-contrast stops:
    - $t = 0.00$ (Ground / Base): Deep Emerald-Teal `Color::new(0.00, 0.72, 0.58, alpha)` (`#00B894`)
    - $t = 0.25$ (Low Rise): Chartreuse Green `Color::new(0.53, 0.88, 0.25, alpha)` (`#88E040`)
    - $t = 0.50$ (Mid Elevation): Neon Amber-Gold `Color::new(1.00, 0.70, 0.00, alpha)` (`#FFB300`)
    - $t = 0.75$ (High Slope): Velocity Coral-Orange `Color::new(1.00, 0.34, 0.13, alpha)` (`#FF5722`)
    - $t = 1.00$ (Peak / Bridge Crest): Electric Magenta-Rose `Color::new(1.00, 0.16, 0.52, alpha)` (`#FF2A85`)
- **Banking-Aware Cross-Section Vertex Coloring**:
  - For each sample $s_i$ with centerline elevation $z_i$, width $w_i$, and banking angle $\theta_i$ (`bank_angle` in degrees), the left and right road edge elevations are evaluated as:
    $$z_{i,\text{left}} = z_i - \frac{w_i}{2} \sin(\theta_i), \qquad z_{i,\text{right}} = z_i + \frac{w_i}{2} \sin(\theta_i)$$
  - Each road quad is split along the centerline into left and right triangles/quads with per-vertex colors $(c_{\text{left}}, c_{\text{center}}, c_{\text{right}})$ at $\alpha = 0.58$, visually revealing both longitudinal elevation changes and lateral banking tilt.

### 3. Topographic Iso-Elevation Contour Lines
- **Contour Intervals**:
  - **Minor Contours**: Every $\Delta z_{\text{minor}} = 0.5\text{ m}$. Drawn as crisp translucent lines (`thickness = 0.22 m`, $\alpha = 0.65$).
  - **Major Index Contours**: Every $\Delta z_{\text{major}} = 2.0\text{ m}$ (i.e. $2.0\text{ m}, 4.0\text{ m}, 6.0\text{ m}, \dots$). Drawn as bold lines (`thickness = 0.42 m`, $\alpha = 0.92$) with short downhill hachure tick marks pointing toward lower elevation.
- **Cross-Ribbon Iso-Height Segments**:
  - Across each sample interval $[s_i, s_{i+1}]$ on `track.spline`, `track.network` branch segments, `track.pit_lane`, and `JumpRamp` footprints, every contour level $z_k = k \cdot 0.5\text{ m}$ that intersects the 4-corner elevation quad $(z_{i,L}, z_{i,R}, z_{i+1,R}, z_{i+1,L})$ is extracted via marching-squares quad edge interpolation and rendered in world space.
- **Embankment Topographic Skirts**:
  - Where the road ribbon is elevated above the surrounding ground ($z > 0.5\text{ m}$), intermediate contour levels $z_k \in (0, z_{\text{edge}})$ project outward along the sample normal $\pm \mathbf{n}_i$ at lateral offset $d_k = (z_{\text{edge}} - z_k) \cdot 1.5$ (representing a $1.5:1$ embankment slope), forming continuous topographic rings beside elevated hills and bridge ramps.

### 4. Screen-Space Waypoint Height Badges & Corner Legend Card
- **Waypoint Height & Grade Badges**:
  - In the screen-space editor label pass, each waypoint on the main spline and network branches renders a compact label next to the node showing `"{:.1}m"` (e.g. `0.0m`, `3.5m`) and, when longitudinal slope $|\text{grade}| \ge 1.0\%$, the signed grade percentage (e.g. `+4.5%` or `-3.2%`).
- **Corner Elevation Legend Card**:
  - Drawn in the bottom-left of the Track Studio canvas (above the status bar, avoiding the left tool palette and right inspector):
    - Displays the title `ELEVATION OVERLAY [E]` and active sub-mode (`COLOR + CONTOURS`, `COLOR MAP`, or `CONTOURS`).
    - Renders a vertical or horizontal multi-stop color gradient bar labeled with 5 metric elevation ticks from $z_{\min}$ to $z_{\max}$ (e.g. `0.0m`, `2.5m`, `5.0m`, `7.5m`, `10.0m`).
    - Shows summary circuit elevation telemetry: `Min: X.Xm · Max: Y.Ym · Max Grade: Z.Z%` (and an explicit `FLAT CIRCUIT (0.0m)` badge when all samples are at $0.0\text{ m}$).

---

## ⚙️ Backend Models & API Endpoints

### 1. Editor State & Tool Settings (`crates/tdrace-app/src/editor/tools.rs`)
```rust
/// Visualization mode for the Track Studio elevation overlay (Spec 123).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ElevationOverlayMode {
    #[default]
    Off,
    ColorAndContours,
    ColorMapOnly,
    ContoursOnly,
}

impl ElevationOverlayMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Off => Self::ColorAndContours,
            _ => Self::Off,
        }
    }

    pub fn cycle_submode(self) -> Self {
        match self {
            Self::Off | Self::ContoursOnly => Self::ColorAndContours,
            Self::ColorAndContours => Self::ColorMapOnly,
            Self::ColorMapOnly => Self::ContoursOnly,
        }
    }

    pub fn shows_color_map(self) -> bool {
        matches!(self, Self::ColorAndContours | Self::ColorMapOnly)
    }

    pub fn shows_contours(self) -> bool {
        matches!(self, Self::ColorAndContours | Self::ContoursOnly)
    }
}
```

### 2. Pure Mathematical Helpers
- `compute_track_elevation_bounds(track: &Track) -> (f32, f32, f32)`:
  - Returns `(min_elev, max_elev, max_grade_pct)` across `track.spline`, `track.network` branches, `track.pit_lane`, and `track.geometry.jump_ramps`.
- `hypsometric_color(elevation: f32, min_elev: f32, max_elev: f32, alpha: f32) -> Color`:
  - Pure deterministic mapping from metric elevation to RGBA color using the 5-stop hypsometric ramp.
- `quad_iso_contour_segment(p00: Vec2, z00: f32, p10: Vec2, z10: f32, p11: Vec2, z11: f32, p01: Vec2, z01: f32, level: f32) -> Option<(Vec2, Vec2)>`:
  - Extracts the 2D line segment where a 4-vertex road quad intersects elevation `level`.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

- **Zero Impact on Headless Physics & Track Schema**:
  - No changes to `wheelbase`, `race-kit`, or the `tracks/*/*.json` serialization schema.
  - All overlay state (`elevation_overlay: ElevationOverlayMode`) lives exclusively in `crates/tdrace-app/src/editor/`.
- **Viewport Culling & 60 FPS Editor Responsiveness**:
  - Ribbon color quads and contour segments are culled against the active `EditorCamera` world bounding box (`view_bounds`) so zoomed-in editing on large circuits maintains 60 FPS with zero frame-time spikes.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- Run Track Studio unit & integration tests:
  `cargo test -p tdrace-app --test track_editor_tests`
- Run workspace-wide Rust test suite:
  `cargo test --workspace --exclude tdrace-py`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Toggling and cycling the elevation overlay in Track Studio**
  - [x] **Given** a circuit open in Track Studio with `elevation_overlay` initially set to `ElevationOverlayMode::Off`
  - [x] **When** the author presses the `E` key
  - [x] **Then** `elevation_overlay` transitions to `ElevationOverlayMode::ColorAndContours` and pressing `E` a second time returns it to `ElevationOverlayMode::Off`
  - [x] **And** pressing `Shift+E` cycles sequentially through `ColorAndContours`, `ColorMapOnly`, and `ContoursOnly`

- **Scenario: Flat circuit displays uniform baseline color and flat telemetry badge**
  - [x] **Given** a flat circuit such as Lydden Hill where all waypoints have `elevation: 0.0` and no jump ramps exceed `0.0` m
  - [x] **When** the elevation overlay is enabled (`ColorAndContours` or `ColorMapOnly`)
  - [x] **Then** `compute_track_elevation_bounds` reports `min_elev = 0.0` and `max_elev = 0.0` and the scale uses the `[0.0m, 10.0m]` reference range without NaN or division by zero
  - [x] **And** the entire track ribbon renders in the uniform ground-level emerald-teal color (`t = 0.0`) with `0.0m` waypoint badges

- **Scenario: Elevated circuit renders hypsometric gradient and 0.5m/2.0m iso-elevation contour lines**
  - [x] **Given** a circuit containing waypoints rising from `0.0` m to `6.0` m (or a jump ramp with `height > 1.0` m)
  - [x] **When** the elevation overlay is active in `ColorAndContours` mode
  - [x] **Then** the ribbon color map smoothly transitions through emerald, chartreuse, amber, coral, and magenta stops proportional to sample elevation
  - [x] **And** cross-ribbon iso-elevation contour segments are generated at every `0.5` m minor step and `2.0` m major index step along the slope

- **Scenario: Banked turn tilts iso-elevation contours across the road width**
  - [x] **Given** a spline segment with non-zero banking (`bank_angle = 15.0` degrees) and road width `14.0` m
  - [x] **When** edge elevations and iso-elevation contours are evaluated across the segment
  - [x] **Then** the left and right road edges have distinct elevations offset by `±(width / 2) * sin(bank_angle)`
  - [x] **And** the extracted contour line crosses diagonally across the banked road quad rather than perpendicular to the centerline

- **Scenario: Corner elevation legend card renders scale ticks and circuit statistics**
  - [x] **Given** the elevation overlay is active in Track Studio (`ColorAndContours`, `ColorMapOnly`, or `ContoursOnly`)
  - [x] **When** the editor UI pass renders
  - [x] **Then** a legend card is drawn in the bottom-left viewport displaying the 5-stop color bar, metric elevation ticks, minimum/maximum elevation, and maximum grade percentage

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[x]` `crates/tdrace-app/src/editor/tools.rs` $\to$ Adds `ElevationOverlayMode`, `compute_track_elevation_bounds`, `hypsometric_color`, `quad_iso_contour_segment`, world-space ribbon color/contour pass, and screen-space waypoint height badges.
- `[x]` `crates/tdrace-app/src/editor/ui.rs` $\to$ Renders the bottom-left Elevation Legend Card and status bar `[E] Elevation` indicator.
- `[x]` `crates/tdrace-app/src/game/mod.rs` $\to$ Wires `KeyCode::E` and `Shift + KeyCode::E` in `update_track_editor` and invokes the elevation overlay passes in `render_track_editor`.
- `[x]` `crates/tdrace-app/tests/track_editor_tests.rs` $\to$ Unit and integration tests verifying `ElevationOverlayMode` transitions, flat vs. elevated bounds, hypsometric color stops, and banked/unbanked marching-quad contour extraction.

### Verification Assertions
- `crates/tdrace-app/src/editor/tools.rs` and `crates/tdrace-app/tests/track_editor_tests.rs` reference `Spec 123` in their doc comments.
