//! Authors and bakes authentic Joker Lap branching segments into the 20 World RX circuits.
//!
//! Spec 081: Rallycross Joker Lap Segments for Classic and OpenStreetMap Circuits.
//!
//! ```text
//! python3 scripts/osm_importer.py rally --jokers
//! cargo run --bin build_world_rx_joker -- [tracks/rally] [assets/osm/rx_jokers.json]
//! ```
//! Where OSM maps the joker, the branch follows it (`JokerSource::Osm`); elsewhere it is synthetic.
//!
//! Spec 102: each circuit's old joker is fitted with a branch layout of two junction components (`rx_joker_fit`),
//! which compiles into the network. A circuit that cannot be fitted keeps the legacy network already in its file,
//! untouched. The run writes `docs/circuits/branch_junction_migration.md`.

#[path = "rx_joker_fit/mod.rs"]
mod rx_joker_fit;

use std::collections::HashMap;
use std::path::Path;
use glam::Vec2;
use serde::Deserialize;
use tdrace_core::track::network::{RoadSegment, SegmentId};
use tdrace_core::track::spline::TrackWaypoint;
use tdrace_core::track::Track;
use tdrace_core::SurfaceType;

struct TrackJokerConfig {
    slug: &'static str,
    name: &'static str,
    source: JokerSource,
}

/// Where a track's joker comes from.
enum JokerSource {
    /// The joker ways mapped in OpenStreetMap, from the file `osm_importer.py rally --jokers` writes.
    Osm,
    /// OSM maps no joker: the main line between two waypoints, bulged sideways until it is `target_delta` metres
    /// longer (42 m where nothing describes the real joker).
    Synthetic {
        split_idx: usize,
        merge_idx: usize,
        side: f32, // +1.0 for left normal, -1.0 for right normal
        surface: SurfaceType,
        bank_angle: f32,
        target_delta: f32,
    },
}

/// One mapped joker in track coordinates, from its split node on the lap to its merge node.
#[derive(Deserialize)]
struct OsmJoker {
    points: Vec<[f32; 2]>,
    surfaces: Vec<SurfaceType>,
}

/// Joker points closer than this to the point before them are dropped, so no joker step is a few metres long.
const MIN_JOKER_STEP_M: f32 = 3.0;

/// Spacing of the joker points where it follows the main line to the mapped split and from the mapped merge.
const JOKER_LEAD_STEP_M: f32 = 10.0;

/// The mapped joker ends must lie on the main road (half its 13-14.5 m width), or the joker and the lap are not
/// in one frame. They are not on the centre line: the spline cuts the corners of the OSM lap (estering_rx: 4.7 m).
const MAX_JOKER_END_OFFSET_M: f32 = 6.5;

