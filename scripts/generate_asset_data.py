#!/usr/bin/env python3
import json
import math
import re
import tomllib
from pathlib import Path

def generate_circuit_svg(data: dict, cat: str, output_path: Path):
    samples = data.get("spline", {}).get("samples", [])
    if not samples:
        waypoints = data.get("spline", {}).get("waypoints", [])
        if not waypoints:
            return
        pts = [(wp["point"][0], -wp["point"][1]) for wp in waypoints]
    else:
        # Downsample to ~180 points for smooth yet lightweight SVG
        step = max(1, len(samples) // 180)
        sampled = samples[::step]
        if samples[-1] != sampled[-1]:
            sampled.append(samples[-1])
        pts = [(s["point"][0], -s["point"][1]) for s in sampled]

    if len(pts) < 2:
        return

    min_x = min(p[0] for p in pts)
    max_x = max(p[0] for p in pts)
    min_y = min(p[1] for p in pts)
    max_y = max(p[1] for p in pts)

    tw = max(max_x - min_x, 1.0)
    th = max(max_y - min_y, 1.0)

    W, H = 400, 260
    pad = 36
    avail_w = W - pad * 2
    avail_h = H - pad * 2
    scale = min(avail_w / tw, avail_h / th)

    cx_track = (min_x + max_x) / 2
    cy_track = (min_y + max_y) / 2
    cx_svg = W / 2
    cy_svg = H / 2

    screen_pts = [
        (round(cx_svg + (x - cx_track) * scale, 1), round(cy_svg + (y - cy_track) * scale, 1))
        for x, y in pts
    ]

    path_d = f"M {screen_pts[0][0]} {screen_pts[0][1]} " + " ".join(f"L {p[0]} {p[1]}" for p in screen_pts[1:]) + " Z"

    # Start/finish normal vector & tick
    dx = screen_pts[1][0] - screen_pts[0][0]
    dy = screen_pts[1][1] - screen_pts[0][1]
    dist = math.hypot(dx, dy) or 1.0
    nx, ny = -dy / dist, dx / dist
    fl_len = 8.0
    fl_x1 = round(screen_pts[0][0] - nx * fl_len, 1)
    fl_y1 = round(screen_pts[0][1] - ny * fl_len, 1)
    fl_x2 = round(screen_pts[0][0] + nx * fl_len, 1)
    fl_y2 = round(screen_pts[0][1] + ny * fl_len, 1)

    category_colors = {
        "gt": {"stroke": "#3b82f6", "glow": "#2563eb"},
        "nascar": {"stroke": "#ef4444", "glow": "#dc2626"},
        "rally": {"stroke": "#10b981", "glow": "#059669"},
        "kart": {"stroke": "#a855f7", "glow": "#9333ea"},
        "offroad": {"stroke": "#f59e0b", "glow": "#d97706"},
        "extreme_offroad": {"stroke": "#f59e0b", "glow": "#d97706"},
        "classic": {"stroke": "#06b6d4", "glow": "#0891b2"},
    }
    colors = category_colors.get(cat, {"stroke": "#38bdf8", "glow": "#0284c7"})

    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="100%" height="100%" class="circuit-miniature">
  <defs>
    <filter id="glow-{cat}" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="4" result="blur" />
      <feMerge>
        <feMergeNode in="blur" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
  </defs>
  <!-- Background Grid Accent -->
  <rect width="{W}" height="{H}" fill="#0b0f19" rx="12" />
  <path d="M 0 65 H 400 M 0 130 H 400 M 0 195 H 400 M 100 0 V 260 M 200 0 V 260 M 300 0 V 260" stroke="#1e293b" stroke-width="0.75" stroke-dasharray="3 3" opacity="0.4" />

  <!-- Outer Glow Track -->
  <path d="{path_d}" fill="none" stroke="{colors['glow']}" stroke-width="9" stroke-linecap="round" stroke-linejoin="round" opacity="0.3" filter="url(#glow-{cat})" />

  <!-- Track Base Asphalt Ribbon -->
  <path d="{path_d}" fill="none" stroke="#1e293b" stroke-width="6" stroke-linecap="round" stroke-linejoin="round" />

  <!-- Track Main Neon Racing Line -->
  <path d="{path_d}" fill="none" stroke="{colors['stroke']}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" />

  <!-- Start/Finish Timing Gate -->
  <line x1="{fl_x1}" y1="{fl_y1}" x2="{fl_x2}" y2="{fl_y2}" stroke="#22c55e" stroke-width="3" stroke-linecap="round" />
  <circle cx="{screen_pts[0][0]}" cy="{screen_pts[0][1]}" r="3.5" fill="#ffffff" stroke="#22c55e" stroke-width="1.5" />
</svg>"""

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(svg, encoding="utf-8")


def generate_assets():
    root = Path(__file__).resolve().parent.parent
    portals_data = root / "portals" / "shared" / "data"
    portals_data.mkdir(parents=True, exist_ok=True)

    print("🚗 Ingesting game assets into portals/shared/data...")

    # 1. Ingest Circuits from tracks/
    tracks_dir = root / "tracks"
    circuits_img_dir = root / "assets" / "textures" / "circuits"
    circuits = []
    
    CLASSIC_TRACK_CATEGORIES = {
        "classic_grand_prix": "gt",
        "drift_park": "gt",
        "figure_eight": "gt",
        "oval_speedway": "nascar",
        "dirty_oval_speedway": "nascar",
        "oasis_rally": "rally",
        "classic_rallycross": "rally",
        "dirt_figure_eight": "offroad",
        "ramp_raceway": "offroad",
        "kart_arena": "kart",
    }

    for json_file in sorted(tracks_dir.rglob("*.json")):
        if json_file.name.startswith("."):
            continue
        try:
            data = json.loads(json_file.read_text(encoding="utf-8"))
            name = data.get("name", json_file.stem.replace("_", " ").title())
            desc = data.get("description", "Competition racing venue.")
            module_name = json_file.parent.name
            
            if module_name == "classic":
                circuit_category = CLASSIC_TRACK_CATEGORIES.get(json_file.stem, "gt")
            elif module_name == "extreme_offroad":
                circuit_category = "offroad"
            else:
                circuit_category = module_name

            modality = circuit_category.upper()
            
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
                surfaces_present.add("Asphalt" if circuit_category in ["gt", "nascar", "kart"] else "Dirt")

            # Extract scale and reference URLs
            scale = data.get("scale", "1:1")
            wikipedia_url = data.get("wikipedia_url")
            osm_url = data.get("osm_url")
            country_code = data.get("country_code")
            country_name = data.get("country_name")
            is_inspired = data.get("is_inspired", False)

            # Compute min and max track width
            samples = data.get("spline", {}).get("samples", [])
            widths = [wp.get("width", 0.0) for wp in waypoints if isinstance(wp, dict) and wp.get("width", 0.0) > 0]
            if not widths:
                widths = [s.get("width", 0.0) for s in samples if isinstance(s, dict) and s.get("width", 0.0) > 0]
            min_width = round(min(widths), 1) if widths else 12.0
            max_width = round(max(widths), 1) if widths else 12.0

            # Generate SVG track miniature
            svg_file = circuits_img_dir / module_name / f"{json_file.stem}.svg"
            generate_circuit_svg(data, circuit_category, svg_file)
            image_url = f"/textures/circuits/{module_name}/{json_file.stem}.svg" if svg_file.exists() else None

            circuits.append({
                "id": json_file.stem,
                "name": name,
                "module": module_name,
                "category": circuit_category,
                "modality": modality,
                "description": desc,
                "scale": scale,
                "wikipedia_url": wikipedia_url,
                "osm_url": osm_url,
                "country_code": country_code,
                "country_name": country_name,
                "min_width_meters": min_width,
                "max_width_meters": max_width,
                "is_inspired": is_inspired,
                "length_meters": round(length_m, 1) if length_m > 0 else 2500.0,
                "turns_count": len([wp for wp in waypoints if wp.get("left_curb") or wp.get("right_curb")]),
                "surfaces": sorted(list(surfaces_present)),
                "has_jumps": has_jumps,
                "image_url": image_url,
                "rel_path": str(json_file.relative_to(root))
            })
        except Exception as e:
            print(f"  ⚠️ Failed parsing {json_file.name}: {e}")

    print(f"  ✅ Parsed {len(circuits)} circuits with SVG miniatures.")
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

        ref_file = root / "assets" / "textures" / "vehicles" / "references" / mod_id / f"{car_id}.jpg"
        lateral_file = root / "assets" / "textures" / "vehicles" / "laterals" / mod_id / f"{car_id}.png"
        thumb_file = root / "assets" / "textures" / "vehicles" / "laterals" / mod_id / f"{car_id}_thumb.png"
        topdown_file = root / "assets" / "textures" / "vehicles" / "topdown" / mod_id / f"{car_id}.png"

        image_ref = f"/textures/vehicles/references/{mod_id}/{car_id}.jpg" if ref_file.exists() else None
        image_lateral = f"/textures/vehicles/laterals/{mod_id}/{car_id}.png" if lateral_file.exists() else None
        image_thumb = f"/textures/vehicles/laterals/{mod_id}/{car_id}_thumb.png" if thumb_file.exists() else None
        image_topdown = f"/textures/vehicles/topdown/{mod_id}/{car_id}.png" if topdown_file.exists() else None

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
            "image_ref": image_ref,
            "image_lateral": image_lateral,
            "image_thumb": image_thumb,
            "image_topdown": image_topdown,
        })

    print(f"  ✅ Compiled {len(vehicles)} vehicles across 5 motorsport modules.")
    (portals_data / "vehicles.json").write_text(json.dumps(vehicles, indent=2), encoding="utf-8")

    # 3. Ingest Surfaces Matrix
    surfaces = [
        {"name": "Asphalt", "friction": 1.00, "rolling_resistance": 1.0, "surface_drag": 1.00, "smoke": True, "roost": False, "splash": False, "layer": "BelowTrack", "description": "Standard dry tarmac; optimal grip baseline, full tire smoke on heavy slip."},
        {"name": "Concrete", "friction": 0.95, "rolling_resistance": 1.05, "surface_drag": 1.00, "smoke": True, "roost": False, "splash": False, "layer": "BelowTrack", "description": "Poured solid pavement; high grip with low rolling drag for grandstands and stadium bowls."},
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
