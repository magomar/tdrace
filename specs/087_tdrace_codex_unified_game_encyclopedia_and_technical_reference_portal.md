---
type: Feature Spec
template: feature
title: "TDRace Codex: Unified Game Encyclopedia and Technical Reference Portal"
description: "Grows the Motorsport Showroom into the TDRace Codex: one Astro site with Showroom, Technical, Driving and Racing sections, fed by a Rust exporter that serialises the real game data (cars, chassis, suspension, tyres, surfaces, drivetrain, damage, assists, controls, HUD, formats, careers, AI rivals) instead of regex-scraping source files."
status: in_progress
verified: { by: "human:mario", at: "2026-10-04T20:30:47Z", hash: "b8b76d5a8826" }
created: 2026-10-04
generated: { by: agent/claude-opus-5-5, at: 2026-10-04T18:00:00Z }
depends_on:
  - "011"
---

# Feature Spec: TDRace Codex — Unified Game Encyclopedia and Technical Reference Portal 📖🏁

The Motorsport Showroom ([Spec 011](011_game_asset_catalogue_and_physics_reference_portals.md)) shows car cards, circuit cards and one physics page. Since then the game gained chassis skeletons (075), suspension archetypes (076), tyre compounds and wheel geometry (074), directional damage and repair (078, 063), a cockpit telemetry HUD (079), five steering profiles (041), joker laps (081, 082), pit stops (062), 6-tier careers (051, 052) and an academy (060). None of this is on the web. This spec turns the showroom into the **TDRace Codex**: a single encyclopedia of the game, where the showroom is one section.

---

## 🎯 Executive Summary & Problem Statement

### 1.1 Sweep findings (2026-10-04)

**The current portals**
- `portals/showroom` has 4 routes: `/`, `/vehicles`, `/circuits`, `/physics`. It has no detail pages and no compare view.
- `portals/option-a-starlight` is a Starlight wiki. Its content is a symlink to `docs/`. It has no images.
- Neither portal can be built in a fresh worktree: no `node_modules`, and the `tracks/` submodule is not initialised.
- `python3 scripts/verify_okf.py` fails with 87 errors. 86 are receipts that Starlight excludes anyway. 1 is a broken `bbox` link in `docs/engineering/circuit_building_analysis.md`.

**The data pipeline is stale and partly invented**
- `scripts/generate_asset_data.py` reads `crates/tdrace-app/src/catalog/mod.rs` with regular expressions.
- `vehicles.json` has 124 cars. The code has 129 (110 in `ALL_REAL_CARS`, 12 in `CLASSIC_ARCADE_CARS`, 7 in `VAULT_ARCHIVE_CARS`). Five classic vintage cars are missing. `classic_ax_brawler` shows 420 bhp; the code says 250.
- `engine_force`, `drive_bias` and `brakes_kn` are re-computed in Python, not read from the game. Downforce and drag are parsed from a free-text string.
- `surfaces.json` is hard-coded in Python with 12 surfaces. `SurfaceType` (`crates/wheelbase/src/surface.rs:6`) now has 15. `circuits.json` already uses the new names, so the two files disagree.
- `portals/shared/schemas/okf.ts` defines Zod schemas, but no page imports them.
- Vault (archived) cars are published.

**The real data is serde-ready but unused**
- `CarConfig` (`crates/wheelbase/src/config.rs:1095`) and its parts derive `Serialize`: `ChassisSkeleton` :645, `SuspensionArchetype` :768, `SuspensionConfig` :854, `DifferentialType` :597, `EnginePlacement` :1066, `TireConfig` :180, `WheelAssemblyConfig` :9, `DriverAssistsConfig` :301.
- `CompoundId` (`surface.rs:360`, 8 compounds), `SurfaceAffinityMap::for_compound` (`surface.rs:464`, an 8 × 15 grip matrix) and `TireCompoundConfig::from_id` (`tire.rs:521`) are also serialisable.
- `RealCarModel` (`catalog/mod.rs:11`) has no serde. `to_car_config()` (`:134`) gives the real per-car physics.

