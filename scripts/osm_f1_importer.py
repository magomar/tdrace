#!/usr/bin/env python3
"""
OSM F1 Track Importer for tdrace

Extracts real-world motorsport raceway waypoints from OpenStreetMap (OSM),
projects them to metric 2D Cartesian coordinates, scales them to exactly 0.5x
official FIA homologation lengths, aligns the start/finish straight with the +X axis,
adds apex curbs, elevation for bridges/hills, and generates ready-to-use Rust track
definitions for crates/tdrace-app/src/module/f1.rs.
"""

import math
import os
import xml.etree.ElementTree as ET

CACHE_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "target", "osm_cache")

CIRCUIT_CONFIGS = {
    "monza": {
        "name": "Monza Autodromo Nazionale",
        "description": "High-speed Italian Grand Prix temple of speed.",
        "file": "monza.osm",
        "rel_id": 284565,
        "fia_length": 5793.0,
        "num_waypoints": 28,
        "default_width": 13.5,
        "straight_width": 15.0,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 4.5,
        "default_laps": 4,
        "tag": "TEMPLE OF SPEED",
    },
    "spa": {
        "name": "Circuit de Spa-Francorchamps",
        "description": "Belgian Ardennes rollercoaster featuring Eau Rouge and Pouhon.",
        "file": "spa.osm",
        "rel_id": 284560,
        "fia_length": 7004.0,
        "num_waypoints": 32,
        "default_width": 14.0,
        "straight_width": 15.0,
        "barrier": "BarrierType::Armco",
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
        "file": "silverstone.osm",
        "rel_id": 51160,
        "fia_length": 5891.0,
        "num_waypoints": 30,
        "default_width": 13.5,
        "straight_width": 14.5,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 4.0,
        "default_laps": 4,
        "tag": "HOME OF BRITISH MOTORSPORT",
    },
    "monaco": {
        "name": "Circuit de Monaco",
        "description": "Legendary Monte Carlo street circuit with Loews Hairpin, Tunnel, and Swimming Pool.",
        "file": "monaco.osm",
        "rel_id": 148194,
        "fia_length": 3337.0,
        "num_waypoints": 26,
        "default_width": 10.5,
        "straight_width": 11.5,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 3.0,
        "default_laps": 6,
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
        "file": "suzuka.osm",
        "rel_id": 284570,
        "fia_length": 5807.0,
        "num_waypoints": 34,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 3.8,
        "default_laps": 4,
        "tag": "JAPANESE FIGURE-8",
        "crossover": True,
    },
    "interlagos": {
        "name": "Autodromo Jose Carlos Pace (Interlagos)",
        "description": "Thrilling anti-clockwise Brazilian Grand Prix circuit with Senna 'S', Ferradura, and Juncao.",
        "file": "interlagos.osm",
        "rel_id": 6781071,
        "fia_length": 4309.0,
        "num_waypoints": 28,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 3.5,
        "default_laps": 5,
        "tag": "BRAZILIAN ROLLERCOASTER",
    },
    "montreal": {
        "name": "Circuit Gilles Villeneuve (Montreal)",
        "description": "High-speed Canadian island circuit featuring Virage Senna, L'Epingle hairpin, and Wall of Champions.",
        "file": "montreal.osm",
        "rel_id": 284595,
        "fia_length": 4361.0,
        "num_waypoints": 28,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Concrete",
        "barrier_offset": 3.0,
        "default_laps": 5,
        "tag": "ILE NOTRE-DAME",
    },
    "red_bull_ring": {
        "name": "Red Bull Ring (Spielberg)",
        "description": "High-speed Austrian alpine circuit with steep uphill climbs and heavy downhill braking into Remus.",
        "file": "red_bull_ring.osm",
        "rel_id": 5309181,
        "fia_length": 4318.0,
        "num_waypoints": 26,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 3.5,
        "default_laps": 5,
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
        "file": "catalunya.osm",
        "way_id": "831804327",
        "fia_length": 4657.0,
        "num_waypoints": 28,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 3.5,
        "default_laps": 5,
        "tag": "SPANISH GP BENCHMARK",
        "elevations": {
            # Turn 9 Campsa uphill crest
            15: 2.0, 16: 3.5, 17: 2.0,
        },
    },
    "zandvoort": {
        "name": "Circuit Zandvoort",
        "description": "Dune rollercoaster in the Netherlands featuring 18-degree banked corners at Hugenholtz and Arie Luyendyk.",
        "file": "zandvoort.osm",
        "rel_id": 13545573,
        "fia_length": 4259.0,
        "num_waypoints": 28,
        "default_width": 12.5,
        "straight_width": 14.0,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 3.0,
        "default_laps": 5,
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
        "file": "bahrain.osm",
        "rel_id": 284538,
        "fia_length": 5412.0,
        "num_waypoints": 30,
        "default_width": 13.5,
        "straight_width": 15.0,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 3.8,
        "default_laps": 4,
        "tag": "DESERT GRAND PRIX",
    },
    "marina_bay": {
        "name": "Marina Bay Street Circuit (Singapore)",
        "description": "High-intensity Singapore night race through the dazzling city streets and harbor waterfront.",
        "file": "marina_bay_rel.osm",
        "rel_id": 421263,
        "fia_length": 4940.0,
        "num_waypoints": 30,
        "default_width": 11.5,
        "straight_width": 13.0,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 3.0,
        "default_laps": 4,
        "tag": "SINGAPORE NIGHT RACE",
        "marina_filter": True,
    },
    "cota": {
        "name": "Circuit of the Americas (COTA)",
        "description": "Austin Texas spectacle with steep uphill Turn 1 blind crest, Maggotts-inspired Esses, and multi-apex carousel.",
        "file": "cota_rel.osm",
        "rel_id": 6537729,
        "fia_length": 5513.0,
        "num_waypoints": 30,
        "default_width": 13.5,
        "straight_width": 15.0,
        "barrier": "BarrierType::Armco",
        "barrier_offset": 4.0,
        "default_laps": 4,
        "tag": "AUSTIN SPECTACLE",
        "elevations": {
            # Steep Turn 1 uphill crest
            2: 2.0, 3: 4.5, 4: 2.5,
        },
    },
    "madring": {
        "name": "MadRing Circuito de Madrid",
        "description": "Spanish Grand Prix hybrid street circuit navigating the IFEMA complex and Valdebebas avenues.",
        "file": "madring.osm",
        "rel_id": 18813472,
        "fia_length": 5474.0,
        "num_waypoints": 30,
        "default_width": 13.0,
        "straight_width": 14.5,
        "barrier": "BarrierType::Concrete",
        "barrier_offset": 3.5,
        "default_laps": 4,
        "tag": "SPANISH STREET GP",
        "madring_filter": True,
    },
}


