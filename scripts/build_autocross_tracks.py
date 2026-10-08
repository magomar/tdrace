#!/usr/bin/env python3
"""Builds 17 FIA Autocross circuits from real-world OpenStreetMap (OSM) survey data.

Fetches closed raceway ways from OpenStreetMap API, projects coordinates to metric
tangent plane, aligns heading along +X, scales to FIA homologation length,
resamples to 30 uniform waypoints, detects apex curbs, and outputs tracks/autocross/<id>.json.
"""

import json
import math
import os
import sys
import time
import urllib.request

sys.path.insert(0, os.path.dirname(__file__))
from osm_importer import compute_spline_length, shift_start, smooth_hairpin_arc_fans

TRACKS = [
    {
        "id": "nova_paka_ax",
        "name": "Nová Paka - Štikovská rokle",
        "description": "The legendary cathedral of European Autocross. Brutal 25-meter elevation drops, high-speed clay bowls, and unpredictable terrain changes.",
        "way_id": 166701854,
        "target_length": 1060.0,
        "default_laps": 6,
        "country_code": "CZ",
        "country_name": "Czech Republic",
        "tag": "EURO AX CZECH REPUBLIC",
        "wikipedia_url": "https://cs.wikipedia.org/wiki/%C5%A0tikovsk%C3%A1_rokle",
    },
    {
        "id": "prerov_ax",
        "name": "Přerov - Přerovská rokle",
        "description": "Historic Moravian dirt cathedral famed for the terrifying 15-meter 'Mamut' jump drop and deep amphitheatre sweepers.",
        "way_id": 159576897,
        "target_length": 1000.0,
        "default_laps": 5,
        "country_code": "CZ",
        "country_name": "Czech Republic",
        "tag": "EURO AX CZECH REPUBLIC",
        "wikipedia_url": "https://en.wikipedia.org/wiki/P%C5%99erov",
    },
    {
        "id": "humpolec_ax",
        "name": "Humpolec - Pod Vilémovským lomem",
        "description": "Quarry-side natural amphitheatre with high-grip loam berms, blind crests, and fast sweeping technical turns.",
        "way_id": 100006815,
        "target_length": 980.0,
        "default_laps": 5,
        "country_code": "CZ",
        "country_name": "Czech Republic",
        "tag": "EURO AX CZECH REPUBLIC",
        "wikipedia_url": "https://en.wikipedia.org/wiki/Humpolec",
    },
    {
        "id": "matschenberg_ax",
        "name": "Matschenberg Offroad Arena",
        "description": "The fastest dirt circuit in Europe, set in Upper Lusatia with a radical 30-meter vertical plunge into a high-speed clay bowl.",
        "way_id": 1363616670,
        "target_length": 1180.0,
        "default_laps": 5,
        "country_code": "DE",
        "country_name": "Germany",
        "tag": "EURO AX GERMANY",
        "wikipedia_url": "https://de.wikipedia.org/wiki/Cunewalde",
    },
    {
        "id": "seelow_ax",
        "name": "Circuit Am Weinberg Seelow",
        "description": "Traditional season opener for the FIA European Championship in Brandenburg, featuring wide sandy banked turns.",
        "way_id": 119083954,
        "target_length": 1170.0,
        "default_laps": 4,
        "country_code": "DE",
        "country_name": "Germany",
        "tag": "EURO AX GERMANY",
        "wikipedia_url": "https://de.wikipedia.org/wiki/Seelow",
    },
    {
        "id": "schluechtern_ax",
        "name": "Ewald-Pauli-Ring Schlüchtern",
        "description": "Technical Hessian hillside circuit demanding precise throttle control through rutted clay off-camber hairpins.",
        "way_id": 134067270,
        "target_length": 1200.0,
        "default_laps": 5,
        "country_code": "DE",
        "country_name": "Germany",
        "tag": "EURO AX GERMANY",
        "wikipedia_url": "https://de.wikipedia.org/wiki/Schl%C3%BCchtern",
    },
    {
        "id": "uelzen_ax",
        "name": "Uhlenköper-Ring Uelzen",
        "description": "Undulating northern German dirt venue in Lower Saxony with technical rhythm sections, jumps, and tight clay corners.",
        "way_id": 62020741,
        "target_length": 810.0,
        "default_laps": 5,
        "country_code": "DE",
        "country_name": "Germany",
        "tag": "EURO AX GERMANY",
        "wikipedia_url": "https://de.wikipedia.org/wiki/Uelzen",
    },
    {
        "id": "st_georges_ax",
        "name": "Circuit du Bouvreau Saint-Georges",
        "description": "The Temple of French Autocross in Vendée, hosting 20,000 spectators around high-speed banked red clay bowls.",
        "way_id": 674904429,
        "target_length": 940.0,
        "start_offset_m": 62.0,
        "default_laps": 5,
        "country_code": "FR",
        "country_name": "France",
        "tag": "EURO AX FRANCE",
        "wikipedia_url": "https://fr.wikipedia.org/wiki/Saint-Georges-de-Montaigu",
    },
    {
        "id": "faleyras_ax",
        "name": "Circuit de Faleyras",
        "description": "Prestigious French natural amphitheatre track in Gironde with dramatic elevation plunges and wide dirt bowls.",
        "way_id": 808191053,
        "target_length": 1150.0,
        "default_laps": 5,
        "country_code": "FR",
        "country_name": "France",
        "tag": "EURO AX FRANCE",
        "wikipedia_url": "https://fr.wikipedia.org/wiki/Faleyras",
    },
    {
        "id": "st_junien_ax",
        "name": "Circuit de Saint-Junien",
        "description": "Classic French national autocross venue in Haute-Vienne featuring fast sweeping turns and technical chicanes.",
        "way_id": 685441292,
        "target_length": 1260.0,
        "default_laps": 5,
        "country_code": "FR",
        "country_name": "France",
        "tag": "EURO AX FRANCE",
        "wikipedia_url": "https://fr.wikipedia.org/wiki/Saint-Junien",
    },
    {
        "id": "bazaigues_ax",
        "name": "Circuit de Bazaigues",
        "description": "Technical French sprint track in Indre with rolling terrain and wide, multi-line dirt hairpins.",
        "way_id": 803500149,
        "target_length": 1050.0,
        "start_offset_m": 72.0,
        "default_laps": 4,
        "country_code": "FR",
        "country_name": "France",
        "tag": "EURO AX FRANCE",
        "wikipedia_url": "https://fr.wikipedia.org/wiki/Bazaiges",
    },
    {
        "id": "maggiora_ax",
        "name": "Autodromo Pragiarolo Maggiora",
        "description": "Legendary Italian dirt colosseum in Piedmont with dramatic cambers, high-speed esses, and blind crests.",
        "way_id": 152379106,
        "target_length": 1390.0,
        "default_laps": 5,
        "country_code": "IT",
        "country_name": "Italy",
        "tag": "EURO AX ITALY",
        "wikipedia_url": "https://it.wikipedia.org/wiki/Maggiora",
    },
    {
        "id": "musa_ax",
        "name": "Mūsa Raceland Bauska",
        "description": "The Baltic autocross capital in Latvia, known for its ultra-wide sandy loam surface in a scenic natural bowl.",
        "way_id": 131117868,
        "target_length": 1120.0,
        "default_laps": 5,
        "country_code": "LV",
        "country_name": "Latvia",
        "tag": "EURO AX LATVIA",
        "wikipedia_url": "https://en.wikipedia.org/wiki/Bauska",
    },
    {
        "id": "vilkyciai_ax",
        "name": "Vilkyčiai Autosporto Kompleksas",
        "description": "Demanding Lithuanian circuit with natural gullies, heavy braking zones, and deep wheel ruts.",
        "way_id": 431154657,
        "target_length": 880.0,
        "default_laps": 4,
        "country_code": "LT",
        "country_name": "Lithuania",
        "tag": "EURO AX LITHUANIA",
        "wikipedia_url": "https://lt.wikipedia.org/wiki/Vilky%C4%8Diai",
    },
    {
        "id": "arteixo_ax",
        "name": "Circuito José Ramón Losada Arteixo",
        "description": "'La Catedral' of Spanish Autocross in Galicia, featuring high-speed stadium carousels lined with thousands of fans.",
        "way_id": 192380335,
        "target_length": 1325.0,
        "default_laps": 5,
        "country_code": "ES",
        "country_name": "Spain",
        "tag": "EURO AX SPAIN",
        "wikipedia_url": "https://es.wikipedia.org/wiki/Arteijo",
    },
    {
        "id": "carballo_ax",
        "name": "Circuito Motor Bértoa Carballo",
        "description": "Galician technical dirt bowl with sharp switchbacks, elevation changes, and high traction demand.",
        "way_id": 854048813,
        "target_length": 965.0,
        "default_laps": 5,
        "country_code": "ES",
        "country_name": "Spain",
        "tag": "EURO AX SPAIN",
        "wikipedia_url": "https://es.wikipedia.org/wiki/Carballo_(La_Coru%C3%B1a)",
    },
    {
        "id": "castelo_branco_ax",
        "name": "Parque de Desportos Motorizados Castelo Branco",
        "description": "Premier Portuguese off-road venue with undulating high-traction clay and loose shale margins.",
        "way_id": 436180509,
        "target_length": 1250.0,
        "default_laps": 5,
        "country_code": "PT",
        "country_name": "Portugal",
        "tag": "EURO AX PORTUGAL",
        "wikipedia_url": "https://pt.wikipedia.org/wiki/Castelo_Branco",
    },
]

