---
type: Feature Spec
template: feature
title: "Mandatory Rallycross Joker Lap Tracking and Penalty Enforcement"
description: "Makes Rallycross joker laps work end to end in the live race (correct route data, surfaces, lap counting and bot route choice) and enforces one mandatory joker per race with a HUD badge, a final-lap warning and a +30 s results penalty."
status: in_progress
verified: { by: human:mario, at: 2026-10-04T14:50:25Z }
created: 2026-10-03
generated: { by: agent/antigravity, at: 2026-10-03T15:12:25Z }
depends_on:
  - "006"
  - "081"
---

# Feature Spec: Mandatory Rallycross Joker Lap Tracking and Penalty Enforcement ⏱️

In Rallycross, every driver must take the joker route once per race. A driver who does not gets a time penalty. TDRace has the joker geometry (spec 081) and a multi-route lap tracker (spec 006), but the live race does not use them as a rule: nothing tells the player about the joker, nothing penalises a missed joker, and bots switch routes in an unreliable way.

Rallycross ships in the first Steam release (Classic, Karting, Autocross and Rallycross modules). This spec makes the joker a working, enforced game rule in that release.

## Current State (audit, 2026-10-04)

What works today:
- 23 official tracks have a `TrackNetwork` with a `main` and a `joker` layout: `tracks/classic/rx_quarry_sprint.json`, `rx_hilltop_leap.json`, `rx_canyon_flyer.json` and all 20 `tracks/rally/*.json`. `classic_rallycross` is an alias of `rx_quarry_sprint`.
- `Track::from_json` calls `trim_walls_for_network()`, so the main walls open at the split and merge throats.
- The race renderer (`crates/race-ui/src/render/track.rs`) draws the joker ribbon and the junction gores.
- `RaceWorld::step` (`crates/race-kit/src/world.rs`) calls `TrackProgressTracker::update_network` on network tracks. `MultiRouteProgressTracker` (`crates/arcade-race-core/src/track/checkpoint.rs`) counts `joker_laps_completed` at the finish line.

What is wrong or missing:
1. **Route data.** Some checkpoints lie on one route but are listed in the other route's layout (seen on `hell_rx`, `montalegre_rx`, `nyirad_rx` and `spa_rx`). A car on the affected route can miss a checkpoint and lose its lap.
2. **Surfaces.** `Track::sample_surface` checks the main spline's run-off corridor before it checks the network, so joker road inside that corridor can read as run-off.
3. **No end-to-end test.** No test drives a car through `RaceWorld::step` on an official RX track along the joker route.
4. **Bots.** `decide_active_layout` (`crates/race-kit/src/ai/mod.rs`) runs every tick, so a bot can change route inside a branch. `DynamicTrafficAvoidance` has no joker limit. The bot's `current_lap` increments in two places per lap, so `planned_joker_lap` fires on the wrong lap.
5. **No rule.** Nothing reads `joker_laps_completed`. `RaceRules` has no joker setting and `RaceWorld::results` has no penalty.
6. **No UI.** The HUD and results screen show no joker state.
7. **LAN.** `FinishRecord` (`crates/cabinet/src/net/protocol.rs`) carries no penalty.
8. **Menu.** Picking the `joker` layout in the menu makes it the default layout and replaces `track.spline` (`GameState::load_track_for_session`), which breaks the rule.

