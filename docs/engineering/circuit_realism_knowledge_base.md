---
type: Architecture Spec
title: "Circuit Realism Knowledge Base"
description: "Reference for making TdRace circuits more realistic and pleasant: OSM tagging and queries, runoff derivation from OSM, FIA design rules translated to game scale, placement rules for grandstands, pits and scenery, visual design, and an improvement plan."
status: active
category: engineering
tags: [circuits, osm, overpass, runoff, scenery, grandstands, pit-lane, fia, visual-design]
---

# Circuit Realism Knowledge Base

This document collects the knowledge needed to improve existing circuits. The goal
is more realism and a more pleasant environment. It answers four questions:

1. What does a real circuit look like around the track? (§2)
2. What does OpenStreetMap (OSM) know about it, and how do we get it? (§3, §4)
3. How do we turn OSM data into runoff, barriers, grandstands, pits and scenery? (§5, §6)
4. What must change in the engine, and in which order? (§8, §9)

Read [Circuit Building & Environment Analysis](circuit_building_analysis.md) first. It
describes the current code. This document refers to its defects as D1–D13.

OSM counts come from taginfo and live Overpass queries on 2026-09-27.
FIA numbers come from the public regulations listed in §11.

---

## 1. Key facts in one place

- **OSM has no standard tag for runoff, gravel traps or kerbs.** Gravel traps are
  mapped as `natural=sand` / `natural=shingle` polygons. Barriers are `barrier=wall`,
  `barrier=tyres`, `barrier=guard_rail` ways. These are the real signals.
- **Pit lanes are usually plain `highway=raceway` ways with a name** ("Pit Lane",
  "Boxenstraße"). Find them by name and by topology, not by a tag.
- **Measured on the Red Bull Ring** (892 samples at 10 m): about 80 % of samples per
  side found a barrier within 150 m. Median distance 13–21 m, p90 76–93 m.
- **FIA runoff depth is about 30–100 m** on the outside of corners, 1–5 m verge
  everywhere. GT circuits here are at **0.5× scale**, so this becomes **15–50 m**.
- **The camera shows 60–250 m.** Environment value is highest in the first 0–60 m
  from the track edge. Far background only matters in the overview zoom.
- **Our data model already supports per-side, per-waypoint runoff width and surface.**
  The fastest visible win is to fill those fields from OSM or from corner rules.

---

## 2. How real circuits are built (FIA/FIM rules)

### 2.1 Cross-section of a permanent road circuit

```
paddock | garages | inner lane | fast lane | pit wall | verge | TRACK | verge | runoff (asphalt → gravel) | barrier | fence | service road | grandstand
```

| Element | Real value | Source |
|---|---|---|
| Track width | ≥ 12 m (new circuits); change ≤ 1 m per 20 m | FIA App. O 2024 |
| Grid width | ≥ 15 m to first corner exit; gradient ≤ 2 % | FIA App. O |
| Grid length | ≥ 6 m per car (8 m F1); ≥ 250 m to first corner | FIA App. O |
| Edge line | white, ≥ 10 cm | FIA App. O |
| Verge | 1–5 m, flush with track | FIA App. O |
| Runoff | "principally on the exterior of corners", about 30–100 m deep | FIA App. O |
| Runoff slope | ≤ 25 % up (not gravel), ≤ 3 % down | FIA App. O |
| Pit lane | ≥ 12 m, next to start straight, behind pit wall | FIA App. O |
| Pit lane lanes | fast lane ≤ 3.5 m by the wall; inner (work) lane by the garages | F1 Sporting Regs 2025 |
| Pit speed | 80 km/h (60 km/h at tight venues) | F1 Sporting Regs 2025 |
| Grid | staggered 1×1, rows 16 m apart → 8 m between cars | F1 Sporting Regs 2025 |
| Overhead gantry | ≥ 4.5 m clearance; light panel bottom ≥ 2.5 m | FIA App. O |
| Structures | ≥ 1 m behind barriers | FIA App. O |
| Ad boards | ≥ 3 m from track edge, never on outside of corners | FIA App. O |
| Marshal posts | ≤ 500 m apart (FIA), ≤ 300 m (FIM), each sees its neighbours | FIA App. H |
| Gravel bed | 25 cm deep, 8–20 mm stones, surface 1–2 cm below track | FIM Europe RR07 |
| Pit building | 10–12 teams × 2–3 bays × 6–8 m → 250–350 m long | Sepang: bays 8×24 m |

### 2.2 Which runoff where

| Place | Real practice |
|---|---|
| Straights | Narrow grass verge, then armco or concrete. Low impact angle. |
| Inside of corners | Grass verge, sausage kerb at chicanes, armco. |
| Outside of slow corners / hairpins after a long straight | Asphalt runoff (lets cars brake and rejoin), gravel beyond it. Tyre wall at the end. |
| Outside of fast corners | Deep gravel (stops the car), often an asphalt strip first. Tecpro or tyres in front of armco. |
| Straight-on escape at heavy braking | Asphalt or gravel escape road in line with the approach. |
| Street circuits | 1–3 m, concrete walls with debris fence. Escape roads at the ends of straights. |

