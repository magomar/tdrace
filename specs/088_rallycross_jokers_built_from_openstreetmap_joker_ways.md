---
type: Feature Spec
template: feature
title: "Rallycross Jokers Built From OpenStreetMap Joker Ways"
description: "Builds the World RX joker branch from OpenStreetMap on the 13 circuits where OSM shows it, places and sizes the synthetic joker from the published venue description elsewhere, replaces the 30-70 m joker length rule with a lap-time rule, keeps joker walls out of the main road's wall gap, and lets a bot stuck on the other branch follow it."
status: draft
created: 2026-10-05
generated: { by: agent/antigravity, at: 2026-10-04T22:08:08Z }
depends_on:
  - "081"
amends:
  - "081"
  - "082"
---


# Feature Spec: Rallycross Jokers Built From OpenStreetMap Joker Ways 🌟

[Spec 081](081_rallycross_joker_lap_segments_for_classic_and_openstreetmap_circuits.md) says the World RX joker paths are "extracted directly from OpenStreetMap". They are not. `crates/tdrace-app/src/bin/build_world_rx_joker.rs` builds all 20 jokers the same way: the main line between two hand-picked waypoints, bulged sideways until it is exactly 42 m longer. Meanwhile `scripts/osm_importer.py` drops the joker ways it finds in OSM on at least 5 circuits. This spec builds the joker from OSM wherever OSM shows it, and places and sizes the synthetic joker from the published venue description elsewhere. It amends spec 081 and one bot rule of [spec 082](082_mandatory_rallycross_joker_lap_tracking_and_penalty_enforcement.md); both are `implemented` and cannot change in place. Beads: `tdrace-3z9u`.

Decisions (Mario):
- 2026-10-04: use the mapped jokers with their real length; judge a joker by the time it costs, not by its length; fix the bot that gets stuck on the real Höljes joker in this change.
- 2026-10-05: also use the OSM roads that are not tagged as a joker but match the venue description: Killarney, Mettet, and Riga with its two parallel roads swapped (the lap was on the joker road). Elsewhere, first check whether the lap is on the joker road; if not, make the synthetic joker match the description.

---

## 🗺️ User Flow & Interface Design

No new screens. On the 13 circuits below, the joker branch a player or bot drives follows OSM: it leaves and rejoins the main road where the real one does, with its real length and the surface its OSM ways carry.

### Jokers from OSM

| Circuit | OSM joker ways | Joker vs. main branch | Joker lap cost (model) |
| :--- | :--- | ---: | ---: |
| `holjes_rx` | 599300792, 599300808, 599300811, 599300815 ("Joker"), 599300817 | +28 m | +3.5 s |
| `hell_rx` | 1069390969 ("Joker") | +90 m | +4.4 s |
| `loheac_rx` | 787615502, 787615507 ("Tour Joker") | −24 m | +4.7 s |
| `estering_rx` | 282853377 ("Joker") | −19 m | +1.7 s |
| `montalegre_rx` | 1096210263 ("Joker") | +3 m | +1.9 s |
| `catalunya_rx` | 831804325, 921317985, 1560896068 (relation 11362868 `joker_lap`) | +101 m | +6.0 s |
| `essay_rx` | 787532794 ("Tour Joker") | +30 m | +2.3 s |
| `dreux_rx` | 297738881, 1311041714 ("Tour Joker") | −45 m | +2.2 s |
| `lavare_rx` | 858729742 ("Tour Jocker"), 858729748 ("Tour Joker") | +21 m | +3.9 s |
| `lessay_rx` | 788196385, 788196391 ("Tour Joker") | +57 m | +5.0 s |
| `killarney_rx` | 1214903815, part of 42125321, 1214903819 (untagged) | +83 m | +6.4 s |
| `mettet_rx` | parts of 178240077, 178240078, 178240074 (untagged) | +32 m | +7.0 s |
| `riga_rx` | the lap's old northern road (588947722 … 588947714) | +17 m | +2.4 s |

How the last three match the published descriptions:
- **Killarney:** the joker "split[s] left, running around the outside initially on tarmac before tightening into a right-hander, back onto an unsealed surface", and merges before the finish. The two oneway ways and the old circuit piece between them do exactly that. Official: about +78 m.
- **Mettet:** the joker is "at the end of the lap before the flying finish", and "slow", about +72 m. OSM has an asphalt loop off the start straight. Of its three routes, the cut through 178240078 (+32 m) is the one bots drive cleanly. The +95 m route turns at 2.7 m and costs 10 s; the full loop (+139 m) stopped 2 of 3 bots.
- **Riga:** the joker runs "completely side by side" with the main track, each with a jump, at the end of the lap, about +60 m. Two parallel roads leave node 5098959004 and rejoin way 1435177485. The lap was on the northern one; the main lap now takes the southern one (588947713 …), and the joker the northern one, 8.6 m longer in OSM.

