#!/usr/bin/env python3
"""
OSM Circuit Importer for tdrace

One importer for the OpenStreetMap (OSM) circuits of three disciplines:

  gt     18 GT / F1 circuits, scaled to 0.5x the official FIA length
         (Rust output: a full `track_<id>()` for crates/tdrace-app/src/module/gt.rs)
  kart   CIK-FIA kart circuits at 1:1 (waypoint vec for crates/tdrace-app/src/module/kart.rs)
  rally  World RX rallycross circuits at 1:1 with asphalt/dirt surfaces
         (waypoint vec for crates/arcade-race-core/src/track/presets.rs)

Every circuit goes through the same steps: select the raceway nodes, project them to
metres, rotate the start straight onto +X, scale to the official length, resample to
evenly spaced waypoints, then assign widths and inside-apex kerbs from the deflection
angle at each waypoint.

Map data is read from <cache_dir>/<track_id>.osm (default cache dir: <repo>/assets/osm).
`download` fetches it for the real circuits listed in crates/arcade-race-core/src/track/provenance.rs:
the OSM map API area around the circuit's osm_url element (plus the kart/rally config bbox) with a
300 m margin, or, when that area is too large, an Overpass extract of everything near the raceways.

Warnings on stderr flag suspect geometry: a join between two nodes that are not an OSM
edge and are more than 45 m apart, a way that could not be stitched, and a raw OSM loop
length more than 10% away from the official length.

Usage:
  python3 scripts/osm_importer.py download --track monza
  python3 scripts/osm_importer.py gt --track monza --rust
  python3 scripts/osm_importer.py gt --track monza --json   # then run the printed track_bake command
  python3 scripts/osm_importer.py rally
  python3 scripts/osm_importer.py kart --cache-dir /path/to/osm_cache

Map data (c) OpenStreetMap contributors, available under the Open Database Licence (ODbL).
"""

import argparse
import json
import math
import os
import re
import sys
import time
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_CACHE_DIR = os.path.join(REPO_ROOT, "assets", "osm")
USER_AGENT = "tdrace-osm-tool/1.0"

# Joins between consecutive chain nodes that are not OSM edges must be closer than this.
MAX_JOIN_GAP_M = 45.0
# Raw OSM loop length must be within this fraction of the official length.
LENGTH_RATIO_TOLERANCE = 0.10


# ---------------------------------------------------------------------------
# Shared helpers
# ---------------------------------------------------------------------------


def warn(label, message):
    print(f"WARNING [{label}]: {message}", file=sys.stderr)


def latlon_to_meters(lat, lon, lat0, lon0):
    r = 6378137.0
    x = (math.radians(lon) - math.radians(lon0)) * math.cos(math.radians(lat0)) * r
    y = (math.radians(lat) - math.radians(lat0)) * r
    return x, y


def geo_dist_m(p1, p2):
    """Distance in meters between two (lat, lon) points (local tangent plane, fine at circuit scale)."""
    x, y = latlon_to_meters(p2[0], p2[1], p1[0], p1[1])
    return math.hypot(x, y)


def polyline_length(pts, closed=True):
    total = 0.0
    n = len(pts)
    for i in range(n if closed else n - 1):
        p0 = pts[i]
        p1 = pts[(i + 1) % n]
        total += math.hypot(p1[0] - p0[0], p1[1] - p0[1])
    return total


def rotate_points(points, heading):
    """Rotate points by -heading so that `heading` maps onto the +X axis."""
    cos_a = math.cos(-heading)
    sin_a = math.sin(-heading)
    return [(x * cos_a - y * sin_a, x * sin_a + y * cos_a) for x, y in points]


def resample_polyline(points, target_count, props=None):
    """Resample a closed polyline to `target_count` points at equal arc-length steps.

    `props` (optional) holds one value per input point; each output point takes the value
    of the input segment it falls on. Returns (points, props or None, total_length).
    """
    n = len(points)
    cum_dists = [0.0]
    for i in range(n):
        p0 = points[i]
        p1 = points[(i + 1) % n]
        cum_dists.append(cum_dists[-1] + math.hypot(p1[0] - p0[0], p1[1] - p0[1]))

    total_len = cum_dists[-1]
    step = total_len / target_count
    resampled = []
    resampled_props = [] if props is not None else None
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
        resampled.append((p0[0] + t * (p1[0] - p0[0]), p0[1] + t * (p1[1] - p0[1])))
        if props is not None:
            resampled_props.append(props[seg_idx])

    return resampled, resampled_props, total_len


def deflection_sine(points, i):
    """Sine of the turn angle at closed-loop point `i` (>0 left, <0 right), independent of spacing and scale."""
    n = len(points)
    x, y = points[i]
    p_prev = points[(i - 1 + n) % n]
    p_next = points[(i + 1) % n]
    v1 = (x - p_prev[0], y - p_prev[1])
    v2 = (p_next[0] - x, p_next[1] - y)
    len1 = math.hypot(v1[0], v1[1])
    len2 = math.hypot(v2[0], v2[1])
    if len1 > 1e-6 and len2 > 1e-6:
        return (v1[0] * v2[1] - v1[1] * v2[0]) / (len1 * len2)
    return 0.0


def way_edges(node_lists):
    """Set of undirected node-id pairs that are consecutive in at least one OSM way."""
    edges = set()
    for nds in node_lists:
        for a, b in zip(nds, nds[1:]):  # noqa: RUF007 - itertools.pairwise needs 3.10; system python3 is 3.9
            edges.add(frozenset((a, b)))
    return edges


def check_loop_joins(label, node_ids, coords, edges, max_gap=MAX_JOIN_GAP_M):
    """Warn for every consecutive pair of the closed chain (including last -> first) that is
    neither the same node nor an OSM edge and lies more than `max_gap` meters apart."""
    n = len(node_ids)
    bad = 0
    for i in range(n):
        a = node_ids[i]
        b = node_ids[(i + 1) % n]
        if a == b or frozenset((a, b)) in edges:
            continue
        gap = geo_dist_m(coords[a], coords[b])
        if gap > max_gap:
            bad += 1
            where = "loop closure" if i == n - 1 else f"chain index {i}"
            warn(label, f"{gap:.1f} m jump between nodes {a} and {b} ({where}) is not an OSM edge")
    return bad


def check_length_ratio(label, raw_len, official_len, tolerance=LENGTH_RATIO_TOLERANCE):
    """Warn when the raw OSM loop is much longer or shorter than the official length (missing or doubled sections)."""
    if official_len <= 0:
        return
    ratio = raw_len / official_len
    if abs(ratio - 1.0) > tolerance:
        warn(label, f"raw OSM loop is {raw_len:.1f} m, {ratio:.3f}x the official {official_len:.1f} m")


def map_api_to_elements(xml_text):
    """Convert OSM map API XML into Overpass-style JSON elements (nodes and ways)."""
    tree = ET.fromstring(xml_text)
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
    return {"elements": elements}


def osm_file_path(cache_dir, track_id):
    return os.path.join(cache_dir, f"{track_id}.osm")


def load_osm_elements(cache_dir, track_id):
    """Nodes and ways of <cache_dir>/<track_id>.osm as Overpass-style JSON elements."""
    path = osm_file_path(cache_dir, track_id)
    if not os.path.exists(path):
        raise SystemExit(f"Missing {path}: run `python3 scripts/osm_importer.py download --track {track_id}` first")
    with open(path, "r", encoding="utf-8") as f:
        return map_api_to_elements(f.read())


def heading_ahead(points, min_dist=60.0):
    """Heading from points[0] to the first point at least `min_dist` meters away (or the last point)."""
    p0 = points[0]
    p_ahead = points[1]
    for p in points[1:]:
        p_ahead = p
        if math.hypot(p[0] - p0[0], p[1] - p0[1]) >= min_dist:
            break
    return math.atan2(p_ahead[1] - p0[1], p_ahead[0] - p0[0])


def chain_segments(ways, segments):
    """Node ids along (way id, first node or None) segments; each segment runs to its way's last node.

    Consecutive segments share their join node, which is kept once; a closing repeat of the first
    node is dropped. Returns [(node id, way id)].
    """
    chain = []
    for wid, first in segments:
        nds = ways[wid]["nodes"]
        nds = nds[nds.index(first):] if first is not None else nds
        if chain and chain[-1][0] == nds[0]:
            nds = nds[1:]
        chain.extend((nid, wid) for nid in nds)
    if len(chain) > 1 and chain[-1][0] == chain[0][0]:
        chain.pop()
    return chain


