---
type: Architecture Spec
template: architecture
title: "Publish-Ready Shared Crates"
description: "Phase 4 of spec 049: a separate game repo can depend on wheelbase, arcade-race-core, race-kit, race-ui and cabinet by one git tag, with a README and a runnable minimal race example, a platform changelog, and gamepad profile paths that follow the game's own name."
status: draft
created: 2026-09-29
generated: { by: agent/claude-opus-5-5, at: 2026-09-29T17:30:00Z }
---

# Architecture Spec: Publish-Ready Shared Crates 🏗️

Phase 4 of [spec 049](049_reusable_racing_platform_layers.md), which sets the chariot-first order. It builds on [spec 056](056_racekit_headless_race_world.md) (`race-kit`) and [spec 058](058_raceui_rendering_camera_effects_and_hud_primitives.md) (`race-ui`).

**Goal:**
- The chariot game (Phase 5) lives in its own repo. It uses the shared crates from this repo through one git tag.
- A new developer can start from a README and one small example that runs.

---

## 🗺️ Current vs. Proposed System Architecture

### 1. Current Architecture

Measured on 2026-09-29 with a scratch crate outside the repo that uses `race-kit` and `race-ui` through `git = "file:///…/tdrace", branch = "main"`:
- **It builds and runs.** Cargo finds each crate by name inside the workspace. None of them depends on `tdrace-core` or on the `tracks/` folder at build time.
- **Cargo also fetches the `tracks` submodule.** Cargo always updates the submodules of a git dependency, and it has no option to skip them. The `tdrace` repo is public, but `tdrace-tracks` is private. So the build works only where an SSH key can read `tdrace-tracks`, as on Mario's machine. One checkout is 637 MB: 297 MB of assets and 134 MB of circuits.

Other gaps:
- There is no README or example for `race-kit` or `race-ui`. A new game has to read `tdrace-app` to learn the step and draw order.
- There is no record of what changed in the shared crates, and no tag that a game can pin.
- `cabinet::GamepadManager::candidate_profile_paths` (`crates/cabinet/src/input/gamepad.rs:283-299`) looks for `gamepad_profile.json` in fixed `tdrace` and `asteroids` folders. A chariot game would look in the tdrace folders.

### 2. Proposed Architecture

**One tag for the whole platform.** All five shared crates use the workspace version (`0.1.0`) and ship from the same commit. So one tag, `platform-v0.1.0`, pins all of them together:

```toml
[dependencies]
race-kit = { git = "https://github.com/magomar/tdrace", tag = "platform-v0.1.0" }
race-ui  = { git = "https://github.com/magomar/tdrace", tag = "platform-v0.1.0" }
```

Creating the tag is part of this phase. Pushing it needs Mario's go-ahead.

**README and example.**
- `crates/race-kit/README.md`: what the crate does, the git dependency line, and the step loop in about 20 lines: spawn cars, drive them with `BotAiDriver`, call `RaceWorld::step`, and read events and results.
- `crates/race-ui/README.md`: the draw order (track, barriers, cars, effects, HUD), the camera, and `set_asset_root`.
- `crates/race-ui/examples/minimal_race.rs`: a window with the generated oval, 4 bot cars in a 3-lap race, the follow camera, dust and skid marks, the lap timer and the minimap. It uses only the shared crates. `race-kit` is a dev-dependency of `race-ui`, so `race-ui` itself still does not depend on it.
- A small car renderer inside the example draws each car as a coloured box. The tdrace car sprites stay in `tdrace-app`, and Phase 5 brings a shared vehicle sprite interface.

**Changelog.** One file, `docs/platform/CHANGELOG.md`, for the five crates, because they share one version and one tag. The first entry lists what specs 049, 054, 056 and 058 added.

**Gamepad profile paths.** A new `cabinet::input::set_app_id(id)` sets the game's folder name, in the same way as `race_ui::render::set_asset_root`. Once it is set, the profile search is:
- `gamepad_profile.json`
- `../gamepad-mapper/gamepad_profile.json`
- `../<id>/gamepad_profile.json`
- `~/.config/gamepad-mapper/gamepad_profile.json`
- `~/.config/<id>/gamepad_profile.json`

Without `set_app_id`, the list is today's list, unchanged, so tdrace behaves the same.

