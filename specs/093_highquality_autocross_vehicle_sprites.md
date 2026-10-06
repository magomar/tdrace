---
type: Feature Spec
template: feature
title: "High-Quality Autocross Vehicle Sprites"
description: "High-fidelity 2D lateral (1024x512), thumbnail (256x128), and top-down (512x512) sprites for all 15 Continental Autocross vehicles, facing right (+X), inspired by original models before rebranding, with strict zero-IP compliance (no words, no numbers, maximum 3 bodywork colors)."
status: implemented
verified: { by: "human:mario", at: "2026-10-05T21:02:53Z", hash: "fb132487bc0c" }
created: 2026-10-05
generated: { by: agent/antigravity, at: 2026-10-05T21:00:00Z }
depends_on:
  - "050"
supersedes:
  - "090"
---

# Feature Spec 093: High-Quality Autocross Vehicle Sprites 🏎️🎨✨

When the FIA Autocross championship module was introduced in [Spec 050](050_fia_autocross_championship_and_vehicle_roster.md), 15 off-road competition machines spanning 5 tiers (Cross Car Junior, Cross Car Senior, Buggy 1600, TouringAutocross, and SuperBuggy) were integrated into the simulation physics, sound synthesis, and catalog. [Spec 090](090_autocross_fictional_vehicle_sprites_and_livery_pipeline.md) introduced an initial asset pipeline to populate textures; however, it relied on rudimentary 2D procedural vector primitives (simple lines, polygons, and circles) generated via script. As a result, the Autocross fleet lacks the volumetric lighting, metallic depth, realistic mechanical chassis detail, and rich visual polish exhibited by the rest of the game's high-quality fleet (GT, NASCAR, Rally, Extreme Off-Road, and Classic modules).

This specification establishes the blueprint for authoring and integrating **high-quality, high-fidelity 2D sprite textures** for all 15 Autocross vehicles (45 total image assets: 15 laterals at 1024×512, 15 thumbnails at 256×128, and 15 top-downs at 512×512). The artwork takes structural and aerodynamic inspiration from the authentic real-world machines prior to rebranding, while strictly adhering to the project's intellectual property removal mandate defined in [Spec 047](047_fictional_branding_and_realworld_ip_removal_for_steam_release.md): complete omission of words and numbers, a maximum 3-color bodywork paint rule, debadged surfaces, and uniform right-facing (+X direction) orientation.

---

## 🎯 Objectives & Success Criteria

1. **High Visual Caliber**: Replace flat geometric placeholder graphics with high-resolution, pseudo-3D rendered sprites featuring volumetric shading, realistic highlights, ambient occlusion, grounded drop shadows, and authentic mechanical detail (exposed tubular spaceframes, beadlock knobby tires, coilover dampers, and aerodynamic wings).
2. **Pre-Rebranding Inspiration**: Vehicle profiles, proportions, stances, and aero appendages directly echo the authentic competition architectures of the original machines before rebranding (LifeLive, Speedcar, Semog, Peters Autosport, Alfa Racing, Fast & Speed, Škoda, Mitsubishi, and Audi) without infringing on trademarks.
3. **Spec 047 Zero-IP & Clean Livery Rules**:
   - **No Words or Lettering**: Zero sponsor names, manufacturer marks, driver monikers, series labels, or brand typography on bodywork or glass.
   - **No Numbers**: No competition racing numbers or digits baked into the sprite graphics (player custom numbers are handled dynamically by [Spec 092](092_player_racing_number_customization_and_persistent_nonconflicting_bot_rosters.md)).
   - **Maximum 3 Bodywork Colors**: Body paint schemes use at most 3 distinct colors on painted panels (excluding neutral functional parts: black rubber tires, metallic rims, steel cages, dampers, glass/mesh, and titanium exhausts).
4. **Strict Right-Facing Orientation (+X Axis)**:
   - Lateral side views point nose-forward to the right (+X), rear to the left (-X), grounded on the lower canvas margin.
   - Top-down views point forward to the right (+X / $0^\circ$ heading), centered on the physics centroid.

