---
type: Asset Catalog
title: "NASCAR & Stock Car Ovals Directory"
description: "Directory of 12 American speedways: high-banked superspeedways, short tracks, dirt ovals, and road courses."
status: active
category: circuits
tags: [circuits, nascar, ovals, daytona, talladega, bristol]
---

# NASCAR & Stock Car Ovals Directory 🏁🇺🇸

The **NASCAR & Stock Car** track catalog in [`tracks/nascar/`](../../tracks/nascar) features 12 authentic American speedway layouts. From restrictor-plate superspeedways where 3-wide pack drafting decides victory, to bruising short-track coliseums and high-banked clay dirt bowls.

---

## 📋 Speedway Roster & Specifications (17 Circuits)

### Tier 1: Short Tracks & Dirt Bowls (5 Starter Circuits)
| Venue | File Key | Lap Distance | Turn Banking | Track Type | Racing Characteristics |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Martinsville Speedway** | `martinsville.json` | $846\text{ m}$ ($0.526\text{ mi}$) | $12^\circ$ | Paperclip Short Track | Concrete corners / asphalt straights, extreme brake rotor heat, hairpin pivots. |
| **Bristol Motor Speedway** | `bristol.json` | $858\text{ m}$ ($0.533\text{ mi}$) | $28^\circ$ | Concrete Short Track | High-banked concrete bowl, bump-and-run passes, steel SAFER barrier rims. |
| **Eldora Speedway** | `eldora.json` | $805\text{ m}$ ($0.5\text{ mi}$) | $24^\circ$ | Clay Dirt Oval | Mud/dirt cushion, sliding right against the outside wall, clay roost plumes. |
| **Bowman Gray Stadium** | `bowman_gray.json` | $402\text{ m}$ ($0.25\text{ mi}$) | Flat ($0^\circ$) | Quarter-Mile Stadium Flat Track | The Madhouse: claustrophobic flat bullring surrounded by football grandstands. |
| **Lucas Oil Indianapolis Raceway Park** | `irp_oval.json` | $1,104\text{ m}$ ($0.686\text{ mi}$) | $12^\circ$ | Short Oval Speedway | Classic Midwestern short oval with continuous sweeping banking and tight apron lines. |

### Tier 2: Intermediate Speedways (3 Circuits)
| Venue | File Key | Lap Distance | Turn Banking | Track Type | Racing Characteristics |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Charlotte Motor Speedway** | `charlotte.json` | $2,414\text{ m}$ ($1.5\text{ mi}$) | $24^\circ$ | Quad-Oval | Intermediate 1.5-mile oval, aerodynamic wake wash in dirty air. |
| **Darlington Raceway** | `darlington.json` | $2,198\text{ m}$ ($1.366\text{ mi}$) | $25^\circ / 23^\circ$ | Asymmetric Egg Oval | "Too Tough to Tame", right-side wall scrapes, narrow turns 3 & 4. |
| **North Wilkesboro Speedway** | `north_wilkesboro.json` | $1,006\text{ m}$ ($0.625\text{ mi}$) | $14^\circ$ | Uphill/Downhill Short Track | Historic moonshine-era bullring with distinctive downhill frontstretch and uphill backstretch. |

### Tier 3: National Road Courses & Tri-Ovals (3 Circuits)
| Venue | File Key | Lap Distance | Turn Banking | Track Type | Racing Characteristics |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Iowa Speedway** | `iowa.json` | $1,408\text{ m}$ ($0.875\text{ mi}$) | $12^\circ-14^\circ$ | Tri-Oval Short Track | Progressive banking multi-groove racing. |
| **Watkins Glen International** | `watkins_glen.json` | $3,942\text{ m}$ ($2.45\text{ mi}$) | $6^\circ-10^\circ$ | Natural Road Course | High-speed esses, outer loop carousel, heavy braking into inner loop bus stop. |
| **Road America** | `road_america.json` | $6,515\text{ m}$ ($4.048\text{ mi}$) | Flat | Road Course | High-speed straights, downhill braking into Turn 5, The Kink, Canada Corner. |

### Tier 4: Premier Superspeedways & Urban Streets (3 Circuits)
| Venue | File Key | Lap Distance | Turn Banking | Track Type | Racing Characteristics |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Indianapolis Motor Speedway** | `indianapolis.json` | $4,023\text{ m}$ ($2.5\text{ mi}$) | $9^\circ$ | Rectangular Oval | Historic Brickyard, low banking requiring heavy braking into 4 distinct corners. |
| **Pocono Raceway** | `pocono.json` | $2,011.5\text{ m}$ ($1.25\text{ mi}$) | $14^\circ / 8^\circ / 6^\circ$ | Tri-Oval "Tricky Triangle" ($0.5\times$ scale) | 3 completely different corner radiuses and banking angles connected by massive straights. Scaled at $0.5\times$ for competitive gameplay pacing. |
| **Chicago Street Course** | `chicago.json` | $3,540\text{ m}$ ($2.2\text{ mi}$) | Flat ($0^\circ$) | Urban Street Circuit | Concrete barrier-lined streets, sharp $90^\circ$ intersections, manhole covers. |

### Tier 5: World Championship Superspeedways (3 Circuits)
| Venue | File Key | Lap Distance | Turn Banking | Track Type | Racing Characteristics |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Daytona International** | `daytona.json` | $4,023\text{ m}$ ($2.5\text{ mi}$) | $31^\circ$ | Superspeedway | Wide tri-oval, high-speed pack drafting, double yellow line boundary. |
| **Talladega Superspeedway** | `talladega.json` | $4,280\text{ m}$ ($2.66\text{ mi}$) | $33^\circ$ | Superspeedway | Steepest banking, 4-wide pack racing, high risk of multi-car pileups. |
| **Phoenix Raceway** | `phoenix.json` | $1,609\text{ m}$ ($1.0\text{ mi}$) | $8^\circ-11^\circ$ | Dogleg Desert Oval | Asymmetric tri-oval with radical banked dogleg cutting opportunity. |

---

## 🏎️ Oval Racing Physics
* **Pack Drafting & Slipstream**: Trailing vehicles inside the drafting cone gain a $+8\%$ aerodynamic drag reduction ($C_{\text{air\_drag}} \times 0.92$), creating slingshot passing opportunities.
* **Banked Corner Dynamics**: Centripetal force vectoring perpendicular to banking reduces lateral tire shear load, enabling cornering velocities over $310\text{ km/h}$.

For stock car specifications, view the [NASCAR & Trans-Am Roster](../vehicles/stock_car.md).
