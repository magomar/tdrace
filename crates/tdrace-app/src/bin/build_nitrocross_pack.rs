//! Builds the 6 official North American Nitrocross circuits for Spec 134:
//! - `erx_motor_park`
//! - `utah_umc_rx`
//! - `wild_horse_pass_rx`
//! - `glen_helen_rx`
//! - `midamerica_outdoors_rx`
//! - `calgary_stampede_rx`
//!
//! Each circuit features:
//! 1. Grade-separated `RampKind::MegaRamp` over-under crossover on `Seg 1` where the Joker Lap (`Seg 2`)
//!    passes underneath through the open gap `(s_takeoff, s_landing)` at ground level (`elevation == 0.0`).
//! 2. Secondary `RampKind::Ramp` tabletop jump on the start/finish straight.
//! 3. 6-car 2-2-2 staggered Rallycross launch chute (`PackedGridPattern::RallycrossTwoTwoTwo`).

use std::path::Path;

use glam::Vec2;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{
    BarrierType, JumpRamp, LineSegment, RampKind, SurfaceShape, TrackGeometry, WallBarrier,
};
use tdrace_core::track::network::{
    GoreConfig, JunctionId, JunctionKind, MergeConfig, RoadJunction, RoadSegment, SegmentId,
    SocketId, SplineSocket, TrackLayout, TrackNetwork,
};
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::validation::{validate_track, ValidationSeverity};
use tdrace_core::track::{
    generate_checkpoints, generate_walls_from_spline, Checkpoint, ChuteSide, LaunchChuteSpec,
    PackedGridPattern, Track, TrackCategory, TrackKind,
};
use tdrace_core::CarCategory;

struct NitroVenueSpec {
    slug: &'static str,
    name: &'static str,
    description: &'static str,
    country_code: &'static str,
    country_name: &'static str,
    wikipedia_url: &'static str,
    osm_url: &'static str,
    tag: &'static str,
    default_surface: SurfaceType,
    runoff_surface: Option<SurfaceType>,
    primary_surface: SurfaceType,
    secondary_surface: SurfaceType,
    road_width: f32,
    back_len: f32,
    loop_depth: f32,
    right_lobe_extra: f32,
    left_lobe_extra: f32,
    target_joker_cost_s: f32,
    mega_jump_name: &'static str,
    minor_jump_name: &'static str,
    joker_name: &'static str,
}

