---
type: Architecture Spec
title: "Circuit Building & Environment Analysis"
description: "As-built analysis of how TdRace builds tracks, runoff, walls, scenery and real-world OSM circuits, with the defects and gaps that block realism."
status: active
category: engineering
tags: [circuits, tracks, osm, scenery, runoff, rendering, analysis]
---

# Circuit Building & Environment Analysis

This report describes how circuits are built **today** (commit `884a99b`, 2026-09-27).
It covers the track data model, the environment around the track, and the
OpenStreetMap (OSM) import procedure. It ends with a list of defects and gaps.

The companion document [Circuit Realism Knowledge Base](circuit_realism_knowledge_base.md)
holds the external knowledge (OSM tagging, FIA design rules, visual design) and the
improvement plan.

All paths are relative to the repository root. Line numbers are for `884a99b`.

---

## 0. Summary

1. **One centreline per track.** A track is one Catmull-Rom spline of `TrackWaypoint`s.
   Everything else (ribbon, kerbs, runoff, walls) is an offset from that spline.
   There is no pit lane road, no branch, and no second spline.
2. **The data model can already do per-side, per-waypoint runoff.** Each waypoint has
   `left/right_wall_distance` and `left/right_runoff_surface`. But only
   **6 waypoints in all 96 circuits** use them. In practice every circuit has one
   wall offset, one wall type and one runoff surface for the whole lap.
3. **The environment is not generated.** Scenery (trees, grandstands) is typed in
   by hand with absolute coordinates. It exists on 11–14 of the core presets. **The
   18 GT circuits have no scenery, no surface zones and no pit box.**
4. **The OSM import uses only the centreline.** Four Python scripts read
   `highway=raceway` node positions. They throw away barriers, gravel traps,
   grandstands, pit lanes, trees and buildings, even when that data is in the cache.
5. **GT and NASCAR road courses are built at 0.5× length but at real width.** They are
   labelled `scale: "1:1"`. This halves the lateral space between track sections.
   Any OSM distance (runoff, grandstand offset) must be scaled by the same factor.
6. **Several bugs** make the current OSM output less accurate than it could be
   (GT kerbs on almost every waypoint, broken gap guard in way stitching). See §6.

---

## 1. Track data model

### 1.1 `Track` — [track/mod.rs:120-159](../../crates/arcade-race-core/src/track/mod.rs)

| Field | Type | Notes |
|---|---|---|
| `spline` | `TrackSpline` | Centreline and baked samples. |
| `geometry` | `TrackGeometry` | Walls, zones, obstacles, ramps, scenery. |
| `kind` | `TrackKind` | `Circuit` (default), `Arena{boundary_hull, floor_surface, perimeter_barrier}`, `Hybrid{…}`. |
| `checkpoints` | `Vec<Checkpoint>` | Timing gates. Can carry pit entry/exit flags. |
| `grid_positions` | `Vec<SpawnPose>` | Start grid. |
| `default_surface` | `SurfaceType` | Backdrop terrain. Default `Grass`. |
| `pit_box_area` | `Option<SurfaceShape>` | The only pit concept. Used only by `classic_grand_prix`. |
| `scale` | `String` | Label only. Default `"1:1"`. Not used in any calculation. |
| `osm_url`, `wikipedia_url`, `country_*` | `Option<String>` | Filled at runtime from `provenance.rs` by name. |

### 1.2 `TrackWaypoint` — the authoring unit — [spline.rs:15-54](../../crates/arcade-race-core/src/track/spline.rs)