def stitch_ways(label, way_nodes_list, nodes_coords, max_gap=MAX_JOIN_GAP_M):
    """Greedily chain ways by nearest endpoint, reversing ways as needed.

    `nodes_coords` maps node id -> (lat, lon); gaps are in meters. Stops with a warning when
    the nearest remaining way is more than `max_gap` away.
    """
    if not way_nodes_list:
        return []
    remaining = [list(w) for w in way_nodes_list]
    chain = remaining.pop(0)

    while remaining:
        curr_end = chain[-1]
        curr_end_pt = nodes_coords[curr_end]

        best_idx = None
        best_rev = False
        min_d = float("inf")

        for idx, w in enumerate(remaining):
            d_start = geo_dist_m(curr_end_pt, nodes_coords[w[0]])
            d_end = geo_dist_m(curr_end_pt, nodes_coords[w[-1]])
            if d_start < min_d:
                min_d = d_start
                best_idx = idx
                best_rev = False
            if d_end < min_d:
                min_d = d_end
                best_idx = idx
                best_rev = True

        if min_d > max_gap:
            warn(label, f"stitching stopped: nearest way is {min_d:.1f} m away (max {max_gap:.1f} m); "
                        f"{len(remaining)} way(s) left unjoined")
            break

        next_w = remaining.pop(best_idx)
        if best_rev:
            next_w.reverse()

        if next_w[0] == curr_end:
            chain.extend(next_w[1:])
        else:
            chain.extend(next_w)

    return chain


# ---------------------------------------------------------------------------
# GT / F1 circuits (0.5x FIA length)
# ---------------------------------------------------------------------------

GT_CIRCUITS = {
    "monza": {
        "name": "Monza Autodromo Nazionale",
        "description": "High-speed Italian Grand Prix temple of speed.",
        "rel_id": 284565,
        "fia_length": 5793.0,
        "num_waypoints": 28,
        "default_width": 13.5,
        "straight_width": 15.0,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 4.5,
        "default_laps": 3,
        "tag": "TEMPLE OF SPEED",
    },
    "spa": {
        "name": "Circuit de Spa-Francorchamps",
        "description": "Belgian Ardennes rollercoaster featuring Eau Rouge and Pouhon.",
        "rel_id": 284560,
        "fia_length": 7004.0,
        "num_waypoints": 32,
        "default_width": 14.0,
        "straight_width": 15.0,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 4.5,
        "default_laps": 3,
        "tag": "ARDENNES ROLLERCOASTER",
        "elevations": {
            # Eau Rouge & Raidillon uphill climb
            3: 2.0, 4: 4.5, 5: 3.0,
        },
    },
    "silverstone": {
        "name": "Silverstone Grand Prix Circuit",
        "description": "High-speed sweeping esses through Maggotts, Becketts and Chapel.",
        "rel_id": 51160,
        "fia_length": 5891.0,
        "num_waypoints": 30,
        "default_width": 13.5,
        "straight_width": 14.5,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 4.0,
        "default_laps": 3,
        "tag": "HOME OF BRITISH MOTORSPORT",
    },
    "monaco": {
        "name": "Circuit de Monaco",
        "description": "Legendary Monte Carlo street circuit with Loews Hairpin, Tunnel, and Swimming Pool.",
        "rel_id": 148194,
        "fia_length": 3337.0,
        "num_waypoints": 26,
        "default_width": 10.5,
        "straight_width": 11.5,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 3.0,
        "default_laps": 3,
        "tag": "JEWEL IN THE CROWN",
        "monaco_filter": True,
        "start_way": "166399479",
        "elevations": {
            # Beau Rivage uphill
            2: 2.0, 3: 3.5,
            # Casino Square crest
            4: 4.0, 5: 3.0,
            # Loews Hairpin descent
            6: 2.0, 7: 1.0,
        },
    },
    "suzuka": {
        "name": "Suzuka International Racing Course",
        "description": "Iconic Japanese figure-8 layout featuring Esses, Degner, overpass crossover bridge, and 130R.",
        "rel_id": 284570,
        "fia_length": 5807.0,
        "num_waypoints": 34,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 3.8,
        "default_laps": 3,
        "tag": "JAPANESE FIGURE-8",
        "crossover": True,
    },
    "interlagos": {
        "name": "Autodromo Jose Carlos Pace (Interlagos)",
        "description": "Thrilling anti-clockwise Brazilian Grand Prix circuit with Senna 'S', Ferradura, and Juncao.",
        "rel_id": 6781071,
        "fia_length": 4309.0,
        "num_waypoints": 28,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 3.5,
        "default_laps": 3,
        "tag": "BRAZILIAN ROLLERCOASTER",
    },
    "montreal": {
        "name": "Circuit Gilles Villeneuve (Montreal)",
        "description": "High-speed Canadian island circuit featuring Virage Senna, L'Epingle hairpin, and Wall of Champions.",
        "rel_id": 284595,
        "fia_length": 4361.0,
        "num_waypoints": 28,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Concrete",
        "barrier_offset": 3.0,
        "default_laps": 3,
        "tag": "ILE NOTRE-DAME",
    },
    "red_bull_ring": {
        "name": "Red Bull Ring (Spielberg)",
        "description": "High-speed Austrian alpine circuit with steep uphill climbs and heavy downhill braking into Remus.",
        "rel_id": 5309181,
        "fia_length": 4318.0,
        "num_waypoints": 26,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 3.5,
        "default_laps": 3,
        "tag": "AUSTRIAN ALPS",
        "rbr_filter": True,
        "elevations": {
            # Turn 1 to Turn 3 Remus uphill climb
            1: 1.5, 2: 3.5, 3: 5.0, 4: 4.0,
            # Downhill to Schlossgold
            5: 2.0, 6: 1.0,
        },
    },
    "catalunya": {
        "name": "Circuit de Barcelona-Catalunya",
        "description": "Famous Spanish GP circuit in Montmelo featuring Curva Renault, Campsa crest, and restored high-speed final sector.",
        "way_id": "831804327",
        "fia_length": 4657.0,
        "num_waypoints": 28,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 3.5,
        "default_laps": 3,
        "tag": "SPANISH GP BENCHMARK",
        "elevations": {
            # Turn 9 Campsa uphill crest
            15: 2.0, 16: 3.5, 17: 2.0,
        },
    },
    "zandvoort": {
        "name": "Circuit Zandvoort",
        "description": "Dune rollercoaster in the Netherlands featuring 18-degree banked corners at Hugenholtz and Arie Luyendyk.",
        "rel_id": 13545573,
        "fia_length": 4259.0,
        "num_waypoints": 28,
        "default_width": 12.5,
        "straight_width": 14.0,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 3.0,
        "default_laps": 3,
        "tag": "DUTCH DUNES",
        "elevations": {
            # Hugenholtz bowl
            4: 2.0, 5: 3.5, 6: 2.0,
            # Dunes crest
            10: 2.5, 11: 3.0, 12: 1.5,
            # Arie Luyendyk banked turn
            25: 1.5, 26: 2.0,
        },
    },
    "bahrain": {
        "name": "Bahrain International Circuit (Sakhir)",
        "description": "High-power desert battleground under the floodlights with heavy braking zones and abrasive tarmac.",
        "rel_id": 284538,
        "fia_length": 5412.0,
        "num_waypoints": 30,
        "default_width": 13.5,
        "straight_width": 15.0,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 3.8,
        "default_laps": 3,
        "tag": "DESERT GRAND PRIX",
    },
    "marina_bay": {
        "name": "Marina Bay Street Circuit (Singapore)",
        "description": "High-intensity Singapore night race through the dazzling city streets and harbor waterfront.",
        "rel_id": 421263,
        "fia_length": 4940.0,
        "num_waypoints": 30,
        "default_width": 11.5,
        "straight_width": 13.0,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 3.0,
        "default_laps": 3,
        "tag": "SINGAPORE NIGHT RACE",
        "marina_filter": True,
        "start_node": "4281759834",
        "predefined_car": "hypercar_prototype",
        "module_id": "gt",
        "modules": ["gt"],
    },
    "cota": {
        "name": "Circuit of the Americas (COTA)",
        "description": "Austin Texas spectacle with steep uphill Turn 1 blind crest, Maggotts-inspired Esses, and multi-apex carousel.",
        "rel_id": 6537729,
        "fia_length": 5513.0,
        "num_waypoints": 30,
        "default_width": 13.5,
        "straight_width": 15.0,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 4.0,
        "default_laps": 3,
        "tag": "AUSTIN SPECTACLE",
        "elevations": {
            # Steep Turn 1 uphill crest
            2: 2.0, 3: 4.5, 4: 2.5,
        },
    },
    "madring": {
        "name": "MadRing Circuito de Madrid",
        "description": "Spanish Grand Prix hybrid street circuit navigating the IFEMA complex and Valdebebas avenues.",
        "rel_id": 18813472,
        "fia_length": 5474.0,
        "num_waypoints": 30,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Concrete",
        "barrier_offset": 3.5,
        "default_laps": 3,
        "tag": "SPANISH STREET GP",
        "madring_filter": True,
    },
    "nurburgring_gp": {
        "name": "Nurburgring Grand Prix-Strecke",
        "description": "Challenging Eifel circuit featuring Castrol-S chicane, Mercedes Arena, and Schumacher S.",
        "rel_id": 38567,
        "fia_length": 5148.0,
        "num_waypoints": 30,
        "default_width": 13.5,
        "straight_width": 15.0,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 4.0,
        "default_laps": 3,
        "tag": "EIFEL MOTORSPORT MECCA",
        "start_node": "3099078401",
        "predefined_car": "gt4_clubsport",
        "module_id": "gt",
        "modules": ["gt"],
        "elevations": {
            # Mercedes-Arena descent & Dunlop hairpin climb
            3: -1.0, 4: -2.0, 5: -1.5,
            # Schumacher-S crest
            11: 1.5, 12: 2.5, 13: 2.0,
        },
    },
    "bathurst": {
        "name": "Mount Panorama (Bathurst)",
        "description": "The iconic Australian mountain rollercoaster: Hell Corner, Skyline, The Dipper, and Conrod Straight.",
        "rel_id": 6942508,
        "fia_length": 6213.0,
        "num_waypoints": 32,
        "default_width": 12.5,
        "straight_width": 14.5,
        "barrier": "BarrierType::Concrete",
        "barrier_offset": 3.5,
        "default_laps": 3,
        "tag": "MOUNTAIN ROLLERCOASTER",
        "start_node": "3890232203",
        "predefined_car": "gt3_evo",
        "module_id": "gt",
        "modules": ["gt"],
        "elevations": {
            # Mountain Straight climb up to Skyline crest
            4: 1.5, 5: 3.0, 6: 4.5, 7: 5.5, 8: 6.0, 9: 5.5, 10: 4.5, 11: 3.5,
            # The Esses / Dipper steep descent
            12: 2.5, 13: 1.5, 14: 0.5,
        },
    },
    "portimao_gp": {
        "name": "Autodromo Internacional do Algarve",
        "description": "Spectacular undulating Portuguese rollercoaster featuring Torre VIP and sweeping downhill Galp curve.",
        "rel_id": 7509968,
        "fia_length": 4653.0,
        "num_waypoints": 28,
        "default_width": 13.5,
        "straight_width": 15.0,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 4.0,
        "default_laps": 3,
        "tag": "PORTUGUESE ROLLERCOASTER",
        "start_node": "5006070798",
        "predefined_car": "gt2_biturbo",
        "module_id": "gt",
        "modules": ["gt"],
        "elevations": {
            # Torre VIP hairpin crest & plunge
            6: 2.0, 7: 4.0, 8: 3.0,
            # Samsung crest
            12: 2.0, 13: 3.0, 14: 2.0,
            # Downhill plunge into Galp
            24: 2.0, 25: 1.0,
        },
    },
    "le_mans_sarthe": {
        "name": "Circuit de la Sarthe (Le Mans)",
        "description": "The crown jewel of endurance motorsport: Dunlop Bridge, Mulsanne Straight, Indianapolis, and Porsche Curves.",
        "rel_id": 2126739,
        "fia_length": 13626.0,
        "num_waypoints": 36,
        "default_width": 13.5,
        "straight_width": 15.0,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 4.5,
        "default_laps": 3,
        "tag": "24 HOURS OF LE MANS",
        "start_node": "3599294865",
        "predefined_car": "gt1_legend",
        "module_id": "gt",
        "modules": ["gt"],
        "elevations": {
            # Dunlop curve & bridge uphill crest
            1: 1.5, 2: 3.0, 3: 2.0,
        },
    },
}


