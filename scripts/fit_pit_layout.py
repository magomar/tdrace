#!/usr/bin/env python3
"""Fits spec 101 pit lane layouts to the free-form GT pit lanes.

See specs/101_parametric_pit_lane_kit_blocks_on_spline_circuits.md, Pillar VIII. For each circuit that has a
free-form `pit_lane` and no `pit_lane_layout`, the script builds candidate layouts, bakes them with `track_bake`
(the Rust compile is the only judge of the guards), measures how far each compiled lane is from the old lane, and
keeps the candidate with the smallest deviation. A circuit converts only if that deviation is <= 3.0 m.

Deviation: the old centreline is sampled every 1 m and each point is measured to the new lane; the new centreline
is sampled every 1 m and each point that lies beside the old lane (its projection falls inside the old lane, not on
an end) is measured to the old lane. The largest distance is the deviation. The new lane starts on the main road
inside its entry junction and so is longer than the old lane; that extra part has no old counterpart and is not
counted.

Usage:
    python3 scripts/fit_pit_layout.py [--dry-run] [--tracks-dir tracks] [--bake-bin target/release/track_bake]
                                      [--jobs N] [--report docs/circuits/pit_layout_migration.md] [circuit ...]

Without --dry-run the script adds `pit_lane_layout` to each converted circuit file and writes the report. Then
re-bake the converted circuits with `track_bake <files> --rebuild` two times. With --dry-run nothing is written and
the report goes to stdout.
"""

import argparse
import copy
import glob
import json
import math
import os
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

MAX_DEVIATION = 3.0
# A junction ends where the old lane first reaches the divider gap, less this tolerance (m).
GAP_REACHED_TOLERANCE = 0.1
# Old waypoints closer than this to a junction span, or nearer the track than the gap, are left out of the road (m).
ROAD_MARGIN = 10.0
ROAD_GAP_TOLERANCE = 0.2
BOX_STOP_RADIUS = 3.0
# Room kept between a junction free end and the nearest stall edge (m). At the entry a car crosses the gate at up
# to the pit speed limit and must stop at the first stall: on cota a 3 m gap let bots roll past stall 0.
BOX_ENTRY_ROOM = 20.0
BOX_ROAD_MARGIN = 2.0
DEFAULT_ROAD_WIDTH = 7.0


def load_json(path):
    with open(path) as fh:
        return json.load(fh)


def dist(a, b):
    return math.hypot(a[0] - b[0], a[1] - b[1])


class Polyline:
    """Nearest-point queries on a polyline, with a uniform grid over its segments."""

    CELL = 25.0

    def __init__(self, points, closed=False):
        self.points = [tuple(p) for p in points]
        self.closed = closed
        n = len(self.points)
        self.segments = [(i, (i + 1) % n) for i in range(n if closed else n - 1)]
        self.cum = [0.0]
        for i, j in self.segments:
            self.cum.append(self.cum[-1] + dist(self.points[i], self.points[j]))
        self.length = self.cum[-1]
        self.grid = {}
        for k, (i, j) in enumerate(self.segments):
            a, b = self.points[i], self.points[j]
            for cx in range(math.floor(min(a[0], b[0]) / self.CELL), math.floor(max(a[0], b[0]) / self.CELL) + 1):
                for cy in range(math.floor(min(a[1], b[1]) / self.CELL), math.floor(max(a[1], b[1]) / self.CELL) + 1):
                    self.grid.setdefault((cx, cy), []).append(k)

    def project(self, p):
        """Returns (distance, arc length, segment index, t on segment)."""
        cx, cy = math.floor(p[0] / self.CELL), math.floor(p[1] / self.CELL)
        best = None
        for ring in range(200):
            cand = set()
            for dx in range(-ring, ring + 1):
                for dy in range(-ring, ring + 1):
                    if max(abs(dx), abs(dy)) == ring:
                        cand.update(self.grid.get((cx + dx, cy + dy), ()))
            for k in cand:
                hit = self._project_segment(k, p)
                if best is None or hit[0] < best[0]:
                    best = hit
            # Any segment outside the rings seen so far is at least ring * CELL away.
            if best is not None and best[0] <= ring * self.CELL:
                return best
        return best if best is not None else min(self._project_segment(k, p) for k in range(len(self.segments)))

    def _project_segment(self, k, p):
        i, j = self.segments[k]
        a, b = self.points[i], self.points[j]
        dx, dy = b[0] - a[0], b[1] - a[1]
        l2 = dx * dx + dy * dy
        t = 0.0 if l2 < 1e-12 else max(0.0, min(1.0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2))
        c = (a[0] + t * dx, a[1] + t * dy)
        return (dist(p, c), self.cum[k] + t * math.sqrt(l2), k, t)

    def resample(self, step=1.0):
        out = []
        s = 0.0
        while s <= self.length:
            out.append(self.at(s))
            s += step
        return out

    def at(self, s):
        k = max(0, min(len(self.segments) - 1, _bisect(self.cum, s) - 1))
        i, j = self.segments[k]
        seg = self.cum[k + 1] - self.cum[k]
        t = 0.0 if seg < 1e-9 else (s - self.cum[k]) / seg
        a, b = self.points[i], self.points[j]
        return (a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1]))