Gravel is returning to F1 at several circuits because asphalt runoff causes
track-limits abuse. A modern mix is: kerb → 2–5 m asphalt strip → gravel → barrier.

### 2.3 Barriers — what and where

| Barrier | Where | Game type today |
|---|---|---|
| Concrete wall | Street circuits, pit wall, ovals, straights | `Concrete` |
| Armco (guard rail) | Straights, low-angle zones, often with fence behind | `Steel` |
| Tyre wall | End of runoff, high-angle impact zones, karting | `TireWall` |
| Tecpro (foam blocks) | High-speed impact zones with runoff in front, in front of armco | none (use `TireWall`) |
| SAFER barrier | Ovals, on the concrete outside wall, full lap at Daytona | none (use `Concrete`) |
| Debris / catch fence | Behind the barrier, in front of spectators; 22 ft at Daytona | none |
| Plastic blocks | Karting, over grass | none (use `TireWall`) |

### 2.4 Kerbs

- **Flat painted kerbs**, flush: inside apex, outside exit.
- **Sausage kerbs** (5–10 cm bump): behind apex kerbs at chicanes to stop cutting.
- **Placement rule for a game:** inside kerb from turn-in to just past the apex; outside
  kerb from the apex to about two car lengths past the exit. Blocks about 1–1.5 m.
- Colours vary by circuit: red/white (most), red/green (outer strips), blue/white,
  yellow/black (sausage). This is a cheap identity marker per circuit.

### 2.5 Discipline templates

**NASCAR oval** (Daytona example). From outside in: grandstand → catch fence →
SAFER/concrete wall **at the track edge** → banked racing surface (≈ 12 m + wide) →
flat apron (different colour, 4–9 m) → grass or inside wall → infield (lake, parking,
garages). Pit road runs along the **inside** of the front stretch (≈ 490 m at Daytona).

**Karting** (FIA Karting App. 13). Width 8/7/6 m for grade 1/2/3 (start straight ≥ 8 m).
Length 800–1,700 m. Verge 1.8 m. ≥ 6 m between adjacent sections. Tyre and plastic
barriers over grass. Pit deceleration lane 3–4 m.

**Rallycross** (App. O supplement). Length 800–1,400 m, width 10–25 m (World RX ≥ 12 m).
30–60 % sealed surface, the rest earth or gravel. Start line ≥ 14.5 m wide, ≥ 100 m to
the first bend. **Joker lap** is mandatory, 10–12 m wide, physically separated, not at
the first or last corner. Needs branching (spec 006, D10).

**Street circuit**. 1–3 m runoff, concrete everywhere, debris fence, buildings and
roads within 15 m, escape roads at the end of long straights.

---

## 3. OSM tagging reference for circuits

### 3.1 Centreline and track

| Tag | Use | Notes |
|---|---|---|
| `highway=raceway` (way) | Centreline | 45,704 ways. Often one way per named corner. Stitch into a loop. |
| `sport=motor/karting/motocross` | Discipline | On 85 % of raceways. Use to drop karting/motocross tracks in the same bbox. |
| `oneway=yes` | Racing direction | 21 %. Way direction usually = racing direction. |
| `width=*` | Width | Only 2.6 %, often an estimate. Do not trust. |
| `surface=*` | Surface | 35 %. Useful for rallycross (asphalt/dirt split). |
| `bridge=yes`, `tunnel=yes`, `layer=*` | Crossovers | Suzuka, Monza banking. Gives elevation order. |
| `highway=raceway` + `area=yes`, `area:highway=raceway` | Track surface polygon | Rare (215). Do not rely on it; buffer the centreline. |

**Pitfalls:** alternative layouts share ways (Silverstone National/International,
Daytona road/oval, MotoGP chicanes, RBR "Long Lap Penalty"). Karting and motocross
tracks sit in the same bbox. A few relations use `type=circuit` (not standard).

### 3.2 `raceway=*` values that really exist

There is no wiki page for `raceway`. It has 1,010 uses in total.

| Value | Count | Value | Count |
|---|---|---|---|
| `start-finish` | 346 | `marshal_post` | 10 |
| `start` | 147 | `grid` | 10 |
| `finish` | 91 | `run-off` | 7 |
| `kerb` | 72 | `stands` | 7 |
| `service` | 53 | `pit` | 6 |
| `gravel trap` (with space) | 50 | `track` | 5 |
| `pitlane` | 47 | `paddock` | 4 |
| `pit_lane` | 28 | `escape` | 11 |
| `runoff_area` | 20 | | |

These do **not** exist: `raceway=gravel_trap`, `raceway=runoff`, `highway=pit_lane`,
`barrier=tecpro`, `barrier=safer`. Treat `raceway=*` as a bonus, not a source.

### 3.3 Pit lane

Mapped as `highway=raceway` + `oneway=yes` + a name, sometimes `maxspeed`
(Daytona: `55 mph`). Detect it with **both**:
- **Name regex:** `pit|box|boxen|corsia|stand|garage|paddock` (case-insensitive, several languages).
- **Topology:** a branch that leaves the main loop before the start/finish line, runs
  roughly parallel to the main straight within about 10–40 m, and rejoins after it.