---

## 🗺️ User Flow & Interface Design

### 1. Interactive Garage & Vehicle Selection Turntable
In the Garage, Career Selection, and Showroom menus:
- When browsing the Autocross module across Tiers 1 through 5, [`get_vehicle_lateral_texture`](../crates/tdrace-app/src/render/vehicle_assets.rs) displays the full-resolution 1024×512 lateral texture from `textures/vehicles/laterals/autocross/{model_id}.png`.
- The vehicle sits on a grounded turntable facing **RIGHT (+X direction)**, matching the orientation of GT, NASCAR, Rally, Karting, Extreme Off-Road, and Classic vehicles.
- Vehicle selection ribbons, dossier cards, and summary dialogs consume the crisp 256×128 thumbnail (`_thumb.png`).

### 2. In-Race Top-Down (Cenital) View
During live sprint heats on European dirt circuits:
- The orthographic top-down sprite from `textures/vehicles/topdown/autocross/{model_id}.png` renders centered on the vehicle's physics centroid.
- The default sprite orientation faces **RIGHT (+X direction)** corresponding to yaw $\theta = 0\text{ rad}$, seamlessly rotating with physics heading.
- Exposed cockpits render pilot helmets, steering columns, and protective mesh or transparent windshields. Open-wheel cross cars and buggies clearly display wide-track knobby beadlock tires and high-mount rear wings.

---

## 🧭 Visual Standards & Remediation Criteria

### 1. Strict Spec 047 Compliance
- **Debadged & Logo-Free**: All manufacturer badges, emblems, sponsor decals, tire brand lettering, and fuel/oil marks are strictly prohibited.
- **Zero Inscriptions / Typography**: No text, words, or character glyphs may appear on wings, sidepods, hoods, or windscreens.
- **Zero Numeric Digits**: No baked numbers (`#1`, `#7`, etc.) appear on the chassis, roof fin, or wing endplates.
- **3-Color Bodywork Rule**: The painted exterior panels (nosecone, cowl, sidepods, roof scoop, mudguards, and rear wing elements) are limited to a maximum of 3 colors (e.g., base primary, high-contrast secondary block, and accent stripe). Functional mechanical components (spaceframe tubing, shock springs, wheels, tires, exhaust headers, and brake calipers) do not count against this 3-color palette.

### 2. Dimensional & Processing Standards
- **Chroma-Key Processing Pipeline**: Raw renders authored against pure magenta background (`#FF00FF`) are processed using `scripts/process_vehicle_sprites.py` to achieve anti-aliased alpha transparency without fringing:
  - **Lateral Canvas**: 1024 × 512 RGBA PNG, vehicle nose to the right (+X), tires grounded at `Y = 500`.
  - **Thumbnail Canvas**: 256 × 128 RGBA PNG, downscaled with high-quality Lanczos resampling.
  - **Top-Down Canvas**: 512 × 512 RGBA PNG, vehicle nose pointing right (+X), centered at `(256, 256)`.

---

## 🏎️ Vehicle Roster & Visual Design Matrix

The 15 Autocross vehicles are organized across the 5 competitive tiers:

