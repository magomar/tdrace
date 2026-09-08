#!/usr/bin/env python3
"""
OSM Kart Track Importer for tdrace

Extracts real-world kart raceways from OpenStreetMap (OSM) via direct OSM API or Overpass,
projects them to metric 2D Cartesian coordinates, scales them to official CIK-FIA homologation
lengths, aligns the start/finish straight with the +X axis, calculates inside corner apex curbs,
and generates ready-to-use Rust track definitions for crates/tdrace-app/src/module/kart.rs.
"""

import argparse
import json
import math
import os
import re
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET

CACHE_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "target", "osm_cache")
os.makedirs(CACHE_DIR, exist_ok=True)

KART_TRACK_SPECS = {
    "lonato": {
        "name": "South Garda Karting (Lonato)",
        "description": "The global Mecca of Karting featuring Curva del Paddock, Pettine hairpin, and Variante Nuova.",
        "bbox": (10.503, 45.422, 10.510, 45.428),
        "ways": [75490872],
        "fia_length": 1200.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 30,
        "elevation_fn": None,
    },
    "sarno": {
        "name": "Circuito Internazionale Napoli (Sarno)",
        "description": "The Temple of Speed under Mount Vesuvius with massive full-throttle straights and technical Esses.",
        "bbox": (14.560, 40.832, 14.572, 40.840),
        "ways": [643206090],
        "fia_length": 1550.0,
        "default_width": 8.5,
        "straight_width": 9.5,
        "num_waypoints": 32,
        "elevation_fn": None,
    },
    "genk": {
        "name": "Karting Genk (Home of Champions)",
        "description": "Legendary Belgian proving grounds featuring the high-G G-Curve carousel, Europabocht, and Champions Chicane.",
        "bbox": (5.555, 50.980, 5.575, 50.995),
        "ways": [67660672],
        "fia_length": 1360.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 30,
        "elevation_fn": None,
    },
    "pfi": {
        "name": "PF International Kart Circuit (PFI)",
        "description": "Britain's premier FIA kart venue featuring the world-famous elevated flyover crossover bridge and underpass.",
        "bbox": (-0.666, 53.034, -0.654, 53.042),
        "ways": [1208588289, 517085040, 1208588288, 240643724, 1208588290],
        "start_node_id": 2483650497,
        "fia_length": 1382.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 32,
        "elevation_fn": "pfi_bridge",
    },
    "zuera": {
        "name": "Circuito Internacional de Zuera",
        "description": "Ultra-fast Spanish supertrack with enormous drafting straights, Curva del Cierzo, and wide passing sweepers.",
        "bbox": (-0.818, 41.823, -0.806, 41.832),
        "ways": [490212650],
        "fia_length": 1700.0,
        "default_width": 9.0,
        "straight_width": 10.0,
        "num_waypoints": 32,
        "elevation_fn": None,
    },
    "le_mans_kart": {
        "name": "Le Mans Karting International",
        "description": "Alain Prost circuit at the Le Mans 24 Hours complex with Dunlop chicane, Bugatti Esses, and Courbe des 24H.",
        "bbox": (0.208, 47.936, 0.220, 47.943),
        "ways": [482284812],
        "fia_length": 1384.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 30,
        "elevation_fn": None,
        "reverse": True,
    },
    "portimao_kart": {
        "name": "Kartodromo Internacional do Algarve",
        "description": "Undulating Portuguese rollercoaster circuit with dramatic elevation drops, sweeping downhill turns, and Curva do Sol.",
        "bbox": (-8.639, 37.230, -8.631, 37.236),
        "ways": [363049742, 836390616],
        "fia_length": 1531.0,
        "default_width": 8.5,
        "straight_width": 9.5,
        "num_waypoints": 32,
        "elevation_fn": None,
        "reverse": True,
    },
    "franciacorta": {
        "name": "Franciacorta Karting Track",
        "description": "Modern premier Italian world championship venue with technical switchback chicanes and trail-braking hairpins.",
        "bbox": (9.998, 45.508, 10.012, 45.518),
        "ways": [1474753177],
        "fia_length": 1300.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 30,
        "elevation_fn": None,
    },
}