def process_gt_circuit(cid, cache_dir):
    cfg = GT_CIRCUITS[cid]
    path = osm_file_path(cache_dir, cid)
    if not os.path.exists(path):
        raise SystemExit(f"Missing {path}: run `python3 scripts/osm_importer.py download --track {cid}` first")
    root = ET.parse(path).getroot()

    nodes = {n.get("id"): (float(n.get("lat")), float(n.get("lon"))) for n in root.findall("node")}
    ways = {w.get("id"): [nd.get("ref") for nd in w.findall("nd")] for w in root.findall("way")}

    if "way_id" in cfg:
        chain_nodes = ways[cfg["way_id"]]
    else:
        rel = next(r for r in root.findall("relation") if r.get("id") == str(cfg["rel_id"]))

        if cfg.get("monaco_filter"):
            exclude = {
                "850261588", "1388331347", "39839529", "161752645", "348019480",
                "1551240829", "1551240830", "1082515450", "4230006", "434567309",
                "160004393", "1230247123", "41929969", "1451501763", "60767229",
                "1453878835"
            }
            w_ids = [m.get("ref") for m in rel.findall("member") if m.get("type") == "way" and m.get("ref") not in exclude and m.get("ref") in ways]
            seen = set()
            ordered = [w for w in w_ids if not (w in seen or seen.add(w))]
            idx_start = ordered.index(cfg["start_way"])
            ordered = ordered[idx_start:] + ordered[:idx_start]
            chain_nodes = stitch_ways(cid, [ways[wid] for wid in ordered], nodes)
        elif cfg.get("rbr_filter"):
            f1_ways = [
                "822592410", "822592403", "822592404", "347958266", "822592398",
                "822592399", "822592400", "822592401", "822592402", "822592405",
                "822592406", "822592407", "822592408", "822592409"
            ]
            chain_nodes = stitch_ways(cid, [ways[wid] for wid in f1_ways if wid in ways], nodes)
        elif cfg.get("marina_filter"):
            way_members = [m.get("ref") for m in rel.findall("member") if m.get("type") == "way" and m.get("role") != "pit_lane" and m.get("ref") in ways]
            filtered = []
            for wid in way_members:
                w_elem = next(w for w in root.findall("way") if w.get("id") == wid)
                tags = {t.get("k"): t.get("v") for t in w_elem.findall("tag")}
                name = tags.get("name", "")
                highway = tags.get("highway", "")
                if "pit" not in name.lower() and highway != "service":
                    filtered.append(wid)
            seen = set()
            ordered = [w for w in filtered if not (w in seen or seen.add(w))]
            chain_nodes = stitch_ways(cid, [ways[wid] for wid in ordered], nodes)
        elif cfg.get("madring_filter"):
            way_members = [
                m.get("ref") for m in rel.findall("member")
                if m.get("type") == "way" and m.get("ref") != "1552567031" and m.get("ref") in ways
            ]
            chain_nodes = stitch_ways(cid, [ways[wid] for wid in way_members], nodes)
        else:
            w_ids = [m.get("ref") for m in rel.findall("member") if m.get("type") == "way" and m.get("role") != "pit_lane" and m.get("ref") in ways]
            seen = set()
            ordered = [w for w in w_ids if not (w in seen or seen.add(w))]
            chain_nodes = stitch_ways(cid, [ways[wid] for wid in ordered], nodes)

    if "start_node" in cfg and cfg["start_node"] in chain_nodes:
        idx_start = chain_nodes.index(cfg["start_node"])
        chain_nodes = chain_nodes[idx_start:] + chain_nodes[:idx_start]

    check_loop_joins(cid, chain_nodes, nodes, way_edges(ways.values()))

    lat0 = sum(nodes[n][0] for n in chain_nodes) / len(chain_nodes)
    lon0 = sum(nodes[n][1] for n in chain_nodes) / len(chain_nodes)
    metric_pts = [latlon_to_meters(nodes[n][0], nodes[n][1], lat0, lon0) for n in chain_nodes]

    # Start heading from the first nodes of the chain
    p_start = metric_pts[0]
    p_ahead = metric_pts[min(6, len(metric_pts) - 1)]
    heading = math.atan2(p_ahead[1] - p_start[1], p_ahead[0] - p_start[0])
    rotated_pts = rotate_points(metric_pts, heading)

    measured_len = polyline_length(rotated_pts, closed=True)
    check_length_ratio(cid, measured_len, cfg["fia_length"])
    target_half_len = cfg["fia_length"] * 0.5
    scale_factor = target_half_len / measured_len if measured_len > 0 else 1.0

    scaled_pts = [(x * scale_factor, y * scale_factor) for x, y in rotated_pts]

    x0, y0 = scaled_pts[0]
    aligned_pts = [(x - x0, y - y0) for x, y in scaled_pts]

    resampled, _, final_len = resample_polyline(aligned_pts, cfg["num_waypoints"])

    # Start line alignment along +X
    dx = resampled[1][0] - resampled[0][0]
    dy = resampled[1][1] - resampled[0][1]
    final_pts = rotate_points(resampled, math.atan2(dy, dx))

    fx0, fy0 = final_pts[0]
    final_pts = [(x - fx0, y - fy0) for x, y in final_pts]

    # Suzuka start straight orientation: align using approach vector to ensure grid slots lie on Y=0
    if cfg.get("crossover"):
        # The main straight on Suzuka runs from waypoint index 31 through 0 into 1
        dx_s = final_pts[0][0] - final_pts[-3][0]
        dy_s = final_pts[0][1] - final_pts[-3][1]
        suzuka_pts = rotate_points(final_pts, math.atan2(dy_s, dx_s))
        # Zero out start line at (0, 0)
        sx0, sy0 = suzuka_pts[0]
        final_pts = [(x - sx0, y - sy0) for x, y in suzuka_pts]
        # Make the straight waypoints leading up to 0 exactly collinear along y=0
        final_pts[-1] = (final_pts[-1][0], 0.0)
        final_pts[-2] = (final_pts[-2][0], 0.0)
        final_pts[-3] = (final_pts[-3][0], 0.0)
        final_pts[1] = (final_pts[1][0], 0.0)

    n = len(final_pts)
    waypoints = []
    custom_elevations = cfg.get("elevations", {})

    for i in range(n):
        x, y = final_pts[i]
        norm_cross = deflection_sine(final_pts, i)

        is_straight = abs(norm_cross) < 0.18 and (i < 3 or i > n - 3)
        width = cfg["straight_width"] if is_straight else cfg["default_width"]

        left_curb = False
        right_curb = False
        if norm_cross > 0.35:
            left_curb = True
        elif norm_cross < -0.35:
            right_curb = True

        elev = custom_elevations.get(i, 0.0)

        # Suzuka bridge elevation and corridor clearance
        wall_dist = None
        if cfg.get("crossover"):
            if i in [24, 25, 26]:
                elev = 5.0
            elif i in [23, 27]:
                elev = 2.5
            else:
                elev = 0.0

            # Narrow width and wall distance at parallel hairpin corridor 16-17 / 22-23
            if i in [16, 17, 22, 23]:
                width = 11.5
                wall_dist = 2.2

        wp_entry = {
            "x": round(x, 1),
            "y": round(y, 1),
            "width": round(width, 1),
            "left_curb": left_curb,
            "right_curb": right_curb,
            "elevation": elev,
        }
        if wall_dist is not None:
            wp_entry["wall_dist"] = wall_dist
        waypoints.append(wp_entry)

    return {
        "id": cid,
        "name": cfg["name"],
        "description": cfg["description"],
        "tag": cfg["tag"],
        "barrier": cfg["barrier"],
        "barrier_offset": cfg["barrier_offset"],
        "default_laps": cfg["default_laps"],
        "fia_length": cfg["fia_length"],
        "half_length": target_half_len,
        "final_len": round(final_len, 1),
        "waypoints": waypoints,
    }


