#!/usr/bin/env python3
import json
import math
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
        "autocross": {"stroke": "#eab308", "glow": "#ca8a04"},
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
        "gt_coastal_grand_prix": "gt",
        "gt_ridge_ring": "gt",
        "gt_velocity_park": "gt",
        "stock_tri_oval_speedway": "nascar",
        "stock_thunder_bowl": "nascar",
        "stock_roval": "nascar",
        "kart_pine_grove": "kart",
        "kart_riverbend_circuit": "kart",
        "kart_summit_international": "kart",
        "kart_hangar_sprint": "kart",
        "kart_warehouse_twister": "kart",
        "kart_tower_labyrinth": "kart",
        "rx_quarry_sprint": "rally",
        "rx_hilltop_leap": "rally",
        "rx_canyon_flyer": "rally",
        "ax_meadow_sprint": "autocross",
        "ax_clay_bowl": "autocross",
        "ax_hillside_hammer": "autocross",
        "at_dune_sea": "offroad",
        "at_mudbath_valley": "offroad",
        "at_frostbite_pass": "offroad",
    }

    for json_file in sorted(tracks_dir.rglob("*.json")):
        if json_file.name.startswith(".") or json_file.parent.name == "vault" or json_file.name == "MANIFEST.json":
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

    print(f"  ✅ Generated SVG miniatures for {len(circuits)} circuits.")

if __name__ == "__main__":
    generate_assets()

