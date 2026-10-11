---
type: Feature Spec
template: feature
title: "Racing Number Placement by Car Type"
description: "Makes the racing number on the car smaller and puts it where each kind of car carries it: no number on karts, on the bonnet of buggies and GT cars, on the middle of the roof of stock cars and rally cars."
status: implemented
receipt: "docs/receipts/spec-111-receipt.md"
verified: { by: human:mario, at: 2026-10-11T18:23:57Z, hash: "fc8920532624" }
created: 2026-10-11
generated: { by: agent/claude-opus-5.5, at: 2026-10-11T17:38:06Z }
depends_on:
  - "092"
amends:
  - "092"
---


# Feature Spec: Racing Number Placement by Car Type 🔢

[Spec 092](092_player_racing_number_customization_and_persistent_nonconflicting_bot_rosters.md) draws the racing number on every car, centred on the cabin or roof, as large as the cabin allows. In play the numbers are too large, and the cabin is the wrong place for most kinds of car. Mario's rules (2026-10-11): the numbers are smaller; karts show no number on the car; buggies show it at the front; stock cars and rally cars show it on the roof. This spec amends spec 092, which is `implemented` and cannot change in place. Only the number drawn on the car changes. The nameplate, the starting grid, the driver card, the profile editor and the number allocation stay as spec 092 made them.

---

## 🗺️ User Flow & Interface Design

The player sees the change only in the top-down race view.

### 1. Placement by car type
The placement comes from the car's `VehicleVisualType`, so sprite cars and procedural cars of the same kind get the same placement.

| `VehicleVisualType` | Kind of car | Number on the car |
| :--- | :--- | :--- |
| `GoKart` | Karts | None |
| `OpenWheel` | Superkarts (the only cars with this body) | None |
| `SandRail` | Buggies: Autocross cross cars, off-road sand rails and trucks | Front |
| `StockCar` | Stock cars | Roof |
| `RallyHatch` | Rally and Rallycross cars | Roof |
| `TouringGT` | GT and sports cars | Front (bonnet) |

GT is not in Mario's rules. Real GT cars carry the number on the doors and the bonnet, seldom on the roof. The doors cannot be seen from above, so this spec uses the bonnet. A contact sheet of every catalog car (`examples/racing_number_sheet.rs`) showed that all `OpenWheel` bodies are superkarts, so they follow the kart rule.

### 2. Anchors
Both anchors come from the existing `ChassisSkeleton`. `ChassisSkeleton` is not changed.
- **Roof:** the middle of the roof. The skeleton cabin includes the windscreen, so on the sprites the roof is behind the cabin centre. The number sits 15% of the cabin length behind the cabin centre (Mario, 2026-10-11: rally numbers slightly more to the rear; the contact sheets show the same for stock cars).
- **Front:** the zone from the front edge of the cabin to the front end of the body. The number sits 35% of the way from the cabin to the nose: on the bonnet, clear of the rounded nose.
- In both zones the digits stay inside the zone.

### 3. Size
- The digit height is at most 35% of the body width (spec 092 allowed up to about 63%).
- The digits stay inside their zone along the car with a margin, as in spec 092.
- Contrast colours, font, rotation and the 5-pixel minimum stay as in spec 092.

---

## ⚙️ Backend Models & API Endpoints

No data, save or database change. The change is in `crates/tdrace-app/src/render/car.rs`:
- A pure function gives the placement (`None`, `Front`, `Roof`) for a `VehicleVisualType`.
- `number_plate` takes the placement and returns the zone for it.
- `CarNumberItem::for_car` takes the visual type. It returns `None` for a kart.
- `RaceSession::collect_car_numbers` passes each car's visual type.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

1. **Fail closed:** a number outside `1..=99` is never drawn (unchanged from spec 092).
2. **Headless boundary:** the placement function is pure and unit tested; drawing code stays in `render`.
3. **Determinism:** no simulation code is changed; the physics crates are not touched.

---

## 🧪 Verification & Acceptance Criteria

- **Scenario: Karts show no number on the car**
  - [x] **Given** a race with karts or superkarts and the player number 17
  - [x] **When** the race is drawn
  - [x] **Then** no kart has a number drawn on its body
  - [x] **And** the nameplates and the starting grid still show the kart numbers

- **Scenario: Buggies and GT cars show the number on the bonnet**
  - [x] **Given** every catalog model and car preset whose visual type is `SandRail` or `TouringGT`
  - [x] **When** the number plate is placed
  - [x] **Then** the plate centre is ahead of the cabin centre and between the front edge of the cabin and the front end of the body
  - [x] **And** the digits are inside the body outline

- **Scenario: Stock cars and rally cars show the number on the roof**
  - [x] **Given** every catalog model and car preset whose visual type is `StockCar` or `RallyHatch`
  - [x] **When** the number plate is placed
  - [x] **Then** the plate centre is 15% of the cabin length behind the cabin centre
  - [x] **And** the digits are inside the cabin and inside the body outline

- **Scenario: Numbers are smaller**
  - [x] **Given** any car that shows a number
  - [x] **When** the digit size is computed
  - [x] **Then** the digit height is at most 35% of the body width
  - [x] **And** two digits fit inside their zone along the car

- **Scenario: Screenshots per category**
  - [x] **Given** a Classic kart race, a Classic Autocross race and a Classic Rallycross race at the default camera zoom
  - [x] **When** the race is drawn
  - [x] **Then** the karts show no number, the Autocross buggies show it at the front and the Rallycross cars on the roof (one screenshot per category)

---

## 🚫 Out of Scope
- Numbers on the doors or sides (lateral showroom sprites), as in spec 092.
- Changes to the nameplate, grid, driver card, profile editor or allocator.
- Per-model number positions (one placement per visual type only).

---

## 🔗 Traceability & Codebase Mapping

### Modified Files
- `[x]` [`crates/tdrace-app/src/render/car.rs`](../crates/tdrace-app/src/render/car.rs) -> placement by visual type, front and roof zones, smaller digits.
- `[x]` [`crates/tdrace-app/src/game/racing_numbers.rs`](../crates/tdrace-app/src/game/racing_numbers.rs) -> pass the visual type of each car.
- `[x]` [`crates/tdrace-app/tests/racing_number_render_tests.rs`](../crates/tdrace-app/tests/racing_number_render_tests.rs) -> placement and size tests.
- `[x]` `crates/tdrace-app/examples/racing_number_sheet.rs` (new) -> contact sheet of every catalog car with its number.

### Verification Assertions
- Verification receipts will be recorded via `keel receipt 111 --cmd "cargo test -p tdrace-app --test racing_number_render_tests"`.
