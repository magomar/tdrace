---
type: Feature Spec
template: feature
title: "Unified UI and UX Consistency Across TDrace and Shared Platform Crate"
description: "Establishes a unified platform UI/UX architecture and reusable component suite in cabinet (stacks, filter bars, accordions, card grids, text input, tables, toasts, modals, footer prompts, gamepad navigation) and defines the migration blueprints for tdrace screens to achieve seamless gamepad/keyboard parity and visual consistency across all projects."
status: in_progress
receipt: "docs/receipts/spec-068-receipt.md"
created: 2026-09-30
verified: { by: "human:mario", at: "2026-09-30T08:31:00Z" }
generated: { by: agent/antigravity, at: 2026-09-30T08:15:42Z }
---

# Feature Spec: Unified UI and UX Consistency Across TDrace and Shared Platform Crate 🕹️

## Executive Summary

As **TdRace** evolved from an arcade prototype into a multi-discipline motorsport ecosystem, user interface elements were authored ad-hoc across separate game states and modules. This organic growth produced noticeable inconsistencies:
1. **Navigation Dead-Ends & Input Asymmetry**: Components such as filter bars, category pills, search fields, and catalog tabs were frequently reachable only via mouse input. In the Circuit Selector, keyboard and gamepad focus was initially trapped in the track cards, leaving the category pills and presets/custom tabs unreachable.
2. **Duplicated Layout & Viewport Math**: Windowed card lists, 2D matrix grids, viewport scrolling calculations, and item distribution algorithms were rewritten independently in `menu.rs`, `track_manager_ui.rs`, `garage.rs`, `starting_grid.rs`, `profile_ui.rs`, and `career_select.rs`.
3. **Inconsistent Visual Focus Languages**: Different screens communicated selection through disparate cues (some used background color shifts, others used subtle borders, and others used varying thickness without consistent active/hover demarcation).
4. **Ad-Hoc Text & Table Input Handlers**: Text editing (callsigns, track names, LAN IP addresses) and tabular data displays (race results, championship standings, Hall of Fame leaderboards) relied on bespoke, non-standardized implementations with fragile cursor logic and lack of gamepad accessibility.
5. **Platform Fragmentation**: Sibling games and tools built on the shared platform crate [`cabinet`](../crates/cabinet) lacked a standard set of gamepad-first, accessible, ergonomic UI components, resulting in duplicate implementations across projects.

This specification elevates [`cabinet::ui`](../crates/cabinet/src/ui/mod.rs) into the canonical design system and reusable component suite for both **TdRace** and any future arcade titles built on the shared engine. It formalizes gamepad/keyboard parity as a non-negotiable core contract, defines the reusable component library across 10 functional archetypes, and provides surgical migration blueprints for existing screens.

---

## 🗺️ User Flow & Interface Design

### 1. Core Architectural Principles: Gamepad & Keyboard First

Every component and layout container defined under `cabinet::ui` must satisfy the following platform invariants:

```mermaid
flowchart TD
    subgraph InputLayer ["1. Unified Input Layer"]
        GP[Gamepad D-Pad / Stick]
        KB[Keyboard Arrows / WASD]
        SC[Global Shortcuts: Bumpers LB/RB, Q/E, 1-8]
        MS[Mouse Pointer & Clicks]
    end

    subgraph NavEngine ["2. Orthogonal 2D Navigation Engine"]
        Focus[Current Focus Container]
        Exit[Boundary Exit Protocol: ExitTop, ExitBottom, ExitLeft, ExitRight]
        Handoff[Container Focus Handoff]
    end

    subgraph ComponentSuite ["3. Reusable Platform Components"]
        Layout[HStack / VStack / CardGrid]
        Nav[FilterBar / Accordion]
        Data[DataTable / LeaderboardTable]
        Input[TextInputWidget / SwatchPicker]
        Feedback[ToastOverlay / HeroActionButton / ChecklistModal]
    end

    subgraph Presentation ["4. Unified Presentation & Feedback"]
        Gold[Palette::NEON_GOLD 2.4px Focus Border]
        Chevrons[Directional Chevrons: ► ACTIVE ◄]
        Audio[Tactile Audio Cues: Blip / Confirm / Cancel]
    end

    InputLayer --> NavEngine
    NavEngine --> ComponentSuite
    ComponentSuite --> Presentation
```

