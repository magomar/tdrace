---
type: Feature Spec
template: feature
title: "Compact Track Studio Inspector with Unified Single and Multi-Selection Editing"
description: "Makes the Track Studio inspector fit on screen by replacing long button lists with compact controls (surface dropdown, segmented toggles, inline steppers) inside collapsible, clipped and scrollable sections; gives multi-selection the same controls as single selection with mixed-value display; fixes the inspector's undo-flood and sticky-slider defects; and gates every alternate layout on a human design review."
status: implemented
receipt: "docs/receipts/spec-086-receipt.md"
verified: { by: "human:mario", at: "2026-10-07T18:23:39Z", hash: "1c3015a696bf" }
created: 2026-10-04
generated: { by: agent/claude-opus-5-5, at: 2026-10-04T18:50:50Z }
---


# Feature Spec: Compact Track Studio Inspector with Unified Single and Multi-Selection Editing 🧰

The Track Studio inspector (`render_inspector` in [`crates/tdrace-app/src/editor/ui.rs`](../crates/tdrace-app/src/editor/ui.rs)) is a single ~1,960-line immediate-mode function. It draws long button lists into a fixed-height card with no clipping and no scrolling. Content runs off the bottom of the card for most selections, so controls are hidden under the status bar or off-screen while their hidden buttons still react to clicks. Multi-selection shows a different, reduced set of controls than single selection. This spec makes the inspector compact, makes it always fit inside its card, and gives single and multi-selection the same controls. It also fixes two interaction defects found during the analysis. Alternate layouts are validated by a human before they are built.

---

## 🎯 Problem Statement (Exploratory Analysis, 2026-10-04)

Analysis method: a code inventory of `render_inspector`, plus in-engine screenshots at 1920×1080 of every selection kind (a throwaway local hook seeded a surface zone, obstacles and a jump ramp; it was not committed). All heights below are in 720p reference pixels (`UiScaler` base 1280×720, so the overflow is the same at every 16:9 resolution). The content area of the card is about **584 px**.

### 1. Content does not fit and is not clipped
| Inspector view | Content height | Over budget | Lost controls |
|---|---|---|---|
| `Waypoint` | ≈ 688 | ≈ 104 | Water/Oil surface row (under status bar), Duplicate, Delete (off-screen) |
| `MultipleWaypoints` | ≈ 820 | ≈ 236 | 7 of 12 batch surfaces, Duplicate All, Delete All |
| `JumpRamp` | ≈ 637 | ≈ 53 | Duplicate (under status bar), Delete (off-screen) |
| `None` + Road Spline tool | ≈ 1136 | ≈ 550 | Entire global block (wall offset, off-track surface, category, grid) |
| `None` + Jump Ramp tool | ≈ 968 | ≈ 384 | Most of the global block |
| `None` + Road Split tool | ≈ 764 | ≈ 180 | Grid controls, Rebuild Geometry |
| `None` + other tools | ≈ 610 | ≈ 26 | Rebuild Geometry (under status bar) |
| `SurfaceZone` | ≈ 550 | — | fits, but 350 px is one 14-button material column |

- `render_inspector` ignores its height argument (`_h`). No `begin_clip_rect`, no scroll offset.
- `draw_ui_btn` hit-tests only its own rectangle, so a button drawn under the status bar still fires.

### 2. Long lists dominate the panel
Seven separate surface pickers exist, each with its own labels, order, item count and active colour:

| Picker | Items | Layout | Height | Active colour |
|---|---|---|---|---|
| Waypoint surface | 14 | 2 columns | 182 | gold |
| Multi-waypoint batch surface | 12 (no Water/Oil) | 1 column | 300 | none |
| Surface zone material | 14 (descriptive labels) | 1 column | 350 | cyan |
| Ramp material | 14 | 2 columns | 175 | cyan |
| Road Spline tool surface | 14 | 2 columns | 182 | gold |
| Jump Ramp tool surface | 14 | 2 columns | 182 | gold |
| Global off-track surface | 11 (Sheet Ice missing) | 2 columns | 156 | gold |

Other redundancy: wall type (3 buttons) repeated 4 times, banking rows (4 + 5 buttons) 3 times, wall distance 3 times, and grid slot count has three controls for one value (bar, counter, chips).