def query_osm_bbox(name, bbox):
    cache_file = os.path.join(CACHE_DIR, f"{name}.json")
    if os.path.exists(cache_file):
        with open(cache_file, "r") as f:
            return json.load(f)

    min_lon, min_lat, max_lon, max_lat = bbox
    map_url = f"https://api.openstreetmap.org/api/0.6/map?bbox={min_lon},{min_lat},{max_lon},{max_lat}"
    try:
        req = urllib.request.Request(map_url, headers={"User-Agent": "tdrace-osm-tool/1.0"})
        with urllib.request.urlopen(req, timeout=15) as resp:
            tree = ET.fromstring(resp.read().decode("utf-8"))
        elements = []
        for n in tree.findall("node"):
            tags = {t.get("k"): t.get("v") for t in n.findall("tag")}
            elements.append({
                "type": "node",
                "id": int(n.get("id")),
                "lat": float(n.get("lat")),
                "lon": float(n.get("lon")),
                "tags": tags,
            })
        for w in tree.findall("way"):
            tags = {t.get("k"): t.get("v") for t in w.findall("tag")}
            nodes = [int(nd.get("ref")) for nd in w.findall("nd")]
            elements.append({
                "type": "way",
                "id": int(w.get("id")),
                "nodes": nodes,
                "tags": tags,
            })
        data = {"elements": elements}
        with open(cache_file, "w") as f:
            json.dump(data, f)
        return data
    except Exception as e:
        print(f"Direct OSM API failed for {name}: {e}, trying Overpass...")

    query = f"""
[out:json][timeout:25];
(
  way["highway"="raceway"]({min_lat},{min_lon},{max_lat},{max_lon});
);
out body;
>;
out skel qt;
"""
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
    raise RuntimeError(f"Failed to query OSM for {name}")


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


def resample_polyline(points, target_count):
    n = len(points)
    cum_dists = [0.0]
    for i in range(n):
        p0 = points[i]
        p1 = points[(i + 1) % n]
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

        p0 = points[seg_idx]
        p1 = points[(seg_idx + 1) % n]
        x = p0[0] + t * (p1[0] - p0[0])
        y = p0[1] + t * (p1[1] - p0[1])
        resampled.append((x, y))

    return resampled, total_len