1. **Orthogonal 2D Traversal**:
   - `Up` / `Down` strictly navigates rows, vertical card lists, or container hierarchy (parent to child).
   - `Left` / `Right` strictly navigates horizontal tabs, columns, pill segments, or numeric steppers.
   - Analog stick input passes through a deadzone filter (`0.45`) with edge-trigger debouncing to prevent high-frequency frame skipping.
2. **Boundary Exit Protocol**:
   - Component classes do not hardcode assumptions about surrounding screens. Instead, when navigation reaches an extremity (e.g., pressing `Up` from index `0` of a card list), the component emits an explicit signal ([`NavBoundaryExit::ExitTop`](../crates/cabinet/src/ui/layout.rs), `ExitBottom`, `ExitLeft`, `ExitRight`). The parent screen catches this signal and transfers focus to the sibling container.
3. **Global Shortcuts**:
   - Bumper triggers (`LB` / `RB`), `Q` / `E`, and `[` / `]` are reserved for rapid horizontal tab or category cycling regardless of where deep child focus currently rests.
   - Direct numerical keys (`1`..=`8`) provide instantaneous index jumping.
4. **Visual Focus Standards**:
   - Focused elements must always render with a high-contrast accent border using `Palette::NEON_GOLD`, a minimum border thickness of `2.4px`, and directional chevrons (`► ... ◄`).
5. **Universal Tactile Audio Feedback**:
   - Focus transitions trigger `CabinetAudioSink::play_ui_blip()`.
   - Confirmations trigger `play_ui_confirm()`.
   - Cancellations or backwards escapes trigger `play_ui_cancel()`.

---

### 2. Standardized Component Architecture Catalog

The platform suite provides 10 reusable component building blocks:

| Component | Responsibility | Primary TdRace Usage |
| :--- | :--- | :--- |
| **`HStack` / `VStack`** | 1D linear spatial layout with auto-spacing & boundary detection | Circuit list, vertical settings drawers, toolbars |
| **`FilterBar`** | Segmented horizontal tab/pill filter with keyboard & bumper cycling | Circuit categories, catalog switch, vehicle classes |
| **`Accordion<T>`** | Collapsible vertical drawer hierarchy with animated expand/collapse | Career tier championships, race setup accordions |
| **`CardGrid<T>`** | 2D matrix layout container with column wrapping and 2D navigation | Vehicle showroom, trophy cabinet, track cards |
| **`DataTable<T>`** | Multi-column sortable table with rank badges and player highlight | Race results, season standings, Hall of Fame |
| **`TextInputWidget`** | Virtual/physical keyboard input with cursor blink and validation | Callsign editor, track renaming, LAN IP entry |
| **`ToastOverlay`** | Non-blocking transient notification stack with time decay | Lap records, driving assist toggles, unlock alerts |
| **`ChecklistModal<T>`**| Multi-select modal dialog with toggle checkboxes & batch actions | Track category promotion, series rule toggles |
| **`SwatchPicker`** | Horizontal color palette ribbon with directional selection | Livery primary/secondary colors, driver suits |
| **`HeroActionButton` / `ScreenFooter`** | Standardized bottom CTA button with contextual controller prompts | Menu footer, start race CTA, editor export action |

---

### 3. Application Migration Blueprints (TdRace Screens)