**Docs have drifted**
- `docs/vehicles/index.md` says 80 cars, 25 categories, 5 modules (no autocross).
- `docs/physics/tire_pacejka.md` shows the pre-043 tyre model.
- `docs/physics/surfaces.md` lists 12 surfaces.
- `docs/physics/powertrain.md` does not mention differentials.
- No doc outside the receipts mentions suspension archetypes, chassis skeletons, compounds or the damage model.

**Game-data bugs found during the sweep** (filed as separate Beads issues, not fixed here)
- Drivetrain strings `"AWD Hybrid"`, `"RWD Hybrid"` and `"4WD 4WS"` fall to the default arm in `to_car_config()` (`catalog/mod.rs:150-154`). Hybrid AWD hypercars and 4WD trucks simulate as RWD.
- `get_tier_name` (`catalog/mod.rs:3846-3852`) has rally tiers 1–5 only, with labels that do not match the catalogue. Rally cars in tiers 6–7 show "Tier 1".
- Sound overrides `rally_audi_sport_quattro_e2`, `rally_quattro_s1` and `rally_ford_rs200` (`catalog/mod.rs:490-507`) never match a catalogue id.

### 1.2 Important fact for the "chassis models" request
Per-car physics comes from about 12 base presets (`CarChoice::config()`, `ui/menu.rs:675`, plus the GT presets in `module/gt.rs`). `to_car_config()` changes mass, power, top speed, drive bias, brakes, grip, steering, inertia, aero and terrain flotation. It does **not** change chassis skeleton, suspension archetype, wheel geometry, compound, engine placement or differential per model. So the codex will show about 9 chassis skeletons, 6 suspension archetypes, 8 compounds and 3 differential types, each with the list of cars that use it. More variety per car is a game-design change, not a codex change, and is out of scope here.

### 1.3 Core design principles
1. **The game is the source of truth.** All numbers come from a Rust exporter that calls the same functions the game calls. No values are copied into Python, TypeScript or Markdown.
2. **Stale data fails the build.** A Rust test compares the committed JSON with fresh exporter output.
3. **Show the model, then the numbers.** Each technical page explains the idea in plain words, then shows a diagram, then the table.
4. **One site for players and curious readers.** Starlight stays as the contributor manual for internals and specs.
5. **Launch-scope aware.** One build flag gives the Steam v1 view (Classic, Karting, Autocross, Rallycross). Vault content never appears.

---

## 🗺️ User Flow & Interface Design

### 1. Site map

`portals/showroom` is renamed to `portals/codex`. Port 4322 stays. Top navigation has five sections.