### 3. Multi-selection loses controls
- `MultipleWaypoints` has no sliders, no current values, no active highlight, no per-side curb control, no Water/Oil, and different clamp ranges (width 6..40 vs 4..30; wall distance 0..25 vs 0..20).
- `Selection::Multi` (any mix, and also several zones, ramps or obstacles of one kind) shows only counts, a reduced ramp block, and Duplicate/Delete. Selected waypoints inside a `Multi` get no waypoint controls, although every `batch_*` function in `tools.rs` already supports `Multi`.
- Multiple zones get no material or layer controls; multiple ramps lose the five bars, fit buttons, material and profile diagram.

### 4. Interaction defects in inspector controls
- **Undo flood:** `draw_bar_control` returns a value every frame while dragging, and each call site calls `record_undo`. `HistoryStack` holds 50 entries, so a one-second drag erases the whole undo history. `draw_counter` repeats every frame while held, with no delay.
- **Sticky slider:** `tools.selected_bar` is never cleared in production code. The mouse wheel over the canvas then zooms the camera *and* changes the last-clicked slider value. Bar ids are shared across entities, so the effect follows the user to the next selection.
- **No drag capture:** leaving the slider ends a drag; entering it with the button held starts one.

### 5. Consistency and discoverability
- Button heights 20/22/24/26/28, label fonts 11–13 px, row gaps 2/4/6 px, header styles "Road Width:", "BATCH WIDTH:", "Global Off-Track Surface".
- Labels overflow their buttons ("0m Flush", "FIT HEIGHT: 3.2m", "DUPLICATE OBSTACLE [Ctrl+D]").
- `GridSlot` shows an empty panel. The multi-ramp modal label says "0°–365°".
- No tooltips. Shortcuts such as B, Shift+B, [ / ], R, Ctrl+F/Ctrl+B are not shown anywhere in the inspector.

---

## 🗺️ User Flow & Interface Design

### 1. Design principles
1. **Fit first.** The inspector never draws outside its card. Every single-entity view fits fully expanded at 1280×720 without scrolling.
2. **One value, one control.** Each property has exactly one control. Presets live inside that control (dropdown entries or a compact chip row), not as extra rows.
3. **Same view for one or many.** A property shows the same control whether one or many entities are selected. When the selected values differ, the control shows a mixed state.
4. **Progressive disclosure.** Rare properties live in collapsible sections. Destructive and frequent actions live in a fixed footer.
5. **Consistency.** One surface catalogue, one row height, one gap, one accent colour for "active", one header style.

### 2. Target layout (default proposal — subject to human checkpoint HC-1)
```
┌ INSPECTOR ─────────────────────────┐
│ Waypoint #12            [3 selected]│  header: entity kind, name/index, count
├─────────────────────────────────────┤
│ ▾ ROAD                              │  collapsible section
│   Width     [-][■■■■■□□□ 13.0m][+]  │  inline stepper bar (drag / click-to-type)
│   Surface   [▣ Asphalt         ▾]   │  dropdown with colour swatch
│ ▾ BANKING                           │
│   Angle     [-][■■■□□□□□ +0.0°][+]  │
│   Presets   (0°)(10°)(18°)(22°)(±)  │  one chip row
│ ▾ EDGES                 L      R    │
│   Curb               [ on ][ off ]  │  segmented per-side toggles
│   Wall               [ on ][ on  ]  │
│   Wall dist  L [-][■■□ 1.5m][+]     │  per-side (closes tdrace-fnqg gap)
│              R [-][■■□ 1.5m][+]     │
│   Wall type  [Concrete|Steel|Tyres] │  3-segment control
│ ▸ ADVANCED                          │  collapsed by default
│                                 ▐▌  │  scrollbar only when needed
├─────────────────────────────────────┤
│ [ Duplicate  Ctrl+D ] [ Delete Del ]│  sticky footer, always visible
└─────────────────────────────────────┘
```

### 3. Control vocabulary (replaces today's button rows)
| Today | Replacement |
|---|---|
| 11–14-button surface grids and columns (×7) | One **surface dropdown**: closed = one row with colour swatch and name; open = scrollable list with swatches, max visible rows, keyboard Up/Down/Enter/Esc. One shared catalogue (`SurfaceType` order and display names). |
| Bar + separate −/+ rows + preset rows | **Inline stepper bar**: `[-] bar value [+]` in one row; presets as one chip row or as dropdown entries. |
| L/R curb and wall `draw_toggle` pairs with ON/OFF subtitle | **Segmented per-side toggle** (`L`/`R`) on one row per property. |
| Wall type 2 + 1 button rows | **3-segment control** on one row. |
| Grid slots: bar + counter + 6 chips | One stepper bar plus one preset chip row. |
| Car category 6-button grid | Dropdown. |
| Duplicate / Delete at the end of each branch | **Sticky footer**, shown for every non-empty selection. |

