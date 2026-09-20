---
type: Architecture Spec
title: "UI Terminology & Screen Architecture Guide"
description: "Formal reference for graphical UI component terminology, Cabinet platform definitions, and canonical screen names across the TdRace motorsport platform."
status: active
category: engineering
tags: [ui, terminology, screens, cabinet, architecture, okf]
---

# UI Terminology & Screen Architecture Guide 🏎️📐

This document establishes an authoritative vocabulary for the graphical user interface across **TdRace** and the underlying **Cabinet** arcade engine. It resolves ambiguity between platform-level primitives and game-specific screens, categorizes all interactive widgets, and provides a catalog of every screen, view, and modal currently implemented in the codebase.

---

## 1. Graphical UI Component Hierarchy & Terminology

The TdRace interface is structured into six hierarchical tiers:

```
Tier 1: Root Screens & Stacks (CabinetScreen, ScreenStack, GameState)
  └── Tier 2: Screen Sub-Views & Tabs (FinishedScreenView, GarageViewMode, TrackManagerTab)
        └── Tier 3: Modals & Dialog Overlays (UniversalConfirmModal, ArcadeSettingsModal)
              └── Tier 4: Layout Columns & Panels (NavGrid2D, LeftPanel, RightPanel)
                    └── Tier 5: Cards & Containers (UiScaler::draw_glass_card, draw_button_card)
                          └── Tier 6: Atomic Widgets & Controls (SliderWidget, DropdownWidget, draw_action_button)
```

### 1.1. Definitions

| Term | Architectural Scope | Definition & Behavior | Example in TdRace |
| :--- | :--- | :--- | :--- |
| **Screen** | Root Viewport State | A full-screen primary application state that completely owns the viewport and game loop lifecycle. Transitioning between screens typically unloads or swaps the active state context. | [`GameState::Menu`](../../crates/tdrace-app/src/game/mod.rs), [`GameState::Garage`](../../crates/tdrace-app/src/game/mod.rs), [`GameState::StartingGrid`](../../crates/tdrace-app/src/game/mod.rs) |
| **View (Sub-View)** | Intra-Screen State | A distinct visual presentation or functional mode *within* a single screen that alters displayed panels without triggering a top-level screen transition. | [`FinishedScreenView::Results`](../../crates/tdrace-app/src/game/mod.rs), [`FinishedScreenView::Statistics`](../../crates/tdrace-app/src/game/mod.rs), [`GarageViewMode::SideProfile`](../../crates/tdrace-app/src/ui/garage.rs) |
| **Modal (Dialog)** | Transient Overlay | A focused window layered on top of an active screen that dims background elements and captures exclusive input until confirmed, dismissed, or canceled. | [`UniversalConfirmModal`](../../crates/cabinet/src/state/confirm.rs), [`ArcadeSettingsModal`](../../crates/cabinet/src/state/settings.rs), `UnsavedSettingsModal` |
| **Overlay / HUD** | Layered Non-Modal | A non-blocking graphical layer drawn on top of 2D/3D gameplay or editing canvases to display real-time gauges, timers, or curve warnings without stealing keyboard/stick steering. | [`render_hud`](../../crates/tdrace-app/src/ui/hud.rs), [`render_curve_indicator`](../../crates/tdrace-app/src/ui/curve_indicator.rs) |
| **Panel (Column)** | Layout Division | A major structural container dividing a screen horizontally or vertically. Multi-panel layouts rely on 2D orthogonal navigation (`NavGrid2D`). | Left Setup Panel & Right Starting Grid Roster in `StartingGrid`; 4 Modality columns in `ModalitySelect`. |
| **Card (Tile)** | Visual Container | A self-contained rounded glassmorphic card grouping related telemetry, specs, or settings together. | Setup Cards 0–3 in `StartingGrid`; Track Preview Card in `Menu`. |
| **Menu & Menu Entry** | Navigable List | A navigable sequential list of selectable actions, tracks, or options. A *Menu Entry* is an individual selectable row or button inside the menu. | Circuit list rows in `Menu`; Module entry cards in `ModuleSelect`. |
| **Widget (Control)** | Atomic UI Component | A standardized interactive control that accepts direct user input (stepper, slider, dropdown, button, tab bar). | [`SliderWidget`](../../crates/cabinet/src/ui/widgets.rs), [`DropdownWidget`](../../crates/cabinet/src/ui/widgets.rs), [`draw_action_button`](../../crates/cabinet/src/ui/widgets.rs) |
| **Badge / Chip** | Static/Status Tag | A small pill-shaped indicator displaying status, difficulty, discipline, or tags with distinct neon coloring. | [`draw_chip`](../../crates/cabinet/src/ui/widgets.rs) (`PREDEFINED CAR`, `CUSTOM CIRCUIT`) |
| **Stat Bar** | Visual Progress/Metric | A horizontal bar displaying vehicle capability ratings, telemetry values, or audio volumes. | [`draw_stat_bar`](../../crates/cabinet/src/ui/widgets.rs) (`SPEED`, `ACCEL`, `GRIP`, `DRIFT`) |
| **Header / Banner** | Top Global Strip | The top bar presenting platform branding, active driver profile, discipline badges, and category breadcrumbs. | [`render_profile_badge`](../../crates/tdrace-app/src/ui/profile_ui.rs) |
| **Footer / Prompt Bar** | Bottom Legend Strip | Contextual bottom bar displaying available gamepad buttons and keyboard shortcut hints. | [`starting_grid_footer_prompt`](../../crates/tdrace-app/src/ui/starting_grid.rs) |