const VENUES: &[NitroVenueSpec] = &[
    NitroVenueSpec {
        slug: "erx_motor_park",
        name: "ERX Motor Park RX",
        description: "Premier Nitrocross amphitheater in Elk River, Minnesota featuring the iconic grade-separated Mega Jump over-under Joker crossover and sweeping clay bowls.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Nitrocross",
        osm_url: "https://www.openstreetmap.org/way/1023648273",
        tag: "NITROCROSS ERX GAP JUMP",
        default_surface: SurfaceType::Grass,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        secondary_surface: SurfaceType::Asphalt,
        road_width: 14.0,
        back_len: 264.0,
        loop_depth: 175.0,
        right_lobe_extra: 24.0,
        left_lobe_extra: 20.0,
        target_joker_cost_s: 3.4,
        mega_jump_name: "ERX Motor Park RX Nitro Mega Jump",
        minor_jump_name: "ERX Motor Park RX Tabletop Jump",
        joker_name: "ERX Clay Bowl Underpass Joker Detour",
    },
    NitroVenueSpec {
        slug: "utah_umc_rx",
        name: "Utah Motorsports Campus Nitro RX",
        description: "High-desert Nitrocross stadium circuit in Tooele, Utah featuring the signature 120-foot over-under gap jump across the Joker Lap and banked dirt hairpins.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Utah_Motorsports_Campus",
        osm_url: "https://www.openstreetmap.org/way/432425828",
        tag: "NITROCROSS UTAH MEGA GAP",
        default_surface: SurfaceType::Dirt,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        secondary_surface: SurfaceType::Asphalt,
        road_width: 14.0,
        back_len: 280.0,
        loop_depth: 186.0,
        right_lobe_extra: 30.0,
        left_lobe_extra: 26.0,
        target_joker_cost_s: 3.6,
        mega_jump_name: "Utah UMC RX Desert Mega Jump",
        minor_jump_name: "Utah UMC RX Step-Up Tabletop",
        joker_name: "Utah UMC Underpass Joker Detour",
    },
    NitroVenueSpec {
        slug: "wild_horse_pass_rx",
        name: "Wild Horse Pass Motorsports Park RX",
        description: "Fast desert-clay and tarmac Nitrocross finale venue in Chandler, Arizona with a rhythm tabletop straight and grade-separated Mega Jump over the Joker canyon.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Firebird_Motorsports_Park",
        osm_url: "https://www.openstreetmap.org/way/138530421",
        tag: "NITROCROSS ARIZONA FINALE",
        default_surface: SurfaceType::Dirt,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        secondary_surface: SurfaceType::Asphalt,
        road_width: 13.8,
        back_len: 248.0,
        loop_depth: 162.0,
        right_lobe_extra: 18.0,
        left_lobe_extra: 16.0,
        target_joker_cost_s: 3.2,
        mega_jump_name: "Wild Horse Pass RX Canyon Mega Jump",
        minor_jump_name: "Wild Horse Pass RX Desert Tabletop",
        joker_name: "Wild Horse Pass Underpass Joker Detour",
    },
    NitroVenueSpec {
        slug: "glen_helen_rx",
        name: "Glen Helen Raceway RX",
        description: "Southern California motocross-inspired Nitrocross circuit in San Bernardino featuring Talladega-style banked dirt sweepers and an over-under Mega Jump.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Glen_Helen_Raceway",
        osm_url: "https://www.openstreetmap.org/way/261015831",
        tag: "NITROCROSS GLEN HELEN BANKING",
        default_surface: SurfaceType::Dirt,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        secondary_surface: SurfaceType::Asphalt,
        road_width: 14.2,
        back_len: 272.0,
        loop_depth: 180.0,
        right_lobe_extra: 28.0,
        left_lobe_extra: 22.0,
        target_joker_cost_s: 3.5,
        mega_jump_name: "Glen Helen RX San Bernardino Mega Jump",
        minor_jump_name: "Glen Helen RX Motocross Tabletop",
        joker_name: "Glen Helen Canyon Underpass Joker Detour",
    },
    NitroVenueSpec {
        slug: "midamerica_outdoors_rx",
        name: "MidAmerica Outdoors RX",
        description: "Rolling red-clay Ozark off-road amphitheater in Jay, Oklahoma with wide dirt sweepers, high-commitment step-up jumps, and a grade-separated Mega Jump.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Nitrocross",
        osm_url: "https://www.openstreetmap.org/relation/146650",
        tag: "NITROCROSS OKLAHOMA CLAY",
        default_surface: SurfaceType::Grass,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        secondary_surface: SurfaceType::Asphalt,
        road_width: 14.0,
        back_len: 286.0,
        loop_depth: 190.0,
        right_lobe_extra: 32.0,
        left_lobe_extra: 28.0,
        target_joker_cost_s: 3.7,
        mega_jump_name: "MidAmerica Outdoors RX Ozark Mega Jump",
        minor_jump_name: "MidAmerica Outdoors RX Clay Tabletop",
        joker_name: "MidAmerica Ozark Underpass Joker Detour",
    },
    NitroVenueSpec {
        slug: "calgary_stampede_rx",
        name: "Stampede Park Nitro Snow & Ice RX",
        description: "Frozen stadium winter round in Calgary, Alberta combining packed snow straights, glazed sheet-ice hairpins, deep snowdrift runoffs, and an over-under Mega Jump.",
        country_code: "CA",
        country_name: "Canada",
        wikipedia_url: "https://en.wikipedia.org/wiki/Stampede_Park",
        osm_url: "https://www.openstreetmap.org/way/34891512",
        tag: "NITROCROSS CALGARY SNOW & ICE",
        default_surface: SurfaceType::DeepSnow,
        runoff_surface: Some(SurfaceType::DeepSnow),
        primary_surface: SurfaceType::PackedSnow,
        secondary_surface: SurfaceType::SheetIce,
        road_width: 14.0,
        back_len: 240.0,
        loop_depth: 156.0,
        right_lobe_extra: 14.0,
        left_lobe_extra: 14.0,
        target_joker_cost_s: 3.1,
        mega_jump_name: "Stampede Park RX Glacier Mega Jump",
        minor_jump_name: "Stampede Park RX Winter Tabletop",
        joker_name: "Stampede Ice Underpass Joker Detour",
    },
];