- GT relations may mark it `role=pit_lane` (the GT importer already excludes it).

### 3.4 Barriers

| Tag | Count | Map to |
|---|---|---|
| `barrier=wall` (+ `wall=concrete`, `wall=jersey_barrier`, `wall=tire_barrier`, `wall=SAFER`) | 6.2 M | `Concrete` unless subtype says tyres |
| `barrier=tyres` (also `tires`, `tyre`, `dug-in_tyres`, `material=tyres`) | 796 | `TireWall` |
| `barrier=guard_rail` | 231 k | `Steel` (check `material=tyres` → `TireWall`) |
| `barrier=jersey_barrier`, `barrier=retaining_wall` | 44 k | `Concrete` |
| `barrier=fence` | 9.7 M | Fence (visual). Only near and parallel to the track = debris fence. |
| `barrier=hedge`, `gate`, `kerb`, `bollard` | — | Ignore |

Silverstone has about 80 walls, RBR 51 (plus 26 tyre ways), Monza 38. Most walls have
no subtype. **The pit wall is also `barrier=wall`** — it looks like a close barrier on
the pit side. Handle it on purpose (§5.6).

### 3.5 Runoff and landcover

| Signal | Meaning | Examples |
|---|---|---|
| `natural=sand`, `natural=shingle` | Gravel trap | Silverstone 18 + 3, RBR 14–15, Monza 10, Zandvoort 12 + 7 |
| `surface=gravel/sand` on an area without `highway` | Gravel trap | |
| `surface=asphalt` area, `area:highway=*` | Asphalt runoff | rare (2 at Silverstone) |
| `landuse=grass` | Grass | 243 polygons at Silverstone |
| `golf=bunker` + `surface=sand` | **Not** a gravel trap | 43 at Silverstone — exclude `golf=*` |

### 3.6 Grandstands, buildings and surroundings

| Tag | Use |
|---|---|
| `building=grandstand` | Grandstand footprint. Good coverage: Silverstone 22 named, Monza 44, RBR 8–9. Some have `height`, `roof:shape`. |
| `leisure=stadium` | Oval venue / main stand (Daytona: `height=42.6`). |
| `leisure=bleachers` | Small stands. |
| `leisure=sports_centre` + `sport=motor` | Venue boundary. Best query area and outer fence line. |
| `building=garage/garages/roof/yes` near the pit lane | Pit building. `building=pit` has only 3 uses. |
| `man_made=tower` | TV / timing tower. |
| `natural=wood`, `landuse=forest`, `natural=scrub`, `natural=tree_row`, `natural=tree` | Trees. |
| `natural=water`, `waterway=*` | Lakes (Daytona infield), streams. |
| `amenity=parking`, `highway=service` (+ `service=parking_aisle`) | Car parks, service roads (334 at Silverstone). |

`ele` and `incline` are almost never on raceways. For elevation use a DEM
(SRTM or Copernicus GLO-30) outside this pipeline.

---

## 4. Getting the data

### 4.1 Tested combined Overpass query

Verified on the Red Bull Ring (501 elements). Bbox order is south, west, north, east.

```
[out:json][timeout:120][bbox:47.212,14.750,47.232,14.775];
way[highway=raceway]->.track;
(
  way.track;
  nwr[raceway];
  nwr[highway=raceway][area=yes];
  nwr["area:highway"=raceway];
  way[barrier~"^(wall|guard_rail|tyres|fence|jersey_barrier|retaining_wall|dug-in_tyres)$"](around.track:150);
  nwr[natural~"^(sand|shingle|scrub|wood|water|grassland|tree_row)$"](around.track:400);
  nwr[landuse~"^(grass|forest|meadow|farmland)$"](around.track:400);
  nwr[surface~"^(gravel|sand|pebblestone|fine_gravel|grass|asphalt)$"][!highway][!golf](around.track:150);
  nwr[building](around.track:400);
  nwr[leisure~"^(stadium|sports_centre|bleachers)$"];
  nwr[amenity=parking](around.track:800);
  way[highway=service](around.track:300);
  node[natural=tree](around.track:300);
  nwr[man_made~"^(tower|gantry)$"](around.track:200);
);
out geom qt;
```

- `out geom` puts coordinates on every way. Relations keep member roles
  (`outer`/`inner`); you assemble multipolygon rings yourself.
- Keep **node tags** (do not use `out skel`). The current caches lost them.
- Instead of a bbox you can scope to the venue polygon:
  `way[leisure=sports_centre][sport=motor][name~"Red Bull Ring"]; map_to_area->.a;` then `(area.a)`.
- Big venues can time out (Spa did). Split the bbox or use a mirror
  (`overpass.kumi.systems`, `overpass.private.coffee`).

### 4.2 Rules for fetching

- Fetch **offline, once**, into `target/osm_cache/<id>_env.json`. Never at game runtime.
- Public overpass-api.de: < 10,000 requests/day, < 1 GB/day, default timeout 180 s.
  Send a descriptive User-Agent (the skill uses `tdrace-osm-tool/1.0`).
