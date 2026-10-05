---
type: Feature Spec
template: feature
title: "Rallycross Jokers Built From OpenStreetMap Joker Ways"
description: "Builds the World RX joker branch from the joker ways mapped in OpenStreetMap on the 10 circuits that have them, keeps the synthetic joker elsewhere, replaces the 30-70 m joker length rule with a lap-time rule, keeps joker walls out of the main road's wall gap, and lets a bot stuck on the other branch follow it."
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

[Spec 081](081_rallycross_joker_lap_segments_for_classic_and_openstreetmap_circuits.md) says the World RX joker paths are "extracted directly from OpenStreetMap". They are not. `crates/tdrace-app/src/bin/build_world_rx_joker.rs` builds all 20 jokers the same way: the main line between two hand-picked waypoints, bulged sideways until it is exactly 42 m longer. Meanwhile `scripts/osm_importer.py` drops the joker ways it finds in OSM on at least 5 circuits. This spec builds the joker from OSM wherever OSM maps it, so those 10 circuits race like the real venue. It amends spec 081 and one bot rule of [spec 082](082_mandatory_rallycross_joker_lap_tracking_and_penalty_enforcement.md); both are `implemented` and cannot change in place. Beads: `tdrace-3z9u`. Decisions (Mario, 2026-10-04): use all 10 mapped jokers with their real length and judge a joker by the time it costs, not by its length; fix the bot that gets stuck on the real Höljes joker in this change.

---

## 🗺️ User Flow & Interface Design

No new screens. On the 10 circuits below, the joker branch a player or bot drives is the joker mapped in OSM: it leaves and rejoins the main road where the real one does, with its real length and the surface its OSM ways carry. The main route is unchanged.

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

The other 10 World RX circuits (`lydden_hill`, `nyirad_rx`, `kouvola_rx`, `mettet_rx`, `riga_rx`, `killarney_rx`, `croft_rx`, `spa_rx`, `silverstone_rx`, `erx_motor_park`) have no joker in OSM and keep the synthetic one. `spa_rx` and `silverstone_rx` are not built by the OSM importer at all.

---

## ⚙️ Backend Models & API Endpoints

### 1. OSM importer (`scripts/osm_importer.py`)
- Each mapped circuit's `RALLY_TRACKS` config lists its joker ways in race order as `joker_ways`.
- `rally_joker_path` chains them from the node where they leave the lap to the node where they rejoin it. It fails when a way does not join the previous one, when the ends are not on the lap, or when the joker runs against the lap direction.
- The joker points go through the same transform as the lap: metres around the lap centre, the rounding pre-scale, rotation onto +X, FIA scale and start offset. They are resampled about every 10 m, each with the surface of its OSM way.
- `python3 scripts/osm_importer.py rally --jokers` writes them to `assets/osm/rx_jokers.json`, keyed by catalog id. The file is outside `tracks/` because `crates/tdrace-core/build.rs` takes every JSON file there for a circuit.

```json
{ "holjes_rx": { "osm_ways": [599300792], "points": [[150.9, -4.0]], "surfaces": ["Asphalt"] } }
```

### 2. Joker builder (`crates/tdrace-app/src/bin/build_world_rx_joker.rs`)
- `cargo run --bin build_world_rx_joker -- [tracks/rally] [assets/osm/rx_jokers.json]`.
- Each config has a `JokerSource`: `Osm`, or `Synthetic { split_idx, merge_idx, side, surface, bank_angle }` (the old bulge, unchanged).
- An OSM joker's end nodes must lie within 6.5 m of the main line (the spline cuts the OSM corners: 4.7 m on `estering_rx`).
- The mapped split and merge are where the OSM joker leaves and rejoins the main road, not its end nodes. The `holjes_rx` joker ends in a lane that runs 3 m beside the main line for about 70 m, and a car on the main road there read as on the joker.
- The joker splits at the last main waypoint before the mapped split and merges at the first main waypoint after the mapped merge. So the main route keeps its waypoints and its exact shape. An extra waypoint at the mapped split moved the main line enough to put `dreux_rx` bots on a hairpin wall.
- Between those waypoints the joker follows the main line up to 10 m before the mapped split, then the OSM points, then the main line from 10 m after the mapped merge. Leaving out the fork points lets the spline round the fork. Through them, the joker kinked, `holjes_rx` walls crossed, and `catalunya_rx` cost 2 s more.
- Only a synthetic joker must be 30–70 m longer than its main branch.

### 3. Joker walls (`Track::generate_network_walls`, `crates/arcade-race-core/src/track/mod.rs`)
Two more kinds of joker wall piece are dropped, on every joker track:
- A piece nearer the main road than the main walls stand (the local main wall gap less 0.5 m). One stood about 1 m off the `essay_rx` main road edge, and bots on the main route hit its end.
- A piece within 0.5 m of a main wall along its length. It only doubles that wall, and once the pieces are joined the line zigzags across it (`holjes_rx` validation: `ERR_WALL_SELF_INTERSECTION`).