def latlon_to_meters(lat, lon, lat0, lon0):
    r = 6378137.0
    x = (math.radians(lon) - math.radians(lon0)) * math.cos(math.radians(lat0)) * r
    y = (math.radians(lat) - math.radians(lat0)) * r
    return x, y


def dist(p1, p2):
    return math.hypot(p2[0] - p1[0], p2[1] - p1[1])


def stitch_ways(way_nodes_list, nodes_coords, max_gap=45.0):
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
            d_start = dist(curr_end_pt, nodes_coords[w[0]])
            d_end = dist(curr_end_pt, nodes_coords[w[-1]])
            if d_start < min_d:
                min_d = d_start
                best_idx = idx
                best_rev = False
            if d_end < min_d:
                min_d = d_end
                best_idx = idx
                best_rev = True

        if min_d > max_gap:
            break

        next_w = remaining.pop(best_idx)
        if best_rev:
            next_w.reverse()

        if next_w[0] == curr_end:
            chain.extend(next_w[1:])
        else:
            chain.extend(next_w)

    return chain


def resample_polyline(points, target_count):
    n = len(points)
    cum_dists = [0.0]
    for i in range(n):
        p0 = points[i]
        p1 = points[(i + 1) % n]
        cum_dists.append(cum_dists[-1] + dist(p0, p1))

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