def generate_gt_rust_code(cdata):
    """Full `track_<id>()` in the template used by crates/tdrace-app/src/module/gt.rs."""
    lines = []
    lines.append(f"    /// {cdata['name']}: {cdata['description']}")
    lines.append(f"    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: {cdata['final_len']:.1f}m (Real FIA: {cdata['fia_length']:.0f}m).")
    lines.append(f"    pub fn track_{cdata['id']}() -> Track {{")
    lines.append("        let waypoints = vec![")

    for w in cdata["waypoints"]:
        curb_str = ""
        if w["left_curb"] or w["right_curb"]:
            curb_str = f".with_curbs({str(w['left_curb']).lower()}, {str(w['right_curb']).lower()})"
        elev_str = ""
        if w["elevation"] > 0.0 or w["elevation"] < 0.0:
            elev_str = f".with_elevation({w['elevation']:.1f})"
        wall_dist_str = ""
        if "wall_dist" in w:
            wall_dist_str = f".with_wall_distances(Some({w['wall_dist']:.1f}), Some({w['wall_dist']:.1f}))"
        surf_str = ".with_surface(SurfaceType::Asphalt)"
        lines.append(
            f"            TrackWaypoint::new(Vec2::new({w['x']:.1f}, {w['y']:.1f}), {w['width']:.1f}){surf_str}{curb_str}{elev_str}{wall_dist_str},"
        )

    lines.append("        ];")
    lines.append("")
    lines.append("        let spline = TrackSpline::new(waypoints, true);")
    lines.append("        let (left_walls, right_walls, left_poly, right_poly) =")
    lines.append(f"            generate_walls_from_spline(&spline, {cdata['barrier_offset']:.1f}, {cdata['barrier']});")
    lines.append("")
    lines.append("        let checkpoints = generate_checkpoints(&spline, 20, 3);")
    lines.append("        let starting_grid = generate_grid_positions(&spline, 18, 10.0, 2.5);")
    lines.append("")
    lines.append("        Track {")
    lines.append(f'            name: "{cdata["name"]}".to_string(),')
    lines.append(f'            description: "{cdata["description"]}".to_string(),')
    lines.append("            category: TrackCategory::Main,")
    lines.append("            kind: TrackKind::Circuit,")
    lines.append("            spline,")
    lines.append("            geometry: TrackGeometry {")
    lines.append("                inner_walls: left_walls,")
    lines.append("                outer_walls: right_walls,")
    lines.append("                obstacles: Vec::new(),")
    lines.append("                surface_zones: Vec::new(),")
    lines.append("                jump_ramps: Vec::new(),")
    lines.append("                left_boundary_polyline: left_poly,")
    lines.append("                right_boundary_polyline: right_poly,")
    lines.append("                ..Default::default()")
    lines.append("            },")
    lines.append("            checkpoints,")
    lines.append("            grid_positions: starting_grid,")
    lines.append("            default_surface: SurfaceType::Grass,")
    lines.append("            pit_box_area: None,")
    lines.append(f"            default_laps: {cdata['default_laps']},")
    lines.append("            car_category: CarCategory::Gt,")
    lines.append('            module_id: Some("gt".to_string()),')
    lines.append('            modules: vec!["gt".to_string()],')
    lines.append('            scale: "0.5x".to_string(),')
    lines.append("            wikipedia_url: None,")
    lines.append("            osm_url: None,")
    lines.append("            country_code: None,")
    lines.append("            country_name: None,")
    lines.append("            min_width: None,")
    lines.append("            max_width: None,")
    lines.append("            is_inspired: false,")
    lines.append("        }.with_default_runoff_surfaces()")
    lines.append("    }")
    return "\n".join(lines)


def print_gt_summary(data):
    print(f"[{data['id']:14}] {data['name'][:35]:35} | {len(data['waypoints'])} waypoints | {data['final_len']:6.1f}m (target {data['half_length']:.1f}m, 0.5x FIA)")


# ---------------------------------------------------------------------------
# Kart circuits (1:1 CIK-FIA length)
# ---------------------------------------------------------------------------