**Changes from the spec 049 plan, with reasons:**
- **One tag, not five.** The crates share one workspace version. Five tags on the same commit would only add names to keep in step.
- **`Track.car_category` stays a `CarCategory`.** Making it `Option<String>` changes the track JSON format and about 20 files. The code that reads it (runoff surfaces, barrier offsets) is tdrace product logic, which Phase 9 removes from the engine. A chariot track can leave the field out, and it loads as `Gt` without error.

**Out of scope, with reasons:**
- **Removing the private submodule from the git dependency.** That needs the shared crates in their own repo (`arcade-kit`). Spec 049 plans that split only after the first outside game has used the crates for a while. Until then, the chariot repo builds on machines that can read `tdrace-tracks`.
- **Publishing to crates.io.** Nothing needs it, and the crates are still changing.

---

## 🗄️ Database & Storage Migration Plan

No data format changes.

---

## 🔑 Security, Compliance, & IAM Roles

- No new dependencies. `race-kit` becomes a dev-dependency of `race-ui`, inside this workspace.
- The tag push is outward-facing, so Mario does it.
- The README points to the public `tdrace` URL. It also says that the private `tdrace-tracks` submodule needs read access.

---

## 🛡️ Disaster Recovery, Monitoring, & Fallbacks

- **No behaviour change in tdrace:** `set_app_id` is not called by tdrace, so its gamepad search is unchanged. The golden hashes must not change.
- **Rollback:** a single `--no-ff` merge. A pushed tag can be deleted if the release is wrong.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- Example builds: `cargo build -p race-ui --example minimal_race`
- Gamepad paths: `cargo test -p cabinet`
- Rust suite: `cargo test --workspace --exclude tdrace-py --no-fail-fast`. Only the known failure `tdrace-6eaf` is allowed.
- Builds: `cargo check -p race-ui --target wasm32-unknown-unknown` and `cargo check -p tdrace-app --target wasm32-unknown-unknown`
- Dependencies: `cargo tree -p race-ui -e normal` and `cargo tree -p race-kit -e normal`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: A crate outside the repo builds against the tag**
  - [ ] **Given** a scratch crate outside the repo that depends on `race-kit` and `race-ui` by `git` and `tag = "platform-v0.1.0"` (the local repo URL while the tag is not pushed)
  - [ ] **When** `cargo build` runs in it with no `tracks/` checkout in its own folder
  - [ ] **Then** it builds, and a program that creates a `RaceWorld` and a `RaceCamera` runs

- **Scenario: The minimal race example runs**
  - [ ] **Given** the `minimal_race` example
  - [ ] **When** Mario runs `cargo run -p race-ui --example minimal_race`
  - [ ] **Then** a window shows the oval, 4 cars racing with a follow camera, dust or skid marks, a lap timer and a minimap, and the race ends after 3 laps

- **Scenario: The example uses only the shared crates**
  - [ ] **Given** `crates/race-ui/examples/minimal_race.rs`
  - [ ] **When** you search it for `tdrace`
  - [ ] **Then** there is no match

- **Scenario: A game names its own gamepad profile folder**
  - [ ] **Given** `set_app_id("chariot")`
  - [ ] **When** `candidate_profile_paths` runs
  - [ ] **Then** the list has `../chariot/gamepad_profile.json` and `~/.config/chariot/gamepad_profile.json`, and no `tdrace` or `asteroids` entry. Without `set_app_id` the list is the same as before this spec.

- **Scenario: tdrace is unchanged**
  - [ ] **Given** the golden hashes recorded for debug and release on macOS aarch64
  - [ ] **When** `golden_sim`, `golden_world` and `golden_session` run
  - [ ] **Then** all of them keep their recorded hashes

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[ ]` `crates/race-kit/README.md` -> what the crate does, the dependency line, the step loop.
- `[ ]` `crates/race-ui/README.md` -> the draw order, the camera, the asset root.
- `[ ]` `crates/race-ui/examples/minimal_race.rs` -> a window with a 4-car oval race.
- `[ ]` `crates/race-ui/Cargo.toml` -> `race-kit` as a dev-dependency.
- `[ ]` `crates/cabinet/src/input/gamepad.rs` -> `set_app_id` and the app-specific search list, with tests.
- `[ ]` `docs/platform/CHANGELOG.md` -> the first platform release entry.
- `[ ]` git tag `platform-v0.1.0` -> created locally. Mario pushes it.

### Beads Epic Mapping
- Governed by epic *Fulfill Spec 059: Publish-Ready Shared Crates*.