def process_circuit(cid):
    cfg = CIRCUIT_CONFIGS[cid]
    path = os.path.join(CACHE_DIR, cfg["file"])
    root = ET.parse(path).getroot()

    nodes = {n.get("id"): (float(n.get("lat")), float(n.get("lon"))) for n in root.findall("node")}
    ways = {w.get("id"): [nd.get("ref") for nd in w.findall("nd")] for w in root.findall("way")}

    if "way_id" in cfg:
        chain_nodes = ways[cfg["way_id"]]
    else:
        rel = [r for r in root.findall("relation") if r.get("id") == str(cfg["rel_id"])][0]

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
            chain_nodes = stitch_ways([ways[wid] for wid in ordered], nodes)
        elif cfg.get("rbr_filter"):
            f1_ways = [
                "822592410", "822592403", "822592404", "347958266", "822592398",
                "822592399", "822592400", "822592401", "822592402", "822592405",
                "822592406", "822592407", "822592408", "822592409"
            ]
            chain_nodes = stitch_ways([ways[wid] for wid in f1_ways if wid in ways], nodes)
        elif cfg.get("marina_filter"):
            way_members = [m.get("ref") for m in rel.findall("member") if m.get("type") == "way" and m.get("role") != "pit_lane" and m.get("ref") in ways]
            filtered = []
            for wid in way_members:
                w_elem = [w for w in root.findall("way") if w.get("id") == wid][0]
                tags = {t.get("k"): t.get("v") for t in w_elem.findall("tag")}
                name = tags.get("name", "")
                highway = tags.get("highway", "")
                if "pit" not in name.lower() and highway != "service":
                    filtered.append(wid)
            seen = set()
            ordered = [w for w in filtered if not (w in seen or seen.add(w))]
            chain_nodes = stitch_ways([ways[wid] for wid in ordered], nodes)
        elif cfg.get("madring_filter"):
            way_members = [
                m.get("ref") for m in rel.findall("member")
                if m.get("type") == "way" and m.get("ref") != "1552567031" and m.get("ref") in ways
            ]
            chain_nodes = stitch_ways([ways[wid] for wid in way_members], nodes)
        else:
            w_ids = [m.get("ref") for m in rel.findall("member") if m.get("type") == "way" and m.get("role") != "pit_lane" and m.get("ref") in ways]
            seen = set()
            ordered = [w for w in w_ids if not (w in seen or seen.add(w))]
            chain_nodes = stitch_ways([ways[wid] for wid in ordered], nodes)

    lat0 = sum(nodes[n][0] for n in chain_nodes) / len(chain_nodes)
    lon0 = sum(nodes[n][1] for n in chain_nodes) / len(chain_nodes)
    metric_pts = [latlon_to_meters(nodes[n][0], nodes[n][1], lat0, lon0) for n in chain_nodes]

    # Find longest straight section near start/finish for alignment
    p_start = metric_pts[0]
    p_ahead = metric_pts[min(6, len(metric_pts) - 1)]
    heading = math.atan2(p_ahead[1] - p_start[1], p_ahead[0] - p_start[0])

    cos_a = math.cos(-heading)
    sin_a = math.sin(-heading)
    rotated_pts = []
    for x, y in metric_pts:
        rx = x * cos_a - y * sin_a
        ry = x * sin_a + y * cos_a
        rotated_pts.append((rx, ry))

    measured_len = sum(dist(rotated_pts[i], rotated_pts[(i + 1) % len(rotated_pts)]) for i in range(len(rotated_pts)))
    target_half_len = cfg["fia_length"] * 0.5
    scale_factor = target_half_len / measured_len if measured_len > 0 else 1.0

    scaled_pts = [(x * scale_factor, y * scale_factor) for x, y in rotated_pts]

    x0, y0 = scaled_pts[0]
    aligned_pts = [(x - x0, y - y0) for x, y in scaled_pts]

    resampled, final_len = resample_polyline(aligned_pts, cfg["num_waypoints"])

    # Start line alignment along +X
    dx = resampled[1][0] - resampled[0][0]
    dy = resampled[1][1] - resampled[0][1]
    fine_heading = math.atan2(dy, dx)
    cos_f = math.cos(-fine_heading)
    sin_f = math.sin(-fine_heading)
    final_pts = []
    for x, y in resampled:
        rx = x * cos_f - y * sin_f
        ry = x * sin_f + y * cos_f
        final_pts.append((rx, ry))

    fx0, fy0 = final_pts[0]
    final_pts = [(x - fx0, y - fy0) for x, y in final_pts]

    # Suzuka start straight orientation: align using approach vector to ensure grid slots lie on Y=0
    if cfg.get("crossover"):
        # The main straight on Suzuka runs from waypoint index 31 through 0 into 1
        dx_s = final_pts[0][0] - final_pts[-3][0]
        dy_s = final_pts[0][1] - final_pts[-3][1]
        straight_head = math.atan2(dy_s, dx_s)
        cos_s = math.cos(-straight_head)
        sin_s = math.sin(-straight_head)
        suzuka_pts = []
        for x, y in final_pts:
            rx = x * cos_s - y * sin_s
            ry = x * sin_s + y * cos_s
            suzuka_pts.append((rx, ry))
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
        p_prev = final_pts[(i - 1 + n) % n]
        p_next = final_pts[(i + 1) % n]

        v1 = (x - p_prev[0], y - p_prev[1])
        v2 = (p_next[0] - x, p_next[1] - y)
        cross = v1[0] * v2[1] - v1[1] * v2[0]

        is_straight = abs(cross) < 18.0 and (i < 3 or i > n - 3)
        width = cfg["straight_width"] if is_straight else cfg["default_width"]

        left_curb = False
        right_curb = False
        if cross > 45.0:
            left_curb = True
        elif cross < -45.0:
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