| Section | Route | Pages | Content source |
| :--- | :--- | :--- | :--- |
| **Home** | `/` | Hero, section tiles, "what's new" from recent specs | Exporter counts |
| **Showroom** | `/showroom/cars` | Car catalogue: filters by discipline, tier, drivetrain, chassis, suspension, compound | `cars.json` |
| | `/showroom/cars/[id]` | Car detail: images, spec sheet, derived physics, chassis diagram, suspension, wheels and compound, engine and sound, damage profile, "races in" | `cars.json` + shared tables |
| | `/showroom/compare` | 2–4 cars side by side, stat radar, power-to-weight, grip, braking | `cars.json` |
| | `/showroom/circuits` | Circuit catalogue: filters by discipline, country, surface, features (jumps, joker, pit lane, junctions) | `circuits.json` |
| | `/showroom/circuits/[id]` | Circuit detail: layout SVG, surface strip, length, width, turns, joker route, pit lane, OSM/Wikipedia provenance | `circuits.json` |
| **Technical** | `/technical/chassis` | Chassis skeletons: overhangs, body width, cabin offset, light anchors, collision hull; top-down SVG per skeleton; body visual types | `chassis.json` |
| | `/technical/suspension` | 6 archetypes; per-corner springs, damping, travel, camber; anti-roll bars; robustness and part-cost factors; failure modes (bottom-out, landing, camber loss, steering pull, collapsed pushrod) | `suspension.json` |
| | `/technical/tyres` | `TireConfig` grip curve (slip angle → force, interactive); 8 compounds with temperature window, wear, overheat; wheel geometry and inertia | `tyres.json` |
| | `/technical/surfaces` | 15 surfaces: friction, rolling resistance, drag; compound × surface heatmap (8 × 15) | `surfaces.json` |
| | `/technical/drivetrain` | Engine force from bhp, drive bias, differentials (open, spool, LSD with power/coast lock and preload), engine placement | `drivetrain.json` |
| | `/technical/damage` | 8 impact zones, zone × engine placement weight table, energy dead zone, health capacities, power-versus-engine-health curve, field repair caps, pit repair, garage invoice rules | `damage.json` |
| | `/technical/physics-lab` | The existing Pacejka graph and formula cards, moved here | Existing components |
| **Driving** | `/driving/controls` | 4 binding presets (Hybrid, WASD, Arrows, Classic) as keyboard and gamepad diagrams; in-race hotkeys; gamepad profile and dead zones | `controls.json` |
| | `/driving/steering` | 5 steering profiles with their numbers and a step-response chart | `driving.json` |
| | `/driving/assists` | Arcade, Sport, Pro: TCS, ESC, ABS, counter-steer, handbrake bypass; low-speed authority and grip-aware steering | `driving.json` |
| | `/driving/hud` | Annotated HUD: speedometer, position and lap, timing tower, minimap, compound badge, joker badge, pit messages; cockpit hologram KINEMATICS and DYNAMICS modes; curve helper styles and colour schemes; nameplates; locator aids; debug overlays | `hud.json` + hand-written MDX + screenshots |
| | `/driving/cameras` | Camera modes, look-ahead, speed zoom | Hand-written MDX |
| **Racing** | `/racing/disciplines` | Each discipline: identity, cars, circuits, tiers | Exporter |
| | `/racing/formats` | Laps and time attack; joker rule; pit service; qualifying, stage rally, elimination | `racing.json` |
| | `/racing/championships` | Point systems (FIA, MotoGP, ClassicArcade, NascarCup) and the 33 series presets | `racing.json` |
| | `/racing/career` | Tier ladders per discipline, promotion gates, credits and XP, repair costs | `racing.json` + `docs/career/index.md` |
| | `/racing/academy` | 4 lessons, medals, licence grades D to S | `racing.json` |
| | `/racing/rivals` | AI driver roster, 6 driving styles, 5 skill tiers, 5 mistake kinds | `rivals.json` |
| | `/racing/multiplayer` | LAN: 2–8 players, collision modes, how to host and join | Hand-written MDX |

Every technical page links each table row to the cars that use it. Every car page links back to the technical page for each of its parts.

### 2. Interactive widgets (client-side, no new libraries)
- **Chassis plan view**: SVG top-down drawing from `ChassisSkeleton` and `WheelAssemblyConfig`, with dimension lines.
- **Suspension corner**: SVG spring and damper; sliders for load show travel and camber.
- **Grip curve**: canvas plot of the `TireConfig` slip-angle curve (reuses the PacejkaGraph pattern).
- **Compound × surface heatmap**: 8 × 15 table coloured by grip.
- **Damage map**: SVG car outline with the 8 impact zones; pick an engine placement to see the weight table; power-versus-health curve.
- **Steering response**: step-response chart for each steering profile.
- **Controls diagram**: SVG keyboard and gamepad with the bound keys lit for the chosen preset.

### 3. Relation to Starlight
- The Codex is the only site that shows game data.
- Starlight stays as the **contributor manual**: engineering notes, netcode, track building, specs-level internals.
- These stale docs move into the Codex as generated pages and are deleted from `docs/`: `docs/vehicles/*`, `docs/physics/surfaces.md`, `docs/physics/tire_pacejka.md`. `docs/career/index.md` is reused as MDX in `/racing/career`.
- Each site links to the other in its header.

---

## ⚙️ Backend Models & API Endpoints

### 1. Rust exporter (`crates/tdrace-app/src/bin/export_codex.rs`)
A new binary, like the existing `track_bake` and `generate_surface_textures` bins.

```bash
cargo run -p tdrace-app --bin export_codex -- --out portals/shared/data/codex --scope all
cargo run -p tdrace-app --bin export_codex -- --out portals/shared/data/codex --scope launch
cargo run -p tdrace-app --bin export_codex -- --check
```