def _bisect(arr, x):
    lo, hi = 0, len(arr)
    while lo < hi:
        mid = (lo + hi) // 2
        if arr[mid] <= x:
            lo = mid + 1
        else:
            hi = mid
    return lo


class Main:
    """Main spline from the circuit's baked samples."""

    def __init__(self, circuit):
        sp = circuit["spline"]
        self.samples = sp["samples"]
        self.closed = sp["closed"]
        self.total = sp["total_length"]
        self.line = Polyline([s["point"] for s in self.samples], closed=self.closed)

    def frame(self, p):
        """Returns (main s, signed lateral offset (+ left), half width, tangent) at the point nearest `p`."""
        _, _, k, t = self.line.project(p)
        i, j = self.line.segments[k]
        a, b = self.samples[i], self.samples[j]
        s = a["distance"] + t * self.line.cum[k + 1] - t * self.line.cum[k]
        c = (a["point"][0] + t * (b["point"][0] - a["point"][0]), a["point"][1] + t * (b["point"][1] - a["point"][1]))
        n = a["normal"]
        lat = (p[0] - c[0]) * n[0] + (p[1] - c[1]) * n[1]
        half_w = 0.5 * (a["width"] + t * (b["width"] - a["width"]))
        return s, lat, half_w, a["tangent"]

    def ahead(self, s_from, s_to):
        """Distance from s_from forward to s_to along the driving direction."""
        d = s_to - s_from
        return d % self.total if self.closed else d


def heading_angle_deg(u, v):
    return abs(math.degrees(math.atan2(u[0] * v[1] - u[1] * v[0], u[0] * v[0] + u[1] * v[1])))


def turnoff_angle(main, pts, from_start):
    """Largest angle between the old lane and the main tangent over its first (or last) 30 m."""
    seq = pts if from_start else list(reversed(pts))
    best, travelled = 0.0, 0.0
    for a, b in zip(seq, seq[1:]):  # noqa: RUF007 - itertools.pairwise needs 3.10; system python3 is 3.9
        step = dist(a, b)
        if step < 1e-6:
            continue
        travelled += step
        d = ((b[0] - a[0]) / step, (b[1] - a[1]) / step)
        if not from_start:
            d = (-d[0], -d[1])
        best = max(best, heading_angle_deg(d, main.frame(a)[3]))
        if travelled > 30.0:
            break
    return float(min(59.0, max(10.0, round(best))))


def span(kind, length):
    if kind == "Taper":
        return length
    theta = math.radians(kind["TurnOff"]["angle_deg"])
    return length / theta * math.sin(theta)