### Amendment (found during implementation, 2026-10-04)
The end-to-end test (item 3) found more than the audit did. These fixes are part of this spec:
- **Item 1 was misread.** The checkpoints lie on the right route, but on the same 4 tracks the checkpoint at the merge point is tagged `segment_id: 0` and is listed before the joker gate. The cause is the tagging rule in `crates/tdrace-app/src/bin/build_world_rx_joker.rs`.
- **Folded jokers.** That builder made each World RX joker by offsetting the main branch sideways along raw waypoint normals. Where the offset was larger than the bend radius, the joker folded back on itself (turn radius 0.0–0.8 m on `loheac_rx`, `lessay_rx`, `riga_rx` and others), and a car on it read as wrong way. The builder now offsets the smooth main samples and switches to the other side when the configured side turns tighter than 3 m. All 20 World RX files are rebaked from `tracks` commit `77a0abe` (the state before the first joker bake), so no wall gaps from the old jokers remain. The 3 Classic RX jokers are unchanged.
- **Tracker at the split.** `MultiRouteProgressTracker` picked the branch nearest the car at the split and never checked the other branch again, and it followed junction sockets that on all 23 networks make the merge feed both the return straight and the start straight. It now takes the next segment from the layouts, moves to a sibling branch when the car leaves the current one, and measures lap distance along a layout that contains the current segment.
- **Surface sampling** also let one segment's curb win over another segment's road near a junction (`TrackNetwork::sample_surface`).
- **Narrow joker gate.** The joker gate spans only the road width (13 m; other gates are about 20 m), so a bot or a player running wide drove round its end and lost the joker. The tracker now credits the joker when a car on the joker branch drives past the gate's point on that branch.
- **Bots stuck on Classic RX and Autocross (not fixed here).** A one-bot sweep found bots stuck for 44–138 s per lap on `rx_quarry_sprint`, `rx_hilltop_leap`, `rx_canyon_flyer` and `ax_clay_bowl`, also on the main route, and unable to finish the joker on `kouvola_rx`, `lavare_rx` and `killarney_rx`. This predates this spec and is filed as separate bugs. The bot compliance scenario uses `holjes_rx` and `hell_rx`, where a bot drives both routes cleanly.

---

## 🗺️ User Flow & Interface Design

### 1. In-race HUD badge (own car)
In a Rallycross race only, a pill is drawn next to the position/lap widget (`render_position_and_lap`, `crates/race-ui/src/hud/widgets.rs`), and next to its split-screen copy:
- **Pending**: amber `JOKER` pill.
- **Done**: green `JOKER DONE` pill (plain ASCII: emoji glyphs render as boxes in the UI fonts, see `tdrace-lm6l`).
- **Final lap, still pending**: red `JOKER THIS LAP!` pill that pulses (alpha 0.5 → 1.0 at 2 Hz).

No badge is drawn in non-Rallycross races.

### 2. Results screen
In `render_results_screen` (`crates/tdrace-app/src/ui/menu.rs`), each row of a Rallycross race shows:
- A green `J` when the driver took the joker.
- An amber `+30.0s NO JOKER` tag when a penalty applies. The time column shows the penalised time, and the rows are in penalised order.

Championship points and credit payouts use this penalised order.

### 3. Layout picker
For a Rallycross race, the menu's layout choice does not change the race's default layout. The race always starts on `main`, and the driver picks the joker route by driving it.

---

## ⚙️ Backend Models & API Endpoints

### 1. Race rule (`crates/race-kit/src/world.rs`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JokerRule {
    /// Joker laps each vehicle must complete. 0 turns the rule off.
    pub mandatory: u32,
    /// Seconds added to the finish time of a vehicle with fewer jokers than `mandatory`.
    pub penalty_s: f32,
}
// Default: { mandatory: 0, penalty_s: 0.0 }