const WORLD_RX_CONFIGS: &[TrackJokerConfig] = &[
    TrackJokerConfig {
        slug: "holjes_rx",
        name: "Höljes Velodrome Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "lydden_hill",
        name: "Chessons Drift Joker Detour",
        // "Between Pilgrims and Chessons Drift" (waypoints 9-14 and 14-18), 1420 m vs the 1335 m lap: +85 m,
        // +73 m on this 1150 m lap. OSM maps no joker. From waypoint 12 to 16 the bulge left a main wall stub in the
        // split throat that stopped bots on both routes; to 18 it is gentler.
        source: JokerSource::Synthetic {
            split_idx: 12,
            merge_idx: 18,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 6.0,
            target_delta: 73.0,
        },
    },
    TrackJokerConfig {
        slug: "hell_rx",
        name: "Lånkebanen Downhill Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "loheac_rx",
        name: "Lohéac Hairpin Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "estering_rx",
        name: "Estering Turn 1 Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "montalegre_rx",
        name: "Montalegre Stadium Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "nyirad_rx",
        name: "Nyirád Red Cauldron Joker Detour",
        // The joker leaves "on the inside of a turn ... where drivers turn left" (the left at waypoint 45) and
        // rejoins "at a very sharp left hander" (waypoint 49) before the right-hand climb to the finish: 1290 m vs
        // the 1220 m lap, +70 m, +62 m on this 1075 m lap. OSM maps no joker. This section wiggles: on the inside
        // (left) the bulge folds for every split and merge tried, and longer bulges on the right fold, reach the
        // return road, or overlap the asphalt where the main road turns to dirt (wheel surface mismatches). So it
        // stays as it was: waypoints 44-52, +42 m (the fold check puts it on the right).
        source: JokerSource::Synthetic {
            split_idx: 58,
            merge_idx: 68,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 6.0,
            target_delta: 42.0,
        },
    },
    TrackJokerConfig {
        slug: "kouvola_rx",
        name: "Tykkimäki Velodrome Joker Detour",
        // "A joker section towards the end of the lap", 1120 m vs the 1060 m lap: +60 m, +54 m on this 951 m lap.
        // Round the outside of the last hairpin (waypoints 26-28). OSM maps no joker.
        source: JokerSource::Synthetic {
            split_idx: 25,
            merge_idx: 28,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 7.0,
            target_delta: 54.0,
        },
    },
    TrackJokerConfig {
        slug: "catalunya_rx",
        name: "Barcelona Stadium Chicane Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "mettet_rx",
        name: "Mettet Arena Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "lavare_rx",
        name: "Circuit de Lavaré Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "riga_rx",
        name: "Biķernieki Forest Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "killarney_rx",
        name: "Table Mountain Sweep Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "lessay_rx",
        name: "Circuit de Lessay Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "essay_rx",
        name: "Circuit des Ducs La Butte Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "dreux_rx",
        name: "Dreux Switchback Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "croft_rx",
        name: "Croft Infield Joker Detour",
        source: JokerSource::Synthetic {
            split_idx: 16,
            merge_idx: 21,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 6.0,
            target_delta: 42.0,
        },
    },
    TrackJokerConfig {
        slug: "spa_rx",
        name: "Spa Raidillon Crest Joker Detour",
        source: JokerSource::Synthetic {
            split_idx: 16,
            merge_idx: 22,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 7.0,
            target_delta: 42.0,
        },
    },
    TrackJokerConfig {
        slug: "silverstone_rx",
        name: "Silverstone Stowe Joker Detour",
        source: JokerSource::Synthetic {
            split_idx: 9,
            merge_idx: 14,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 6.0,
            target_delta: 42.0,
        },
    },
    TrackJokerConfig {
        slug: "erx_motor_park",
        name: "ERX Clay Bowl Joker Detour",
        source: JokerSource::Synthetic {
            split_idx: 12,
            merge_idx: 20,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 8.0,
            target_delta: 42.0,
        },
    },
];

/// Tightest turn a baked joker may have, in metres. The folds this guards against had radii of 0.0-0.8 m.
const MIN_JOKER_RADIUS_M: f32 = 7.0;

/// Smallest turn radius between consecutive samples of `seg`, in metres. Samples within 2 m of `main`
/// are skipped: there the joker follows the main road, whose OSM geometry has its own tight kinks.
fn min_turn_radius(seg: &RoadSegment, main: &RoadSegment) -> f32 {
    seg.samples
        .windows(2)
        .filter(|w| main.project_point(w[0].point).distance_to_spline > 2.0)
        .filter_map(|w| {
            let angle = w[0].tangent.dot(w[1].tangent).clamp(-1.0, 1.0).acos();
            let ds = w[1].distance - w[0].distance;
            (angle > 1e-4 && ds > 0.0).then(|| ds / angle)
        })
        .fold(f32::INFINITY, f32::min)
}

/// The old joker between the main waypoints where it splits and merges, as the old network builder cut it.
struct JokerCut {
    /// Main waypoint indices of the split and the merge.
    split_idx: usize,
    merge_idx: usize,
    /// Split to merge along the joker; its first and last waypoints are the split and merge waypoints.
    joker: Vec<TrackWaypoint>,
}

