---
type: Feature Spec
template: feature
title: "Classic Circuits Revamp"
description: "The Classic module replaces its 10 flat, bare circuits with 18 new fictional circuits (3 indoor karting, 3 rallycross, 3 autocross, 3 GT, 3 stock car, 3 all-terrain) and gains 3 Autocross fantasy cars; the circuits are built and play-tested first (Stage 1) and decorated with stands, trees, rocks, water and buildings second (Stage 2)."
status: in_progress
created: 2026-09-29
generated: { by: agent/claude-opus-5-5, at: 2026-09-29T07:47:27Z }
verified: { by: "human:mario", at: "2026-09-29T08:27:38Z" }
---


# Feature Spec: Classic Circuits Revamp 🏁

The Classic module (`"classic"`) has 10 arcade circuits. All of them are flat (elevation 0 m
everywhere), none has a bridge, and none has a tree, a grandstand or an obstacle. This spec
replaces them with **18 new fictional circuits** in 6 groups of 3, and adds **3 Autocross
fantasy cars**:

| Group | Classic car (`car_category`) | Circuits |
|---|---|---|
| Karting | Turbo Dart 200cc (`kart`) | 3 packed indoor circuits with bridges |
| Rallycross | Trailfire Turbo 4WD (`rally`) | 3 mixed-surface circuits with many jumps |
| Autocross | 3 new cars, one per circuit (`autocross`, new) | 3 all-dirt sprint circuits with crests and berms |
| GT | Apex Phantom GT (`gt`) | 3 road circuits with long straights, chicanes and raised ground |
| Stock Cars | Thunderbolt Stock V8 (`nascar`) | short oval, tri-oval, roval |
| All-Terrain | Vortex Dune Crusher (`off_road`) | 1 sand, 1 mud, 1 snow circuit with dunes, hills and ramps |

The Stock Cars group uses the existing code id `nascar`. Only the group name in this spec
changes.

The work has **two stages**, and Stage 2 starts only after Mario play-tests Stage 1:

1. **Stage 1 — Circuits and cars.** Build the 18 circuits (road, surfaces, runoff, traps, walls,
   jumps, bridges, elevation, banking) and the 3 Autocross cars. Retire the old circuits.
2. **Stage 2 — Decoration.** Add grandstands, trees and plants, rocks, water and buildings.
   Rocks, new plants and buildings are new scenery types.

Decisions from Mario:
- 2026-09-28: Delete all 10 old Classic circuits, **except `dirt_figure_eight`**. It stays for
  the three Extreme Off-Road series that race on it, and it leaves the Classic list.
- 2026-09-28: The stock car set is a short high-banked oval, a tri-oval and a roval.
- 2026-09-28: Stage 2 adds rocks, new plants **and** buildings.
- 2026-09-29: The group is named **Stock Cars**, not NASCAR.
- 2026-09-29: Add an **Autocross** group: 3 circuits and 3 fantasy cars. **Each Autocross circuit
  races its own car.**
- 2026-09-29: This spec adds the `autocross` car category. Spec 050 (FIA Autocross module, draft)
  reuses it and does not add it again.

All names are fictional. No circuit, car, maker, stand or building uses a real venue, sponsor or
brand name (spec 047).

---

## 🗺️ User Flow & Interface Design

### 1. What the player sees

- The Classic circuit selector lists the 18 new circuits in this order: Karting, Rallycross,
  Autocross, GT, Stock Cars, All-Terrain. Inside each group, the order is easy → hard.
- Each circuit picks its Classic car as today, from `car_category`
  (`ui/menu.rs::resolve_predefined_car_for_track`). An Autocross circuit picks the car that it
  names in `car_model_id` (see Backend Models). The player and all bots race that car.
- The Classic Garage shows 8 cars: the 5 current ones and the 3 Autocross cars.
- The Classic championship ("TDRace Grand Championship", `module/classic.rs`) runs 6 rounds, the
  first circuit of each group: `kart_pine_grove`, `rx_quarry_sprint`, `ax_meadow_sprint`,
  `gt_velocity_park`, `stock_thunder_bowl`, `at_dune_sea`.
- The Classic default circuit becomes `gt_coastal_grand_prix`.
- The Kart module no longer lists `kart_arena` and `drift_park`. The Rallycross module no longer
  lists `oasis_rally` and `classic_rallycross`. Their own circuits do not change.

### 2. Circuit roster (Stage 1)

Lengths are targets. A built lap may differ by ±15 %.

