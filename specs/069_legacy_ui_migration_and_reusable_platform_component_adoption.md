---
type: Feature Spec
template: feature
title: "Legacy UI Migration and Reusable Platform Component Adoption"
description: "Migrates all legacy, ad-hoc UI rendering loops and navigation state across TdRace to the reusable platform component suite in cabinet, expanding the component catalog with newly identified containers (SplitPane, ModalContainer) and widgets (ValueStepper, MetricBar, KpiTile)."
status: draft
created: 2026-09-30
generated: { by: agent/antigravity, at: 2026-09-30T11:15:00Z }
---

# Feature Spec: Legacy UI Migration and Reusable Platform Component Adoption 🕹️

## Executive Summary

While **Spec 068** established the fundamental platform component catalog in [`cabinet::ui`](../crates/cabinet/src/ui/mod.rs) (`HStack`, `VStack`, `FilterBar`, `Accordion`, `CardGrid`, `DataTable`, `TextInputWidget`, `ToastOverlay`, `ScreenFooter`, `SwatchPicker`, `ChecklistModal`) and repaired the interaction logic of the Circuit Selector, the screens across **TdRace** still rely on legacy, ad-hoc drawing routines and duplicated navigation logic.

An exhaustive codebase inspection of all interactive screens in [`crates/tdrace-app/src/ui/`](../crates/tdrace-app/src/ui/) reveals that over **18,000 lines of UI code** contain duplicated spatial layout calculations, manual cursor timers, bespoke table formatting, and fragmented keyboard/gamepad event handlers. Furthermore, this deep review uncovered several common recurring patterns across existing screens that were not captured in the initial 10 platform components:
1. **Two-Column Split Layouts (`SplitPane`)**: Reimplemented manually across 5 major screens (`menu.rs`, `starting_grid.rs`, `race_stats.rs`, `driver_card.rs`, and `profile_ui.rs`).
2. **Numeric Parameter Adjustments (`ValueStepper<T>`)**: Ad-hoc horizontal stepping with clamped increments for laps, bot counts, AI tiers, and rules.
3. **Uniform Modal Windows (`ModalContainer`)**: Inconsistent screen dimming, border sizing, and title framing across 15+ distinct modal dialogs.
4. **Standard Performance & Ability Bars (`MetricBar`)**: Duplicated implementations of progress/stat bars with custom labels and threshold coloring.
5. **High-Impact Metric Summaries (`KpiTile`)**: Prominent stat cards with big value readouts and subtext.

This specification orchestrates the **complete surgical migration of TdRace's legacy screens** onto the unified `cabinet::ui` architecture, expands `cabinet::ui` with the newly identified primitives, and enforces gamepad-first 2D navigation parity across the entire game.

---

## 🗺️ User Flow & Interface Design

### 1. Deep Review: Current UI Inventory & Migration Matrix

A comprehensive analysis of all visual modules in `tdrace-app` and `cabinet` establishes the following migration inventory:

