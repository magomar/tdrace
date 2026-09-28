---
type: Feature Spec
template: feature
title: "Rallycross Labels, RX Tier 4-5 Cars, and Mint 400 Desert Circuits"
description: "The Rally module shows Rallycross everywhere on screen, its tiers 4 and 5 hold RX1e and Nitrocross Group E cars, the Rally Raid T1+ and Stadium Super Truck cars move to Extreme Off-Road as unranked tiers 6 and 7, and Extreme Off-Road gets three circuits built from the official Mint 400 GPX course files."
status: implemented
created: 2026-09-28
generated: { by: agent/claude-opus-5-5, at: 2026-09-28T13:58:17Z }
---


# Feature Spec: Rallycross Labels, RX Tier 4-5 Cars, and Mint 400 Desert Circuits 🏁

The Rally module (`"rally"`) is a rallycross module: its 17 circuits are World RX and Euro RX
venues. But some screens called it "RALLY" or "RALLY CROSS", and its tiers 4 and 5 held
desert and stadium cars (Dakar T1+ prototypes and Stadium Super Trucks), not RX cars. This spec:

1. Renames the on-screen labels of the module to **Rallycross**. The module id `"rally"`
   stays the same, because player saves, the LAN beacon, series files and track JSON store it.
2. Replaces rally tiers 4 and 5 with real RX cars.
3. Parks the six desert/stadium cars in Extreme Off-Road, outside its five career tiers.
   Mario will reorganize them later (decided 2026-09-28).
4. Adds three Extreme Off-Road circuits cut from a real desert race, the Mint 400.

Background research (2026-09-28): rally-raid events (Dakar, Hungarian Baja) have no fixed
circuits: the route changes every year and the roadbook is secret. US desert races are the
closest fixed paths. The Mint 400 publishes its course files as GPX/KML/USR at
<https://themint400.com/race-format/>.

---

## 🗺️ User Flow & Interface Design

### 1. Labels

| Where | Before | After |
|---|---|---|
| Garage module title | `RALLYCROSS & ALL-TERRAIN` | `RALLYCROSS` |
| Garage fleet gallery tab | `RALLY` | `RALLYCROSS` |
| Circuit selector module name | `RALLY CROSS` | `RALLYCROSS` |
| Track Manager filter tab, category badge, dossier | `RALLY`, `Rally Cross`, `RALLY CROSS` | `RALLYCROSS`, `Rallycross` |
| Track Manager module card | `Rally Cross Championship` / "Dirt tracks, dunes & rugged mountain stages" | `Rallycross Championship` / "World RX & Euro RX mixed-surface circuits" |
| Profile telemetry filter | `RALLY` | `RALLYCROSS` |
| Circuit editor category / module | `Rally`, `RALLY` | `Rallycross`, `RALLYCROSS` |
| `CarCategory::Rally.title()` | `RALLY` | `RALLYCROSS` |

The module id, the car ids (`rally_*`) and the circuit ids do not change.

### 2. Rally tiers 4 and 5

| Tier | Class | Cars (id) |
|---|---|---|
| 4 | RX1e Electric Supercar (680 bhp, 880 Nm, 1,330 kg) | Peugeot 208 RX1e (`rally_peugeot_208_rx1e`), Volkswagen Polo RX1e (`rally_polo_rx1e`), Lancia Delta Evo-e RX (`rally_lancia_delta_evo_e_rx`) |
| 5 | Nitrocross Group E (1,070 bhp, 1,100 Nm, 1,245 kg) | Olsbergs MSE FC1-X (`rally_omse_fc1x`), Vermont SportsCar FC1-X (`rally_vsc_fc1x`), Dodge Hornet R/T FC1-X (`rally_dodge_hornet_fc1x`) |

- The career series of tiers 4 and 5 become **RX1e Electric Championship**
  (`rally_rx1e_electric_championship`) and **Nitrocross Group E Series** (`rally_nitrocross_group_e`).
  They keep the rounds of the old Dakar Rally Raid Trophy and Stadium Super Trucks World Series.
- The 12 rally drivers' tier 4 and 5 favourite cars point at the new cars.
- The new cars have no own art yet. Their lateral, thumbnail and top-down PNGs are copies of the
  closest RX car (208 Rally4, Polo RX, Delta S4, S1 RX, i20 RX), and the procedural lateral
  renderer uses the same bodies. Engine sounds follow the tier (`SupercarRx1`, `GroupBInline5`),
  as before; the game has no electric engine sound.
- Rally cars are no longer eligible for PackedSand circuits (that rule was for the T1+ cars).

### 3. Unranked Extreme Off-Road tiers 6 and 7

- The three Rally Raid T1+ cars move to `extreme_offroad` tier 6 ("Unranked: Rally Raid T1+"),
  and the three Stadium Super Trucks to tier 7 ("Unranked: Stadium Super Truck"). Their ids stay.
- Their physics stay the same: base `RallyCar`, 17.5 N per bhp, and the terrain flotation of the
  old rally tiers 4 and 5. Deep mud stays closed to them (only off-road tiers 4 and 5).
- The Garage lets the player step to tiers 6 and 7 in Extreme Off-Road
  (`catalog::garage_tier_count`). The header says `UNRANKED`. Career levels stop at 5, so these
  cars are locked outside dev mode, with the message `UNRANKED VEHICLE — DEV MODE ONLY`.
- Lighting now comes from the catalog module first, so these `rally_*` cars get off-road lights.
- Their textures move to `assets/textures/vehicles/*/extreme_offroad/`.

### 4. Mint 400 circuits

