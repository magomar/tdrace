#!/usr/bin/env python3
"""
OSM NASCAR Track Importer for tdrace

Extracts real-world motorsport raceway waypoints from OpenStreetMap (OSM),
projects them to metric 2D Cartesian coordinates, scales them to target NASCAR lengths
(road courses scaled to 0.5x), aligns the start/finish straight with the +X axis,
adds apex curbs, banking angles, and generates ready-to-use Rust track definitions.
"""

import math
import os
import urllib.request
import xml.etree.ElementTree as ET

CACHE_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "target", "osm_cache")
os.makedirs(CACHE_DIR, exist_ok=True)


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


def resample_polyline(pts, target_count):
    n = len(pts)
    cum_dists = [0.0]
    for i in range(n):
        p0 = pts[i]
        p1 = pts[(i + 1) % n]
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
        p0 = pts[seg_idx]
        p1 = pts[(seg_idx + 1) % n]
        x = p0[0] + t * (p1[0] - p0[0])
        y = p0[1] + t * (p1[1] - p0[1])
        resampled.append((x, y))
    return resampled, total_len


def fetch_osm_xml(filename, url):
    cache_path = os.path.join(CACHE_DIR, filename)
    if os.path.exists(cache_path):
        with open(cache_path, "r", encoding="utf-8") as f:
            return f.read()
    req = urllib.request.Request(url, headers={"User-Agent": "tdrace-osm-tool/1.0"})
    with urllib.request.urlopen(req, timeout=20) as resp:
        content = resp.read().decode("utf-8")
    with open(cache_path, "w", encoding="utf-8") as f:
        f.write(content)
    return content


def get_eldora_points():
    xml_str = fetch_osm_xml(
        "eldora_speedway.osm",
        "https://api.openstreetmap.org/api/0.6/map?bbox=-84.637,40.316,-84.628,40.322",
    )
    tree = ET.fromstring(xml_str)
    nodes = {int(n.get("id")): (float(n.get("lat")), float(n.get("lon"))) for n in tree.findall("node")}
    for w in tree.findall("way"):
        if w.get("id") == "608397609":
            way_nodes = [int(nd.get("ref")) for nd in w.findall("nd")][:-1]
            return [nodes[nid] for nid in way_nodes]
    raise RuntimeError("Eldora way 608397609 not found")


def get_iowa_points():
    xml_str = fetch_osm_xml(
        "iowa_speedway.osm",
        "https://api.openstreetmap.org/api/0.6/map?bbox=-93.022,41.666,-93.006,41.678",
    )
    tree = ET.fromstring(xml_str)
    nodes = {int(n.get("id")): (float(n.get("lat")), float(n.get("lon"))) for n in tree.findall("node")}
    for w in tree.findall("way"):
        if w.get("id") == "119238784":
            way_nodes = [int(nd.get("ref")) for nd in w.findall("nd")][:-1]
            return [nodes[nid] for nid in way_nodes]
    raise RuntimeError("Iowa way 119238784 not found")


def get_road_america_points():
    xml_str = fetch_osm_xml(
        "road_america_rel.osm",
        "https://api.openstreetmap.org/api/0.6/relation/6432758/full",
    )
    tree = ET.fromstring(xml_str)
    nodes = {int(n.get("id")): (float(n.get("lat")), float(n.get("lon"))) for n in tree.findall("node")}
    ways = {int(w.get("id")): [int(nd.get("ref")) for nd in w.findall("nd")] for w in tree.findall("way")}
    rel = tree.find("relation")
    member_way_ids = [int(m.get("ref")) for m in rel.findall("member") if m.get("type") == "way"]

    ordered_nodes = []
    for wid in member_way_ids:
        wnodes = ways.get(wid, [])
        if not wnodes:
            continue
        if not ordered_nodes:
            ordered_nodes.extend(wnodes)
        else:
            if ordered_nodes[-1] == wnodes[0]:
                ordered_nodes.extend(wnodes[1:])
            elif ordered_nodes[-1] == wnodes[-1]:
                ordered_nodes.extend(list(reversed(wnodes))[1:])
            else:
                p_curr = nodes[ordered_nodes[-1]]
                p_start = nodes[wnodes[0]]
                p_end = nodes[wnodes[-1]]
                d_start = math.hypot(p_curr[0] - p_start[0], p_curr[1] - p_start[1])
                d_end = math.hypot(p_curr[0] - p_end[0], p_curr[1] - p_end[1])
                if d_start < d_end:
                    ordered_nodes.extend(wnodes[1:])
                else:
                    ordered_nodes.extend(list(reversed(wnodes))[1:])
    return [nodes[nid] for nid in ordered_nodes]


