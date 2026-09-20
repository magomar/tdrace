---
type: Asset Catalog
title: "World Circuit & Track Directory"
description: "Master directory of circuits in TdRace across Classic Heritage, NASCAR, Rallycross, Karting, Extreme Off-Road, and F1/GT modalities."
status: active
category: circuits
tags: [circuits, tracks, directory, catalog, venues]
---

# World Circuit & Track Directory 🏁🗺️

**TdRace** features an expansive catalog of meticulously mapped circuits and off-road venues, stored as vector geometry JSON files under [`tracks/`](../../tracks) and procedural presets in [`crates/arcade-race-core`](../../crates/arcade-race-core). The circuit engine renders road ribbons, kerbs, barrier perimeters, and split-route networks (such as World RX Joker Laps).

---

## 🌍 Track Categories & Modalities

| Modality | Document Reference | Circuit Count | Surface Makeup & Special Mechanics |
| :--- | :--- | :---: | :--- |
| **Classic Heritage** | [classic.md](classic.md) | 10 | Asphalt GP ribbons, figure-eight cross-overs, jump raceways, dirt ovals. Standalone arcade catalog. |
| **NASCAR & Ovals** | [nascar.md](nascar.md) | 12 | High-banked tri-ovals (Daytona, Talladega), short tracks (Bristol, Martinsville), dirt ovals (Eldora), and road courses. |
| **Rallycross & Mixed-Surface** | [rally.md](rally.md) | 15 | 60/40 Asphalt-Gravel splits, alternative Joker Lap routing (`TrackNetwork`), jumps, and authentic World RX circuits. |
| **Karting Arenas** | [kart.md](kart.md) | 15 | Tight-radius hairpin chicanes, high-grip sprint asphalt, curb hopping (Lonato, Genk, Sarno, Valencia, Campillos). |
| **Extreme Off-Road & Arenas** | [offroad.md](offroad.md) | 15 | Open arena sand dunes, mud bog pits, arctic ice tracks, rhythm stadium whoops, and jump ramps. |
| **Formula 1 & GT Endurance** | [f1_gt.md](f1_gt.md) | 18 | FIA Grade 1 homologated road courses (Spa, Monza, Silverstone, Suzuka, Circuit de la Sarthe / Le Mans, Nürburgring GP, MadRing, Portimão). |

---

## 📐 Circuit Geometry Architecture

Each circuit JSON defines:
1. **Centerline Spline**: Continuous cubic Hermite / Catmull-Rom spline coordinates with track width and banking angles.
2. **Surface Zones**: Explicit polygon definitions for `Asphalt`, `Dirt`, `Gravel`, `Curb`, `Grass`, `Sand`, `Water`, and `Snow`.
3. **Barrier Perimeter**: Line segments and bounding polylines assigned barrier types (`Concrete`, `Steel`, `TireWall`, `CurbWall`).
4. **Branching Networks**: Directed graph nodes for alternative layouts and mandatory Rallycross Joker laps.
5. **Elevation & Jump Ramps**: 2.5D takeoff coordinates and flight launch vectors.

Select any modality above to inspect venue lengths, turn counts, and surface breakdowns.