fn synthetic_cut(
    cfg: &TrackJokerConfig,
    track: &Track,
    split_idx: usize,
    merge_idx: usize,
    side: f32,
    surface: SurfaceType,
    bank_angle: f32,
    target_delta: f32,
) -> JokerCut {
    let wps = &track.spline.waypoints;
    let (s_idx, m_idx) = (split_idx, merge_idx);

    assert!(s_idx < m_idx && m_idx < wps.len());

    let seg1_wps = wps[s_idx..=m_idx].to_vec();

    let seg1_prelim = RoadSegment::new(SegmentId(1), "Main Racing Line", seg1_wps.clone());
    let main_len = seg1_prelim.length;

    // Offset the main line's smooth samples sideways. Sampling the raw waypoint polyline gave normals that
    // jump at every waypoint, and an offset larger than the bend radius on the inside of a bend folds the
    // road back on itself (loheac_rx, lessay_rx, riga_rx, nyirad_rx, erx_motor_park).
    let total_main_dist = seg1_prelim.length;
    let sample_main = |dist: f32| -> (Vec2, Vec2) {
        let s = seg1_prelim.sample_at_distance(dist.clamp(0.0, total_main_dist));
        (s.point, Vec2::new(-s.tangent.y, s.tangent.x))
    };

    // Number of waypoints along Joker detour
    let num_joker_steps = 16usize;

    // Binary search for peak lateral offset D_peak such that seg2.length - seg1.length == target_delta
    let build_joker_wps = |side_sign: f32| -> Vec<TrackWaypoint> {
        let mut low = 0.0f32;
        let mut high = 150.0f32;
        let mut best_seg2_wps = Vec::new();

        for _ in 0..30 {
            let mid = (low + high) * 0.5;
            let mut candidate_wps = Vec::new();

            for k in 0..=num_joker_steps {
                let t = k as f32 / num_joker_steps as f32;
                let (p, norm) = sample_main(t * total_main_dist);
                let offset = mid * (std::f32::consts::PI * t).sin().powi(2);
                let pt = p + norm * (offset * side_sign);

                let elev = wps[s_idx].elevation + (wps[m_idx].elevation - wps[s_idx].elevation) * t;
                let width = if k == 0 {
                    wps[s_idx].width
                } else if k == num_joker_steps {
                    wps[m_idx].width
                } else {
                    13.0
                };
                let bank = if k == 0 || k == num_joker_steps {
                    0.0
                } else {
                    bank_angle
                };

                let wp = TrackWaypoint::new(pt, width)
                    .with_surface(if k == 0 {
                        wps[s_idx].surface.unwrap_or(SurfaceType::Asphalt)
                    } else if k == num_joker_steps {
                        wps[m_idx].surface.unwrap_or(SurfaceType::Asphalt)
                    } else {
                        surface
                    })
                    .with_elevation(elev)
                    .with_bank_angle(bank);

                candidate_wps.push(wp);
            }

            let test_seg = RoadSegment::new(SegmentId(2), cfg.name, candidate_wps.clone());
            let cur_delta = test_seg.length - main_len;

            if cur_delta < target_delta {
                low = mid;
            } else {
                high = mid;
            }
            best_seg2_wps = candidate_wps;
        }
        best_seg2_wps
    };

    // Keep the configured side unless its joker turns tighter than MIN_JOKER_RADIUS_M; then take the other
    // side if that one is smoother.
    let preferred = build_joker_wps(side);
    let preferred_radius = min_turn_radius(&RoadSegment::new(SegmentId(2), cfg.name, preferred.clone()), &seg1_prelim);
    let best_seg2_wps = if preferred_radius >= MIN_JOKER_RADIUS_M {
        preferred
    } else {
        let other = build_joker_wps(-side);
        let other_radius = min_turn_radius(&RoadSegment::new(SegmentId(2), cfg.name, other.clone()), &seg1_prelim);
        println!(
            "  {} -> side {:+} turns at {:.1} m, side {:+} at {:.1} m",
            cfg.slug, side, preferred_radius, -side, other_radius
        );
        if other_radius > preferred_radius { other } else { preferred }
    };

    JokerCut { split_idx: s_idx, merge_idx: m_idx, joker: best_seg2_wps }
}