| Field | Default | Meaning |
|---|---|---|
| `point: Vec2` | — | Centreline node, in metres. |
| `width: f32` | — | Full drivable width. Kerbs sit **outside** this width. |
| `left_curb`, `right_curb: bool` | `false` | Kerb on that side. |
| `surface: Option<SurfaceType>` | `None` = Asphalt | Ribbon surface. |
| `elevation: f32` | `0.0` | Metres. Clamped to ≥ 0 after interpolation. |
| `bank_angle: f32` | `0.0` | Degrees. Positive = right side higher. |
| `left_wall`, `right_wall: bool` | **`true`** | Wall exists on that side. |
| `left_wall_distance`, `right_wall_distance: Option<f32>` | `None` | Gap from track edge to wall. `None` = global offset. |
| `wall_type: Option<BarrierType>` | `None` | **One value for both sides.** |
| `left_runoff_surface`, `right_runoff_surface: Option<SurfaceType>` | `None` | Surface between track edge and wall. |

Builders: `with_curbs`, `with_walls`, `with_wall_distance(s)`, `with_wall_type`,
`with_surface`, `with_elevation`, `with_bank_angle`, `with_runoff_surface(s)` (`spline.rs:57-135`).

### 1.3 From waypoints to samples — `TrackSpline::new` ([spline.rs:263-602](../../crates/arcade-race-core/src/track/spline.rs))

- **Spline:** uniform Catmull-Rom, tension 0.5 (`catmull_rom_2d`, `:1047`). Not centripetal.
  Uniform Catmull-Rom overshoots when waypoint spacing is uneven.
- **Sampling:** `steps_per_segment = 24`, uniform in *t* (not in arc length).
  Sample spacing = waypoint spacing / 24: about 4 m on Monza, 1.6 m on karts, 7.6 m on Le Mans.
- **Per-attribute interpolation** (`:350-391`):

  | Attribute | Interpolation |
  |---|---|
  | position, `elevation`, `bank_angle` | Catmull-Rom (`elevation` clamped ≥ 0) |
  | `width` | linear |
  | `*_wall_distance` | linear if **both** neighbours define it; otherwise it **jumps** at t = 0.5 |
  | kerbs, walls, `wall_type`, runoff surfaces | nearest waypoint (t < 0.5 → first) |
  | `surface` | segment-start waypoint |

- **Normals** point **left**: `normal = (-t.y, t.x)` (`:560-567`).
- **Bridges** are detected automatically: crossing samples with ≥ 2.5 m height difference (`:441-517`).
- **Curves** (`TrackCurve`, [curve.rs:25-48](../../crates/arcade-race-core/src/track/curve.rs))
  are extracted after sampling: `direction`, `degree` 1–5, `entry_distance`,
  `apex_distance`, `exit_distance`, `min_radius`. Threshold |κ| > 0.007 (R < 142 m).
  Degree bands by radius: ≤18 m → 5, ≤32 → 4, ≤55 → 3, ≤85 → 2, else 1.
  **This is the best hook for automatic placement** of kerbs, gravel traps and grandstands.

Constants: `DEFAULT_CURB_WIDTH = 1.4` m (`:246`), `DEFAULT_WALL_DISTANCE = 8.0` m (`:249`).

### 1.4 `TrackGeometry` — [geometry.rs:998-1011](../../crates/arcade-race-core/src/track/geometry.rs)

| Field | Content |
|---|---|
| `inner_walls` / `outer_walls` | `Vec<WallBarrier>` = **left / right** walls (historic names). |
| `obstacles` | `Obstacle{shape: Circle/Box/Polygon, restitution, friction, name, elevation}`. |
| `surface_zones` | `SurfaceZone{shape, surface, name, layer: BelowTrack/AboveTrack}`. |
| `jump_ramps` | `JumpRamp{…}`. |
| `left/right_boundary_polyline` | Wall offset lines. Stored, not read by gameplay. |
| `grandstands`, `trees` | Scenery (§2.3). |

### 1.5 Barrier types — [geometry.rs:149-235](../../crates/arcade-race-core/src/track/geometry.rs)