def generate_rust_code(cdata):
    cid = cdata["id"]
    func_name = f"track_{cid}"
    lines = []
    lines.append(f"    /// {cdata['name']}: {cdata['description']}")
    lines.append(f"    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: {cdata['final_len']:.1f}m (Real FIA: {cdata['fia_length']:.0f}m).")
    lines.append(f"    pub fn {func_name}() -> Track {{")
    lines.append("        let waypoints = vec![")

    for w in cdata["waypoints"]:
        curb_str = ""
        if w["left_curb"] or w["right_curb"]:
            curb_str = f".with_curbs({str(w['left_curb']).lower()}, {str(w['right_curb']).lower()})"
        elev_str = ""
        if w["elevation"] > 0.0:
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
    lines.append(f"        let (left_walls, right_walls, left_poly, right_poly) =")
    lines.append(f"            generate_walls_from_spline(&spline, {cdata['barrier_offset']:.1f}, {cdata['barrier']});")
    lines.append("")
    lines.append("        let checkpoints = generate_checkpoints(&spline, 20, 3);")
    lines.append("        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);")
    lines.append("")
    lines.append("        Track {")
    lines.append(f'            name: "{cdata["name"]}".to_string(),')
    lines.append(f'            description: "{cdata["description"]}".to_string(),')
    lines.append("            category: TrackCategory::Main,")
    lines.append("            spline,")
    lines.append("            geometry: TrackGeometry {")
    lines.append("                inner_walls: left_walls,")
    lines.append("                outer_walls: right_walls,")
    lines.append("                obstacles: Vec::new(),")
    lines.append("                surface_zones: Vec::new(),")
    lines.append("                jump_ramps: Vec::new(),")
    lines.append("                left_boundary_polyline: left_poly,")
    lines.append("                right_boundary_polyline: right_poly,")
    lines.append("            },")
    lines.append("            checkpoints,")
    lines.append("            grid_positions: starting_grid,")
    lines.append("            default_surface: SurfaceType::Grass,")
    lines.append("            pit_box_area: None,")
    lines.append(f"            default_laps: {cdata['default_laps']},")
    lines.append('            predefined_car: Some("f1_car".to_string()),')
    lines.append('            module_id: Some("f1".to_string()),')
    lines.append('            modules: vec!["f1".to_string()],')
    lines.append("        }")
    lines.append("    }")
    return "\n".join(lines)


def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--circuit", help="Specific circuit ID")
    parser.add_argument("--rust", action="store_true", help="Print Rust code")
    args = parser.parse_args()

    cids = [args.circuit] if args.circuit else list(CIRCUIT_CONFIGS.keys())
    for cid in cids:
        data = process_circuit(cid)
        if args.rust:
            print(generate_rust_code(data))
            print()
        else:
            print(f"[{cid:14}] {data['name'][:35]:35} | {len(data['waypoints'])} waypoints | {data['final_len']:6.1f}m (target {data['half_length']:.1f}m, 0.5x FIA)")


if __name__ == "__main__":
    main()