| Group | id | Name | Tag | Category label | Lap | Laps | Difficulty |
|---|---|---|---|---|---|---|---|
| Karting | `kart_pine_grove` | Pine Grove | `OUTDOOR SPRINT` | Karting | ~390 m | 8 | easy |
| | `kart_riverbend_circuit` | Riverbend Circuit | `TECHNICAL CLUB` | Karting | ~600 m | 7 | medium |
| | `kart_summit_international` | Summit International | `GRAND PRIX` | Karting | ~760 m | 6 | hard |
| Rallycross | `rx_quarry_sprint` | Quarry Sprint | `QUARRY RX` | Rallycross | ~750 m | 6 | easy |
| | `rx_hilltop_leap` | Hilltop Leap | `HILLTOP RX` | Rallycross | ~950 m | 5 | medium |
| | `rx_canyon_flyer` | Canyon Flyer | `CANYON RX` | Rallycross | ~1,150 m | 4 | hard |
| Autocross | `ax_meadow_sprint` | Meadow Sprint | `MEADOW AX` | Autocross | ~800 m | 6 | easy |
| | `ax_clay_bowl` | Clay Bowl | `CLAY BOWL` | Autocross | ~1,000 m | 5 | medium |
| | `ax_hillside_hammer` | Hillside Hammer | `HILLSIDE AX` | Autocross | ~1,250 m | 4 | hard |
| GT | `gt_velocity_park` | Velocity Park | `POWER CIRCUIT` | GT Circuit | ~1,400 m | 4 | easy |
| | `gt_ridge_ring` | Ridge Ring | `HILL CIRCUIT` | GT Circuit | ~1,300 m | 4 | medium |
| | `gt_coastal_grand_prix` | Coastal Grand Prix | `GRAND PRIX` | GT Circuit | ~1,800 m | 3 | hard |
| Stock Cars | `stock_thunder_bowl` | Thunder Bowl | `SHORT TRACK` | Stock Oval | ~500 m | 10 | easy |
| | `stock_tri_oval_speedway` | Tri-Oval Speedway | `TRI-OVAL` | Stock Oval | ~1,100 m | 6 | medium |
| | `stock_roval` | Roval | `ROVAL` | Stock Roval | ~1,300 m | 5 | hard |
| All-Terrain | `at_dune_sea` | Dune Sea | `SAND DUNES` | All-Terrain | ~1,100 m | 4 | sand |
| | `at_mudbath_valley` | Mudbath Valley | `MUD BATH` | All-Terrain | ~950 m | 4 | mud |
| | `at_frostbite_pass` | Frostbite Pass | `SNOW PASS` | All-Terrain | ~1,200 m | 4 | snow |

Every circuit: `kind: circuit`, `module_id: "classic"`, `modules: ["classic"]`,
`category: "main"`, `is_inspired: false`, at least 10 grid slots, 0 validation errors.

No turn right after a bridge: after the last bridge sample (`is_bridge`; the raised run is flagged
while it is 1.2 m or more above the ground), the road runs straight for 20 m or more. Its heading
changes by 5° or less there. A car that comes down from a bridge gets time to settle before it
turns. The builder stops with an error when a circuit breaks this rule (`turns_after_bridges`).

No turn under a bridge: where the lower road passes under a deck (a bridge sample 2.5 m or more above,
closer than the two half widths plus 1 m), it runs straight (heading change 5° or less). A car under a
deck is hidden, so it must not have to turn there (`turns_under_bridges`).

No inner wall step on a turn: when a turn leads, less than 10 m later, into a turn in the same direction,
the inner wall of the second turn is not more than 0.6 m closer to the road. A kerbed turn keeps its
walls 2.0 m out and cars drive on the kerb; a wall that steps in there stopped the bots
(`check_inner_wall_steps`).

### 3. Design rules per group

**Karting — packed indoor circuits** (inspiration: multi-level indoor kart halls)
- Surface Asphalt, `default_surface` Concrete (the hall floor), walls `TireWall`, kerbs on the
  apex of most corners.
- Widths: Hangar Sprint 8 m, Warehouse Twister 7 m, Tower Labyrinth 6.5 m.
- The whole lap fits in a small box: Hangar Sprint ≤ 90 × 60 m, Warehouse Twister ≤ 100 × 70 m,
  Tower Labyrinth ≤ 130 × 80 m. Density (lap length ÷ box area) is at least 0.065 m per m² (Tower
  Labyrinth 0.064). Tower Labyrinth was 120 × 80 m; its 6 pocket legs need 10 m more width with 6.5 m
  hairpins (Mario, 2026-09-30).
- Parallel runs sit close: 3–7 m between road edges (the 6.5 m hairpins set the gap in the pockets),
  with a tyre wall on both sides. The first draft said 1.0–1.5 m; bots got stuck in hairpins that tight.
- Bridges: Hangar Sprint 1, Warehouse Twister 2, Tower Labyrinth 3. Each bridge deck is at least
  4.0 m above the road below it (the validator errors below 3.5 m and warns below 4.0 m). Each
  crossing is at 30° or more, so a kart under a bridge is hidden for less than 15 m.
- Ramps up to a bridge have a grade of 12 % or less.
- Hairpins and U-turns have a design radius of 6.5 m (the track spline bakes them about 25 % tighter).
  Bots got stuck on tighter ones.
- Corners per lap: at least 8, 12 and 16. Warehouse Twister has a chicane. Tower Labyrinth has a
  double hairpin and a corner that tightens.

**Rallycross — jumps and mixed surfaces**
- 30–60 % of the lap is Asphalt, the rest is Gravel or Dirt, set per waypoint. Width 11–14 m.
  Walls `TireWall` and `Steel`.