/// Fillet sharp vertices along mapped joker route to guarantee corner radii >= target_radius
fn fillet_joker_points(
    points: &[(Vec2, f32, SurfaceType)],
    target_radius: f32,
) -> Vec<(Vec2, f32, SurfaceType)> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }
    let mut out = vec![points[0]];
    for i in 1..n - 1 {
        let p_prev = points[i - 1].0;
        let p_curr = points[i].0;
        let p_next = points[i + 1].0;
        let width = points[i].1;
        let surface = points[i].2;

        let u_vec = p_curr - p_prev;
        let v_vec = p_next - p_curr;
        let lu = u_vec.length();
        let lv = v_vec.length();
        if lu < 1e-3 || lv < 1e-3 {
            out.push(points[i]);
            continue;
        }
        let u = u_vec / lu;
        let v = v_vec / lv;

        let dot = (u.dot(v)).clamp(-1.0, 1.0);
        let defl = dot.acos();
        let deg = defl.to_degrees();
        let det = u.x * v.y - u.y * v.x;

        if deg > 20.0 && det.abs() > 1e-4 {
            let is_left = det > 0.0;
            let norm_in = if is_left {
                Vec2::new(-u.y, u.x)
            } else {
                Vec2::new(u.y, -u.x)
            };
            let half_defl = defl * 0.5;
            let t_des = target_radius * half_defl.tan();
            let t_max = (lu - 3.5).max(3.5).min((lv - 3.5).max(3.5));
            let t = t_des.min(t_max);
            let r_act = t / half_defl.tan().max(1e-4);

            let p_entry = p_curr - u * t;
            let p_exit = p_curr + v * t;
            let center = p_entry + norm_in * r_act;

            let a_in = (p_entry.y - center.y).atan2(p_entry.x - center.x);
            let a_out = (p_exit.y - center.y).atan2(p_exit.x - center.x);
            let mut da = a_out - a_in;
            if is_left && da < 0.0 {
                da += std::f32::consts::TAU;
            } else if !is_left && da > 0.0 {
                da -= std::f32::consts::TAU;
            }

            let a_mid = a_in + da * 0.5;
            let p_apex = center + Vec2::new(a_mid.cos(), a_mid.sin()) * r_act;

            if let Some(prev) = out.last() {
                if prev.0.distance(p_entry) >= 3.0 {
                    out.push((p_entry, width, surface));
                }
            } else {
                out.push((p_entry, width, surface));
            }
            out.push((p_apex, width, surface));
            out.push((p_exit, width, surface));
        } else {
            if let Some(prev) = out.last() {
                if prev.0.distance(p_curr) >= 3.0 {
                    out.push((p_curr, width, surface));
                }
            } else {
                out.push((p_curr, width, surface));
            }
        }
    }
    if let Some(prev) = out.last() {
        if prev.0.distance(points[n - 1].0) >= 3.0 {
            out.push(points[n - 1]);
        }
    } else {
        out.push(points[n - 1]);
    }
    out
}

