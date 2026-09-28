---
type: Feature Spec
template: feature
title: "LAN Lobby Reuses Circuit Selector and Garage"
description: "The LAN host picks the circuit in the full-screen circuit selector and every LAN player picks the car in the Garage, instead of cycling hard-coded lists inside the lobby; the lobby keeps the network alive while those screens are open."
status: in_progress
verified: { by: "human:mario", at: "2026-09-28T07:05:00Z" }
created: 2026-09-28
generated: { by: agent/claude-opus-5-5, at: 2026-09-28T07:00:00Z }
---


# Feature Spec: LAN Lobby Reuses Circuit Selector and Garage 🌐

Quick Race, Career and Split Screen use two shared screens: the full-screen circuit selector (`GameState::Menu`) and the Garage (`GameState::Garage`). The LAN lobbies (spec 036) do not. They cycle short, hard-coded lists inside the lobby screen. This spec makes the LAN lobbies open the same two screens, and return to the lobby with the choice applied.

---

## 🔍 Review of the Current LAN Interface

| # | Defect | Where |
|---|---|---|
| L-1 | The host cycles circuits with ENTER on one row. The list is `TrackChoice::ALL` for **all** modules, so a GT host can pick a kart track. The row shows the raw slug (e.g. `spa_francorchamps`), not the title. There is no preview. | `game/mod.rs` `select_lan_hub_option`, `host_screen.rs` `cycle_track` |
| L-2 | The first host track list is five made-up names ("Circuit de Spa-Francorchamps", …) that are not track slugs. `with_tracks` replaces them, but the defaults are dead data. | `host_screen.rs::new` |
| L-3 | The host can not change the own car or livery in the lobby. The host car is the car that was selected before the lobby opened. Spec 036 wireframe 2 shows "[C] Change My Car / Livery", but `C` copies the IP. | `host_screen.rs` |
| L-4 | The client cycles six hard-coded cars (5 GT + 1 kart) and nine hard-coded liveries. The list ignores the host discipline, career unlocks and the catalog. | `client_lobby_screen.rs::new` |
| L-5 | Slot rows show raw ids (`gt_ferrari_296_gt3: corsa_red`). The client header shows the track slug. | `host_screen.rs::draw`, `client_lobby_screen.rs::draw` |
| L-6 | When the host changes the circuit, `LanHost.discipline` stays at the bind value, so the beacon can advertise the wrong discipline. | `host.rs::set_track_and_rules` |
| L-7 | The host can pick a custom circuit (via `track_choice` at bind). A client does not have that file, so `launch_lan_race_session` falls back to `ClassicGrandPrix` on the client only. The two machines race on different tracks. | `game/mod.rs::launch_lan_race_session` |

Related work: spec 044 (`feat/robust-lan-race-sync`) replaces the in-race netcode and also edits `host_screen.rs`, `client_lobby_screen.rs` and the LAN launch path in `game/mod.rs`. This spec changes only the lobby rows, the sub-screen flow and the keep-alive pump, not the launch or race packets. The branch merged second must resolve the overlap in those files.

Out of scope (tracked separately): the LAN hub card screen, the join browser, and the failing test `test_lan_livery_synchronization_and_countdown_handshake` (`tdrace-lan-countdown-is-in-race-qj0h`).

---

## 🗺️ User Flow & Interface Design

### 1. Flow

```mermaid
stateDiagram-v2
    LanHostLobby --> Menu: CIRCUIT row [ENTER]
    Menu --> LanHostLobby: [ENTER] on circuit (sets track)
    Menu --> LanHostLobby: [ESC] (no change)
    LanHostLobby --> Garage: MY CAR row [ENTER]
    LanClientLobby --> Garage: MY CAR row [ENTER]
    Garage --> LanHostLobby: [ENTER] select (host)
    Garage --> LanClientLobby: [ENTER] select (client)
    Garage --> LanHostLobby: [ESC] (host, no change)
    Garage --> LanClientLobby: [ESC] (client, no change)
```