KART_TRACKS = {
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
    "wackersdorf": {
        "name": "Prokart Raceland Wackersdorf",
        "description": "Premier German CIK-FIA World Championship kart venue featuring fast chicane sweeps, long drafting straights, and technical hairpins.",
        "bbox": (12.210, 49.324, 12.220, 49.329),
        "ways": [156019609],
        "fia_length": 1190.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 30,
        "elevation_fn": None,
    },
    "kristianstad": {
        "name": "Kristianstad Karting Klubb (Åsum Ring)",
        "description": "Historic Swedish European and World Championship circuit famous for technical rhythm sections, elevation sweeps, and high-G esses.",
        "bbox": (14.118, 55.983, 14.128, 55.989),
        "ways": [87888593],
        "fia_length": 1221.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 30,
        "elevation_fn": None,
    },
    "seven_laghi": {
        "name": "Circuito Internazionale 7 Laghi (Castelletto)",
        "description": "Demanding Italian proving ground in Castelletto di Branduzzo featuring technical switchbacks, heavy trail-braking hairpins, and undulating sweepers.",
        "bbox": (9.095, 45.061, 9.105, 45.068),
        "ways": [80905675],
        "fia_length": 1256.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 30,
        "elevation_fn": None,
    },
    "ampfing": {
        "name": "Schweppermannring Ampfing",
        "description": "Famous Bavarian outdoor kart arena with flowing mid-speed esses, double-apex hairpin turns, and high-speed drafting straights.",
        "bbox": (12.438, 48.243, 12.448, 48.249),
        "ways": [110580562],
        "fia_length": 1063.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 28,
        "elevation_fn": None,
    },
    "silverstone_national_kart": {
        "name": "Silverstone National Karting Circuit",
        "description": "The agile National sprint layout at the premier Kart Silverstone complex, featuring snappy switchbacks, Priory hairpin, and tight apex rumble curbs.",
        "bbox": (-1.022, 52.073, -1.014, 52.079),
        "ways": [],
        "nodes_cycle": [
            13912520266, 13912520264, 13912520267, 13148241482, 13912520268, 13912520265, 13912520269,
            13912520270, 13912520271, 13912520222, 13912520223, 13912520224, 13613991005, 13912520225,
            13912520226, 13912520227, 13148241490, 13148241489, 13148241488, 13912520263, 13912520262,
            13912520261, 13912520260, 13912520259, 13912520258, 13912520257, 13912520251, 13912520250,
            13912520249, 13912520248, 13912520247, 13912520246, 13912520433, 13912520245, 13912520388,
            13912520387, 13912520386, 13912520385, 13912520384, 13912520234, 13912520256, 13912520233,
            13912520232, 13912520231
        ],
        "fia_length": 520.0,
        "default_width": 8.0,
        "straight_width": 8.8,
        "num_waypoints": 24,
        "elevation_fn": None,
    },
    "laval_kart": {
        "name": "Laval Karting (Circuit Louis Beuvron)",
        "description": "Legendary French CIK-FIA Grade 1 karting arena in Mayenne featuring banked parabolique, rapid esses, and technical chicanes.",
        "ways": [183330357],
        "fia_length": 1232.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 50,  # ~25 m spacing keeps the hairpins within ~3 m of the OSM line
        "elevation_fn": None,
    },
    "whilton_mill": {
        "name": "Whilton Mill Kart Circuit",
        "description": "Premier British National karting venue in Northamptonshire featuring challenging downhill esses, Ashby hairpin, and rapid chicanes.",
        # Full lap without the pit lane, through Christmas Corner, Ashby, Zulu, Back Straight and Pit Bend.
        "ways": [1208351124, 1208351123, 244268438, 1208351121, 1208351112, 1208351122, 1208351119, 1208351120,
                 1208351115, 1208351114, 1208351118, 1208351116, 1208351125, 1208351117, 1208351126],
        # The spec lists 1200 m, but the complete OSM lap is ~1044 m; keep the mapped (1:1) length.
        "fia_length": 1044.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 50,  # ~21 m spacing keeps the esses within ~3 m of the OSM line
        "elevation_fn": None,
    },
    "campillos": {
        "name": "Kartcenter Campillos",
        "description": "FIA Karting World Championship venue in Andalusia featuring fast sweeping curves, undulating esses, and technical braking zones.",
        "ways": [639076279],
        "fia_length": 1580.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 32,
        "elevation_fn": None,
    },
    "valencia_kart": {
        "name": "Kartodromo Internacional Lucas Guerrero (Valencia)",
        "description": "Premier Spanish championship venue in Chiva featuring sweeping esses, technical hairpins, and wide overtaking zones.",
        "ways": [751513226],
        "fia_length": 1428.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 32,
        "elevation_fn": None,
    },
}


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
                if direct_dist / arc_len > 0.980 and direct_dist > best_len:
                    best_len = direct_dist
                    best_start_idx = i
                    best_heading = math.atan2(pb[1] - points[i][1], pb[0] - points[i][0])

    return best_start_idx, best_heading


def process_kart_track(track_id, cache_dir):
    spec = KART_TRACKS[track_id]
    data = load_osm_elements(cache_dir, track_id)

    nodes = {e["id"]: (e["lat"], e["lon"]) for e in data["elements"] if e["type"] == "node"}
    ways = {e["id"]: e for e in data["elements"] if e["type"] == "way"}

    raw_node_ids = []
    if "nodes_cycle" in spec:
        raw_node_ids = list(spec["nodes_cycle"])
    else:
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

    if spec.get("reverse", False):
        raw_node_ids = list(reversed(raw_node_ids))
    check_loop_joins(track_id, raw_node_ids, nodes, way_edges(w["nodes"] for w in ways.values()))

    raw_pts = [nodes[nid] for nid in raw_node_ids]
    lat0 = sum(p[0] for p in raw_pts) / len(raw_pts)
    lon0 = sum(p[1] for p in raw_pts) / len(raw_pts)

    metric_pts = [latlon_to_meters(p[0], p[1], lat0, lon0) for p in raw_pts]

    # If start_node_id specified, use it; otherwise find longest straight
    if "start_node_id" in spec:
        start_idx = raw_node_ids.index(spec["start_node_id"])
        reordered_pts = metric_pts[start_idx:] + metric_pts[:start_idx]
        heading = heading_ahead(reordered_pts)
    else:
        start_idx, heading = find_longest_straight(metric_pts)
        reordered_pts = metric_pts[start_idx:] + metric_pts[:start_idx]

    # Rotate so start straight heads along +X
    rotated_pts = rotate_points(reordered_pts, heading)

    # Scale to exact FIA homologation length
    current_len = polyline_length(rotated_pts, closed=True)
    check_length_ratio(track_id, current_len, spec["fia_length"])
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
    resampled, _, final_len = resample_polyline(aligned_pts, spec["num_waypoints"])

    # Compute curvature, assign widths, and inside apex curbs
    n = len(resampled)
    waypoints = []
    for i in range(n):
        x, y = resampled[i]
        norm_cross = deflection_sine(resampled, i)

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


def generate_kart_rust_code(track_data):
    wps = track_data["waypoints"]
    lines = []
    lines.append("        let waypoints = vec![")
    for w in wps:
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


def print_kart_summary(res):
    print(f"=== {res['name']} ({res['id']}) ===")
    print(f"  Waypoints: {len(res['waypoints'])}, Total Length: {res['total_length']} m (FIA Target: {res['fia_length']} m)")


# ---------------------------------------------------------------------------
# Rallycross circuits (1:1 FIA length, asphalt/dirt surfaces)
# ---------------------------------------------------------------------------