| Screen / Module | Source File | Current Legacy Implementation | Reusable Platform Target | New Candidates Utilized |
| :--- | :--- | :--- | :--- | :--- |
| **Circuit Selector** | [`ui/menu.rs`](../crates/tdrace-app/src/ui/menu.rs) | Manual pill rendering loop, manual tab offsets, hardcoded track card y-calculations | `FilterBar` (Categories) + `FilterBar` (Catalog) + `VStack` (Tracks) | `SplitPane` (Track List vs Dossier) |
| **Modality Select** | [`ui/menu.rs`](../crates/tdrace-app/src/ui/menu.rs) | Bespoke top category glass card loop, hardcoded card heights and gaps | `FilterBar` (1P / MP / Options) + `VStack` (Modality Cards) | `ScreenFooter` (Prompts) |
| **Starting Grid** | [`ui/starting_grid.rs`](../crates/tdrace-app/src/ui/starting_grid.rs) | 5 individual rect calculators with hardcoded math, manual participant row loop | `VStack` (Setup Cards) + `DataTable` (Grid Participants) + `HeroActionButton` | `SplitPane`, `ValueStepper` (Laps/Bots) |
| **Garage Showroom** | [`ui/garage.rs`](../crates/tdrace-app/src/ui/garage.rs) | Bespoke class tab loop, 16-card manual grid loop in Fleet Gallery, custom stat bars | `FilterBar` (Modules) + `CardGrid` (Fleet Roster) + `SwatchPicker` (Paint) | `MetricBar`, `HeroActionButton` (Buy/Select) |
| **Career Select** | [`ui/career_select.rs`](../crates/tdrace-app/src/ui/career_select.rs) | Manual collapsed (52px) vs expanded (148px) math and custom scroll viewport offset | `Accordion<CareerSelectChampionshipCard>` | `ScreenFooter` |
| **Driver Dossier** | [`ui/driver_card.rs`](../crates/tdrace-app/src/ui/driver_card.rs) | Ad-hoc two-column division, manual ability stat bars, hardcoded prompt text | `SplitPane` + `MetricBar` (Radar Stats) | `ScreenFooter` (Cycling & Exit) |
| **Hall of Fame & Results** | [`ui/hall_of_fame.rs`](../crates/tdrace-app/src/ui/hall_of_fame.rs) | Manual column text printing with manual alignment, ad-hoc name input modal | `DataTable<HallOfFameEntry>` + `TextInputWidget` (Name Input) | `ModalContainer`, `ScreenFooter` |
| **Race Telemetry & Stunts**| [`ui/race_stats.rs`](../crates/tdrace-app/src/ui/race_stats.rs) | Bespoke 2-column layout, manual lap time table, hardcoded stunt score boxes | `SplitPane` + `DataTable<LapSplit>` | `KpiTile` (Stunt Scores), `ScreenFooter` |
| **Player Profile & Dossier**| [`ui/profile_ui.rs`](../crates/tdrace-app/src/ui/profile_ui.rs) | Manual tab bar, 6 hardcoded overview tiles, manual 5x5 trophy loops, bespoke form fields | `FilterBar` (Tabs) + `HStack` (KPIs) + `CardGrid` (Trophies) + `TextInputWidget` | `KpiTile`, `SwatchPicker`, `ScreenFooter` |
| **Roster Manager** | [`ui/profile_ui.rs`](../crates/tdrace-app/src/ui/profile_ui.rs) | Bespoke two-column split, manual driver list windowing, ad-hoc text fields | `SplitPane` + `VStack` (Drivers) + `TextInputWidget` (Callsign) | `SwatchPicker`, `ScreenFooter` |
| **Track Manager** | [`ui/track_manager_ui.rs`](../crates/tdrace-app/src/ui/track_manager_ui.rs) | Custom category chips, manual list scrolling, bespoke edit/promote/delete dialogs | `FilterBar` + `VStack` (Tracks) + `TextInputWidget` (Metadata) + `ChecklistModal` | `SplitPane`, `ModalContainer`, `UniversalConfirmModal` |
| **Series Editor** | [`ui/series_editor.rs`](../crates/tdrace-app/src/ui/series_editor.rs) | Bespoke tab cards, manual status toast popup, ad-hoc text modal, manual stepper keys | `FilterBar` (Tabs) + `TextInputWidget` + `ToastOverlay` | `ValueStepper` (Points/Laps), `ScreenFooter` |
| **LAN Hub & Join** | [`ui/lan_ui.rs`](../crates/tdrace-app/src/ui/lan_ui.rs), [`net/ui`](../crates/cabinet/src/net/ui/mod.rs) | Hardcoded 2-card side-by-side math, isolated network numeric keypad | `HStack` (Host/Join Cards) | `VirtualKeypad` (Alphanumeric/IP) |

---

### 2. Standardized Interaction Lifecycle