### 4. Bot on the other branch (`BotAiDriver::compute_controls`, `crates/race-kit/src/ai/mod.rs`) — amends spec 082
Spec 082 says a bot keeps its route from the split to the merge, and that every finished bot takes exactly one joker. On `holjes_rx` the main route turns left at the split and the joker goes straight on. A main-route bot that ran wide there ended on the joker, kept steering for the main road behind the joker's inside wall, and was stuck for about three minutes. It then drove out through the joker and was credited a second joker.
- A bot whose car is more than 2 m outside its own road, inside a branch road that its route does not use, **and** stuck (its no-progress watchdog has fired at least once), switches to the route of that branch until its next route choice at the start of the next lap.
- Without the "stuck" condition, a `killarney_rx` main-route bot that drifts onto the synthetic joker every lap took it every lap.
- A bot that follows the joker this way can finish with more than one joker. That is no penalty; it costs the time of the joker.

### 5. Data
The 10 mapped circuits are rebaked from `tracks` commit `77a0abe`, the state before any joker bake, as in the spec 082 amendment. The other 10 World RX files and the Classic RX files are unchanged.

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

- **Scenario: Mapped jokers follow OSM**
  - [ ] **Given** the 10 circuits whose joker is mapped in OSM
  - [ ] **When** `python3 scripts/osm_importer.py rally --jokers` runs
  - [ ] **Then** `assets/osm/rx_jokers.json` holds one joker per circuit, each starting and ending within 6.5 m of the main line
- **Scenario: Unmapped circuits keep the synthetic joker**
  - [ ] **Given** the 20 World RX files from `tracks` commit `77a0abe`
  - [ ] **When** `cargo run --bin build_world_rx_joker` bakes them
  - [ ] **Then** the files of the 10 unmapped circuits are byte-identical to the committed ones (`spa_rx` excepted: its committed file already differs from a fresh bake)
- **Scenario: The main route keeps its shape**
  - [ ] **Given** a mapped circuit
  - [ ] **When** its main layout is built
  - [ ] **Then** it uses exactly the main spline's waypoints
- **Scenario: Every joker costs lap time**
  - [ ] **Given** the 23 official circuits with a joker layout
  - [ ] **When** a car limited by top speed (40 m/s), corner grip (10 m/s²), acceleration (8 m/s²) and braking (10 m/s²) drives the main lap and the joker lap
  - [ ] **Then** the joker lap is between 1.0 and 6.5 seconds slower (`test_rx_joker_costs_lap_time`), and on `holjes_rx` between 2.0 and 4.5 seconds slower
- **Scenario: Joker walls stay out of the main road's wall gap**
  - [ ] **Given** the 23 joker circuits
  - [ ] **When** their joker walls are built on load
  - [ ] **Then** no wall crosses another (`test_rally_module_tracks_integrity_and_validation`), and the joker branch keeps its walls (`test_rx_joker_branch_has_walls_that_stay_off_every_road`)
- **Scenario: Bots take one joker unless stuck on the joker (amends spec 082)**
  - [ ] **Given** an 8-bot, 5-lap Rallycross race on `holjes_rx` and on `hell_rx`, bots of all tiers
  - [ ] **When** the race runs (`test_bot_ai_strategic_joker_rx_race_compliance`)
  - [ ] **Then** a bot changes its route between the split and the merge only while its car is off the road of the route it leaves
  - [ ] **And** every finished bot has no penalty, and `jokers == 1` unless it moved onto the joker that way
- **Scenario: Bots drive the mapped jokers**
  - [ ] **Given** each of the 10 mapped circuits and a Pro bot of the Balanced, Smooth and Aggressive styles
  - [ ] **When** each bot drives 3 laps on the main route and 3 laps on the joker route in the bot harness
  - [ ] **Then** every bot finishes (on 2026-10-05 the longest stop was 5.2 s, on the `catalunya_rx` joker)

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[x]` `scripts/osm_importer.py` -> `joker_ways`, `rally_joker_path`, `--jokers`, `write_rx_jokers`.
- `[x]` `assets/osm/rx_jokers.json` -> The 10 mapped jokers in track coordinates.
- `[x]` `crates/tdrace-app/src/bin/build_world_rx_joker.rs` -> `JokerSource`, `osm_cut`, `synthetic_cut`.
- `[x]` `crates/arcade-race-core/src/track/mod.rs` -> `generate_network_walls`.
- `[x]` `crates/race-kit/src/ai/mod.rs` -> `layout_of_branch_under`, the off-route branch switch in `compute_controls`.
- `[x]` `crates/tdrace-app/tests/rally_tracks_tests.rs` -> `speed_limited_lap_time`, `test_rx_joker_costs_lap_time`, the Höljes time-delta test.
- `[x]` `crates/tdrace-app/tests/ai_tests.rs` -> The bot compliance test allows the off-route switch.
- `[x]` `tracks/rally/{holjes_rx,hell_rx,loheac_rx,estering_rx,montalegre_rx,catalunya_rx,essay_rx,dreux_rx,lavare_rx,lessay_rx}.json` -> Rebaked.
- `[x]` `docs/circuits/rally.md` -> Which jokers are real.

### Known Gaps (not fixed here)
- Bots on synthetic-joker circuits still fail to finish some runs: `lydden_hill`, `kouvola_rx`, `riga_rx` and `spa_rx` (`spa_rx` also on the main route). This predates this spec (`tdrace-xg1x`). With the wall and bot changes above, 9 of 60 such runs fail, as before the bot change; before the wall change it was 11.
- `killarney_rx` has a short main wall piece between the main road and the joker in its hairpin (about (−281, −81) to (−278, −86)). A main-route bot gets stuck on it now and then and follows the joker out.
- OSM gives no banking, and the importer has no surface for joker ways without a `surface` tag (`catalunya_rx`, `estering_rx`: asphalt).
