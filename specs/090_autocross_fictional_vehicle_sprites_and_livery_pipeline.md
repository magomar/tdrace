---
type: Feature Spec
template: feature
title: "Autocross Fictional Vehicle Sprites and Livery Pipeline"
description: "End-to-end asset generation pipeline delivering 45 2D textures (15 lateral 1024x512, 15 thumb 256x128, 15 top-down 512x512) for all 5 tiers of the Continental Autocross fleet, facing right (+X), inspired by authentic racing liveries with zero IP markings."
status: implemented
created: 2026-10-05
generated: { by: agent/antigravity, at: 2026-10-05T12:11:12Z }
verified: { by: "human:mario", at: "2026-10-05T17:40:00Z", hash: "dfc7bbf7efb3" }
depends_on:
  - "050"
---

# Feature Spec: Autocross Fictional Vehicle Sprites and Livery Pipeline 🏎️🎨

When the dedicated FIA Autocross championship module was introduced in [Spec 050](050_fia_autocross_championship_and_vehicle_roster.md), 15 authentic off-road competition machines spanning 5 tiers (Cross Car Junior, Cross Car Senior, Buggy 1600, TouringAutocross, and SuperBuggy) were integrated into the simulation physics, sound synthesis, and catalog. However, corresponding 2D raster artwork was never authored or generated, leaving the vehicle entries marked as `"N/A"` in [`docs/legal/ip_rename_registry.toml`](../docs/legal/ip_rename_registry.toml) and rendering garage cards and showroom views blank.

This specification establishes an automated high-fidelity asset pipeline delivering 45 canonical 2D sprite textures (15 lateral 1024×512, 15 thumbnail 256×128, and 15 top-down 512×512) for the entire Continental Autocross fleet. Crucially, in strict compliance with the project's IP-removal mandate ([Spec 047](047_fictional_branding_and_realworld_ip_removal_for_steam_release.md)), the visual designs take aesthetic inspiration from the bodywork geometry and vibrant racing paint schemes of the real-world machines while systematically stripping all manufacturer badges, sponsor trademarks, logos, and proprietary brand lettering.

---

## 🗺️ User Flow & Interface Design

### 1. Garage & Showroom Turntable Interaction
In the Garage and Showroom menus, the player inspects and selects vehicles from the Autocross module:
- When scrolling through the vehicle selection carousel or viewing car statistics, [`get_vehicle_lateral_texture`](../crates/tdrace-app/src/render/vehicle_assets.rs) loads the 1024×512 lateral texture from `textures/vehicles/laterals/autocross/{model_id}.png`.
- The vehicle is displayed facing **RIGHT (+X direction)**, matching the rest of the game's lateral garage cards (GT, NASCAR, Rallycross, Extreme Off-Road, Classic, and Karting).
- Quick selection ribbons and dossier cards consume the downsampled 256×128 thumbnail (`_thumb.png`).

### 2. In-Race Top-Down (Cenital) Vehicle Rendering
During live race heats on dirt circuits:
- The top-down sprite from `textures/vehicles/topdown/autocross/{model_id}.png` renders centered on the vehicle's physics centroid.
- The sprite is oriented facing **RIGHT (+X direction)**, where the physics engine angle $\theta = 0$ corresponds to eastward motion along the track spline.
- Single-seat buggies and cross-cars clearly depict exposed tubular spaceframes, beadlock knobby tires, roof scoops, and pilot helmets facing the direction of travel.

---

## 🧭 Visual Standards & Technical Conventions

### 1. Orientation & Dimensional Integrity
- **Facing Right (+X Axis):**
  - **Lateral (Side View):** The vehicle nose points toward the right edge of the canvas (+X), rear towards the left (-X), tires seated on the baseline with a grounded contact shadow. Resolution: **1024 × 512 RGBA PNG**, plus a **256 × 128 RGBA PNG** thumbnail.
  - **Top-Down (Cenital View):** The vehicle nose points toward the right edge (+X), rear toward the left (-X), cockpit and driver centered. Resolution: **512 × 512 RGBA PNG**.
  - This guarantees seamless visual alignment with existing fleet assets and ensures vehicles face forward on race start grids and garage cards.

### 2. Strict Real-World IP Removal & Livery Policy
- **No Trademarked Logos or Badges:** No manufacturer emblems (e.g. LifeLive, Speedcar, Semog, Peters Autosport, Alfa Racing, Fast & Speed, Škoda, Mitsubishi, Audi) appear anywhere on the liveries.
- **No Commercial Sponsor Markings:** Zero proprietary sponsor logos (oil, tire, energy drink, parts, or apparel brands).
- **Geometric Racing Inspiration:** Livery aesthetics are inspired exclusively by authentic dirt-sprint livery motifs:
  - High-contrast geometric racing stripes, chevron wraps, and color blocks.
  - Vibrant off-road color palettes (fluorescent yellows, electric blues, fiery oranges, matte composites, and bold dual-tone contrasts).
  - Exposed mechanical details: powder-coated tubular spaceframe roll bars, knobby off-road beadlock rims, aluminum wing endplates, and heat-tempered exhaust headers.