- It walks `ALL_REAL_CARS` and `CLASSIC_ARCADE_CARS`. It skips `VAULT_ARCHIVE_CARS`.
- For each car it writes a `CodexCar` DTO (hand-written, because `RealCarModel` holds macroquad colours) plus the result of `to_car_config()`.
- Shared tables are written once and referenced by id: chassis skeletons, suspension archetypes, compounds, surfaces, affinity matrix, differentials, engine placements, damage constants, assist profiles, steering profiles, input presets, HUD element list, race formats, point systems, series presets, tier names, academy lessons, licence grades, AI roster, driving styles, skill tiers, mistake kinds.
- Circuits come from the `tracks/` JSON through the existing loader (not regex), plus `EmbeddedCircuit` metadata.
- Every file carries `{ "schema_version": 1, "generated_from": "<git sha>", "scope": "all|launch" }`.
- `--scope launch` keeps modules `classic`, `kart`, `autocross`, `rally` only.
- Constants that today live inside function bodies (damage capacities, dead zone, repair caps, power curve, suspension damage thresholds) are lifted into named `pub const` items so the exporter reads them. This is the only change to game code. Values do not change.

Output files under `portals/shared/data/codex/`: `cars.json`, `circuits.json`, `chassis.json`, `suspension.json`, `tyres.json`, `surfaces.json`, `drivetrain.json`, `damage.json`, `driving.json`, `controls.json`, `hud.json`, `racing.json`, `rivals.json`, `meta.json`.

### 2. Freshness guard
- Test `crates/tdrace-app/tests/codex_export_tests.rs` runs the exporter in memory and compares it with the committed JSON (ignoring `generated_from`).
- A changed physics constant without a re-export fails `cargo test`.

### 3. Schema enforcement (`portals/shared/schemas/codex.ts`)
- Zod schemas for every file, loaded as Astro content collections with the `file()` loader.
- A schema mismatch stops `bun run build` with a non-zero exit code.
- `portals/shared/schemas/okf.ts` keeps the OKF frontmatter schema only. Its old vehicle, circuit and surface schemas are removed.

### 4. Retired pieces
- `scripts/generate_asset_data.py` loses its vehicle and surface code. It keeps only the circuit SVG thumbnail generation, until the exporter can write thumbnails.
- `portals/shared/data/{vehicles,surfaces}.json` are deleted. `portals/shared/data/circuits.json` moves into `codex/`.
- The hard-coded module-name aliases in `vehicles.astro` and the hard-coded counts in `index.astro` are removed; counts come from `meta.json`.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

### 1. Static output only
- The Codex is a static Astro build. No server, no analytics, no external requests at runtime.

### 2. Publication rules
- **Do not publish the Codex outside the team until [Spec 047](047_fictional_branding_and_realworld_ip_removal_for_steam_release.md) is implemented.** The catalogue still holds real manufacturer names and car names.
- A public build must use `--scope launch`. GT, NASCAR and off-road are DLC and are not shown before release.
- Vault content is never exported.
- The exporter shows tuning numbers, not save files, credentials or player data.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- `cargo test -p tdrace-app --test codex_export_tests` — committed JSON matches exporter output; car count equals the catalogue (minus vault); every car references a valid chassis, suspension, compound and differential id; `--scope launch` output has no `gt`, `nascar` or `extreme_offroad` record.
- `cd portals && bun run build:codex` — exit code 0; Zod schemas accept every data file.
- `python3 scripts/verify_okf.py` — zero errors (receipts excluded like in Starlight; `bbox` link fixed).

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Codex replaces the showroom**
  - [ ] **Given** a fresh worktree with `tracks/` initialised and `bun install` done in `portals/`
  - [ ] **When** I run `make codex`
  - [ ] **Then** the Codex opens on port 4322
  - [ ] **And** the header shows Showroom, Technical, Driving and Racing.

- **Scenario: Car data comes from the game**
  - [ ] **Given** the car `classic_ax_brawler`
  - [ ] **When** I open `/showroom/cars/classic_ax_brawler`
  - [ ] **Then** the page shows 250 bhp, the same value as `catalog/mod.rs`
  - [ ] **And** it shows its chassis skeleton, suspension archetype, compound, differential and engine placement, each linked to its technical page.