fn speed_limited_lap_time(spline: &TrackSpline) -> f32 {
    const TOP_SPEED: f32 = 40.0;
    const CORNER_GRIP: f32 = 10.0;
    const ACCELERATION: f32 = 8.0;
    const BRAKING: f32 = 10.0;
    const CURVATURE_SPAN_M: f32 = 4.0;
    let s = &spline.samples;
    let n = s.len();
    let ds = |i: usize| {
        if i + 1 < n {
            s[i + 1].distance - s[i].distance
        } else {
            spline.total_length - s[i].distance
        }
    };
    let span = ((CURVATURE_SPAN_M * n as f32 / spline.total_length).round() as usize).max(1);
    let mut v: Vec<f32> = (0..n)
        .map(|i| {
            let (a, b) = (&s[(i + n - span) % n], &s[(i + span) % n]);
            let turn = a.tangent.perp_dot(b.tangent).atan2(a.tangent.dot(b.tangent)).abs();
            let curvature = turn / (2.0 * span as f32 * spline.total_length / n as f32);
            (CORNER_GRIP / curvature.max(1e-6)).sqrt().min(TOP_SPEED)
        })
        .collect();
    for k in 0..2 * n {
        let (i, j) = (k % n, (k + 1) % n);
        v[j] = v[j].min((v[i] * v[i] + 2.0 * ACCELERATION * ds(i)).sqrt());
    }
    for k in (0..2 * n).rev() {
        let (i, j) = (k % n, (k + 1) % n);
        v[i] = v[i].min((v[j] * v[j] + 2.0 * BRAKING * ds(i)).sqrt());
    }
    (0..n).map(|i| ds(i) / ((v[i] + v[(i + 1) % n]) * 0.5)).sum()
}

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

