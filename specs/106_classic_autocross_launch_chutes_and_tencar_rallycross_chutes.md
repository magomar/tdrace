---
type: Feature Spec
template: feature
title: "Classic Autocross launch chutes and ten-car Rallycross chutes"
description: "Gives the 3 Classic Autocross circuits the Autocross launch chute (8 cars), makes every Rallycross chute hold 10 cars with a 3-2-3-2 grid, and moves the Classic grandstands that stood inside the wall line, on Meadow Sprint partly on the road."
status: draft
created: 2026-10-09
generated: { by: agent/claude-opus-5.5, at: 2026-10-09T18:34:29Z }
depends_on:
  - "103"
amends:
  - "103"
---


# Feature Spec: Classic Autocross launch chutes and ten-car Rallycross chutes 🌟

[Spec 103](103_autocross_and_rallycross_launch_chutes_and_templated_track_components.md) put launch chutes on the 17 Autocross circuits and the 23 Rallycross circuits. It left out the 3 Classic Autocross circuits (`ax_clay_bowl`, `ax_hillside_hammer`, `ax_meadow_sprint`), which still start on a two-by-two grid on the main straight. Its Rallycross grid, 3-2-3, holds 8 cars. Mario's rule (2026-10-09): every Autocross circuit has a chute with room for at least 8 cars, and every Rallycross circuit a chute with room for 10. This spec amends spec 103, which is `implemented` and cannot change in place. It also fixes a collision bug found on the way: on Meadow Sprint a grandstand stood partly on the road. Beads: `tdrace-w4167` (chutes), `tdrace-t5ir2` (grandstands).

This spec is written after the change. The code and circuits are on branch `claude/meadow-sprint-building-collision-79ba98` (tdrace `84710e02`, tdrace-tracks `c3267d3`). They do not land on `main` until this spec is approved.

---

## 🗺️ User Flow & Interface Design

- **Races**: a race on a Classic Autocross circuit starts on a concrete chute with the 5-3 grid, as on the other 17 Autocross circuits. The grid size is the chute's slot count, so these races now start 8 cars (before: 10). A Rallycross race starts 10 cars (before: 8).
- **Track Studio**: the chute inspector's grid choices are `AX 5-3`, `RX 3-2-3-2` and `4 x 3`.
- **Meadow Sprint**: no grandstand stands on the road or inside the walls.

---

## ⚙️ Backend Models & API Endpoints

### 1. Packed grid pattern (`crates/arcade-race-core/src/track/presets.rs`)
`PackedGridPattern::RallycrossThreeTwoThree` (rows 3, 2, 3) is replaced by:
```rust
/// 3 cars on Row 1, 2 on Row 2, 3 on Row 3, 2 on Row 4 (FIA Rallycross 3-2-3-2 staggered, 10 cars).
RallycrossThreeTwoThreeTwo,
```
`row_counts()` returns `[3, 2, 3, 2]`. With the chute's 6.5 m row spacing and 4 m rear margin, the front row stands 23.5 m from the rear wall, inside the default 40 m pad.

### 2. Rollout tool (`crates/tdrace-app/src/bin/build_launch_chutes.rs`)
`--all` also takes the 3 Classic Autocross circuits. The Rallycross template uses `RallycrossThreeTwoThreeTwo`.

The 23 Rallycross chutes are stamped again from the circuit files before spec 103's rollout (tdrace-tracks `c595832^`), so that each chute keeps its merge point and pad. Removing a chute and placing it again is not a fixed point: it moved 9 of 23 chutes by 15-165 m, so it is not used.

### 3. Circuit data (tdrace-tracks)
- `classic/ax_clay_bowl.json`, `classic/ax_hillside_hammer.json`, `classic/ax_meadow_sprint.json`: a network with the chute, 8 grid slots.
- The 20 `rally/*.json` and 3 `classic/rx_*.json` files: only `network.launch_chute.grid_slots` and `grid_positions` change (8 to 10 slots).
- Grandstands moved out to 1 m past the wall line (centre text only): Meadow Sprint grandstand 1 (was 0.3 m on the road, 3.8 m inside the wall line) and 2 (1.2 m inside), Kart Pine Grove grandstand 2 (0.1 m inside), Quarry Sprint grandstand 2 (0.7 m inside).

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