### Synthetic jokers

| Circuit | What the description says | Joker | Joker lap cost |
| :--- | :--- | :--- | ---: |
| `lydden_hill` | "between Pilgrims and Chessons Drift", 1420 m vs the 1335 m lap | waypoints 12-18 (Pilgrims 9-14, Chessons 14-18), +73 m (the official +85 m on this 1150 m lap) | +2.6 s |
| `kouvola_rx` | "a joker section towards the end of the lap", 1120 m vs 1060 m | around the outside of the last hairpin, waypoints 25-28, +54 m (+60 m scaled) | +3.5 s |
| `nyirad_rx` | leaves on the inside of a left turn, rejoins at a very sharp left before the climb to the finish, 1290 m vs 1220 m | unchanged: waypoints 44-52 on the right, +42 m (see below) | +2.3 s |
| `croft_rx`, `silverstone_rx`, `erx_motor_park` | nothing found | unchanged, +42 m | |
| `spa_rx` | the joker is 11 m *shorter* than the lap, parallel to it | unchanged, +42 m: a sideways bulge can only be longer | |

No unmapped circuit has an OSM road that would make its lap the joker road (the Riga test), except Riga itself. Nyirád keeps its old joker: on the inside (left) the bulge folds back on itself for every split and merge tried, and longer bulges on the right fold, reach the return road, or overlap the asphalt where the main road turns to dirt. Kouvola's only OSM bypass goes back along the circuit and turns 180°.

---

## ⚙️ Backend Models & API Endpoints

### 1. OSM importer (`scripts/osm_importer.py`)
- Each mapped circuit's `RALLY_TRACKS` config lists its joker ways in race order as `joker_ways`. An entry is a way id, or `(way id, first node, last node)` for part of a way.
- `rally_joker_path` chains them from the node where they leave the lap to the node where they rejoin it. It fails when a way does not join the previous one, when the ends are not on the lap, or when the joker runs against the lap direction.
- The joker points go through the same transform as the lap: metres around the lap centre, the rounding pre-scale, rotation onto +X, FIA scale and start offset. They are resampled about every 10 m, each with the surface of its OSM way.
- `finish_shift_m` moves the start/finish line along the lap in the same frame, so the scenery stays in place. Mettet moves it 3 waypoints back (123.1 m) onto the start straight: its joker leaves the straight 68 m before the old line and rejoins just after it, and a hairpin follows 20 m later. Riga moves it 1 waypoint on (35.4 m): its joker rejoins 17 m before the old line. A whole number of waypoints keeps the waypoints, and the walls built on them, where they were.
- `python3 scripts/osm_importer.py rally --jokers` writes the jokers to `assets/osm/rx_jokers.json`, keyed by catalog id. The file is outside `tracks/` because `crates/tdrace-core/build.rs` takes every JSON file there for a circuit.

```json
{ "holjes_rx": { "osm_ways": [599300792], "points": [[150.9, -4.0]], "surfaces": ["Asphalt"] } }
```

### 2. Joker builder (`crates/tdrace-app/src/bin/build_world_rx_joker.rs`)
- `cargo run --bin build_world_rx_joker -- [tracks/rally] [assets/osm/rx_jokers.json]`.
- Each config has a `JokerSource`: `Osm`, or `Synthetic { split_idx, merge_idx, side, surface, bank_angle, target_delta }`. A synthetic joker is built to its `target_delta` (42 m where nothing describes the real one) and must come within 3 m of it.
- An OSM joker's end nodes must lie within 6.5 m of the main line (the spline cuts the OSM corners: 4.7 m on `estering_rx`).
- The mapped split and merge are where the OSM joker leaves and rejoins the main road, not its end nodes. The `holjes_rx` joker ends in a lane that runs 3 m beside the main line for about 70 m, and a car on the main road there read as on the joker.
- The joker splits at the last main waypoint before the mapped split and merges at the first main waypoint after the mapped merge. So the main route keeps its waypoints and its exact shape. An extra waypoint at the mapped split moved the main line enough to put `dreux_rx` bots on a hairpin wall.
- Between those waypoints the joker follows the main line up to 10 m before the mapped split, then the OSM points, then the main line from 10 m after the mapped merge. Leaving out the fork points lets the spline round the fork. Through them, the joker kinked, `holjes_rx` walls crossed, and `catalunya_rx` cost 2 s more.