- Jumps per lap: at least 3, 4 and 6. A jump is a `JumpRamp` (tabletop, 1.5–3.0 m high) or a
  crest in the waypoint elevation that lifts the car.
- Hilltop Leap has a crest on the start straight and a gravel hairpin.
- Canyon Flyer has a gap jump: an above-track `Water` zone after the ramp slows a car that lands
  short. It also has a whoops section (a row of small ramps, like
  `presets.rs::generate_whoops_array`).
- No joker lap: branching roads (spec 006) are not on `main`.

**Autocross — all-dirt sprint circuits** (inspiration: European autocross, see spec 050)
- The road is 100 % unpaved: Dirt (clay and loam), Gravel or PackedSand, set per waypoint. Only
  the start straight may have a Concrete launch pad of 60 m or less.
- Width 12–16 m, so the field can start several cars wide.
- No `JumpRamp`s. The circuits use crests in the waypoint elevation, banked dirt berms (bank
  6–12° into the corner) and, on Hillside Hammer, off-camber corners (bank that tilts away from
  the corner).
- Runoff Grass or Dirt. Walls `TireWall` on the outside of fast corners; open runoff elsewhere.
- Meadow Sprint: flowing corners, 2 crests, 1 berm, 1 hairpin, elevation range 2 m or more.
- Clay Bowl: a lap inside a natural bowl, elevation 0–6 m, 3 berms, a crest into a downhill
  hairpin, a Concrete launch pad.
- Hillside Hammer: climbs of 8 m or more, 2 off-camber corners, crests that lift the car, a tight
  hairpin at the top, Gravel and PackedSand mixed in the Dirt.

**GT — speed, braking and runoff**
- Surface Asphalt, kerbs on apexes and chicanes, walls `Steel` with `TireWall` at the end of fast
  straights.
- Velocity Park: 2 straights of 300 m or more, each ending in a chicane. Wide Asphalt runoff
  (20–30 m) at the braking zones, Gravel elsewhere.
- Ridge Ring: a raised section that climbs to about 8 m and comes back down, **without crossing
  another part of the lap**. Grade 8 % or less. A crest, a downhill corner after it, esses and a
  hairpin. Narrow DeepSand traps (5–8 m) and Grass.
- Coastal Grand Prix: a 400 m straight, a bus-stop chicane, a carousel (one long turn of 150° or
  more), a fast sweeper and a raised plateau of about 5 m.
- Runoff width changes along each lap (3–30 m per side, set with `left/right_wall_distance`).
  Each circuit uses at least 3 of these runoff and trap surfaces: Gravel, DeepSand, PackedSand,
  Grass, Asphalt. Together the 3 circuits use all 5. Traps are `SurfaceZone`s with
  `layer: below_track` or runoff surfaces per waypoint.

**Stock Cars — banked ovals**
- Races run anticlockwise (left turns), so the outer side is the right side and `bank_angle` is
  positive in the turns.
- Thunder Bowl: two 180° turns banked 24–26°, straights banked 4–6°, Concrete surface, outer wall
  `Concrete` close to the road, a Concrete apron on the inside.
- Tri-Oval Speedway: turns banked 20°, a dogleg in the front stretch banked 8°, back straight
  banked 5°, Asphalt.
- Roval: an oval banked 18° plus an infield road part (flat, with kerbs, a chicane and a hairpin)
  that leaves the oval and joins it again. The lap does not cross itself.
- Infield `default_surface` Grass. Oval walls `Concrete` or `Steel`, infield walls `TireWall`.

**All-Terrain — sand, mud, snow**
- Walls `TireWall` where the lap needs them; open runoff elsewhere.
- Dune Sea: PackedSand road, DeepSand runoff. Dune waves between 0 and 6 m, with a crest every
  40–70 m; some crests lift the car. 2 ramps. An oasis `Water` zone next to the line.
- Mudbath Valley: MudTrack road with DeepMud patches (above-track zones), Water puddles, a whoops
  section, a hill of about 8 m and 2 ramps.
- Frostbite Pass: PackedSnow road, DeepSnow runoff, a frozen-lake part with SheetIce road, a pass
  that climbs about 10 m and comes down, and at least 1 ramp.
- Elevation range on each lap is at least 5 m. Elevation is 0 m or more (`spline.rs` clamps it),
  so dunes and hills are bumps above 0 m.

In Stage 1, a `SurfaceZone` is added only when it touches the road or its runoff (traps, water
hazards, mud patches). Water and other zones that are only for looks come in Stage 2.

### 4. Autocross fantasy cars (Stage 1)

Three new Classic cars, one for each Autocross circuit. Makers are fictional. Each car gets a
lateral image, a thumbnail and a top-down image, made by
`scripts/generate_classic_fantasy_sprites.py`, like the other Classic cars. Engine sounds reuse
existing `EngineSoundType`s.

