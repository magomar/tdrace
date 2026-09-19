#!/usr/bin/env python3
import json
import math
import re
import tomllib
from pathlib import Path

def generate_assets():
    root = Path(__file__).resolve().parent.parent
    portals_data = root / "portals" / "shared" / "data"
    portals_data.mkdir(parents=True, exist_ok=True)

    print("🚗 Ingesting game assets into portals/shared/data...")

    # 1. Ingest Circuits from tracks/
    tracks_dir = root / "tracks"
    circuits = []
    
    for json_file in sorted(tracks_dir.rglob("*.json")):
        if json_file.name.startswith("."):
            continue
        try:
            data = json.loads(json_file.read_text(encoding="utf-8"))
            name = data.get("name", json_file.stem.replace("_", " ").title())
            desc = data.get("description", "Competition racing venue.")
            cat = json_file.parent.name
            
            # Compute track length from waypoints
            waypoints = data.get("spline", {}).get("waypoints", [])
            length_m = 0.0
            if len(waypoints) > 1:
                for i in range(len(waypoints)):
                    p1 = waypoints[i].get("point", [0, 0])
                    p2 = waypoints[(i + 1) % len(waypoints)].get("point", [0, 0])
                    length_m += math.dist(p1, p2)
            
            # Identify surfaces and special features
            has_jumps = any(wp.get("elevation", 0) > 0 for wp in waypoints) or "ramp" in json_file.stem or "jump" in json_file.stem
            surfaces_present = set()
            for wp in waypoints:
                s = wp.get("surface")
                if s:
                    surfaces_present.add(s)
            if not surfaces_present:
                surfaces_present.add("Asphalt" if cat in ["f1", "gt", "nascar", "kart", "classic"] else "Dirt")

            circuits.append({
                "id": json_file.stem,
                "name": name,
                "category": cat,
                "modality": cat.upper(),
                "description": desc,
                "length_meters": round(length_m, 1) if length_m > 0 else 2500.0,
                "turns_count": len([wp for wp in waypoints if wp.get("left_curb") or wp.get("right_curb")]),
                "surfaces": sorted(list(surfaces_present)),
                "has_jumps": has_jumps,
                "rel_path": str(json_file.relative_to(root))
            })
        except Exception as e:
            print(f"  ⚠️ Failed parsing {json_file.name}: {e}")

    print(f"  ✅ Parsed {len(circuits)} circuits.")
    (portals_data / "circuits.json").write_text(json.dumps(circuits, indent=2), encoding="utf-8")

    # 2. Ingest Vehicles from crates/tdrace-app/src/catalog/mod.rs (80 Authentic Real-World Motorsport Cars)
    catalog_path = root / "crates" / "tdrace-app" / "src" / "catalog" / "mod.rs"
    catalog_content = catalog_path.read_text(encoding="utf-8")
    start_pos = catalog_content.find("pub static ALL_REAL_CARS: &[RealCarModel] = &[")
    end_pos = catalog_content.rfind("];")
    slice_content = catalog_content[start_pos:end_pos]

    blocks = list(re.finditer(r"RealCarModel\s*\{", slice_content))
    vehicles = []

    MODULE_NAMES = {
        "gt": "Gran Turismo & Endurance",
        "nascar": "NASCAR Stock Car Racing",
        "rally": "Rallycross & All-Terrain",
        "extreme_offroad": "Extreme Off-Road & Arenas",
        "kart": "Karting & Micro-Racers",
    }

    BADGES = {
        "gt": {1: "GT4", 2: "GT3", 3: "GT2", 4: "GT1", 5: "LMH"},
        "nascar": {1: "STREET", 2: "LATE", 3: "ARCA", 4: "TRUCK", 5: "TA1"},
        "rally": {1: "RALLY4", 2: "WRX", 3: "GRPB", 4: "T1+", 5: "SST"},
        "extreme_offroad": {1: "RAIL", 2: "TROPHY", 3: "ICE", 4: "MUD", 5: "MONSTER"},
        "kart": {1: "CADET", 2: "OK-J", 3: "KZ2", 4: "MOWER", 5: "SUPER"},
    }

    for i, b in enumerate(blocks):
        next_pos = blocks[i + 1].start() if i + 1 < len(blocks) else len(slice_content)
        sub = slice_content[b.start():next_pos]

        car_id = re.search(r'id:\s*"([^"]+)"', sub).group(1)
        name = re.search(r'name:\s*"([^"]+)"', sub).group(1)
        mfr = re.search(r'manufacturer:\s*"([^"]+)"', sub).group(1)
        year = int(re.search(r'year:\s*(\d+)', sub).group(1))
        mod_id = re.search(r'module_id:\s*"([^"]+)"', sub).group(1)
        cat_name = re.search(r'category_name:\s*"([^"]+)"', sub).group(1)
        tier = int(re.search(r'tier:\s*(\d+)', sub).group(1))
        bhp = int(re.search(r'bhp:\s*(\d+)', sub).group(1))
        torque = int(re.search(r'torque_nm:\s*(\d+)', sub).group(1))
        weight = int(re.search(r'weight_kg:\s*(\d+)', sub).group(1))
        top_speed = int(re.search(r'top_speed_kmh:\s*(\d+)', sub).group(1))
        accel = float(re.search(r'accel_0_100:\s*([\d\.]+)', sub).group(1))
        drivetrain = re.search(r'drivetrain:\s*"([^"]+)"', sub).group(1)
        engine = re.search(r'engine_desc:\s*"([^"]+)"', sub).group(1)
        aero = re.search(r'aero_downforce:\s*"([^"]+)"', sub).group(1)
        brakes = re.search(r'brakes_desc:\s*"([^"]+)"', sub).group(1)
        bio = re.search(r'history_bio:\s*"([^"]+)"', sub).group(1)

        cl_m = re.search(r'Cl\s*([\d\.]+)', aero)
        cd_m = re.search(r'Cd\s*([\d\.]+)', aero)
        downforce = float(cl_m.group(1)) if cl_m else 0.5
        drag = float(cd_m.group(1)) if cd_m else 0.45

        stats_m = re.search(r'stats:\s*\(([^)]+)\)', sub)
        raw_s = [float(x.strip()) for x in stats_m.group(1).split(',')]
        stats = {
            "speed": int(round(raw_s[0] * 100)),
            "acceleration": int(round(raw_s[1] * 100)),
            "grip": int(round(raw_s[2] * 100)),
            "agility": int(round(raw_s[3] * 100)),
            "downforce": int(round(raw_s[5] * 100)),
        }

        force_per_bhp = 38.0 if mod_id == 'kart' else (14.0 if mod_id == 'extreme_offroad' and tier >= 4 else 17.5)
        engine_force = int(bhp * force_per_bhp)
        drive_bias = 1.0 if drivetrain == 'FWD' else (0.5 if drivetrain in ['AWD', '4WD'] else 0.0)

        brake_rating = raw_s[4]
        brakes_kn = round(5.0 + brake_rating * 25.0, 1)

        vehicles.append({
            "id": car_id,
            "name": name,
            "manufacturer": mfr,
            "year": year,
            "module": MODULE_NAMES.get(mod_id, mod_id),
            "tier": tier,
            "category": cat_name,
            "class_badge": BADGES.get(mod_id, {}).get(tier, f"T{tier}"),
            "mass": weight,
            "power_bhp": bhp,
            "torque_nm": torque,
            "engine_force": engine_force,
            "top_speed_kmh": top_speed,
            "accel_0_100": accel,
            "drive_bias": drive_bias,
            "drivetrain": drivetrain,
            "engine_desc": engine,
            "downforce": downforce,
            "drag": drag,
            "brakes_desc": brakes,
            "brakes_kn": brakes_kn,
            "stats": stats,
            "summary": bio,
        })

    print(f"  ✅ Compiled {len(vehicles)} vehicles across 5 motorsport modules.")
    (portals_data / "vehicles.json").write_text(json.dumps(vehicles, indent=2), encoding="utf-8")

    # 3. Ingest Surfaces Matrix
    surfaces = [
        {"name": "Asphalt", "friction": 1.00, "rolling_resistance": 1.0, "surface_drag": 1.00, "smoke": True, "roost": False, "splash": False, "layer": "BelowTrack", "description": "Standard dry tarmac; optimal grip baseline, full tire smoke on heavy slip."},
        {"name": "Curb", "friction": 0.88, "rolling_resistance": 1.3, "surface_drag": 1.05, "smoke": True, "roost": False, "splash": False, "layer": "BelowTrack", "description": "Apex kerb / rumble strip; subtle haptic vibration, high grip with mild drag."},
        {"name": "Dirt", "friction": 0.78, "rolling_resistance": 1.2, "surface_drag": 1.10, "smoke": False, "roost": True, "splash": False, "layer": "BelowTrack", "description": "Compacted clay / gravel rally track; predictable sliding and drift control."},
        {"name": "Gravel", "friction": 0.70, "rolling_resistance": 2.5, "surface_drag": 1.25, "smoke": False, "roost": True, "splash": False, "layer": "BelowTrack", "description": "Loose stone rally stage / runoff; moderate grip with heavy stone roost."},
        {"name": "Mud", "friction": 0.52, "rolling_resistance": 6.5, "surface_drag": 3.20, "smoke": False, "roost": True, "splash": False, "layer": "AboveTrack", "description": "Viscous mud bog; heavy deceleration drag, low lateral bite, brown roost plumes."},
        {"name": "Grass", "friction": 0.45, "rolling_resistance": 18.0, "surface_drag": 2.20, "smoke": False, "roost": True, "splash": False, "layer": "BelowTrack", "description": "Standard off-track runoff; heavy rolling resistance penalizing corner cuts."},
        {"name": "Snow", "friction": 0.34, "rolling_resistance": 3.0, "surface_drag": 1.60, "smoke": False, "roost": True, "splash": False, "layer": "AboveTrack", "description": "Packed/powder snow; slippery winter rallying, white roost plumes."},
        {"name": "Sand", "friction": 0.30, "rolling_resistance": 30.0, "surface_drag": 4.50, "smoke": False, "roost": True, "splash": False, "layer": "BelowTrack", "description": "Deep gravel / sand trap; severe vehicle deceleration trap, sand rooster tails."},
        {"name": "Water", "friction": 0.22, "rolling_resistance": 3.5, "surface_drag": 2.00, "smoke": False, "roost": False, "splash": True, "layer": "AboveTrack", "description": "Standing puddle hazard; hydroplaning risk, aqua spray plumes."},
        {"name": "Oil", "friction": 0.12, "rolling_resistance": 0.8, "surface_drag": 0.95, "smoke": False, "roost": False, "splash": False, "layer": "AboveTrack", "description": "Oil slick hazard; extreme spin hazard, breaks rear traction instantly."},
        {"name": "Ice", "friction": 0.08, "rolling_resistance": 0.4, "surface_drag": 0.90, "smoke": False, "roost": False, "splash": False, "layer": "AboveTrack", "description": "Frozen lake; near-zero traction, near-frictionless gliding with no braking."}
    ]
    (portals_data / "surfaces.json").write_text(json.dumps(surfaces, indent=2), encoding="utf-8")
    print(f"  ✅ Compiled {len(surfaces)} surfaces physics matrix.")

if __name__ == "__main__":
    generate_assets()