### 4. Multi-selection behaviour
- **Same kind** (several waypoints, zones, ramps, obstacles, checkpoints): the inspector shows the **same sections and controls** as for one entity of that kind. The header shows the count.
- **Mixed values:** a control whose selected values differ shows a mixed state: bars show `—` and no fill; dropdowns show `Mixed`; segmented toggles show no active segment.
- **Edit semantics:** setting an absolute value (drag, type, dropdown, segment, preset chip) applies that value to every selected entity of that kind. Step buttons `[-]`/`[+]` apply a relative delta to each entity and keep their differences.
- **Mixed kinds** (e.g. waypoints + zones + a ramp): presentation chosen at **HC-3** from two options built on the real engine:
  - **Option A — stacked sections per kind:** one collapsible group per kind ("Waypoints (3)", "Zones (2)"), each with that kind's normal sections.
  - **Option B — kind chips:** a chip row ("Waypoints 3 · Zones 2 · Ramps 1") selects which kind's controls are shown; edits apply only to that kind.
- **One range per property:** single and batch paths use the same clamp constants. The range is the one `validate_track` accepts; where today's single and batch ranges differ, the implementer records the chosen range and its source in the commit message.
- **One undo step per edit gesture**, for one or many entities.

### 5. Inspector interaction rules
- Body content is clipped to the card. Controls outside the visible body area do not receive input.
- When content is taller than the body, the mouse wheel over the inspector scrolls the body and a thin scrollbar appears. The scroll position resets when the selection kind changes.
- **Wheel focus rule (default, reviewable at HC-1):** the wheel scrolls the panel. The wheel changes a slider value only while that slider has focus (it was clicked and focus was not moved). Focus is cleared by a click anywhere else, by Esc, and when the selection changes. The wheel over the canvas never changes a slider.
- Slider drags capture the mouse from press to release, even when the pointer leaves the bar.
- Hold-to-repeat on `[-]`/`[+]`: first step on press, repeat after 400 ms at 10 Hz. The whole hold is one undo step.
- Section expanded/collapsed state is kept per section id for the editor session (not persisted to disk).
- Hovering a control for 500 ms shows a tooltip with its name and keyboard shortcut, if one exists (reuse `cabinet::ui::Tooltip`).

### 6. Track-level view (`Selection::None`)
- Two sections: **TOOL** (placement defaults of the active tool, only when that tool has any) and **CIRCUIT** (global wall offset, global wall type, off-track surface, car category, grid slots, auto checkpoints, rebuild geometry).
- The global off-track dropdown lists every entry of `SurfaceType::OFF_TRACK_TYPES` (adds the missing Sheet Ice).
- This view may scroll; it must never draw outside the card.

### 7. Small fixes inside the rebuilt controls
- `GridSlot` shows "Grid Slot #N", its read-only position, and the footer.
- The multi-ramp exact-angle label reads "0°–360°".
- Surface zone material edits call `revalidate` like other edits.
- Wall distance is per side (L and R). The Flush preset writes the smallest distance that `validate_track` accepts, not `0.0` (open part of `tdrace-fnqg`).

### 8. Human checkpoints (required)
The implementer stops and asks the user at each checkpoint. Work after a checkpoint starts only after the user answers in chat. Each answer is recorded as a comment on the matching Beads task.

| Id | When | What the human gets | Decision |
|---|---|---|---|
| **HC-1** Layout direction | Before any inspector code | Static mockups (HTML/SVG, no engine code) at 1280×720 of the Waypoint, Jump Ramp, multi-waypoint and track-level views, for **Layout A** (collapsible sections + scroll + sticky footer, §2) and **Layout B** (tabs: Shape · Edges · Surface · Actions, no scroll). Includes two surface-picker styles: **list dropdown with swatches** and **swatch-grid popover**. | Pick layout, picker style, and confirm or change the wheel focus rule. |
| **HC-2** Compact controls review | After the shared controls exist, before migrating all kinds | In-engine screenshots (via the dev flag below) of the Waypoint view only, built with the chosen layout. | Approve the control look and density, or request changes. |
| **HC-3** Mixed-kind multi-selection | After same-kind multi-selection works | In-engine screenshots of one mixed selection rendered with Option A and Option B (§4). | Pick A or B. |
| **HC-4** Hands-on acceptance | Before closing the spec | A 10-step checklist (edit width, banking, walls, surface on 1 and 3 waypoints; edit a zone and a ramp; mixed selection; undo after a drag; scroll the track-level view; check tooltips). | Sign off or file changes. |