| Type | restitution | friction | scrape decel (m/s²) | energy absorption |
|---|---|---|---|---|
| `Concrete` | 0.65 | 0.32 | 9 | 0.10 |
| `Steel` (armco) | 0.42 | 0.45 | 14 | 0.40 |
| `TireWall` | 0.18 | 0.80 | 24 | 0.75 |
| `CurbWall` | 0.30 | 0.40 | 7 | 0.25 |
| `Virtual` | 0 | 0 | 0 | 1.0 (not physical) |

`docs/physics/walls_barriers.md` lists older values. The code values above are correct.
There is no fence, catch fence, Tecpro, SAFER or layered barrier.

---

## 2. How the environment is built

### 2.1 Cross-section, from the centre out

```
 backdrop (default_surface) | wall | runoff corridor | kerb 1.4 m | ribbon (width) | kerb | runoff | wall | backdrop
                                   <-- wall_distance -->
```

- The runoff corridor starts at the **track edge** and ends at the wall.
  The kerb is drawn inside the corridor.
- **Default runoff surface** — `Track::default_runoff_surface()` ([mod.rs:548-615](../../crates/arcade-race-core/src/track/mod.rs)):
  one surface for the whole lap, picked from name, module and `default_surface`:

  | Condition | Runoff |
  |---|---|
  | snow / ice / arctic / glacier / frozen | `DeepSnow` |
  | sand / dune / sahara / atacama | `DeepSand` |
  | dirt / mud | `Dirt` |
  | kart | `Concrete` |
  | GT, rally, classic, " rx", "grand prix" | `Gravel` |

- **Wall offset inference** — `effective_barrier_offset()` (`mod.rs:619-664`): median
  wall distance over the first 10 samples. Fallbacks: kart 2.0, nascar 1.8,
  extreme_offroad 6.0, others 3.5 m.
- `apply_default_runoff_surfaces()` (`mod.rs:668-701`) writes these defaults into
  every waypoint and sample that has `None`. Every preset calls it at the end.

**Result on a GT circuit:** a gravel strip of the same width (4.5 m at Monza) on
both sides of every straight and every corner, then armco, then flat grass.

### 2.2 Walls — `generate_walls_from_spline` ([presets.rs:367-445](../../crates/arcade-race-core/src/track/presets.rs))

1. For each sample: `wall point = point ± normal × (width/2 + wall_distance.unwrap_or(offset))`.
   On bridges the offset blends to `curb_extra + 0.5` (tight bridge walls).
2. `untangle_polyline` removes loops in the offset line (inside of tight corners).
3. One `WallBarrier` per sample pair per side, if `left_wall`/`right_wall` is true on both.
4. `trim_walls_at_crossings` and `trim_corner_intersections` cut walls that cross the
   road or each other. The second one is O(N²).
5. Collision ([collision/wall.rs:43-263](../../crates/arcade-race-core/src/collision/wall.rs)):
   car OBB against thin line segments, 2 passes per frame, no spatial index.

Global offsets used today:

| Module | Offset | Type |
|---|---|---|
| GT | 3.0–4.5 m | Steel or Concrete |
| Kart | 2.0–2.5 m | TireWall |
| NASCAR ovals | 1.2–1.8 m | Concrete |
| Rallycross | 3.5 m | TireWall |
| Desert off-road | 6–8 m | TireWall |

At 1.2 m (Bristol, Martinsville, Chicago, Eldora) the wall sits on the 1.4 m kerb band.

### 2.3 Scenery — [track/scenery.rs](../../crates/arcade-race-core/src/track/scenery.rs)

Only two scenery types exist.

**`Tree`** (`:93-175`): `position`, `tree_type` (Pine, Palm, Oak, Cypress, Sakura,
AutumnMaple), `scale` (0.2–5.0), `rotation`, `elevation`.
- Trunk = solid circle obstacle (restitution 0.25).
- Canopy = soft drag zone (`velocity *= 1 - drag·dt`) and leaf particles.
- Canopy is drawn **above** cars; alpha 0.6 when a car is under it.