---

## 2. Platform Architecture: Cabinet vs. TdRace App

A frequent architectural question is: **Which components are provided by the `cabinet` platform, and which belong to `tdrace-app`?**

### 2.1. Cabinet Platform (`crates/cabinet`)

The `cabinet` platform crate is the reusable, arcade-grade foundation designed for motorsport, retro, and arcade games. It contains:

1. **State Machine & Lifecycle**:
   - [`CabinetScreen`](../../crates/cabinet/src/state/stack.rs): Core trait for pushdown-automata screens and modals (`name()`, `update()`, `draw()`, `is_transparent()`).
   - [`ScreenStack`](../../crates/cabinet/src/state/stack.rs): Screen stack manager supporting modal overlays, screen popping, and iris/fade transitions.
   - [`ScreenAction`](../../crates/cabinet/src/state/stack.rs): Transitions (`None`, `Pop`, `PopWith`, `Push`, `PushWith`, `Switch`, `SwitchWith`, `Quit`).
   - [`CabinetContext`](../../crates/cabinet/src/state/stack.rs): Frame execution context bundling `UiScaler`, `Fonts`, `CabinetTheme`, `GamepadSnapshot`, `dt`, and audio sinks.
2. **Navigation & Input Routing**:
   - [`NavGrid2D`](../../crates/cabinet/src/input/nav2d.rs): 2D orthogonal navigation router enforcing standard arcade controls (horizontal = panels/columns/tabs; vertical = items/rows).
3. **Universal Pre-Built Modals**:
   - [`UniversalConfirmModal`](../../crates/cabinet/src/state/confirm.rs): Generic confirmation dialog (`[YES / NO]`).
   - [`UniversalPauseModal`](../../crates/cabinet/src/state/pause.rs): Reusable arcade pause modal.
   - [`ArcadeSettingsModal`](../../crates/cabinet/src/state/settings.rs): Complete settings modal for Audio, Controls, Display, and Gameplay options.
   - [`ProfileSelectModal`](../../crates/cabinet/src/state/profile_select.rs): Player driver selector popup.
   - [`LeaderboardModal`](../../crates/cabinet/src/state/leaderboard.rs): Hall of Fame leaderboard modal.