If the user rejects every option at a checkpoint, the implementer proposes at most two new alternatives and repeats the checkpoint. The normative scenarios below do not depend on which option wins.

### 9. Verification support
- Dev-only CLI argument `--editor-select <kind>` (used with the existing `editor`, `--screenshot` and `--frames` arguments) selects a fixed set of entities after the editor opens. Kinds: `waypoint`, `waypoints`, `zone`, `zones`, `obstacle`, `obstacles`, `ramp`, `ramps`, `checkpoint`, `grid`, `pit`, `mixed`, `none`.
- A fixture circuit `crates/tdrace-app/tests/fixtures/editor_inspector_fixture.json` contains at least 3 waypoints, 2 surface zones, 3 obstacles, 2 jump ramps, checkpoints, grid slots and a pit lane, so every kind can be selected. (No official circuit currently has zones, obstacles or ramps.)
- Before/after screenshots for HC-2, HC-3 and the closure walkthrough are saved under `docs/design/086/`.

### 10. Out of scope (file as follow-up beads)
- New editable properties: obstacle shape/size/rotation, ramp launch speed, pit lane parameters, waypoint elevation, joker/branch entities.
- Gamepad or keyboard focus navigation inside the inspector.
- Left tool sub-palette overflow at 720p (it ends at y ≈ 746–766).
- Spec 069 scenario 11 is ticked, but the inspector still uses `draw_ui_btn` rows and has no gamepad input. Record the gap; do not reopen 069.

---

## ⚙️ Backend Models & API Endpoints

No persisted data, file format or network API changes. Track JSON is unchanged.

### 1. Editor state (in memory only)
```rust
// crates/tdrace-app/src/editor/tools.rs (ToolSettings) or a new inspector module
pub struct InspectorViewState {
    pub scroll_offset: f32,                 // reset when selection kind changes
    pub collapsed_sections: HashSet<&'static str>,
    pub focused_control: Option<String>,    // replaces sticky selected_bar semantics
    pub drag_capture: Option<String>,       // control id that owns the current drag
    pub open_dropdown: Option<String>,
}

/// Result of reading one property across the selection.
pub enum Common<T> { Same(T), Mixed, Empty }
```

### 2. Layout model (testable without a window)
- A pure function builds the inspector rows for the current selection, for example `build_inspector_rows(&EditorState, &ToolSettings) -> Vec<InspectorRow>`, where each row has a section id, a control kind and a height in reference pixels. Drawing iterates these rows.
- A pure helper reads common values across the selection, for example `common_value<T: PartialEq>(values) -> Common<T>`.
- These two functions are what the automated tests use to check fit, sections and mixed values.

### 3. Shared crate API (`cabinet`, additive only)
- Extend `DropdownWidget` / `draw_dropdown_popup` in [`crates/cabinet/src/ui/widgets.rs`](../crates/cabinet/src/ui/widgets.rs) with: a maximum visible row count with scrolling, an optional colour swatch per option, a compact "field" closed style (one row, label outside), a mixed/no-selection display, and popup placement that stays inside a given bounding rectangle.
- Existing signatures and default behaviour stay unchanged. Settings screens that use the dropdown today must render exactly as before.
- If a segmented-control widget is needed and none exists, add it to `cabinet` (game-agnostic) rather than to the editor. `RadioGroup` (`widgets.rs`) is checked first for reuse.
- No new third-party dependencies.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