/// The main line cut where the mapped joker leaves and rejoins it, and the joker through its OSM points.
///
/// The mapped split and merge are where the OSM joker leaves and rejoins the main road, not its end nodes: the
/// holjes_rx joker ends in a lane that runs 3 m beside the main line for ~70 m, and a car on the main road there
/// read as on the joker. The joker splits at the last main waypoint before the mapped split and merges at the first
/// one after the mapped merge, and follows the main line in between. The main route then keeps its waypoints and its exact shape: an
/// extra waypoint at the mapped split moved the essay_rx and dreux_rx main line enough to put bots on a wall.
fn osm_cut(cfg: &TrackJokerConfig, track: &Track, joker: &OsmJoker) -> JokerCut {
    let spline = &track.spline;
    let wps = &spline.waypoints;
    let (first, last) = (Vec2::from(joker.points[0]), Vec2::from(joker.points[joker.points.len() - 1]));
    let (split, merge) = (spline.project_point(first), spline.project_point(last));
    for (end, proj) in [("split", &split), ("merge", &merge)] {
        assert!(
            proj.distance_to_spline < MAX_JOKER_END_OFFSET_M,
            "{}: the mapped joker {} is {:.1} m off the main line",
            cfg.slug,
            end,
            proj.distance_to_spline
        );
    }
    let pts: Vec<Vec2> = joker.points.iter().map(|p| Vec2::from(*p)).collect();
    let off_main = |p: Vec2| {
        let proj = spline.project_point(p);
        proj.distance_to_spline >= proj.track_width * 0.5
    };
    let leave = (1..pts.len() - 1)
        .find(|&i| off_main(pts[i]))
        .unwrap_or_else(|| panic!("{}: the mapped joker never leaves the main road", cfg.slug));
    let rejoin = (1..pts.len() - 1).rev().find(|&i| off_main(pts[i])).unwrap();
    let d_split = spline.project_point(pts[leave - 1]).progress_distance;
    let d_merge = spline.project_point(pts[rejoin + 1]).progress_distance;
    assert!(d_split < d_merge, "{}: the mapped joker crosses the start line", cfg.slug);
    // The spline runs through its waypoints; the start waypoint is at 0 m (a projection could give the lap length).
    let wp_dist = |i: usize| if i == 0 { 0.0 } else { spline.project_point(wps[i].point).progress_distance };
    let s_idx = (1..wps.len()).rev().find(|&i| wp_dist(i) <= d_split).expect("a main waypoint before the split");
    let m_idx = (1..wps.len())
        .find(|&i| wp_dist(i) >= d_merge)
        .unwrap_or_else(|| panic!("{}: the mapped joker merges after the last main waypoint", cfg.slug));

    // Joker points between the two waypoints: the main line up to one step before the mapped split, the OSM points,
    // then the main line from one step after the mapped merge. Leaving out the split and merge points lets the
    // spline round the fork; through them the joker kinked there (holjes_rx walls crossed, catalunya_rx +2 s).
    // (point, width, surface)
    let main_points = |from: f32, to: f32| -> Vec<(Vec2, f32, SurfaceType)> {
        if to <= from {
            return Vec::new();
        }
        let n = ((to - from) / JOKER_LEAD_STEP_M).ceil() as usize;
        (0..=n)
            .map(|k| {
                let p = spline.sample_at_distance(from + (to - from) * k as f32 / n as f32);
                (p.point, p.width, p.surface)
            })
            .collect()
    };
    let mut inner = main_points(wp_dist(s_idx), d_split - JOKER_LEAD_STEP_M);
    inner.extend((leave..=rejoin).map(|i| (pts[i], 13.0, joker.surfaces[i])));
    inner.extend(main_points(d_merge + JOKER_LEAD_STEP_M, wp_dist(m_idx)));
    let mut points: Vec<(Vec2, f32, SurfaceType)> = Vec::new();
    for q in inner {
        let prev = points.last().map_or(wps[s_idx].point, |p| p.0);
        if q.0.distance(prev) >= MIN_JOKER_STEP_M && q.0.distance(wps[m_idx].point) >= MIN_JOKER_STEP_M {
            points.push(q);
        }
    }
    assert!(!points.is_empty(), "{}: the mapped joker has no points between its ends", cfg.slug);
    let points = fillet_joker_points(&points, 11.0);

    let (s_wp, m_wp) = (&wps[s_idx], &wps[m_idx]);
    let steps = points.len() + 1;
    let elevation = |k: usize| s_wp.elevation + (m_wp.elevation - s_wp.elevation) * k as f32 / steps as f32;
    let mut joker_wps = vec![TrackWaypoint::new(s_wp.point, s_wp.width)
        .with_surface(s_wp.surface.unwrap_or(SurfaceType::Asphalt))
        .with_elevation(s_wp.elevation)];
    for (k, (p, width, surface)) in points.iter().enumerate() {
        joker_wps.push(TrackWaypoint::new(*p, *width).with_surface(*surface).with_elevation(elevation(k + 1)));
    }
    joker_wps.push(
        TrackWaypoint::new(m_wp.point, m_wp.width)
            .with_surface(m_wp.surface.unwrap_or(SurfaceType::Asphalt))
            .with_elevation(m_wp.elevation),
    );

    JokerCut { split_idx: s_idx, merge_idx: m_idx, joker: joker_wps }
}