4. **Standard Interactive Widgets (`crates/cabinet/src/ui/widgets.rs`)**:
   - [`draw_action_button`](../../crates/cabinet/src/ui/widgets.rs): Interactive button with title, shortcut subtitle, hover, and focus glow.
   - [`draw_chip`](../../crates/cabinet/src/ui/widgets.rs): Rounded metadata badge tag.
   - [`draw_stat_bar`](../../crates/cabinet/src/ui/widgets.rs): Labeled horizontal progress and stat metric bar.
   - [`SliderWidget`](../../crates/cabinet/src/ui/widgets.rs) & [`draw_slider`](../../crates/cabinet/src/ui/widgets.rs): Slider with range, step, unit suffix, mouse drag, and gamepad stepping.
   - [`DropdownWidget`](../../crates/cabinet/src/ui/widgets.rs) & [`draw_dropdown`](../../crates/cabinet/src/ui/widgets.rs): Collapsible selection dropdown.
   - [`draw_stepper`](../../crates/cabinet/src/ui/widgets.rs): Horizontal numeric/option stepper with `<` / `>` buttons.
   - [`TabBar`](../../crates/cabinet/src/ui/widgets.rs) & [`draw_tab_bar`](../../crates/cabinet/src/ui/widgets.rs): Horizontal tab navigation bar.
5. **Design System & Scaling**:
   - [`UiScaler`](../../crates/cabinet/src/ui/scaler.rs): Virtual resolution scaler (`1920x1080` base) providing responsive spacing `s()`, font size `font_s()`, `draw_glass_card()`, and `draw_button_card()`.
   - [`CabinetTheme`](../../crates/cabinet/src/ui/theme.rs) & [`Palette`](../../crates/cabinet/src/ui/theme.rs): Color tokens (neon accents, glass cards, text variants).
   - [`Fonts`](../../crates/cabinet/src/ui/font.rs): Typography loader and text rendering routines.

### 2.2. Game Application Layer (`crates/tdrace-app`)

The `tdrace-app` crate implements motorsport domain logic and concrete game screens by consuming Cabinet primitives:

- The top-level [`GameState`](../../crates/tdrace-app/src/game/mod.rs) enum representing game states.
- Domain-specific multi-view screen layouts (`StartingGrid`, `Menu`, `Garage`, `TrackManager`, `Finished`).
- Physics simulation and telemetry HUD overlays (`render_hud`, `render_curve_indicator`).
- Vector track spline preview and rendering (`render_track_detailed_preview`, `render_track_thumbnail`).
- Track CAD Editor toolbars and Bezier node canvas (`GameState::TrackEditor`).

---

## 3. Canonical Names for TdRace Screens

Below is the authoritative directory of all screens currently implemented in `tdrace`:

| # | Canonical Screen Name | Enum / State Struct | Category | Primary Purpose |
| :-: | :--- | :--- | :--- | :--- |
| **1** | **Grand Hub** | [`GameState::ModuleSelect`](../../crates/tdrace-app/src/game/mod.rs) | Hub | Primary platform launchpad. Selects motorsport discipline (Classic Arcade, Rallycross World Cup, Karting World Cup, GT World Challenge, NASCAR Cup Series). |
| **2** | **Race Modality Selection** | [`GameState::ModalitySelect`](../../crates/tdrace-app/src/game/mod.rs) | Hub | Categorizes racing format: Single Player (Quick, Custom, Career, Time Trial, Free Ride), Multiplayer (Split Screen, LAN, Cloud), Vehicle Roster, and Circuit Catalogue. |
| **3** | **Track & Setup Menu** (Circuit Selection) | [`GameState::Menu`](../../crates/tdrace-app/src/game/mod.rs) | Menu | Circuit selection catalog (Presets vs Custom), 2D track vector map preview, telemetry analysis, and car specifications overview. |
| **4** | **Starting Grid & Roster Setup** | [`GameState::StartingGrid`](../../crates/tdrace-app/src/game/mod.rs) | Setup | Pre-race setup screen: Left panel controls game mode (Standard, Career, Experimental, Split Screen, Time Trial, Free Ride), vehicle model, and AI bot count; Right panel displays starting grid slots (P1–P8) or shadow car records. |
| **5** | **Race Countdown** | [`GameState::Countdown(f32)`](../../crates/tdrace-app/src/game/mod.rs) | Simulation | Cinematic 3-2-1 start countdown with interactive engine revving, starting lights, and camera alignment. |
| **6** | **Race Simulation & HUD** | [`GameState::Racing`](../../crates/tdrace-app/src/game/mod.rs) | Simulation | Active 120 Hz fixed-step physical simulation featuring camera tracking, telemetry HUD, speedo, tachometer, lap delta, and curve warnings. |
| **7** | **Race Paused Overlay** | [`GameState::Paused`](../../crates/tdrace-app/src/game/mod.rs) | Overlay | Pauses simulation; provides options to resume, restart, adjust driving assists, open settings modal, or abort to menu. |
| **8** | **Race Results & Post-Race Hub** | [`GameState::Finished`](../../crates/tdrace-app/src/game/mod.rs) | Post-Race | Post-race session screen containing three navigable sub-views (Podium Results, Hall of Fame, Detailed Race Telemetry). |
| **9** | **Championship Standings** | [`GameState::ChampionshipStandings`](../../crates/tdrace-app/src/game/mod.rs) | Tournament | Multi-round championship tournament progress, driver season points table, and upcoming calendar schedule. |
| **10** | **Controls & Assists Help** | [`GameState::ControlsHelp(bool)`](../../crates/tdrace-app/src/game/mod.rs) | Reference | Controller and keyboard mapping diagrams and driving aids switcher (`Arcade`, `Sport`, `Pro`). |
| **11** | **Driver Dossier Cards** | [`GameState::DriverCards(DriverCardsOrigin)`](../../crates/tdrace-app/src/game/mod.rs) | Dossier | Full-screen driver profiles displaying character vector portrait, nationality flag, biography, pedigree, and AI behavior parameters. |
| **12** | **Profile Manager** | [`GameState::ProfileManager`](../../crates/tdrace-app/src/game/mod.rs) | Profile | Multi-profile management dashboard showing career statistics (wins, podiums, win rate, best laps per circuit, stunt points). |
| **13** | **Profile Editor & Livery Customizer** | [`GameState::ProfileCreate`](../../crates/tdrace-app/src/game/mod.rs) | Profile | Driver creation/editing form for callsign alias, nationality flag, and vehicle livery color scheme. |
| **14** | **Circuit Hub & Workshop** (Track Manager) | [`GameState::TrackManager`](../../crates/tdrace-app/src/game/mod.rs) | Workshop | Motorsport discipline track catalog manager. Allows cloning, creating, editing metadata, and categorizing custom circuits. |
| **15** | **CAD Spline Studio** (Track Editor) | [`GameState::TrackEditor`](../../crates/tdrace-app/src/game/mod.rs) | Editor | In-game vector track designer and CAD spline creation suite with Bezier curves, surface zoning, elevation, and jump ramp placement. |
| **16** | **Garage Showroom** | [`GameState::Garage(GarageOrigin)`](../../crates/tdrace-app/src/game/mod.rs) | Showroom | Interactive vehicle showroom featuring dual-view rendering (2D lateral profile and 360° top-down turntable), historical dossier, engineering specs, performance radar, and throttle rev audio sampler. |

---

## 4. Screen Sub-Views Directory

Screens that contain multiple distinct visual presentations without modifying the parent `GameState`:

### 4.1. Post-Race Sub-Views (`GameState::Finished`)
- **`FinishedScreenView::Results` (Podium Results)**: The default post-race screen. Shows finishing order, driver names, cars, total times, best lap, leader deltas, and career XP badges.
- **`FinishedScreenView::HallOfFame` (Leaderboard)**: Circuit all-time leaderboard displaying top record times and initials.
- **`FinishedScreenView::Statistics` (Race Telemetry & Stunts)**: Granular analysis breaking down individual lap sector splits, personal best deltas, top speed ($km/h$ & $m/s$), total drift points, drift counter, air time, jump count, and stunt combos. Toggled via `Tab` / `S` / Gamepad `X`.

### 4.2. Garage Showroom Sub-Views (`GameState::Garage`)
- **`GarageViewMode::SideProfile` (2D Lateral Profile)**: Detailed lateral vector model displaying spoke alloy rims, brake calipers, sponsor liveries, and floor mirror reflection.
- **`GarageViewMode::TopDownTurntable` (360° Turntable)**: Dynamic rotating overhead view showing the exact car model and collision footprint used in top-down racing. Toggled via `Tab` / Gamepad `X`.