- No authentication, roles, network or persistence surfaces change.
- **Input bounds:** every edit path (single and batch) clamps through one shared range constant per property. Typed values in click-to-type fields are parsed, clamped, and rejected if not finite.
- **Hidden-input safety:** controls outside the clipped body or under a modal/popup receive no input. An open dropdown popup consumes the click that closes it, so the click does not also reach the canvas.
- **Dev flag safety:** `--editor-select` only changes the in-memory selection. It never writes files and is ignored when the indices do not exist.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- Workspace tests: `cargo test --workspace --exclude tdrace-py`
- Editor tests: `cargo test -p tdrace-app --test track_editor_tests`
- Shared crate tests: `cargo test -p cabinet`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- New tests (in `crates/tdrace-app/tests/track_editor_tests.rs` or a new `crates/tdrace-app/tests/editor_inspector_tests.rs`) cover: per-kind fit at 1280×720 using the row model, same sections for one vs many, `common_value` mixed detection, absolute vs relative batch edits, one undo per drag and per hold, focus clearing, shared clamp ranges, and the off-track list containing every `OFF_TRACK_TYPES` entry.
- Screenshot evidence: `cargo run -p tdrace-app -- editor --editor-select <kind> --screenshot docs/design/086/<kind>.png --frames 20` for each kind.

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Human chooses the inspector layout before implementation**
  - [x] **Given** two layout mockups and two surface-picker mockups at 1280×720 for the Waypoint, Jump Ramp, multi-waypoint and track-level views
  - [x] **When** the implementer presents them to the user at checkpoint HC-1
  - [x] **Then** no inspector code is changed until the user picks a layout, a picker style and the wheel rule in chat, and the choice is recorded on the Beads task

- **Scenario: Every single-entity view fits without scrolling**
  - [x] **Given** the editor at 1280×720 with the fixture circuit and all sections expanded
  - [x] **When** a single waypoint, surface zone, obstacle, jump ramp, checkpoint, grid slot or pit box is selected
  - [x] **Then** the row model height is at most the card body height, no scrollbar is shown, and the footer actions are visible

- **Scenario: Nothing draws or reacts outside the inspector card**
  - [x] **Given** the track-level view with the Road Spline tool active, whose content is taller than the card
  - [x] **When** the user scrolls the inspector and clicks on the status bar area below the card
  - [x] **Then** the content is clipped to the card, a scrollbar shows the position, all controls are reachable by scrolling, and the status-bar click changes no inspector value

- **Scenario: Surface is chosen from one compact dropdown**
  - [x] **Given** a selected waypoint, surface zone or jump ramp, or the track-level off-track setting
  - [x] **When** the user opens the surface control
  - [x] **Then** one dropdown opens with a colour swatch per entry, in one shared order and naming, inside the screen, and choosing an entry applies it and closes the dropdown

- **Scenario: Off-track surface list is complete**
  - [x] **Given** the track-level view
  - [x] **When** the off-track surface dropdown is opened
  - [x] **Then** it lists every entry of `SurfaceType::OFF_TRACK_TYPES`, including Sheet Ice

- **Scenario: Several waypoints show the same controls as one waypoint**
  - [x] **Given** three selected waypoints with equal width and different banking
  - [x] **When** the inspector is shown
  - [x] **Then** it shows the same sections and controls as for one waypoint, the header shows "3 selected", width shows the common value, and banking shows the mixed state

- **Scenario: Absolute and relative edits on a multi-selection**
  - [x] **Given** three selected waypoints with banking 0°, 5° and 10°
  - [x] **When** the user chooses the 18° preset, undoes, and then presses `[+]` once on the 1° step
  - [x] **Then** the preset sets all three to 18°, the undo restores 0°/5°/10°, and the step gives 1°/6°/11°

- **Scenario: Several zones and several ramps keep their controls**
  - [x] **Given** two selected surface zones, and separately two selected jump ramps
  - [x] **When** the inspector is shown for each selection
  - [x] **Then** the zones show material and layer controls, the ramps show angle, length, width, height, pitch, fit actions and material, and an edit applies to both entities

- **Scenario: Mixed-kind selection follows the human's choice**
  - [x] **Given** a selection with waypoints, a surface zone and a jump ramp, and the option the user picked at HC-3
  - [x] **When** the inspector is shown
  - [x] **Then** each kind's controls are reachable as that option describes, and an edit changes only entities of the kind whose control was used

- **Scenario: Single and batch paths share clamp ranges**
  - [x] **Given** one selected waypoint, and separately three selected waypoints
  - [x] **When** the user types a width and a wall distance above the maximum and below the minimum
  - [x] **Then** both selections clamp to the same minimum and maximum values

- **Scenario: One slider drag is one undo step**
  - [x] **Given** an undo history with at least one earlier edit
  - [x] **When** the user drags the width slider for two seconds across many values and then presses undo once
  - [x] **Then** the width returns to its value before the drag and the earlier edit is still in the history

- **Scenario: Holding a step button repeats with a delay and one undo step**
  - [x] **Given** a selected waypoint
  - [x] **When** the user holds `[+]` on width for one second and then presses undo once
  - [x] **Then** the first step happens on press, repeats start after 400 ms, and one undo restores the original width