**`Grandstand`** (`:206-337`): `center`, `length` (2–500 m), `depth` (2–100 m),
`angle`, `tiers` (2–40, default 6), `style` (OpenBleachers, CoveredStadium,
HillsideBleachers), `seat_color`, `elevation`.
- Solid box obstacle and `Concrete` surface.
- The front edge is at `center − facing_normal × depth/2`. The doc comment on
  `facing_normal` says the opposite. A stand on the **right** side of the track must
  use `angle = tangent_angle + π` to face the track.
- `surface_zone()` and `front_barrier()` exist but are never called at runtime.

**Generic elements:** `Obstacle::circle` draws as a tyre stack; box/polygon draws as
concrete. `SurfaceZone` polygons can paint any surface anywhere.

**Where scenery exists:** 41 trees on 11 presets, 16 grandstands on 14 presets
(mostly rallycross, NASCAR short tracks, karts). Zero on the GT module
([module/gt.rs](../../crates/tdrace-app/src/module/gt.rs)).

**Placement is by absolute world coordinates only.** There is no rule, no seed, no
density, and no "distance along the lap" anchor. If the geometry changes, scenery
does not follow.

### 2.4 Pit, grid and start/finish

- **Pit:** `pit_box_area` (one AABB, a dark rectangle) plus checkpoints with
  `is_pit_entry` / `is_pit_exit`. No pit road, pit wall, garages or speed limit.
  Only `classic_grand_prix` uses it (`presets.rs:660-711`).
- **Grid:** `generate_grid_positions` (`presets.rs:484-522`): slot at
  `finish − 15 − i·spacing`, alternating ±stagger. GT 10.0 m / 2.5 m ×18,
  kart 5.5/1.8 ×14, NASCAR 7.0/3.0 ×16.
- **Start/finish:** a checkerboard across the finish gate. The gate is
  `width/2 + 4.0` per side (`presets.rs:460`), so the checkerboard extends 4 m into
  the runoff. No gantry, lights or timing tower.

### 2.5 Rendering — [render/track.rs](../../crates/tdrace-app/src/render/track.rs), [render/barrier.rs](../../crates/tdrace-app/src/render/barrier.rs), [render/scenery.rs](../../crates/tdrace-app/src/render/scenery.rs)

**Draw order** (`game/mod.rs:13818-14050`):

1. Backdrop colour + one textured quad over the camera view.
2. BelowTrack zones → pit box → runoff pass → kerb pass → ribbon + markings →
   AboveTrack zones → jump ramps → grid boxes → finish checkerboard.
3. Skidmarks.
4. Grandstand and tree shadows → grandstands → wall shadows and bodies → tree trunks.
5. Cars (ground level) → bridges → elevated walls → elevated cars → particles.
6. Tree canopies.

**Technique:**
- Immediate mode (macroquad). Meshes are **rebuilt every frame** from samples with
  `BatchMeshBuilder` (flush every 2000 vertices). Only textures are cached.
- Culling by camera AABB for ribbon, walls, trees and grandstands. Not culled:
  surface zones, pit box, grid, finish line, ramps.
- Textures: `assets/textures/surfaces/*.png` (512×512), world-space UVs, per-surface
  tile scale (asphalt 4 m, gravel 3.5 m, grass 6 m). On High quality: ±18 % vertex
  colour macro-variation and a 0.6 m soft fringe at the runoff edge.
- Kerbs: one style, red/white, 1.35 m drawn (1.4 m in physics), stripes every 1.5 m
  or `curb_teeth.png` with `u = distance / 3`.
- Asphalt and concrete get a **dashed white centre line** (`track.rs:1511-1514`).
  This looks like a public road, not a race track.
- Lighting: one fixed shadow offset `SHADOW_OFFSET = (0.35, 0.45)` for all objects.
  No time-of-day.

**Camera scale** ([config.rs:97-122](../../crates/tdrace-app/src/config.rs)): 5–22 px per metre
at 720p. The visible world is about 60–250 m across. **Detail more than about 150 m
from the track is not seen** outside the overview zoom. Most environment value is in
the first 0–60 m from the track edge.