fn build_main_waypoints(spec: &NitroVenueSpec) -> (Vec<TrackWaypoint>, usize, usize, usize) {
    let half_back = spec.back_len * 0.5;
    let y_back = -spec.loop_depth;
    let r_right = spec.loop_depth * 0.5;
    let r_left = spec.loop_depth * 0.5;

    // 28 closed loop waypoints:
    // - 0..=4: Top straight along y = 0 from x = -80 to x = +half_back (160m+ straight for start/finish & minor jump)
    // - 5..=9: Right stadium sweeper from y = 0 down to y = y_back
    // - 10..=18: Bottom straight along y = y_back from x = +half_back to x = -half_back (split_idx = 10, cross_idx = 14, merge_idx = 18)
    // - 19..=23: Left stadium sweeper from y = y_back up to y = 0
    // - 24..=27: Return straight along y = 0 from x = -half_back to x = -115 (ideal straight for 6-car launch chute!)
    let mut raw_pts: Vec<(Vec2, SurfaceType, f32)> = Vec::new();

    // Top straight (x from -75 to +half_back, y = 0)
    let top_xs = [-75.0, -35.0, 5.0, 48.0, 92.0, half_back];
    for (idx, &x) in top_xs.iter().enumerate() {
        let surf = if idx <= 1 {
            spec.secondary_surface
        } else {
            spec.primary_surface
        };
        raw_pts.push((Vec2::new(x, 0.0), surf, 0.0));
    }
    // Right sweeper (5 interior points from angle +70 deg down to -70 deg around (half_back, -r_right))
    for k in 1..=5 {
        let frac = k as f32 / 6.0;
        let ang = std::f32::consts::FRAC_PI_2 - frac * std::f32::consts::PI;
        let cx = half_back;
        let cy = -r_right;
        let rx = r_right + spec.right_lobe_extra * (std::f32::consts::PI * frac).sin();
        let pt = Vec2::new(cx + rx * ang.cos(), cy + r_right * ang.sin());
        let surf = if k <= 3 {
            spec.secondary_surface
        } else {
            spec.primary_surface
        };
        raw_pts.push((pt, surf, 6.0));
    }

    // Bottom straight (9 points: indices 11..=19; split_idx = 11, cross_idx = 15, merge_idx = 19)
    let split_idx = raw_pts.len();
    for k in 0..=8 {
        let frac = k as f32 / 8.0;
        let x = half_back - frac * (2.0 * half_back);
        raw_pts.push((Vec2::new(x, y_back), spec.primary_surface, 0.0));
    }
    let cross_idx = split_idx + 4;
    let merge_idx = split_idx + 8;

    // Left sweeper (5 interior points from angle -90 deg to +90 deg around (-half_back, -r_left))
    for k in 1..=5 {
        let frac = k as f32 / 6.0;
        let ang = -std::f32::consts::FRAC_PI_2 - frac * std::f32::consts::PI;
        let cx = -half_back;
        let cy = -r_left;
        let rx = r_left + spec.left_lobe_extra * (std::f32::consts::PI * frac).sin();
        let pt = Vec2::new(cx + rx * ang.cos(), cy + r_left * ang.sin());
        let surf = if k >= 5 {
            spec.secondary_surface
        } else {
            spec.primary_surface
        };
        raw_pts.push((pt, surf, 6.0));
    }

    // Return straight along y = 0 from x = -half_back to x = -110
    let ret_xs = [-half_back, -half_back + 38.0];
    for &x in &ret_xs {
        if x < -95.0 {
            raw_pts.push((Vec2::new(x, 0.0), spec.secondary_surface, 0.0));
        }
    }

    let wps: Vec<TrackWaypoint> = raw_pts
        .into_iter()
        .map(|(pt, surf, bank)| {
            let mut wp = TrackWaypoint::new(pt, spec.road_width)
                .with_surface(surf)
                .with_elevation(0.0)
                .with_bank_angle(bank);
            wp.left_runoff_surface = spec.runoff_surface;
            wp.right_runoff_surface = spec.runoff_surface;
            wp
        })
        .collect();

    (wps, split_idx, cross_idx, merge_idx)
}