### 3. Joker walls (`Track::generate_network_walls`, `crates/arcade-race-core/src/track/mod.rs`)
Two more kinds of joker wall piece are dropped, on every joker track:
- A piece nearer the main road than the main walls stand: the local main wall gap less 0.5 m, at most 3.5 m. One stood about 1 m off the `essay_rx` main road edge, and bots on the main route hit its end. Without the 3.5 m cap, `riga_rx` (main walls about 10 m off on its run-off) lost the walls of its side-by-side joker.
- A piece within 0.5 m of a main wall along its length. It only doubles that wall, and once the pieces are joined the line zigzags across it (`holjes_rx` validation: `ERR_WALL_SELF_INTERSECTION`).

### 4. Bot on the other branch (`BotAiDriver::compute_controls`, `crates/race-kit/src/ai/mod.rs`) — amends spec 082
Spec 082 says a bot keeps its route from the split to the merge, and that every finished bot takes exactly one joker. On `holjes_rx` the main route turns left at the split and the joker goes straight on. A main-route bot that ran wide there ended on the joker, kept steering for the main road behind the joker's inside wall, and was stuck for about three minutes. It then drove out through the joker and was credited a second joker.
- A bot whose car is more than 2 m outside its own road, inside a branch road that its route does not use, **and** stuck (its no-progress watchdog has fired at least once), switches to the route of that branch until its next route choice at the start of the next lap.
- Without the "stuck" condition, a `killarney_rx` main-route bot that drifts onto the joker every lap took it every lap.
- A bot that follows the joker this way can finish with more than one joker. That is no penalty; it costs the time of the joker.

### 5. Data
The changed circuits are rebaked from `tracks` commit `77a0abe`, the state before any joker bake, as in the spec 082 amendment. Two need a new base before the joker bake; `track_bake --rebuild` does not reproduce either of the hand-fixed files (on `riga_rx` it fails validation even unchanged):
- **`mettet_rx`:** waypoints rotated 3 back (the `finish_shift_m` above), checkpoints and grid rebaked, walls kept. The 4 main wall pieces inside the joker island (between the main branch and the loop) are removed: with the joker's inside wall they formed a wedge that stopped a joker-route bot for 100 s.
- **`riga_rx`:** the main lap re-imported with the swapped roads, the two hairpin narrowings (width 7.5 m) put back, then `track_bake --rebuild`. Where that left a wall in the road, crossing another wall or a crossing curb, the wall or curb on that side is turned off at the nearest waypoint (left walls at waypoints 9-11, 21-23 and 29; left curbs at 28-29), as the earlier hand fix did. Then waypoints rotated 1 on.
The other World RX files and the Classic RX files are unchanged.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

No access control is involved. `rally_joker_path` and `osm_cut` fail closed on joker data that does not join the lap, so a bad OSM edit stops the import or bake instead of baking a broken joker.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- `cargo test -p tdrace-app --test rally_tracks_tests`
- `cargo test -p tdrace-app --test ai_tests`
- `ruff check scripts/osm_importer.py`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Jokers follow OSM where OSM shows them**
  - [ ] **Given** the 13 circuits in the OSM table
  - [ ] **When** `python3 scripts/osm_importer.py rally --jokers` runs
  - [ ] **Then** `assets/osm/rx_jokers.json` holds one joker per circuit, each starting and ending within 6.5 m of the main line
- **Scenario: Synthetic jokers follow the venue description**
  - [ ] **Given** `lydden_hill` and `kouvola_rx`
  - [ ] **When** `cargo run --bin build_world_rx_joker` bakes them
  - [ ] **Then** each joker lies where the description places it and is within 3 m of its scaled official length (+73 m, +54 m)
- **Scenario: Undescribed circuits keep their joker**
  - [ ] **Given** the 20 World RX files from `tracks` commit `77a0abe` and the new Mettet and Riga bases
  - [ ] **When** `cargo run --bin build_world_rx_joker` bakes them
  - [ ] **Then** the files of `nyirad_rx`, `croft_rx`, `silverstone_rx` and `erx_motor_park` are byte-identical to the committed ones (`spa_rx` excepted: its committed file already differs from a fresh bake)
