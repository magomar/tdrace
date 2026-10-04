//! Authors and bakes authentic Joker Lap branching segments into the 20 World RX circuits.
//!
//! Spec 081: Rallycross Joker Lap Segments for Classic and OpenStreetMap Circuits.
//!
//! ```text
//! python3 scripts/osm_importer.py rally --jokers
//! cargo run --bin build_world_rx_joker -- [tracks/rally] [assets/osm/rx_jokers.json]
//! ```
//! Where OSM maps the joker, the branch follows it (`JokerSource::Osm`); elsewhere it is synthetic.

use std::collections::HashMap;
use std::path::Path;
use glam::Vec2;
use serde::Deserialize;
use tdrace_core::track::checkpoint::Checkpoint;
use tdrace_core::track::geometry::{BarrierType, LineSegment};
use tdrace_core::track::network::{
    GoreConfig, JunctionId, MergeConfig, RoadJunction, RoadSegment, SegmentId, SocketId,
    SplineSocket, TrackLayout, TrackNetwork,
};
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
    /// OSM maps no joker: the main line between two waypoints, bulged sideways until it is 42 m longer.
    Synthetic {
        split_idx: usize,
        merge_idx: usize,
        side: f32, // +1.0 for left normal, -1.0 for right normal
        surface: SurfaceType,
        bank_angle: f32,
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
        source: JokerSource::Synthetic {
            split_idx: 12,
            merge_idx: 16,
            side: 1.0,
            surface: SurfaceType::Gravel,
            bank_angle: 6.0,
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
        source: JokerSource::Synthetic {
            split_idx: 44,
            merge_idx: 52,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 6.0,
        },
    },
    TrackJokerConfig {
        slug: "kouvola_rx",
        name: "Tykkimäki Velodrome Joker Detour",
        source: JokerSource::Synthetic {
            split_idx: 20,
            merge_idx: 26,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 7.0,
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
        source: JokerSource::Synthetic {
            split_idx: 14,
            merge_idx: 19,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 6.0,
        },
    },
    TrackJokerConfig {
        slug: "lavare_rx",
        name: "Circuit de Lavaré Joker Detour",
        source: JokerSource::Osm,
    },
    TrackJokerConfig {
        slug: "riga_rx",
        name: "Biķernieki Forest Joker Detour",
        source: JokerSource::Synthetic {
            split_idx: 19,
            merge_idx: 24,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 6.0,
        },
    },
    TrackJokerConfig {
        slug: "killarney_rx",
        name: "Table Mountain Sweep Joker Detour",
        source: JokerSource::Synthetic {
            split_idx: 17,
            merge_idx: 22,
            side: 1.0,
            surface: SurfaceType::Gravel,
            bank_angle: 6.0,
        },
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
            surface: SurfaceType::Gravel,
            bank_angle: 6.0,
        },
    },
    TrackJokerConfig {
        slug: "spa_rx",
        name: "Spa Raidillon Crest Joker Detour",
        source: JokerSource::Synthetic {
            split_idx: 16,
            merge_idx: 22,
            side: 1.0,
            surface: SurfaceType::Gravel,
            bank_angle: 7.0,
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
        },
    },
    TrackJokerConfig {
        slug: "erx_motor_park",
        name: "ERX Clay Bowl Joker Detour",
        source: JokerSource::Synthetic {
            split_idx: 13,
            merge_idx: 19,
            side: 1.0,
            surface: SurfaceType::Dirt,
            bank_angle: 8.0,
        },
    },
];

/// Tightest turn a baked joker may have, in metres. The folds this guards against had radii of 0.0-0.8 m.
const MIN_JOKER_RADIUS_M: f32 = 3.0;

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

/// The main line cut at the joker's split and merge, and the joker between them.
struct JokerCut {
    /// Start line to split (segment 0); its last waypoint is the split.
    start: Vec<TrackWaypoint>,
    /// Split to merge along the main line (segment 1).
    main: Vec<TrackWaypoint>,
    /// Split to merge along the joker (segment 2).
    joker: Vec<TrackWaypoint>,
    /// Merge back to the start line (segment 3); its first waypoint is the merge.
    finish: Vec<TrackWaypoint>,
    t_split: Vec2,
    t_merge: Vec2,
    /// Joker road surface where it leaves and where it rejoins the main line.
    joker_surfaces: (SurfaceType, SurfaceType),
}

fn synthetic_cut(
    cfg: &TrackJokerConfig,
    track: &Track,
    split_idx: usize,
    merge_idx: usize,
    side: f32,
    surface: SurfaceType,
    bank_angle: f32,
) -> JokerCut {
    let wps = &track.spline.waypoints;
    let (s_idx, m_idx) = (split_idx, merge_idx);

    assert!(s_idx < m_idx && m_idx < wps.len());

    let seg0_wps = wps[0..=s_idx].to_vec();
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
    let num_joker_steps = 8usize;
    let target_delta = 42.0f32;

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

    let (t_split, t_merge) = (waypoint_tangent(wps, s_idx), waypoint_tangent(wps, m_idx));

    let mut seg3_wps = wps[m_idx..].to_vec();
    seg3_wps.push(wps[0].clone());

    JokerCut {
        start: seg0_wps,
        main: seg1_wps,
        joker: best_seg2_wps,
        finish: seg3_wps,
        t_split,
        t_merge,
        joker_surfaces: (surface, surface),
    }
}