def deviation(old_pts, new_pts):
    old = Polyline(old_pts)
    new = Polyline(new_pts)
    worst = max(new.project(p)[0] for p in old.resample(1.0))
    last = len(old.segments) - 1
    for p in new.resample(1.0):
        d, _, k, t = old.project(p)
        if (k == 0 and t <= 0.0) or (k == last and t >= 1.0):
            continue
        worst = max(worst, d)
    return worst


def fit_inputs(circuit):
    """Measures the old lane: side, gap, where it reaches the gap, angles, boxes."""
    main = Main(circuit)
    lane = circuit["pit_lane"]
    road_width = lane.get("road_width", DEFAULT_ROAD_WIDTH)
    old_pts = [tuple(s["point"]) for s in lane["spline"]["samples"]]
    frames = [main.frame(p) for p in old_pts]
    left = sum(1 for f in frames if f[1] > 0)
    side = "Left" if left > len(frames) / 2 else "Right"
    sign = 1.0 if side == "Left" else -1.0
    gaps = [f[1] * sign - f[2] - road_width / 2 for f in frames]
    old_lane = Polyline(old_pts)
    boxes = lane["pit_boxes"]
    box_s = [old_lane.project(b["position"])[1] for b in boxes]
    spacing = (box_s[-1] - box_s[0]) / (len(box_s) - 1) if len(box_s) > 1 else 12.0
    # The divider gap is the narrowest old gap along the box stretch, so both junctions end outside it.
    lane_s = [old_lane.project(p)[1] for p in old_pts]
    stretch = [g for g, s in zip(gaps, lane_s) if box_s[0] - BOX_STOP_RADIUS <= s <= box_s[-1] + BOX_STOP_RADIUS]
    raw_gap = min(stretch or gaps)
    gap = max(0.0, min(10.0, raw_gap))
    reached = [i for i, g in enumerate(gaps) if g >= gap - GAP_REACHED_TOLERANCE]
    first, last = (reached[0], reached[-1]) if reached else (0, len(frames) - 1)
    return {
        "main": main,
        "old_pts": old_pts,
        "old_waypoints": [tuple(w["point"]) for w in lane["spline"]["waypoints"]],
        "side": side,
        "sign": sign,
        "road_width": road_width,
        "speed_limit": lane.get("speed_limit", 16.67),
        "half_width": frames[first][2],
        "s_gap_start": frames[first][0],
        "s_gap_end": frames[last][0],
        "angle_in": turnoff_angle(main, old_pts, True),
        "angle_out": turnoff_angle(main, old_pts, False),
        "gap": round(gap, 2),
        "raw_gap": raw_gap,
        "boxes": [tuple(b["position"]) for b in boxes],
        "box_count": len(boxes),
        "box_spacing": round(spacing, 3),
    }


def lateral_model(kind, length, offset_d, half_w, road_width, x):
    """Sideways offset of a junction centreline at distance x from its anchor, going away from the main track.

    Uses the main track's own frame, so it is exact for a Taper and close for a TurnOff where the main is straight.
    Returns None where x lies before the anchor.
    """
    if x < 0.0:
        return None
    if kind == "Taper":
        u = min(1.0, x / length)
        return offset_d * u * u * (3.0 - 2.0 * u)
    theta = math.radians(kind["TurnOff"]["angle_deg"])
    r = length / theta
    d0 = offset_d - r * (1.0 - math.cos(theta))
    if x >= r * math.sin(theta):
        return offset_d
    return d0 + r * (1.0 - math.cos(math.asin(x / r)))