| id | Name | Maker | Type | Drive | Power | Weight | Top speed | Engine sound | Circuit |
|---|---|---|---|---|---|---|---|---|---|
| `classic_ax_mudlark` | Mudlark Cross Car | Mirebrook Racing | single-seat cross car, motorcycle engine | RWD | 150 bhp | 420 kg | ~160 km/h | `CrossCarMotorcycle` | Meadow Sprint |
| `classic_ax_brawler` | Brawler Touring AX | Stonecairn Works | touring silhouette car | AWD | 420 bhp | 1,150 kg | ~185 km/h | `Rally2Turbo` | Clay Bowl |
| `classic_ax_talon` | Talon Super Buggy | Harrowfield Offroad | open-wheel super buggy | AWD | 560 bhp | 800 kg | ~200 km/h | `SandRailBoxer` | Hillside Hammer |

- Handling: the Mudlark is light and turns fast but has little power; the Brawler is heavy and
  stable and slides wide; the Talon is the fastest and needs the most throttle control. All three
  use arcade assists, like the other Classic cars.
- Each car has its own `CarConfig` function in `module/classic.rs`, based on the closest existing
  base (`CarConfig::sand_rail()` for the cross car and the buggy, the rally car base for the
  touring car).
- `category_name`: "Arcade Cross Car", "Arcade Touring AX", "Arcade Super Buggy". Tier 1.

### 5. Raised-ground cue (Stage 1)

Today only bridges and road slopes are shaded (`render/track.rs:1424`). A flat raised part
looks the same as flat ground. `render/track.rs` draws an embankment shade along both road
edges where a sample is higher than about 0.8 m and is not a bridge. The shade gets wider with
the height, with an upper limit. GT raised parts, Autocross hills and All-Terrain dunes use this.

### 6. Retiring the old circuits (end of Stage 1)

This happens after all 18 new circuits are in the catalog, so the game always has circuits to
fall back to.

| Old id | What happens | Alias to |
|---|---|---|
| `classic_grand_prix` | deleted | `gt_coastal_grand_prix` |
| `oval_speedway` | deleted | `stock_tri_oval_speedway` |
| `dirty_oval_speedway` | deleted | `stock_thunder_bowl` |
| `figure_eight` | deleted | `gt_velocity_park` |
| `drift_park` | deleted (also leaves Kart) | `gt_ridge_ring` |
| `kart_arena` | deleted (also leaves Kart) | `kart_hangar_sprint` |
| `ramp_raceway` | deleted | `rx_hilltop_leap` |
| `oasis_rally` | deleted (also leaves Rallycross) | `at_dune_sea` |
| `classic_rallycross` | deleted (also leaves Rallycross) | `rx_quarry_sprint` |
| `dirt_figure_eight` | kept; file moves to `tracks/extreme_offroad/`, `modules: ["extreme_offroad"]` | – |

- Aliases go in `tracks/.aliases.json`, so old saves, replays and records still load.
  `catalog::find` and `official_track` already resolve aliases (`crates/tdrace-core/src/catalog.rs`).
- `TrackChoice` (`ui/menu.rs`) keeps its 7 named variants, because saves store them. Their old
  ids resolve through the aliases.
- Every code path that names a deleted id is changed: defaults (`config.toml`, `config.rs`,
  `module/classic.rs`, `tracks/official.rs` fallback, `config.kart.toml` and `config.rally.toml`
  to the first circuit of their own module), the classic championship, `ai/bot_harness.rs`,
  `crates/tdrace-py/src/engine.rs`, `python/tdrace/env.py`.
- Tests that used an old circuit as a plain fixture move to a new circuit or to a circuit of the
  same kind in another module. Tests that checked old-circuit details (jump counts, water circle
  positions, recorded AI control hashes) are rewritten for the new circuits.
- Docs and portal data follow: `docs/circuits/classic.md`, `scripts/generate_asset_data.py`
  (`CLASSIC_TRACK_CATEGORIES`), `portals/shared/data/circuits.json`,
  `assets/textures/circuits/classic/*.svg`.
- Specs 050–052 add circuits to other modules. The count assertions use the catalog on the day
  this lands. On 2026-09-29 the counts become `[18, 21, 18, 17, 17, 17]` (classic,
  extreme_offroad, gt, kart, nascar, rally), 108 in total.

### 7. Play-test gate

When Stage 1 is on `main`, Mario plays the 18 circuits and the 3 Autocross cars and gives
feedback. The Stage 1 fixes happen first. Mario then closes the gate task, and only then does
Stage 2 start.

### 8. Decoration (Stage 2)

New scenery types (see Backend Models): rocks, 3 new plants, buildings. Existing types are
reused: grandstands (they already draw a crowd), the 6 tree types, tyre stacks (`Obstacle`
circles) and `Water` zones.

| Group | Theme |
|---|---|
| Karting | Indoor hall: hall walls around the lap, pillars in the gaps between runs, tyre stacks at the hairpins, small balcony stands. No trees. |
| Rallycross | Hillside stands, paddock tents, rocks, pines and oaks, a pond at the Canyon Flyer gap jump. |
| Autocross | Countryside: spectator banks with hillside bleachers, paddock tents and a small control tower, oaks, pines and bushes, rocks, tyre stacks on the berms. |
| GT | Covered stands on the main straight, pit garages and a control tower, cypress and oak trees, a lake. |
| Stock Cars | Big stands along the front stretch and turns, infield garages and a tower, an infield lake. |
| All-Terrain | Sand: cactus, palms, sandstone rocks, an oasis, tents. Mud: oaks, bushes, puddles, rocks, bleachers. Snow: snow pines, snow-capped rocks, tents. |

