# TdRace Surface & Wall Physics Specifications

This document provides a comprehensive technical reference for all **surface types** (track, off-track, and dynamic hazards) and **wall barrier types** implemented in **TdRace**.

---

## 1. Overview & Type Counts

TdRace implements a modular per-wheel physics engine ([`crates/wheelbase`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs)) and track geometry system ([`crates/arcade-race-core`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core/src/track/geometry.rs)) supporting:

* **11 Surface Types** ([`SurfaceType`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L4-L29)): Full split-$\mu$ per-wheel sampling across tarmac, dirt, gravel, sand, mud, snow, ice, oil, water, kerbs, and grass.
  * **7 Global Off-Track Terrains** (`OFF_TRACK_TYPES`): `Grass`, `Sand`, `Dirt`, `Asphalt`, `Mud`, `Snow`, `Gravel`.
  * **5 On-Track Dynamic Hazards** (`is_on_track_hazard`): `Water`, `Oil`, `Ice`, `Mud`, `Snow`.
* **4 Wall & Barrier Types** ([`BarrierType`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core/src/track/geometry.rs#L146-L157)): `Concrete`, `Steel`, `TireWall`, `CurbWall`.
* **3 Static Obstacle Geometries** ([`ObstacleShape`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core/src/track/geometry.rs#L402-L408)): `Circle`, `Box`, `Polygon`.

---

## 2. Surface Types Reference

### 2.1 Physics Parameter Matrix

Each surface defines distinct friction, rolling resistance, aerodynamic/viscous drag multipliers, and particle emissions in [`SurfaceType`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L33-L144):

| Surface Type | Friction ($\mu$) | Rolling Resistance Multiplier | Surface Drag Multiplier | Tire Smoke | Debris Roost | Water Splash | Layer Default | Role & Use Case |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| **`Asphalt`** | **`1.00`** | `1.0×` | `1.00×` | Yes | No | No | `BelowTrack` | Standard dry tarmac; optimal grip baseline, full tire smoke on slip. |
| **`Curb`** | **`0.88`** | `1.3×` | `1.05×` | Yes | No | No | `BelowTrack` | Apex kerb / rumble strip; slight vibration, high grip with mild drag. |
| **`Dirt`** | **`0.78`** | `1.2×` | `1.10×` | No | Yes | No | `BelowTrack` | Compacted clay / gravel rally track; predictable sliding and drift control. |
| **`Gravel`** | **`0.70`** | `2.5×` | `1.25×` | No | Yes | No | `BelowTrack` | Loose stone gravel stage / runoff; moderate grip with heavy stone roost. |
| **`Mud`** | **`0.52`** | `6.5×` | `3.20×` | No | Yes | No | `AboveTrack` | Viscous mud bog; heavy deceleration drag, low lateral bite, brown roost. |
| **`Grass`** | **`0.45`** | `18.0×` | `2.20×` | No | Yes | No | `BelowTrack` | Standard off-track runoff; heavy rolling resistance penalizing corner cuts. |
| **`Snow`** | **`0.34`** | `3.0×` | `1.60×` | No | Yes | No | `AboveTrack` | Packed/powder snow; slippery winter rallying, white roost plumes. |
| **`Sand`** | **`0.30`** | `30.0×` | `4.50×` | No | Yes | No | `BelowTrack` | Deep gravel / sand trap; severe vehicle deceleration trap, sand rooster tails. |
| **`Water`** | **`0.22`** | `3.5×` | `2.00×` | No | No | Yes | `AboveTrack` | Standing puddle / wet patch hazard; aquaplaning risk, aqua spray plumes. |
| **`Oil`** | **`0.12`** | `0.8×` | `0.95×` | No | No | No | `AboveTrack` | Oil slick hazard; extreme spin hazard, breaks rear traction instantly. |
| **`Ice`** | **`0.08`** | `0.4×` | `0.90×` | No | No | No | `AboveTrack` | Frozen sheet; near-zero traction, near-frictionless gliding with no braking. |

---

### 2.2 Detailed Surface Behavior & Visual Profiles

#### 1. Asphalt ([`SurfaceType::Asphalt`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L9))
* **Characteristics**: Baseline racing surface with maximum lateral and longitudinal grip ($\mu = 1.0$).
* **Visual Rendering**: Dark charcoal (`#292B33`), rendered with white border lines (`#F5F5FA`) and dashed centerline stripes on flat track, or 3-tone lighting gradient bands on banked corners (`#1F2129` low apron, `#292B33` mid, `#3D404A` high rim).
* **FX**: Generates dark rubber skid marks and dense white tire smoke clouds during hard braking or power oversteer.

#### 2. Curb ([`SurfaceType::Curb`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L13))
* **Characteristics**: Track edge rumble strips ($\mu = 0.88$, $1.3\times$ rolling resistance).
* **Visual Rendering**: Alternating red (`#EA262E`) and white (`#FAFAFF`) kerb teeth with 1.35m standard width.
* **FX**: Rubber skid marks and tire smoke on aggressive clipping.

#### 3. Dirt ([`SurfaceType::Dirt`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L11))
* **Characteristics**: Compacted dirt/clay rallying surface ($\mu = 0.78$, $1.2\times$ rolling resistance).
* **Visual Rendering**: Warm clay brown (`#7A5938`) with darker tire groove tracks (`#614229`) and light border edges (`#997547`). On banked oval/rally turns, renders into 3 dynamic gradient bands (moist bottom groove to dry high cushion).
* **FX**: Emits dual-tone dirt and clay roost particles backward from driven wheels.

#### 4. Gravel ([`SurfaceType::Gravel`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L29))
* **Characteristics**: Loose stone gravel track / rally runoff ($\mu = 0.70$, $2.5\times$ rolling resistance, $1.25\times$ surface drag). Features loose-surface progressive breakaway and drifting.
* **Visual Rendering**: Flint grey-stone body (`#858075`), darker stone pebble groove tracks (`#666159`), light flint border edges (`#A6A194`), and dedicated off-track backdrop (`#948F85`).
* **FX**: Emits dense grey-stone pebble and gravel roost particles backward when slipping.

#### 5. Mud ([`SurfaceType::Mud`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L25))
* **Characteristics**: Deep deformable mud ($\mu = 0.52$, $6.5\times$ rolling resistance, $3.2\times$ surface drag).
* **Visual Rendering**: Deep dark brown sludge (`#52381F`) with dark rut seams (`#3D2914`).
* **FX**: Heavy dark-brown muddy spray plumes and wheel spray.

#### 6. Grass ([`SurfaceType::Grass`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L15))
* **Characteristics**: Natural turf runoff ($\mu = 0.45$, $18.0\times$ rolling resistance, $2.2\times$ surface drag).
* **Visual Rendering**: Forest green (`#2E7A3D`) backdrop with darker green runoff patches (`#246630`).
* **FX**: Green turf clumps and brown earth debris roost when tires lose traction.

#### 7. Snow ([`SurfaceType::Snow`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L27))
* **Characteristics**: Packed winter snow ($\mu = 0.34$, $3.0\times$ rolling resistance, $1.6\times$ drag).
* **Visual Rendering**: Crisp white-blue snow blanket (`#EBF0F7`) with frosty blue edge lines (`#D1DBEB`) and twin packed tire ruts.
* **FX**: Powder snow roost plumes trailing sliding wheels.

#### 8. Sand ([`SurfaceType::Sand`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L17))
* **Characteristics**: Deep runaway sand / gravel trap ($\mu = 0.30$, $30.0\times$ rolling resistance, $4.5\times$ drag). Acts as an emergency arrestor bed.
* **Visual Rendering**: Golden sand (`#D9BD7A`) with darker tan border lines (`#BFA361`).
* **FX**: Large golden-yellow sand rooster tails thrown high during wheelspin.

#### 9. Water ([`SurfaceType::Water`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L19))
* **Characteristics**: Standing water hazard ($\mu = 0.22$, $3.5\times$ rolling resistance, $2.0\times$ surface drag).
* **Visual Rendering**: Translucent cyan-blue puddle (`#2E94E0` at 80% opacity) with bright aqua ripple borders (`#66D1FF`).
* **FX**: Emits multi-particle water splash droplet plumes and white foam sprites scaled dynamically with vehicle velocity.

#### 10. Oil ([`SurfaceType::Oil`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L21))
* **Characteristics**: Petroleum spill hazard ($\mu = 0.12$, $0.8\times$ rolling resistance, $0.95\times$ drag). Induces sudden spinouts.
* **Visual Rendering**: Dark iridescent black slick (`#1F1F26` at 95% opacity) with purple-sheen outer contour borders (`#594066`).
* **FX**: Zero smoke; sudden loss of lateral resistance causes instant high-RPM wheelspin.

#### 11. Ice ([`SurfaceType::Ice`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L23))
* **Characteristics**: Sheet black ice ($\mu = 0.08$, $0.4\times$ rolling resistance, $0.90\times$ drag). Near zero steering or braking authority.
* **Visual Rendering**: Glacial frost blue (`#D9EBF9` at 95% opacity) with icy highlight rim lines (`#A6D1F2`).
* **FX**: Suppresses standard rubber smoke; creates continuous frictionless sliding.

---

### 2.3 Split-$\mu$ Per-Wheel Physics Architecture

TdRace executes **per-wheel independent surface queries** at 60 Hz (with 2 physics sub-steps):

$$\vec{F}_{\text{friction\_max}, i} = \mu(\text{surface}_i) \cdot F_{z, i}$$

1. Each wheel position $\vec{p}_{\text{wheel}, i}$ samples the track ribbon, elevated bridges, and surface zones via [`SurfaceSampler`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/surface.rs#L194-L197).
2. Normal loads $F_{z, i}$ account for static weight distribution, dynamic longitudinal acceleration squat/dive, lateral cornering roll, aerodynamic downforce ($F_{\text{downforce}} \propto v^2$), and cross-slope banking inclination.
3. The **Pacejka Magic Formula** combined-slip solver determines individual longitudinal driving/braking forces ($F_{x, i}$) and lateral cornering forces ($F_{y, i}$) constrained by each wheel's local surface friction circle.

---

## 3. Wall & Barrier Types Reference

### 3.1 Barrier Physics Parameter Matrix

Defined in [`crates/arcade-race-core/src/track/geometry.rs`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core/src/track/geometry.rs#L146-L220) and resolved in [`crates/arcade-race-core/src/collision/wall.rs`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core/src/collision/wall.rs):

| Barrier Type | Restitution ($e$) | Coulomb Friction ($\mu$) | Scraping Decel ($a_{\text{scrape}}$) | Snag Torque Factor | Energy Absorption Factor | Primary Collision Particle FX |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`Concrete`** | **`0.65`** | **`0.32`** | **`9.0 m/s²`** (~0.9g) | **`0.12`** | **`0.10`** (Stiffest, 90% energy to car) | Orange/Yellow Sparks |
| **`Steel`** | **`0.42`** | **`0.45`** | **`14.0 m/s²`** (~1.4g) | **`0.25`** | **`0.40`** (Deformable, absorbs 40%) | High-Velocity Spark Plumes |
| **`TireWall`** | **`0.18`** | **`0.80`** | **`24.0 m/s²`** (~2.4g) | **`0.45`** | **`0.75`** (Cushion, absorbs 75%) | White Rubber Smoke Puffs |
| **`CurbWall`** | **`0.30`** | **`0.40`** | **`7.0 m/s²`** (~0.7g) | **`0.10`** | **`0.25`** (Low mount, absorbs 25%) | Low Sparks / Scraping |

---

### 3.2 Detailed Barrier Profiles & Mechanics

#### 1. Concrete Barrier ([`BarrierType::Concrete`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core/src/track/geometry.rs#L150))
* **Role**: Permanent perimeter wall on street circuits and high-speed superspeedway perimeters.
* **Physics Dynamics**: High bounce elasticity ($e = 0.65$), low sliding friction ($\mu = 0.32$), and low snag torque ($0.12$). Allows high-speed glancing wall-rides with minimal yaw deflection, but stiff chassis impacts ($90\%$ impact energy transferred into vehicle damage).
* **Visual Profile**: 0.64m wide solid concrete block (`#CCCECE`), top highlight bevel line (`#F0F0F0`), and 2.5D drop shadow (0.70m thickness).

#### 2. Steel Armco Guardrail ([`BarrierType::Steel`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core/src/track/geometry.rs#L152))
* **Role**: Standard road-course and hillclimb guardrail barrier.
* **Physics Dynamics**: Moderate restitution ($e = 0.42$), higher friction ($\mu = 0.45$), and corrugated post snagging ($0.25$ snag factor, $14.0\,\text{m/s}^2$ scraping resistance). Absorbs $40\%$ of impact energy.
* **Visual Profile**: Metallic rail beam (`#BFC7D9`), dark center groove (`#66707A`), steel support posts (`#66707A`, 0.22m radius) rendered every 2.5m, and 0.50m drop shadow.
* **FX**: Generates dense showers of sparks ($1.3\times$ multiplier) on high-speed scrapes.

#### 3. Tire Wall Stack ([`BarrierType::TireWall`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core/src/track/geometry.rs#L154))
* **Role**: High-risk runoff corner protection and chicane apex barriers.
* **Physics Dynamics**: Highly damped impact cushion ($e = 0.18$), extremely high rubber friction ($\mu = 0.80$), violent scraping braking deceleration ($24.0\,\text{m/s}^2 \approx 2.45\text{g}$), and aggressive corner snagging ($0.45$ snag torque factor). Absorbs $75\%$ of impact energy, protecting vehicle structural health while arresting speed rapidly.
* **Visual Profile**: 0.84m wide heavy matte rubber body (`#24262B`), center binding tension strap (`#616673`), stacked tire division ribs every 1.0m, and 0.85m drop shadow.
* **FX**: Puffs dense white rubber smoke instead of sparks upon impact.

#### 4. Curb Wall ([`BarrierType::CurbWall`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core/src/track/geometry.rs#L156))
* **Role**: Low concrete/plastic boundary wall delineating track limits or pit entrances.
* **Physics Dynamics**: Low restitution ($e = 0.30$), low snagging torque ($0.10$), and gentle scraping deceleration ($7.0\,\text{m/s}^2$).
* **Visual Profile**: Compact 0.30m red curb profile (`#EA262E`) with 0.40m drop shadow.

---

### 3.3 Wall Collision & Scraping Resolution Algorithm

When a vehicle contacts a wall segment, [`crates/arcade-race-core/src/collision/wall.rs`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core/src/collision/wall.rs) executes a 3-stage rigid body solver:

1. **Penetration Pushout**: Displaces the vehicle along the collision normal $\hat{n}$ to eliminate boundary overlap:
   $$\vec{p}_{\text{car}} \leftarrow \vec{p}_{\text{car}} + \hat{n} \cdot (d_{\text{penetration}} + 0.002)$$
2. **Normal Impulse Resolution**: Computes contact velocity $\vec{v}_c = \vec{v} + \vec{\omega} \times \vec{r}$ and applies normal bounce impulse $j_n$:
   $$j_n = \frac{-(1 + e) v_n}{\frac{1}{m} + \frac{(\vec{r} \times \hat{n})^2}{I}}$$
3. **Tangential Friction & Wall-Scraping Resistance**:
   * Coulomb impact friction: $j_{t, \text{impact}} = \mu_{\text{wall}} \cdot j_n$
   * Continuous scraping brake impulse: $j_{t, \text{scrape}} = m \cdot a_{\text{scrape}} \cdot \Delta t_{\text{sub}} \cdot \text{clamp}\left(\frac{v_t}{1.0}, 0, 1\right)$
   * Snag torque applies a yaw deflection moment to the chassis:
     $$\tau_{\text{snag}} = \left(\vec{r} \times \vec{j}_t\right) \cdot \left(\text{ratio}_{\text{impact}} + (1 - \text{ratio}_{\text{impact}}) \cdot \text{snag\_factor}\right)$$

---

## 4. Track Layering & Elevation Interaction

1. **Ground Layer (`SurfaceLayer::BelowTrack`)**: Base off-track terrains (grass, sand runoff, dirt paths, gravel beds) rendered beneath the track spline ribbon.
2. **Track Ribbon**: Primary Catmull-Rom spline drivable surface (asphalt, dirt, gravel, curbs) with elevation contours and banking cross-slopes.
3. **Hazard Overlays (`SurfaceLayer::AboveTrack`)**: Dynamic surface hazards (water puddles, oil slicks, ice patches, mud bogs) rendered over the road ribbon that override underlying physics.
4. **Elevated Overpass Bridges ($\text{elevation} \ge 0.6\,\text{m}$)**: Concrete deck undertrays, bridge drop shadows, elevated curbs, and mounted steel guardrails.
5. **Jump Ramps & Airborne Dynamics**: Mid-air vehicles ($\text{elevation} > 0$) have ground contact scaled down to $0$, suppressing surface friction, skidmarks, and ground wheel particles until touchdown (which triggers landing dust bursts).