def latlon_to_meters(lat, lon, lat0, lon0):
    r = 6378137.0
    x = (math.radians(lon) - math.radians(lon0)) * math.cos(math.radians(lat0)) * r
    y = (math.radians(lat) - math.radians(lat0)) * r
    return x, y

def fetch_osm_way(way_id):
    url = f"https://api.openstreetmap.org/api/0.6/way/{way_id}/full.json"
    req = urllib.request.Request(url, headers={"User-Agent": "tdrace-autocross-importer/1.0"})
    with urllib.request.urlopen(req) as resp:
        data = json.loads(resp.read().decode("utf-8"))
    
    elems = data.get("elements", [])
    node_map = {e["id"]: (e["lat"], e["lon"]) for e in elems if e["type"] == "node"}
    way = next(e for e in elems if e["type"] == "way" and e["id"] == way_id)
    node_ids = way["nodes"]
    last_id = node_ids[-1]
    if last_id in node_ids[:-1]:
        first_idx = node_ids.index(last_id)
        node_ids = node_ids[first_idx:]
    
    # Extract coordinates in order
    coords = [node_map[nid] for nid in node_ids if nid in node_map]
    return coords

def resample_polyline(points, num_points):
    """Resamples a closed polyline to num_points equidistant points."""
    # Ensure closed loop
    pts = list(points)
    if pts[0] != pts[-1]:
        pts.append(pts[0])
    
    seg_lens = []
    total_len = 0.0
    for i in range(len(pts) - 1):
        dx = pts[i+1][0] - pts[i][0]
        dy = pts[i+1][1] - pts[i][1]
        l = math.hypot(dx, dy)
        seg_lens.append(l)
        total_len += l
    
    step = total_len / num_points
    resampled = []
    curr_seg = 0
    curr_dist = 0.0
    accum_dist = 0.0
    
    for i in range(num_points):
        target_dist = i * step
        while curr_seg < len(seg_lens) and accum_dist + seg_lens[curr_seg] < target_dist:
            accum_dist += seg_lens[curr_seg]
            curr_seg += 1
        
        if curr_seg >= len(seg_lens):
            resampled.append(pts[-1])
        else:
            t = (target_dist - accum_dist) / max(seg_lens[curr_seg], 1e-6)
            x = pts[curr_seg][0] + t * (pts[curr_seg+1][0] - pts[curr_seg][0])
            y = pts[curr_seg][1] + t * (pts[curr_seg+1][1] - pts[curr_seg][1])
            resampled.append((x, y))
    
    return resampled