Minimum per circuit: 2 grandstands and 3 buildings. Outdoor circuits have at least 30 trees and
plants. Rallycross, Autocross and All-Terrain circuits have at least 10 rocks. No scenery sits on
the road, on a kerb or between the road and its walls.

---

## ⚙️ Backend Models & API Endpoints

### 1. Stage 1: two small schema changes

**Autocross car category.** `crates/arcade-race-core/src/car_category.rs` gains a variant. Spec
050 reuses it later.

```rust
pub enum CarCategory {
    Gt,
    Nascar,
    Rally,
    Kart,
    #[serde(rename = "off_road", alias = "offroad", alias = "off-road", alias = "extreme_offroad")]
    OffRoad,
    /// Autocross cross cars, buggies and touring autocross cars.
    Autocross, // serde: "autocross"
}
```

- `title()` is `AUTOCROSS`. `from_id()` maps `classic_ax_*` ids to `Autocross`.
- `CarChoice` (`ui/menu.rs`) gains a `CrossCar` base whose `category()` is `Autocross`. Every
  `match` on `CarCategory` or `CarChoice` gets an `Autocross` / `CrossCar` arm.

**Circuit car.** `Track` (`crates/arcade-race-core/src/track/mod.rs`) gains an optional field:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub car_model_id: Option<String>, // e.g. "classic_ax_talon"
```

- In Classic, the car of a circuit is `car_model_id` when it is set, else the car of its
  `car_category` (as today). The player car and the bot pool both use it
  (`game/mod.rs::eligible_opponent_cars`, `catalog::get_classic_model_for_category` and their
  callers).
- `skip_serializing_if` keeps every other circuit file unchanged.
- The 3 Autocross circuits set it. No other circuit does.

All 18 circuits otherwise use the current track JSON schema (`crates/arcade-race-core/src/track/`).

**Circuit builder** — `scripts/classic_circuit_builder.py` (Python standard library only, no new
dependency). It holds the 18 circuits as data, so Mario's feedback is a small edit and a rebuild.
- A circuit is a list of segments: `straight(length)` and `arc(radius, degrees)`. Each segment
  sets width, road surface, elevation (start → end, eased), bank angle, kerbs, and per side the
  runoff surface, wall on/off, wall distance and barrier type.
- The builder checks that the lap closes (end point within 0.5 m and heading within 1° of the
  start) and stops with an error if it does not.
- It works in two passes:
  1. Write the waypoints to `tracks/classic/<id>.json`, then run
     `cargo run --bin track_bake -- tracks/classic/<id>.json --rebuild`.
  2. Read the baked samples and place ramps, zones and (Stage 2) scenery by **lap distance +
     side + lateral offset**, then run `track_bake` again to validate.
  Placing by lap distance means that when a corner moves after feedback, the ramps, traps and
  scenery move with the road.
- `--only <id>` builds one circuit. `--check` builds into a temporary folder and fails when the
  result differs from the committed JSON.
- It reuses helpers from `scripts/osm_importer.py` where they fit (for example the waypoint
  writer behind `write_source_json`).

### 2. Stage 2: new scenery types

In `crates/arcade-race-core/src/track/scenery.rs`. New `TrackGeometry` fields use
`#[serde(default)]`, so older JSON and custom circuits still load.

```rust
pub enum RockType { Granite, Sandstone, Slate, SnowCapped }

pub struct Rock {
    pub id: usize,
    pub position: Vec2,
    pub rock_type: RockType,
    pub scale: f32,
    pub rotation: f32,
    pub elevation: f32,
}

// TreeType gains: Bush (canopy only, no solid trunk), Cactus (small solid trunk), SnowPine.

pub enum BuildingStyle { PitGarage, ControlTower, PaddockTent, HallWall, Pillar }

pub struct Building {
    pub id: usize,
    pub center: Vec2,
    pub size: Vec2,        // width, depth in metres
    pub angle: f32,
    pub style: BuildingStyle,
    pub roof_color: [f32; 3],
    pub elevation: f32,
}

// TrackGeometry gains:
//   #[serde(default)] pub rocks: Vec<Rock>,
//   #[serde(default)] pub buildings: Vec<Building>,
```

- Rocks and buildings are solid. They join the collision list in
  `TrackGeometry::all_obstacles_with_scenery()` (`geometry.rs`).
- Drawing: `crates/tdrace-app/src/render/scenery.rs`, with a shadow and view culling like the
  trees. The circuit viewer (`crates/tdrace-app/src/ui/circuit_viewer.rs`) draws them too.
  Buildings show fictional signs only.
- Validation (`track/validation.rs`) gets an error when a tree trunk, rock or building overlaps
  the road, a kerb, or the strip between the road and its walls.
- The builder places all scenery by lap distance, side and offset.

### 3. API endpoints

None. The game has no server API for circuits.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

