#!/usr/bin/env python3
"""
OSM Track Importer for tdrace

Extracts real-world motorsport raceway waypoints from OpenStreetMap (OSM) via Overpass API,
projects them to metric 2D Cartesian coordinates, scales them to official FIA homologation
lengths, aligns the start/finish straight with the +X axis, and outputs ready-to-use Rust
track definitions for tdrace-core/src/track/presets.rs.
"""

import argparse
import json
import math
import os
import urllib.parse
import urllib.request

CACHE_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "target", "osm_cache")
os.makedirs(CACHE_DIR, exist_ok=True)

TRACK_SPECS = {
    "holjes_rx": {
        "name": "Höljes Motorstadion (World RX Sweden)",
        "description": "The Holy Grail of Rallycross in Sweden featuring the legendary Höljes Jump, banked Velodrome & mixed gravel sliding.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](60.910,12.545,60.925,12.565););out body;>;out skel qt;',
        "way_ids": [599300791, 599300793, 599300794, 599300795, 599300796, 599300798, 599300799, 599300800, 599300802, 599300806],
        "fia_length": 1210.0,
        "default_width": 13.5,
        "straight_width": 14.5,
        "num_waypoints": 30,
        "jump": {
            "name": "Höljes Jump Crest",
            "at_fraction": 0.44,  # Approx after turn 2 crest
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
    "lydden_hill": {
        "name": "Lydden Hill Circuit (World RX Great Britain)",
        "description": "The historic birthplace of Rallycross featuring the iconic Chessons Drift gravel slide, North Bend & Devil's Elbow.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](51.170,1.185,51.185,1.205););out body;>;out skel qt;',
        "way_ids": [234347787, 797343741, 797343742, 797343743, 797343744, 797343745, 797343740],
        "fia_length": 1170.0,
        "default_width": 13.0,
        "straight_width": 14.0,
        "num_waypoints": 28,
        "jump": None,
    },
    "hell_rx": {
        "name": "Lånkebanen (World RX Norway)",
        "description": "Welcome to Hell! Fast downhill asphalt sweep, loose gravel carousel, technical esses & high-flying crests.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](63.395,10.895,63.42,10.935););out body;>;out skel qt;',
        "custom_ways": "hell_rx",
        "fia_length": 1019.0,
        "default_width": 13.0,
        "straight_width": 14.0,
        "num_waypoints": 28,
        "jump": {
            "name": "Hell Gravel Jump",
            "at_fraction": 0.58,
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
    "loheac_rx": {
        "name": "Circuit de Lohéac (World RX France)",
        "description": "The French Rallycross classic in Brittany with long asphalt drag straight, gravel tabletop jump & tight switchbacks.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](47.860,-1.898,47.868,-1.890););out body;>;out skel qt;',
        "way_ids": [787615501, 787615503, 787615504, 787615506, 787615505],
        "fia_length": 1088.0,
        "default_width": 13.0,
        "straight_width": 14.5,
        "num_waypoints": 28,
        "jump": {
            "name": "Lohéac Infield Jump",
            "at_fraction": 0.38,
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
    "estering_rx": {
        "name": "Estering Buxtehude (World RX Germany)",
        "description": "The cathedral of German Rallycross featuring the iconic Turn 1 hairpin dive, high-speed forest drag and technical gravel carousel.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](53.435,9.680,53.445,9.705););out body;>;out skel qt;',
        "fia_length": 952.0,
        "default_width": 13.0,
        "straight_width": 14.5,
        "num_waypoints": 26,
        "jump": None,
    },
    "montalegre_rx": {
        "name": "Pista Automóvel de Montalegre (World RX Portugal)",
        "description": "High-altitude mountain thriller in Portugal featuring an undulating drag straight, gravel stadium section and fast table crest.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](41.815,-7.795,41.830,-7.770););out body;>;out skel qt;',
        "fia_length": 1050.0,
        "default_width": 13.0,
        "straight_width": 14.0,
        "num_waypoints": 28,
        "jump": {
            "name": "Montalegre Dirt Table",
            "at_fraction": 0.77,
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
    "nyirad_rx": {
        "name": "Nyirád Racing Center (Euro RX Hungary)",
        "description": "The infamous 'Red Cauldron' carved out of red bauxite quarries, featuring heavy gravel elevation changes and sweeping technical slides.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](46.995,17.435,47.010,17.465););out body;>;out skel qt;',
        "fia_length": 1220.0,
        "default_width": 13.5,
        "straight_width": 14.5,
        "num_waypoints": 30,
        "jump": None,
    },
    "kouvola_rx": {
        "name": "Tykkimäen Moottorirata (World RX Finland)",
        "description": "Finnish rallycross heartland featuring severe elevation rollercoasters, blind gravel drops and the flying Tykkimäki dirt crest.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](60.885,26.785,60.900,26.830););out body;>;out skel qt;',
        "fia_length": 1060.0,
        "default_width": 13.0,
        "straight_width": 14.0,
        "num_waypoints": 28,
        "jump": {
            "name": "Tykkimäki Dirt Jump",
            "at_fraction": 0.62,
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
    "catalunya_rx": {
        "name": "Circuit de Barcelona-Catalunya RX (World RX Spain)",
        "description": "World RX stadium circuit inside the iconic Spanish Grand Prix stadium, featuring downhill gravel hairpin slides and stadium jump.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](41.560,2.250,41.575,2.268););out body;>;out skel qt;',
        "fia_length": 1125.0,
        "default_width": 13.5,
        "straight_width": 14.5,
        "num_waypoints": 28,
        "jump": {
            "name": "Stadium Dirt Jump",
            "at_fraction": 0.48,
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
}


def query_osm_cached(name, query):
    cache_file = os.path.join(CACHE_DIR, f"{name}.json")
    if os.path.exists(cache_file):
        with open(cache_file, "r") as f:
            return json.load(f)

    endpoints = [
        "https://overpass-api.de/api/interpreter",
        "https://overpass.kumi.systems/api/interpreter",
    ]
    for ep in endpoints:
        try:
            req = urllib.request.Request(
                ep, data=query.encode("utf-8"), headers={"User-Agent": "tdrace-osm-tool/1.0"}
            )
            with urllib.request.urlopen(req, timeout=20) as resp:
                data = json.loads(resp.read().decode("utf-8"))
                with open(cache_file, "w") as f:
                    json.dump(data, f)
                return data
        except Exception:
            continue
    raise RuntimeError(f"Failed to query OSM Overpass for {name}")


def latlon_to_meters(lat, lon, lat0, lon0):
    r = 6378137.0
    x = (math.radians(lon) - math.radians(lon0)) * math.cos(math.radians(lat0)) * r
    y = (math.radians(lat) - math.radians(lat0)) * r
    return x, y


def polyline_length(pts, closed=True):
    total = 0.0
    n = len(pts)
    for i in range(n if closed else n - 1):
        p0 = pts[i]
        p1 = pts[(i + 1) % n]
        total += math.hypot(p1[0] - p0[0], p1[1] - p0[1])
    return total


def resample_polyline(points_with_props, target_count):
    n = len(points_with_props)
    cum_dists = [0.0]
    for i in range(n):
        p0 = points_with_props[i][0]
        p1 = points_with_props[(i + 1) % n][0]
        cum_dists.append(cum_dists[-1] + math.hypot(p1[0] - p0[0], p1[1] - p0[1]))

    total_len = cum_dists[-1]
    step = total_len / target_count
    resampled = []
    seg_idx = 0

    for k in range(target_count):
        d = k * step
        while seg_idx < n and cum_dists[seg_idx + 1] < d:
            seg_idx += 1
        d0 = cum_dists[seg_idx]
        d1 = cum_dists[seg_idx + 1]
        t = (d - d0) / (d1 - d0) if (d1 - d0) > 1e-6 else 0.0

        p0 = points_with_props[seg_idx][0]
        p1 = points_with_props[(seg_idx + 1) % n][0]
        x = p0[0] + t * (p1[0] - p0[0])
        y = p0[1] + t * (p1[1] - p0[1])
        surf = points_with_props[seg_idx][1]
        resampled.append(((x, y), surf))

    return resampled, total_len


def process_track(track_id):
    spec = TRACK_SPECS[track_id]
    data = query_osm_cached(track_id, spec["query"])

    nodes = {e["id"]: (e["lat"], e["lon"]) for e in data["elements"] if e["type"] == "node"}
    ways = {e["id"]: e for e in data["elements"] if e["type"] == "way"}

    raw_nodes_surf = []
    if track_id == "hell_rx":
        nodes_67 = ways[1069390967]["nodes"][1:]  # Skip start grid lane (node 0)
        nodes_68 = ways[1069390968]["nodes"]
        nodes_78 = ways[1069390978]["nodes"]
        nodes_77 = ways[1069390977]["nodes"]
        for nid in nodes_67[:-1]:
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
        for nid in nodes_68[:-1]:
            raw_nodes_surf.append((nodes[nid], "Dirt"))
        for nid in nodes_78[:-1]:
            raw_nodes_surf.append((nodes[nid], "Dirt"))
        for nid in nodes_77[19:-1]:
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
    elif track_id == "holjes_rx":
        for wid in spec["way_ids"]:
            w = ways[wid]
            tags = w.get("tags", {})
            surf_tag = tags.get("surface", "")
            surf = "Dirt" if surf_tag in ["gravel", "fine_gravel", "dirt"] else "Asphalt"
            wnodes = w["nodes"]
            if wid == 599300791:
                wnodes = wnodes[2:]  # Skip start drag lane (nodes 0, 1)
            for nid in wnodes[:-1]:
                raw_nodes_surf.append((nodes[nid], surf))
    elif track_id == "loheac_rx":
        for wid in spec["way_ids"]:
            w = ways[wid]
            tags = w.get("tags", {})
            surf_tag = tags.get("surface", "")
            surf = "Dirt" if surf_tag in ["gravel", "fine_gravel", "dirt"] else "Asphalt"
            wnodes = w["nodes"]
            if wid == 787615501:
                wnodes = wnodes[2:]  # Skip start drag lane (nodes 0, 1)
            for nid in wnodes[:-1]:
                raw_nodes_surf.append((nodes[nid], surf))
    elif track_id == "estering_rx":
        w = ways[267094190]["nodes"]
        estering_nodes = w[15:-1] + w[:15]
        for i, nid in enumerate(estering_nodes):
            surf = "Dirt" if 7 <= i <= 28 else "Asphalt"
            raw_nodes_surf.append((nodes[nid], surf))
    elif track_id == "montalegre_rx":
        for nid in ways[532046956]["nodes"][:-1]:
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
        for nid in ways[1096210264]["nodes"][:-1]:
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
        for nid in ways[532046958]["nodes"][:-1]:
            raw_nodes_surf.append((nodes[nid], "Dirt"))
        for nid in ways[1096210265]["nodes"][:-1]:
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
    elif track_id == "nyirad_rx":
        for nid in ways[172413359]["nodes"]:
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
        for nid in reversed(ways[172413357]["nodes"][:15]):
            raw_nodes_surf.append((nodes[nid], "Dirt"))
        for nid in ways[172413356]["nodes"][12:24]:
            raw_nodes_surf.append((nodes[nid], "Dirt"))
    elif track_id == "kouvola_rx":
        for nid in ways[149713976]["nodes"][36:-1]:
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
        for nid in ways[149713976]["nodes"][:7]:
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
        for nid in ways[149713907]["nodes"][:40]:
            raw_nodes_surf.append((nodes[nid], "Dirt"))
    elif track_id == "catalunya_rx":
        for nid in ways[831804327]["nodes"][200:-1]:
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
        for nid in ways[831804327]["nodes"][:3]:
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
        for nid in reversed(ways[921317982]["nodes"]):
            raw_nodes_surf.append((nodes[nid], "Dirt"))
        for nid in reversed(ways[967275593]["nodes"][1:]):
            raw_nodes_surf.append((nodes[nid], "Asphalt"))
        for nid in reversed(ways[921317981]["nodes"][1:-1]):
            raw_nodes_surf.append((nodes[nid], "Dirt"))
    else:
        for wid in spec["way_ids"]:
            w = ways[wid]
            tags = w.get("tags", {})
            name = tags.get("name", "")
            surf_tag = tags.get("surface", "")
            if surf_tag in ["gravel", "fine_gravel", "dirt"] or "Drift" in name or "Slope" in name:
                surf = "Dirt"
            else:
                surf = "Asphalt"
            for nid in w["nodes"][:-1]:
                raw_nodes_surf.append((nodes[nid], surf))

    lat0 = sum(p[0][0] for p in raw_nodes_surf) / len(raw_nodes_surf)
    lon0 = sum(p[0][1] for p in raw_nodes_surf) / len(raw_nodes_surf)

    metric_pts = [
        (latlon_to_meters(p[0][0], p[0][1], lat0, lon0), p[1])
        for p in raw_nodes_surf
    ]

    # Calculate start straight heading (using first 5-8 points)
    p_start = metric_pts[0][0]
    p_ahead = metric_pts[min(6, len(metric_pts) - 1)][0]
    heading = math.atan2(p_ahead[1] - p_start[1], p_ahead[0] - p_start[0])

    # Rotate so start straight heads along +X
    cos_a = math.cos(-heading)
    sin_a = math.sin(-heading)
    rotated_pts = []
    for (x, y), surf in metric_pts:
        rx = x * cos_a - y * sin_a
        ry = x * sin_a + y * cos_a
        rotated_pts.append(((rx, ry), surf))

    # Scale to exact FIA homologation length
    current_len = polyline_length([p[0] for p in rotated_pts], closed=True)
    scale_factor = spec["fia_length"] / current_len if current_len > 0 else 1.0

    scaled_pts = [
        (((x * scale_factor), (y * scale_factor)), surf)
        for (x, y), surf in rotated_pts
    ]

    # Translate so start line is at x=0, y=0
    x_offset = scaled_pts[0][0][0]
    y_offset = scaled_pts[0][0][1]
    aligned_pts = [
        (((x - x_offset), (y - y_offset)), surf)
        for (x, y), surf in scaled_pts
    ]

    # Resample to target waypoint count
    resampled, final_len = resample_polyline(aligned_pts, spec["num_waypoints"])

    # Compute curvature and assign apex curbs
    n = len(resampled)
    waypoints = []
    for i in range(n):
        (x, y), surf = resampled[i]
        p_prev = resampled[(i - 1 + n) % n][0]
        p_next = resampled[(i + 1) % n][0]

        v1 = (x - p_prev[0], y - p_prev[1])
        v2 = (p_next[0] - x, p_next[1] - y)
        cross = v1[0] * v2[1] - v1[1] * v2[0]

        # Width
        is_straight = abs(cross) < 15.0 and i < 4
        width = spec["straight_width"] if is_straight else spec["default_width"]

        # Curbs: if cornering sharp enough
        left_curb = False
        right_curb = False
        if cross > 60.0:  # Turning Left -> right side curb / apex curb
            left_curb = True
        elif cross < -60.0:  # Turning Right -> right side curb
            right_curb = True

        waypoints.append({
            "x": round(x, 1),
            "y": round(y, 1),
            "width": round(width, 1),
            "surface": surf,
            "left_curb": left_curb,
            "right_curb": right_curb,
        })

    return {
        "id": track_id,
        "name": spec["name"],
        "description": spec["description"],
        "waypoints": waypoints,
        "total_length": round(final_len, 1),
        "fia_length": spec["fia_length"],
        "jump": spec["jump"],
    }


def generate_rust_code(track_data):
    wps = track_data["waypoints"]
    lines = []
    lines.append("    let waypoints = vec![")
    for i, w in enumerate(wps):
        curb_str = ""
        if w["left_curb"] or w["right_curb"]:
            curb_str = f".with_curbs({str(w['left_curb']).lower()}, {str(w['right_curb']).lower()})"
        surf_str = f".with_surface(SurfaceType::{w['surface']})"
        lines.append(
            f"        TrackWaypoint::new(Vec2::new({w['x']:.1f}, {w['y']:.1f}), {w['width']:.1f}){surf_str}{curb_str},"
        )
    lines.append("    ];")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Extract and generate tdrace tracks from OSM")
    parser.add_argument("--track", choices=list(TRACK_SPECS.keys()), help="Process specific track")
    parser.add_argument("--all", action="store_true", help="Process all rallycross tracks")
    parser.add_argument("--rust", action="store_true", help="Print Rust waypoint code")
    args = parser.parse_args()

    tracks_to_process = list(TRACK_SPECS.keys()) if args.all or not args.track else [args.track]
    for tid in tracks_to_process:
        res = process_track(tid)
        print(f"=== {res['name']} ({res['id']}) ===")
        print(f"  Waypoints: {len(res['waypoints'])}, Total Length: {res['total_length']} m (FIA Target: {res['fia_length']} m)")
        asphalt_cnt = sum(1 for w in res['waypoints'] if w['surface'] == 'Asphalt')
        dirt_cnt = len(res['waypoints']) - asphalt_cnt
        print(f"  Surface: {asphalt_cnt * 100 // len(res['waypoints'])}% Asphalt, {dirt_cnt * 100 // len(res['waypoints'])}% Dirt")
        if args.rust:
            print(generate_rust_code(res))
            print()


if __name__ == "__main__":
    main()