**Unused assets and hooks:** `asphalt_groove.png`, `macro_noise_texture`,
`TrackWearState` (racing-line rubber), `compute_segment_curvature`,
`Grandstand::front_barrier()`, `Grandstand::surface_zone()`.

### 2.6 Storage and editor

- Presets are Rust functions. Classic/RX/NASCAR/off-road are in
  [presets.rs](../../crates/arcade-race-core/src/track/presets.rs); GT in
  [module/gt.rs](../../crates/tdrace-app/src/module/gt.rs); karts in
  [module/kart.rs](../../crates/tdrace-app/src/module/kart.rs).
- Load order (`track_manager.rs:827-869`): `tracks/<module>/<slug>.json` (git
  submodule, **not initialised** in this worktree) → user override → Rust generator.
- JSON files are **baked** (waypoints + samples + walls). `from_json` does not rebuild
  the spline. Editing waypoints in JSON has no effect until the editor rebuilds.
- User tracks: `~/Library/Application Support/tdrace/tracks` (macOS), with `.backup/`.
- **Editor** ([editor/tools.rs](../../crates/tdrace-app/src/editor/tools.rs), `editor/ui.rs`):
  - Per waypoint: width, banking, kerbs, walls on/off, wall distance, wall type, surface, elevation.
  - The wall-distance slider sets **both sides to the same value** (`ui.rs:995-1000`).
  - It cannot set runoff surfaces, independent left/right distances, `Virtual` walls,
    trees or grandstands.
  - `EditorState::new` resets the global barrier to 4.0 m Steel. Rebuilding Daytona
    in the editor moves its 1.5 m Concrete walls to 4.0 m Steel.

---

## 3. Physics of surfaces around the track

`Track::sample_surface` ([mod.rs:329-409](../../crates/arcade-race-core/src/track/mod.rs)) resolves in this order:

1. jump ramp
2. `AboveTrack` zone
3. ribbon (or `Curb` in the kerb band)
4. runoff corridor (`|lateral| ≤ half_w + wall_distance`)
5. arena floor
6. `BelowTrack` zone
7. grandstand footprint → `Concrete`
8. `default_surface`

Key values (spec 010/019): Asphalt μ 1.0, Curb 0.88, Gravel 0.70 (RR 2.5×),
Grass 0.45 (RR 18×), DeepSand 0.30 (RR 30×). Tyres pick up dirt off track
(`dirt_contamination`, grip × (1 − 0.25·C)).

**Consequence for design:** `SurfaceZone`s (step 6) are **below** the runoff corridor
(step 4). A gravel-trap polygon inside the corridor is hidden by the corridor surface.
To show a trap you must either set the corridor surface on those waypoints, or widen
the corridor with `*_wall_distance` and set `*_runoff_surface`.

**Two render/physics mismatches:**
- Render draws the corridor only where the wall exists; physics does not check walls.
- Render falls back to 8.0 m; physics falls back to `effective_barrier_offset()`.

---

## 4. The OSM import procedure

### 4.1 Tools

| Script | Circuits | Fetch | Output |
|---|---|---|---|
| [osm_track_importer.py](../../scripts/osm_track_importer.py) | 14 rallycross | OSM Map API bbox, then Overpass ×2 | waypoint vec |
| [osm_f1_importer.py](../../scripts/osm_f1_importer.py) | 18 GT | none (pre-downloaded `.osm`, not on disk) | full `track_<id>()` fn |
| [osm_kart_importer.py](../../scripts/osm_kart_importer.py) | 13 kart | Map API, then Overpass ×2 | waypoint vec |
| [osm_nascar_importer.py](../../scripts/osm_nascar_importer.py) | 4 NASCAR | Map API bbox or `relation/<id>/full` | counts only |