def fit_junction(fit, entry):
    """Ranks junction shapes for one end by how well their sideways profile matches the old lane.

    Returns [(error, kind, length, anchor s)], best first. The free end sits where the old lane reaches the gap,
    shifted by up to +-10 m.
    """
    main = fit["main"]
    sign = fit["sign"]
    gap_s = fit["s_gap_start"] if entry else fit["s_gap_end"]
    pts = fit["old_pts"]
    profile = []
    for p in pts:
        s, lat, _, _ = main.frame(p)
        d = main.ahead(gap_s, s) if not entry else -main.ahead(s, gap_s)
        if main.closed and abs(d) > main.total / 2:
            d = d - math.copysign(main.total, d)
        profile.append((d if not entry else -d, lat * sign))
    # profile: (distance from the gap point toward the junction anchor, sideways offset). Keep the junction side.
    profile = [(x, lat) for x, lat in profile if x >= -15.0]
    offset_d = fit["half_width"] + fit["gap"] + fit["road_width"] / 2
    theta_in = fit["angle_in"] if entry else fit["angle_out"]
    options = [("Taper", float(length)) for length in range(10, 205, 5)]
    theta = math.radians(theta_in)
    rise_per_m = (1.0 - math.cos(theta)) / theta
    lo, hi = fit["gap"] + fit["road_width"], offset_d
    for frac in (0.05, 0.25, 0.5, 0.75, 0.95):
        options.append(({"TurnOff": {"angle_deg": theta_in}}, round((lo + frac * (hi - lo)) / rise_per_m, 2)))
    box_ss = [main.frame(b)[0] for b in fit["boxes"]]
    ranked = []
    for kind, length in options:
        sp = span(kind, length)
        for shift in (-10.0, -5.0, 0.0, 5.0, 10.0):
            # x measured from the anchor toward the free end; the free end is at the gap point + shift.
            err = 0.0
            for dist_from_gap, lat in profile:
                x = sp + shift - dist_from_gap
                model = lateral_model(kind, length, offset_d, fit["half_width"], fit["road_width"], min(x, sp))
                if model is None:
                    start = lateral_model(kind, length, offset_d, fit["half_width"], fit["road_width"], 0.0)
                    e = math.hypot(x, lat - start)
                else:
                    e = abs(lat - model)
                err = max(err, e)
            anchor = gap_s - (sp + shift) if entry else gap_s + (sp + shift)
            if main.closed:
                anchor %= main.total
            # The junction must leave the box stretch to the road (guard 8).
            if entry:
                room = main.ahead((anchor + sp) % main.total if main.closed else anchor + sp, box_ss[0])
            else:
                room = main.ahead(box_ss[-1], (anchor - sp) % main.total if main.closed else anchor - sp)
            if main.closed and room > main.total / 2:
                room -= main.total
            if room < BOX_STOP_RADIUS + (BOX_ENTRY_ROOM if entry else BOX_ROAD_MARGIN):
                continue
            ranked.append((err, kind, length, anchor))
    ranked.sort(key=lambda r: r[0])
    # Keep the best shift per shape, so the top picks differ in shape.
    seen, best = set(), []
    for r in ranked:
        key = (json.dumps(r[1]), r[2])
        if key not in seen:
            seen.add(key)
            best.append(r)
    return best


def candidate_layouts(fit, per_end=8):
    main = fit["main"]
    entries = fit_junction(fit, True)[:per_end]
    exits = fit_junction(fit, False)[:per_end]
    out = []
    for _, kin, len_in, s_entry in entries:
        for _, kout, len_out, s_exit in exits:
            sp_in, sp_out = span(kin, len_in), span(kout, len_out)
            road = []
            for w in fit["old_waypoints"]:
                ws, lat, half_w, _ = main.frame(w)
                rel = main.ahead(s_entry, ws)
                inside = sp_in + ROAD_MARGIN < rel < main.ahead(s_entry, s_exit) - sp_out - ROAD_MARGIN
                wide = lat * fit["sign"] - half_w - fit["road_width"] / 2 >= fit["gap"] - ROAD_GAP_TOLERANCE
                if inside and wide:
                    road.append([round(w[0], 3), round(w[1], 3)])
            start_s = main.ahead(s_entry, main.frame(fit["boxes"][0])[0])
            out.append({
                "side": fit["side"],
                "entry": {"s": round(s_entry, 3), "kind": kin, "length": len_in, "divider_gap": fit["gap"]},
                "exit": {"s": round(s_exit, 3), "kind": kout, "length": len_out, "divider_gap": fit["gap"]},
                "road_waypoints": road,
                "road_width": fit["road_width"],
                "speed_limit": fit["speed_limit"],
                "box_row": {
                    "start_s": round(start_s, 3),
                    "count": fit["box_count"],
                    "spacing": fit["box_spacing"],
                    "garages": True,
                },
            })
    return out