```mermaid
flowchart LR
    subgraph Input ["Unified Gamepad / Keyboard"]
        DPAD[D-Pad / Left Stick: Orthogonal 2D Navigation]
        BUMP[Bumpers LB/RB: Instant FilterBar Tab Cycling]
        FACE[South A / Space / Enter: Confirm / Expand]
        BACK[East B / Escape: Back / Dismiss Modal]
    end

    subgraph Hierarchy ["Screen Focus Tree"]
        TopTab[FilterBar / Tabs]
        Split[SplitPane: Left vs Right Panel]
        List[VStack / CardGrid / DataTable]
        Bottom[HeroActionButton / ScreenFooter]
    end

    Input --> Hierarchy
    TopTab -->|ExitDown| Split
    Split -->|ExitTop| TopTab
    Split -->|ExitBottom| Bottom
```

---

## ⚙️ Backend Models & API Endpoints

### 1. New Layout Containers in `cabinet::ui`

```rust
// In crates/cabinet/src/ui/layout.rs

/// Responsive two-column spatial container with focus handoff.
pub struct SplitPane {
    pub bounds: LayoutRect,
    pub left_ratio: f32, // e.g. 0.42 (42% left, 58% right minus gap)
    pub col_gap: f32,
    pub active_pane: usize, // 0 = Left, 1 = Right
}

impl SplitPane {
    pub fn new(x: f32, y: f32, w: f32, h: f32, left_ratio: f32, col_gap: f32) -> Self;
    pub fn left_rect(&self) -> LayoutRect;
    pub fn right_rect(&self) -> LayoutRect;
    pub fn handle_nav(&mut self, left: bool, right: bool) -> NavBoundaryExit;
}
```

### 2. New Interactive Widgets in `cabinet::ui`

```rust
// In crates/cabinet/src/ui/widgets.rs

/// Interactive horizontal stepper for numeric parameters or discrete option sets.
pub struct ValueStepper<T> {
    pub label: String,
    pub value: T,
    pub min: T,
    pub max: T,
    pub step: T,
    pub is_focused: bool,
}

/// Standardized progress / capacity bar with threshold coloring.
pub struct MetricBar {
    pub label: String,
    pub ratio: f32, // 0.0..=1.0
    pub display_val: String,
    pub bar_color: Color,
}

/// High-impact KPI metric card with label, prominent number, and trend badge.
pub struct KpiTile {
    pub label: String,
    pub primary_metric: String,
    pub subtext: Option<String>,
    pub accent_color: Color,
}
```

### 3. Screen Migration Blueprints

#### Phase 1: Primary Navigation & Grid Setup
1. **Circuit Selector (`menu.rs`)**:
   - `category_bar: FilterBar` configured with `FilterBarStyle::Pills` over the 8 motorsport categories.
   - `catalog_bar: FilterBar` configured with `FilterBarStyle::Shelf` (`OFFICIAL` vs `CUSTOM`).
   - `track_stack: VStack` managing windowed track card scrolling.
   - `SplitPane` partitioning track list from track detail dossier.
2. **Modality Select (`menu.rs`)**:
   - `category_bar: FilterBar` for the 3 top tabs (`1. SINGLE PLAYER`, `2. MULTIPLAYER`, `3. OPTIONS & GARAGE`).
   - `cards_stack: VStack` for modality cards (`Quick Race`, `Career`, `Custom Race`, `Time Trial`, `Free Ride`).
   - `ScreenFooter` rendering contextual guidance.
3. **Starting Grid (`starting_grid.rs`)**:
   - `SplitPane`: Left setup column (`0.44` width) vs Right roster column (`0.56` width).
   - Left Column: `VStack` holding Player Card, Circuit Summary, Garage Quick-Select, and `HeroActionButton` ("START RACE").
   - Right Column: `ValueStepper` for Bot Count and AI Difficulty, and `DataTable<GridParticipant>` for the participant list.