Guidance lives in [.agents/skills/osm-circuit-builder/SKILL.md](../../.agents/skills/osm-circuit-builder/SKILL.md).
Cache: `target/osm_cache/<id>.json` (not committed; only 13 files exist in the main checkout).

### 4.2 Pipeline (same in all four scripts, copy-pasted)

1. **Select** ways by hard-coded way IDs, relation IDs and node slices.
   The only tag filter is `way["highway"="raceway"](bbox)`.
2. **Stitch** into a chain. Rallycross/kart: ordered concatenation. GT: greedy
   nearest-endpoint `stitch_ways` (gap guard broken, §6). No closure check.
3. **Project**: equirectangular tangent plane at the centroid, `R = 6378137`.
4. **Rotate** so the start heads along +X (GT does a second fine rotation).
5. **Scale** uniformly to the official length: rallycross/kart 1:1; **GT 0.5×**;
   NASCAR ovals 1:1, road courses 0.5×.
6. **Translate** start to (0, 0).
7. **Resample** to 14–36 equal-arc points (linear). No Douglas-Peucker, no smoothing.
   Monza spacing 103 m, Le Mans 189 m. **Chicanes do not survive.**
8. **Heuristics:** two widths per track (`default_width`, `straight_width`),
   inside-apex kerbs by cross product, hand-typed per-index elevations,
   one barrier type and offset per circuit.
9. **Paste** the printed Rust into the preset file by hand.

### 4.3 What OSM data is ignored

Only node `lat/lon`, way `nodes`, and (rallycross) `surface`/`name` are read.
Relations are dropped by the Map-API converter. The cached bboxes already contain:

| Feature | Found in cache | Used? |
|---|---|---|
| Pit lane | Catalunya "Pit Lane" ways; GT relations `role=pit_lane` | Excluded |
| Barriers | Catalunya `fence` ×32, `wall` ×5; Montalegre `guard_rail` ×5; Kouvola `wall` ×9, `jersey_barrier` ×2 | No |
| Width | Kouvola `width=8` (game uses 13.0) | No |
| Grandstands | Catalunya `building=grandstand` ×16, `leisure=bleachers` ×16 | No |
| Trees / woods | `natural=tree_row`, `natural=wood`, `landuse=forest` | No |
| Gravel / grass | `natural=shingle` ×15, `natural=sand`, `landuse=grass` | No |
| Bridges / grade | `bridge=yes`, `tunnel=yes`, `layer`, `incline` | No |
| Start/finish | `raceway=start` (Estering) | No |
| Water, service roads, parking | yes | No |

The Overpass-format caches (Höljes, Lydden, Hell, Lohéac) hold only raceway ways and
cannot provide any of this. They must be fetched again with a wider query.

### 4.4 Provenance

[provenance.rs](../../crates/arcade-race-core/src/track/provenance.rs) holds 71 records of
`{id, name, aliases, country, osm_url, wikipedia_url}`. It does **not** store the way
IDs used, the transform (rotation, scale, origin), the raw OSM length, the fetch date,
or ODbL attribution. Five `osm_url`s point to a different object than the importer used
(`estering_rx`, `montalegre_rx`, `hell_rx`, `nyirad_rx`, `silverstone_national_kart`).

Length tests (`rally_tracks_tests.rs:278-355`) are circular: the importer forces the
length to the target, so the tests cannot detect a missing or doubled section.
Nyirád is stretched by **1.33×** and nothing catches it.

---

## 5. Circuit inventory (96)

| Module | Count | Source | Scale | Scenery today |
|---|---|---|---|---|
| Classic | 10 | hand-made | — | some zones; Classic GP has pit box |
| GT | 18 | OSM (`osm_f1_importer.py`) | 0.5× | none |
| Kart | 17 | 13 OSM + 4 hand | ~1:1 | 2 with trees |
| NASCAR | 17 | 4 OSM + 13 hand | ovals ~0.5–1×; road 0.5× | a few grandstands |
| Rallycross | 17 | 14 OSM + 3 hand | ~1:1 | most scenery lives here |
| Extreme off-road | 17 | hand / arenas | — | arenas, obstacles |

