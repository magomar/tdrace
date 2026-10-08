---
type: Architecture Spec
template: architecture
title: "DLC Architecture and Modular Base Game Content Packaging"
description: "Establishes a modular content and DLC architecture defining the initial Steam base game release (Classic + Karting + Autocross + Rallycross) and dynamic entitlement-gated expansions (Stock Cars, Extreme Off-Road, GT), decouples static module hardcoding into an extensible ModuleRegistry, introduces the EntitlementProvider abstraction for Steamworks SDK and dev/offline parity, and provides in-game DLC showcase teasers with Steam store overlay hooks."
status: draft
created: 2026-10-07
generated: { by: agent/antigravity, at: 2026-10-07T15:20:57Z }
depends_on:
  - "042"
  - "047"
  - "050"
  - "051"
  - "052"
  - "060"
  - "066"
---

# Architecture Spec 100: DLC Architecture and Modular Base Game Content Packaging 🏗️

TdRace is preparing for its commercial release on Steam. Today, all motorsport disciplines, vehicles, audio profiles, and 96+ circuits are compiled directly into a monolithic binary and unconditionally unlocked. This architecture makes it impossible to ship a focused base game and sell subsequent expansions as Steam Downloadable Content (DLC).

This specification establishes a **modular DLC architecture** that:
1. Defines the **Initial Steam Base Game**: **Classic Arcade + Grassroots Karting + FIA Autocross + Rallycross**, organized along an authentic motorsport license ladder (`Karting [License D] -> Autocross [License C] -> Rallycross [License B]`).
2. Isolates future disciplines (**Stock Cars**, **Extreme Off-Road**, and **GT World Challenge**) as **Expansion DLCs**.
3. Decouples hardcoded module dispatch into an extensible, metadata-driven `ModuleManager` and `ContentRegistry`.
4. Introduces the `EntitlementProvider` abstraction for clean Steamworks SDK integration (`steamworks-rs`) with headless/offline development parity.
5. Implements in-game **Showcase Teasers** for unowned DLCs with direct Steam Overlay store hooks.

---

## 🗺️ Current vs. Proposed System Architecture

### 1. Current Architecture

Currently, `tdrace` operates as a compile-time monolith:
* **Embedded Circuit Monolith**: `crates/tdrace-core/build.rs` scans `tracks/` and compiles all 96+ circuit JSON files into `CIRCUITS: &[EmbeddedCircuit]` using `include_bytes!` (Spec 042). Every track from every discipline is baked into the main binary.
* **Hardcoded Module Switchboard**: `crates/tdrace-app/src/module/mod.rs` and `crates/tdrace-app/src/game/mod.rs` maintain hardcoded functions (`switch_to_gt()`, `switch_to_kart()`, `switch_to_nascar()`, etc.) and exhaustive matches.
* **Uniform UI Presentation**: The Modality Selector and Career Hub treat all disciplines as permanently owned and unlocked, with no concept of content entitlement, store linking, or commercial packaging.

```mermaid
flowchart TD
    subgraph Monolithic Build
        Core["tdrace-core<br/>(Embedded 96+ Tracks)"]
        App["tdrace-app<br/>(Hardcoded Module Enums & Logic)"]
    end

    subgraph Static Disciplines
        M_Classic["Classic (Built-in)"]
        M_Kart["Karting (Built-in)"]
        M_AX["Autocross (Built-in)"]
        M_RX["Rallycross (Built-in)"]
        M_Stock["Stock Cars (Built-in)"]
        M_Offroad["Extreme Off-Road (Built-in)"]
        M_GT["GT Challenge (Built-in)"]
    end

    Core --> App
    App --> M_Classic & M_Kart & M_AX & M_RX & M_Stock & M_Offroad & M_GT
```

---

### 2. Proposed Architecture

The proposed architecture cleanly separates the **Core Base Game** from **Expansion DLCs**, routing all module access through a central `ModuleManager` backed by an `EntitlementProvider`.