- **Scenario: The main route keeps its shape**
  - [ ] **Given** a circuit with an OSM joker
  - [ ] **When** its main layout is built
  - [ ] **Then** it uses exactly the main spline's waypoints
- **Scenario: Every joker costs lap time**
  - [ ] **Given** the 23 official circuits with a joker layout
  - [ ] **When** a car limited by top speed (40 m/s), corner grip (10 m/s²), acceleration (8 m/s²) and braking (10 m/s²) drives the main lap and the joker lap
  - [ ] **Then** the joker lap is between 1.0 and 7.5 seconds slower (`test_rx_joker_costs_lap_time`), and on `holjes_rx` between 2.0 and 4.5 seconds slower
- **Scenario: Joker walls line the joker and stay out of the main road's wall gap**
  - [ ] **Given** the 23 joker circuits
  - [ ] **When** their joker walls are built on load
  - [ ] **Then** no wall crosses another (`test_rally_module_tracks_integrity_and_validation`), and wherever no other road lies beside the joker a wall stands within 6 m of its edge (`test_rx_joker_branch_has_walls_that_stay_off_every_road`; joker walls keep the gap of the main walls around them, 5.4 m on `riga_rx`)
- **Scenario: Bots take one joker unless stuck on the joker (amends spec 082)**
  - [ ] **Given** an 8-bot, 5-lap Rallycross race on `holjes_rx` and on `hell_rx`, bots of all tiers
  - [ ] **When** the race runs (`test_bot_ai_strategic_joker_rx_race_compliance`)
  - [ ] **Then** a bot changes its route between the split and the merge only while its car is off the road of the route it leaves
  - [ ] **And** every finished bot has no penalty, and `jokers == 1` unless it moved onto the joker that way
- **Scenario: Bots drive every changed joker**
  - [ ] **Given** each of the 15 circuits whose joker this spec changes, and a Pro bot of the Balanced, Smooth and Aggressive styles
  - [ ] **When** each bot drives 3 laps on the main route and 3 laps on the joker route in the bot harness
  - [ ] **Then** every bot finishes (on 2026-10-05 the longest stop was 12.5 s, on the `kouvola_rx` joker)

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[x]` `scripts/osm_importer.py` -> `joker_ways` (whole and partial ways), `rally_joker_path`, `finish_shift_m`, `--jokers`, `write_rx_jokers`; the Riga main lap swap.
- `[x]` `assets/osm/rx_jokers.json` -> The 13 OSM jokers in track coordinates.
- `[x]` `crates/tdrace-app/src/bin/build_world_rx_joker.rs` -> `JokerSource`, `osm_cut`, `synthetic_cut`, per-joker `target_delta`.
- `[x]` `crates/arcade-race-core/src/track/mod.rs` -> `generate_network_walls`.
- `[x]` `crates/race-kit/src/ai/mod.rs` -> `layout_of_branch_under`, the off-route branch switch in `compute_controls`.
- `[x]` `crates/tdrace-app/tests/rally_tracks_tests.rs` -> `speed_limited_lap_time`, `test_rx_joker_costs_lap_time`, the Höljes time-delta test, the 6 m joker wall reach.
- `[x]` `crates/tdrace-app/tests/ai_tests.rs` -> The bot compliance test allows the off-route switch.
- `[x]` `tracks/rally/{holjes_rx,hell_rx,loheac_rx,estering_rx,montalegre_rx,catalunya_rx,essay_rx,dreux_rx,lavare_rx,lessay_rx,killarney_rx,mettet_rx,riga_rx,lydden_hill,kouvola_rx}.json` -> Rebaked.
- `[x]` `docs/circuits/rally.md` -> Which jokers come from OSM.

### Known Gaps (not fixed here)
- `spa_rx` bots fail 4 of 6 runs, also on the main route (`tdrace-xg1x`); its joker is unchanged.
- The Mettet and Riga bases are not reproducible from the importer alone: Mettet keeps 4 hand widths of the old file, and Riga's wall and curb switches come from the remediation pass above.
- Riga's one jump ramp (about (−30, 170)) is outside the swapped section and stays on the main road. Neither parallel road has a ramp, although the official ones each have a jump; the southern road has a crest in OSM (`incline` up then down).
- OSM gives no banking, and the importer has no surface for joker ways without a `surface` tag (`catalunya_rx`, `estering_rx`, `mettet_rx`: asphalt).