No new access paths. The circuits are embedded in the binary as today (spec 042). The builder
runs only on a developer machine and reads and writes files inside the repository.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- `cargo test --workspace --exclude tdrace-py`
- `uv run pytest tests/python` and `uv run ruff check scripts python tests`
- `python3 scripts/classic_circuit_builder.py --check` (the committed circuits are reproducible)
- New `crates/tdrace-core/tests/classic_circuits_tests.rs`: the group rules of section 3 (bridge
  count, footprint and density, jump count, unpaved share, straight length, chicanes, banking,
  elevation range, runoff surfaces, no self-crossing on GT circuits).
- New `crates/tdrace-app/tests/classic_circuits_bot_tests.rs`: 4 bots finish 2 laps on each new
  circuit with `ai::bot_harness::run_harness_race`, in the circuit's own car.
- Unit tests for `CarCategory::Autocross` (serde name, `from_id`) and for `car_model_id` (absent
  field loads as `None`; Classic picks the named car).
- Stage 2: unit tests for the new scenery types (serde defaults, collision, validation) and a
  render test that draws each new type.

### Manual Acceptance Criteria (Pseudo-Gherkin)

#### Stage 1 — Circuits and cars

- **Scenario: Classic lists the 18 new circuits**
  - [ ] **Given** the Classic module
  - [ ] **When** the player opens the circuit selector
  - [ ] **Then** it lists exactly the 18 circuits of section 2, grouped Karting, Rallycross, Autocross, GT, Stock Cars, All-Terrain, easy → hard
  - [ ] **And** each circuit starts a race with its Classic car

- **Scenario: Every new circuit is valid and raceable**
  - [ ] **Given** each of the 18 new circuits
  - [ ] **When** `validate_track` runs and 4 bots race 2 laps in the bot harness
  - [ ] **Then** there are 0 validation errors and every bot finishes both laps

- **Scenario: Karting circuits are packed indoor circuits with bridges**
  - [x] **Given** the 3 karting circuits
  - [x] **When** their baked samples are measured
  - [x] **Then** they have 1, 2 and 3 bridges, each at least 4.0 m clear
  - [x] **And** each fits its box of section 3 with the density of section 3

- **Scenario: No turn under a bridge**
  - [ ] **Given** each new circuit with a bridge
  - [ ] **When** a car drives on the lower road under a deck
  - [ ] **Then** the lower road runs straight there (heading change 5° or less)

- **Scenario: No turn right after a bridge**
  - [ ] **Given** each new circuit with a bridge
  - [ ] **When** a car comes off the end of a bridge
  - [ ] **Then** the road runs straight for 20 m or more (heading change 5° or less)

- **Scenario: Rallycross circuits have many jumps**
  - [ ] **Given** the 3 rallycross circuits
  - [ ] **When** the player drives a lap
  - [ ] **Then** the car leaves the ground at least 3, 4 and 6 times per lap
  - [ ] **And** each lap has both Asphalt and Gravel or Dirt road

- **Scenario: Autocross circuits are all-dirt sprint circuits**
  - [ ] **Given** the 3 autocross circuits
  - [ ] **When** their waypoints are measured
  - [ ] **Then** the road is unpaved everywhere except a launch pad of 60 m or less on the start straight
  - [ ] **And** they have no `JumpRamp`, each has at least 1 banked berm, and Hillside Hammer has 2 off-camber corners

- **Scenario: Each Autocross circuit races its own car**
  - [ ] **Given** Meadow Sprint, Clay Bowl and Hillside Hammer
  - [ ] **When** the player starts a race on each one
  - [ ] **Then** the player and all bots drive the Mudlark Cross Car, the Brawler Touring AX and the Talon Super Buggy
  - [ ] **And** the Classic Garage shows the 3 cars with their images and specs

- **Scenario: GT circuits reward speed and braking**
  - [ ] **Given** the 3 GT circuits
  - [ ] **When** their layouts are measured
  - [ ] **Then** each has a straight of at least 300 m and at least 1 chicane
  - [ ] **And** Ridge Ring and Coastal Grand Prix have raised ground of 5 m or more with no self-crossing
  - [ ] **And** each uses at least 3 runoff or trap surfaces, and runoff width changes along the lap

- **Scenario: Stock Car circuits are banked**
  - [ ] **Given** Thunder Bowl, Tri-Oval Speedway and Roval
  - [ ] **When** the player drives a lap
  - [ ] **Then** the main turns are banked at least 18° (24° on Thunder Bowl)
  - [ ] **And** Roval leaves the oval for an infield part with a chicane and joins it again

- **Scenario: All-terrain circuits have themed surfaces and hills**
  - [ ] **Given** Dune Sea, Mudbath Valley and Frostbite Pass
  - [ ] **When** the player drives a lap
  - [ ] **Then** the road is sand, mud and snow, and each lap climbs and drops at least 5 m
  - [ ] **And** Dune Sea crests and the ramps lift the car

- **Scenario: Raised ground is visible**
  - [ ] **Given** Ridge Ring and Dune Sea
  - [ ] **When** the camera shows a raised part that is not a bridge
  - [ ] **Then** the road edges show the embankment shade, and flat parts do not