RALLY_TRACKS = {
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
        "query": '[out:json][timeout:25];(way["highway"="raceway"](53.444,9.685,53.453,9.700););out body;>;out skel qt;',
        "fia_length": 952.0,
        "default_width": 13.0,
        "straight_width": 14.5,
        "num_waypoints": 26,
        "jump": None,
    },
    "montalegre_rx": {
        "name": "Pista Automóvel de Montalegre (World RX Portugal)",
        "description": "High-altitude mountain thriller in Portugal featuring an undulating drag straight, gravel stadium section and fast table crest.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](41.842,-7.762,41.850,-7.752););out body;>;out skel qt;',
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
        "query": '[out:json][timeout:25];(way["highway"="raceway"](46.963,17.410,46.974,17.428););out body;>;out skel qt;',
        "fia_length": 1220.0,
        "default_width": 13.5,
        "straight_width": 14.5,
        "num_waypoints": 30,
        "jump": None,
    },
    "kouvola_rx": {
        "name": "Tykkimäen Moottorirata (World RX Finland)",
        "description": "Finnish rallycross heartland featuring severe elevation rollercoasters, blind gravel drops and the flying Tykkimäki dirt crest.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](60.878,26.785,60.890,26.810););out body;>;out skel qt;',
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
    "mettet_rx": {
        "name": "Circuit Jules Tacheny Mettet (World RX Belgium)",
        "description": "Belgian World RX showdown featuring rapid asphalt sweeps, banked dirt esses, and the notorious Mettet tabletop jump.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](50.295,4.640,50.310,4.665););out body;>;out skel qt;',
        "way_ids": [178384323, 178384334, 178384335, 178384337, 178384345, 178384346, 178384349, 178384356, 178384358, 178384360, 178384364, 178384367, 178384383, 178384386],
        "fia_length": 1149.0,
        "default_width": 13.0,
        "straight_width": 14.0,
        "num_waypoints": 28,
        "jump": {
            "name": "Mettet Arena Dirt Jump",
            "at_fraction": 0.42,
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
    "silverstone_rx": {
        "name": "Silverstone Circuit RX (World RX Great Britain)",
        "description": "Speedmachine Festival circuit carved into the legendary Silverstone Stowe complex, featuring high-speed tarmac drifts and loose gravel switchbacks.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](52.060,-1.035,52.075,-1.005););out body;>;out skel qt;',
        "way_ids": [169851260, 227310197, 259160216, 227339144],
        "fia_length": 972.0,
        "default_width": 13.0,
        "straight_width": 14.0,
        "num_waypoints": 28,
        "jump": {
            "name": "Silverstone Arena Dirt Jump",
            "at_fraction": 0.55,
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
    "riga_rx": {
        "name": "Biķernieku Trase (World RX Latvia)",
        "description": "The historic Riga cathedral of speed featuring a punishing forest drag, sweeping double parallel dirt jump crests and high-grip technical gravel curves.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](56.955,24.215,56.975,24.245););out body;>;out skel qt;',
        "way_ids": [256784387, 945640986, 588947722, 588947720, 588947717, 588947719, 588947714],
        "fia_length": 1294.0,
        "default_width": 13.5,
        "straight_width": 14.5,
        "num_waypoints": 30,
        "jump": {
            "name": "Biķernieki Double Jump Crest",
            "at_fraction": 0.60,
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
    "killarney_rx": {
        "name": "Killarney International Raceway (World RX South Africa)",
        "description": "Scenic Cape Town thriller in the shadow of Table Mountain, featuring a rapid asphalt drag, loose dirt jumps and high-drift hairpin transitions.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](-33.840,18.520,-33.825,18.535););out body;>;out skel qt;',
        "way_ids": [42125321, 1214903811, 1214903812, 1214903813, 1214903814, 1214903816, 1214903817, 42481058],
        "fia_length": 1067.0,
        "default_width": 13.0,
        "straight_width": 14.0,
        "num_waypoints": 28,
        "jump": {
            "name": "Killarney Dirt Kicker Jump",
            "at_fraction": 0.82,
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
    "yas_marina_rx": {
        "name": "Yas Marina RX Arena (World RX Abu Dhabi)",
        "description": "Spectacular twilight rallycross inside the Yas Marina amphitheater, featuring stadium dirt jumps, tight desert hairpins and high-speed grandstand sweeps.",
        "query": '[out:json][timeout:25];(way["highway"="raceway"](24.460,54.595,24.475,54.615););out body;>;out skel qt;',
        "way_ids": [1083519983, 1083519984, 1083519985, 1083519986, 1083519987],
        "fia_length": 1050.0,
        "default_width": 13.0,
        "straight_width": 14.0,
        "num_waypoints": 28,
        "jump": {
            "name": "Yas Marina Arena Dirt Jump",
            "at_fraction": 0.58,
            "height": 1.3,
            "angle_deg": 5.5,
            "launch_speed": 2.2,
            "dist": 16.0,
            "surface": "Dirt",
        },
    },
    "essay_rx": {
        "name": "Circuit des Ducs (Essay RX)",
        "description": "Historic French rallycross proving ground in Normandy featuring a high-speed asphalt start, the iconic 'La Butte' dirt jump crest, and scenic Norman woods.",
        # Main lap (the Tour Joker way 787532794 is left out). (way, first node) pairs; None = whole way.
        "segments": [(787532793, 7363261553), (787532796, None), (787532795, None)],
        # The spec lists 1115 m, but the mapped lap is ~925 m (the old preset was 914 m); keep 1:1.
        "fia_length": 925.0,
        "default_width": 13.0,
        "straight_width": 14.0,
        "num_waypoints": 44,  # ~21 m spacing keeps the lap within ~3 m of the OSM line
        "jump": None,
    },
    "dreux_rx": {
        "name": "Circuit de l'Ouest Parisien (Dreux RX)",
        "description": "French Rallycross Championship venue in Dreux featuring high-speed sweeping tarmac, technical loose dirt hairpins, and tabletop jump.",
        # "Circuit Mixte de l'Ouest Parisien" from the start straight; the Tour Joker is left out.
        "segments": [(297738880, 3016400386), (1328613408, None), (787140597, None), (1311041712, None),
                     (787140601, None), (1311041713, None), (787140602, None), (787140604, None)],
        "fia_length": 1050.0,
        "default_width": 13.5,
        "straight_width": 14.5,
        "num_waypoints": 44,  # ~24 m spacing keeps the lap within ~2 m of the OSM line
        "jump": None,
    },
}


def rally_way_surface(tags):
    surf_tag = tags.get("surface", "")
    return "Dirt" if surf_tag in ["gravel", "fine_gravel", "dirt"] else ("Concrete" if surf_tag == "concrete" else "Asphalt")


def process_rally_track(track_id, cache_dir):
    spec = RALLY_TRACKS[track_id]
    data = load_osm_elements(cache_dir, track_id)

    nodes = {e["id"]: (e["lat"], e["lon"]) for e in data["elements"] if e["type"] == "node"}
    ways = {e["id"]: e for e in data["elements"] if e["type"] == "way"}

    # (node id, surface) along the lap
    raw_nodes_surf = []
    if "segments" in spec:
        for nid, wid in chain_segments(ways, spec["segments"]):
            raw_nodes_surf.append((nid, rally_way_surface(ways[wid].get("tags", {}))))
    elif track_id == "hell_rx":
        nodes_67 = ways[1069390967]["nodes"][1:]  # Skip start grid lane (node 0)
        nodes_68 = ways[1069390968]["nodes"]
        nodes_78 = ways[1069390978]["nodes"]
        nodes_77 = ways[1069390977]["nodes"]
        for nid in nodes_67[:-1]:
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in nodes_68[:-1]:
            raw_nodes_surf.append((nid, "Dirt"))
        for nid in nodes_78[:-1]:
            raw_nodes_surf.append((nid, "Dirt"))
        for nid in nodes_77[19:-1]:
            raw_nodes_surf.append((nid, "Asphalt"))
    elif track_id in ("holjes_rx", "loheac_rx"):
        drag_lane_way = 599300791 if track_id == "holjes_rx" else 787615501
        for wid in spec["way_ids"]:
            w = ways[wid]
            surf = rally_way_surface(w.get("tags", {}))
            wnodes = w["nodes"]
            if wid == drag_lane_way:
                wnodes = wnodes[2:]  # Skip start drag lane (nodes 0, 1)
            for nid in wnodes[:-1]:
                raw_nodes_surf.append((nid, surf))
    elif track_id == "estering_rx":
        w = ways[267094190]["nodes"]
        estering_nodes = w[15:-1] + w[:15]
        for i, nid in enumerate(estering_nodes):
            surf = "Dirt" if 7 <= i <= 28 else "Asphalt"
            raw_nodes_surf.append((nid, surf))
    elif track_id == "montalegre_rx":
        for nid in ways[532046956]["nodes"][:-1]:
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in ways[1096210264]["nodes"][:-1]:
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in ways[532046958]["nodes"][:-1]:
            raw_nodes_surf.append((nid, "Dirt"))
        for nid in ways[1096210265]["nodes"][:-1]:
            raw_nodes_surf.append((nid, "Asphalt"))
    elif track_id == "nyirad_rx":
        for nid in ways[172413359]["nodes"]:
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in reversed(ways[172413357]["nodes"][:15]):
            raw_nodes_surf.append((nid, "Dirt"))
        for nid in ways[172413356]["nodes"][12:24]:
            raw_nodes_surf.append((nid, "Dirt"))
    elif track_id == "kouvola_rx":
        for nid in ways[149713976]["nodes"][36:-1]:
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in ways[149713976]["nodes"][:7]:
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in ways[149713907]["nodes"][:40]:
            raw_nodes_surf.append((nid, "Dirt"))
    elif track_id == "catalunya_rx":
        for nid in ways[831804327]["nodes"][200:-1]:
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in ways[831804327]["nodes"][:3]:
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in reversed(ways[921317982]["nodes"]):
            raw_nodes_surf.append((nid, "Dirt"))
        for nid in reversed(ways[967275593]["nodes"][1:]):
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in reversed(ways[921317981]["nodes"][1:-1]):
            raw_nodes_surf.append((nid, "Dirt"))
    elif track_id == "killarney_rx":
        w42_nodes = ways[42125321]["nodes"]
        w_cut = ways[42481058]["nodes"]
        join_idx = w_cut.index(ways[1214903817]["nodes"][-1])
        segments = [
            (w42_nodes[24:45], "Asphalt"),
            (ways[1214903811]["nodes"][1:], "Dirt"),
            (ways[1214903812]["nodes"][1:], "Asphalt"),
            (w42_nodes[54:57][1:], "Asphalt"),
            (ways[1214903813]["nodes"][1:], "Asphalt"),
            (ways[1214903814]["nodes"][1:], "Dirt"),
            (ways[1214903816]["nodes"][1:], "Dirt"),
            (ways[1214903817]["nodes"][1:], "Asphalt"),
            (w_cut[join_idx:][1:], "Asphalt"),
        ]
        for nds, surf in segments:
            for nid in nds:
                raw_nodes_surf.append((nid, surf))
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
                raw_nodes_surf.append((nid, surf))

    node_ids = [nid for nid, _ in raw_nodes_surf]
    surfaces = [surf for _, surf in raw_nodes_surf]
    check_loop_joins(track_id, node_ids, nodes, way_edges(w["nodes"] for w in ways.values()))

    raw_pts = [nodes[nid] for nid in node_ids]
    lat0 = sum(p[0] for p in raw_pts) / len(raw_pts)
    lon0 = sum(p[1] for p in raw_pts) / len(raw_pts)
    metric_pts = [latlon_to_meters(p[0], p[1], lat0, lon0) for p in raw_pts]

    # Start straight heading from the first nodes of the lap (segment laps: first node 60 m ahead)
    if "segments" in spec:
        heading = heading_ahead(metric_pts)
    else:
        p_start = metric_pts[0]
        p_ahead = metric_pts[min(6, len(metric_pts) - 1)]
        heading = math.atan2(p_ahead[1] - p_start[1], p_ahead[0] - p_start[0])

    # Rotate so start straight heads along +X
    rotated_pts = rotate_points(metric_pts, heading)

    # Scale to exact FIA homologation length
    current_len = polyline_length(rotated_pts, closed=True)
    check_length_ratio(track_id, current_len, spec["fia_length"])
    scale_factor = spec["fia_length"] / current_len if current_len > 0 else 1.0
    scaled_pts = [((x * scale_factor), (y * scale_factor)) for x, y in rotated_pts]

    # Translate so start line is at x=0, y=0
    x_offset, y_offset = scaled_pts[0]
    aligned_pts = [((x - x_offset), (y - y_offset)) for x, y in scaled_pts]

    # Resample to target waypoint count
    resampled, resampled_surfaces, final_len = resample_polyline(aligned_pts, spec["num_waypoints"], surfaces)

    # Compute curvature and assign apex curbs
    n = len(resampled)
    waypoints = []
    for i in range(n):
        x, y = resampled[i]
        norm_cross = deflection_sine(resampled, i)

        # Straight width on start straight (first 4 and last 2 waypoints with low curvature)
        is_straight = abs(norm_cross) < 0.18 and (i < 4 or i >= n - 2)
        width = spec["straight_width"] if is_straight else spec["default_width"]

        # Hairpin apex clearance: narrow road width to prevent wall intersection
        if abs(norm_cross) > 0.70:
            width = min(width, 11.5)

        # Curbs: assigned only to inside corner apexes (deflection > 20 deg), never on start straight or waypoint 0
        left_curb = False
        right_curb = False
        if not is_straight and i != 0:
            if norm_cross > 0.35:  # Left turn -> inside curb on left side
                left_curb = True
            elif norm_cross < -0.35:  # Right turn -> inside curb on right side
                right_curb = True

        waypoints.append({
            "x": round(x, 1),
            "y": round(y, 1),
            "width": round(width, 1),
            "surface": resampled_surfaces[i],
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
        "jump": spec.get("jump"),
    }


def generate_rally_rust_code(track_data):
    wps = track_data["waypoints"]
    lines = []
    lines.append("    let waypoints = vec![")
    for w in wps:
        curb_str = ""
        if w["left_curb"] or w["right_curb"]:
            curb_str = f".with_curbs({str(w['left_curb']).lower()}, {str(w['right_curb']).lower()})"
        surf_str = f".with_surface(SurfaceType::{w['surface']})"
        lines.append(
            f"        TrackWaypoint::new(Vec2::new({w['x']:.1f}, {w['y']:.1f}), {w['width']:.1f}){surf_str}{curb_str},"
        )
    lines.append("    ];")
    return "\n".join(lines)


def print_rally_summary(res):
    print(f"=== {res['name']} ({res['id']}) ===")
    print(f"  Waypoints: {len(res['waypoints'])}, Total Length: {res['total_length']} m (FIA Target: {res['fia_length']} m)")
    asphalt_cnt = sum(1 for w in res['waypoints'] if w['surface'] == 'Asphalt')
    dirt_cnt = len(res['waypoints']) - asphalt_cnt
    print(f"  Surface: {asphalt_cnt * 100 // len(res['waypoints'])}% Asphalt, {dirt_cnt * 100 // len(res['waypoints'])}% Dirt")


# ---------------------------------------------------------------------------
# Download (all real circuits with an osm_url in tracks/)
# ---------------------------------------------------------------------------

# Extra ground around the circuit, so barriers, gravel traps and grandstands are included.
DOWNLOAD_MARGIN_M = 300.0
MAP_API = "https://api.openstreetmap.org/api/0.6"
OVERPASS_API = "https://overpass-api.de/api/interpreter"


def provenance_osm_urls(tracks_dir=None, cache_dir=DEFAULT_CACHE_DIR):
    """{circuit id: osm_url} for every official circuit JSON in tracks/ that has an osm_url.

    The id is the catalog id, or an older alias when the OSM file is already saved under that alias
    (e.g. the NASCAR files `daytona.osm`), so existing map files keep working.
    """
    tracks_dir = tracks_dir or TRACKS_DIR
    aliases = load_aliases(tracks_dir)
    urls = {}
    for module in sorted(os.listdir(tracks_dir)):
        module_dir = os.path.join(tracks_dir, module)
        if module.startswith(".") or not os.path.isdir(module_dir):
            continue
        for name in sorted(os.listdir(module_dir)):
            if not name.endswith(".json"):
                continue
            with open(os.path.join(module_dir, name), "r", encoding="utf-8") as f:
                osm_url = json.load(f).get("osm_url")
            if not osm_url:
                continue
            cid = name[: -len(".json")]
            saved_as = [a for a, target in aliases.items() if target == cid and os.path.exists(osm_file_path(cache_dir, a))]
            urls[saved_as[0] if saved_as and not os.path.exists(osm_file_path(cache_dir, cid)) else cid] = osm_url
    return urls


def http_get(url, data=None, timeout=120):
    req = urllib.request.Request(url, data=data, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.read().decode("utf-8")


def element_bounds(osm_url):
    """(south, west, north, east) of the OSM element an osm_url points to."""
    m = re.search(r"openstreetmap\.org/(node|way|relation)/(\d+)", osm_url)
    if not m:
        raise ValueError(f"not an OSM element URL: {osm_url}")
    kind, eid = m.groups()
    url = f"{MAP_API}/{kind}/{eid}" + ("" if kind == "node" else "/full")
    root = ET.fromstring(http_get(url))
    lats = [float(n.get("lat")) for n in root.findall("node")]
    lons = [float(n.get("lon")) for n in root.findall("node")]
    return min(lats), min(lons), max(lats), max(lons)


def config_bbox(track_id):
    """(south, west, north, east) from a kart or rallycross config, when it has one."""
    if "bbox" in KART_TRACKS.get(track_id, {}):
        min_lon, min_lat, max_lon, max_lat = KART_TRACKS[track_id]["bbox"]
        return min_lat, min_lon, max_lat, max_lon
    if "query" in RALLY_TRACKS.get(track_id, {}):
        m = re.search(r"\(([-0-9.]+),([-0-9.]+),([-0-9.]+),([-0-9.]+)\)", RALLY_TRACKS[track_id]["query"])
        lat1, lon1, lat2, lon2 = map(float, m.groups())
        return min(lat1, lat2), min(lon1, lon2), max(lat1, lat2), max(lon1, lon2)
    return None


def expand_bbox(bbox, margin_m):
    s, w, n, e = bbox
    dlat = margin_m / 111320.0
    dlon = margin_m / (111320.0 * math.cos(math.radians((s + n) / 2)))
    return s - dlat, w - dlon, n + dlat, e + dlon


def download_circuit(track_id, osm_url, cache_dir, margin_m=DOWNLOAD_MARGIN_M):
    """Save the OSM map area around one circuit to <cache_dir>/<track_id>.osm. Returns a status word."""
    bbox = element_bounds(osm_url)
    cfg = config_bbox(track_id)
    if cfg:
        bbox = (min(bbox[0], cfg[0]), min(bbox[1], cfg[1]), max(bbox[2], cfg[2]), max(bbox[3], cfg[3]))
    s, w, n, e = expand_bbox(bbox, margin_m)

    source = "map"
    try:
        xml_text = http_get(f"{MAP_API}/map?bbox={w:.6f},{s:.6f},{e:.6f},{n:.6f}")
    except urllib.error.HTTPError as err:
        if err.code not in (400, 509):
            raise
        # Area too large for the map API (50k nodes / 0.25 deg2): keep only what lies near the raceways.
        source = "overpass"
        query = (
            f"[out:xml][timeout:300][bbox:{s:.6f},{w:.6f},{n:.6f},{e:.6f}];"
            f"way[highway=raceway]->.t;(node(around.t:{margin_m:.0f});way(around.t:{margin_m:.0f}););"
            "(._;>;)->.env;rel(bw.t)->.r;(.env;.r;.r>;);out body;"
        )
        xml_text = http_get(OVERPASS_API, data=query.encode("utf-8"), timeout=360)

    root = ET.fromstring(xml_text)
    raceways = [w_el for w_el in root.findall("way") if any(
        t.get("k") == "highway" and t.get("v") == "raceway" for t in w_el.findall("tag"))]
    os.makedirs(cache_dir, exist_ok=True)
    with open(osm_file_path(cache_dir, track_id), "w", encoding="utf-8") as f:
        f.write(xml_text)
    if not raceways:
        warn(track_id, "no highway=raceway way in the downloaded area: the circuit may not be mapped")
    return f"{source}, {len(raceways)} raceway ways"


def download(track_ids, cache_dir, force=False):
    urls = provenance_osm_urls()
    for tid in track_ids:
        path = osm_file_path(cache_dir, tid)
        if os.path.exists(path) and not force:
            print(f"[{tid}] exists, skipped")
            continue
        try:
            status = download_circuit(tid, urls[tid], cache_dir)
            print(f"[{tid}] saved {path} ({os.path.getsize(path) // 1024} KiB, {status})")
        except (OSError, ValueError, ET.ParseError) as err:
            warn(tid, f"download failed: {err}")
        time.sleep(1.0)  # be polite to the public OSM servers


# ---------------------------------------------------------------------------
# Command line
# ---------------------------------------------------------------------------

# ---------------------------------------------------------------------------
# JSON output (spec 042): official circuits live only in tracks/<module>/<id>.json
# ---------------------------------------------------------------------------

TRACKS_DIR = os.path.join(REPO_ROOT, "tracks")
BAKE_COMMAND = "cargo run --bin track_bake --"


def load_aliases(tracks_dir):
    path = os.path.join(tracks_dir, ".aliases.json")
    if not os.path.exists(path):
        return {}
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def catalog_id(module, track_id, tracks_dir):
    """The catalog id (file name) for an importer track id, following tracks/.aliases.json."""
    if os.path.exists(os.path.join(tracks_dir, module, f"{track_id}.json")):
        return track_id
    return load_aliases(tracks_dir).get(track_id, track_id)


def source_waypoint(w, discipline):
    """One importer waypoint as a Track JSON waypoint (the fields the Rust code used to set)."""
    surface = w.get("surface") or ("Asphalt" if discipline == "gt" else None)
    wp = {
        "point": [round(w["x"], 1), round(w["y"], 1)],
        "width": round(w["width"], 1),
        "left_curb": bool(w["left_curb"]),
        "right_curb": bool(w["right_curb"]),
        "surface": surface,
        "elevation": round(w.get("elevation", 0.0), 1),
    }
    if "wall_dist" in w:
        wp["left_wall_distance"] = wp["right_wall_distance"] = round(w["wall_dist"], 1)
    return wp


def write_source_json(discipline, data, tracks_dir=None):
    """Writes the imported waypoints to tracks/<module>/<id>.json; returns (path, track_bake command).

    An existing circuit keeps every other field (names, tag, provenance, scenery, walls); only the waypoints
    change, and `track_bake --rebuild` then regenerates spline, walls, checkpoints and grid from them, keeping
    the current wall setup. A new circuit gets a minimal file and is appended to tracks/.track_order.json.
    """
    tracks_dir = tracks_dir or TRACKS_DIR
    module = discipline
    cid = catalog_id(module, data["id"], tracks_dir)
    path = os.path.join(tracks_dir, module, f"{cid}.json")
    waypoints = [source_waypoint(w, discipline) for w in data["waypoints"]]

    if os.path.exists(path):
        with open(path, "r", encoding="utf-8") as f:
            track = json.load(f)
        track["spline"]["waypoints"] = waypoints
        track["spline"]["closed"] = True
        bake_args = ["--rebuild"]
    else:
        track = {
            "name": data["name"],
            "description": data.get("description", ""),
            "category": "main",
            "kind": {"type": "circuit"},
            "spline": {"waypoints": waypoints, "closed": True, "samples": [], "total_length": 0.0, "curves": []},
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
            "default_laps": data.get("default_laps", 3),
            "car_category": module,
            "module_id": module,
            "modules": [module],
            "scale": "0.5x" if module == "gt" else "1:1",
            "is_inspired": False,
        }
        if data.get("tag"):
            track["tag"] = data["tag"]
        order_path = os.path.join(tracks_dir, ".track_order.json")
        with open(order_path, "r", encoding="utf-8") as f:
            order = json.load(f)
        if cid not in order.setdefault(module, []):
            order[module].append(cid)
            with open(order_path, "w", encoding="utf-8") as f:
                json.dump(order, f, indent=2, ensure_ascii=False)
        bake_args = []
        if "barrier_offset" in data:
            bake_args += ["--barrier-offset", f"{data['barrier_offset']:.1f}"]
        if "barrier" in data:
            bake_args += ["--barrier-type", data["barrier"].split("::")[-1]]

    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(track, f, indent=2, ensure_ascii=False)
    rel = os.path.relpath(path, REPO_ROOT)
    return path, " ".join([BAKE_COMMAND, rel] + bake_args)


DISCIPLINES = {
    "gt": (GT_CIRCUITS, process_gt_circuit, generate_gt_rust_code, print_gt_summary),
    "kart": (KART_TRACKS, process_kart_track, generate_kart_rust_code, print_kart_summary),
    "rally": (RALLY_TRACKS, process_rally_track, generate_rally_rust_code, print_rally_summary),
}


def main():
    parser = argparse.ArgumentParser(description="Extract and generate tdrace circuits from OpenStreetMap")
    parser.add_argument("--cache-dir", default=DEFAULT_CACHE_DIR, help="OSM cache directory (default: assets/osm)")
    sub = parser.add_subparsers(dest="discipline", required=True)
    for name, (specs, _, _, _) in DISCIPLINES.items():
        p = sub.add_parser(name, help=f"{name} circuits")
        p.add_argument("--track", choices=list(specs.keys()), help="Process one circuit (default: all)")
        p.add_argument("--rust", action="store_true", help="Print Rust code")
        p.add_argument(
            "--json", action="store_true", help="Write tracks/<module>/<id>.json and print the track_bake command"
        )
    real_ids = list(provenance_osm_urls().keys())
    p = sub.add_parser("download", help="Download OSM map data for the real circuits in tracks/")
    p.add_argument("--track", choices=real_ids, action="append", help="Circuit id (repeatable; default: all)")
    p.add_argument("--force", action="store_true", help="Download again even if the file exists")
    args = parser.parse_args()

    if args.discipline == "download":
        download(args.track or real_ids, args.cache_dir, args.force)
        return

    specs, process, generate_rust, print_summary = DISCIPLINES[args.discipline]
    track_ids = [args.track] if args.track else list(specs.keys())
    for tid in track_ids:
        data = process(tid, args.cache_dir)
        print_summary(data)
        if args.rust:
            print(generate_rust(data))
            print()
        if args.json:
            path, bake_cmd = write_source_json(args.discipline, data)
            print(f"  Wrote {path}")
            print(f"  Next: {bake_cmd}")


if __name__ == "__main__":
    main()