def bake_all(bake_bin, files, jobs):
    """Bakes files in parallel groups. Returns {file: error text or None}."""
    groups = [files[i::jobs] for i in range(jobs) if files[i::jobs]]

    def run(group):
        proc = subprocess.run([bake_bin, *group, "--rebuild"], capture_output=True, text=True, check=False)
        result = {}
        for f in group:
            errs = [line[len(f) + 2:] for line in proc.stderr.splitlines() if line.startswith(f + ": ")]
            ok = any(line.startswith(f + ": baked") for line in proc.stdout.splitlines())
            result[f] = None if ok else ("; ".join(errs) or "not baked")
        return result

    out = {}
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        for part in pool.map(run, groups):
            out.update(part)
    return out


def kind_name(kind):
    return "Taper" if kind == "Taper" else "TurnOff {:.0f}°".format(kind["TurnOff"]["angle_deg"])


def short_reason(err):
    for key in ("pit lane layout: ", "Error "):
        if key in err:
            return err.split(key, 1)[1][:160]
    return err[:160]


def fit_circuits(paths, bake_bin, jobs, workdir):
    circuits = {}
    files = []
    owner = {}
    for path in paths:
        circuit = load_json(path)
        cid = os.path.splitext(os.path.basename(path))[0]
        fit = fit_inputs(circuit)
        cands = candidate_layouts(fit)
        circuits[cid] = {"path": path, "circuit": circuit, "fit": fit, "cands": cands}
        for k, layout in enumerate(cands):
            data = copy.deepcopy(circuit)
            data["pit_lane_layout"] = layout
            f = os.path.join(workdir, f"{cid}__{k:03d}.json")
            with open(f, "w") as fh:
                json.dump(data, fh)
            files.append(f)
            owner[f] = (cid, k)

    results = bake_all(bake_bin, files, jobs)

    best = {}
    for f, err in results.items():
        cid, k = owner[f]
        entry = circuits[cid]
        if err is not None:
            entry.setdefault("errors", []).append((k, short_reason(err)))
            continue
        baked = load_json(f)
        new_pts = [tuple(s["point"]) for s in baked["pit_lane"]["spline"]["samples"]]
        dev = deviation(entry["fit"]["old_pts"], new_pts)
        if cid not in best or dev < best[cid][0]:
            best[cid] = (dev, k, new_pts)

    # Refine the box row start on the best lane, then bake that layout once more.
    finals = {}
    for cid, (dev, k, new_pts) in best.items():
        layout = copy.deepcopy(circuits[cid]["cands"][k])
        lane = Polyline(new_pts)
        layout["box_row"]["start_s"] = round(lane.project(circuits[cid]["fit"]["boxes"][0])[1], 3)
        data = copy.deepcopy(circuits[cid]["circuit"])
        data["pit_lane_layout"] = layout
        f = os.path.join(workdir, f"{cid}__final.json")
        with open(f, "w") as fh:
            json.dump(data, fh)
        finals[f] = (cid, layout, k, dev)
    refined = bake_all(bake_bin, list(finals), jobs)

    rows = []
    for cid in sorted(circuits):
        entry = circuits[cid]
        row = {"id": cid, "path": entry["path"], "layout": None, "deviation": None, "reason": ""}
        if entry["fit"]["raw_gap"] < 0.0:
            row["reason"] = "old lane overlaps the main road by {:.1f} m at the boxes".format(-entry["fit"]["raw_gap"])
            if cid not in best:
                rows.append(row)
                continue
        if cid not in best:
            if not entry["cands"]:
                row["reason"] = "no junction shape fits between the old lane ends and the boxes"
            else:
                errors = sorted(entry.get("errors", []))
                row["reason"] = "no candidate passed the guards; best fit: " + (errors[0][1] if errors else "unknown")
            rows.append(row)
            continue
        f = next(f for f, v in finals.items() if v[0] == cid)
        _, layout, k, dev = finals[f]
        if refined[f] is None:
            baked = load_json(f)
            dev = deviation(entry["fit"]["old_pts"], [tuple(s["point"]) for s in baked["pit_lane"]["spline"]["samples"]])
        else:
            layout = entry["cands"][k]
        row["deviation"] = dev
        row["layout"] = layout
        if dev > MAX_DEVIATION and not row["reason"]:
            row["reason"] = f"best deviation {dev:.2f} m > {MAX_DEVIATION:.1f} m"
        rows.append(row)
    return rows