No access rules change. Determinism: grid slots are fixed data in the circuit files; replays of races on changed circuits differ from replays recorded before this spec.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- `cargo test --workspace --exclude tdrace-py --no-fail-fast`
- `cargo test -p tdrace-core --test ax_rx_circuit_launch_chute_validation`
- `cargo test --release -p tdrace-app --test launch_chute_race_tests -- --include-ignored` (bot race on all 43 circuits)

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Ten-car Rallycross grid**
  - [ ] **Given** a 16 m launch pad
  - [ ] **When** `generate_packed_launch_grid` is called with `RallycrossThreeTwoThreeTwo`
  - [ ] **Then** 10 spawn poses are generated in rows of 3, 2, 3 and 2
  - [ ] **And** every pose stays inside the pad, and no car of a two-car row stands directly behind a car of the row in front

- **Scenario: Every Autocross and Rallycross circuit has a chute of the right size**
  - [ ] **Given** the 20 official Autocross circuits (17 Autocross module + 3 Classic) and the 23 official Rallycross circuits
  - [ ] **When** `validate_track()` runs on each
  - [ ] **Then** all 43 have a valid `LaunchChuteConfig` with no validation errors
  - [ ] **And** each Autocross chute holds 8 slots (`AutocrossFiveThree`) and each Rallycross chute 10 (`RallycrossThreeTwoThreeTwo`), equal to the circuit's grid

- **Scenario: Rallycross chutes keep their place**
  - [ ] **Given** the 23 Rallycross circuits before and after this spec
  - [ ] **When** the chute's merge point and rear pad point are compared
  - [ ] **Then** none moved by more than 0.05 m, and no data outside the chute grid changed

- **Scenario: Bots launch from every chute**
  - [ ] **Given** each of the 43 circuits with a full chute grid of its own category's car (8 on Autocross, 10 on Rallycross)
  - [ ] **When** the bots race 3 laps
  - [ ] **Then** every bot starts on the pad and leaves the chute, none returns to it after lap 1, and at most 2 bots miss lap 1

- **Scenario: No grandstand inside the walls**
  - [ ] **Given** the 18 Classic circuits
  - [ ] **When** each grandstand and building footprint is compared with the road and its wall line
  - [ ] **Then** none stands on the road or inside the wall line

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[x]` `crates/arcade-race-core/src/track/presets.rs` -> `PackedGridPattern::RallycrossThreeTwoThreeTwo`.
- `[x]` `crates/tdrace-app/src/bin/build_launch_chutes.rs` -> Classic Autocross in `--all`, 10-car Rallycross template.
- `[x]` `crates/tdrace-app/src/editor/inspector.rs` -> `RX 3-2-3-2` label.
- `[x]` `crates/tdrace-core/tests/ax_rx_circuit_launch_chute_validation.rs` -> 43 circuits, 8/10 slots.
- `[x]` `crates/tdrace-app/tests/launch_chute_race_tests.rs` -> 43-circuit sweep, Classic Autocross cars, 10 bots on Rallycross.
- `[x]` `crates/arcade-race-core/tests/launch_chute_tests.rs`, `crates/tdrace-app/tests/track_editor_launch_chute_tests.rs`, `crates/tdrace-app/tests/grid_positions_sync_tests.rs`, `crates/tdrace-app/tests/championship_grid_slot_tests.rs`, `crates/tdrace-app/tests/controls_ui_tests.rs` -> grid counts.
- `[x]` `tracks/classic/*.json`, `tracks/rally/*.json` -> chutes and grandstands (tdrace-tracks `c3267d3`).