def main():
    tracks_dir = os.path.join(os.path.dirname(__file__), "..", "tracks")
    ax_dir = os.path.join(tracks_dir, "autocross")
    os.makedirs(ax_dir, exist_ok=True)
    
    track_ids = []

    for tr in TRACKS:
        cid = tr["id"]
        track_ids.append(cid)
        print(f"--> Processing {cid} (OSM way/{tr['way_id']})...")
        coords = fetch_osm_way(tr["way_id"])
        if len(coords) < 3:
            raise ValueError(f"Too few points for {cid}: {len(coords)}")
        
        # 1. Project to meters around datum (coord[0])
        lat0, lon0 = coords[0]
        pts_m = [latlon_to_meters(lat, lon, lat0, lon0) for lat, lon in coords]

        # Shift start if configured (to move start/finish away from acute hairpins)
        if tr.get("start_offset_m"):
            pts_m = shift_start(pts_m, tr["start_offset_m"])

        # 2. Heading alignment along +X using first straight segment
        dx = pts_m[1][0] - pts_m[0][0]
        dy = pts_m[1][1] - pts_m[0][1]
        heading = math.atan2(dy, dx)
        cos_h = math.cos(-heading)
        sin_h = math.sin(-heading)
        pts_rot = [(x * cos_h - y * sin_h, x * sin_h + y * cos_h) for x, y in pts_m]

        # 3. Scale to official FIA target length (1:1 scale)
        measured_len = 0.0
        for i in range(len(pts_rot) - 1):
            measured_len += math.hypot(pts_rot[i+1][0] - pts_rot[i][0], pts_rot[i+1][1] - pts_rot[i][1])

        scale = tr["target_length"] / max(measured_len, 1e-3)
        pts_scaled = [(x * scale, y * scale) for x, y in pts_rot]

        # 4. Uniformly resample to 30 points
        resampled = resample_polyline(pts_scaled, 30)

        # 5. Detect apex curbs from normalized cross product
        n = len(resampled)
        waypoints = []
        for i in range(n):
            prev_p = resampled[(i - 1 + n) % n]
            curr_p = resampled[i]
            next_p = resampled[(i + 1) % n]

            v1 = (curr_p[0] - prev_p[0], curr_p[1] - prev_p[1])
            v2 = (next_p[0] - curr_p[0], next_p[1] - curr_p[1])
            l1 = max(math.hypot(v1[0], v1[1]), 1e-6)
            l2 = max(math.hypot(v2[0], v2[1]), 1e-6)
            nv1 = (v1[0] / l1, v1[1] / l1)
            nv2 = (v2[0] / l2, v2[1] / l2)
            cross = nv1[0] * nv2[1] - nv1[1] * nv2[0]

            left_curb = cross > 0.35
            right_curb = cross < -0.35

            waypoints.append({
                "point": [round(curr_p[0], 1), round(curr_p[1], 1)],
                "width": 13.0,
                "left_curb": left_curb,
                "right_curb": right_curb,
                "surface": "Dirt",
                "elevation": 0.0,
                "bank_angle": 0.0,
                "left_wall": True,
                "right_wall": True,
                "left_wall_distance": None,
                "right_wall_distance": None,
                "left_runoff_surface": "Grass",
                "right_runoff_surface": "Grass",
            })

        # Spec 097 Pillar V: Multi-waypoint hairpin arc fans for acute corners
        waypoints = smooth_hairpin_arc_fans(
            waypoints,
            target_radius=13.5,
            min_deflection_deg=45.0,
            default_width=13.0,
        )

        # Spec 097 Pillar II: Calibrate spline length to exactly match target_length (+-0.5%)
        pts = [w["point"] for w in waypoints]
        cur_splen = compute_spline_length(pts)
        if cur_splen > 0:
            calib = tr["target_length"] / cur_splen
            for w in waypoints:
                w["point"] = [round(w["point"][0] * calib, 1), round(w["point"][1] * calib, 1)]
            
        track_json = {
            "name": tr["name"],
            "description": tr["description"],
            "category": "main",
            "kind": {"type": "circuit"},
            "spline": {
                "waypoints": waypoints,
                "closed": True,
                "samples": [],
                "total_length": 0.0,
                "curves": [],
            },
            "geometry": {
                "inner_walls": [],
                "outer_walls": [],
                "obstacles": [],
                "surface_zones": [],
                "jump_ramps": [],
                "left_boundary_polyline": [],
                "right_boundary_polyline": [],
            },
            "checkpoints": [],
            "grid_positions": [],
            "default_surface": "Grass",
            "pit_box_area": None,
            "default_laps": tr["default_laps"],
            "car_category": "autocross",
            "module_id": "autocross",
            "modules": ["autocross"],
            "scale": "1:1",
            "wikipedia_url": tr["wikipedia_url"],
            "osm_url": f"https://www.openstreetmap.org/way/{tr['way_id']}",
            "country_code": tr["country_code"],
            "country_name": tr["country_name"],
            "is_inspired": False,
            "tag": tr["tag"],
            "category_label": "FIA Autocross",
        }
        
        out_path = os.path.join(ax_dir, f"{cid}.json")
        with open(out_path, "w", encoding="utf-8") as f:
            json.dump(track_json, f, indent=2)
        print(f"    Wrote {out_path}")
        time.sleep(0.15)
        
    # Update tracks/.track_order.json
    order_path = os.path.join(tracks_dir, ".track_order.json")
    with open(order_path, "r", encoding="utf-8") as f:
        order = json.load(f)
    
    order["autocross"] = track_ids
    with open(order_path, "w", encoding="utf-8") as f:
        json.dump(order, f, indent=2)
    print(f"Updated {order_path} with 'autocross' {len(track_ids)} circuits.")

if __name__ == "__main__":
    main()