| Tier | Model ID | Fictional Model Name | Original Inspiration | Visual Archetype & Structural Identity | 3-Color Bodywork Palette |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **T1: Junior** | `autocross_ardennes_junior_t1` | Ardennes Junior Cross 600 T1 | LifeLive TN5 Junior | Compact tubular roll cage, curved aerodynamic fiberglass nose, single-plane rear wing | Electric Cyan, Dark Charcoal, Neon Yellow |
| **T1: Junior** | `autocross_iberian_furia_t1` | Iberian Furia Junior Cross T1 | Speedcar Xtrem Junior | Angular wedge cowl, dual radiator sidepods, swept rear spoiler | Crimson Red, Satin Black, Sunburst Yellow |
| **T1: Junior** | `autocross_cosmo_nova_t1` | Cosmo Nova Junior Cross T1 | Planet Kart Cross K3 Junior | High roof snorkel scoop, flared rear fenders, narrow cockpit | Pure White, Sky Blue, Navy Blue |
| **T2: Senior** | `autocross_ardennes_pro_t2` | Ardennes Pro Cross 850 T2 | LifeLive TN11 Senior | Bi-plane rear wing, aggressive front nose rake, exposed titanium exhaust | Gunmetal Grey, Acid Lime, Signal White |
| **T2: Senior** | `autocross_iberian_relampago_t2` | Iberian Relampago Cross T2 | Speedcar Wonder | Faceted nosecone, sculpted radiator cowls, wide wing endplates | Racing Orange, Jet Black, Silver Metallic |
| **T2: Senior** | `autocross_lusitania_bravo_t2` | Lusitania Bravo Sport Cross T2 | Semog Bravo Sport | Curving overhead cage bars, dual side scoops, wide knobby rear stance | Royal Blue, Bright White, Gold Yellow |
| **T3: Buggy 1600** | `autocross_petersen_buggy1600_t3` | Petersen Buggy 1600 T3 | Peters Autosport Buggy1600 | Low-slung single-seater, long wheelbase, massive high-mount rear wing | Dutch Orange, Slate Grey, Pure White |
| **T3: Buggy 1600** | `autocross_bologna_buggy1600_t3` | Bologna Buggy 1600 T3 | Alfa Racing Buggy1600 | Needle nosecone, sculpted aerodynamic cockpit pod, low-profile sidepods | Rosso Red, Pure White, Carbon Black |
| **T3: Buggy 1600** | `autocross_rapid_buggy1600_t3` | Rapid Dynamics Buggy 1600 T3 | Fast & Speed Buggy1600 | Complex exposed tubular trellis, top-mounted rear radiator, high rake angle | Cobalt Blue, Fluorescent Yellow, Matte Black |
| **T4: Touring AX** | `autocross_bohemia_veloce_t4` | Bohemia Veloce Touring AX T4 | Škoda Fabia TAX | Bulging composite widebody hatchback, giant multi-element dirt wing | Rally Green, Pure White, Graphite Grey |
| **T4: Touring AX** | `autocross_shinano_tsunami_t4` | Shinano Tsunami Touring AX T4 | Mitsubishi Lancer Evo IX TAX | Muscular rallycross sedan silhouette, vented hood, roof air scoop | Pearl White, Crimson Red, Carbon Black |
| **T4: Touring AX** | `autocross_vortek_quattro_t4` | Vortek Quattro Touring AX T4 | Audi A4 Quattro TAX | Box-flared DTM-style dirt coupe, giant carbon diffuser & endplate wing | Nardo Grey, Signal Red, Gloss Black |
| **T5: SuperBuggy** | `autocross_petersen_superbuggy_t5` | Petersen SuperBuggy V8 T5 | Peters Autosport SuperBuggy V8 | Massive wide-track open-wheel stance, huge exposed rear V8 engine, dual exhaust | Deep Navy, Bright White, Neon Orange |
| **T5: SuperBuggy** | `autocross_bologna_superbuggy_t5` | Bologna SuperBuggy Twin-Turbo T5 | Alfa Racing SuperBuggy TT | Twin-turbo plumbing, intricate side aero bargeboards, needle nose | Italian Crimson, Carbon Fiber, Titanium Silver |
| **T5: SuperBuggy** | `autocross_rapid_superbuggy_t5` | Rapid Dynamics SuperBuggy Biturbo T5 | Fast & Speed SuperBuggy Biturbo | Extreme dual-intercooler rear pods, high-downforce aero, wide knobbies | Fluorescent Yellow, Stealth Black, Bright Cyan |

---

## ⚙️ Backend Models & API Endpoints

### 1. Asset Storage & File Structure
High-quality sprites replace existing assets under the canonical engine asset hierarchy:
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

