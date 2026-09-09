# Circuit Rendering & Menu Performance Report

## Executive Summary

During testing of the circuit catalog and long mixed-surface circuits (specifically **Höljes Motorstadion** in Sweden), two prominent performance issues were observed:
1. **Menu Navigation Lag**: Severe pauses and dropped frames when navigating the circuit catalog and switching tracks in both the Track Selection Menu and the Track Manager.
2. **In-Game Frame Drops & Input Lag**: Significant stutter and sluggish vehicle response during active gameplay on circuits with extensive tire barrier perimeters.

This document outlines the root causes detected across the rendering pipeline, procedural generation engine, and physics collision loop, followed by the architectural optimizations applied to resolve them.

---

## Issue 1: Circuit Selection Menu Lag & Stutter

### Root Cause Analysis
- In [`render_track_select_menu`](../crates/tdrace-app/src/ui/menu.rs) and [`render_track_manager_screen`](../crates/tdrace-app/src/ui/track_manager_ui.rs), `resolve_track_for_menu` was called **every single frame (60 FPS)** for:
  - Every visible circuit card in the scrolling list (up to 6 items).
  - The currently highlighted circuit dossier telemetry preview.
- For procedurally generated circuits like Höljes Motorstadion (`holjes_rx`), each call executed:
  - Resampling 30 Catmull-Rom spline waypoints into 721 fine samples.
  - Generating 1,440 inner and outer wall barriers.
  - Polyline untangling and crossroad trimming.
  - `trim_corner_intersections`, which ran an $O(N^2)$ nested intersection search across all 1,440 wall segments (~1,000,000 candidate segment pairs), with full spline projection calls inside the loop.
- A single build of Höljes required **~200 ms of pure CPU time**. Executing 6–7 procedural track builds per frame 60 times a second resulted in severe frame drops (down to single-digit FPS) whenever browsing the catalog.

### Fix Applied
- **Thread-Safe In-Memory Caching**:
  - Implemented `MENU_TRACK_CACHE` (a `Mutex<HashMap<String, Track>>`) inside [`crates/tdrace-app/src/ui/menu.rs`](../crates/tdrace-app/src/ui/menu.rs).
  - `resolve_track_for_menu_with_dir` now queries the cache before running the procedural generator or file loader.
  - The first access builds or loads the track once; subsequent frames retrieve the cached circuit in **46 microseconds** (**~4,350× speedup**).
- **Cache Invalidation Hooks**:
  - Provided `clear_menu_track_cache()`.
  - Wired into [`TrackManager`](../crates/tdrace-app/src/track_manager.rs) to automatically clear the cache whenever custom tracks are edited, saved, or deleted.

---

## Issue 2: In-Game Frame Rate Drops on Long Circuits (Höljes RX)

### Root Cause Analysis
1. **Tire Wall Rendering Overhead**:
   - Höljes Motorstadion is an FIA rallycross circuit (~1,210 m) with 1,440 wall segments configured with `BarrierType::TireWall`.
   - In [`crates/tdrace-app/src/render/barrier.rs`](../crates/tdrace-app/src/render/barrier.rs), `BarrierType::TireWall` rendered individual tires along each segment:
     - For each segment, it ran a loop using `draw_circle`, `draw_circle_lines` (which tessellated 30–40 line quads per circle), and `draw_circle`.
     - Across 1,440 segments, this generated **12,960 circle draw calls** and **over 500,000 triangles every single frame** into Macroquad's immediate-mode CPU vertex buffer.
2. **Lack of Viewport Frustum Culling**:
   - Neither [`render_ground_track`](../crates/tdrace-app/src/render/track.rs) nor [`render_ground_barriers_and_obstacles`](../crates/tdrace-app/src/render/barrier.rs) culled off-screen elements.
   - Even though the player camera only views ~5% of the total track area, 100% of all spline quads, curbs, drop shadows, and 1,440 wall barriers across the entire 1.2 km facility were tessellated and submitted every frame.

### Fix Applied
1. **Streamlined Tire Barrier Geometry**:
   - Replaced the high-overhead `draw_circle_lines` loop with a continuous rubber barrier quad (`draw_quad`), a top rubber binder strap (`draw_line`), and transverse tire division ribs.
   - Matches the visual appearance of FIA conveyor-belt-wrapped rallycross tire barriers while cutting triangle counts and draw calls by >90%.
2. **Camera Viewport Culling**:
   - Added `visible_world_bounds(&self, margin)` to [`RaceCamera`](../crates/tdrace-app/src/camera/mod.rs) to compute the visible axis-aligned world bounds plus a margin.
   - Implemented `render_ground_track_culled`, `render_elevated_track_culled`, and `render_ground_barriers_and_obstacles_culled`.
   - Off-screen spline segments, curbs, drop shadows, and wall barriers are rejected before emitting draw calls.