- New `MenuOrigin::LanHostLobby`. In this origin the circuit selector:
  - shows only official circuits of the host module (the Custom tab is not available, see L-7);
  - on confirm, sets the host track and returns to `GameState::LanHostLobby`. It does **not** call `init_race`;
  - on ESC, returns to `GameState::LanHostLobby` with no change;
  - disables the shortcuts that leave the LAN flow (Garage `G`, Track Manager `T`, championship `F`, Profile `P`, Controls `K`, Settings `X`/`O`). The circuit viewer (`V`) stays available and returns to the selector.
  - The confirm hint reads `SET AS LAN CIRCUIT` instead of `TO RACE`.
- New `GarageOrigin::LanLobby`. In this origin the Garage:
  - opens on the lobby module and on the tier of the current car;
  - on select, sets the player car and returns to the lobby that opened it;
  - on ESC, returns to that lobby with no change;
  - disables module switching (keys `1..5` and the fleet gallery module tabs), so the car is always from the host discipline;
  - keeps the normal career unlock and buy rules.

### 2. Lobby rows

Host lobby (right panel), top to bottom:

| Row | Action |
|---|---|
| CIRCUIT: *title* | Opens the circuit selector |
| LAPS | Cycles laps (unchanged) |
| COLLISIONS | Cycles mode (unchanged) |
| MY CAR: *catalog name* | Opens the Garage |
| MY LIVERY: *name* | Cycles livery |
| MY STATUS | Toggles host ready (unchanged) |
| START RACE | Unchanged |
| DISBAND ROOM | Unchanged |

Client lobby (right panel): MY CAR (opens the Garage), MY LIVERY (cycles), READY, LEAVE ROOM.

Slot rows on both lobbies show the catalog car name and the livery name. The client header shows the circuit title.

### 3. Network keep-alive

While the host or client is in the circuit selector, the circuit viewer or the Garage, `RaceSession::update` calls a new `pump_network(dt)` on the parked lobby screen every frame. This processes packets, pings and timeouts, but not lobby input. So:
- no peer times out (host timeout 3.5 s, client timeout 4.5 s);
- if the client is disconnected while in the Garage, the game returns to `GameState::LanHub { selected_idx: 1 }`;
- when a client opens the Garage, the client sends `is_ready = false`, so the host can not start the race while the client is choosing.

---

## ⚙️ Backend Models & API Endpoints

No wire protocol change. This spec adds no packet and does not change `PROTOCOL_VERSION`.

### `cabinet::net` API changes

```rust
/// Request from a LAN lobby screen that the game must open one of its own screens.
pub enum LanLobbyRequest { PickCircuit, PickCar }

impl CabinetLanHostScreen {
    pub fn new(host: LanHost) -> Self;                       // no default track list
    pub fn set_track(&mut self, track_id: &str, title: &str); // replaces available_tracks / cycle_track / with_tracks
    pub fn set_local_car(&mut self, car_model_id: &str);      // calls LanHost::update_host_slot
    pub fn take_request(&mut self) -> Option<LanLobbyRequest>;
    pub fn pump_network(&mut self, dt: f32);
    pub fn with_labels(self, car_label: fn(&str) -> String) -> Self;
}

impl CabinetLanClientLobbyScreen {
    pub fn set_local_car(&mut self, car_model_id: &str);      // replaces car_models / cycle_car
    pub fn set_ready(&mut self, is_ready: bool);
    pub fn take_request(&mut self) -> Option<LanLobbyRequest>;
    pub fn pump_network(&mut self, dt: f32);
    pub fn with_labels(self, car_label: fn(&str) -> String, track_label: fn(&str) -> String) -> Self;
}

impl LanHost {
    pub fn set_discipline(&mut self, discipline: &str);       // fixes L-6
}
```

Livery lists stay in `cabinet`, shared by both screens (one const table instead of the client-only list).

### `tdrace-app` changes
- `MenuOrigin::LanHostLobby`, `GarageOrigin::LanLobby`.
- `RaceSession::lan_lobby_module()` returns the lobby module: `active_module_id` on the host; the module of the loaded host track on the client.
- On host open: if `track_choice` is not an official circuit of the active module, the lobby starts on the first official circuit of that module.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