def find_longest_straight(points):
    """
    Finds the index and heading of the longest straight section to orient as start/finish straight.
    Returns (start_idx, heading_vector).
    """
    n = len(points)
    best_len = 0.0
    best_start_idx = 0
    best_heading = 0.0

    for i in range(n):
        arc_len = 0.0
        for span in range(1, n // 2):
            pa = points[(i + span - 1) % n]
            pb = points[(i + span) % n]
            arc_len += math.hypot(pb[0] - pa[0], pb[1] - pa[1])
            if arc_len > 400.0:
                break
            if arc_len >= 35.0:
                direct_dist = math.hypot(pb[0] - points[i][0], pb[1] - points[i][1])
                if direct_dist / arc_len > 0.980:
                    if direct_dist > best_len:
                        best_len = direct_dist
                        best_start_idx = i
                        best_heading = math.atan2(pb[1] - points[i][1], pb[0] - points[i][0])

    return best_start_idx, best_heading


def process_kart_track(track_id):
    spec = KART_TRACK_SPECS[track_id]
    data = query_osm_bbox(track_id, spec["bbox"])

    nodes = {e["id"]: (e["lat"], e["lon"]) for e in data["elements"] if e["type"] == "node"}
    ways = {e["id"]: e for e in data["elements"] if e["type"] == "way"}

    raw_node_ids = []
    for wid in spec["ways"]:
        w = ways[wid]
        w_nodes = w["nodes"]
        if not raw_node_ids:
            raw_node_ids.extend(w_nodes[:-1] if w_nodes[0] == w_nodes[-1] else w_nodes)
        else:
            if raw_node_ids[-1] == w_nodes[0]:
                raw_node_ids.extend(w_nodes[1:-1] if w_nodes[0] == w_nodes[-1] else w_nodes[1:])
            else:
                raw_node_ids.extend(w_nodes)

    raw_pts = [nodes[nid] for nid in raw_node_ids]
    if spec.get("reverse", False):
        raw_pts = list(reversed(raw_pts))
    lat0 = sum(p[0] for p in raw_pts) / len(raw_pts)
    lon0 = sum(p[1] for p in raw_pts) / len(raw_pts)

    metric_pts = [latlon_to_meters(p[0], p[1], lat0, lon0) for p in raw_pts]

    # If start_node_id specified, use it; otherwise find longest straight
    if "start_node_id" in spec:
        start_idx = raw_node_ids.index(spec["start_node_id"])
        reordered_pts = metric_pts[start_idx:] + metric_pts[:start_idx]
        p0 = reordered_pts[0]
        p_ahead = reordered_pts[1]
        for k in range(1, len(reordered_pts)):
            d = math.hypot(reordered_pts[k][0] - p0[0], reordered_pts[k][1] - p0[1])
            if d >= 60.0:
                p_ahead = reordered_pts[k]
                break
            p_ahead = reordered_pts[k]
        heading = math.atan2(p_ahead[1] - p0[1], p_ahead[0] - p0[0])
    else:
        start_idx, heading = find_longest_straight(metric_pts)
        reordered_pts = metric_pts[start_idx:] + metric_pts[:start_idx]

    # Rotate so start straight heads along +X
    cos_a = math.cos(-heading)
    sin_a = math.sin(-heading)
    rotated_pts = []
    for x, y in reordered_pts:
        rx = x * cos_a - y * sin_a
        ry = x * sin_a + y * cos_a
        rotated_pts.append((rx, ry))

    # Scale to exact FIA homologation length
    current_len = polyline_length(rotated_pts, closed=True)
    scale_factor = spec["fia_length"] / current_len if current_len > 0 else 1.0

    scaled_pts = [
        ((x * scale_factor), (y * scale_factor))
        for x, y in rotated_pts
    ]

    # Translate so start point is at x=0, y=0
    x_offset = scaled_pts[0][0]
    y_offset = scaled_pts[0][1]
    aligned_pts = [
        ((x - x_offset), (y - y_offset))
        for x, y in scaled_pts
    ]

    # Uniform arc-length resampling
    resampled, final_len = resample_polyline(aligned_pts, spec["num_waypoints"])

    # Compute curvature, assign widths, and inside apex curbs
    n = len(resampled)
    waypoints = []
    for i in range(n):
        x, y = resampled[i]
        p_prev = resampled[(i - 1 + n) % n]
        p_next = resampled[(i + 1) % n]

        v1 = (x - p_prev[0], y - p_prev[1])
        v2 = (p_next[0] - x, p_next[1] - y)
        len1 = math.hypot(v1[0], v1[1])
        len2 = math.hypot(v2[0], v2[1])
        if len1 > 1e-4 and len2 > 1e-4:
            norm_cross = (v1[0] * v2[1] - v1[1] * v2[0]) / (len1 * len2)
        else:
            norm_cross = 0.0

        # Straight width on start straight
        is_straight = abs(norm_cross) < 0.15 and (i < 4 or i >= n - 2)
        width = spec["straight_width"] if is_straight else spec["default_width"]

        # Hairpin clearance
        if abs(norm_cross) > 0.65:
            width = min(width, 8.0)

        # Curbs: inside corner apexes, never on start straight or waypoint 0
        left_curb = False
        right_curb = False
        if not is_straight and i != 0:
            if norm_cross > 0.30:  # Left turn -> inside curb on left side
                left_curb = True
            elif norm_cross < -0.30:  # Right turn -> inside curb on right side
                right_curb = True

        waypoints.append({
            "x": round(x, 1),
            "y": round(y, 1),
            "width": round(width, 1),
            "left_curb": left_curb,
            "right_curb": right_curb,
            "elevation": 0.0,
        })

    # Specific elevation and corridor clearance for PFI
    if spec["elevation_fn"] == "pfi_bridge":
        # Bridge spans crossover between waypoints 8..11
        waypoints[8]["elevation"] = 2.2
        waypoints[9]["elevation"] = 4.2
        waypoints[10]["elevation"] = 4.2
        waypoints[11]["elevation"] = 2.2

        # Ensure corridor clearance at the far west hairpin turnaround to avoid wall intersection
        waypoints[24]["y"] = round(waypoints[24]["y"] - 3.5, 1)
        waypoints[25]["y"] = round(waypoints[25]["y"] - 3.5, 1)
        waypoints[28]["y"] = round(waypoints[28]["y"] + 3.0, 1)
        waypoints[29]["y"] = round(waypoints[29]["y"] + 3.0, 1)

    return {
        "id": track_id,
        "name": spec["name"],
        "description": spec["description"],
        "waypoints": waypoints,
        "total_length": round(final_len, 1),
        "fia_length": spec["fia_length"],
    }


def generate_rust_code(track_data):
    wps = track_data["waypoints"]
    lines = []
    lines.append("        let waypoints = vec![")
    for i, w in enumerate(wps):
        curb_str = ""
        if w["left_curb"] or w["right_curb"]:
            curb_str = f".with_curbs({str(w['left_curb']).lower()}, {str(w['right_curb']).lower()})"
        elev_str = ""
        if w.get("elevation", 0.0) > 0.0:
            elev_str = f".with_elevation({w['elevation']:.1f})"
        lines.append(
            f"            TrackWaypoint::new(Vec2::new({w['x']:.1f}, {w['y']:.1f}), {w['width']:.1f}){elev_str}{curb_str},"
        )
    lines.append("        ];")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Extract and generate tdrace kart tracks from OSM")
    parser.add_argument("--track", choices=list(KART_TRACK_SPECS.keys()), help="Process specific kart track")
    parser.add_argument("--all", action="store_true", help="Process all kart tracks")
    parser.add_argument("--rust", action="store_true", help="Print Rust waypoint code")
    args = parser.parse_args()

    tracks_to_process = list(KART_TRACK_SPECS.keys()) if args.all or not args.track else [args.track]
    for tid in tracks_to_process:
        res = process_kart_track(tid)
        print(f"=== {res['name']} ({res['id']}) ===")
        print(f"  Waypoints: {len(res['waypoints'])}, Total Length: {res['total_length']} m (FIA Target: {res['fia_length']} m)")
        if args.rust:
            print(generate_rust_code(res))
            print()


if __name__ == "__main__":
    main()