#### Phase 2: Motorsport Showroom & Career
4. **Garage & Fleet Gallery (`garage.rs`)**:
   - Class selector converted to `FilterBar`.
   - Fleet Gallery converted to `CardGrid<RealCarModel>` with 4 columns and 2D matrix navigation.
   - Livery customization converted to `SwatchPicker`.
   - Vehicle specs and radar converted to `MetricBar` instances.
   - Buy/Select CTA converted to `HeroActionButton`.
5. **Career Championship Selector (`career_select.rs`)**:
   - Replace custom scrolling and height interpolation with `Accordion<CareerSelectChampionshipCard>`.
   - Single-expand mode: selecting a championship smoothly expands its calendar, prize pool, and vehicle restrictions.
6. **Driver Dossier (`driver_card.rs`)**:
   - `SplitPane` container: Left column (Bio, Signature Rides) vs Right column (Handling Radar).
   - Radar stats rendered via standardized `MetricBar`.
   - Footer navigation prompts rendered via `ScreenFooter`.

#### Phase 3: Driver Identity, Records & Statistics
7. **Hall of Fame & Results (`hall_of_fame.rs`, `menu.rs`)**:
   - Standings table converted to `DataTable<HallOfFameEntry>` with rank badges (Gold, Silver, Bronze) and player row highlighting.
   - Driver name entry converted to modal dialog wrapping `TextInputWidget`.
   - Navigation prompts converted to `ScreenFooter`.
8. **Detailed Race Telemetry (`race_stats.rs`)**:
   - `SplitPane` layout: Left column (Lap Splits Table via `DataTable<LapSplit>`) vs Right column (Stunt Score & Driving Telemetry).
   - Stunt scores rendered via `KpiTile` widgets.
   - Bottom action bar rendered via `ScreenFooter`.
9. **Player Profile & Roster Manager (`profile_ui.rs`)**:
   - Top tab ribbon converted to `FilterBar` (`Overview`, `Disciplines`, `Cabinet`, `Championships`, `Telemetry`).
   - Overview KPI row converted to `HStack` containing 6 `KpiTile` widgets.
   - Trophy Cabinet converted to `CardGrid` (5x5 matrix).
   - Callsign and alias editors in Roster Manager converted to `TextInputWidget`.
   - Livery customization converted to `SwatchPicker`.
   - Roster driver list converted to `VStack`.

#### Phase 4: Studio & Network Tools
10. **Track Manager (`track_manager_ui.rs`)**:
    - Top tabs and category filters converted to `FilterBar`.
    - Track list converted to `VStack`.
    - Track metadata editing dialog converted to modal containing two `TextInputWidget` fields.
    - Category assignment dialog converted to `ChecklistModal`.
11. **Declarative Championship Editor (`series_editor.rs`)**:
    - Workflow tabs (`Rules`, `Calendar`, `Grid`, `Export`) converted to `FilterBar`.
    - Numerical rules (laps, rounds, points systems) converted to `ValueStepper`.
    - In-editor notifications converted to `ToastOverlay`.
    - Footer guidance converted to `ScreenFooter`.
12. **LAN Multiplayer Hub (`lan_ui.rs`)**:
    - Host/Join selection converted to `HStack` with 2 interactive cards.
    - Promote `ip_keypad` into a general `VirtualKeypad` component under `cabinet::ui` for arcade/gamepad input parity.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

### 1. State Boundary Sanitization & Clamping
- UI navigation states (`MenuPanelFocus`, active filter indexes, card grid coordinates, and accordion expansion sets) must be validated against current collection lengths on every frame. If dynamic updates (such as deleting a user track, modifying filters, or receiving remote player disconnections) reduce the item count, selection indexes must immediately clamp or wrap without panic or index-out-of-bounds errors.

### 2. Input Event Sandboxing & Headless Safety
- All macroquad input checks (`is_key_pressed`, `is_mouse_button_pressed`, `mouse_position`) are wrapped in `catch_unwind` guards via `safe_key_pressed` and `safe_mouse_pos` within `cabinet` components to prevent headless test panics or thread-local uninitialized context failures.

