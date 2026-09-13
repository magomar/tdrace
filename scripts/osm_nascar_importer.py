#!/usr/bin/env python3
"""
OSM NASCAR Track Importer for tdrace

Extracts real-world NASCAR raceway waypoints from OpenStreetMap (OSM) via direct OSM API,
projects them to metric 2D Cartesian coordinates, aligns the start/finish straight with
the +X axis, scales them to 0.5x official length (standard for tdrace Grand Prix & superspeedways),
assigns authentic turn banking, and outputs ready-to-use Rust track definitions for
crates/arcade-race-core/src/track/presets.rs.
"""

import math
import os
import urllib.request
import xml.etree.ElementTree as ET

CACHE_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "target", "osm_cache")
os.makedirs(CACHE_DIR, exist_ok=True)

IMS_BBOX = (-86.242, 39.785, -86.228, 39.807)

# 10 contiguous ways in counter-clockwise order forming the 2.500-mile rectangular oval:
# Start/finish line at Yard of Bricks is node 654509391 on the Main Straight.
IMS_WAY_SEQUENCE = [
    "588780349",  # Main Straight: from Yard of Bricks south to Turn 1 entry
    "51308226",   # Turn 1: 90° left turn, 9° 12' banking
    "588780333",  # South Straight (South Short Chute)
    "588780334",  # Turn 2: 90° left turn, 9° 12' banking
    "588780335",  # Backstretch Part 1
    "588780332",  # Backstretch Part 2
    "588780343",  # Turn 3: 90° left turn, 9° 12' banking
    "588780345",  # North Straight (North Short Chute)
    "588780347",  # Turn 4: 90° left turn, 9° 12' banking
    "588780351",  # Main Straight: Turn 4 exit back to Yard of Bricks
]

TURN_WAYS = {"51308226", "588780334", "588780343", "588780347"}


def fetch_or_load_osm(cache_path: str, bbox: tuple) -> bytes:
    if os.path.exists(cache_path) and os.path.getsize(cache_path) > 0:
        with open(cache_path, "rb") as f:
            return f.read()

    min_lon, min_lat, max_lon, max_lat = bbox
    url = f"https://api.openstreetmap.org/api/0.6/map?bbox={min_lon},{min_lat},{max_lon},{max_lat}"
    req = urllib.request.Request(url, headers={"User-Agent": "tdrace-osm-tool/1.0"})
    with urllib.request.urlopen(req) as resp:
        content = resp.read()

    with open(cache_path, "wb") as f:
        f.write(content)
    return content


def build_indianapolis_waypoints(num_waypoints: int = 24):
    osm_path = os.path.join(CACHE_DIR, "indianapolis.osm")
    content = fetch_or_load_osm(osm_path, IMS_BBOX)
    root = ET.fromstring(content)

    nodes = {nd.get("id"): (float(nd.get("lat")), float(nd.get("lon"))) for nd in root.findall("node")}
    ways = {w.get("id"): w for w in root.findall("way")}

    node_seq = []
    way_id_per_node = []
    for wid in IMS_WAY_SEQUENCE:
        w = ways[wid]
        nds = [nd.get("ref") for nd in w.findall("nd")]
        if node_seq:
            assert node_seq[-1] == nds[0], f"Topology break between {node_seq[-1]} and {nds[0]}"
            node_seq.extend(nds[1:])
            way_id_per_node.extend([wid] * (len(nds) - 1))
        else:
            node_seq.extend(nds)
            way_id_per_node.extend([wid] * len(nds))

    assert node_seq[0] == node_seq[-1], "Loop must close back to start node"

    pts_latlon = [nodes[nid] for nid in node_seq]
    lat0, lon0 = pts_latlon[0]
    r = 6378137.0  # WGS84 Earth radius

    # Metric equirectangular tangent projection
    pts_m = []
    for lat, lon in pts_latlon:
        x = (math.radians(lon) - math.radians(lon0)) * math.cos(math.radians(lat0)) * r
        y = (math.radians(lat) - math.radians(lat0)) * r
        pts_m.append((x, y))

    # Align frontstretch heading along +X
    dx0 = pts_m[3][0] - pts_m[0][0]
    dy0 = pts_m[3][1] - pts_m[0][1]
    heading = math.atan2(dy0, dx0)
    rot = -heading
    cos_r = math.cos(rot)
    sin_r = math.sin(rot)

    pts_rot = []
    for x, y in pts_m:
        xr = x * cos_r - y * sin_r
        yr = x * sin_r + y * cos_r
        pts_rot.append((xr, yr))

    # Calculate raw perimeter and scale to 0.5x FIA length (2011.68m)
    total_len = 0.0
    seg_lens = []
    for i in range(len(pts_rot) - 1):
        d = math.hypot(pts_rot[i + 1][0] - pts_rot[i][0], pts_rot[i + 1][1] - pts_rot[i][1])
        seg_lens.append(d)
        total_len += d

    target_len = 2011.68
    scale = target_len / total_len

    # Center Y coordinate so frontstretch is at Y = -180.0 (matching NASCAR conventions)
    pts_scaled = [(x * scale, (y * scale) - 180.0) for x, y in pts_rot]

    cum_dist = [0.0]
    for d in seg_lens:
        cum_dist.append(cum_dist[-1] + d * scale)

    # Uniform resampling along cumulative arc length
    step = target_len / num_waypoints
    waypoints = []
    for i in range(num_waypoints):
        target_d = i * step
        idx = 0
        while idx < len(cum_dist) - 2 and cum_dist[idx + 1] < target_d:
            idx += 1
        t = (target_d - cum_dist[idx]) / (cum_dist[idx + 1] - cum_dist[idx])
        p1 = pts_scaled[idx]
        p2 = pts_scaled[idx + 1]
        rx = p1[0] + t * (p2[0] - p1[0])
        ry = p1[1] + t * (p2[1] - p1[1])
        wid = way_id_per_node[idx]

        # Authentic banking: 9.2° (9° 12') on turns, 0.0° on straights and short chutes
        # Short chutes (South Straight around WP 05, North Straight around WP 17) are flat
        in_turn = wid in TURN_WAYS and not (wid == "588780343" and ry < 30.0 and ry > -60.0)
        bank = 9.2 if in_turn else 0.0
        waypoints.append((round(rx, 1), round(ry, 1), bank, wid))

    return waypoints


if __name__ == "__main__":
    wps = build_indianapolis_waypoints(24)
    print(f"Generated {len(wps)} waypoints for Indianapolis Motor Speedway:")
    for i, (x, y, bank, wid) in enumerate(wps):
        print(f"    TrackWaypoint::new(Vec2::new({x:6.1f}, {y:6.1f}), 20.0).with_bank_angle({bank:.1f}), // WP {i:02d}")