- **Scenario: Old circuits are retired safely**
  - [ ] **Given** a save, replay or record that names one of the 9 deleted ids
  - [ ] **When** the game loads it
  - [ ] **Then** it loads the alias circuit of section 6 and does not crash
  - [ ] **And** no code, test, config or doc still names a deleted id outside `.aliases.json`, `TrackChoice` and old specs

- **Scenario: dirt_figure_eight stays in Extreme Off-Road**
  - [ ] **Given** the Extreme Off-Road module
  - [ ] **When** the player starts one of its 3 series that race on `dirt_figure_eight`
  - [ ] **Then** the round loads as before, and the Classic list does not show it

- **Scenario: The builder reproduces the circuits**
  - [ ] **Given** the committed files in `tracks/classic/`
  - [ ] **When** `python3 scripts/classic_circuit_builder.py --check` runs
  - [ ] **Then** it exits 0

- **Scenario: Mario signs off Stage 1**
  - [ ] **Given** Stage 1 is on `main`
  - [ ] **When** Mario has played the 18 circuits and the 3 Autocross cars, and his fixes are in
  - [ ] **Then** he closes the play-test gate task, and Stage 2 tasks become ready

#### Stage 2 — Decoration

- **Scenario: Rocks and new plants work**
  - [ ] **Given** a circuit with rocks, bushes, cacti and snow pines
  - [ ] **When** a car drives into them
  - [ ] **Then** rocks and cacti stop the car like a tree trunk, and bushes only slow it
  - [ ] **And** they draw with a shadow in the race and in the circuit viewer

- **Scenario: Buildings work**
  - [ ] **Given** a circuit with each building style
  - [ ] **When** the race and the circuit viewer show them
  - [ ] **Then** they draw with a roof and a shadow, and a car cannot drive through them
  - [ ] **And** no sign shows a real brand

- **Scenario: Scenery never blocks the road**
  - [ ] **Given** each decorated circuit
  - [ ] **When** `validate_track` runs
  - [ ] **Then** no tree trunk, rock or building overlaps the road, a kerb or the strip inside the walls

- **Scenario: Each group has its theme**
  - [ ] **Given** the 18 decorated circuits
  - [ ] **When** Mario looks at each one
  - [ ] **Then** it matches the theme of section 8 and meets its minimum counts
  - [ ] **And** the karting circuits look like indoor halls, with no trees

- **Scenario: Decoration does not slow the game**
  - [ ] **Given** the circuit with the most scenery
  - [ ] **When** the build embeds the catalog and a race runs
  - [ ] **Then** the build shows no bundle-size warning (8 MB) and new scenery uses view culling

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files

Stage 1:
- `[ ]` `scripts/classic_circuit_builder.py` -> Builds the 18 circuits from data (new).
- `[x]` `tests/python/test_classic_circuit_builder.py` -> Builder unit tests and a two-pass build test (new).
- `[ ]` `tracks/classic/*.json` -> 18 new circuits; 9 old files deleted.
- `[ ]` `tracks/extreme_offroad/dirt_figure_eight.json` -> Moved from `tracks/classic/`, Extreme Off-Road only.
- `[ ]` `tracks/.track_order.json`, `tracks/.aliases.json` -> New Classic order, old ids removed from Kart and Rallycross, aliases.
- `[x]` `crates/arcade-race-core/src/track/presets.rs` -> Wall trimming keeps untrimmed walls shorter than
  0.10 m. Before, it dropped the whole inner wall of tight turns on circuits with dense samples (the kart
  hairpin tips were open); regression test in `crates/arcade-race-core/src/track/mod.rs`.
- `[x]` `crates/race-kit/src/ai/mod.rs` -> Bots on tight, walled circuits: the steering target is at most 75° around a
  bend, and the line to it, and the car itself, keep 1.2 m from walls set close by the waypoints.
- `[x]` `crates/tdrace-app/src/game/mod.rs` -> A car under a bridge is drawn under the deck (same projection as the
  physics, no flicker); the player's ground aura is drawn above the deck while the car is under it.
- `[ ]` `crates/arcade-race-core/src/car_category.rs` -> `CarCategory::Autocross`.
- `[ ]` `crates/arcade-race-core/src/track/mod.rs` -> `Track.car_model_id`.
- `[ ]` `crates/tdrace-app/src/ui/menu.rs`, `crates/tdrace-app/src/catalog/mod.rs`, `crates/tdrace-app/src/game/mod.rs` -> `CarChoice::CrossCar`, the 3 cars in `CLASSIC_ARCADE_CARS`, car choice by `car_model_id`.
- `[ ]` `crates/tdrace-app/src/module/classic.rs` -> 3 Autocross `CarConfig`s and vehicle entries, defaults, 6-round championship.
- `[ ]` `scripts/generate_classic_fantasy_sprites.py`, `assets/textures/vehicles/{laterals,topdown}/classic/classic_ax_*.png` -> Autocross car images.
- `[ ]` `crates/tdrace-app/src/render/track.rs` -> Raised-ground embankment shade.
- `[ ]` `config.toml`, `crates/tdrace-app/src/config.rs`, `config.kart.toml`, `config.rally.toml`, `crates/tdrace-app/src/tracks/official.rs` -> Defaults.
- `[ ]` `crates/tdrace-app/src/ai/bot_harness.rs`, `crates/tdrace-py/src/engine.rs`, `python/tdrace/env.py` -> Circuit ids.
- `[ ]` Tests that name old ids (`crates/*/tests/*.rs`, `crates/arcade-race-core/src/track/{mod,validation}.rs`, `tests/python/test_official_tracks.py`) -> New fixtures and counts.
- `[ ]` `crates/tdrace-core/tests/classic_circuits_tests.rs`, `crates/tdrace-app/tests/classic_circuits_bot_tests.rs` -> New tests.
- `[ ]` `docs/circuits/classic.md`, `scripts/generate_asset_data.py`, `portals/shared/data/{circuits,vehicles}.json`, `assets/textures/circuits/classic/*.svg` -> Docs and portal.