### 3. Text Input Sanitization
- `TextInputWidget` strictly validates and sanitizes all typed input:
  - Enforces UTF-8 validity and strips non-printable control characters (`\x00`..=`\x1F`).
  - Clamps string buffers to `max_chars` to prevent memory exhaustion.
  - Enforces alphanumeric and safe symbol allowlists when used for file names or network addresses, blocking path traversal sequences (`../`, `..\\`).

### 4. Modality Licensing & Progression Guard
- Filtering by category does not bypass career or driver tier requirements:
  - If a track or championship requires an unreached license tier, the UI visually highlights the lock state with `Palette::RED` / `Palette::UI_CARD_BORDER` and disables launch actions.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- Command to run all cabinet UI tests:
  ```bash
  cargo test -p cabinet
  ```
- Command to run modality and navigation flow tests:
  ```bash
  cargo test -p tdrace-app --test modality_flow_tests
  ```
- Command to run workspace unit tests:
  ```bash
  cargo test --workspace --lib
  ```
- Command to run Keel spec linter:
  ```bash
  keel validate .
  ```

### Manual Acceptance Criteria (Pseudo-Gherkin)

#### Scenario 1: Circuit Selector uses Platform Components
- [ ] **Given** the player enters the Circuit Selector menu (`GameState::Menu`)
- [ ] **When** inspecting the category pills and catalog tabs
- [ ] **Then** they are rendered using `FilterBar` instances with `FilterBarStyle::Pills` and `FilterBarStyle::Shelf`
- [ ] **And** pressing `Left`/`Right` or Gamepad Bumpers cycles through categories seamlessly
- [ ] **And** pressing `Down` from the catalog tabs transfers focus to the track `VStack`

#### Scenario 2: Starting Grid SplitPane and ValueStepper Integration
- [ ] **Given** the player is on the Starting Grid setup screen (`GameState::StartingGrid`)
- [ ] **When** inspecting the layout
- [ ] **Then** the screen is structured using a `SplitPane` container
- [ ] **And** the bot count and lap count adjustments are handled by `ValueStepper` widgets responsive to `Left`/`Right` keys
- [ ] **And** the participants list is rendered via `DataTable<GridParticipant>` with distinct podium rank badges

#### Scenario 3: Fleet Gallery CardGrid Traversal in Garage
- [ ] **Given** the player opens the Fleet Gallery in the Garage (`GameState::Garage`)
- [ ] **When** navigating with Gamepad D-pad or Arrow keys
- [ ] **Then** focus moves across the vehicle roster in a 2D matrix using `CardGrid<RealCarModel>`
- [ ] **And** reaching column boundaries wraps or emits exit signals without index overflow
- [ ] **And** vehicle performance stats are rendered using standardized `MetricBar` components

#### Scenario 4: Career Championship Accordion Drawer Mechanics
- [ ] **Given** the player is in Career Championship Select (`GameState::CareerSelect`)
- [ ] **When** navigating between championship tiers and pressing `Enter` or Gamepad `A`
- [ ] **Then** the selected tier expands via `Accordion<CareerSelectChampionshipCard>` while collapsing other tiers
- [ ] **And** the viewport automatically adjusts to keep the expanded tier centered

#### Scenario 5: Hall of Fame DataTable and Record Name Entry Modal
- [ ] **Given** a player qualifies for the Top 10 Hall of Fame upon race completion
- [ ] **When** the record submission dialog opens
- [ ] **Then** it renders inside a standard modal using `TextInputWidget` for callsign entry
- [ ] **When** inspecting the all-time Hall of Fame records screen
- [ ] **Then** records are rendered in a `DataTable<HallOfFameEntry>` with highlighted human player row

#### Scenario 6: Player Profile Overview KPI Tiles and SwatchPicker
- [ ] **Given** the player opens the Player Profile screen (`GameState::ProfileManager`)
- [ ] **When** viewing the Overview tab
- [ ] **Then** the 6 key career statistics are rendered via an `HStack` of `KpiTile` widgets
- [ ] **When** opening the Driver Customization modal
- [ ] **Then** livery color selection is operated using a `SwatchPicker`