---

## 🏎️ Vehicle Roster & Livery Design Mappings

The 15 Autocross vehicles are grouped across the 5 championship tiers:

| Tier | Model ID | Fictional Model Name | Real Inspiration | Body Profile & Visual Identity | Primary Livery Palette |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **T1: Cross Car Junior** | `autocross_ardennes_junior_t1` | Ardennes Junior Cross 600 T1 | LifeLive TN5 Junior | Compact tubular roll cage, rounded fiberglass nose cone, high rear wing | Electric Cyan & Neon Yellow stripes on Dark Charcoal |
| **T1: Cross Car Junior** | `autocross_iberian_furia_t1` | Iberian Furia Junior Cross T1 | Speedcar Xtrem Junior | Angular wedge cowl, dual side cooling pods, single-plane rear spoiler | Fiery Crimson & Sunburst Yellow on Satin Black |
| **T1: Cross Car Junior** | `autocross_cosmo_nova_t1` | Cosmo Nova Junior Cross T1 | Planet Kart Cross K3 | Aerodynamic swept cage, high roof snorkel scoop, flared rear fenders | Pure White, Sky Blue & Navy geometric shards |
| **T2: Cross Car Senior** | `autocross_ardennes_pro_t2` | Ardennes Pro Cross 850 T2 | LifeLive TN11 Senior | Aggressive bi-plane rear wing, wide stance, exposed titanium exhaust | Matte Gunmetal & Acid Lime speed ribbons |
| **T2: Cross Car Senior** | `autocross_iberian_relampago_t2` | Iberian Relampago Cross T2 | Speedcar Wonder | Sharp faceted nosecone, sculpted side radiators, wide endplates | Racing Orange, Jet Black & Metallic Silver accents |
| **T2: Cross Car Senior** | `autocross_lusitania_bravo_t2` | Lusitania Bravo Sport Cross T2 | Semog Bravo Sport | Curving overhead cage bars, dual side scoops, exposed rear coilovers | Royal Blue, Bright White & Gold speed bands |
| **T3: Buggy 1600** | `autocross_petersen_buggy1600_t3` | Petersen Buggy 1600 T3 | Peters Autosport Buggy1600 | Low-slung single-seater, long wheelbase, massive high-mount rear wing | Dutch Orange, Silver & White competition blocks |
| **T3: Buggy 1600** | `autocross_bologna_buggy1600_t3` | Bologna Buggy 1600 T3 | Alfa Racing Buggy1600 | High needle nose, sculpted aerodynamic cockpit, low sidepods | Rosso Corsa Red, Pure White & Gold pinstripes |
| **T3: Buggy 1600** | `autocross_rapid_buggy1600_t3` | Rapid Dynamics Buggy 1600 T3 | Fast & Speed Buggy1600 | Complex exposed tubular frame, rear top radiator, high rake angle | Cobalt Blue, Fluorescent Yellow & Black |
| **T4: TouringAutocross** | `autocross_bohemia_veloce_t4` | Bohemia Veloce Touring AX T4 | Škoda Fabia TAX | Bulging composite widebody hatchback, giant multi-tier rear wing | Rally Green, Pure White & Graphite geometric wrap |
| **T4: TouringAutocross** | `autocross_shinano_tsunami_t4` | Shinano Tsunami Touring AX T4 | Mitsubishi Lancer Evo IX TAX | Muscular rallycross sedan silhouette, vented hood, massive roof scoop | Pearl White, Crimson Red & Carbon Black sunburst |
| **T4: TouringAutocross** | `autocross_vortek_quattro_t4` | Vortek Quattro Touring AX T4 | Audi A4 Quattro TAX | Wide box-flared DTM-style dirt coupe, giant carbon diffuser & wing | Slate Grey, Signal Red & Gloss Black racing stripes |
| **T5: SuperBuggy** | `autocross_petersen_superbuggy_t5` | Petersen SuperBuggy V8 T5 | Peters Autosport SuperBuggy V8 | Massive wide-track open-wheel stance, huge rear V8 engine, dual exhaust | Midnight Blue, Bright White & Neon Orange trim |
| **T5: SuperBuggy** | `autocross_bologna_superbuggy_t5` | Bologna SuperBuggy Twin-Turbo T5 | Alfa Racing SuperBuggy TT | Twin-turbo plumbing, intricate side aero plates, needle nose | Deep Italian Crimson, Carbon Fiber & Metallic Silver |
| **T5: SuperBuggy** | `autocross_rapid_superbuggy_t5` | Rapid Dynamics SuperBuggy Biturbo T5 | Fast & Speed SuperBuggy Biturbo | Extreme dual-intercooler rear pods, high-downforce aero, wide knobbies | Fluorescent Neon Yellow, Stealth Black & Teal |

---

## ⚙️ Backend Models & API Endpoints