pub struct RaceRules { pub format: RaceFormat, pub collision: CollisionParams, pub joker: JokerRule }
```
- `RaceRules::default()` keeps `joker` off, so existing callers (`..RaceRules::default()`, including `tdchariots`) do not change.
- `RaceWorld` gets `pub fn jokers_taken(&self, car: usize) -> u32`, which reads `trackers[car].multi_route.joker_laps_completed` (0 when there is no network).
- `RaceWorld` gets `penalty: Vec<f32>`. When a vehicle finishes with `jokers_taken < joker.mandatory`, its entry is set to `joker.penalty_s`.
- `standings()` orders finished vehicles by `time + penalty`, with crossing position as the tie-break. Without penalties, this order is the same as today.
- `ParticipantResult` gets `pub penalty: f32` and `pub jokers: u32`. `time` stays the raw time. In `results()`, finished and projected rows are ordered by `time + penalty`.

### 2. Who sets the rule (`crates/tdrace-app/src/game/mod.rs`)
When a race starts on a track with `track.car_category == CarCategory::Rally` and a `network` that has a `joker` layout, the app sets `RaceRules.joker = JokerRule { mandatory: 1, penalty_s: 30.0 }`. Every other race keeps the default (off).

### 3. Route data and surfaces
- Fix the layout checkpoint lists in the affected `tracks/` JSON files, so every checkpoint in a layout lies on a segment of that layout. The fix goes into the `tracks` submodule, which is pushed before `main` (see the project memory on tracks).
- `Track::sample_surface` and `sample_surface_near` (`crates/arcade-race-core/src/track/mod.rs`) check network segment road before the main spline's run-off corridor.

### 4. Bot route choice (`crates/race-kit/src/ai/mod.rs`)
- A bot chooses its route once per lap, before the split junction, and keeps it until it passes the merge.
- When the bot runs inside a `RaceWorld`, it takes its lap number and joker count from the race tracker. Remove the two internal `current_lap += 1` increments for that case. The standalone path (no tracker) keeps its own counting.
- A bot never takes more jokers than `mandatory`.
- On the last lap (and on lap N−1 if the planned lap has passed), a bot that still owes a joker always takes it.
- The existing strategies (`RallycrossJoker { lap }`, `DynamicTrafficAvoidance` undercut) only choose which lap the joker happens on.

### 5. LAN (`crates/cabinet/src/net/protocol.rs`, `crates/tdrace-app/src/game/lan.rs`)
- `FinishRecord` gets `penalty_ms: u32`. The car's owner computes its own penalty and reports it with `finish_ms`. The host and the clients sort LAN results by `finish_ms + penalty_ms`.
- `PROTOCOL_VERSION` goes from 2 to 3.

### Out of scope (follow-up Beads issues)
- Walls along the joker branch (`RoadSegment.walls` is empty on all tracks).
- A live timing tower with a per-driver joker column (`render_timing_tower_row` exists but nothing calls it).
- Spotter or voice callouts (no spotter audio system exists).
- Trackside joker signage.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

### Invariants
1. **Only the tracker credits a joker.** `joker_laps_completed` only goes up when `MultiRouteProgressTracker` sees the joker layout's checkpoints in order and the car crosses the finish line. No UI or debug path changes it.
2. **Deterministic.** The penalty depends only on tracker state at the finish step. Given the same inputs, two runs give the same results. The race-kit golden state hashes do not change for races with the rule off.
3. **No change for other categories.** Races that are not Rallycross keep `JokerRule` off and show no joker UI.
4. **LAN trust model is unchanged.** Each owner already reports its own finish (spec 044). The penalty follows the same model.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- `cargo test -p race-kit --test joker_rule_tests`: rule, penalty, standings and results ordering on a synthetic network.
- `cargo test -p tdrace-app --test rally_tracks_tests`: layout data check and end-to-end drive on all 23 RX tracks.
- `cargo test -p tdrace-app --test ai_tests`: bot route lock, joker cap and 8-bot compliance.
- `cargo test -p cabinet`: `FinishRecord` round trip with `penalty_ms`.
- Full suite and lint as listed in `specs/constitution/TECH_STACK.md`.

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Every RX layout's checkpoints lie on its own route**
  - [ ] **Given** the 23 official tracks that have a joker layout
  - [ ] **When** each checkpoint of each layout is checked against the segments of that layout
  - [ ] **Then** every checkpoint centre is within the road width of a segment in that layout

- **Scenario: No joker folds back on itself**
  - [ ] **Given** the joker-only segment of each of the 23 RX tracks
  - [ ] **When** the turn radius between consecutive samples is measured, skipping points within 2 m of the main branch
  - [ ] **Then** no turn is tighter than 3 m

- **Scenario: A car that drives the joker route gets its lap and its joker**
  - [ ] **Given** each of the 23 RX tracks in a `RaceWorld` with `RaceFormat::Laps(2)`
  - [ ] **When** a car is moved along the `joker` centerline on lap 1 and along the `main` centerline on lap 2
  - [ ] **Then** the car finishes with `current_lap == 3` and no wrong-way state
  - [ ] **And** `jokers_taken` is 1

- **Scenario: Joker road reads as road, not run-off**
  - [ ] **Given** each RX track's joker segment
  - [ ] **When** the surface is sampled at points along the joker centerline
  - [ ] **Then** every sample returns the joker segment's road surface

- **Scenario: A driver who took the joker gets no penalty**
  - [ ] **Given** a 5-lap Rallycross race with `JokerRule { mandatory: 1, penalty_s: 30.0 }`
  - [ ] **When** a car takes the joker on lap 3 and finishes
  - [ ] **Then** its result row has `penalty == 0.0` and `jokers == 1`
  - [ ] **And** the in-race HUD showed the green `JOKER DONE` pill after lap 3

- **Scenario: A missed joker adds 30 s and drops the driver down the order**
  - [ ] **Given** the same race, where car A crosses the line first without a joker and car B crosses 5 s later with a joker
  - [ ] **When** the results are built
  - [ ] **Then** car A has `penalty == 30.0` and is classified behind car B
  - [ ] **And** the results screen shows `+30.0s NO JOKER` on car A's row

- **Scenario: Final-lap warning**
  - [ ] **Given** a driver that starts the last lap without a joker
  - [ ] **When** the driver crosses the line into the last lap
  - [ ] **Then** the HUD shows the pulsing red `JOKER THIS LAP!` pill (checked by screenshot)

- **Scenario: Bots take exactly one joker and never switch inside a branch**
  - [ ] **Given** an 8-bot, 5-lap Rallycross race on `holjes_rx` and on `hell_rx`, bots of all tiers
  - [ ] **When** the race runs to the end
  - [ ] **Then** every bot that finishes has `jokers == 1` and no penalty
  - [ ] **And** no bot changes its active layout between the split and the merge

- **Scenario: Other categories are not affected**
  - [ ] **Given** a race on a GT, Karting or Autocross track
  - [ ] **When** the race starts and ends
  - [ ] **Then** `RaceRules.joker.mandatory == 0`, no joker HUD pill is drawn, and no penalty is applied
  - [ ] **And** the race-kit golden state hash tests pass unchanged

- **Scenario: RX race ignores the joker layout pick**
  - [ ] **Given** the player picked the `joker` layout in the menu for an RX track
  - [ ] **When** the race loads
  - [ ] **Then** the race's default layout is `main`

- **Scenario: LAN results carry the penalty**
  - [ ] **Given** a `FinishRecord` with `finish_ms = 120000` and `penalty_ms = 30000`
  - [ ] **When** it is encoded and decoded with protocol version 3
  - [ ] **Then** both fields round-trip, and LAN results order uses 150000 ms for that car

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `crates/race-kit/src/world.rs` -> `JokerRule`, `jokers_taken`, penalty in `standings()` and `results()`.
- `crates/race-kit/tests/joker_rule_tests.rs` -> rule and ordering tests (new).
- `crates/race-kit/src/ai/mod.rs` -> once-per-lap route lock, joker cap, lap count from tracker.
- `crates/arcade-race-core/src/track/mod.rs` -> network road checked before main run-off in surface sampling.
- `crates/arcade-race-core/src/track/network.rs` -> segment road wins over another segment's curb.
- `crates/arcade-race-core/src/track/checkpoint.rs` -> layout-based segment transitions, sibling branch recovery, lap distance along the segment's layout.
- `crates/tdrace-app/src/bin/build_world_rx_joker.rs` -> smooth offset, fold check with side switch, merge checkpoint tagging.
- `tracks/rally/*.json` (submodule) -> 20 World RX jokers rebaked from `77a0abe`.
- `crates/tdrace-app/src/game/mod.rs` -> set `JokerRule` for Rallycross races; keep `main` as the RX default layout.
- `crates/race-ui/src/hud/widgets.rs`, `crates/tdrace-app/src/ui/hud.rs` -> joker pill.
- `crates/tdrace-app/src/ui/menu.rs` -> results screen joker mark and penalty tag.
- `crates/cabinet/src/net/protocol.rs`, `crates/tdrace-app/src/game/lan.rs` -> `penalty_ms`, protocol version 3.
- `crates/tdrace-app/tests/rally_tracks_tests.rs`, `crates/tdrace-app/tests/ai_tests.rs` -> data, end-to-end and bot tests.

### Beads Epic Mapping
- Governed by Beads Epic: `tdrace-eo64` ("Fulfill Spec 082: Mandatory Rallycross Joker Lap Tracking and Penalty Enforcement").