| Circuit (id) | Source GPX track | Scale | Lap | Laps |
|---|---|---|---|---|
| Mint 400 Short Course (`mint400_short_course`) | Youth 170/250 course, 2.35 km | 1:1 | 2,142 m | 4 |
| Mint 400 Qualifying Loop (`mint400_qualifying_loop`) | Qualifying course, 9.78 km | 1:1 | 9,511 m | 2 |
| Mint 400 Grand Loop (`mint400_grand_loop`) | Car/Truck/UTV course, 148.2 km | 0.05x | 4,421 m | 2 |

- Surface PackedSand, runoff DeepSand, tire walls. All three open at career tier 1.
- `scripts/gpx_importer.py` builds them from `assets/gpx/*.gpx`. It reuses the OSM importer
  helpers: GPX → local metres → scale → cut spurs → resample → fillet corners → resample.
  It cuts every part of the course that comes back closer than 2.2 track widths to an earlier
  part (the out-and-back spurs and the start spur), so no two walls overlap. That is why the
  laps are a little shorter than the GPX courses.
- The circuits have `wikipedia_url` (Mint 400) but no `osm_url`, because they do not come from
  OpenStreetMap.

---

## ⚙️ Backend Models & API Endpoints

No schema change. The saved module id stays `"rally"`.

- A saved `unlocked_cars` list can still hold the old tier 4/5 rally ids; they now resolve to
  Extreme Off-Road cars and are harmless.
- An old award or active championship with id `rally_dakar_raid_trophy` or
  `rally_super_trucks_series` no longer matches a preset.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

No new access paths. The GPX files are public course files from the event organiser. They are
cached in the repo; the game does not download anything at run time.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- `cargo test --workspace --exclude tdrace-py`
- `python3 scripts/gpx_importer.py` prints each lap length and the smallest gap between two parts
  of the lap (must be at least 2 track widths).

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Rally module is named Rallycross on screen**
  - [x] **Given** the Garage, the circuit selector and the Track Manager
  - [x] **When** the player opens the rally module
  - [x] **Then** every module label reads `RALLYCROSS` or `Rallycross`, and none reads `RALLY CROSS` or `ALL-TERRAIN`

- **Scenario: Rally tiers 4 and 5 hold RX cars**
  - [x] **Given** the rally module in the Garage
  - [x] **When** the player steps to tier 4 and tier 5
  - [x] **Then** tier 4 lists the three RX1e cars and tier 5 lists the three FC1-X cars
  - [x] **And** the tier 4 and 5 career cups are the RX1e Electric Championship and the Nitrocross Group E Series

- **Scenario: Desert cars are parked in Extreme Off-Road**
  - [x] **Given** the Extreme Off-Road module in the Garage
  - [x] **When** the player steps past tier 5
  - [x] **Then** an `UNRANKED` page lists the three Rally Raid T1+ cars, and the next one the three Stadium Super Trucks
  - [x] **And** outside dev mode they show `UNRANKED VEHICLE — DEV MODE ONLY`

- **Scenario: Mint 400 circuits are playable**
  - [x] **Given** a new Extreme Off-Road career at tier 1
  - [x] **When** the player opens the circuit selector
  - [x] **Then** the three Mint 400 circuits are listed and unlocked
  - [x] **And** each one validates (closed loop, walls, 20 checkpoints, 12 grid slots)

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[x]` `crates/tdrace-app/src/catalog/mod.rs` -> RX tier 4-5 cars, unranked tiers 6-7, `garage_tier_count`, physics/surface/sound/tier-name rules.
- `[x]` `crates/tdrace-app/src/game/mod.rs` -> Garage tier stepping, rally career cup names.
- `[x]` `crates/tdrace-app/src/ui/garage.rs`, `ui/menu.rs`, `ui/track_manager_ui.rs`, `ui/profile_ui.rs`, `editor/ui.rs`, `track_manager.rs`, `arcade-race-core/src/car_category.rs` -> labels.
- `[x]` `crates/tdrace-app/src/module/rally.rs`, `ai/driver.rs`, `profile/mod.rs` -> favourite and starter cars, Mint 400 unlocks.
- `[x]` `crates/tdrace-app/src/render/lateral.rs`, `render/lighting.rs` -> bodies and lights for the moved and new cars.
- `[x]` `series/rally/rally_rx1e_electric_championship.toml`, `series/rally/rally_nitrocross_group_e.toml`, `crates/tdrace-app/src/series/manager.rs` -> tier 4-5 series.
- `[x]` `scripts/gpx_importer.py`, `assets/gpx/*.gpx`, `tracks/extreme_offroad/mint400_*.json`, `tracks/.track_order.json` -> Mint 400 circuits.
- `[x]` `scripts/generate_asset_data.py`, `portals/shared/data/{circuits,vehicles}.json` -> portal data.
- `[x]` `docs/circuits/offroad.md`, `docs/circuits/index.md` -> circuit docs.

### Verification Assertions
- `crates/tdrace-app/tests/garage_tests.rs`, `render_tests.rs`: 86 cars; Extreme Off-Road tiers 1..=7.
- `crates/tdrace-app/tests/extreme_offroad_module_tests.rs`, `track_manager_tests.rs`, `crates/tdrace-core/tests/official_catalog_tests.rs`: 20 Extreme Off-Road circuits, 99 in total.
- `crates/tdrace-app/tests/module_track_unlock_tests.rs`: Extreme Off-Road tier 1 opens 8 circuits.
- `crates/tdrace-app/tests/series_tests.rs`, `profile_tests.rs`: new tier 4-5 series and cup names.