Stage 2:
- `[ ]` `crates/arcade-race-core/src/track/scenery.rs`, `crates/arcade-race-core/src/track/geometry.rs` -> `Rock`, `RockType`, `Building`, `BuildingStyle`, new `TreeType`s, collision.
- `[ ]` `crates/arcade-race-core/src/track/validation.rs` -> Scenery-on-road error.
- `[ ]` `crates/tdrace-app/src/render/scenery.rs`, `crates/tdrace-app/src/ui/circuit_viewer.rs`, `crates/tdrace-app/src/game/mod.rs` -> Drawing and draw order.
- `[ ]` `scripts/classic_circuit_builder.py`, `tracks/classic/*.json` -> Decoration of the 18 circuits.

### Verification Assertions
- `crates/tdrace-core/tests/official_catalog_tests.rs`: 18 Classic circuits and the new totals.
- `crates/tdrace-core/tests/classic_circuits_tests.rs`: group rules of section 3.
- `crates/tdrace-app/tests/classic_circuits_bot_tests.rs`: bots finish 2 laps on all 18.
- `crates/tdrace-app/tests/garage_tests.rs`, `render_tests.rs`: 3 more cars than before; the Classic Garage has 8.
- `crates/tdrace-app/tests/series_tests.rs`: the 3 Extreme Off-Road series still find `dirt_figure_eight`.

### Beads Epic Mapping
Tracked via Beads Epic `tdrace-classic-circuits-revamp-dh6k` ("Fulfill Spec 055: Classic Circuits
Revamp"). Only this epic names the spec number; the child epics name their parent id instead, so
`keel validate` maps the spec to exactly one epic. Ids below drop the
`tdrace-classic-circuits-revamp-` prefix.

| Stage | Item | Id | Waits for |
|---|---|---|---|
| 1 | Task: circuit builder script | `dh6k.1` | – |
| 1 | Task: raised-ground embankment shade | `dh6k.2` | – |
| 1 | Task: Autocross car category, circuit car field and 3 Autocross cars | `dh6k.18` | – |
| 1 | Epic: Karting circuits (`.3.1`–`.3.3`) | `dh6k.3` | `.1` |
| 1 | Epic: Rallycross circuits (`.4.1`–`.4.3`) | `dh6k.4` | `.1` |
| 1 | Epic: Autocross circuits (`.19.1`–`.19.3`) | `dh6k.19` | `.1`, `.2`, `.18` |
| 1 | Epic: GT circuits (`.5.1`–`.5.3`) | `dh6k.5` | `.1`, `.2` |
| 1 | Epic: Stock Car circuits (`.6.1`–`.6.3`) | `dh6k.6` | `.1` |
| 1 | Epic: All-Terrain circuits (`.7.1`–`.7.3`) | `dh6k.7` | `.1`, `.2` |
| 1 | Task: retire the old Classic circuits | `dh6k.8` | `.3`–`.7`, `.19` |
| 1 | Task: Mario play-test sign-off (gate) | `dh6k.9` | `.8` |
| 2 | Task: rocks and new plant types | `dh6k.10` | `.9` |
| 2 | Task: building scenery type | `dh6k.11` | `.9` |
| 2 | Epic: Karting decoration (`.12.1`–`.12.3`) | `dh6k.12` | `.9`, `.10`, `.11` |
| 2 | Epic: Rallycross decoration (`.13.1`–`.13.3`) | `dh6k.13` | `.9`, `.10`, `.11` |
| 2 | Epic: Autocross decoration (`.20.1`–`.20.3`) | `dh6k.20` | `.9`, `.10`, `.11` |
| 2 | Epic: GT decoration (`.14.1`–`.14.3`) | `dh6k.14` | `.9`, `.10`, `.11` |
| 2 | Epic: Stock Car decoration (`.15.1`–`.15.3`) | `dh6k.15` | `.9`, `.10`, `.11` |
| 2 | Epic: All-Terrain decoration (`.16.1`–`.16.3`) | `dh6k.16` | `.9`, `.10`, `.11` |
| – | Task: receipt and completion check | `dh6k.17` | `.12`–`.16`, `.20` |
