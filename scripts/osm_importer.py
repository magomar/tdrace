#!/usr/bin/env python3
"""
OSM Circuit Importer for tdrace

One importer for the OpenStreetMap (OSM) circuits of four disciplines:

  gt     18 GT / F1 circuits, scaled to 0.5x the official FIA length
  kart   CIK-FIA kart circuits at 1:1
  rally  World RX rallycross circuits at 1:1 with asphalt/dirt surfaces
  nascar NASCAR ovals and road courses: 1:1 under 3 km, Road America 0.5x, the rest 0.75x,
         with banking per corner

With --json the waypoints go to tracks/<module>/<id>.json (spec 042: official circuits are JSON
only); then run the printed `cargo run --bin track_bake -- ... --rebuild` command.

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
    """Node ids along (way id, first node or None[, last node]) segments; a segment runs to its
    way's last node unless a last node is given, and runs against the way's direction when the
    last node comes before the first one.

    Consecutive segments share their join node, which is kept once; a closing repeat of the first
    node is dropped. Returns [(node id, way id)].
    """
    chain = []
    for wid, first, *last in segments:
        nds = ways[wid]["nodes"]
        i = nds.index(first) if first is not None else 0
        j = nds.index(last[0]) if last else len(nds) - 1
        nds = nds[i:j + 1] if i <= j else nds[j:i + 1][::-1]
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
        # Relation 148194 "Circuit de Monaco" without the pit lane (3324 m in OSM). Its ways leave two
        # small gaps (18 m at Avenue d'Ostende / Casino, 6 m on Avenue de Monte-Carlo): the lap jumps them.
        "segments": [
            (1081401617, 1737389117, 1799099566), (168681959, 1799099566, 477617968), (60767229, 477617968, 25192130),
            (4226740, 25192130, 1074585009), (1019174508, 1074585009, 1868404468), (166399479, 1868404468, 1726583852),
            (1453878835, 1726583852, 6444966511), (1081401616, 6444966511, 1690130866), (157719644, 1690130866, 252387589),
            (254596486, 252387589, 918118157), (1551240830, 12571844664, 14109303023), (1551240829, 14109303023, 21912962),
            (161775592, 21912962, 1204288376), (1082515450, 1204288376, 7568749016), (161752645, 7568749016, 25240075),
            (4229658, 25240075, 21913117), (4230009, 21913117, 1699777574), (4230006, 1699777574, 272637923),
            (434567309, 272637923, 21914841), (166399501, 21914841, 1737114648), (568187257, 1737114648, 21914666),
            (4230007, 21914666, 6485591888), (1148675745, 6485591888, 273246211), (1470365906, 273246211, 13484626647),
            (1470365907, 13484626647, 10687075425), (1148199871, 10687075425, 1685061923), (41929969, 1685061923, 519323656),
            (4230891, 519323656, 11407697331), (1230247123, 11407697331, 21914343), (160004393, 21914343, 6485591919),
            (4229536, 6485591919, 13312466205), (1451501763, 13312466205, 6485591925), (348019480, 6485591925, 2422123083),
            (1081401613, 2422123083, 1737389192), (485746484, 1737389192, 25193217), (503475642, 25193217, 6485591957),
            (1081401614, 6485591957, 25191729), (214636589, 25191729, 2241375226), (1081401615, 2241375226, 25191634),
            (39839529, 25191634, 1737389117),
        ],
        "start_node": "1868404468",  # Boulevard Albert 1er, the start straight
        "fia_length": 3337.0,
        "num_waypoints": 26,
        "default_width": 10.5,
        "straight_width": 11.5,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 3.0,
        "default_laps": 3,
        "tag": "JEWEL IN THE CROWN",
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
        # Relation 284540 "Grand Prix Circuit Without Chicane" (the current F1 layout, 4666 m in OSM);
        # OSM split the old single way 831804327 on 2026-09-20.
        "segments": [
            (831804327, 1300807861, 1644327603), (1560896065, 1644327603, 9760170477), (1560896066, 9760170477, 7765791210),
            (1560896061, 7765791210, 385973423), (893732520, 385973423, 8553774009), (831804325, 8553774009, 7765791228),
            (990483278, 7765791228, 1300807861),
        ],
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
        # Relation 421263 "Marina Bay Street Circuit" without the pit lane: its only closed lap near the
        # official length (4972 m in OSM), anticlockwise. Public-road oneway tags are ignored.
        "segments": [
            (465064165, 5411304502, 1161612993), (686323967, 1161612993, 6432919892), (686335799, 6432919892, 6433028833),
            (686323968, 6433028833, 1446076215), (686335800, 1446076215, 6433028834), (21805129, 6433028834, 462263951),
            (654223306, 462263951, 235070344), (672222982, 235070344, 3076643620), (174262763, 3076643620, 3736885019),
            (383365018, 3736885019, 3736885022), (528537303, 3736885022, 5136090633), (383365019, 5136090633, 4726227141),
            (584419619, 4726227141, 233879899), (176298699, 233879899, 233879872), (479540202, 233879872, 4600854642),
            (633871920, 4600854642, 1832823668), (1415079998, 1832823668, 13003578482), (1415079999, 13003578482, 9946316973),
            (633871919, 9946316973, 5864347585), (479745479, 5864347585, 4281759839), (479745478, 4281759839, 4727947772),
            (649944021, 4727947772, 1720073406), (479540200, 1720073406, 1720073355), (635373148, 1720073355, 5270981855),
            (545362403, 5270981855, 1832823705), (545362402, 1832823705, 5270981856), (723073393, 5270981856, 6782303341),
            (173791997, 6782303341, 595497945), (75067163, 595497945, 233880373), (635375722, 233880373, 1720073397),
            (649336906, 1720073397, 6094193512), (763525862, 6094193512, 3059486893), (1498693105, 3059486893, 233879798),
            (21701074, 233879798, 3059486888), (763525863, 3059486888, 6355011388), (763525864, 6355011388, 233497179),
            (51511352, 233497179, 172510077), (459186771, 172510077, 7759646003), (831044956, 7759646003, 1818164136),
            (16688065, 1818164136, 233879753), (51511339, 233879753, 12071463879), (1303467716, 12071463879, 1398079475),
            (156457347, 1398079475, 232353353), (650084964, 232353353, 4513378919), (634770511, 4513378919, 232353185),
            (750071183, 232353185, 12522242619), (1392512181, 12522242619, 3071267396), (479565615, 3071267396, 232353226),
            (173779754, 232353226, 232376467), (845894319, 232376467, 7891770123), (647084783, 7891770123, 6077839914),
            (649520780, 6077839914, 6095372887), (480250096, 6095372887, 4732525705), (155907160, 4732525705, 232279092),
            (21584500, 232279092, 6997023277), (747875236, 6997023277, 6997023278), (747875237, 6997023278, 232353925),
            (475217889, 232353925, 6856578455), (759511978, 6856578455, 3010901912), (448084962, 3010901912, 6413721257),
            (684544691, 6413721257, 6413721265), (1015882723, 6413721265, 9370421301), (1015882722, 9370421301, 4451336954),
            (1121759130, 4451336954, 4451336947), (475215022, 4451336947, 6988663055), (480247943, 6988663055, 9370421307),
            (29377119, 9370421307, 237054799), (1121759129, 237054799, 9569954749), (1121759131, 9569954749, 2511487801),
            (1206760203, 2511487801, 237012287), (528539591, 237012287, 3010901881), (35037538, 3010901881, 410939418),
            (767388313, 410939418, 1832823650), (750071184, 1832823650, 4726775785), (581766308, 4726775785, 233879421),
            (479605911, 233879421, 4726775783), (479565613, 4726775783, 7138225045), (764062390, 7138225045, 233879440),
            (479565612, 233879440, 1743939385), (634832601, 1743939385, 1764796980), (633311062, 1764796980, 1446076214),
            (764062389, 1446076214, 5270623340), (303314681, 5270623340, 2466494363), (767388315, 2466494363, 1780184888),
            (763531722, 1780184888, 1832823586), (633343709, 1832823586, 5978628576), (633340452, 5978628576, 233879574),
            (479565608, 233879574, 1832823584), (842953325, 1832823584, 1832823583), (633340456, 1832823583, 1832823581),
            (635394837, 1832823581, 1832823579), (180551367, 1832823579, 6340950179), (581942152, 6340950179, 5165925965),
            (173791995, 5165925965, 479983003), (152783801, 479983003, 233879603), (479284866, 233879603, 634071195),
            (303311924, 634071195, 3076643588), (686335806, 3076643588, 1161613127), (686335807, 1161613127, 5411304502),
        ],
        "fia_length": 4940.0,
        "num_waypoints": 30,
        "default_width": 11.5,
        "straight_width": 13.0,
        "barrier": "BarrierType::Steel",
        "barrier_offset": 3.0,
        "default_laps": 3,
        "tag": "SINGAPORE NIGHT RACE",
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

    if "segments" in cfg:
        seg_ways = {int(wid): {"nodes": [int(n) for n in nds]} for wid, nds in ways.items()}
        chain_nodes = [str(nid) for nid, _ in chain_segments(seg_ways, cfg["segments"])]
    elif "way_id" in cfg:
        chain_nodes = ways[cfg["way_id"]]
    else:
        rel = next(r for r in root.findall("relation") if r.get("id") == str(cfg["rel_id"]))

        if cfg.get("rbr_filter"):
            f1_ways = [
                "822592410", "822592403", "822592404", "347958266", "822592398",
                "822592399", "822592400", "822592401", "822592402", "822592405",
                "822592406", "822592407", "822592408", "822592409"
            ]
            chain_nodes = stitch_ways(cid, [ways[wid] for wid in f1_ways if wid in ways], nodes)
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
        "num_waypoints": 80,  # ~20 m spacing keeps the lap within ~3 m of the OSM line
        "elevation_fn": None,
    },
    "valencia_kart": {
        "name": "Kartodromo Internacional Lucas Guerrero (Valencia)",
        "description": "Premier Spanish championship venue in Chiva featuring sweeping esses, technical hairpins, and wide overtaking zones.",
        "ways": [751513226],
        # First node of the way, 143 m into the main straight: the 87 m start grid stays on the straight.
        "start_node_id": 7025550140,
        "fia_length": 1428.0,
        "default_width": 8.5,
        "straight_width": 9.2,
        "num_waypoints": 80,  # ~18 m spacing keeps the lap within ~3 m of the OSM line
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
        # Longest simple loop of the 8 untagged raceway ways (1216 m, the official lap is 1220 m), in the
        # old lap's direction. The web also holds shorter alternative paths; the game has no branching yet.
        "segments": [(172413358, 1833190235, 1833190246), (172413356, 1833190246, 1833190241),
                     (172413357, 1833190241, 1833190239), (172413360, 1833190239, 1833190218),
                     (172413359, 1833190218, 1833190276), (172413357, 1833190276, 1833190231),
                     (172413356, 1833190231, 1833190235)],
        # Start on the long asphalt way 358, clear of the other sections of the web (at the 359/360
        # junction the grid hit a wall of the neighbouring section); 280 m keeps the grid on asphalt
        # with 64 m to the first corner.
        "start_offset_m": 280.0,
        # Clay on 356 and 357 as before; the rest is asphalt (~40% of the lap; docs/circuits/rally.md says 48%).
        "loose_ways": [172413356, 172413357],
        # The hairpins are single sharp nodes where two ways meet (up to 131 deg, one a narrow V):
        # round them to 13 m, a normal rallycross hairpin, so the inner wall (10.25 m from the centre
        # line) stays off the road. The rounding shortens the 1220 m lap to ~1080 m.
        "min_radius_m": 13.0,
        "query": '[out:json][timeout:25];(way["highway"="raceway"](46.963,17.410,46.974,17.428););out body;>;out skel qt;',
        "fia_length": 1220.0,
        "default_width": 13.5,
        "straight_width": 14.5,
        "num_waypoints": 80,  # ~14 m spacing: the 13 m hairpin arcs need several waypoints each
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
        # Relation 11362868 "World RX of Barcelona" without its joker_lap ways (1137 m in OSM). It runs
        # against the F1 oneway tags; the two "unpaved" ways are the loose sections.
        "segments": [
            (967275593, 8553773977, 9719993934), (1560896064, 9719993934, 8553773972), (921317981, 8553773972, 14202158154),
            (1560896060, 14202158154, 7765791210), (1560896061, 7765791210, 385973423), (893732520, 385973423, 8553774009),
            (921317984, 8553774009, 8553774005), (921317983, 8553774005, 1644321708), (1560896062, 1644321708, 7765791151),
            (1560896059, 7765791151, 8553773999), (921317982, 8553773999, 8553773977),
        ],
        "loose_ways": [921317981, 921317982],
        "start_offset_m": 338.0,  # where the old lap started, so its scenery stays in place
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
        # Loose as in the old lap (Mettet RX is ~60/40 tarmac/gravel); OSM tags 178240082 asphalt.
        "loose_ways": [178240082, 178240080],
        "start_offset_m": 68.0,  # where the old lap started, so its scenery stays in place
        "query": '[out:json][timeout:25];(way["highway"="raceway"](50.295,4.640,50.310,4.665););out body;>;out skel qt;',
        # The raceway loop that matches the old lap best (1050 m in OSM, 0.91x the official 1149 m).
        "segments": [
            (178240082, 1886090191, 1886090150), (178384349, 1886090150, 1886090019), (178384323, 1886090019, 1886089974),
            (178384345, 1886089974, 1886089890), (178384322, 1886089890, 1886089916), (178384358, 1886089916, 1886090017),
            (178384335, 1886090017, 1886090080), (178384356, 1886090080, 1886090174), (178384337, 1886090174, 2463534884),
            (178240080, 2463534884, 1886090408), (178240082, 1886090408, 1886090191),
        ],
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
        "start_offset_m": 659.0,  # where the old lap started, so its scenery stays in place
        "query": '[out:json][timeout:25];(way["highway"="raceway"](56.955,24.215,56.975,24.245););out body;>;out skel qt;',
        # The raceway loop that matches the old lap best (1071 m in OSM; the official 1294 m is 17% longer,
        # so the lap stays at the mapped length). The three "gravel" ways are the loose sections.
        "segments": [
            (588947722, 5098958979, 5624243846), (588947720, 5624243846, 5624243875), (588947717, 5624243875, 5624243878),
            (588947719, 5624243878, 5624243876), (588947714, 5624243876, 1080703212), (1435177485, 1080703212, 1080702484),
            (93229455, 1080702484, 1080702566), (1120162743, 1080702566, 279576058), (1120158806, 279576058, 5363424528),
            (945640986, 5363424528, 1080377982), (1120160990, 1080377982, 277946516), (523849729, 277946516, 5098958979),
        ],
        "loose_ways": [588947722, 588947717, 588947719],
        "fia_length": 1071.0,  # mapped length (the official 1294 m is 17% longer)
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
    "croft_rx": {
        "name": "Croft Rallycross Circuit",
        "description": "British Rallycross Championship venue in North Yorkshire: tarmac from Clervaux and Hawthorn, then a long loose-surface loop through the infield back to the pit straight.",
        # From the pit exit: Clervaux, Hawthorn, into the Chicane, then off onto the "Rallycross" way
        # (no surface tag; the tarmac ways are tagged asphalt, so it is taken as the loose section).
        "segments": [(222641546, None), (26261500, None), (26261494, None), (222641550, None, 1241209962),
                     (108510895, None)],
        "loose_ways": [108510895],
        "start_offset_m": 125.0,  # 125 m down the pit straight, so the whole 12-car grid (~103 m) is on tarmac
        # No official rallycross lap length is published; this is the mapped OSM loop (1:1).
        "fia_length": 1251.0,
        "default_width": 13.5,
        "straight_width": 14.5,
        "num_waypoints": 52,  # ~24 m spacing keeps the lap within ~2 m of the OSM line
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
        loose = set(spec.get("loose_ways", []))
        for nid, wid in chain_segments(ways, spec["segments"]):
            surf = "Dirt" if wid in loose else rally_way_surface(ways[wid].get("tags", {}))
            raw_nodes_surf.append((nid, surf))
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
    elif track_id == "kouvola_rx":
        for nid in ways[149713976]["nodes"][36:-1]:
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in ways[149713976]["nodes"][:7]:
            raw_nodes_surf.append((nid, "Asphalt"))
        for nid in ways[149713907]["nodes"][:40]:
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
        if spec.get("min_radius_m"):
            # Scale from the OSM loop itself, then round: the rounding shortens the lap, and scaling
            # after it would stretch the whole circuit back up.
            raw_len = polyline_length(metric_pts, closed=True)
            check_length_ratio(track_id, raw_len, spec["fia_length"])
            f = spec["fia_length"] / raw_len
            metric_pts, surfaces = fillet_corners([(x * f, y * f) for x, y in metric_pts], surfaces, spec["min_radius_m"])
        if spec.get("start_offset_m"):
            metric_pts, surfaces = shift_start(metric_pts, spec["start_offset_m"], surfaces)
        heading = heading_ahead(metric_pts)
    else:
        p_start = metric_pts[0]
        p_ahead = metric_pts[min(6, len(metric_pts) - 1)]
        heading = math.atan2(p_ahead[1] - p_start[1], p_ahead[0] - p_start[0])

    # Rotate so start straight heads along +X
    rotated_pts = rotate_points(metric_pts, heading)

    # Scale to exact FIA homologation length (a rounded lap is already scaled)
    current_len = polyline_length(rotated_pts, closed=True)
    if spec.get("min_radius_m"):
        scale_factor = 1.0
    else:
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


def print_rally_summary(res):
    print(f"=== {res['name']} ({res['id']}) ===")
    print(f"  Waypoints: {len(res['waypoints'])}, Total Length: {res['total_length']} m (FIA Target: {res['fia_length']} m)")
    asphalt_cnt = sum(1 for w in res['waypoints'] if w['surface'] == 'Asphalt')
    dirt_cnt = len(res['waypoints']) - asphalt_cnt
    print(f"  Surface: {asphalt_cnt * 100 // len(res['waypoints'])}% Asphalt, {dirt_cnt * 100 // len(res['waypoints'])}% Dirt")


# ---------------------------------------------------------------------------
# NASCAR ovals and road courses (1:1 under 3 km, 0.75x above; Road America 0.5x)
# ---------------------------------------------------------------------------

# Each lap is a list of (way id, first node or None) segments in race direction (see chain_segments).
# start_node (+ start_offset_m along the lap) is the start/finish line: a mapped start-finish node, or
# the middle of the pit road straight.
# corner_banks lists the banking of each corner in lap order from the start line (one value is
# used for all corners); bend_bank is for gentle bends such as a tri-oval or a dogleg.
NASCAR_TRACKS = {
    "bowman_gray": {
        "name": "Bowman Gray Stadium",
        "segments": [(914237156, None)],
        "start_node": 8492580790,  # start of the south straight; no pit road is mapped
        "start_offset_m": 40.0,  # middle of that straight
        "official_length": 402.0,
        "scale": 1.0,
        "num_waypoints": 16,
        "width": 14.0,
        "straight_bank": 0.0,
        "corner_banks": [0.0],
        "kerbs": True,
    },
    "bristol": {
        "name": "Bristol Motor Speedway",
        "segments": [(116606212, None)],
        "start_node": 1314158509,  # both straights have pit road; the start straight is a guess
        "start_offset_m": 84.0,  # middle of that straight
        "official_length": 858.0,
        "scale": 1.0,
        "num_waypoints": 24,
        "width": 16.0,
        "straight_bank": 10.0,
        "corner_banks": [28.0],
        "surface": "Concrete",
    },
    "martinsville": {
        "name": "Martinsville Speedway",
        "segments": [(448515178, None), (448515177, None), (402168721, None)],
        "start_node": 4045926977,  # middle of the straight before the First Turn Tower grandstand
        "official_length": 847.0,
        "scale": 1.0,
        "num_waypoints": 24,
        "width": 16.0,
        "straight_bank": 0.0,
        "corner_banks": [12.0],
        "corner_surface": "Concrete",
        "kerbs": True,
    },
    "north_wilkesboro": {
        "name": "North Wilkesboro Speedway",
        "segments": [(18928710, None)],
        "start_node": 2313815791,
        "start_offset_m": 58.0,  # middle of the pit road straight
        "official_length": 1006.0,  # the OSM way is 846 m, the inside line
        "scale": 1.0,
        "num_waypoints": 24,
        "width": 16.0,
        "straight_bank": 3.0,
        "corner_banks": [14.0],
        "straight_elevations": [-1.5, 2.0],  # downhill frontstretch, uphill backstretch
        "kerbs": True,
    },
    "irp_oval": {
        "name": "Lucas Oil Indianapolis Raceway Park",
        "segments": [(123830268, None)],
        "start_node": 1379538172,  # mapped motorsport=start-finish node
        "official_length": 1104.0,
        "scale": 1.0,
        "num_waypoints": 24,
        "width": 16.0,
        "straight_bank": 2.0,
        "corner_banks": [12.0],
        "kerbs": True,
    },
    "phoenix": {
        "name": "Phoenix Raceway",
        "segments": [(29333335, None)],
        "start_node": 322700088,
        "start_offset_m": 182.0,  # middle of the pit road straight
        "official_length": 1645.0,
        "scale": 1.0,
        "num_waypoints": 32,
        "width": 18.0,
        "straight_bank": 3.0,
        "corner_banks": [10.0],  # turns 1-2, the dogleg and turns 3-4 (as in the hand-made preset)
        "kerbs": True,
    },
    "darlington": {
        "name": "Darlington Raceway",
        "segments": [(104277971, None)],
        "start_node": 13658651458,
        "start_offset_m": 147.0,  # middle of the pit road straight
        "official_length": 2198.0,
        "scale": 1.0,
        "num_waypoints": 40,
        "width": 18.0,
        "straight_bank": 3.0,
        "corner_banks": [25.0, 23.0],  # turns 1-2, turns 3-4
    },
    "charlotte": {
        "name": "Charlotte Motor Speedway",
        # (min_lon, min_lat, max_lon, max_lat) of the lap: the osm_url way alone does not cover it for download
        "bbox": (-80.686, 35.347, -80.679, 35.356),
        "segments": [(396483278, None), (116034341, None), (402168709, None), (1052107104, None), (402168711, None)],
        "start_node": 9668619996,  # mapped raceway=start-finish node
        "official_length": 2414.0,
        "scale": 1.0,
        "num_waypoints": 40,
        "width": 20.0,
        "straight_bank": 5.0,
        "corner_banks": [24.0],
        "bend_bank": 5.0,  # quad-oval frontstretch
    },
    "chicago": {
        "name": "Chicago Street Course",
        # Relation 16546690 lists its ways against the race direction; this is the list reversed.
        "segments": [(w, None) for w in (
            435561738, 435559503, 316907584, 435558482, 435558479, 25026604, 314943087, 255623675, 435559507,
            621319020, 1377694368, 313874094, 313874095, 1377697560, 5010709, 372671686, 772541923, 33116710,
            23888140, 435657099, 25026653, 435559504, 435559500, 435559499, 231237737, 435559505, 518573327,
            435559501, 231237526, 435558483, 435558481, 25026606, 1227037106, 235741029, 435561742, 90707299,
            1287253578, 435561319, 235714536)],
        "start_node": 6061506190,  # Columbus Drive, halfway between Roosevelt Road and Balbo Drive
        "official_length": 3541.0,
        "scale": 0.75,
        "num_waypoints": 64,  # ~45 m spacing keeps the lap within ~7 m of the OSM line
        "width": 13.0,
        "straight_bank": 0.0,
        "corner_banks": [0.0],
        "kerbs": True,
    },
    "watkins_glen": {
        "name": "Watkins Glen International",
        # NASCAR short course: the Inner Loop, then the Short Course cut past the Boot.
        "segments": [(293208067, None), (20163576, None), (293208063, None), (293208062, None), (293208064, None),
                     (293208074, None), (428026652, None), (293208056, None), (293208060, None), (293208070, None),
                     (293208065, None), (293208075, None)],
        "start_node": 2967820339,  # pit entry on the front straight
        "start_offset_m": 400.0,  # middle of pit road, ~170 m before the Ninety
        "official_length": 3943.0,
        "scale": 0.75,
        "num_waypoints": 64,  # ~45 m spacing keeps the lap within ~7 m of the OSM line
        "width": 16.0,
        "straight_bank": 0.0,
        "corner_banks": [0.0],
        "kerbs": True,
    },
    "daytona": {
        "name": "Daytona International Speedway",
        "segments": [(11371365, None), (352067004, None), (352070311, None), (352070309, None), (352070316, None)],
        "start_node": 101134873,
        "start_offset_m": 12.0,  # tri-oval, level with the middle of pit road
        "official_length": 4023.0,
        "scale": 0.75,
        "num_waypoints": 40,
        "width": 22.0,
        "straight_bank": 3.0,
        "corner_banks": [31.0],
        "bend_bank": 18.0,  # tri-oval
    },
    "indianapolis": {
        "name": "Indianapolis Motor Speedway",
        "segments": [(588780351, None), (588780349, None), (51308226, None), (588780333, None), (588780334, None),
                     (588780335, None), (588780332, None), (588780343, None), (588780345, None), (588780347, None)],
        "start_node": 654509391,  # nearest lap node to the painted IMS start-finish line
        "official_length": 4023.0,
        "scale": 0.75,
        "num_waypoints": 48,
        "width": 20.0,
        "straight_bank": 0.0,
        "corner_banks": [9.2],
    },
    "pocono": {
        "name": "Pocono Raceway",
        "segments": [(109767460, None)],
        "start_node": 1255320153,
        "start_offset_m": 436.0,  # middle of the pit road straight
        "official_length": 4023.0,
        "scale": 0.75,
        "num_waypoints": 40,
        "width": 18.0,
        "straight_bank": 0.0,
        "corner_banks": [14.0, 8.0, 6.0],  # turn 1, Tunnel Turn, turn 3
        "kerbs": True,
    },
    "talladega": {
        "name": "Talladega Superspeedway",
        # (min_lon, min_lat, max_lon, max_lat) of the lap: the osm_url way alone does not cover it for download
        "bbox": (-86.072, 33.559, -86.060, 33.575),
        "segments": [(426163860, None), (426163859, None), (532106116, None), (8835825, None)],
        "start_node": 13757090309,  # nearest lap node to the mapped raceway=start-finish node
        "official_length": 4281.0,
        "scale": 0.75,
        "num_waypoints": 40,
        "width": 24.0,
        "straight_bank": 3.0,
        "corner_banks": [33.0],
        "bend_bank": 16.5,  # tri-oval
    },
    "eldora": {
        "name": "Eldora Speedway",
        "segments": [(608397609, None)],
        "start_node": 5764072235,  # mapped raceway=start-finish node
        "official_length": 805.0,  # the OSM way is 686 m, the inside line
        "scale": 1.0,
        "num_waypoints": 16,
        "width": 18.0,
        "straight_bank": 8.0,
        "corner_banks": [24.0],
        "surface": "Dirt",
    },
    "iowa": {
        "name": "Iowa Speedway",
        "segments": [(119238784, None)],
        "start_node": 1340393901,
        "start_offset_m": 41.0,  # frontstretch, level with the middle of pit road
        "official_length": 1408.0,
        "scale": 1.0,
        "num_waypoints": 24,
        "width": 20.0,
        "straight_bank": 4.0,
        "corner_banks": [14.0],
        "bend_bank": 10.0,  # frontstretch dogleg
    },
    "road_america": {
        "name": "Road America",
        # Relation 6432758 ("Road America Circuit"), in race order from the main straight.
        "segments": [(w, None) for w in (
            122090289, 122090268, 122090299, 122090258, 122090260, 122090240, 122090275, 122090272, 122090284,
            122090298, 110527567, 122090265, 122090279, 122090248, 122090267, 122090239, 122090291, 122090276,
            122090286, 122090295, 122090252, 122090243, 122090251, 122090282, 122090297, 122090270, 122090263)],
        "start_node": 1262000220,
        "start_offset_m": 257.0,  # main straight, level with the middle of the pit lane
        "official_length": 6515.0,
        "scale": 0.5,
        "num_waypoints": 80,  # ~41 m spacing keeps the lap within ~5 m of the OSM line
        "width": 14.0,
        "straight_bank": 0.0,
        "corner_banks": [0.0],
        "kerbs": True,
    },
}

# Curvature classes, relative to the tightest waypoint of the lap.
NASCAR_CORNER_CURVATURE = 0.6
NASCAR_BEND_CURVATURE = 0.12


def shift_start(points, offset_m, props=None):
    """Closed polyline starting `offset_m` meters further along the lap (a new first point is inserted).

    `props` (optional, one value per point, as in resample_polyline) is rotated the same way;
    then (points, props) is returned.
    """
    n = len(points)
    left = offset_m
    for i in range(n):
        a, b = points[i], points[(i + 1) % n]
        seg = math.hypot(b[0] - a[0], b[1] - a[1])
        if left <= seg:
            t = left / seg if seg > 0 else 0.0
            p = (a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1]))
            shifted = [p] + points[i + 1:] + points[:i + 1]
            if props is None:
                return shifted
            return shifted, [props[i]] + props[i + 1:] + props[:i + 1]
        left -= seg
    return points if props is None else (points, props)


def point_along(pts, i, dist, step):
    """(index of the segment start, point) `dist` m along a closed polyline from vertex i (dist < 0: backwards)."""
    n = len(pts)
    j = i
    left = abs(dist)
    while True:
        k = (j + step) % n
        seg = math.dist(pts[j], pts[k])
        if left <= seg:
            f = left / seg
            p = (pts[j][0] + f * (pts[k][0] - pts[j][0]), pts[j][1] + f * (pts[k][1] - pts[j][1]))
            return (j if step > 0 else k), p
        left -= seg
        j = k


def round_hairpin(pts, props, i, radius, arc_step=2.0):
    """Replace the narrow V hairpin at vertex i with a half circle between the first points on each
    leg that are 2 * `radius` apart, bulging towards the apex."""
    n = len(pts)
    dist = radius
    while True:
        ja, a = point_along(pts, i, -dist, -1)
        jb, b = point_along(pts, i, dist, 1)
        if math.dist(a, b) >= 2 * radius or dist > polyline_length(pts) / 4:
            break
        dist += 1.0
    centre = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
    r = math.dist(a, b) / 2
    start = math.atan2(a[1] - centre[1], a[0] - centre[0])
    apex = math.atan2(pts[i][1] - centre[1], pts[i][0] - centre[0])
    sweep = math.pi if math.cos(start + math.pi / 2 - apex) > 0 else -math.pi  # pass the apex side
    m = max(4, round(r * math.pi / arc_step))
    curve = [(centre[0] + r * math.cos(start + sweep * q / m), centre[1] + r * math.sin(start + sweep * q / m))
             for q in range(m + 1)]
    curve_props = [props[ja]] * (m // 2 + 1) + [props[jb]] * (m - m // 2)
    removed = {q % n for q in range(ja + 1, jb + 1 if jb > ja else jb + n + 1)}
    out_pts, out_props, placed = [], [], False
    for q in range(n):
        if q in removed:
            if not placed:
                out_pts += curve
                out_props += curve_props
                placed = True
            continue
        out_pts.append(pts[q])
        out_props.append(props[q])
    return out_pts, out_props


def fillet_corners(points, props, radius, min_turn_deg=20.0, arc_step=2.0):
    """Closed polyline with every node turning more than `min_turn_deg` replaced by a circular arc
    of `radius` tangent to its two legs (a road-design fillet). When a leg is too short for the
    arc, the corner takes in the next leg too. OSM draws some corners as one sharp node where two
    ways meet; the spline would turn on a few metres there and the walls would fold into the road.
    `props` has one value per point (its outgoing segment). Returns (points, props)."""
    pts, props = list(points), list(props)
    done = [False] * len(pts)

    def turn(i):
        n = len(pts)
        a, b, c = pts[i - 1], pts[i], pts[(i + 1) % n]
        h = math.atan2(c[1] - b[1], c[0] - b[0]) - math.atan2(b[1] - a[1], b[0] - a[0])
        return abs((h + math.pi) % (2 * math.pi) - math.pi)

    while True:
        n = len(pts)
        todo = [i for i in range(n) if not done[i] and turn(i) > math.radians(min_turn_deg)]
        if not todo:
            return pts, props
        i = max(todo, key=turn)
        a, b = i - 1, i  # incoming and outgoing segment: pts[a] -> pts[a+1], pts[b] -> pts[b+1]
        while True:
            a0, a1 = pts[a % n], pts[(a + 1) % n]
            b0, b1 = pts[b % n], pts[(b + 1) % n]
            la, lb = math.dist(a0, a1), math.dist(b0, b1)
            u = ((a1[0] - a0[0]) / la, (a1[1] - a0[1]) / la)
            v = ((b1[0] - b0[0]) / lb, (b1[1] - b0[1]) / lb)
            cross = u[0] * v[1] - u[1] * v[0]
            if abs(cross) < 1e-6 or b - a > n // 3:
                break
            s_x = ((b0[0] - a0[0]) * v[1] - (b0[1] - a0[1]) * v[0]) / cross  # corner X = a0 + s_x * u
            x = (a0[0] + s_x * u[0], a0[1] + s_x * u[1])
            theta = math.acos(max(-1.0, min(1.0, u[0] * v[0] + u[1] * v[1])))
            t = radius * math.tan(theta / 2)
            s1 = s_x - t  # tangent point on the incoming line, from a0
            s2 = (x[0] - b0[0]) * v[0] + (x[1] - b0[1]) * v[1] + t  # tangent point on the outgoing line, from b0
            if s1 < 0:
                a -= 1
            elif s2 > lb:
                b += 1
            else:
                break
        done[i] = True
        if abs(cross) < 1e-6 or b - a > n // 3:
            # a narrow V hairpin: its legs run back almost parallel, so no arc of `radius` fits them
            pts, props = round_hairpin(pts, props, i, radius, arc_step)
            done = [False] * len(pts)  # indices moved; the curve itself turns less than min_turn_deg per point
            continue
        side = 1.0 if cross > 0 else -1.0
        t1 = (a0[0] + s1 * u[0], a0[1] + s1 * u[1])
        centre = (t1[0] - side * u[1] * radius, t1[1] + side * u[0] * radius)
        start = math.atan2(t1[1] - centre[1], t1[0] - centre[0])
        m = max(2, round(radius * theta / arc_step))
        arc = [(centre[0] + radius * math.cos(start + side * theta * q / m),
                centre[1] + radius * math.sin(start + side * theta * q / m)) for q in range(m + 1)]
        arc_props = [props[a % n]] * (m // 2 + 1) + [props[b % n]] * (m - m // 2)
        removed = {q % n for q in range(a + 1, b + 1)}
        at = (a + 1) % n
        new_pts, new_props, new_done = [], [], []
        for q in range(n):
            if q == at:
                new_pts += arc
                new_props += arc_props
                new_done += [True] * len(arc)
            if q not in removed:
                new_pts.append(pts[q])
                new_props.append(props[q])
                new_done.append(done[q])
        pts, props, done = new_pts, new_props, new_done


def curvature_classes(points):
    """(class, signed smoothed curvature) per closed-loop point; class is corner, bend or straight."""
    n = len(points)
    turn = []
    for i in range(n):
        a, b, c = points[i - 1], points[i], points[(i + 1) % n]
        h1 = math.atan2(b[1] - a[1], b[0] - a[0])
        h2 = math.atan2(c[1] - b[1], c[0] - b[0])
        dh = (h2 - h1 + math.pi) % (2 * math.pi) - math.pi
        turn.append(dh / (0.5 * (math.dist(a, b) + math.dist(b, c))))
    smooth = [(turn[i - 1] + 2 * turn[i] + turn[(i + 1) % n]) / 4 for i in range(n)]
    k_max = max(abs(k) for k in smooth) or 1.0
    classes = []
    for k in smooth:
        r = abs(k) / k_max
        classes.append("corner" if r >= NASCAR_CORNER_CURVATURE else "bend" if r >= NASCAR_BEND_CURVATURE else "straight")
    return classes, smooth


def runs_of(classes, wanted):
    """Index of each run of `wanted` classes in lap order (-1 elsewhere); a run through waypoint 0 is one run."""
    n = len(classes)
    run = [-1] * n
    if all(c in wanted for c in classes):
        return [0] * n, 1
    start = next(i for i in range(n) if classes[i] not in wanted)  # begin outside a run
    count = -1
    for k in range(1, n + 1):
        i = (start + k) % n
        if classes[i] in wanted:
            if classes[(i - 1) % n] not in wanted:
                count += 1
            run[i] = count
    # renumber so that the first run met from waypoint 0 is run 0
    order = []
    for i in range(n):
        if run[i] >= 0 and run[i] not in order:
            order.append(run[i])
    return [order.index(r) if r >= 0 else -1 for r in run], len(order)


def process_nascar_track(track_id, cache_dir):
    spec = NASCAR_TRACKS[track_id]
    data = load_osm_elements(cache_dir, track_id)
    nodes = {e["id"]: (e["lat"], e["lon"]) for e in data["elements"] if e["type"] == "node"}
    ways = {e["id"]: e for e in data["elements"] if e["type"] == "way"}

    node_ids = [nid for nid, _ in chain_segments(ways, spec["segments"])]
    k = node_ids.index(spec["start_node"])
    node_ids = node_ids[k:] + node_ids[:k]
    check_loop_joins(track_id, node_ids, nodes, way_edges(w["nodes"] for w in ways.values()))

    lat0 = sum(nodes[n][0] for n in node_ids) / len(node_ids)
    lon0 = sum(nodes[n][1] for n in node_ids) / len(node_ids)
    metric_pts = [latlon_to_meters(*nodes[n], lat0, lon0) for n in node_ids]
    if spec.get("start_offset_m"):
        metric_pts = shift_start(metric_pts, spec["start_offset_m"])

    raw_len = polyline_length(metric_pts, closed=True)
    check_length_ratio(track_id, raw_len, spec["official_length"])
    target_len = spec["official_length"] * spec["scale"]
    factor = target_len / raw_len
    scaled = [(x * factor, y * factor) for x, y in metric_pts]
    x0, y0 = scaled[0]
    scaled = [(x - x0, y - y0) for x, y in scaled]

    resampled, _, _ = resample_polyline(scaled, spec["num_waypoints"])
    # Start line along +X, from the first waypoint to the next one.
    pts = rotate_points(resampled, math.atan2(resampled[1][1], resampled[1][0]))

    classes, curv = curvature_classes(pts)
    corner_run, n_corners = runs_of(classes, ("corner",))
    banks = spec["corner_banks"]
    if len(banks) > 1 and len(banks) != n_corners:
        raise SystemExit(f"[{track_id}] {n_corners} corners found, but corner_banks has {len(banks)} values")
    straight_run, n_straights = runs_of(classes, ("straight", "bend"))
    elevs = spec.get("straight_elevations")
    if elevs and len(elevs) != n_straights:
        raise SystemExit(f"[{track_id}] {n_straights} straights found, but straight_elevations has {len(elevs)} values")

    waypoints = []
    for i, (x, y) in enumerate(pts):
        cls = classes[i]
        if cls == "corner":
            bank = banks[corner_run[i]] if len(banks) > 1 else banks[0]
        elif cls == "bend":
            bank = spec.get("bend_bank", spec["straight_bank"])
        else:
            bank = spec["straight_bank"]
        surface = spec.get("corner_surface") if cls == "corner" else None
        surface = surface or spec.get("surface")
        kerb = spec.get("kerbs", False) and cls == "corner"
        waypoints.append({
            "x": round(x, 1),
            "y": round(y, 1),
            "width": spec["width"],
            "bank": bank,
            "elevation": elevs[straight_run[i]] if elevs and straight_run[i] >= 0 else 0.0,
            "surface": surface,
            "left_curb": kerb and curv[i] > 0,
            "right_curb": kerb and curv[i] < 0,
        })

    return {
        "id": track_id,
        "name": spec["name"],
        "waypoints": waypoints,
        "raw_length": round(raw_len, 1),
        "target_length": round(target_len, 1),
        "official_length": spec["official_length"],
        "scale": spec["scale"],
        "corners": n_corners,
    }


def print_nascar_summary(res):
    print(f"=== {res['name']} ({res['id']}) ===")
    print(f"  Waypoints: {len(res['waypoints'])}, target {res['target_length']} m "
          f"({res['scale']}x of {res['official_length']} m; raw OSM loop {res['raw_length']} m), "
          f"{res['corners']} corners")


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
    """(south, west, north, east) from a kart, NASCAR or rallycross config, when it has one."""
    for specs in (KART_TRACKS, NASCAR_TRACKS):
        if "bbox" in specs.get(track_id, {}):
            min_lon, min_lat, max_lon, max_lat = specs[track_id]["bbox"]
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
    if w.get("bank"):
        wp["bank_angle"] = round(w["bank"], 1)
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
        if data.get("scale") and data["scale"] != 1.0:
            track["scale"] = f"{data['scale']:g}x"
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
    "gt": (GT_CIRCUITS, process_gt_circuit, print_gt_summary),
    "kart": (KART_TRACKS, process_kart_track, print_kart_summary),
    "rally": (RALLY_TRACKS, process_rally_track, print_rally_summary),
    "nascar": (NASCAR_TRACKS, process_nascar_track, print_nascar_summary),
}


def main():
    parser = argparse.ArgumentParser(description="Extract and generate tdrace circuits from OpenStreetMap")
    parser.add_argument("--cache-dir", default=DEFAULT_CACHE_DIR, help="OSM cache directory (default: assets/osm)")
    sub = parser.add_subparsers(dest="discipline", required=True)
    for name, (specs, _, _) in DISCIPLINES.items():
        p = sub.add_parser(name, help=f"{name} circuits")
        p.add_argument("--track", choices=list(specs.keys()), help="Process one circuit (default: all)")
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

    specs, process, print_summary = DISCIPLINES[args.discipline]
    track_ids = [args.track] if args.track else list(specs.keys())
    for tid in track_ids:
        data = process(tid, args.cache_dir)
        print_summary(data)
        if args.json:
            path, bake_cmd = write_source_json(args.discipline, data)
            print(f"  Wrote {path}")
            print(f"  Next: {bake_cmd}")


if __name__ == "__main__":
    main()