```mermaid
flowchart LR
    subgraph CircuitSelector ["Circuit Selector (menu.rs)"]
        CSF[FilterBar: 8 Categories]
        CST[FilterBar: Official / Custom]
        CSL[VStack / CardList: Circuits]
        CSF -->|ExitDown| CST
        CST -->|ExitDown| CSL
        CSL -->|ExitTop| CST
        CST -->|ExitUp| CSF
    end

    subgraph Showroom ["Garage Showroom (garage.rs)"]
        GCF[FilterBar: Motorsport Classes]
        GCG[CardGrid: Vehicle Roster]
        GCS[SwatchPicker: Liveries]
        GCD[SpecDossierCard: Car Specs]
        GCF -->|ExitDown| GCG
        GCG -->|ExitRight| GCS
    end

    subgraph TrackManager ["Track Manager (track_manager_ui.rs)"]
        TMF[FilterBar: Categories]
        TMG[CardGrid: Saved Circuits]
        TMI[TextInputWidget: Search / Rename]
        TMC[ChecklistModal: Tags]
    end

    subgraph CareerSelect ["Career Select (career_select.rs)"]
        CAC[Accordion: Tier Championship Drawers]
        CAD[SpecDossierCard: Calendar & Rewards]
    end

    subgraph StartingGrid ["Starting Grid (starting_grid.rs)"]
        SGS[Accordion: Setup Drawers]
        SGP[NumberStepper: Laps & Bots]
        SGH[HeroActionButton: Start Race]
    end

    subgraph ResultsScreen ["Results & Standings (menu.rs)"]
        RST[DataTable: Post-Race Roster]
    end
```

#### Circuit Selector (`menu.rs`)
- **Categories**: Migrated from hard-coded pill rendering loop to `FilterBar` with `FilterBarStyle::Pills`. The `All` filter is permanently excluded, strictly enforcing the 8 motorsport categories: `Classic`, `Rally`, `Kart`, `Gt`, `Nascar`, `ExtremeOffroad`, `Autocross`, `Custom`.
- **Catalog Tabs**: Migrated to `FilterBar` (`OFFICIAL` vs `CUSTOM`).
- **Circuits**: Virtualized via `VStack`.
- **Focus Hierarchy**: Seamless 2D traversal:
  - `CategoryFilter` $\xleftrightarrow{\text{Down / Up}}$ `CatalogFilter` $\xleftrightarrow{\text{Down / Up}}$ `LeftTracks`.

#### Fleet Gallery & Showroom (`garage.rs`)
- **Current State**: Vehicle class selector tabs drawn using custom glass boxes; vehicle tiles manually laid out with hardcoded x/y loops.
- **Target**:
  - Class tabs converted to `FilterBar` with `LB` / `RB` bumper cycling.
  - Vehicle roster converted to `CardGrid<VehicleDef>` with 2D orthogonal navigation (`Up/Down/Left/Right`).
  - Paint selection converted to `SwatchPicker`.
  - Vehicle telemetry panel converted to `SpecDossierCard`.

#### Track Manager & CAD Studio (`track_manager_ui.rs`)
- **Current State**: Uses custom chip drawing loops around line 440 with manual mouse hit-testing and no keyboard/gamepad focus support for filter switching.
- **Target**:
  - Replace category chips with `FilterBar`.
  - Replace circuit draft cards with `CardGrid`.
  - Standardize circuit search and rename dialogues with `TextInputWidget`.
  - Adopt `ChecklistModal` for bulk category assignment.

#### Career Championship Selector (`career_select.rs`)
- **Current State**: Lines 553–589 manually compute collapsed (52px) vs expanded (148px) heights and viewport scrolling.
- **Target**: Convert directly to `Accordion<ChampionshipTierData>` with single-expand mode, automatic viewport centering, and `SpecDossierCard` for prize/circuit preview.

#### Starting Grid & Race Setup (`starting_grid.rs`)
- **Current State**: Manual state switching between Setup drawer, Roster slots, and numerical parameter adjustments.
- **Target**: Adopt `Accordion` for tuning parameter drawers, `NumberStepper` for lap/bot counts, and `HeroActionButton` + `ScreenFooter` for launch control.