- The OSM Map API (`/api/0.6/map`) is for editors. It has a 0.25 deg² / 50k-node limit.
  Prefer Overpass for read-only bulk data.
- Record fetch date and query text with the cache (see §5.1).

### 4.3 Licence (ODbL)

- Credit **"© OpenStreetMap contributors"** with a reference to
  openstreetmap.org/copyright. In a game: credits screen, splash or about menu.
- Rendered track visuals are a *Produced Work*: attribution only.
- A shipped or published extract (cached JSON, generated track JSON with OSM
  geometry) is a *Derivative Database*: it must be offered under ODbL.
- Today nothing in the repo carries this attribution (D12).

---

## 5. Algorithms: from OSM to game data

Constraint: the importers are Python standard library only. Keep it that way unless
`TECH_STACK.md` approves a geometry library (it is still a template today). All
algorithms below need only `math`.

### 5.1 Keep the transform

To add any feature to an existing circuit, you must project it with **the same
transform** that made the centreline. Today no transform is stored.

Store per circuit (new `OsmImportRecord`, or extend `CircuitProvenance`):

| Field | Example |
|---|---|
| `origin_lat`, `origin_lon` | centroid used for projection |
| `rotation_rad` | total rotation (all passes, including GT fine rotation) |
| `scale` | 0.5 for GT |
| `translation` | offset that put the start at (0, 0) |
| `way_ids`, `relation_id`, node slices | what was stitched |
| `raw_osm_length_m`, `target_length_m` | ratio flags broken stitches (Nyirád 1.33×) |
| `fetched_at`, `query` | reproducibility |
| `attribution` | "© OpenStreetMap contributors, ODbL" |

Forward transform for any OSM point:

```
(x, y) = equirect(lat, lon, origin)          # x = Δλ·cos φ0·R, y = Δφ·R
(x, y) = rotate((x, y), rotation_rad)
(x, y) = (x, y) · scale + translation
```

**Check before trusting it:** re-run the importer, then compare the new waypoints with
the preset in code. Max deviation must be < 0.5 m. OSM may have been edited since the
preset was made. If it moved, stop and decide: keep the old centreline and fit the
new features to it, or re-import.

### 5.2 Stitching and start

1. Build a graph: raceway ways are edges, shared end nodes are vertices.
2. Main loop = the cycle that contains the `raceway=start-finish` (or `start`) node
   and is the longest simple cycle with `sport=motor`, excluding pit-regex names.
3. Measure gaps **in metres after projection** (fixes D2). Max gap 5 m; closure gap
   15 m (the skill already states these rules; no script implements them).
4. Start point = `raceway=start-finish` node when present. Otherwise keep the current
   choice (hand-picked or `find_longest_straight`).
5. Take heading over a **distance** (e.g. 60 m), not over "6 nodes".

### 5.3 Resampling (better corners)

Today: 26–36 equal points (≈ 100 m on GT). Chicanes vanish.

Better: **curvature-adaptive** points.
1. Resample the raw polyline every 2 m.
2. Douglas-Peucker with tolerance 0.5–1.0 m (game scale).
3. Enforce max spacing 40 m (GT) / 15 m (kart), min spacing 8 m (validation needs ≥ 3 m).
4. Switch the spline to **centripetal Catmull-Rom** (α = 0.5) when spacing is uneven.
   Uniform Catmull-Rom overshoots on uneven spacing.

**Trade-off:** this changes lap feel, lap length, AI lines and the length tests.
Do it per circuit with a before/after lap-time check, not in a bulk pass.
Normalise the curvature test (D1): `sinθ = cross / (|v1|·|v2|)`.

### 5.4 Per-side distance to barriers (runoff width)

For each centreline sample `p` (every 5–10 m) with unit tangent `t` and left normal
`n = (-t.y, t.x)`:

```
for side in (+1 left, -1 right):
    ray from p along side·n, inside a ±15–20° cone
    for each segment (a, b) of each barrier way:
        u = clamp(((p-a)·(b-a)) / |b-a|², 0, 1)
        q = a + u·(b-a);  d = |q - p|
        s = t.x·(q.y-p.y) - t.y·(q.x-p.x)        # >0 left, <0 right
        keep if sign(s) == side and |t·(q-p)| ≤ 0.5·d + 5 m   # perpendicular filter
    d_barrier[side] = min kept d
```

- Without the perpendicular filter, the barrier of the **next** corner wins.
- Also record the distance to **other track sections** on the same ray. The infield
  often faces another part of the circuit, not a wall.
- `wall_distance = d_barrier − track_half_width` (the field is measured from the edge).

### 5.5 Surface bands on each side

Ray-cast along `±n` against polygons (`natural=sand|shingle`, `surface=*` areas,
`landuse=grass`). Record enter/exit distances for every polygon hit, e.g.:

```
right: grass 0–3 m, gravel 3–45 m, tyres at 47 m
```

Classify: sand/shingle/gravel → `Gravel`; asphalt area → `Asphalt`; else `Grass`.
Use even-odd point-in-polygon to know if the ray **starts** inside a polygon.