### 4.3. Track Manager Discipline Tabs (`GameState::TrackManager`)
- **`TrackManagerTab::Classic`**: Standard arcade road and oval circuits.
- **`TrackManagerTab::Rally`**: Mixed-surface dirt, sand, and off-road stages.
- **`TrackManagerTab::Karting`**: High-density sprint circuits and technical indoor arenas.
- **`TrackManagerTab::Formula1`**: FIA Grade 1 Grand Prix road courses and DRS straights.

---

## 5. Modal Overlays Directory

| Modal Name | Host Screen(s) | Trigger | Dismiss / Actions | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **Exit Application Dialog** | `ModuleSelect` | `Escape` / Gamepad `B` | `Enter` / `Y` (Exit)<br>`Escape` / `N` (Cancel) | Confirms exiting the game executable. |
| **Arcade Settings Modal** | `ModuleSelect`, `Menu`, `Paused` | `X` (Hub), `O` (Menu), `O` / Gamepad `Y` (Paused) | `Save & Close`<br>`Restore Defaults`<br>`Escape` / `B` (Close) | Comprehensive settings overlay for Audio, Controls, Display, and Gameplay options. |
| **Unsaved Settings Confirmation** | `ArcadeSettingsModal` | `Escape` / `B` (when settings modified) | `S` (Save & Exit)<br>`Q` / `D` (Discard & Quit)<br>`Escape` (Cancel) | Prevents losing modified audio, display, or control preferences. |
| **Coming Soon Modal** | `ModalitySelect` | `Enter` on LAN / Cloud cards | `Escape` / `Enter` / `A` / `B` | Informs player that multiplayer format is under development. |
| **Track Metadata Editor** | `TrackManager` | `I` on custom track | `Enter` (Save)<br>`Escape` (Cancel) | Input modal for track title and description. |
| **Module Promotion Selector** | `TrackManager` | `P` / Gamepad `Y` | `Enter` (Apply)<br>`Escape` (Cancel) | Multi-select dialog assigning a custom track across motorsport modules. |
| **Delete Track Confirmation** | `TrackManager` | `Delete` / `Backspace` | `Y` (Confirm)<br>`N` / `Escape` (Cancel) | Deletes custom track JSON file from disk. |
| **Leaderboard Name Input** | `Finished` (Hall of Fame) | Automatic on new record | `Enter` (Submit)<br>`Escape` (Skip) | Records 3-letter driver callsign initials into SQLite database. |

---

## 6. Standard Arcade Navigation Conventions

All TdRace menus and screens strictly follow orthogonal 2D navigation rules enforced by `NavGrid2D`:

1. **Horizontal Axis (`Left` / `Right`, `A` / `D`, Gamepad `D-pad X`, `LB` / `RB`)**:
   - Switches **Panels** (e.g. Left Setup Panel ⇄ Right Roster Panel in `StartingGrid`).
   - Switches **Category Tabs** (e.g. `AUDIO` ⇄ `CONTROLS` ⇄ `DISPLAY` in `ArcadeSettingsModal`).
   - Toggles **Horizontal Filters** (e.g. `[PRESETS]` ⇄ `[CUSTOM]` in `Menu`).
   - Switches **Bottom Action Buttons** (e.g. `[SAVE & EXIT]` ⇄ `[QUIT & LOSE]`).
2. **Vertical Axis (`Up` / `Down`, `W` / `S`, Gamepad `D-pad Y`)**:
   - Moves **Cursor within the active panel/list** (e.g. scrolling circuits in `Menu`, browsing drivers in `StartingGrid`).
   - Navigates **Settings Rows** within a modal category.
3. **Primary Action (`Enter`, `Space`, Gamepad `A`)**:
   - Confirms selection, launches race, or opens highlighted menu entry.
4. **Secondary / Option Action (`Tab`, `D`, `X`, Gamepad `X` / `Y`)**:
   - Opens Driver Dossier, toggles Hall of Fame / Statistics, or toggles Garage view modes.
5. **Back / Cancel Action (`Escape`, Gamepad `B`)**:
   - Dismisses active modal, returns to previous screen, or opens pause menu.
6. **Direct Restart Action (`R`, Gamepad `Y`)**:
   - Exclusively reserved on post-race screens (`GameState::Finished`) to trigger immediate race session restarts.