```mermaid
flowchart TD
    subgraph Steam Ecosystem / OS Environment
        SteamClient["Steam Client / Steamworks API"]
        LocalEnv["Local Dev / Offline Fallback"]
    end

    subgraph Platform Abstraction Layer
        EP["trait EntitlementProvider<br/>• is_dlc_installed(app_id)<br/>• open_store_page(app_id)"]
        SEP["SteamEntitlementProvider<br/>(steamworks::Client)"]
        DEP["DevEntitlementProvider<br/>(Auto-unlock all)"]
    end

    SteamClient --> SEP
    LocalEnv --> DEP
    SEP & DEP -.-> EP

    subgraph Game Runtime
        MM["ModuleManager & ContentRegistry"]
        EP --> MM

        subgraph Base Game (Included)
            B_Classic["Classic Academy & Arcade"]
            B_Kart["Grassroots Karting (License D)"]
            B_AX["FIA Autocross (License C)"]
            B_RX["Rallycross (License B)"]
        end

        subgraph Expansion DLCs (Gated)
            D_Stock["Stock Car Cup (DLC AppID: TBD)"]
            D_Offroad["Extreme Off-Road (DLC AppID: TBD)"]
            D_GT["GT & Prototypes (DLC AppID: TBD)"]
        end

        MM --> B_Classic & B_Kart & B_AX & B_RX
        MM -->|Entitled| D_Stock & D_Offroad & D_GT
        MM -->|Unowned| Showcase["Interactive Turntable Teaser<br/>+ [View on Steam Store]"]
    end
```

---

## 📦 Content Packaging & License Progression

### 1. Base Game Roster (Initial Steam Release)

The initial base game focuses on the **tactical sweet spot of top-down arcade racing**: agile handling, surface friction transitions (asphalt to dirt), chicanes, and bumper-to-bumper pack racing.

| Discipline | Identifier | Career License | Key Focus & Physics Characteristics | Content Volume |
| :--- | :--- | :--- | :--- | :--- |
| **Classic Arcade** | `classic` | Academy Starter | Pickup-and-play arcade fantasy racing, driving academy tutorials, credit seed fund. | 18 Circuits, 12 Cars |
| **Grassroots Karting** | `kart` | **License D** (National C) | Ultra-direct steering, caster-jacking inside wheel lift, high lateral Gs, zero downforce. | 20 Circuits, 6 Tiers |
| **FIA Autocross** | `autocross` | **License C** (National B) | Pure loose dirt racing, buggy counter-steering, sprint heats, 4WD cross cars. | 17 Circuits, 5 Tiers |
| **Rallycross** | `rally` | **License B** (National A) | Mixed asphalt/gravel transitions, 600 BHP launch acceleration, Joker Lap strategy. | 20 Circuits, 6 Tiers |

#### The Natural Progression Flow:
* **Onboarding**: Player completes Classic Academy missions or starter races to acquire driving license D.
* **Tier 1 (License D - Grassroots)**: Grassroots Karting teaches racing lines, late braking, and weight transfer.
* **Tier 2 (License C - Clubman Dirt)**: FIA Autocross challenges players to master low-grip dirt steering and slide recovery.
* **Tier 3 (License B - Pro-Am All-Terrain)**: Rallycross combines high horsepower with hybrid asphalt/dirt tracks and mandatory Joker Lap tactical decisions.
* **Tier 4 (License A - Apex Superlicense)**: GT World Challenge is reserved for master drivers, introducing tactical depth:
  * **Tyre Compound Strategy (Spec 098)**: Selecting Soft, Medium, or Hard slicks in the garage based on circuit wear and race length.
  * **Pit Stop Procedures (Spec 062 & 077)**: Speed-limited pit lane entries, pit box stops, and fresh tire service.
  * **Aerodynamic High-Downforce Dynamics**: Cornering at extreme speeds with high downforce and engine durability/damage risks (Spec 078).

---

### 2. Expansion DLC Modules

Future DLCs act as **Lateral Career Specializations** or **Prestige Superlicense Expansions**:

| DLC Expansion | Identifier | Type | Career License | Role & Description |
| :--- | :--- | :--- | :--- | :--- |
| **Stock Car Cup** | `nascar` | Lateral Specialization | Oval License | 5 tiers of high-speed oval drafting, aerodynamic slingshotting, stage racing, and superspeedways (Daytona, Talladega). |
| **Extreme Off-Road** | `extreme_offroad` | Lateral Specialization | Off-Road License | Desert sand dunes, rock crawls, mud bogs, and stadium stunt arenas with massive jump ramps. |
| **GT World Challenge** | `gt` | Prestige Expansion | **License A** (Superlicense) | High-downforce endurance GT3/GT1/Hypercar racing requiring tire compound selection, pit box stops, and top-down high-speed precision. |

---

## 🛠️ Detailed Component Architecture

### 1. The `EntitlementProvider` Abstraction

To ensure that the game runs seamlessly in continuous integration, local testing, headless simulation harnesses, and on Steam, entitlement queries are encapsulated behind a trait:

```rust
pub trait EntitlementProvider: Send + Sync {
    /// Checks whether the user owns and has installed the specified DLC.
    fn is_dlc_installed(&self, steam_app_id: u32) -> bool;

    /// Activates the platform store page / overlay for the DLC.
    fn open_store_page(&self, steam_app_id: u32);

    /// Name of the provider for telemetry and logging.
    fn provider_name(&self) -> &'static str;
}
```

#### Provider Implementations:
1. **`SteamEntitlementProvider`** (`crates/tdrace-app/src/platform/steam.rs`):
   * Compiled under `#[cfg(feature = "steam")]`.
   * Initializes `steamworks::Client::init_app(BASE_APP_ID)`.
   * Calls `client.apps().is_dlc_installed(steam_app_id)`.
   * Calls `client.friends().activate_game_overlay_to_store(steam_app_id, OverlayToStoreFlag::None)`.
2. **`DevEntitlementProvider`** (`crates/tdrace-app/src/platform/dev.rs`):
   * Default implementation for `cargo run`, `cargo test`, and builds without the `steam` feature.
   * Unconditionally returns `true` for all DLC IDs, allowing development and testing of all content without requiring a running Steam client or purchased licenses.
3. **`MockEntitlementProvider`**:
   * Configurable set of active DLC IDs for unit and UI testing of locked/unlocked states.

---

### 2. The `ModuleManager` & `ModuleDescriptor`

`tdrace-app` introduces a centralized `ModuleManager`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleAvailability {
    /// Always available as part of the base game.
    Core,
    /// Requires DLC entitlement.
    Dlc {
        steam_app_id: u32,
        store_slug: &'static str,
    },
}

pub struct ModuleDescriptor {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub availability: ModuleAvailability,
    pub license_requirement: Option<LicenseTier>,
    pub factory: Box<dyn Fn() -> Box<dyn GameModule> + Send + Sync>,
}

pub struct ModuleManager {
    descriptors: Vec<ModuleDescriptor>,
    entitlement: Box<dyn EntitlementProvider>,
}

impl ModuleManager {
    /// Returns all registered modules.
    pub fn all_modules(&self) -> &[ModuleDescriptor] {
        &self.descriptors
    }

    /// Checks if a module is currently unlocked and playable.
    pub fn is_unlocked(&self, module_id: &str) -> bool {
        let Some(desc) = self.descriptors.iter().find(|d| d.id == module_id) else {
            return false;
        };
        match desc.availability {
            ModuleAvailability::Core => true,
            ModuleAvailability::Dlc { steam_app_id, .. } => {
                self.entitlement.is_dlc_installed(steam_app_id)
            }
        }
    }