#### Race Results, Season Standings & Hall of Fame (`menu.rs`, `hall_of_fame.rs`)
- **Current State**: Manual text rendering loops with disparate column spacing and inconsistent rank badge colors.
- **Target**: Replace with `DataTable<RaceParticipantResult>` featuring rank medals (Gold, Silver, Bronze), highlighted player row, and column sorting.

#### Player Profile & Dossier (`profile_ui.rs`)
- **Current State**: Ad-hoc string cursor management for callsign editing; hardcoded trophy shelf rendering.
- **Target**: `TextInputWidget` for callsign entry, `SwatchPicker` for helmet/suit themes, and `CardGrid` for trophy badges.

#### In-Race HUD & System Alerts (`hud.rs`)
- **Current State**: Hardcoded flash timers for lap records and personal best notifications.
- **Target**: `ToastOverlay` for non-intrusive queued notifications with severity styling (Info, Record, Warning).

---

## ⚙️ Backend Models & API Endpoints

### 1. Geometry & Layout Containers (`cabinet::ui::layout`)

```rust
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavBoundaryExit {
    ExitLeft,
    ExitRight,
    ExitTop,
    ExitBottom,
}

pub struct HStack {
    pub bounds: LayoutRect,
    pub count: usize,
    pub gap: f32,
    pub custom_widths: Option<Vec<f32>>,
}

pub struct VStack {
    pub base_x: f32,
    pub base_y: f32,
    pub width: f32,
    pub item_height: f32,
    pub gap: f32,
}
```

### 2. Segmented Filter Bar Model (`cabinet::ui::filter_bar`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterBarStyle {
    #[default]
    Pills,
    Shelf,
}

