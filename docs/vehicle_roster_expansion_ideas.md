# Vehicle Roster Expansion: Ideas, Variants & Archetypes

**Document Status:** PROPOSED / EXPLORATORY  
**Author:** Antigravity Pairing Assistant  
**Date:** September 14, 2026  
**Target Crates:**  
1. [`crates/wheelbase`](file:///home/mario/workspace/games/tdrace/crates/wheelbase) (Physics, Pacejka Tire Model, Surface Interactions, 2.5D Jump Ballistics)  
2. [`crates/tdrace-app`](file:///home/mario/workspace/games/tdrace/crates/tdrace-app) (Vehicle Roster, Render Models, Audio Profiles, Module Definitions)  
3. [`crates/arcade-race-core`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core) (OBB Collisions, Track Splines, Surface Zones)  

---

## 1. Executive Summary

This document captures prospective vehicle concepts, experimental archetypes, and class variants for **TdRace**. The goal is to dramatically expand gameplay diversity, tactical line choices, and arcade fun by leveraging the game engine's advanced physical features:
- **Pacejka 'Magic Formula' Slip Curves:** Non-linear tire grip, slip angles, and progressive breakaway.
- **Dynamic Weight Transfer:** Authentic longitudinal squat/dive and lateral body roll affecting tire load.
- **Surface Reaction Modeling:** Dedicated friction and drag scaling across `Asphalt`, `Dirt`, `Curb`, `Grass`, `Sand`, `Water` (hydroplaning), `Oil`, and `Ice`.
- **2.5D Elevation & Jump Ramp Ballistics:** Parabolic flight, air-scale projection, suspension compression, and bounce.
- **Drive Layout Differentiation:** Front-Wheel Drive (`drive_bias: 1.0`), All-Wheel Drive (`drive_bias: 0.5`), and Rear-Wheel Drive (`drive_bias: 0.0`).

---

## 2. Architectural & Physics Levers

Each proposed vehicle is parameterized using the existing [`CarConfig`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/config.rs) and [`TireConfig`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/config.rs) models in `wheelbase`:

```rust
pub struct CarConfig {
    pub mass: f32,
    pub inertia: f32,
    pub wheelbase: f32,
    pub track_width: f32,
    pub cg_to_front: f32,
    pub cg_to_rear: f32,
    pub cg_height: f32,                    // Controls body roll and rollover susceptibility
    pub max_engine_force: f32,
    pub max_brake_force: f32,
    pub handbrake_force: f32,
    pub drive_bias: f32,                    // 0.0 = pure RWD, 0.5 = 50/50 AWD, 1.0 = pure FWD
    pub top_speed_mps: f32,
    pub max_steer_angle: f32,
    pub weight_transfer_longitudinal: f32,  // Pitch: squat under acceleration / dive under braking
    pub weight_transfer_lateral: f32,       // Roll: chassis lean during cornering
    pub downforce_coefficient: f32,         // Speed-squared aero vertical load (Cl * A)
    pub tire: TireConfig,
    pub assists: DriverAssistsConfig,
}
```

---

## 3. Vehicle Concepts by Category

### 3.1 Off-Road & Jump Brawlers

#### 1. Baja Sand Buggy / Sand Rail
* **Concept:** Lightweight tubular spaceframe dune buggy with an exposed rear air-cooled flat-four, long-travel trailing arm suspension, and oversized paddle/knobby tires.
* **Physics Levers:**
  * **Mass / CG:** Ultra-light ~650 kg, rear-biased CG (`cg_to_front: 0.9`, `cg_to_rear: 1.5`), elevated ride height (`cg_height: 0.46`).
  * **Drivetrain:** Pure RWD (`drive_bias: 0.0`) with aggressive wheelspin off the line.
  * **Terrain Interaction:** Exceptional compliance on `Dirt` and `Sand`; immune to deep sand drag penalties.
  * **Ballistics:** High launch velocity over jump ramps with soft, highly damped landings.
* **Visuals & Audio:** Exposed roll-cage tubing, visible rear engine block, fluttering whip antenna with sand flag; raspy, high-revving boxer engine audio with distinct gear whine.
* **Gameplay Rationale:** High-agility dirt specialist that rewards bold curb-hopping and sand-trap shortcutting on mixed-surface and rallycross circuits.

#### 2. Stadium Super Truck (SST) / Trophy Truck
* **Concept:** Purpose-built desert/stadium racing truck with 800+ BHP V8, fiberglass silhouette body, and colossal suspension travel.
* **Physics Levers:**
  * **Mass / CG:** ~1350 kg with a significantly elevated CG (`cg_height: 0.56`).
  * **Weight Transfer:** Extreme lateral roll multiplier (`weight_transfer_lateral: 1.6`).
  * **Handling:** Lifts the inside front tire off the ground under aggressive cornering; prone to bicycle-wheeling on tight asphalt curbs, but indestructible over jump ramps.
* **Visuals & Audio:** Massive ground clearance, flared wheel arches, visible heavy-duty skid plates; booming naturally aspirated pushrod V8 rumble with throaty exhaust barking.
* **Gameplay Rationale:** Thrilling spectacle in close pack racing. The dramatic body roll and jump-landing bounce force drivers to balance aggressive curb attacks against rollover risk.

#### 3. Monster Truck ("Crusher V8")
* **Concept:** Heavyweight entertainment behemoth with 66-inch agricultural tires, tubular chassis, and supercharged methanol big-block V8.
* **Physics Levers:**
  * **Mass / Inertia:** Heavy ~2200 kg chassis with high rotational inertia (`inertia: 3200.0`).
  * **Dimensions:** Enormous track width (2.2 m), wide wheelbase (3.2 m), and sky-high CG (`cg_height: 0.78`).
  * **Drivetrain:** Locked 50/50 4WD (`drive_bias: 0.5`).
  * **Terrain Trait:** Complete immunity to grass, gravel, and curb slowdowns—bulldozes through off-track areas without speed loss.
  * **Tradeoff:** Slow steering response, wide turning circle, and heavy braking distance.
* **Visuals & Audio:** Massive tires extending far outside the body, neon shock absorbers, high cab; roaring supercharged blower whine layered over deep V8 rumble.
* **Gameplay Rationale:** A hilarious arcade "juggernaut" archetype. Devastating on tracks with wide infields where it can ignore the road entirely and carve direct lines.

---

### 3.2 Kart & Micro-Racer Variants

#### 4. 250cc Superkart / Division 1 "Pocket Rocket"
* **Concept:** Long-circuit Superkart with full aerodynamic bodywork, enclosed front nosecone, sidepod wings, and a 100 BHP twin-cylinder 2-stroke engine.
* **Physics Levers:**
  * **Mass:** ~225 kg including driver.
  * **Aero:** High downforce package (`downforce_coefficient: 1.85`).
  * **Top Speed:** ~245 km/h (68 m/s).
  * **Handling:** Unmatched lateral grip in medium-to-high speed corners (pulling >3.5G). Snap-spins if traction breaks, punishing over-driving.
* **Visuals & Audio:** Sleek full-body carbon fairing, low windscreen, prominent bi-plane rear wing; screaming high-RPM twin 2-stroke buzz reminiscent of 90s GP motorcycles.
* **Gameplay Rationale:** Turns full-sized GP circuits (Monza, Silverstone) into ultra-fast reflex tests where lifting off throttle in sweepers is rare and mistakes are catastrophic.

#### 5. Racing Lawnmower / Backyard Yard-Kart
* **Concept:** Modified garden tractor stripped of cutting blades, lowered on kart slicks, powered by a modified 450cc 4-stroke single-cylinder engine.
* **Physics Levers:**
  * **Mass / Dimensions:** ~190 kg, very short wheelbase (1.1 m), narrow track width (0.85 m), high seating position.
  * **Handling:** Prone to lift-off oversteer and tail-hopping over curbs; centrifugal clutch delay on standing starts.
* **Visuals & Audio:** Boxy tractor hood, front grille, exposed vertical exhaust stack spitting flames; uneven 4-stroke "thumper" idle turning into a high-pitched tractor drone under load.
* **Gameplay Rationale:** Pure comedic arcade charm. Ideal for tight, technical kart tracks and backyard dirt circuits with close bumper-to-bumper bumper rubbing.

#### 6. Drift Trike / Hard-Slick Slide Kart
* **Concept:** Purpose-built drifting tricycle / kart with low-friction PVC rear tire sleeves and high front steering angle.
* **Physics Levers:**
  * **Tire Model:** Extremely low rear lateral friction (`drift_slide_friction: 0.32`, `peak_d: 0.40`).
  * **Steering:** Wide steering angle (`max_steer_angle: 0.90` / ~52° lock).
  * **Handling:** Effortless slide initiation at any speed; car transitions smoothly through 180° and 360° pendulum drift angles.
* **Visuals & Audio:** Exposed rear plastic-wrapped rims, low-slung bucket seat; constant high-pitched tire squeal and scraping plastic audio cues.
* **Gameplay Rationale:** Dedicated party vehicle designed for score-attack drift parks, continuous donuts, and backwards-entry cornering.

---

### 3.3 Sport, GT & Classic Variants

#### 7. Trackday Ultralight (Ariel Atom / Caterham 7 style)
* **Concept:** Minimalist open-cockpit track toy with visible exoskeleton frame, motorcycle-derived rev limit (10,000+ RPM), and no electronic aids.
* **Physics Levers:**
  * **Mass:** Featherweight ~520 kg.
  * **Aero:** Minimal aerodynamic downforce (`downforce_coefficient: 0.22`), relying entirely on mechanical tire grip.
  * **Braking:** Exceptional braking deceleration without ABS lockup; lightning-fast turn-in response.
* **Visuals & Audio:** Exposed perimeter frame, visible pushrod suspension rockers, cycle fenders over front wheels; high-pitched naturally aspirated 4-cylinder intake roar.
* **Gameplay Rationale:** The purist driver's car. Demands precise braking and throttle control, rewarding smooth line discipline and rewarding trail braking into technical corners.

#### 8. Group 5 "Super Silhouette" Turbo Racer
* **Concept:** Late 1970s / early 1980s DRM & IMSA GTX monster featuring extreme aerodynamic box flares, a massive "snowplow" front splitter, and explosive turbo boost lag.
* **Physics Levers:**
  * **Mass / Engine:** ~1080 kg, 720 BHP twin-turbo engine with deliberate throttle lag: sluggish below 40% speed, followed by a sudden exponential torque surge.
  * **Downforce:** High rear deck wing producing strong high-speed downforce (`downforce_coefficient: 2.2`).
  * **Handling:** Sudden power oversteer if throttle is mashed mid-corner; glorious straight-line speed once on boost.
* **Visuals & Audio:** Enormous box fender flares, side-exit exhaust rocker panel pipes; pronounced turbo spooling whistle, violent wastegate chatter, and huge backfire flame plumes on throttle lift.
* **Gameplay Rationale:** Thrilling risk-reward balance. The driver must anticipate corner exits and manage boost onset before spinning the rear wheels.

#### 9. Electric Hypercar (EV Quad-Motor)
* **Concept:** State-of-the-art electric hypercar (e.g. Rimac Nevera / Lotus Evija inspired) with four independent electric motors and simulated active torque vectoring.
* **Physics Levers:**
  * **Mass:** Heavy ~1950 kg battery chassis with very low center of gravity (`cg_height: 0.24`).
  * **Acceleration:** Unrivaled 0-100 km/h acceleration (<1.9s) with instant maximum torque at 0 RPM.
  * **Drivetrain:** 50/50 AWD (`drive_bias: 0.5`) with high traction control assist strength.
  * **Tradeoff:** Heavy curb weight requires earlier braking zones; higher tire wear and inertia when sliding.
* **Visuals & Audio:** Aerodynamic active flaps, full-width LED taillight bar, rear carbon venturi tunnels; synthetic multi-frequency electric motor inverter whine and regenerative braking hum.
* **Gameplay Rationale:** "Point-and-shoot" corner exit performance. Out-accelerates every gas-powered vehicle out of hairpins, creating unique overtaking dynamics against lighter cars.

#### 10. Classic American Muscle Cruiser (Late 60s Fastback V8)
* **Concept:** Iconic heavy American muscle car featuring a 7.0L 427 Big Block V8, live rear axle, soft leaf-spring suspension, and bias-ply tires.
* **Physics Levers:**
  * **Mass:** ~1520 kg with high longitudinal weight transfer (`weight_transfer_longitudinal: 1.45`).
  * **Suspension:** Significant nose-dive under hard braking and steep rear squat on launch.
  * **Handling:** Low cornering tire stiffness (`stiffness_b: 7.2`) causing progressive, lazy power slides.
* **Visuals & Audio:** Long sculpted hood with shaker scoop, chrome bumpers, dual rear exhaust tips; deep, loping, cammed V8 idle that turns into an earth-shaking roar under wide-open throttle.
* **Gameplay Rationale:** Great for flowing, wide-open road courses and high-speed oval layouts where straight-line grunt and controllable power slides shine.

---

### 3.4 Novelty & Party Archetypes

#### 11. Tuned Kei Micro-Van / Delivery Box
* **Concept:** Japanese 660cc micro-van modified with aggressive wheel negative camber, lowered sport coilovers, and a swapped turbo engine.
* **Physics Levers:**
  * **Mass / Dimensions:** ~820 kg, very tall body height (`cg_height: 0.52`), narrow track width (1.20 m).
  * **Weight Transfer:** Dramatic visual chassis roll in tight turns; lifting off mid-corner triggers sudden rotation.
  * **Slipstream:** High aerodynamic drag coefficient (`air_drag_coefficient: 0.55`) making slipstream drafting behind competitors crucial.
* **Visuals & Audio:** Upright boxy silhouette, sliding door seams, roof rack; chirping blow-off valve and buzzing 3-cylinder turbo engine note.
* **Gameplay Rationale:** The ultimate underdog machine. Hilarious to watch leaning precariously into turns while slipstreaming high-powered GT cars down long straights.

#### 12. European Racing Super Truck (FIA ETRC Style)
* **Concept:** 5-ton, 1200 BHP race-prepared cab-over semi-truck with liquid-cooled drum brakes and 5,000 Nm of diesel torque.
* **Physics Levers:**
  * **Mass / Inertia:** ~5200 kg with enormous collision momentum.
  * **Braking:** Heavy braking distances required; water-spray brake cooling system simulation.
  * **Collisions:** Shrugs off bumping from passenger cars without deviating from its line; immune to water puddle hydroplaning.
* **Visuals & Audio:** Imposing cab-over front grille, rear fifth-wheel spoiler, visible water spray from brake drums; deep turbo-diesel whistle, air-brake hiss on deceleration, and thick black exhaust bursts.
* **Gameplay Rationale:** Creates an asymmetric "David vs. Goliath" dynamic in mixed-class races, serving as a moving chicane or impenetrable defensive racer on narrow tracks.

---

## 4. Physics Parameter Reference Matrix

The following table provides baseline target values for implementing these archetypes into `CarConfig`:

| Vehicle | Mass (kg) | Drive Bias | Engine Force (N) | Top Speed | CG Height | Downforce Cl | Special Gameplay Dynamic |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| **Baja Sand Buggy** | 650 | 0.0 (RWD) | 5,800 | 175 km/h | 0.46 | 0.15 | Sand & dirt immunity; high jump compliance |
| **Stadium Super Truck** | 1,350 | 0.2 (RWD bias) | 9,800 | 225 km/h | 0.56 | 0.35 | 3-wheel cornering roll; plush ramp landings |
| **Monster Truck** | 2,200 | 0.5 (AWD) | 14,000 | 160 km/h | 0.78 | 0.05 | Grass/sand ignore; bulldozing collision mass |
| **Superkart 250cc** | 225 | 0.0 (RWD) | 3,200 | 245 km/h | 0.15 | 1.85 | 3.5G lateral grip; snap spin punishment |
| **Racing Lawnmower** | 190 | 0.0 (RWD) | 1,800 | 105 km/h | 0.32 | 0.00 | Lift-off oversteer; bumpy curb bouncing |
| **Drift Trike / Slick** | 160 | 1.0 (FWD pull) | 1,500 | 85 km/h | 0.20 | 0.00 | Zero-friction rear slides; 360° rotation |
| **Trackday Ultralight** | 520 | 0.0 (RWD) | 5,200 | 230 km/h | 0.22 | 0.22 | Trail-braking agility; mechanical grip |
| **Silhouette Turbo** | 1,080 | 0.0 (RWD) | 10,500 | 315 km/h | 0.28 | 2.20 | Turbo boost lag; power-oversteer rocket |
| **Electric Hypercar** | 1,950 | 0.5 (AWD) | 16,500 | 340 km/h | 0.24 | 1.40 | Instant 0-100 km/h torque; high inertia |
| **Muscle Cruiser** | 1,520 | 0.0 (RWD) | 9,200 | 215 km/h | 0.36 | 0.10 | Pitch/dive body motion; loose tail |
| **Tuned Kei Van** | 820 | 0.0 (RWD) | 4,200 | 165 km/h | 0.52 | 0.10 | Steep body roll; slipstream dependence |
| **Racing Super Truck** | 5,200 | 0.0 (RWD) | 26,000 | 160 km/h | 0.65 | 0.20 | Unstoppable momentum; long braking zones |

---

## 5. Proposed Visual Archetype Extensions

In [`crates/tdrace-app/src/render/car.rs`](file:///home/mario/workspace/games/tdrace/crates/tdrace-app/src/render/car.rs), the `VehicleVisualType` enum can be expanded with dedicated variants:

```rust
pub enum VehicleVisualType {
    OpenWheel { front_wing_span: f32, rear_wing_height: f32, halo: bool },
    GoKart { exposed_driver: bool, side_bumpers: bool },
    RallyHatch { roof_scoop: bool, mudflaps: bool, large_wing: bool },
    TouringGT { widebody: bool, gt_wing: bool, diffuser: bool },
    StockCar { tall_wing: bool, roof_fins: bool, window_net: bool },
    
    // Prospective extensions:
    DuneBuggy { exposed_engine: bool, roll_cage: bool, spare_tire: bool },
    MonsterTruck { tire_diameter_scale: f32, high_chassis_lift: f32 },
    Superkart { aerodynamic_pod: bool, biplane_wing: bool },
    KeiVan { tall_profile: bool, roof_rack: bool },
    SuperTruck { cab_over: bool, rear_exhaust_stacks: bool },
}
```

---

## 6. Implementation Roadmap

1. **Preset Definition Phase:**
   - Add new constructor presets (e.g. `CarConfig::baja_buggy()`, `CarConfig::superkart()`, `CarConfig::monster_truck()`) to [`crates/wheelbase/src/config.rs`](file:///home/mario/workspace/games/tdrace/crates/wheelbase/src/config.rs).
   - Add unit tests verifying mass, inertia, and steering limits.
2. **Visual & Rendering Phase:**
   - Add 2.5D chassis primitives for exposed roll cages, oversized wheels, and tall boxy bodies in [`render/car.rs`](file:///home/mario/workspace/games/tdrace/crates/tdrace-app/src/render/car.rs).
   - Tune roll/pitch scaling for high-CG vehicles (`Stadium Super Truck`, `Kei Van`).
3. **Audio & FX Phase:**
   - Wire sound profiles (screaming twin 2-stroke, diesel turbo whistle, electric inverter whine, cammed V8).
   - Add terrain-specific particle effects (sand rooster tails for buggies, water spray clouds for super trucks).
4. **Game Module / Tournament Integration:**
   - Create dedicated mixed-surface arena modules or integrate into existing modules (`classic`, `rally`, `kart`).