### 2. Processing Pipeline Integration
Assets generated from raw source renders are processed via `scripts/process_vehicle_sprites.py`:
```bash
python3 scripts/process_vehicle_sprites.py \
  --module autocross \
  --vehicle-id <model_id> \
  --lateral-raw <raw_lateral_path> \
  --topdown-raw <raw_topdown_path>
```
The script Chroma-keys out the magenta background (`#FF00FF`), crops to the bounding box, pads into the target resolution, downsamples the thumbnail, and validates right-facing orientation.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

### 1. Intellectual Property & Trademark Protection
- **No Proprietary Marks**: Assets are verified against `tests/test_ip_denylist.rs` to guarantee no real-world maker, series, sponsor, or model name is present.
- **Independent Art Assets**: Vehicle silhouettes are distinct artistic expressions inspired by motorsport engineering standards rather than direct copies of protected OEM automotive designs.

### 2. Client-Side Asset Sandbox Isolation
- Desktop native application architecture; sprites are static local textures loaded through Macroquad GPU texture loaders.
- PNG headers and image payloads are verified for valid dimensions, 8-bit RGBA depth, and sane buffer lengths before allocating texture memory.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests & Quality Gates
- Verify vehicle asset completeness: `python3 scripts/verify_vehicle_assets.py` (Autocross must report 15 laterals, 15 thumbnails, 15 top-downs).
- Verify IP denylist test gate: `cargo test -p tdrace-app --test test_ip_denylist`
- Verify Keel spec and framework health: `keel validate . && keel doctor .`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Full Autocross Fleet Asset Completeness**
  - [x] **Given** the repository root with `assets/textures/vehicles/`
  - [x] **When** `scripts/verify_vehicle_assets.py` is executed
  - [x] **Then** the Autocross module reports 15 cars, 15 2D Laterals (1024×512), 15 Thumbs (256×128), and 15 Top-Downs (512×512) with 100% coverage

- **Scenario: Sprites Uniform Right-Facing Orientation**
  - [x] **Given** any generated lateral or top-down sprite in `assets/textures/vehicles/*/autocross/`
  - [x] **When** visual bounds and nose orientation are inspected
  - [x] **Then** the vehicle nose faces toward the right canvas margin (+X direction) in both lateral and top-down views

- **Scenario: Strict Zero-IP Decal, Word, and Number Cleanliness**
  - [x] **Given** all 45 Autocross vehicle textures
  - [x] **When** inspected visually and audited against legal requirements
  - [x] **Then** zero manufacturer logos, sponsor badges, trademarked text, words, or racing numbers appear anywhere on the vehicles

- **Scenario: Maximum 3-Color Bodywork Palette Enforcement**
  - [x] **Given** the painted body panels of any Autocross vehicle sprite
  - [x] **When** livery colors are sampled and counted
  - [x] **Then** no vehicle exceeds 3 distinct bodywork paint colors, excluding neutral mechanical elements

- **Scenario: Visual Rendering Quality in Garage and In-Race Views**
  - [x] **Given** the game application in Garage or live race view
  - [x] **When** selecting and driving any of the 15 Autocross vehicles
  - [x] **Then** the vehicles render with high-fidelity volumetric shading, grounded shadows, realistic mechanical detail, and zero alpha fringing

---

## 🔗 Traceability & Codebase Mapping

### Created / Modified Files
- `[x]` `assets/textures/vehicles/laterals/autocross/*.png` -> 15 lateral full-resolution textures (1024×512).
- `[x]` `assets/textures/vehicles/laterals/autocross/*_thumb.png` -> 15 lateral thumbnail textures (256×128).
- `[x]` `assets/textures/vehicles/topdown/autocross/*.png` -> 15 top-down textures (512×512).
- `[x]` `docs/legal/ip_rename_registry.toml` -> Verification of sprite paths and fictional model registrations.
- `[x]` `specs/constitution/ROADMAP.md` -> Inclusion of Spec 093 under Phase 2.
- `[x]` `specs/index.md` -> Progressive index synchronization.