The current model has one runoff surface per side. Until bands exist (§8, P2), pick
the **dominant surface in the first 60 % of the corridor**. That is what cars hit.

### 5.6 Cleanup and fallbacks

1. **Pit side:** if the pit lane runs on side S within about 25 m, set side S's
   wall to the pit wall: `Concrete`, placed at about half the gap between track and
   pit lane. Ignore the other walls behind it.
2. **Outliers:** rolling median over 5–9 samples, then Gaussian smoothing
   (σ ≈ 20–40 m of arc). This fills gaps at gates and marshal openings.
3. **Rate limit:** ≤ 1 m lateral change per 2 m of arc, so walls do not zig-zag.
4. **Clamp:** min = verge (1 m) + kerb (1.4 m); max per module (§6.1).
5. **No barrier found** (≈ 20 % of samples at RBR): use the corner rules in §6.1.
6. **Street circuit** (buildings or roads within 15 m, or known street name):
   1–3 m, `Concrete` everywhere.
7. **Oval** (≤ 4 corners, `leisure=stadium`): outside wall at the edge; inside apron + grass.

### 5.7 Scale correction (important)

GT and NASCAR road courses are built at **0.5× length but real width**. So:
- Multiply every OSM lateral distance by `scale` (0.5) → `d_game`.
- **Lateral space is tighter than in reality.** Two parallel sections 40 m apart in
  reality are 20 m apart in the game. With 13.5 m of track width, only about 6.5 m is
  left for both runoffs. Always clamp:
  `wall_distance ≤ (gap_to_other_section − own_half_width − other_half_width) / 2`.
- Grandstands and buildings: project their centres with the transform, but keep their
  **depth** real-sized (people and cars are real-sized) and scale their **length**
  by `scale`.
- Fix the `scale` label (D3) so tools know the factor.

### 5.8 Reduce to waypoints

The authoring unit is the waypoint. The spline interpolates `*_wall_distance`
linearly **only if both neighbours set it** (D5). So:
- Set `left_wall_distance` and `right_wall_distance` on **every** waypoint of the circuit.
- Per waypoint, use the **minimum** smoothed distance in the window
  [previous midpoint, next midpoint]. Minimum = safe (no wall inside the runoff).
- With 100 m waypoint spacing the runoff can only change every 100 m. That is fine
  for straights but crude at corners. Adaptive resampling (§5.3) fixes this.

### 5.9 Grandstands, buildings, trees

- **Grandstand** from a `building=grandstand` polygon:
  1. Minimum-area bounding rectangle (rotating calipers on the convex hull).
  2. `length` = long side × scale, `depth` = short side (real), `center` = transformed centre.
  3. `angle` = long-side direction. Flip by π if the front (`center − facing·depth/2`)
     points away from the nearest track sample.
  4. `tiers` from `height` (≈ 1 tier per 0.8 m) else 6–10. `style`: `CoveredStadium`
     if `roof:shape` is present or a `building=roof` overlaps, else `OpenBleachers`.
  5. Push it back so its front is ≥ wall + 2 m (fence + FIA "1 m behind barrier").
- **Pit building:** the longest building within about 30 m of the pit lane, on the side
  away from the track.
- **Trees:** keep `natural=tree` nodes as they are. Fill `natural=wood` / `landuse=forest`
  polygons with Poisson-disc samples (6–10 m spacing). Keep out of a buffer of
  `wall + 5 m`. Cap the count per circuit until meshes are cached (§8, P2).
  Pick species by biome: Monza oak (park), Spa/Nürburgring pine, Interlagos/Bahrain/
  Marina Bay palm, Suzuka sakura/cypress, Montreal maple.
- **Water, parking, service roads:** render-only polygons and thin ribbons in a new
  environment layer (§8, P2). Clip everything against the track + runoff keep-out buffer.

---

## 6. Placement rules when OSM has no data

Use `spline.curves` (`TrackCurve`: `degree`, `direction`, `entry/apex/exit_distance`,
`min_radius`). "Outside" of a corner = the side opposite to `direction`.
"Approach" = straight length before `entry_distance`.

### 6.1 Runoff by corner (game metres, GT at 0.5×)

| Case | Outside | Inside | Barrier |
|---|---|---|---|
| Straight | 2–4 m grass verge | 2–4 m grass | `Steel` / `Concrete` |
| Degree 1–2 (fast) | 15–30 m gravel (5 m asphalt strip first on modern tracks) | 3–5 m grass | `TireWall` at the end |
| Degree 3 (medium) | 10–20 m gravel | 3–5 m grass | `TireWall` |
| Degree 4–5 (slow/hairpin) after approach > 300 m | 8–15 m asphalt + gravel beyond; straight-on escape at entry | 2–4 m grass | `TireWall` |
| Chicane (two opposite curves within 60 m) | 6–10 m asphalt | sausage-kerb zone | `Steel` |
| Street circuit | 1–3 m | 1–3 m | `Concrete` |

