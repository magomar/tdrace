# Specification: Rallycross & All-Terrain World Cup Career Mode

**Document Status:** PROPOSED  
**Author:** Antigravity Pairing Assistant & Motorsport Simulation Team  
**Date:** September 18, 2026  
**Primary Target Crates:**  
1. [`crates/wheelbase`](file:///home/mario/workspace/games/tdrace/crates/wheelbase) (Mixed-Surface Friction Transitions, AWD Center Differentials, Long-Travel Jumps, Lift-off Oversteer)  
2. [`crates/arcade-race-core`](file:///home/mario/workspace/games/tdrace/crates/arcade-race-core) (Joker Lap Split Spline Topology, Gravel/Mud/Asphalt Multi-Zone Surfaces, Jump Ramp Ballistics)  
3. [`crates/tdrace-app`](file:///home/mario/workspace/games/tdrace/crates/tdrace-app) (World RX Tournament Format, Heats/Semi-Finals/Finals, Career Progress & XP Engine)  

---

## 1. Executive Summary & Career Architecture

The **Rallycross & All-Terrain World Cup Career Mode** models the progression from grassroots front-wheel-drive rally hatchbacks to ferocious 600+ BHP mixed-surface monsters, desert raid beasts, and stadium jumping trucks in **TdRace**. 

Combining tarmac grip, loose gravel drifting, mud spray, and massive jump launch ramps, this career model develops:
* **Lift-Off Oversteer & Scandinavian Flicks:** Weight transfer dynamics across slippery and high-grip surfaces.
* **AWD Traction & Anti-Lag Boost:** Instant torque management out of slow hairpin berms.
* **Tactical Joker Lap Execution:** Split-second strategic route choices and gap timing.
* **High-Impact Suspension Management:** Landing stabilization and chassis roll over extreme jumps and dunes.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                   RALLYCROSS & ALL-TERRAIN CAREER PROGRESSION LADDER                   │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│   [Tier 1: Rally Junior FWD] (210 BHP, FWD Hot Hatch, 1,080 kg)                        │
│   • Trail-braking, lift-off weight transfer, learning gravel/asphalt transitions       │
│   • Venues: Höljes (Sweden), Lydden Hill (UK), Mettet (Belgium - NEW)                  │
│                                           │                                            │
│                                           ▼ (Earn 1,500 XP)                            │
│   [Tier 2: WRC / RX Turbo Supercar] (380 BHP, AWD Spaceframe/RX, 1,190 kg)             │
│   • Explosive anti-lag acceleration, 0-100 in 2.0s, aggressive joker lap tactics       │
│   • Venues: Hell / Lånkebanen (Norway), Lohéac (France), Silverstone RX (UK - NEW)     │
│                                           │                                            │
│                                           ▼ (Earn 3,500 XP)                            │
│   [Tier 3: Group B Beast] (550 BHP, Mid-Engine Turbo Monster, 960 kg)                  │
│   • Severe turbo lag, astronomical power-to-weight, high slip-angle pendulum drifts    │
│   • Venues: Estering (Germany), Montalegre (Portugal), Riga / Biķernieki (Latvia - NEW)│
│                                           │                                            │
│                                           ▼ (Earn 6,500 XP)                            │
│   [Tier 4: All-Terrain Rally Raid T1+] (450 BHP, Heavy Spaceframe Dakar, 2,010 kg)     │
│   • Long-travel 350mm dampers, deep mud/sand absorption, dune climbing endurance       │
│   • Venues: Nyirád "Red Cauldron" (Hungary), Tykkimäki (Finland), Killarney RX (NEW)   │
│                                           │                                            │
│                                           ▼ (Earn 10,000 XP)                           │
│   [Tier 5: Stadium Super Truck / SST] (650 BHP, Spaceframe V8, 1,380 kg RWD)           │
│   • High CG roll, 3-wheel cornering, 40-foot aerial jump ramps, stadium spectacle      │
│   • Venues: Barcelona-Catalunya RX, Oasis Desert Rally, Yas Marina RX Arena (NEW)      │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. 5-Tier Vehicle Hierarchy & Prototypical Engineering Specs

```
                     RALLYCROSS & ALL-TERRAIN SILHOUETTES (LATERAL 2D)
                     
    Tier 1 Rally Junior FWD (Peugeot 208 / Fiesta / Clio):
         ___/‾‾‾‾\___
      ==[ FWD Hatch  \_]===
        (O)           (O)

    Tier 4 Rally Raid T1+ (Hilux T1+ / RS Q e-tron / Hunter):
           ______/‾‾‾‾\_______   [Massive 37" Tires]
       ===[ Raised Skid-plate ]===
         ( O )             ( O )

    Tier 5 Stadium Super Truck (SST V8 / Robby Gordon Spec):
           _____/‾‾‾‾|______
       ===[ High CG Roll Bar ]=== [Long-Travel Dual Shocks]
         ( O )             ( O )
```

### 2.1 Tier 1: Rally Junior FWD (Grassroots Hot Hatch)
* **Design Philosophy:** Production-based lightweight front-wheel-drive hot hatches. With modest horsepower and no rear drive, drivers must master braking into corners to provoke rear-end rotation (lift-off oversteer) before pinning the throttle.
* **Core Physics:**
  * Power: $210\,\text{BHP}$ ($157\,\text{kW}$) $1.2\,\text{L}$ Turbo 3-cylinder / $1.6\,\text{L}$ 16V.
  * Curb Weight ($m$): $1,080.0\,\text{kg}$ | Yaw Moment of Inertia ($I_z$): $1,250.0\,\text{kg}\cdot\text{m}^2$.
  * Drive Layout: Front-Wheel Drive (`drive_bias: 1.0`), mechanical plated limited-slip differential.
  * Top Speed ($v_{\text{max}}$): $52.0\,\text{m/s}$ ($\approx 187\,\text{km/h}$).
  * Steering Rack Speed: $8.2\,\text{rad/s}$ (Rapid counter-steer response).
  * Tires: Mixed-surface rally gravel/tarmac tires ($B = 7.2$, $C = 1.35$, $D = 1.02$, $E = -0.16$).
* **Prototypical Vehicles:**
  1. **Peugeot 208 Rally4:** Crisp front-end turn-in bite, high agility through tight gravel hairpins.
  2. **Ford Fiesta Rally4:** Punchy turbo low-end boost, progressive rear-end breakaway on asphalt.
  3. **Renault Clio Rally4:** Ultra-stable chassis over crests, forgiving curb compliance.

### 2.2 Tier 2: WRC / RX Turbo Supercar (Modern Rallycross Benchmark)
* **Design Philosophy:** Custom tubular/monocoque all-wheel-drive supercars. Equipped with aggressive anti-lag systems (ALS), locked differentials, and sequential 6-speed gearboxes, achieving $0$–$100\,\text{km/h}$ in under $2.0\,\text{seconds}$.
* **Core Physics:**
  * Power: $380\,\text{BHP}$ ($283\,\text{kW}$) $2.0\,\text{L}$ Turbocharged I4 with ALS ($650\,\text{Nm}$ torque).
  * Curb Weight ($m$): $1,190.0\,\text{kg}$ | Yaw Moment of Inertia ($I_z$): $1,450.0\,\text{kg}\cdot\text{m}^2$.
  * Drive Layout: All-Wheel Drive (`drive_bias: 0.50` 50/50 split), spool center lock.
  * Top Speed ($v_{\text{max}}$): $64.0\,\text{m/s}$ ($\approx 230\,\text{km/h}$).
  * Aerodynamics: High-downforce rally bi-plane rear wing ($C_L \cdot A = 0.85$).
  * Handbrake Dynamics: Hydraulic handbrake decouples center diff to instantly swing the rear wheels $180^\circ$.
* **Prototypical Vehicles:**
  1. **Hyundai i20 RX:** Short wheelbase, razor-sharp rotation into ninety-degree dirt switches.
  2. **Volkswagen Polo RX:** Exceptional launch traction off the grid; planted stability on abrasive tarmac.
  3. **Audi S1 EKS RX:** Aggressive Quattro torque delivery, high curb-skipping tolerance.

### 2.3 Tier 3: Group B Beast (1980s Homologation Monsters)
* **Design Philosophy:** Mid-engine, lightweight spaceframe homologation specials from the golden era. Characterized by explosive boost thresholds, high polar moment of inertia, large turbo lag, and massive rooster tails.
* **Core Physics:**
  * Power: $550\,\text{BHP}$ ($410\,\text{kW}$) twin-charged/turbocharged monster ($8,500\,\text{RPM}$).
  * Curb Weight ($m$): $960.0\,\text{kg}$ | Yaw Moment of Inertia ($I_z$): $1,180.0\,\text{kg}\cdot\text{m}^2$.
  * Power-to-Weight Ratio: $572\,\text{BHP/tonne}$ ($0$–$100\,\text{km/h}$ in $2.3\,\text{s}$ on dirt).
  * Turbo Lag Simulation: Boost builds progressively above $4,200\,\text{RPM}$; below this RPM, engine output drops by $40\%$.
  * Assists: 100% Raw (`DriverAssistsConfig::raw()`).
* **Prototypical Vehicles:**
  1. **Audi Sport Quattro S1 E2:** Iconic 5-cylinder warble, massive front snowplow splitter and tall rear wing.
  2. **Peugeot 205 T16 EVO 2:** Mid-engine balance, explosive mid-range punch, agile Scandinavian flicks.
  3. **Lancia Delta S4:** Supercharged and turbocharged twin-boost layout, ferocious low-and-high RPM response.

### 2.4 Tier 4: All-Terrain Rally Raid T1+ (Cross-Country Dakar Spec)
* **Design Philosophy:** Purpose-built Dakar and Baja endurance prototypes designed to conquer broken terrain, washboard whoops, mud, and sand dunes. Features heavy reinforced spaceframes, 37-inch tires, and $350\,\text{mm}$ of wheel travel.
* **Core Physics:**
  * Power: $450\,\text{BHP}$ ($335\,\text{kW}$) Twin-Turbo V6 or electric-drivetrain generator.
  * Curb Weight ($m$): $2,010.0\,\text{kg}$ | Yaw Moment of Inertia ($I_z$): $3,100.0\,\text{kg}\cdot\text{m}^2$.
  * Ground Clearance: $0.35\,\text{m}$ | Suspension Travel: $350\,\text{mm}$ bypass dampers.
  * Terrain Immunity: Mud and loose sand viscous drag reduced by $65\%$; zero bounce penalty on large jump landings.
  * Top Speed ($v_{\text{max}}$): $47.2\,\text{m/s}$ ($\approx 170\,\text{km/h}$ governed for safety).
* **Prototypical Vehicles:**
  1. **Toyota GR DKR Hilux T1+:** Bulletproof reliability, massive suspension stroke, unstoppably flat over rock fields.
  2. **Audi RS Q e-tron:** Instant electric torque delivery across all 4 wheels, silent high-speed dune surfing.
  3. **Prodrive Hunter T1+:** Aggressive Ian Callum styling, wide track width, and high-speed stability through sand ruts.

### 2.5 Tier 5: Stadium Super Truck / SST (High-Flying V8 Brawler)
* **Design Philosophy:** 650 BHP V8 tube-chassis trucks competing on courses with oversized metal jumps. High center of gravity causes dramatic body roll, frequent 3-wheel cornering, and spectacular 40-foot aerial launches.
* **Core Physics:**
  * Power: $650\,\text{BHP}$ ($485\,\text{kW}$) naturally aspirated Chevrolet LS V8.
  * Curb Weight ($m$): $1,380.0\,\text{kg}$ | Yaw Moment of Inertia ($I_z$): $1,850.0\,\text{kg}\cdot\text{m}^2$.
  * Center of Gravity Height ($h_{\text{cg}}$): $0.78\,\text{m}$ (Substantial body roll up to $14^\circ$).
  * Drive Layout: Pure Rear-Wheel Drive (`drive_bias: 0.0`) with locked spool rear differential.
  * 3-Wheel Cornering: Inside front tire lifts off the ground under maximum lateral cornering load ($a_y > 1.2\,\text{G}$).
  * Ramp Jump Tolerance: Specialized hydraulic bump stops absorb vertical impacts of up to $8.0\,\text{m/s}$ without chassis damage.
* **Prototypical Vehicles:**
  1. **SST V8 Truck:** Spec tube-frame chassis, fiberglass pickup shell, roaring side-exhaust.
  2. **Robby Gordon Edition SST Spec:** Stiffer front sway-bar tuning, aggressive curb-hopping recovery.

---

## 3. 15-Venue Championship Calendar (3 per Tier)

```
                    RALLYCROSS & ALL-TERRAIN 15-VENUE CALENDAR
                    
[Tier 1: Traditional Dirt & RX Heritage]
 ├─ Höljes Motorstadion (Sweden - The Temple of Rallycross)
 ├─ Lydden Hill (Great Britain - Birthplace of RX)
 └─ [NUEVO] Mettet - Circuit Jules Tacheny (Belgium - Technical Mixed)
 
[Tier 2: Mixed Ovals & Rapid RX Circuits]
 ├─ Lånkebanen / Hell RX (Norway - Dramatic Elevation Drops)
 ├─ Circuit de Lohéac (France - Loose Gravel Sweepers)
 └─ [NUEVO] Silverstone RX (Great Britain - F1 Infield Section)
 
[Tier 3: Historic European Proving Grounds]
 ├─ Estering Buxtehude (Germany - Iconic Turn 1 Hairpin)
 ├─ Pista de Montalegre (Portugal - Alpine RX Changing Weather)
 └─ [NUEVO] Biķernieki Complex / Riga RX (Latvia - High-Grip Tarmac & Dirt)
 
[Tier 4: Wild Broken Terrain & Red Earth]
 ├─ Nyirád Racing Center (Hungary - "The Red Cauldron")
 ├─ Tykkimäen Moottorirata (Finland - Nordic High-Speed Flow)
 └─ [NUEVO] Killarney International Raceway RX (South Africa - Coastal Dirt)
 
[Tier 5: Monumental Stadiums & All-Terrain Extremes]
 ├─ Barcelona-Catalunya RX (Spain - Olympic Stadium RX Arena)
 ├─ Oasis Desert Rally (North Africa - Vast Sand Dunes & Raid Stage)
 └─ [NUEVO] Yas Marina RX Arena (Abu Dhabi - Floodlit Stunt Arena)
```

### 3.1 Tier 1 Venues: Traditional Dirt & RX Heritage
1. **Höljes Motorstadion (Sweden):** $1,210\,\text{m}$ (60% Tarmac, 40% Dirt). Legendary "Höljes Crest" jump where cars launch over $30\,\text{m}$ into Turn 2.
2. **Lydden Hill (Great Britain):** $1,170\,\text{m}$ (55% Tarmac, 45% Chalk/Dirt). The cradle of rallycross featuring the high-speed Chessons Drift and North Bend.
3. **Mettet - Circuit Jules Tacheny *(NUEVO)***:
   * **Location & Country:** Mettet, Wallonia, Belgium ($1,031\,\text{m}$, 61% Tarmac, 39% Dirt).
   * **Layout Highlights:** Ultra-technical banked dirt hairpin, tight tarmac chicanes, and a wide Joker lap loop.
   * **Pedagogy:** Perfect training ground for managing front-wheel-drive understeer transitions onto loose dirt.

### 3.2 Tier 2 Venues: Mixed Ovals & Rapid RX
1. **Hell RX / Lånkebanen (Norway):** $1,019\,\text{m}$ (63% Tarmac, 37% Gravel). Radical $24\,\text{m}$ downhill plunge into Turn 1 and high-speed joker merge.
2. **Circuit de Lohéac (France):** $1,150\,\text{m}$ (33% Tarmac, 67% Loose Gravel). Massive crowd favorite with the highest percentage of dirt in World RX.
3. **Silverstone RX *(NUEVO)***:
   * **Location & Country:** Northamptonshire, Great Britain ($972\,\text{m}$, 60% Tarmac, 40% Gravel).
   * **Layout Highlights:** Situated within the historic Wing section; features a massive stadium jump table and high-camber dirt bowl turn.
   * **Racing Dynamic:** Exploits the Supercar's 0-100 acceleration down the National straight before diving into loose dirt whoops.

### 3.3 Tier 3 Venues: Historic European Proving Grounds
1. **Estering Buxtehude (Germany):** $952\,\text{m}$ (60% Tarmac, 40% Dirt). Infamous 180-degree Turn 1 hairpin where Scandinavian flicks are mandatory.
2. **Pista de Montalegre (Portugal):** $1,050\,\text{m}$ (60% Tarmac, 40% Dirt). High altitude ($1,000\,\text{m}$ above sea level) reduces naturally aspirated engine power, making turbo boost crucial.
3. **Biķernieki Complex / Riga RX *(NUEVO)***:
   * **Location & Country:** Riga, Latvia ($1,295\,\text{m}$, 60% Tarmac, 40% Dirt).
   * **Layout Highlights:** Extremely abrasive, high-grip tarmac banked corners juxtaposed with three technical dirt sections and parallel jump crests.
   * **Challenge:** Demands extreme throttle modulation to keep the 550 BHP Group B monster from snap-spinning into concrete barrier walls.

### 3.4 Tier 4 Venues: Broken Terrain & Red Earth
1. **Nyirád Racing Center (Hungary):** $1,220\,\text{m}$ (48% Tarmac, 52% Red Clay). "The Red Cauldron" features deep ruts and red bauxite clay that punishes standard touring suspensions.
2. **Tykkimäen Moottorirata (Finland):** $1,350\,\text{m}$ (50% Tarmac, 50% Sand/Gravel). Fast Nordic layout with rolling elevation crests and high-speed drift sweepers.
3. **Killarney International Raceway RX *(NUEVO)***:
   * **Location & Country:** Cape Town, South Africa ($1,067\,\text{m}$, 60% Tarmac, 40% Gravel).
   * **Layout Highlights:** Fast coastal venue nestled beneath Table Mountain; features a wide tarmac straight into a blind off-camber gravel switchback.
   * **Vehicle Synergy:** Tests the Raid T1+ prototype's long-travel suspension over sudden surface ruts.

### 3.5 Tier 5 Venues: Monumental Stadiums & All-Terrain Extremes
1. **Barcelona-Catalunya RX (Spain):** $1,125\,\text{m}$ (67% Tarmac, 33% Gravel). Stadium stadium environment in the stadium section of the F1 venue.
2. **Oasis Desert Rally (North Africa):** $2,450\,\text{m}$ open cross-country stage (`SurfaceType::Sand` and `Dirt`). Dunes, dried wadi riverbeds, and zero pavement.
3. **Yas Marina RX Arena *(NUEVO)***:
   * **Location & Country:** Abu Dhabi, UAE ($1,100\,\text{m}$, 55% Tarmac, 45% Sand/Gravel).
   * **Layout Highlights:** Night-lit stadium course under high-power floodlights; features dual elevated metal launch kickers engineered specifically for Stadium Super Trucks.
   * **Spectacle:** Trucks jump 30 feet in the air across the Marina straight before landing into high-camber sand berms.

---

## 4. World RX Tournament Rules & Career Progression

### 4.1 Career Progression & Unlock Schedule

| Career Level | Category | Tier Name | Required XP | Car Unlocks | Circuit Unlocks |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Level 1** | Rally Junior FWD | **Grassroots RX Rookie** | $0\,\text{XP}$ | `peugeot_208_r4`, `fiesta_r4`, `clio_r4` | `holjes_rx`, `lydden_hill`, `mettet_rx` |
| **Level 2** | RX Supercar | **World RX Contender** | $1,500\,\text{XP}$ | `hyundai_i20_rx`, `polo_rx`, `audi_s1_rx` | `hell_rx`, `loheac_rx`, `silverstone_rx` |
| **Level 3** | Group B Beast | **Group B Legend** | $3,500\,\text{XP}$ | `audi_quattro_s1`, `peugeot_205_t16`, `lancia_s4` | `estering_rx`, `montalegre_rx`, `riga_rx` |
| **Level 4** | Rally Raid T1+ | **Dakar Desert Master** | $6,500\,\text{XP}$ | `hilux_t1_plus`, `audi_rs_q_etron`, `hunter_t1_plus` | `nyirad_rx`, `tykkimaki_rx`, `killarney_rx` |
| **Level 5** | Stadium Super Truck | **SST High-Flyer Champion** | $10,000\,\text{XP}$ | `sst_v8_truck`, `robby_gordon_sst` | `catalunya_rx`, `oasis_desert`, `yas_marina_rx` |

### 4.2 World RX Tournament Structure & Joker Lap Rules
Each Tier Cup consists of a realistic World RX weekend progression:
1. **Qualifying Heats (Q1–Q4):** 4-lap sprint races (4 cars per grid). Position times are converted into intermediate ranking points.
2. **Semi-Finals (Top 12):** Two 6-car races of 6 laps each. Top 3 from each semi-final advance to the Final.
3. **The Final (6 Cars):** 6-lap showdown for the podium trophy and championship points.
4. **Mandatory Joker Lap Rule:**
   * Every driver **must take the Joker Lap exactly once** during the race.
   * Taking the Joker Lap twice, or failing to take it before the checkered flag, incurs an automatic **30-second time penalty**.
   * The HUD renders a dynamic Joker status indicator: `JOKER: REQUIRED` (Red) $\rightarrow$ `JOKER: COMPLETED` (Green).

---

## 5. Acceptance Criteria (Pseudo-Gherkin)

```gherkin
Feature: Rallycross & All-Terrain World Cup Career Mode

  Scenario: Level 1 Driver starts Rally Junior FWD Career
    Given the player selects the "rally" module with a fresh profile
    When the player launches Career Mode
    Then Tier 1 "Grassroots RX Rookie" is available with 0 XP
    And cars "peugeot_208_r4", "fiesta_r4", and "clio_r4" are selectable
    And the circuit calendar includes "holjes_rx", "lydden_hill", and "mettet_rx"

  Scenario: Mandatory Joker Lap is validated at race finish
    Given the player is competing in the 6-lap Final at "hell_rx"
    When the player crosses the finish line having taken 0 Joker laps
    Then a 30-second penalty is appended to their total race time
    And the finishing position drops accordingly

  Scenario: Completing Tier 4 unlocks Tier 5 Stadium Super Trucks
    Given the player accumulates 10,000 XP in the Rallycross module
    When the career progress synchronizes
    Then Tier 5 is unlocked
    And "catalunya_rx", "oasis_desert", and "yas_marina_rx" are unlocked in the track registry
    And "sst_v8_truck" and "robby_gordon_sst" become available
```