- **Scenario: Stale export fails the tests**
  - [ ] **Given** I change `SuspensionArchetype::robustness_factor` for one archetype
  - [ ] **When** I run `cargo test -p tdrace-app --test codex_export_tests` without re-exporting
  - [ ] **Then** the test fails and names the changed file.

- **Scenario: Technical section explains the models**
  - [ ] **Given** the Technical section
  - [ ] **When** I open chassis, suspension, tyres, surfaces, drivetrain and damage
  - [ ] **Then** each page has a plain-words explanation, at least one diagram and a data table
  - [ ] **And** surfaces shows 15 surfaces and an 8 × 15 compound heatmap.

- **Scenario: Driving section matches the game**
  - [ ] **Given** the Hybrid controls preset in `crates/cabinet/src/input/mapping.rs`
  - [ ] **When** I open `/driving/controls`
  - [ ] **Then** the keyboard diagram lights Q, ↑, A, ↓, O, ←, P, →, Space
  - [ ] **And** `/driving/steering` shows Smooth, Balanced, Sharp and Raw with the values from `DigitalInputConfig::from_profile`.

- **Scenario: Launch-scope build**
  - [ ] **Given** an export with `--scope launch`
  - [ ] **When** I build the Codex
  - [ ] **Then** no page lists a GT, NASCAR, off-road or vault car or circuit.

---

## 📦 Delivery Phases

Each phase is one Beads task under the epic and ends with a green build.

| Phase | Scope | Exit check |
| :--- | :--- | :--- |
| **P0 Foundation** | Rename showroom → codex; exporter bin with cars, circuits, surfaces, meta; Zod collections; freshness test; scope flag; new nav shell; current pages work on new data | Tests + build green; counts match catalogue |
| **P1 Showroom** | Car and circuit detail pages; compare view; richer filters | Every car and circuit has a page |
| **P2 Technical** | Lift damage and suspension constants to `pub const`; export chassis, suspension, tyres, drivetrain, damage; six technical pages and their widgets; move physics lab | Technical scenario passes |
| **P3 Driving** | Export controls, steering, assists, HUD list; controls, steering, assists, HUD and camera pages; HUD screenshots | Driving scenario passes |
| **P4 Racing** | Export formats, series, tiers, academy, rivals; six racing pages; reuse career doc | All racing routes build |
| **P5 Clean-up** | Delete stale `docs/` pages and old JSON; trim `generate_asset_data.py`; cross-links with Starlight; fix `verify_okf.py` | `verify_okf.py` zero errors |

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files

| File | Change |
| :--- | :--- |
| `crates/tdrace-app/src/bin/export_codex.rs` | New exporter |
| `crates/tdrace-app/src/codex/mod.rs` | New DTOs (`CodexCar` and table rows) |
| `crates/tdrace-app/tests/codex_export_tests.rs` | New freshness and integrity tests |
| `crates/wheelbase/src/car.rs`, `crates/wheelbase/src/config.rs` | Lift damage and suspension thresholds to `pub const` (no value change) |
| `portals/showroom/` → `portals/codex/` | Rename; new routes listed above |
| `portals/shared/schemas/codex.ts` | New Zod schemas |
| `portals/shared/data/codex/*.json` | New generated data |
| `portals/package.json`, `Makefile`, `justfile` | `codex` targets replace `showroom` targets |
| `scripts/generate_asset_data.py` | Keep circuit thumbnails only |
| `docs/vehicles/*`, `docs/physics/surfaces.md`, `docs/physics/tire_pacejka.md` | Removed after P5 |

### Beads Epic Mapping
- Epic: `tdrace-b3u4`. Phase tasks: P0 `tdrace-b3u4.1`, P1 `.2`, P2 `.3`, P3 `.4`, P4 `.5`, P5 `.6`.
- Related bugs filed during the sweep: `tdrace-0jvg` (hybrid/4WS drivetrain fallthrough), `tdrace-ilec` (rally tier names 6–7), `tdrace-t8vw` (unmatched sound override ids).