GT list: monza, red_bull_ring, nurburgring_gp, silverstone, catalunya, bathurst, spa,
zandvoort, portimao_gp, suzuka, interlagos, le_mans_sarthe, monaco, madring,
marina_bay, bahrain, montreal, cota.

---

## 6. Defects found

| # | Defect | Where | Effect |
|---|---|---|---|
| D1 | GT curvature cross product is not normalised; thresholds 18 and 45 are in m² | `osm_f1_importer.py:578-588` | Kerbs on 23/28 Monza waypoints, 30/32 Spa; straight width almost never used; portal "turns" inflated. |
| D2 | `stitch_ways` measures gaps in degrees; `max_gap=45.0` never triggers | `osm_f1_importer.py:369-395` | Wrong joins pass silently. Spa doubles back at wp12→13 (not verified: `spa.osm` missing). |
| D3 | `scale: "1:1"` on 0.5× tracks | `module/gt.rs:90` and others | Portal and future tools get wrong scale. |
| D4 | Negative elevation clamped to 0 | `spline.rs:353` | Nürburgring dips vanish; no dips anywhere. |
| D5 | Wall distance jumps at t = 0.5 when only one waypoint sets it | `spline.rs:350-391` | Step in the wall when you add one override. |
| D6 | Render/physics runoff mismatch (wall check, 8.0 m fallback) | `render/track.rs:846-853` vs `mod.rs:355-376` | Invisible runoff surface in some cases. |
| D7 | Finish checkerboard spans the gate (track + 8 m) | `presets.rs:460`, `track.rs:1595` | Checkerboard painted on the runoff. |
| D8 | Dashed centre line on race asphalt | `track.rs:1511-1514` | Road look, not circuit look. |
| D9 | Editor resets barrier to 4.0 m Steel; slider sets both sides | `editor/state.rs:534`, `ui.rs:995` | Editing destroys per-track wall tuning. |
| D10 | Spec 006 (branching / `TrackNetwork`) says `implemented`; code only on `origin/feat/road-split-branching-tracks` | `specs/006_…md` | No pit-lane or joker branch possible in `main`. |
| D11 | Five `osm_url`s point to a venue polygon, not the imported ways | `provenance.rs` | Wrong provenance. |
| D12 | No ODbL attribution anywhere | — | Licence obligation not met. |
| D13 | Doc/spec drift: barrier coefficients, spec 012 editor tools, spec 016 textures/mowing stripes | `docs/physics/walls_barriers.md`, specs 012/016 | Docs describe features that do not exist. |

---

## 7. Gaps that block realism

1. **No lap-distance anchor.** Nothing can be placed "at 1,240 m, left side, 5 m behind
   the wall". `spline.sample_at_distance(d)` and `spline.curves` exist; a helper does not.
2. **Runoff is one band.** Real runoff is layered (verge → asphalt → gravel → grass →
   barrier). The model has one surface per side per sample.
3. **One `wall_type` per sample for both sides.** A tyre wall on the outside and armco on
   the inside of the same corner is not possible.
4. **Kerbs are booleans spread over half a segment** (≈ 50 m at Monza), one style,
   always on the inside only (importer), often on straights.
5. **No pit lane road, pit wall, garages, gantry, marshal posts, fences, billboards,
   buildings, service roads, water, car parks.**
6. **Coarse waypoints** (≈ 100 m on GT). Chicanes, true corner radii and width changes
   are lost.
7. **Scenery is typed by hand in world coordinates.** It does not follow geometry edits
   and it does not come from OSM.
8. **Meshes rebuilt every frame.** Adding hundreds of trees and fence posts needs cached
   static meshes first.

See [Circuit Realism Knowledge Base](circuit_realism_knowledge_base.md) for how to close
each gap.