#### Scenario 7: Track Manager Dialogs and ChecklistModal
- [ ] **Given** the player opens the Track Manager (`GameState::TrackManager`)
- [ ] **When** editing track metadata or promoting categories
- [ ] **Then** metadata fields use `TextInputWidget` and category selection uses `ChecklistModal`
- [ ] **And** confirmation dialogs use `UniversalConfirmModal`

---

## 🔗 Traceability & Codebase Mapping

### Platform Components to Add / Standardize
- `[ ]` [`crates/cabinet/src/ui/layout.rs`](../crates/cabinet/src/ui/layout.rs) — Add `SplitPane` layout container.
- `[ ]` [`crates/cabinet/src/ui/widgets.rs`](../crates/cabinet/src/ui/widgets.rs) — Add `ValueStepper<T>`, `MetricBar`, and `KpiTile`.
- `[ ]` [`crates/cabinet/src/net/ui/ip_keypad.rs`](../crates/cabinet/src/net/ui/ip_keypad.rs) — Generalize `ip_keypad` into a platform-wide `VirtualKeypad`.
- `[ ]` [`crates/cabinet/src/ui/mod.rs`](../crates/cabinet/src/ui/mod.rs) — Re-export new containers and widgets.

### Application Screens to Migrate
- `[ ]` [`crates/tdrace-app/src/ui/menu.rs`](../crates/tdrace-app/src/ui/menu.rs) — Migrate Circuit Selector & Modality Select to `FilterBar`, `VStack`, and `SplitPane`.
- `[ ]` [`crates/tdrace-app/src/ui/starting_grid.rs`](../crates/tdrace-app/src/ui/starting_grid.rs) — Migrate setup and roster to `SplitPane`, `ValueStepper`, and `HeroActionButton`.
- `[ ]` [`crates/tdrace-app/src/ui/garage.rs`](../crates/tdrace-app/src/ui/garage.rs) — Migrate Fleet Gallery and Showroom to `FilterBar`, `CardGrid`, `SwatchPicker`, and `MetricBar`.
- `[ ]` [`crates/tdrace-app/src/ui/career_select.rs`](../crates/tdrace-app/src/ui/career_select.rs) — Migrate championship cards to `Accordion`.
- `[ ]` [`crates/tdrace-app/src/ui/driver_card.rs`](../crates/tdrace-app/src/ui/driver_card.rs) — Migrate dossier layout to `SplitPane` and `MetricBar`.
- `[ ]` [`crates/tdrace-app/src/ui/hall_of_fame.rs`](../crates/tdrace-app/src/ui/hall_of_fame.rs) — Migrate standings table to `DataTable` and name modal to `TextInputWidget`.
- `[ ]` [`crates/tdrace-app/src/ui/race_stats.rs`](../crates/tdrace-app/src/ui/race_stats.rs) — Migrate race stats dashboard to `SplitPane`, `DataTable`, and `KpiTile`.
- `[ ]` [`crates/tdrace-app/src/ui/profile_ui.rs`](../crates/tdrace-app/src/ui/profile_ui.rs) — Migrate overview tab, cabinet grid, and roster manager to platform components.
- `[ ]` [`crates/tdrace-app/src/ui/track_manager_ui.rs`](../crates/tdrace-app/src/ui/track_manager_ui.rs) — Migrate track lists, metadata fields, and dialogs to platform suite.
- `[ ]` [`crates/tdrace-app/src/ui/series_editor.rs`](../crates/tdrace-app/src/ui/series_editor.rs) — Migrate championship editor tabs, inputs, and steppers.
- `[ ]` [`crates/tdrace-app/src/ui/lan_ui.rs`](../crates/tdrace-app/src/ui/lan_ui.rs) — Migrate LAN hub cards and IP entry.