### 1. Asset Storage & File Structure
Assets are delivered as standalone compressed PNGs directly into the engine's asset hierarchy:
```
assets/textures/vehicles/
├── laterals/
│   └── autocross/
│       ├── autocross_ardennes_junior_t1.png         (1024x512)
│       ├── autocross_ardennes_junior_t1_thumb.png   (256x128)
│       └── ... (15 models + 15 thumbs)
└── topdown/
    └── autocross/
        ├── autocross_ardennes_junior_t1.png         (512x512)
        └── ... (15 models)
```

### 2. Legal IP Rename Registry Updates
All 15 entries in [`docs/legal/ip_rename_registry.toml`](../docs/legal/ip_rename_registry.toml) update their sprite paths from `"N/A"` to the generated relative paths:
```toml
[cars."autocross_lifelive_tn5_junior"]
new_id = "autocross_ardennes_junior_t1"
lateral_sprite_current = "textures/vehicles/laterals/autocross/autocross_ardennes_junior_t1.png"
lateral_sprite_new = "textures/vehicles/laterals/autocross/autocross_ardennes_junior_t1.png"
topdown_sprite_current = "textures/vehicles/topdown/autocross/autocross_ardennes_junior_t1.png"
topdown_sprite_new = "textures/vehicles/topdown/autocross/autocross_ardennes_junior_t1.png"
```

### 3. Procedural Lateral Fallback Engine Integration
In [`crates/tdrace-app/src/render/lateral.rs`](../crates/tdrace-app/src/render/lateral.rs), `render_specific_body` is extended with match arms covering `autocross_*` models, generating procedural vector buggy/cross-car silhouettes with off-road wheels and exposed roll cages if any raster file is absent.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

### 1. Intellectual Property & Trademark Protection
- Assets are created exclusively through algorithmic code generation within the project (`scripts/generate_autocross_sprites.py`).
- All assets are verified against the real-world IP denylist test suite (`tests/test_ip_denylist.rs`) to ensure no registered marks, trade dress infringements, or proprietary logos enter the binary.

### 2. Client-Side Asset Sandbox Isolation
- Pure desktop racing game architecture; vehicle textures are loaded directly from local read-only filesystem paths without remote network fetch or arbitrary code execution vectors.
- PNG loading validates dimensions, RGBA 8-bit channels, and bounds before passing texture handles to the GPU buffer.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests & Diagnostics
- Verify vehicle asset completeness: `python3 scripts/verify_vehicle_assets.py` (Autocross must show 15 lateral, 15 thumb, 15 topdown).
- Verify IP denylist test gate: `cargo test -p tdrace-app --test test_ip_denylist`
- Verify Keel spec integrity: `keel validate`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Full Autocross Fleet Asset Completeness**
  - [x] **Given** the repository root with `assets/textures/vehicles/`
  - [x] **When** `scripts/verify_vehicle_assets.py` is executed
  - [x] **Then** the Autocross module reports 15 cars, 15 2D Laterals, 15 Thumbs, and 15 Top-Downs with 100% coverage

- **Scenario: Sprites Uniform Right-Facing Orientation**
  - [x] **Given** any generated lateral or top-down sprite in `assets/textures/vehicles/*/autocross/`
  - [x] **When** visual bounds and nose orientation are inspected
  - [x] **Then** the vehicle nose faces toward the right canvas margin (+X direction)

- **Scenario: Real-World IP and Logo Cleanliness**
  - [x] **Given** all 45 generated Autocross PNG textures
  - [x] **When** inspected visually and checked against the IP denylist
  - [x] **Then** zero manufacturer logos, sponsor badges, trademarked names, or proprietary graphics appear

- **Scenario: Garage and Showroom Rendering**
  - [x] **Given** the game application in Garage or Vehicle Selection view
  - [x] **When** navigating through Tier 1 to Tier 5 Autocross vehicles
  - [x] **Then** all 15 models display vibrant, high-resolution lateral cards without blank frames or missing textures

- **Scenario: Engine Procedural Fallback Robustness**
  - [x] **Given** `crates/tdrace-app/src/render/lateral.rs`
  - [x] **When** `render_specific_body` is called with an `autocross_*` model ID in the absence of a texture
  - [x] **Then** a clean procedural tubular buggy silhouette is drawn instead of an empty render

---

## 🔗 Traceability & Codebase Mapping

### Created Files
- `[x]` `scripts/generate_autocross_sprites.py` -> High-resolution procedural sprite generator script.
- `[x]` `assets/textures/vehicles/laterals/autocross/*.png` -> 15 lateral full-resolution textures (1024×512).
- `[x]` `assets/textures/vehicles/laterals/autocross/*_thumb.png` -> 15 lateral thumbnail textures (256×128).
- `[x]` `assets/textures/vehicles/topdown/autocross/*.png` -> 15 top-down textures (512×512).

### Modified Files
- `[x]` `docs/legal/ip_rename_registry.toml` -> Update sprite paths from `"N/A"` to valid asset paths.
- `[x]` `crates/tdrace-app/src/render/lateral.rs` -> Add procedural fallback match arms for `autocross_*`.
- `[x]` `specs/constitution/ROADMAP.md` -> Register Spec 090 in the roadmap.