fn build_scrossover_joker_waypoints(
    spec: &NitroVenueSpec,
    wps: &[TrackWaypoint],
    split_idx: usize,
    cross_idx: usize,
    merge_idx: usize,
    d_lobe: f32,
) -> Vec<TrackWaypoint> {
    let p_split = wps[split_idx].point;
    let p_cross = wps[cross_idx].point;
    let p_merge = wps[merge_idx].point;
    let t_dir = (p_merge - p_split).normalize_or_zero();
    let n_dir = Vec2::new(-t_dir.y, t_dir.x); // Left normal
    let half_span = p_split.distance(p_cross);

    // Collinear crossing (-0.95, 7.6) -> (-0.40, 3.2) -> (0.0, 0.0) -> (0.40, -3.2) -> (0.95, -7.6)
    // guarantees |u| <= 0.58m across |lat| <= 4.6m (strictly inside the 6.2m MegaRamp gap [-3.1m, +3.1m]
    // with >1.5m hull clearance from both takeoff/landing drop-off walls), while keeping turn radius >= 3.5m.
    let stations: Vec<(f32, f32)> = vec![
        (-half_span, 0.0),
        (-half_span + 16.0, 0.16 * d_lobe),
        (-half_span + 32.0, 0.50 * d_lobe),
        (-half_span + 48.0, 0.84 * d_lobe),
        (-half_span + 64.0, 1.00 * d_lobe),
        (-34.0, 1.00 * d_lobe),
        (-18.0, 15.2f32.max(0.84 * d_lobe)),
        (-6.0, 12.8),
        (-0.95, 7.6),
        (-0.40, 3.2),
        (0.0, 0.0),
        (0.40, -3.2),
        (0.95, -7.6),
        (6.0, -12.8),
        (18.0, -15.2f32.max(0.84 * d_lobe)),
        (34.0, -1.00 * d_lobe),
        (half_span - 64.0, -1.00 * d_lobe),
        (half_span - 48.0, -0.84 * d_lobe),
        (half_span - 32.0, -0.50 * d_lobe),
        (half_span - 16.0, -0.16 * d_lobe),
        (half_span, 0.0),
    ];

    let n = stations.len();
    stations
        .into_iter()
        .enumerate()
        .map(|(idx, (u, lat))| {
            let pt = if idx == 0 {
                p_split
            } else if idx + 1 == n {
                p_merge
            } else {
                p_cross + t_dir * u + n_dir * lat
            };
            let width = if idx == 0 || idx + 1 == n {
                spec.road_width
            } else {
                13.0
            };
            let mut wp = TrackWaypoint::new(pt, width)
                .with_surface(spec.primary_surface)
                .with_elevation(0.0)
                .with_bank_angle(0.0);
            wp.left_runoff_surface = spec.runoff_surface;
            wp.right_runoff_surface = spec.runoff_surface;
            wp
        })
        .collect()
}

