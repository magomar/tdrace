//! Authors and bakes authentic Joker Lap branching segments into the 20 World RX circuits.
//!
//! Spec 081: Rallycross Joker Lap Segments for Classic and OpenStreetMap Circuits.

use std::path::Path;
use glam::Vec2;
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
    split_idx: usize,
    merge_idx: usize,
    side: f32, // +1.0 for left normal, -1.0 for right normal
    surface: SurfaceType,
    bank_angle: f32,
}

const WORLD_RX_CONFIGS: &[TrackJokerConfig] = &[
    TrackJokerConfig {
        slug: "holjes_rx",
        name: "Höljes Velodrome Joker Detour",
        split_idx: 4,
        merge_idx: 8,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 8.0,
    },
    TrackJokerConfig {
        slug: "lydden_hill",
        name: "Chessons Drift Joker Detour",
        split_idx: 12,
        merge_idx: 16,
        side: 1.0,
        surface: SurfaceType::Gravel,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "hell_rx",
        name: "Lånkebanen Downhill Joker Detour",
        split_idx: 3,
        merge_idx: 7,
        side: -1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 7.0,
    },
    TrackJokerConfig {
        slug: "loheac_rx",
        name: "Lohéac Hairpin Joker Detour",
        split_idx: 22,
        merge_idx: 27,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "estering_rx",
        name: "Estering Turn 1 Joker Detour",
        split_idx: 4,
        merge_idx: 8,
        side: -1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "montalegre_rx",
        name: "Montalegre Stadium Joker Detour",
        split_idx: 18,
        merge_idx: 23,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "nyirad_rx",
        name: "Nyirád Red Cauldron Joker Detour",
        split_idx: 44,
        merge_idx: 52,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "kouvola_rx",
        name: "Tykkimäki Velodrome Joker Detour",
        split_idx: 20,
        merge_idx: 26,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 7.0,
    },
    TrackJokerConfig {
        slug: "catalunya_rx",
        name: "Barcelona Stadium Chicane Joker Detour",
        split_idx: 22,
        merge_idx: 27,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "mettet_rx",
        name: "Mettet Arena Joker Detour",
        split_idx: 14,
        merge_idx: 19,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "lavare_rx",
        name: "Circuit de Lavaré Joker Detour",
        split_idx: 23,
        merge_idx: 28,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "riga_rx",
        name: "Biķernieki Forest Joker Detour",
        split_idx: 19,
        merge_idx: 24,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "killarney_rx",
        name: "Table Mountain Sweep Joker Detour",
        split_idx: 17,
        merge_idx: 22,
        side: 1.0,
        surface: SurfaceType::Gravel,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "lessay_rx",
        name: "Circuit de Lessay Joker Detour",
        split_idx: 19,
        merge_idx: 25,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "essay_rx",
        name: "Circuit des Ducs La Butte Joker Detour",
        split_idx: 31,
        merge_idx: 37,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "dreux_rx",
        name: "Dreux Switchback Joker Detour",
        split_idx: 17,
        merge_idx: 23,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "croft_rx",
        name: "Croft Infield Joker Detour",
        split_idx: 16,
        merge_idx: 21,
        side: 1.0,
        surface: SurfaceType::Gravel,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "spa_rx",
        name: "Spa Raidillon Crest Joker Detour",
        split_idx: 16,
        merge_idx: 22,
        side: 1.0,
        surface: SurfaceType::Gravel,
        bank_angle: 7.0,
    },
    TrackJokerConfig {
        slug: "silverstone_rx",
        name: "Silverstone Stowe Joker Detour",
        split_idx: 9,
        merge_idx: 14,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 6.0,
    },
    TrackJokerConfig {
        slug: "erx_motor_park",
        name: "ERX Clay Bowl Joker Detour",
        split_idx: 13,
        merge_idx: 19,
        side: 1.0,
        surface: SurfaceType::Dirt,
        bank_angle: 8.0,
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

fn build_track_joker(cfg: &TrackJokerConfig, tracks_base_dir: &Path) {
    let path = tracks_base_dir.join(format!("{}.json", cfg.slug));
    println!("Processing {} ({:?})...", cfg.slug, path);

    let mut track = Track::load_from_file(&path)
        .unwrap_or_else(|e| panic!("failed to load {}: {:?}", cfg.slug, e));
    track.checkpoints.retain(|cp| !cp.is_joker);

    let wps = &track.spline.waypoints;
    let s_idx = cfg.split_idx;
    let m_idx = cfg.merge_idx;

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
    let build_joker_wps = |side: f32| -> Vec<TrackWaypoint> {
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
                let pt = p + norm * (offset * side);

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
                    cfg.bank_angle
                };

                let wp = TrackWaypoint::new(pt, width)
                    .with_surface(if k == 0 {
                        wps[s_idx].surface.unwrap_or(SurfaceType::Asphalt)
                    } else if k == num_joker_steps {
                        wps[m_idx].surface.unwrap_or(SurfaceType::Asphalt)
                    } else {
                        cfg.surface
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
    let preferred = build_joker_wps(cfg.side);
    let preferred_radius = min_turn_radius(&RoadSegment::new(SegmentId(2), cfg.name, preferred.clone()), &seg1_prelim);
    let best_seg2_wps = if preferred_radius >= MIN_JOKER_RADIUS_M {
        preferred
    } else {
        let other = build_joker_wps(-cfg.side);
        let other_radius = min_turn_radius(&RoadSegment::new(SegmentId(2), cfg.name, other.clone()), &seg1_prelim);
        println!(
            "  {} -> side {:+} turns at {:.1} m, side {:+} at {:.1} m",
            cfg.slug, cfg.side, preferred_radius, -cfg.side, other_radius
        );
        if other_radius > preferred_radius { other } else { preferred }
    };

    let t_split = if s_idx > 0 && s_idx + 1 < wps.len() {
        (wps[s_idx + 1].point - wps[s_idx - 1].point).normalize_or_zero()
    } else {
        (wps[s_idx + 1].point - wps[s_idx].point).normalize_or_zero()
    };

    let t_merge = if m_idx > 0 && m_idx + 1 < wps.len() {
        (wps[m_idx + 1].point - wps[m_idx - 1].point).normalize_or_zero()
    } else {
        (wps[m_idx].point - wps[m_idx - 1].point).normalize_or_zero()
    };

    let split_sock_in = SplineSocket::new(wps[s_idx].point, t_split, wps[s_idx].width)
        .with_surface(wps[s_idx].surface.unwrap_or(SurfaceType::Asphalt))
        .with_elevation(wps[s_idx].elevation);

    let split_sock_e0 = SplineSocket::new(wps[s_idx].point, t_split, wps[s_idx].width)
        .with_surface(wps[s_idx].surface.unwrap_or(SurfaceType::Asphalt))
        .with_elevation(wps[s_idx].elevation);

    let split_sock_e1 = SplineSocket::new(wps[s_idx].point, t_split, wps[s_idx].width)
        .with_surface(cfg.surface)
        .with_elevation(wps[s_idx].elevation);

    let split_junction = RoadJunction::split(
        JunctionId(0),
        format!("{} Joker Split", cfg.slug),
        split_sock_in.clone(),
        vec![split_sock_e0.clone(), split_sock_e1.clone()],
        Some(GoreConfig::new(
            wps[s_idx].point,
            12.0,
            wps[s_idx].width,
            BarrierType::TireWall,
        )),
    );

    let merge_sock_i0 = SplineSocket::new(wps[m_idx].point, t_merge, wps[m_idx].width)
        .with_surface(wps[m_idx].surface.unwrap_or(SurfaceType::Asphalt))
        .with_elevation(wps[m_idx].elevation);

    let merge_sock_i1 = SplineSocket::new(wps[m_idx].point, t_merge, wps[m_idx].width)
        .with_surface(cfg.surface)
        .with_elevation(wps[m_idx].elevation);

    let merge_sock_eg = SplineSocket::new(wps[m_idx].point, t_merge, wps[m_idx].width)
        .with_surface(wps[m_idx].surface.unwrap_or(SurfaceType::Asphalt))
        .with_elevation(wps[m_idx].elevation);

    let merge_junction = RoadJunction::merge(
        JunctionId(1),
        format!("{} Joker Merge", cfg.slug),
        vec![merge_sock_i0.clone(), merge_sock_i1.clone()],
        merge_sock_eg.clone(),
        Some(MergeConfig {
            convergence_point: wps[m_idx].point,
            merge_angle: 14.0,
            merge_length: 15.0,
        }),
    );
    let mut seg3_wps = wps[m_idx..].to_vec();
    seg3_wps.push(wps[0].clone());

    let mut seg0 = RoadSegment::new(SegmentId(0), "Start / Finish Straight", seg0_wps)
        .with_junctions(Some(SocketId::new(JunctionId(1), 0)), Some(SocketId::new(JunctionId(0), 0)));
    let mut seg1 = RoadSegment::new(SegmentId(1), "Main Racing Line", seg1_wps)
        .with_junctions(Some(SocketId::new(JunctionId(0), 0)), Some(SocketId::new(JunctionId(1), 0)));
    let mut seg2 = RoadSegment::new(SegmentId(2), cfg.name, best_seg2_wps)
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
    assert!(
        delta >= 30.0 && delta <= 70.0,
        "{}: delta {:.1}m outside [30, 70]m",
        cfg.slug,
        delta
    );

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

    println!(
        "Baking authentic Joker Lap TrackNetworks for 20 World Rallycross tracks in {:?}...",
        tracks_dir
    );

    for cfg in WORLD_RX_CONFIGS {
        build_track_joker(cfg, &tracks_dir);
    }

    println!("All 20 World Rallycross tracks successfully baked with authentic TrackNetworks!");
}