- **Scenario: The mouse wheel never edits a slider by accident**
  - [x] **Given** the user clicked a slider and then clicked on the canvas
  - [x] **When** the user scrolls the mouse wheel over the canvas and over the inspector body
  - [x] **Then** the canvas zooms, the inspector scrolls (if it can), and no slider value changes

- **Scenario: Slider drag keeps working outside the bar**
  - [x] **Given** the user pressed the mouse on the width slider
  - [x] **When** the user moves the pointer outside the bar while holding the button and releases it there
  - [x] **Then** the value follows the pointer until release, and pressing outside then moving onto the bar does not start a drag

- **Scenario: Footer actions are always visible**
  - [x] **Given** any non-empty selection, at any scroll position
  - [x] **When** the inspector is shown
  - [x] **Then** Duplicate and Delete are visible in the footer with their shortcuts, and they act on the whole selection

- **Scenario: Tooltips show shortcuts**
  - [x] **Given** a control that has a keyboard shortcut (banking presets, ramp rotate, zone layer, duplicate, delete)
  - [x] **When** the pointer rests on it for 500 ms
  - [x] **Then** a tooltip shows the control name and its shortcut, inside the screen

- **Scenario: Grid slot selection is not empty**
  - [x] **Given** a selected grid slot
  - [x] **When** the inspector is shown
  - [x] **Then** it shows "Grid Slot #N", its position, and the footer

- **Scenario: Shared dropdown change does not alter existing screens**
  - [x] **Given** the settings screens that use `DropdownWidget` today
  - [x] **When** they are rendered after the `cabinet` change
  - [x] **Then** they look and behave as before, and `cargo test -p cabinet` passes

- **Scenario: Human hands-on acceptance**
  - [x] **Given** the finished inspector and the HC-4 checklist
  - [x] **When** the user runs the checklist in the real editor
  - [x] **Then** the user signs off in chat, or each requested change is fixed or filed as a bead before closure

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[x]` `crates/tdrace-app/src/editor/ui.rs` -> `render_inspector` rebuilt on the row model, sections, footer, clip and scroll; `is_mouse_over_editor_ui` uses the same panel rectangle as drawing; `draw_bar_control`, `draw_counter`, `draw_toggle` updated or replaced.
- `[x]` `crates/tdrace-app/src/editor/inspector.rs` (new, if `ui.rs` would grow) -> row model, `common_value`, section definitions, shared surface catalogue.
- `[x]` `crates/tdrace-app/src/editor/tools.rs` -> inspector view state, shared clamp constants, undo-per-gesture, per-side wall distance, batch zone layer, focus clearing.
- `[x]` `crates/tdrace-app/src/editor/state.rs` -> undo gesture grouping if `HistoryStack` needs it.
- `[x]` `crates/tdrace-app/src/game/mod.rs` -> wheel routing (canvas zoom vs inspector scroll vs focused slider).
- `[x]` `crates/tdrace-app/src/main.rs` -> dev-only `--editor-select <kind>` argument.
- `[x]` `crates/cabinet/src/ui/widgets.rs` -> additive `DropdownWidget` extensions; segmented control if needed.
- `[x]` `crates/cabinet/tests/` -> dropdown extension tests.
- `[x]` `crates/tdrace-app/tests/fixtures/editor_inspector_fixture.json` -> fixture circuit with every entity kind.
- `[x]` `crates/tdrace-app/tests/track_editor_tests.rs` and/or `crates/tdrace-app/tests/editor_inspector_tests.rs` -> new tests listed above.
- `[x]` `docs/design/086/` -> HC-1 mockups, HC-2/HC-3 screenshots, closure before/after screenshots.
- `[x]` `docs/engineering/screens_and_navigation.md` -> Track Studio entry (around line 595) describes the new inspector layout.
- `[x]` `crates/tdrace-app/src/editor/ui.rs` (`render_help_modal`) -> "Tools 1-8" corrected to the real tool keys (1-0 and -).

### Verification Assertions
- The inspector module header comment references `specs/086_compact_track_studio_inspector_with_unified_single_and_multiselection_editing.md`.
- Each human checkpoint (HC-1…HC-4) has a recorded decision on its Beads task before the next task starts.
- Related open bug: `tdrace-fnqg` (per-side wall distance, Flush preset writes an invalid 0.0).