    /// Triggers the store overlay for an unowned module.
    pub fn request_purchase(&self, module_id: &str) {
        if let Some(desc) = self.descriptors.iter().find(|d| d.id == module_id) {
            if let ModuleAvailability::Dlc { steam_app_id, .. } = desc.availability {
                self.entitlement.open_store_page(steam_app_id);
            }
        }
    }
}
```

---

### 3. In-Game Showcase Teaser for Unowned DLCs

Unowned DLC modules are not hidden; they appear in the UI as premium expansions to drive player awareness and conversion:

1. **Modality Selector Badge**:
   * Core modules display standard career/racing options.
   * Unowned DLC modules display a gold **"DLC EXPANSION"** ribbon or subtle padlock icon.
2. **Interactive Showcase Screen**:
   * Selecting an unowned DLC opens a dedicated **Showcase Screen**:
     * **3D/2D Car Turntable**: The player can freely rotate the signature vehicle model and view telemetry ratings.
     * **Circuit Tour**: Interactive previews of the included tracks.
     * **Sound Preview**: Ability to rev the engine and hear the custom physical audio profile.
     * **Action Call**: Prominent **[ View on Steam Store ]** button, which invokes `open_store_page()`.
3. **Graceful LAN Multiplayer Handling**:
   * If a LAN host selects a DLC track, clients who do not own the DLC are informed via a clear dialog:
     * *Policy*: Guests without the DLC are permitted to join as guest racers with default liveries (community-friendly design) or spectator mode.

---

### 4. Circuit Catalog Integration ([`catalog.rs`](../crates/tdrace-core/src/catalog.rs))

The embedded catalog in `crates/tdrace-core/src/catalog.rs` is updated to tag circuits with their parent module's packaging status:
* Built-in base circuits (`classic`, `kart`, `autocross`, `rally`) are embedded directly as default assets.
* Gated DLC circuits are registered in the catalog and filtered via the `ModuleManager` entitlement check before being presented in non-career quick race selectors.

---

## 🗄️ Database & Storage Migration Plan

### 1. SQLite Career Profile Integrity
* Player career achievements, trophies, stunt scores, and best lap records are saved in `tdrace_records.db` (governed by `crates/cabinet/src/records`).
* **Uninstallation Safety**: If a player purchases a DLC, earns trophies or lap records, and subsequently uninstalls or disables the DLC in Steam:
  * The database retains the player's records and timestamps without deleting them.
  * Trophies earned in the DLC remain visible in the Player Profile Trophy Cabinet (Spec 027) with a `"DLC Archived"` tag.
  * Reinstalling the DLC seamlessly restores full access to the existing save state with zero data loss.

---

## 🔑 Security, Compliance, & IAM Roles

1. **Steam DRM & Offline Safety**:
   * TdRace does not enforce hostile always-online DRM.
   * If the player launches the game in Steam Offline Mode or without an active internet connection, Steamworks SDK continues to report installed DLC status via cached client tickets.
2. **Steam Deck Parity**:
   * The store overlay trigger (`activate_game_overlay_to_store`) works natively in Steam Deck Game Mode, suspending the game view and opening the Steam store page.
3. **No External Microtransactions & IAM Compliance**:
   * All DLC transactions flow strictly through Valve's official Steam Store infrastructure in compliance with the Steam Distribution Agreement. No external payment processors, auth tokens, or IAM roles are introduced into the client binary.

---

## 🛡️ Disaster Recovery, Monitoring, & Fallbacks

1. **Offline & Steam Client Failure Fallback**:
   * If the Steamworks API fails to initialize (e.g. Steam is not running, binary launched standalone, or network failure), the engine automatically falls back to `DevEntitlementProvider` in debug builds or locks DLCs to graceful preview mode while keeping the 100% of the Base Game playable.
2. **Missing DLC Asset Handling**:
   * If a save file contains references to a car or track from a DLC that is currently uninstalled or missing from disk, the game does not crash: it substitutes fallback default car visuals and prompts the player that the asset requires the DLC expansion.
3. **Telemetry & Error Logging**:
   * All entitlement failures and Steam overlay invocation errors are captured in the game log (`~/.local/share/tdrace/log` or equivalent) with diagnostic codes rather than raising panics.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests

- **Unit & Integration Tests**:
  * `cargo test --package tdrace-app test_module_manager_base_game_unlocked`
  * `cargo test --package tdrace-app test_module_manager_dlc_gating_with_mock`
  * `cargo test --package tdrace-app test_license_progression_hierarchy`
  * `cargo test --package tdrace-core test_base_game_catalog_circuits`

---

### Manual Acceptance Criteria (Pseudo-Gherkin)

#### Scenario 1: Base Game Content Availability Out of the Box
- [ ] **Given** a fresh installation of the base game without any purchased DLCs
- [ ] **When** the player launches the game and opens the Modality Hub
- [ ] **Then** Classic Arcade, Grassroots Karting, FIA Autocross, and Rallycross should all be active and selectable
- [ ] **And** no DLC ownership warnings should be shown for these four modalities.

#### Scenario 2: Unowned DLC Showcase and Store Hook
- [ ] **Given** the player does not own the Stock Car Cup DLC
- [ ] **When** the player highlights the Stock Car modality in the Modality Hub
- [ ] **Then** the card should display a "DLC EXPANSION" badge
- [ ] **And** selecting the card should open the interactive Showcase View showing car telemetry and track previews
- [ ] **And** clicking "View on Steam Store" should open the Steam Overlay to the designated DLC store page.

#### Scenario 3: Entitlement Unlocking
- [ ] **Given** the player purchases and downloads a DLC expansion while the game is running or before launch
- [ ] **When** `EntitlementProvider::is_dlc_installed` returns true for that DLC
- [ ] **Then** the discipline should immediately unlock into full playable status
- [ ] **And** career championships, vehicles, and circuits belonging to that DLC should become active.

#### Scenario 4: Motorsport License Progression Order
- [ ] **Given** a player starting a fresh Career Mode profile with 0 XP
- [ ] **When** examining the career progression tree
- [ ] **Then** Grassroots Karting must be available at License D
- [ ] **And** FIA Autocross must require License C
- [ ] **And** Rallycross must require License B
- [ ] **And** the GT World Challenge DLC (when installed) must require License A (Apex Superlicense) due to tactical tire compound choices and pit stop mechanics
- [ ] **And** other DLC careers (Stock Cars, Extreme Off-Road) must plug in as lateral specializations without blocking base license advancement.

#### Scenario 5: Dev Mode and Offline Parity
- [ ] **Given** the game is compiled without the `steam` feature or launched in development mode
- [ ] **When** `DevEntitlementProvider` is active
- [ ] **Then** all modalities (Base and DLC) should be unlocked for testing and development
- [ ] **And** `cargo test` must pass completely without requiring the Steam client process.

---

## 🔗 Traceability & Codebase Mapping

### Created Files
- `[ ]` `crates/tdrace-app/src/platform/mod.rs` -> Platform entitlement abstraction layer.
- `[ ]` `crates/tdrace-app/src/platform/entitlement.rs` -> `EntitlementProvider` trait and mock implementations.
- `[ ]` `crates/tdrace-app/src/platform/steam.rs` -> Steamworks SDK integration wrapper.
- `[ ]` `crates/tdrace-app/src/platform/dev.rs` -> Local development and offline entitlement provider.
- `[ ]` `crates/tdrace-app/src/module/manager.rs` -> Central `ModuleManager` and `ModuleDescriptor` registry.
- `[ ]` `crates/tdrace-app/src/ui/dlc_showcase.rs` -> Interactive 2D car turntable and DLC teaser screen.
- `[ ]` `crates/tdrace-app/tests/dlc_entitlement_tests.rs` -> Comprehensive automated tests for DLC gating and discovery.

### Modified Files
- `[ ]` `Cargo.toml` -> Adds optional `steamworks` dependency under `[features] steam = ["dep:steamworks"]`.
- `[ ]` `crates/tdrace-app/Cargo.toml` -> Links optional steamworks feature.
- `[ ]` `crates/tdrace-app/src/lib.rs` -> Re-exports `platform` and `module::manager`.
- `[ ]` `crates/tdrace-app/src/game/mod.rs` -> Refactors hardcoded module switches to use `ModuleManager`.
- `[ ]` `crates/tdrace-app/src/ui/menu.rs` -> Updates Modality Select screen to render DLC badges and route unowned modules to the showcase screen.
- `[ ]` `specs/constitution/ROADMAP.md` -> Links Spec 100 under Phase 7 (Steam Release Readiness).
- `[ ]` `specs/index.md` -> Registers Spec 100 in the progressive specification index.
