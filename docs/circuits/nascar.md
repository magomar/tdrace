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

## 📋 Speedway Roster & Specifications

| Venue | File Key | Lap Distance | Turn Banking | Track Type | Racing Characteristics |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **Daytona International** | `daytona.json` | $4,023\text{ m}$ ($2.5\text{ mi}$) | $31^\circ$ | Superspeedway | Wide tri-oval, high-speed pack drafting, double yellow line boundary. |
| **Talladega Superspeedway** | `talladega.json` | $4,280\text{ m}$ ($2.66\text{ mi}$) | $33^\circ$ | Superspeedway | Steepest banking, 4-wide pack racing, high risk of multi-car pileups. |
| **Charlotte Motor Speedway** | `charlotte.json` | $2,414\text{ m}$ ($1.5\text{ mi}$) | $24^\circ$ | Quad-Oval | Intermediate 1.5-mile oval, aerodynamic wake wash in dirty air. |
| **Indianapolis Motor Speedway** | `indianapolis.json` | $4,023\text{ m}$ ($2.5\text{ mi}$) | $9^\circ$ | Rectangular Oval | Historic Brickyard, low banking requiring heavy braking into 4 distinct corners. |
| **Bristol Motor Speedway** | `bristol.json` | $858\text{ m}$ ($0.533\text{ mi}$) | $28^\circ$ | Concrete Short Track | High-banked concrete bowl, bump-and-run passes, steel SAFER barrier rims. |
| **Martinsville Speedway** | `martinsville.json` | $846\text{ m}$ ($0.526\text{ mi}$) | $12^\circ$ | Paperclip Short Track | Concrete corners / asphalt straights, extreme brake rotor heat, hairpin pivots. |
| **Darlington Raceway** | `darlington.json` | $2,198\text{ m}$ ($1.366\text{ mi}$) | $25^\circ / 23^\circ$ | Asymmetric Egg Oval | "Too Tough to Tame", right-side wall scrapes, narrow turns 3 & 4. |
| **Iowa Speedway** | `iowa.json` | $1,408\text{ m}$ ($0.875\text{ mi}$) | $12^\circ-14^\circ$ | Tri-Oval Short Track | Progressive banking multi-groove racing. |
| **Chicago Street Course** | `chicago.json` | $3,540\text{ m}$ ($2.2\text{ mi}$) | Flat ($0^\circ$) | Urban Street Circuit | Concrete barrier-lined streets, sharp $90^\circ$ intersections, manhole covers. |
| **Watkins Glen International** | `watkins_glen.json` | $3,942\text{ m}$ ($2.45\text{ mi}$) | $6^\circ-10^\circ$ | Natural Road Course | High-speed esses, outer loop carousel, heavy braking into inner loop bus stop. |
| **Road America** | `road_america.json` | $6,515\text{ m}$ ($4.048\text{ mi}$) | Flat | Road Course | High-speed straights, downhill braking into Turn 5, The Kink, Canada Corner. |
| **Eldora Speedway** | `eldora.json` | $805\text{ m}$ ($0.5\text{ mi}$) | $24^\circ$ | Clay Dirt Oval | Mud/dirt cushion, sliding right against the outside wall, clay roost plumes. |

---

## 🏎️ Oval Racing Physics
* **Pack Drafting & Slipstream**: Trailing vehicles inside the drafting cone gain a $+8\%$ aerodynamic drag reduction ($C_{\text{air\_drag}} \times 0.92$), creating slingshot passing opportunities.
* **Banked Corner Dynamics**: Centripetal force vectoring perpendicular to banking reduces lateral tire shear load, enabling cornering velocities over $310\text{ km/h}$.

For stock car specifications, view the [NASCAR & Trans-Am Roster](../vehicles/stock_car.md).