- Only the host can open the circuit selector (the client lobby has no CIRCUIT row).
- The host remains the only authority for track, laps and collisions. No new packets, so the spec 036 packet sanitization is unchanged.
- The car id sent by a client is still canonicalized by `canonicalize_car_model_id` at race launch.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- `cargo test -p cabinet --test net_tests`
- `cargo test -p tdrace-app --test lan_integration_tests`
- `cargo test --workspace --exclude tdrace-py` (known unrelated failures are listed in Beads)
- Screenshots with `tdrace-app --gt --lan-host --screenshot <png>` for the host lobby and for the selector in LAN origin.

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Host picks the circuit in the full-screen selector**
  - [ ] **Given** a host lobby in the GT module
  - [ ] **When** the host presses ENTER on the CIRCUIT row
  - [ ] **Then** the full-screen circuit selector opens with only official GT circuits
  - [ ] **When** the host presses ENTER on a circuit
  - [ ] **Then** the game returns to the host lobby, the CIRCUIT row shows that circuit title, and `LanHost::track_id()` is its slug

- **Scenario: ESC from the selector keeps the circuit**
  - [ ] **Given** the circuit selector opened from the host lobby
  - [ ] **When** the host presses ESC
  - [ ] **Then** the game returns to the host lobby and the track is unchanged

- **Scenario: Host picks the own car in the Garage**
  - [ ] **Given** a host lobby
  - [ ] **When** the host presses ENTER on MY CAR and selects an unlocked car in the Garage
  - [ ] **Then** the game returns to the host lobby and slot 1 shows that car name
  - [ ] **And** the client lobby shows the new car in slot 1

- **Scenario: Client picks the car in the Garage**
  - [ ] **Given** a client in the lobby of a GT host, marked READY
  - [ ] **When** the client presses ENTER on MY CAR
  - [ ] **Then** the Garage opens on GT cars and the client slot becomes not ready on the host
  - [ ] **When** the client selects a car
  - [ ] **Then** the game returns to the client lobby and the host shows the new car name in the client slot

- **Scenario: Garage in LAN origin stays in the host discipline**
  - [ ] **Given** the Garage opened from a LAN lobby of a GT host
  - [ ] **When** the player presses `2` (rally) or opens the fleet gallery and changes module tab
  - [ ] **Then** the module stays GT

- **Scenario: Network stays alive while a sub-screen is open**
  - [ ] **Given** a host and a client in the lobby
  - [ ] **When** the client stays in the Garage for 10 s
  - [ ] **Then** the client is still in slot 2 on the host and still connected

- **Scenario: Disconnect while in the Garage**
  - [ ] **Given** a client in the Garage opened from the client lobby
  - [ ] **When** the host disbands the room
  - [ ] **Then** the client returns to `GameState::LanHub { selected_idx: 1 }`

- **Scenario: Custom circuits are not offered**
  - [ ] **Given** a profile with custom circuits and the selector opened from the host lobby
  - [ ] **When** the host presses LEFT / RIGHT / TAB
  - [ ] **Then** the filter stays on official circuits

- **Scenario: Beacon discipline follows the host module**
  - [ ] **Given** a host
  - [ ] **When** `set_discipline("kart")` is called
  - [ ] **Then** the next beacon advertises `discipline = "kart"`

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[ ]` `crates/cabinet/src/net/ui/host_screen.rs` -> CIRCUIT / MY CAR / MY LIVERY rows, `LanLobbyRequest`, `pump_network`, labels.
- `[ ]` `crates/cabinet/src/net/ui/client_lobby_screen.rs` -> MY CAR / MY LIVERY rows, `LanLobbyRequest`, `pump_network`, labels; hard-coded car list removed.
- `[ ]` `crates/cabinet/src/net/ui/mod.rs`, `crates/cabinet/src/net/mod.rs` -> export `LanLobbyRequest` and the shared livery table.
- `[ ]` `crates/cabinet/src/net/host.rs` -> `set_discipline`.
- `[ ]` `crates/tdrace-app/src/game/mod.rs` -> new origins, open/return paths, keep-alive pump, lobby module.
- `[ ]` `crates/tdrace-app/src/ui/menu.rs` -> confirm hint for the LAN origin.
- `[ ]` `crates/tdrace-app/src/main.rs` -> `--lan-host` flag for screenshots.
- `[ ]` `crates/cabinet/tests/net_tests.rs`, `crates/tdrace-app/tests/lan_integration_tests.rs` -> tests for the scenarios above.

### Verification Assertions
- `grep -n "cycle_track\|cycle_car\|available_tracks\|car_models" crates/cabinet/src/net/ui` returns nothing.