Module caps: GT ≤ 50 m, rallycross ≤ 25 m, kart ≤ 15 m (3–15 m typical), NASCAR:
outside 0–1.5 m, inside apron 4–9 m then grass.

Extend the runoff from `entry_distance − 30 m` to `exit_distance + 40 m`, and taper
over 20–40 m at both ends. The biggest runoff is on the outside **at and after the
apex**, where a car that does not make the corner goes.

### 6.2 Kerbs

| Kerb | From | To | Condition |
|---|---|---|---|
| Inside apex | `entry + 0.3·(apex − entry)` | `apex + 0.3·(exit − apex)` | degree ≥ 2 |
| Outside exit | `apex` | `exit + 10 m` | degree ≥ 2 |
| Sausage | behind the inside kerb | same | chicanes only |
| None | straights | | never on straights or at wp0 |

Today kerbs are booleans spread over half a segment (≈ 50 m at Monza). Correct kerb
ranges need either denser waypoints or a kerb-range list (§8, P1).

### 6.3 Grandstands

Rank candidate spots and place the best 4–8 per GT circuit:
1. **Main straight**, opposite the pit building, 100–250 m long (real ×0.5 for length).
2. **Heaviest braking corner** (degree ≥ 4 after the longest straight), outside, facing the apex.
3. **Chicanes** and **hairpins**.
4. Any corner with a large runoff (spectators want a view).
Front ≥ wall + 2 m. Length ≈ corner length (entry → exit). Tiers 6–12.
Ovals: one huge front-stretch stand, `CoveredStadium`, 20–40 tiers.

### 6.4 Pit complex

- Side: from OSM if known; otherwise the **inside** of the main straight for ovals and
  the side with more free space for road circuits.
- Real layout from the track: verge 2–5 m → pit wall (concrete) → fast lane 3.5 m →
  inner lane ≈ 8 m → garages → paddock. Keep **widths real** even at 0.5× length.
- Pit entry before the last corner, exit after the first corner, both off the racing line.
- Garages: 10–12 teams × 2–3 bays; ≈ 250–350 m real (×0.5 at GT scale), box roofs
  with a strong shadow. Paddock behind: motorhomes, trucks (small rectangles).
- Grid: GT spacing today is 10 m / 2.5 m stagger. The F1 rule is 8 m between cars.

### 6.5 Other trackside objects

| Object | Rule | Top-down look |
|---|---|---|
| Start gantry | Across the track at the start line, ≥ 4.5 m high | Bar across the track, drawn above cars, long shadow |
| Marshal posts | Every 250–500 m real, outside of corners, behind the barrier | 2×2 m hut, orange roof, small flag |
| Ad boards | Along straights, ≥ 3 m from edge, never outside of corners | Thin coloured strips on the wall line |
| Debris fence | Behind barriers in front of grandstands and on straights | Thin grey line with posts every 4–6 m |
| Tyre stacks | End of runoff at degree ≥ 3 corners | Existing tyre-bundle obstacles |
| TV / timing tower | Near start line, behind pit building | Small tall block with long shadow |
| Service road | Behind the barrier, 3–4 m wide, loops the circuit | Grey ribbon, no collision |
| Car parks | Outside the venue fence | Grey area with car rows |

---

## 7. Visual design guide

### 7.1 Layer order (back to front)

1. Ground base (backdrop)
2. Large landcover: farmland, forest floor, water with a shore tint
3. Car parks and service roads (desaturated grey)
4. Grass verge
5. Runoff bands (asphalt runoff a little lighter/bluer than the track; gravel)
6. Track asphalt
7. Kerbs
8. Paint: white edges, grid boxes, start/finish, pit lines
9. Rubber racing line (subtle dark overlay)
10. Barriers
11. Contact shadows
12. Tall objects: grandstands, garages, towers, gantries
13. Tree canopies (may overhang runoff, never the track)

The current pipeline already follows most of this (see analysis §2.5). Layers 2, 3,
9 and 12 (except grandstands) are missing.

### 7.2 Readability from above

- Real 10 cm lines and 1 m kerbs vanish. Draw lines 0.3–0.5 m and kerbs 1.5–2 m.
- Highest contrast: track against runoff. Lowest contrast: far environment.
- **Remove the dashed centre line** on race asphalt (D8). Keep it for road/rally stages.
- One light direction for every shadow (the code already uses one `SHADOW_OFFSET`).
- Clip environment detail with a keep-out buffer so nothing reads as an obstacle on track.

### 7.3 Textures

- World-space UVs for ground (already done). Kerbs use UVs along the arc (already done).
- Break tiling with low-frequency tint (already ±18 %) and **mowing stripes** on grass
  (spec 016 promises them; not implemented).
- Asphalt: fine grain, patch repairs, rubber build-up on the racing line
  (`TrackWearState` exists and is unused; `asphalt_groove.png` exists and is unused).
- Gravel: warm beige-grey, soft edge into grass (the 0.6 m fringe already exists).

### 7.4 Starting palette