/// Direction of the main line at waypoint `i`.
fn waypoint_tangent(wps: &[TrackWaypoint], i: usize) -> Vec2 {
    if i > 0 && i + 1 < wps.len() {
        (wps[i + 1].point - wps[i - 1].point).normalize_or_zero()
    } else if i + 1 < wps.len() {
        (wps[i + 1].point - wps[i].point).normalize_or_zero()
    } else {
        (wps[i].point - wps[i - 1].point).normalize_or_zero()
    }
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

    let mut finish = wps[m_idx..].to_vec();
    finish.push(wps[0].clone());
    JokerCut {
        start: wps[..=s_idx].to_vec(),
        main: wps[s_idx..=m_idx].to_vec(),
        joker: joker_wps,
        finish,
        t_split: waypoint_tangent(wps, s_idx),
        t_merge: waypoint_tangent(wps, m_idx),
        joker_surfaces: (points[0].2, points[points.len() - 1].2),
    }
}

fn build_track_joker(cfg: &TrackJokerConfig, tracks_base_dir: &Path, osm_jokers: &HashMap<String, OsmJoker>) {
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
        JokerSource::Synthetic { split_idx, merge_idx, side, surface, bank_angle } => {
            synthetic_cut(cfg, &track, split_idx, merge_idx, side, surface, bank_angle)
        }
    };
    let JokerCut { start: seg0_wps, main: seg1_wps, joker: seg2_wps, finish: seg3_wps, t_split, t_merge, joker_surfaces } = cut;
    let split_wp = seg0_wps[seg0_wps.len() - 1].clone();
    let merge_wp = seg3_wps[0].clone();

    let split_sock_in = SplineSocket::new(split_wp.point, t_split, split_wp.width)
        .with_surface(split_wp.surface.unwrap_or(SurfaceType::Asphalt))
        .with_elevation(split_wp.elevation);

    let split_sock_e0 = SplineSocket::new(split_wp.point, t_split, split_wp.width)
        .with_surface(split_wp.surface.unwrap_or(SurfaceType::Asphalt))
        .with_elevation(split_wp.elevation);

    let split_sock_e1 = SplineSocket::new(split_wp.point, t_split, split_wp.width)
        .with_surface(joker_surfaces.0)
        .with_elevation(split_wp.elevation);

    let split_junction = RoadJunction::split(
        JunctionId(0),
        format!("{} Joker Split", cfg.slug),
        split_sock_in.clone(),
        vec![split_sock_e0.clone(), split_sock_e1.clone()],
        Some(GoreConfig::new(
            split_wp.point,
            12.0,
            split_wp.width,
            BarrierType::TireWall,
        )),
    );

    let merge_sock_i0 = SplineSocket::new(merge_wp.point, t_merge, merge_wp.width)
        .with_surface(merge_wp.surface.unwrap_or(SurfaceType::Asphalt))
        .with_elevation(merge_wp.elevation);

    let merge_sock_i1 = SplineSocket::new(merge_wp.point, t_merge, merge_wp.width)
        .with_surface(joker_surfaces.1)
        .with_elevation(merge_wp.elevation);

    let merge_sock_eg = SplineSocket::new(merge_wp.point, t_merge, merge_wp.width)
        .with_surface(merge_wp.surface.unwrap_or(SurfaceType::Asphalt))
        .with_elevation(merge_wp.elevation);

    let merge_junction = RoadJunction::merge(
        JunctionId(1),
        format!("{} Joker Merge", cfg.slug),
        vec![merge_sock_i0.clone(), merge_sock_i1.clone()],
        merge_sock_eg.clone(),
        Some(MergeConfig {
            convergence_point: merge_wp.point,
            merge_angle: 14.0,
            merge_length: 15.0,
        }),
    );

    let mut seg0 = RoadSegment::new(SegmentId(0), "Start / Finish Straight", seg0_wps)
        .with_junctions(Some(SocketId::new(JunctionId(1), 0)), Some(SocketId::new(JunctionId(0), 0)));
    let mut seg1 = RoadSegment::new(SegmentId(1), "Main Racing Line", seg1_wps)
        .with_junctions(Some(SocketId::new(JunctionId(0), 0)), Some(SocketId::new(JunctionId(1), 0)));
    let mut seg2 = RoadSegment::new(SegmentId(2), cfg.name, seg2_wps)
        .with_junctions(Some(SocketId::new(JunctionId(0), 1)), Some(SocketId::new(JunctionId(1), 1)));
    let mut seg3 = RoadSegment::new(SegmentId(3), "Return Straight", seg3_wps)
        .with_junctions(Some(SocketId::new(JunctionId(1), 0)), Some(SocketId::new(JunctionId(0), 0)));

    seg0.recompute_samples(None, Some(&split_sock_in));
    seg1.recompute_samples(Some(&split_sock_e0), Some(&merge_sock_i0));
    seg2.recompute_samples(Some(&split_sock_e1), Some(&merge_sock_i1));
    seg3.recompute_samples(Some(&merge_sock_eg), None);

    let delta = seg2.length - seg1.length;
    println!(
        "  {} -> main: {:.1}m, joker: {:.1}m, delta: {:.1}m",
        cfg.slug, seg1.length, seg2.length, delta
    );
    // A mapped joker keeps its real length; only the synthetic one is built to a length.
    if let JokerSource::Synthetic { .. } = cfg.source {
        assert!(
            delta >= 30.0 && delta <= 70.0,
            "{}: delta {:.1}m outside [30, 70]m",
            cfg.slug,
            delta
        );
    }

    // Map existing checkpoints to segments
    for cp in &mut track.checkpoints {
        if cp.id == 0 {
            cp.segment_id = Some(SegmentId(0));
            continue;
        }
        let center = (cp.gate.start + cp.gate.end) * 0.5;
        let p_seg0 = seg0.project_point(center);
        let p_seg1 = seg1.project_point(center);
        let p_seg3 = seg3.project_point(center);

        let d0 = (p_seg0.closest_point - center).length();
        let d1 = (p_seg1.closest_point - center).length();
        let d3 = (p_seg3.closest_point - center).length();

        // A checkpoint on the merge point (the end of seg1) belongs to seg3, which both layouts share. Comparing
        // it with seg1 too tagged it seg0 and put it before the joker gate in the joker layout.
        if d1 < d0 && d1 < d3 && p_seg1.progress_distance > 1.0 && p_seg1.progress_distance < seg1.length - 1.0 {
            cp.segment_id = Some(SegmentId(1));
        } else if d3 < d0 {
            cp.segment_id = Some(SegmentId(3));
        } else {
            cp.segment_id = Some(SegmentId(0));
        }
    }

    // Add Joker checkpoint on seg2
    let joker_cp_id = track.checkpoints.len();
    let mid_sample = seg2.sample_at_distance(seg2.length * 0.5);
    let half_w = mid_sample.width * 0.5;
    let joker_gate = LineSegment::new(
        mid_sample.point - mid_sample.normal * half_w,
        mid_sample.point + mid_sample.normal * half_w,
    );
    let joker_cp = Checkpoint::new(
        joker_cp_id,
        joker_gate,
        mid_sample.tangent,
        1,
        false,
    )
    .with_segment(SegmentId(2))
    .with_joker(true)
    .with_elevation(mid_sample.elevation);
    track.checkpoints.push(joker_cp);

    let main_cp_ids: Vec<usize> = track
        .checkpoints
        .iter()
        .filter(|cp| !cp.is_joker)
        .map(|cp| cp.id)
        .collect();

    let mut joker_cp_ids: Vec<usize> = Vec::new();
    for cp in &track.checkpoints {
        if cp.segment_id == Some(SegmentId(0)) {
            joker_cp_ids.push(cp.id);
        }
    }
    joker_cp_ids.push(joker_cp_id);
    for cp in &track.checkpoints {
        if cp.segment_id == Some(SegmentId(3)) {
            joker_cp_ids.push(cp.id);
        }
    }

    let mut layout_main = TrackLayout::new(
        "main",
        "Standard Circuit",
        vec![SegmentId(0), SegmentId(1), SegmentId(3)],
        SegmentId(0),
    )
    .with_checkpoints(main_cp_ids);
    layout_main.total_lap_length = seg0.length + seg1.length + seg3.length;

    let mut layout_joker = TrackLayout::new(
        "joker",
        "Joker Lap Detour",
        vec![SegmentId(0), SegmentId(2), SegmentId(3)],
        SegmentId(0),
    )
    .with_checkpoints(joker_cp_ids);
    layout_joker.total_lap_length = seg0.length + seg2.length + seg3.length;

    let network = TrackNetwork {
        junctions: vec![split_junction, merge_junction],
        segments: vec![seg0, seg1, seg2, seg3],
        layouts: vec![layout_main, layout_joker],
        default_layout_id: "main".to_string(),
        ..Default::default()
    };

    network
        .validate()
        .unwrap_or_else(|e| panic!("{} TrackNetwork validation failed: {:?}", cfg.slug, e));

    track.network = Some(network);
    track.trim_walls_for_network();
    track
        .save_to_file(&path)
        .unwrap_or_else(|e| panic!("failed to save {}: {:?}", cfg.slug, e));
    println!("  Successfully baked {} with TrackNetwork.", cfg.slug);
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
        "Baking authentic Joker Lap TrackNetworks for 20 World RX tracks in {:?}...",
        tracks_dir
    );

    for cfg in WORLD_RX_CONFIGS {
        build_track_joker(cfg, &tracks_dir, &osm_jokers);
    }

    println!("All 20 World RX tracks successfully baked with authentic TrackNetworks!");
}