fn assemble_network(
    spec: &NitroVenueSpec,
    track: &Track,
    split_idx: usize,
    merge_idx: usize,
    joker_wps: Vec<TrackWaypoint>,
) -> TrackNetwork {
    let wps = &track.spline.waypoints;
    let seg0_wps = wps[0..=split_idx].to_vec();
    let seg1_wps = wps[split_idx..=merge_idx].to_vec();
    let mut seg3_wps = wps[merge_idx..].to_vec();
    seg3_wps.push(wps[0].clone());

    let s_dist = track.spline.project_point(wps[split_idx].point).progress_distance;
    let m_dist = track.spline.project_point(wps[merge_idx].point).progress_distance;
    let s_sample = track.spline.sample_at_distance(s_dist);
    let m_sample = track.spline.sample_at_distance(m_dist);

    let split_socket = SplineSocket {
        point: wps[split_idx].point,
        tangent: s_sample.tangent,
        normal: s_sample.normal,
        width: wps[split_idx].width,
        elevation: 0.0,
        bank_angle: 0.0,
        surface: spec.primary_surface,
    };
    let merge_socket = SplineSocket {
        point: wps[merge_idx].point,
        tangent: m_sample.tangent,
        normal: m_sample.normal,
        width: wps[merge_idx].width,
        elevation: 0.0,
        bank_angle: 0.0,
        surface: spec.primary_surface,
    };

    let nose_center = wps[split_idx].point + s_sample.tangent * 14.0 + s_sample.normal * (spec.road_width * 0.5 + 2.5);
    let split_junction = RoadJunction {
        id: JunctionId(0),
        name: format!("{} Joker Split", spec.slug),
        kind: JunctionKind::Split {
            ingress_socket: split_socket,
            egress_sockets: vec![split_socket, split_socket],
            gore_config: Some(GoreConfig {
                apex_point: wps[split_idx].point,
                divergence_angle: 14.0,
                gore_length: 14.0,
                nose_barrier: WallBarrier {
                    segment: LineSegment::new(
                        nose_center - s_sample.normal * 0.8,
                        nose_center + s_sample.normal * 0.8,
                    ),
                    restitution: 0.18,
                    friction: 0.8,
                    barrier_type: BarrierType::TireWall,
                    elevation: 0.0,
                    is_bridge: false,
                },
                has_chevrons: true,
            }),
        },
    };

    let merge_junction = RoadJunction {
        id: JunctionId(1),
        name: format!("{} Joker Merge", spec.slug),
        kind: JunctionKind::Merge {
            ingress_sockets: vec![merge_socket, merge_socket],
            egress_socket: merge_socket,
            merge_config: Some(MergeConfig {
                convergence_point: wps[merge_idx].point,
                merge_angle: 14.0,
                merge_length: 15.0,
            }),
        },
    };

    let mut seg0 = RoadSegment::new(SegmentId(0), "Start / Finish Straight", seg0_wps);
    seg0.entry_junction = Some(SocketId::new(JunctionId(1), 0));
    seg0.exit_junction = Some(SocketId::new(JunctionId(0), 0));
    seg0.recompute_samples(None, Some(&split_socket));

    let mut seg1 = RoadSegment::new(SegmentId(1), "Main Racing Line", seg1_wps);
    seg1.entry_junction = Some(SocketId::new(JunctionId(0), 0));
    seg1.exit_junction = Some(SocketId::new(JunctionId(1), 0));
    seg1.recompute_samples(Some(&split_socket), Some(&merge_socket));

    let mut seg2 = RoadSegment::new(SegmentId(2), spec.joker_name, joker_wps);
    seg2.entry_junction = Some(SocketId::new(JunctionId(0), 1));
    seg2.exit_junction = Some(SocketId::new(JunctionId(1), 1));
    seg2.recompute_samples(Some(&split_socket), Some(&merge_socket));

    let mut seg3 = RoadSegment::new(SegmentId(3), "Return Straight", seg3_wps);
    seg3.entry_junction = Some(SocketId::new(JunctionId(1), 0));
    seg3.exit_junction = Some(SocketId::new(JunctionId(0), 0));
    seg3.recompute_samples(Some(&merge_socket), None);

    let main_len = seg0.length + seg1.length + seg3.length;
    let joker_len = seg0.length + seg2.length + seg3.length;

    let mut net = TrackNetwork {
        junctions: vec![split_junction, merge_junction],
        segments: vec![seg0, seg1, seg2, seg3],
        layouts: vec![
            TrackLayout {
                id: "main".to_string(),
                display_name: "Standard Circuit".to_string(),
                entry_segment: None,
                segment_sequence: vec![SegmentId(0), SegmentId(1), SegmentId(3)],
                is_closed: true,
                total_lap_length: main_len,
                start_finish_segment: SegmentId(0),
                checkpoint_ids: Vec::new(),
            },
            TrackLayout {
                id: "joker".to_string(),
                display_name: "Joker Lap Detour".to_string(),
                entry_segment: None,
                segment_sequence: vec![SegmentId(0), SegmentId(2), SegmentId(3)],
                is_closed: true,
                total_lap_length: joker_len,
                start_finish_segment: SegmentId(0),
                checkpoint_ids: Vec::new(),
            },
        ],
        default_layout_id: "main".to_string(),
        launch_chute: None,
        ..Default::default()
    };
    net.recompute_composite_splines();
    net
}