fn build_track_joker(cfg: &TrackJokerConfig, tracks_base_dir: &Path, osm_jokers: &HashMap<String, OsmJoker>) -> rx_joker_fit::Row {
    let path = tracks_base_dir.join(format!("{}.json", cfg.slug));
    println!("Processing {} ({:?})...", cfg.slug, path);

    let mut track = Track::load_from_file(&path)
        .unwrap_or_else(|e| panic!("failed to load {}: {:?}", cfg.slug, e));
    track.checkpoints.retain(|cp| !cp.is_joker);

    let cut = match cfg.source {
        JokerSource::Osm => {
            let joker = osm_jokers.get(cfg.slug).unwrap_or_else(|| panic!("{}: no mapped joker", cfg.slug));
            osm_cut(cfg, &track, joker)
        }
        JokerSource::Synthetic { split_idx, merge_idx, side, surface, bank_angle, target_delta } => {
            synthetic_cut(cfg, &track, split_idx, merge_idx, side, surface, bank_angle, target_delta)
        }
    };

    // Spec 102: build the branch from junction components when the old joker can be fitted. A circuit that cannot be
    // fitted keeps the legacy network already in its file, untouched.
    track.branch_layout = None;
    let outcome = rx_joker_fit::convert(&mut track, cfg.slug, cfg.name, cut.split_idx, cut.merge_idx, &cut.joker);
    if let rx_joker_fit::Outcome::Converted(fit) = &outcome {
        println!("  {} -> converted: deviation {:.2} m, joker costs {:.2} s", cfg.slug, fit.deviation, fit.cost_s);
        track.save_to_file(&path).unwrap_or_else(|e| panic!("failed to save {}: {:?}", cfg.slug, e));
    }
    rx_joker_fit::Row { circuit: cfg.slug.to_string(), outcome }
}

fn main() {
    let tracks_dir = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("tracks/rally"));
    let jokers_path = std::env::args()
        .nth(2)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("assets/osm/rx_jokers.json"));
    let jokers_json = std::fs::read_to_string(&jokers_path)
        .unwrap_or_else(|e| panic!("failed to read {:?}: {}", jokers_path, e));
    let osm_jokers: HashMap<String, OsmJoker> = serde_json::from_str(&jokers_json)
        .unwrap_or_else(|e| panic!("failed to parse {:?}: {}", jokers_path, e));

    println!(
        "Baking authentic Joker Lap TrackNetworks for 20 World Rallycross tracks in {:?}...",
        tracks_dir
    );

    let rows: Vec<rx_joker_fit::Row> = WORLD_RX_CONFIGS.iter().map(|cfg| build_track_joker(cfg, &tracks_dir, &osm_jokers)).collect();
    rx_joker_fit::write_report(Path::new("docs/circuits/branch_junction_migration.md"), &rows);

    println!("All 20 World Rallycross tracks successfully baked with authentic TrackNetworks!");
}