| Surface | Hex |
|---|---|
| Grass | `#5E8F3A` / `#6FA046` (mowing stripes ±6 % lightness) |
| Dry verge | `#8CA055` |
| Gravel | `#CDB891` / `#B9A57E` |
| Track asphalt | `#4A4D52` |
| Asphalt runoff | `#5C6168` (often with green/blue painted bands) |
| Kerbs | `#D7263D` + `#F2F2F2`; outer strip green `#2E8B57` |
| Walls | `#BFC3C7` |
| Tyre stacks | `#1F1F1F` with coloured belts |
| Water | `#3F7FA8` |
| Car park | `#6E7075` |

### 7.5 Reference games

- **Circuit Superstars:** simple polygon art plus real motorsport details (kerbs,
  gravel, tyre walls, grandstands). Closest target for TdRace.
- **Art of Rally:** limited palette per biome, flat colours, lush grass, bushy trees.
  Restraint makes the road stand out.
- **Micro Machines:** edge objects can *be* the barrier language.
- **Super Sprint:** every element reads as a silhouette.
- Common thread: soft shadows under objects, and dense "life" (spectators, marquees,
  motorhomes, parked cars) kept away from the racing surface.

---

## 8. Engine changes needed (proposed, in priority order)

These are proposals. None is implemented. Each P1+ item needs a Keel spec first.

**P0 — fixes (small, one file each)**

| Fix | Defect |
|---|---|
| Normalise the GT importer curvature test | D1 |
| Measure stitch gaps in metres; add closure check | D2 |
| Set `scale: "0.5x"` on 0.5× tracks | D3 |
| Interpolate wall distance when one side is `None` (use the global offset as the other end) | D5 |
| Same fallback and wall check in render and physics runoff | D6 |
| Finish checkerboard to track width only | D7 |
| No dashed centre line on circuit asphalt | D8 |
| Editor keeps the track's barrier offset/type; separate left/right distance | D9 |

**P1 — data model, still small**

1. `left_wall_type` / `right_wall_type` per waypoint (today one `wall_type` for both).
2. `TrackAnchor { distance, side, offset }` and a helper
   `Track::place_at(d, side, offset) -> (Vec2, angle)` in `track/mod.rs`. Scenery stored
   by anchor follows geometry edits. Handle the grandstand facing flip inside it.
3. Editor: runoff surface per side; `Virtual` wall type.
4. Kerb ranges from `TrackCurve` (§6.2), stored as distance ranges, not booleans.

**P2 — new content**

1. **Layered runoff:** `left_runoff_bands: Vec<(SurfaceType, f32 width)>` per waypoint,
   with matching render (one quad strip per band) and physics lookup.
2. **Environment layer** (render-only): landcover polygons, water, parking, service
   roads, building footprints. Draw before the runoff pass.
3. **New scenery types:** `Building`, `Fence` (polyline), `MarshalPost`, `Billboard`,
   `Gantry`, `TyreStackLine`. Tecpro and SAFER as barrier variants.
4. **Pit lane:** merge spec 006 branching (D10) and build a pit road branch. Interim
   without branching: open the wall at entry/exit (`right_wall = false`), draw the pit
   lane as an `Asphalt` `SurfaceZone` polygon, and build the pit wall from `Obstacle`
   boxes. Note: `BelowTrack` zones lose to the runoff corridor in `sample_surface`
   (step 4 before step 6), so the pit lane must lie outside the corridor.
5. **Static mesh cache:** build ribbon, runoff, kerb and wall meshes once per track,
   chunked by distance for culling. Needed before adding hundreds of trees and posts.

**P3 — pipeline**

1. One shared Python module for projection, stitching, resampling and transform
   (replace the four copy-pasted scripts). Store `OsmImportRecord` (§5.1).
   *Done for GT, kart and rallycross in `scripts/osm_importer.py` (2026-09-27), including
   the join check and the raw-length check. Still to do: NASCAR, and storing the transform.*
2. `osm_environment_importer.py`: runs §5.4–5.9 and prints per-waypoint wall distances,
   runoff surfaces, grandstands and trees as Rust (same paste workflow as today).
3. Raw-length ratio check in tests (flag ratio outside 0.9–1.1), instead of the
   circular length tests.
   *The importer now warns on this ratio; the Rust tests are unchanged.*
4. DEM elevation (and allow negative elevation, D4). Needs a data source decision.
5. ODbL attribution in credits and in the portal (D12).

---

## 9. Recipe: improve one existing circuit

Example: Monza (GT, 0.5×). Monza is well mapped: 38 walls, 10 `natural=sand`,
44 grandstands, 20 + 11 garage buildings, a named "Pit Lane".