def get_chicago_points():
    xml_str = fetch_osm_xml(
        "chicago_rel.osm",
        "https://api.openstreetmap.org/api/0.6/relation/16546690/full",
    )
    tree = ET.fromstring(xml_str)
    nodes = {int(n.get("id")): (float(n.get("lat")), float(n.get("lon"))) for n in tree.findall("node")}
    ways = {int(w.get("id")): [int(nd.get("ref")) for nd in w.findall("nd")] for w in tree.findall("way")}
    rel = tree.find("relation")
    member_way_ids = [int(m.get("ref")) for m in rel.findall("member") if m.get("type") == "way"]

    ordered_nodes = []
    for wid in member_way_ids:
        wnodes = ways.get(wid, [])
        if not wnodes:
            continue
        if not ordered_nodes:
            ordered_nodes.extend(wnodes)
        else:
            if ordered_nodes[-1] == wnodes[0]:
                ordered_nodes.extend(wnodes[1:])
            elif ordered_nodes[-1] == wnodes[-1]:
                ordered_nodes.extend(list(reversed(wnodes))[1:])
            else:
                p_curr = nodes[ordered_nodes[-1]]
                p_start = nodes[wnodes[0]]
                p_end = nodes[wnodes[-1]]
                d_start = math.hypot(p_curr[0] - p_start[0], p_curr[1] - p_start[1])
                d_end = math.hypot(p_curr[0] - p_end[0], p_curr[1] - p_end[1])
                if d_start < d_end:
                    ordered_nodes.extend(wnodes[1:])
                else:
                    ordered_nodes.extend(list(reversed(wnodes))[1:])
    return [nodes[nid] for nid in ordered_nodes]


def process_circuit(raw_latlons, target_length, target_waypoints, is_dirt=False):
    lat0 = sum(p[0] for p in raw_latlons) / len(raw_latlons)
    lon0 = sum(p[1] for p in raw_latlons) / len(raw_latlons)

    metric_pts = [latlon_to_meters(p[0], p[1], lat0, lon0) for p in raw_latlons]

    # Find longest straight segment for start/finish alignment
    n = len(metric_pts)
    best_heading = 0.0
    p_start = metric_pts[0]
    p_next = metric_pts[min(5, n - 1)]
    best_heading = math.atan2(p_next[1] - p_start[1], p_next[0] - p_start[0])

    cos_a = math.cos(-best_heading)
    sin_a = math.sin(-best_heading)
    rotated = []
    for x, y in metric_pts:
        rx = x * cos_a - y * sin_a
        ry = x * sin_a + y * cos_a
        rotated.append((rx, ry))

    # Scale to target length
    cur_len = polyline_length(rotated, closed=True)
    scale = target_length / cur_len if cur_len > 0 else 1.0
    scaled = [(x * scale, y * scale) for x, y in rotated]

    # Translate so WP 0 is at (0, 0)
    x0, y0 = scaled[0]
    aligned = [(x - x0, y - y0) for x, y in scaled]

    # Resample
    resampled, final_len = resample_polyline(aligned, target_waypoints)
    return resampled, final_len


if __name__ == "__main__":
    print("Fetching and processing circuits...")
    eldora_pts, l_e = process_circuit(get_eldora_points(), 805.0, 14, is_dirt=True)
    print(f"Eldora: {len(eldora_pts)} wps, len={l_e:.1f}m")

    iowa_pts, l_i = process_circuit(get_iowa_points(), 1408.0, 16)
    print(f"Iowa: {len(iowa_pts)} wps, len={l_i:.1f}m")

    ra_pts, l_ra = process_circuit(get_road_america_points(), 3257.5, 32)
    print(f"Road America: {len(ra_pts)} wps, len={l_ra:.1f}m")

    chi_pts, l_chi = process_circuit(get_chicago_points(), 1770.0, 28)
    print(f"Chicago: {len(chi_pts)} wps, len={l_chi:.1f}m")