def report(rows):
    lines = [
        "# Pit Lane Layout Migration (Spec 101)",
        "",
        "Generated by `scripts/fit_pit_layout.py`. A circuit converts when a candidate layout passes every guard",
        f"(the `track_bake` compile) and its compiled lane stays within {MAX_DEVIATION:.1f} m of the old lane.",
        "Deviation is measured along the old lane only; see the script header.",
        "",
        "| Circuit | Result | Entry | Exit | Length in / out (m) | Divider gap (m) | Max deviation (m) | Reason |",
        "|---------|--------|-------|------|-----------:|----------------:|------------------:|--------|",
    ]
    for r in rows:
        lay = r["layout"]
        converted = lay is not None and not r["reason"]
        lines.append("| {} | {} | {} | {} | {} | {} | {} | {} |".format(
            r["id"],
            "converted" if converted else "kept free-form",
            kind_name(lay["entry"]["kind"]) if lay else "-",
            kind_name(lay["exit"]["kind"]) if lay else "-",
            "{:.0f} / {:.0f}".format(lay["entry"]["length"], lay["exit"]["length"]) if lay else "-",
            "{:.2f}".format(lay["entry"]["divider_gap"]) if lay else "-",
            "{:.2f}".format(r["deviation"]) if r["deviation"] is not None else "-",
            r["reason"].replace("|", "/") or "-",
        ))
    n = sum(1 for r in rows if r["layout"] is not None and not r["reason"])
    lines += ["", f"{n} of {len(rows)} circuits converted.", ""]
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("circuits", nargs="*", help="circuit ids under <tracks-dir>/gt (default: all with a pit lane)")
    ap.add_argument("--dry-run", action="store_true", help="write nothing; print the report")
    ap.add_argument("--tracks-dir", default="tracks")
    ap.add_argument("--bake-bin", default="target/release/track_bake")
    ap.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 2) // 2))
    ap.add_argument("--report", default="docs/circuits/pit_layout_migration.md")
    args = ap.parse_args()

    if not os.path.exists(args.bake_bin):
        sys.exit(f"{args.bake_bin} not found; run: cargo build --release -p tdrace-app --bin track_bake")
    paths = []
    for path in sorted(glob.glob(os.path.join(args.tracks_dir, "gt", "*.json"))):
        cid = os.path.splitext(os.path.basename(path))[0]
        if args.circuits and cid not in args.circuits:
            continue
        data = load_json(path)
        if data.get("pit_lane") and not data.get("pit_lane_layout"):
            paths.append(path)
    if not paths:
        sys.exit(f"no circuit with a free-form pit lane found under {args.tracks_dir}/gt")

    with tempfile.TemporaryDirectory(prefix="fit_pit_layout_") as workdir:
        rows = fit_circuits(paths, args.bake_bin, args.jobs, workdir)
    text = report(rows)

    if args.dry_run:
        print(text)
        return
    for r in rows:
        if r["layout"] is None or r["reason"]:
            continue
        data = load_json(r["path"])
        data["pit_lane_layout"] = r["layout"]
        with open(r["path"], "w") as fh:
            json.dump(data, fh, indent=2)
            fh.write("\n")
    os.makedirs(os.path.dirname(args.report), exist_ok=True)
    with open(args.report, "w") as fh:
        fh.write(text)
    print(text)
    print("Now re-bake the converted circuits two times: track_bake <files> --rebuild")


if __name__ == "__main__":
    main()