pub struct FilterItem {
    pub id: String,
    pub label: String,
    pub count: Option<usize>,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterBarAction {
    None,
    Changed(usize),
    Confirmed(usize),
    ExitUp,
    ExitDown,
}

pub struct FilterBar {
    pub style: FilterBarStyle,
    pub items: Vec<FilterItem>,
    pub selected_idx: usize,
    pub wrap: bool,
}
```

### 3. Collapsible Drawer Accordion Model (`cabinet::ui::accordion`)

```rust
pub struct AccordionItem<T> {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub badge: Option<String>,
    pub badge_color: Color,
    pub is_expanded: bool,
    pub height_collapsed: f32,
    pub height_expanded: f32,
    pub current_anim_height: f32,
    pub data: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccordionNavAction {
    None,
    Changed(usize),
    Toggled(usize, bool),
    Confirmed(usize),
    ExitTop,
    ExitBottom,
}

pub struct Accordion<T> {
    pub items: Vec<AccordionItem<T>>,
    pub selected_idx: usize,
    pub multi_expand: bool,
    pub anim_speed: f32,
    pub viewport_offset_y: f32,
}
```

### 4. 2D Matrix Grid Container (`cabinet::ui::card_grid`)

```rust
pub struct CardGridItem<T> {
    pub id: String,
    pub data: T,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardGridAction {
    None,
    Selected(usize),
    Confirmed(usize),
    Exit(NavBoundaryExit),
}

pub struct CardGrid<T> {
    pub items: Vec<CardGridItem<T>>,
    pub columns: usize,
    pub card_width: f32,
    pub card_height: f32,
    pub gap_x: f32,
    pub gap_y: f32,
    pub selected_idx: usize,
    pub wrap_navigation: bool,
}
```

### 5. Multi-Column Data & Leaderboard Table (`cabinet::ui::data_table`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlign {
    Left,
    Center,
    Right,
}

pub struct DataColumn<T> {
    pub id: String,
    pub header: String,
    pub width: f32,
    pub align: ColumnAlign,
    pub extractor: fn(&T) -> String,
}

pub struct DataRow<T> {
    pub id: String,
    pub is_player: bool,
    pub rank: Option<usize>,
    pub data: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAction {
    None,
    RowSelected(usize),
    RowConfirmed(usize),
    SortChanged(usize),
    ExitTop,
    ExitBottom,
}

pub struct DataTable<T> {
    pub columns: Vec<DataColumn<T>>,
    pub rows: Vec<DataRow<T>>,
    pub selected_row: usize,
    pub sort_col_idx: Option<usize>,
    pub sort_ascending: bool,
}
```

### 6. Accessible Text Input Buffer (`cabinet::ui::text_input`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInputAction {
    None,
    TextChanged,
    Submitted,
    Cancelled,
    ExitUp,
    ExitDown,
}

pub struct TextInputWidget {
    pub text: String,
    pub cursor_pos: usize,
    pub max_chars: usize,
    pub placeholder: String,
    pub is_active: bool,
    pub blink_timer: f32,
    pub allowed_chars_regex: Option<String>,
}
```

### 7. Toast Notification Stack (`cabinet::ui::toast`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastSeverity {
    Info,
    Success,
    Warning,
    Record,
}

pub struct ToastItem {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub severity: ToastSeverity,
    pub duration_sec: f32,
    pub elapsed_sec: f32,
    pub alpha: f32,
}

pub struct ToastOverlay {
    pub active_toasts: Vec<ToastItem>,
    pub max_visible: usize,
}
```

### 8. Modal Multi-Select Checklist (`cabinet::ui::checklist_modal`)

```rust
pub struct ChecklistItem<T> {
    pub id: String,
    pub label: String,
    pub checked: bool,
    pub disabled: bool,
    pub data: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecklistAction {
    None,
    Toggled(usize, bool),
    Confirmed,
    Cancelled,
}

pub struct ChecklistModal<T> {
    pub title: String,
    pub items: Vec<ChecklistItem<T>>,
    pub focused_idx: usize,
    pub is_open: bool,
}
```

### 9. Color Swatch Picker (`cabinet::ui::swatch_picker`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwatchAction {
    None,
    Changed(usize),
    Confirmed(usize),
    ExitTop,
    ExitBottom,
}

pub struct SwatchPicker {
    pub colors: Vec<Color>,
    pub selected_idx: usize,
    pub swatch_size: f32,
    pub gap: f32,
}
```

### 10. Hero CTA & Controller Screen Footer (`cabinet::ui::screen_footer`)

```rust
pub struct FooterPrompt {
    pub input_label: String, // e.g. "[A]", "[ESC]", "[LB/RB]"
    pub action_name: String, // e.g. "Select", "Back", "Switch Tab"
}

pub struct ScreenFooter {
    pub prompts: Vec<FooterPrompt>,
    pub hero_button_label: Option<String>,
    pub hero_button_focused: bool,
}
```

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
- Command to run cabinet UI unit & integration tests:
  ```bash
  cargo test -p cabinet
  ```
- Command to run app modality flow tests:
  ```bash
  cargo test -p tdrace-app --test modality_flow_tests
  ```
- Command to run all preflight quality checks:
  ```bash
  make preflight
  ```

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: 2D Orthogonal Traversal in Circuit Selector**
  - [ ] **Given** the player is in the Circuit Selection menu (`GameState::Menu`) with focus on the first track (`menu_track_idx == 0`)
  - [ ] **When** the player presses `Up` on the keyboard or Gamepad D-pad
  - [ ] **Then** focus moves to the `CatalogFilter` tabs (`OFFICIAL` / `CUSTOM`) with a gold accent border and audio blip
  - [ ] **When** the player presses `Up` again
  - [ ] **Then** focus moves to the `CategoryFilter` pill bar with `► CATEGORY ◄` markers and gold accent border
  - [ ] **When** the player presses `Down` twice
  - [ ] **Then** focus gracefully returns through `CatalogFilter` back to `LeftTracks`

- **Scenario: Strict Category Isolation and Zero All Fallback**
  - [ ] **Given** the Circuit Selector category filter is active
  - [ ] **When** inspecting all selectable categories
  - [ ] **Then** exactly 8 categories are available (`Classic`, `Rally`, `Kart`, `Gt`, `Nascar`, `ExtremeOffroad`, `Autocross`, `Custom`)
  - [ ] **And** no `All` filter option exists
  - [ ] **And** selecting any category strictly presents only tracks registered to that motorsport discipline

- **Scenario: Directional Tab Selection in FilterBar**
  - [ ] **Given** focus is on the Catalog Filter bar
  - [ ] **When** the player presses `Right`
  - [ ] **Then** `Custom` circuits tab is selected
  - [ ] **When** the player presses `Left`
  - [ ] **Then** `Official` presets tab is selected

- **Scenario: Global Bumper and Direct Key Shortcuts**
  - [ ] **Given** focus is anywhere in the Circuit Selector (including inside the track cards)
  - [ ] **When** the player presses Gamepad `RB`, `E`, or `]`
  - [ ] **Then** the category cycles forward to the next motorsport discipline with wrap-around
  - [ ] **When** the player presses number key `4`
  - [ ] **Then** the category jumps immediately to `GT World Challenge`

- **Scenario: Accordion Drawer Expansion and Centering**
  - [ ] **Given** an `Accordion` component populated with championship tier cards in Career Mode
  - [ ] **When** the player navigates to an inactive card and presses `Enter`, `Space`, or Gamepad `A`
  - [ ] **Then** that card expands to its full details height while the previously expanded card collapses
  - [ ] **And** the viewport scrolling smoothly centers the expanded card

- **Scenario: 2D Matrix Navigation in CardGrid**
  - [ ] **Given** a `CardGrid` with 4 columns and 10 items
  - [ ] **When** focus is at item index `0` and the player presses `Right`
  - [ ] **Then** focus shifts to item index `1`
  - [ ] **When** the player presses `Down`
  - [ ] **Then** focus shifts to item index `5` (row 1, column 1)
  - [ ] **When** the player presses `Up` from index `1`
  - [ ] **Then** the grid emits `NavBoundaryExit::ExitTop` without index out of bounds

- **Scenario: DataTable Sorting and Player Highlighting**
  - [ ] **Given** a `DataTable` rendered for race results containing 12 participants
  - [ ] **When** the table is displayed
  - [ ] **Then** rows with rank `1`, `2`, and `3` display Gold, Silver, and Bronze badges respectively
  - [ ] **And** the human player row is highlighted with `Palette::NEON_GOLD` accent glow
  - [ ] **When** the player triggers sort on the lap time column
  - [ ] **Then** the table reorders rows deterministically while keeping the player selection visible

- **Scenario: TextInputWidget Editing and Boundary Signals**
  - [ ] **Given** an active `TextInputWidget` containing text `"Thunder"`
  - [ ] **When** the user types characters `"bolt"`
  - [ ] **Then** the text updates to `"Thunderbolt"` and cursor advances to position 11
  - [ ] **When** the user presses `Backspace` 4 times
  - [ ] **Then** the text reverts to `"Thunder"`
  - [ ] **When** the user presses Gamepad D-pad `Up`
  - [ ] **Then** `TextInputAction::ExitUp` is emitted to hand off focus to the upper form field

- **Scenario: High-Contrast Focus Visuals and Audio Feedback**
  - [ ] **Given** any component rendered via `cabinet::ui`
  - [ ] **When** the component receives focus
  - [ ] **Then** it renders a distinct `Palette::NEON_GOLD` border with thickness $\ge 2.4\text{px}$
  - [ ] **And** a tactile audio blip (`UiMove`) is triggered through `CabinetAudioSink`

---

## 🔗 Traceability & Codebase Mapping

### Created / Modified Platform Files
- `[x]` [`crates/cabinet/src/ui/layout.rs`](../crates/cabinet/src/ui/layout.rs) — Implements `LayoutRect`, `HStack`, `VStack`, and `NavBoundaryExit`.
- `[x]` [`crates/cabinet/src/ui/filter_bar.rs`](../crates/cabinet/src/ui/filter_bar.rs) — Implements `FilterBar`, `FilterItem`, `FilterBarStyle`, and `FilterBarAction`.
- `[x]` [`crates/cabinet/src/ui/accordion.rs`](../crates/cabinet/src/ui/accordion.rs) — Implements `Accordion`, `AccordionItem`, and `AccordionNavAction`.
- `[x]` [`crates/cabinet/src/ui/card_grid.rs`](../crates/cabinet/src/ui/card_grid.rs) — Implements `CardGrid`, `CardGridItem`, and 2D matrix navigation.
- `[x]` [`crates/cabinet/src/ui/data_table.rs`](../crates/cabinet/src/ui/data_table.rs) — Implements `DataTable`, `DataColumn`, rank badges, and row selection.
- `[x]` [`crates/cabinet/src/ui/text_input.rs`](../crates/cabinet/src/ui/text_input.rs) — Implements `TextInputWidget`, cursor blinking, and input sanitization.
- `[x]` [`crates/cabinet/src/ui/toast.rs`](../crates/cabinet/src/ui/toast.rs) — Implements `ToastOverlay`, severity styling, and decay queue.
- `[x]` [`crates/cabinet/src/ui/checklist_modal.rs`](../crates/cabinet/src/ui/checklist_modal.rs) — Implements `ChecklistModal` multi-select dialog.
- `[x]` [`crates/cabinet/src/ui/swatch_picker.rs`](../crates/cabinet/src/ui/swatch_picker.rs) — Implements `SwatchPicker` palette ribbon.
- `[x]` [`crates/cabinet/src/ui/screen_footer.rs`](../crates/cabinet/src/ui/screen_footer.rs) — Implements `HeroActionButton` and `ScreenFooter`.
- `[x]` [`crates/cabinet/src/ui/mod.rs`](../crates/cabinet/src/ui/mod.rs) — Public re-exports for the platform UI module.
- `[x]` [`crates/cabinet/src/lib.rs`](../crates/cabinet/src/lib.rs) — Top-level crate re-exports.

### Application Integration Files
- `[x]` [`crates/tdrace-app/src/game/mod.rs`](../crates/tdrace-app/src/game/mod.rs) — `update_menu` 2D focus traversal, category cycling, directional tabs, and strict filtering.
- `[x]` [`crates/tdrace-app/src/ui/menu.rs`](../crates/tdrace-app/src/ui/menu.rs) — `render_track_select_menu` focus highlights, gold borders, and contextual footer guidance.
- `[x]` [`crates/tdrace-app/tests/modality_flow_tests.rs`](../crates/tdrace-app/tests/modality_flow_tests.rs) — Automated verification of 2D focus navigation, bumper cycling, and strict category circuit isolation.

### Target Migration Files
- `[ ]` [`crates/tdrace-app/src/ui/track_manager_ui.rs`](../crates/tdrace-app/src/ui/track_manager_ui.rs) — Migrate category chips to `FilterBar`, circuit cards to `CardGrid`, and search to `TextInputWidget`.
- `[ ]` [`crates/tdrace-app/src/ui/garage.rs`](../crates/tdrace-app/src/ui/garage.rs) — Migrate vehicle category tabs to `FilterBar`, vehicle roster to `CardGrid`, liveries to `SwatchPicker`.
- `[ ]` [`crates/tdrace-app/src/ui/career_select.rs`](../crates/tdrace-app/src/ui/career_select.rs) — Migrate championship tier cards to `Accordion`.
- `[ ]` [`crates/tdrace-app/src/ui/starting_grid.rs`](../crates/tdrace-app/src/ui/starting_grid.rs) — Migrate setup drawers to `Accordion` and race launch to `HeroActionButton`.
- `[ ]` [`crates/tdrace-app/src/ui/series_editor.rs`](../crates/tdrace-app/src/ui/series_editor.rs) — Migrate rule tabs to `FilterBar` and points/laps to `NumberStepper`.
- `[ ]` [`crates/tdrace-app/src/ui/profile_ui.rs`](../crates/tdrace-app/src/ui/profile_ui.rs) — Migrate callsign input to `TextInputWidget` and trophy shelf to `CardGrid`.