| Step | Action | Check |
|---|---|---|
| 1 | Fetch the §4.1 query for the Monza bbox into `target/osm_cache/monza_env.json`. | File has barriers, sand, grandstands. |
| 2 | Re-run the centreline importer for Monza. Compute the transform. | New waypoints vs `module/gt.rs` preset: max deviation < 0.5 m. |
| 3 | Project all features with the transform. Plot them over the waypoints (SVG in scratch). | Walls sit on both sides; pit lane parallel to the main straight. |
| 4 | Measure per-side barrier distance (§5.4), scale ×0.5, clean (§5.6), clamp (§5.7). | No wall inside the track; no zig-zag. |
| 5 | Classify runoff surface per side (§5.5). | Gravel on the outside of Parabolica, Lesmo, Ascari. |
| 6 | Reduce to waypoints (§5.8). Set both distances on **every** waypoint. | No step jumps (D5). |
| 7 | Pit side: set pit wall (`Concrete`, close) on the pit side of the main straight. | |
| 8 | Grandstands from OSM (§5.9); keep the 6–10 biggest near the track. | Fronts face the track; ≥ wall + 2 m. |
| 9 | Trees from `natural=wood` (Monza is a park): Oak, capped count. | Frame time still OK in the densest view. |
| 10 | Rebuild, run validation and tests. | `cargo test` green; no `ERR_WALL_INTRUDES_TRACK`. |
| 11 | Look at it in the circuit viewer and in a race. Screenshot before/after. | Visual check, lap time within a few % of before. |

Suggested order of circuits:
1. **GT** (18 circuits, no scenery today, best OSM data): Monza, Silverstone, Red Bull
   Ring, Zandvoort, Spa (split the bbox; Spa timed out).
2. **Rallycross** (already has some scenery; caches exist but the Overpass-format ones
   must be re-fetched with node tags).
3. **NASCAR ovals** (mostly hand-built; use the oval template in §2.5, grandstands
   and pit road on the front stretch; OSM for infield water and parking).
4. **Karts** (small; tyre walls over grass; 1.8 m verge).

---

## 10. Glossary

| Term | Meaning |
|---|---|
| Runoff | Area between the track edge (verge) and the first barrier. |
| Verge | 1–5 m strip right next to the track edge. |
| Gravel trap | Runoff filled with loose gravel to stop cars. |
| Apex | Point of a corner closest to the inside. |
| Armco | Steel guard rail. `BarrierType::Steel`. |
| Tecpro | Linked foam-block barrier. |
| SAFER | Steel-and-foam barrier on oval walls. |
| Apron | Flat paved strip on the inside of an oval. |
| Joker lap | Longer alternative route every rallycross driver must take once. |
| Transform | Projection + rotation + scale + translation from OSM lat/lon to game metres. |
| Overpass | Read-only query API for OSM data. |
| ODbL | Open Database Licence, the OSM data licence. |
| DEM | Digital elevation model (terrain height grid). |

---

## 11. Sources

- OSM wiki: [highway=raceway](https://wiki.openstreetmap.org/wiki/Tag:highway=raceway),
  [barrier=tyres](https://wiki.openstreetmap.org/wiki/Tag:barrier=tyres),
  [area:highway](https://wiki.openstreetmap.org/wiki/Key:area:highway)
- taginfo: [raceway values](https://taginfo.openstreetmap.org/keys/raceway#values)
- Overpass usage policy: <https://dev.overpass-api.de/overpass-doc/en/preface/commons.html>
- OSMF attribution guidelines: <https://osmfoundation.org/wiki/Licence/Attribution_Guidelines>
- FIA Appendix O 2024: <https://www.fia.com/sites/default/files/appendix_o_2024_published_11.06.2024_0.pdf>
- FIA F1 Sporting Regulations 2025: <https://www.fia.com/system/files/documents/fia_2025_formula_1_sporting_regulations_-_issue_5_-_2025-04-30.pdf>
- FIA Appendix H 2022: <https://www.fia.com/sites/default/files/appendix_h_2022_published_23.03.2022.pdf>
- FIA Karting Appendix 13 (2024): <https://www.fiakarting.com/sites/default/files/2024-07/Appendix%2013%20-%20Licence%20criteria%202024.pdf>
- FIM Europe RR07 (2024): <https://www.fim-europe.com/wp-content/uploads/2024/04/RR-07-FIM-EUROPE-STANDARDS-for-Permanent-RR-Circuits-15_04_2024.pdf>
- Barriers: [Tecpro](https://en.wikipedia.org/wiki/Tecpro_barrier), [SAFER](https://en.wikipedia.org/wiki/SAFER_barrier),
  [Armco in F1](https://www.formulaonehistory.com/armco-in-formula-1/), [catch fences](https://buildingspeed.org/2015/07/07/a-catchfence-primer/)
- Gravel vs asphalt: [RaceFans](https://www.racefans.net/2016/11/02/gravel-traps-arent-easy-answer-corner-cutting/),
  [Motor Sport](https://www.motorsportmagazine.com/articles/single-seaters/f1/return-of-the-gravel-trap-why-f1-is-welcoming-them-back/)
- Kerbs: [Flow Racers](https://flowracers.com/blog/types-of-kerbs-f1/)
- Pit boxes: [Sepang architecture](https://www.sepangcircuit.com/architecture)
- Daytona: [Wikipedia](https://en.wikipedia.org/wiki/Daytona_International_Speedway)
- Games: [Circuit Superstars interview](https://www.overtake.gg/news/interview-with-original-fire-games-creator-of-circuit-superstars.265/),
  [Art of Rally interview](https://gamingbolt.com/art-of-rally-interview-art-style-development-and-more)