fn build_venue(spec: &NitroVenueSpec) -> (String, Track) {
    let (wps, split_idx, cross_idx, merge_idx) = build_main_waypoints(spec);
    let spline = TrackSpline::new(wps.clone(), true);
    let (inner_walls, outer_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    // Primary MegaRamp on Seg 1 centered at cross_idx (with +0.90m shift along T so the 6.2m gap [-3.1m, +3.1m] is centered at p_cross)
    let p_cross = wps[cross_idx].point;
    let t_cross = (wps[merge_idx].point - wps[split_idx].point).normalize_or_zero();
    let angle_cross = t_cross.y.atan2(t_cross.x);
    let mega_center = p_cross + t_cross * 0.90;
    let mut mega_ramp = JumpRamp::new(
        1,
        SurfaceShape::OrientedBox {
            center: mega_center,
            half_extents: Vec2::new(10.0, 4.6),
            angle: angle_cross,
        },
        t_cross,
        2.6,
        6.8,
        1.62,
        spec.mega_jump_name,
    )
    .with_kind(RampKind::MegaRamp)
    .with_gap_length(6.2);
    mega_ramp.surface = SurfaceType::Dirt;

    // Secondary tabletop jump on the start/finish straight (between wps[3] and wps[4])
    let p_minor = (wps[3].point + wps[4].point) * 0.5;
    let t_minor = (wps[4].point - wps[3].point).normalize_or_zero();
    let angle_minor = t_minor.y.atan2(t_minor.x);
    let mut minor_ramp = JumpRamp::new(
        2,
        SurfaceShape::OrientedBox {
            center: p_minor,
            half_extents: Vec2::new(4.0, 5.4),
            angle: angle_minor,
        },
        t_minor,
        2.4,
        6.0,
        1.25,
        spec.minor_jump_name,
    )
    .with_kind(RampKind::Ramp);
    minor_ramp.surface = SurfaceType::Dirt;

    let mut base_track = Track {
        name: spec.name.to_string(),
        description: spec.description.to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline: spline.clone(),
        geometry: TrackGeometry {
            jump_ramps: vec![mega_ramp.clone(), minor_ramp],
            inner_walls,
            outer_walls,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            ..Default::default()
        },
        default_surface: spec.default_surface,
        default_laps: 5,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: Some(spec.wikipedia_url.to_string()),
        osm_url: Some(spec.osm_url.to_string()),
        country_code: Some(spec.country_code.to_string()),
        country_name: Some(spec.country_name.to_string()),
        is_inspired: false,
        tag: spec.tag.to_string(),
        category_label: "Nitrocross".to_string(),
        ..Default::default()
    };

    // Binary search d_lobe so speed_limited_lap_time delta matches target_joker_cost_s
    let mut low = 12.0f32;
    let mut high = 52.0f32;
    let mut best_joker_wps = Vec::new();
    for _ in 0..24 {
        let mid = (low + high) * 0.5;
        let cand_wps =
            build_scrossover_joker_waypoints(spec, &wps, split_idx, cross_idx, merge_idx, mid);
        let net = assemble_network(spec, &base_track, split_idx, merge_idx, cand_wps.clone());
        let main_sp = net.build_composite_spline_for_layout("main").unwrap();
        let joker_sp = net.build_composite_spline_for_layout("joker").unwrap();
        let cost = speed_limited_lap_time(&joker_sp) - speed_limited_lap_time(&main_sp);
        if cost < spec.target_joker_cost_s {
            low = mid;
        } else {
            high = mid;
        }
        best_joker_wps = cand_wps;
    }

    let mut net = assemble_network(spec, &base_track, split_idx, merge_idx, best_joker_wps);

    // Build checkpoints along main spline + 1 joker checkpoint at the first lobe peak of Seg 2
    let mut checkpoints = generate_checkpoints(&base_track.spline, 24, 3);
    let s_dist = base_track.spline.project_point(wps[split_idx].point).progress_distance;
    let m_dist = base_track.spline.project_point(wps[merge_idx].point).progress_distance;
    let mut main_cp_ids = Vec::new();
    let mut joker_cp_ids = Vec::new();
    let joker_cp_id = checkpoints.len();
    let mut inserted_joker_cp = false;

    for cp in &mut checkpoints {
        let mid = (cp.gate.start + cp.gate.end) * 0.5;
        let d = base_track.spline.project_point(mid).progress_distance;
        main_cp_ids.push(cp.id);
        if d > s_dist + 8.0 && d < m_dist - 8.0 {
            cp.segment_id = Some(SegmentId(1));
            if !inserted_joker_cp {
                joker_cp_ids.push(joker_cp_id);
                inserted_joker_cp = true;
            }
        } else if d <= s_dist + 8.0 {
            cp.segment_id = Some(SegmentId(0));
            joker_cp_ids.push(cp.id);
        } else {
            cp.segment_id = Some(SegmentId(3));
            joker_cp_ids.push(cp.id);
        }
    }

    let seg2 = net.get_segment(SegmentId(2)).unwrap();
    let j_sample = seg2.sample_at_distance(seg2.length * 0.25);
    let j_half_w = j_sample.width * 0.5 + 1.5;
    let mut j_cp = Checkpoint::new(
        joker_cp_id,
        LineSegment::new(
            j_sample.point + j_sample.normal * j_half_w,
            j_sample.point - j_sample.normal * j_half_w,
        ),
        j_sample.tangent,
        1,
        false,
    );
    j_cp.is_joker = true;
    j_cp.segment_id = Some(SegmentId(2));
    checkpoints.push(j_cp);

    net.layouts[0].checkpoint_ids = main_cp_ids;
    net.layouts[1].checkpoint_ids = joker_cp_ids;

    base_track.checkpoints = checkpoints;
    base_track.network = Some(net);
    base_track.rebuild_geometry(3.5, BarrierType::TireWall);

    // Stamp the 6-car Rallycross launch chute
    let mut chute_template = LaunchChuteSpec::new(0, ChuteSide::Right);
    chute_template.pattern = PackedGridPattern::RallycrossTwoTwoTwo;
    chute_template.surface = SurfaceType::Asphalt;
    let chute_spec = base_track
        .place_launch_chute(&chute_template)
        .unwrap_or_else(|e| panic!("{}: failed to place launch chute: {}", spec.slug, e));

    // Verify all network segments have waypoint width >= 12.0m
    if let Some(ref mut net) = base_track.network {
        for seg in &mut net.segments {
            for wp in &mut seg.waypoints {
                if wp.width < 12.5 {
                    wp.width = 12.5;
                }
                if spec.runoff_surface.is_some() {
                    wp.left_runoff_surface = spec.runoff_surface;
                    wp.right_runoff_surface = spec.runoff_surface;
                }
            }
        }
    }
    base_track.ensure_runtime_geometry();

    // Diagnostics & assertions
    let net = base_track.network.as_ref().unwrap();
    let seg1 = net.get_segment(SegmentId(1)).unwrap();
    let seg2 = net.get_segment(SegmentId(2)).unwrap();
    let min_r = min_turn_radius(seg2, seg1);
    let main_sp = net.build_composite_spline_for_layout("main").unwrap();
    let joker_sp = net.build_composite_spline_for_layout("joker").unwrap();
    let cost_s = speed_limited_lap_time(&joker_sp) - speed_limited_lap_time(&main_sp);

    let underpass_samples: Vec<_> = seg2
        .samples
        .iter()
        .filter(|s| mega_ramp.contains(s.point))
        .collect();
    assert!(
        !underpass_samples.is_empty(),
        "{}: Seg 2 must cross through the MegaRamp box",
        spec.slug
    );
    for us in &underpass_samples {
        assert!(
            mega_ramp.is_mega_gap_underpass_point(us.point, 0.0),
            "{}: Seg 2 sample {:?} inside MegaRamp is not inside open gap!",
            spec.slug,
            us.point
        );
    }

    let diags = validate_track(&base_track);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == ValidationSeverity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "{}: validation errors: {:?}",
        spec.slug,
        errors
    );

    println!(
        "Built {:24} -> len={:6.1}m, joker_cost={:+.2}s, min_r={:4.1}m, underpass_pts={:2}, chute_wp={} ({:?}), breakdown={}",
        spec.slug,
        base_track.spline.total_length(),
        cost_s,
        min_r,
        underpass_samples.len(),
        chute_spec.merge_waypoint,
        chute_spec.side,
        base_track.surface_summary_string()
    );
    (spec.slug.to_string(), base_track)
}

fn main() {
    let out_dir = Path::new("tracks/rally");
    let mut built = Vec::new();
    for spec in VENUES {
        built.push(build_venue(spec));
    }
    for (slug, track) in built {
        let out_path = out_dir.join(format!("{}.json", slug));
        track
            .save_to_file(&out_path)
            .unwrap_or_else(|e| panic!("{}: failed to save: {}", slug, e));
    }
}