---

## Issue 3: Physics Collision & Surface Sampling CPU Overhead

### Root Cause Analysis
1. **Narrowphase-Only Wall Collision Testing**:
   - In [`resolve_car_wall_collision`](../crates/tdrace-core/src/collision/wall.rs), up to 8 cars across 2 sub-iterations tested against all 1,440 walls (23,040 tests/tick).
   - Each test computed `OrientedBox::from_car`, 4 corner projections, normal vector math, and 5 square-root distance calculations without an early-out check, even for walls hundreds of meters away.
2. **Full Spline Linear Scan for Wheel Surfaces**:
   - In [`crates/tdrace-app/src/game/mod.rs`](../crates/tdrace-app/src/game/mod.rs), `sample_car_surfaces` was called for 8 cars * 4 wheels = 32 queries/tick.
   - Each query called `spline.project_point(wheel_pt)`, which performed an unconstrained linear scan over all 721 spline samples (23,072 iterations/tick), even though the vehicle's track progress distance was already known.

### Fix Applied
1. **Broadphase AABB Early-Out**:
   - Added a 4-comparison bounding box test to [`resolve_car_wall_collision`](../crates/tdrace-core/src/collision/wall.rs).
   - Distant walls (>3.6 m from vehicle center) are immediately rejected before computing oriented boxes, corner SAT tests, or square roots, eliminating >99% of computation.
2. **Hinted Surface Sampling**:
   - Added `sample_car_surfaces_with_hint` and `sample_surface_near` in [`crates/tdrace-core/src/track/mod.rs`](../crates/tdrace-core/src/track/mod.rs).
   - Constrained spline projection to a localized 45 m window around the car's known progress distance (`spline.project_point_continuity`), replacing full 721-sample scans with small localized checks.

---

## Modified Codebase Files

| Component | File Path | Changes |
|---|---|---|
| **Menu Cache** | [`crates/tdrace-app/src/ui/menu.rs`](../crates/tdrace-app/src/ui/menu.rs) | In-memory `MENU_TRACK_CACHE`, `clear_menu_track_cache()`, cached resolution. |
| **Track Manager UI** | [`crates/tdrace-app/src/ui/track_manager_ui.rs`](../crates/tdrace-app/src/ui/track_manager_ui.rs) | Use cached track resolver for catalog cards and dossier previews. |
| **Track Manager** | [`crates/tdrace-app/src/track_manager.rs`](../crates/tdrace-app/src/track_manager.rs) | Invalidate menu track cache on custom track save / delete. |
| **Camera** | [`crates/tdrace-app/src/camera/mod.rs`](../crates/tdrace-app/src/camera/mod.rs) | `visible_world_bounds` for world-space view frustum calculations. |
| **Barrier Rendering** | [`crates/tdrace-app/src/render/barrier.rs`](../crates/tdrace-app/src/render/barrier.rs) | Efficient `TireWall` quad/strap/ribs rendering; `is_wall_in_view` viewport culling. |
| **Track Rendering** | [`crates/tdrace-app/src/render/track.rs`](../crates/tdrace-app/src/render/track.rs) | Viewport culling for ground track ribbons, bridges, and curb rumble passes. |
| **Game Loop** | [`crates/tdrace-app/src/game/mod.rs`](../crates/tdrace-app/src/game/mod.rs) | Hinted wheel surface sampling; pass camera view bounds to rendering passes. |
| **Physics Collision** | [`crates/tdrace-core/src/collision/wall.rs`](../crates/tdrace-core/src/collision/wall.rs) | Broadphase AABB early-out in `resolve_car_wall_collision`. |
| **Surface Query** | [`crates/tdrace-core/src/track/mod.rs`](../crates/tdrace-core/src/track/mod.rs) | `sample_car_surfaces_with_hint` and `sample_surface_near`. |

---

## Verification & Benchmarks

- **Menu Track Resolution**:
  - Uncached initial generation (Höljes RX): **~201.8 ms**
  - Cached resolution: **~46.3 µs** (**4,350× speedup**)
- **Test Suite Results**:
  - `cargo test --test track_preview_tests`: Passed (including `test_menu_track_cache_performance_and_consistency`).
  - `cargo test --test rally_tracks_tests`: All 9 rally tests passed.
  - `cargo test --test collision_tests`: All 9 collision tests passed.
  - `cargo test --test track_manager_tests`: All 28 track manager tests passed.
  - 100% of workspace tests passing with zero regressions.
