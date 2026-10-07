//! Authors and bakes authentic Joker Lap branching segments into the 3 Classic RX circuits.
//!
//! Spec 081: Rallycross Joker Lap Segments for Classic and OpenStreetMap Circuits.

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

fn make_socket(point: Vec2, tangent: Vec2, width: f32, surface: SurfaceType, elevation: f32) -> SplineSocket {
    SplineSocket::new(point, tangent, width)
        .with_surface(surface)
        .with_elevation(elevation)
}

fn build_quarry_sprint() {
    let path = "tracks/classic/rx_quarry_sprint.json";
    println!("Processing {}...", path);
    let mut track = Track::load_from_file(path).expect("failed to load rx_quarry_sprint");
    track.checkpoints.retain(|cp| !cp.is_joker);

    let wps = &track.spline.waypoints;
    // Main track has 38 waypoints: 0..=37
    // Split at wp 21 (-8.3, -229.3), Merge at wp 27 (-53.3, -128.8)
    let seg0_wps = wps[0..=21].to_vec();
    let seg1_wps = wps[21..=27].to_vec();

    let seg2_wps = vec![
        TrackWaypoint::new(Vec2::new(-8.3, -229.3), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(-30.0, -230.4), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(-1.2),
        TrackWaypoint::new(Vec2::new(-68.5, -237.4), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(-2.5),
        TrackWaypoint::new(Vec2::new(-72.2, -209.7), 12.0)
            .with_surface(SurfaceType::Concrete)
            .with_elevation(-0.5),
        TrackWaypoint::new(Vec2::new(-66.8, -165.0), 12.0)
            .with_surface(SurfaceType::Concrete)
            .with_elevation(1.8),
        TrackWaypoint::new(Vec2::new(-58.7, -140.0), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(0.5),
        TrackWaypoint::new(Vec2::new(-53.3, -128.8), 12.0)
            .with_surface(SurfaceType::Asphalt)
            .with_elevation(0.0),
    ];

    let mut seg3_wps = wps[27..=37].to_vec();
    seg3_wps.push(wps[0].clone());

    let seg0 = RoadSegment::new(SegmentId(0), "Start / Finish Straight & Sweepers", seg0_wps)
        .with_junctions(Some(SocketId::new(JunctionId(1), 0)), Some(SocketId::new(JunctionId(0), 0)));
    let seg1 = RoadSegment::new(SegmentId(1), "Quarry Basin Main Line", seg1_wps)
        .with_junctions(Some(SocketId::new(JunctionId(0), 0)), Some(SocketId::new(JunctionId(1), 0)));
    let seg2 = RoadSegment::new(SegmentId(2), "Excavated Basin Joker Detour", seg2_wps)
        .with_junctions(Some(SocketId::new(JunctionId(0), 1)), Some(SocketId::new(JunctionId(1), 1)));
    let seg3 = RoadSegment::new(SegmentId(3), "Quarry Plateau Return Straight", seg3_wps)
        .with_junctions(Some(SocketId::new(JunctionId(1), 0)), Some(SocketId::new(JunctionId(0), 0)));

    let split_sock_in = make_socket(Vec2::new(-8.3, -229.3), Vec2::new(-1.0, 0.0), 12.0, SurfaceType::Dirt, 0.0);
    let split_sock_e0 = make_socket(Vec2::new(-8.3, -229.3), Vec2::new(-1.0, 0.0), 12.0, SurfaceType::Dirt, 0.0);
    let split_sock_e1 = make_socket(Vec2::new(-8.3, -229.3), Vec2::new(-0.9987, -0.0506), 12.0, SurfaceType::Dirt, 0.0);

    let split_junction = RoadJunction::split(
        JunctionId(0),
        "Quarry Joker Split",
        split_sock_in,
        vec![split_sock_e0, split_sock_e1],
        Some(GoreConfig::new(
            Vec2::new(-8.3, -229.3),
            8.5,
            12.0,
            BarrierType::TireWall,
        )),
    );

    let merge_sock_i0 = make_socket(Vec2::new(-53.3, -128.8), Vec2::new(0.0, 1.0), 12.0, SurfaceType::Asphalt, 0.0);
    let merge_sock_i1 = make_socket(Vec2::new(-53.3, -128.8), Vec2::new(0.434, 0.901), 12.0, SurfaceType::Dirt, 0.0);
    let merge_sock_eg = make_socket(Vec2::new(-53.3, -128.8), Vec2::new(0.0, 1.0), 12.0, SurfaceType::Asphalt, 0.0);

    let merge_junction = RoadJunction::merge(
        JunctionId(1),
        "Quarry Joker Merge",
        vec![merge_sock_i0, merge_sock_i1],
        merge_sock_eg,
        Some(MergeConfig {
            convergence_point: Vec2::new(-53.3, -128.8),
            merge_angle: 12.0,
            merge_length: 14.0,
        }),
    );

    // Map checkpoints to segment IDs
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

        if d1 < d0 && d1 < d3 && p_seg1.progress_distance > 1.0 && p_seg1.progress_distance < seg1.length - 1.0 {
            cp.segment_id = Some(SegmentId(1));
        } else if d3 < d0 && d3 < d1 && p_seg3.progress_distance > 1.0 {
            cp.segment_id = Some(SegmentId(3));
        } else {
            cp.segment_id = Some(SegmentId(0));
        }
    }

    // Add Joker checkpoint on seg2
    let joker_cp_id = track.checkpoints.len();
    let joker_gate = LineSegment::new(
        Vec2::new(-72.2 - 6.0 * 0.99, -209.7 + 6.0 * 0.12),
        Vec2::new(-72.2 + 6.0 * 0.99, -209.7 - 6.0 * 0.12),
    );
    let joker_cp = Checkpoint::new(
        joker_cp_id,
        joker_gate,
        Vec2::new(0.12, 0.99),
        1,
        false,
    )
    .with_segment(SegmentId(2))
    .with_joker(true)
    .with_elevation(-0.5);
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

    network.validate().expect("rx_quarry_sprint TrackNetwork validation failed");

    let len_main = network.layouts[0].total_lap_length;
    let len_joker = network.layouts[1].total_lap_length;
    let delta = len_joker - len_main;
    println!("  Main length: {:.1} m, Joker length: {:.1} m, Delta: {:.1} m", len_main, len_joker, delta);

    track.network = Some(network);
    track.trim_walls_for_network();
    track.save_to_file(path).expect("failed to save rx_quarry_sprint");
    println!("  Successfully saved rx_quarry_sprint with TrackNetwork.");
}

fn build_hilltop_leap() {
    let path = "tracks/classic/rx_hilltop_leap.json";
    println!("Processing {}...", path);
    let mut track = Track::load_from_file(path).expect("failed to load rx_hilltop_leap");
    track.checkpoints.retain(|cp| !cp.is_joker);

    let wps = &track.spline.waypoints;
    // Split at wp 28 (10.8, -132.2), Merge at wp 35 (-92.0, 0.9)
    let seg0_wps = wps[0..=28].to_vec();
    let seg1_wps = wps[28..=35].to_vec();

    let seg2_wps = vec![
        TrackWaypoint::new(Vec2::new(10.8, -132.2), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(1.5),
        TrackWaypoint::new(Vec2::new(-13.9, -143.2), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(2.5)
            .with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(-52.4, -132.8), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(4.5)
            .with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(-92.4, -93.2), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(3.5)
            .with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(-112.4, -48.9), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(2.2)
            .with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(-105.8, -14.6), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(1.5),
        TrackWaypoint::new(Vec2::new(-92.0, 0.9), 12.0)
            .with_surface(SurfaceType::Asphalt)
            .with_elevation(1.0),
    ];

    let mut seg3_wps = wps[35..=51].to_vec();
    seg3_wps.push(wps[0].clone());

    let seg0 = RoadSegment::new(SegmentId(0), "Start Straight & Downhill Infield", seg0_wps)
        .with_junctions(Some(SocketId::new(JunctionId(1), 0)), Some(SocketId::new(JunctionId(0), 0)));
    let seg1 = RoadSegment::new(SegmentId(1), "Hilltop Crest Main Line", seg1_wps)
        .with_junctions(Some(SocketId::new(JunctionId(0), 0)), Some(SocketId::new(JunctionId(1), 0)));
    let seg2 = RoadSegment::new(SegmentId(2), "Perimeter Ridge Joker Detour", seg2_wps)
        .with_junctions(Some(SocketId::new(JunctionId(0), 1)), Some(SocketId::new(JunctionId(1), 1)));
    let seg3 = RoadSegment::new(SegmentId(3), "Plateau Esses Return", seg3_wps)
        .with_junctions(Some(SocketId::new(JunctionId(1), 0)), Some(SocketId::new(JunctionId(0), 0)));

    let split_sock_in = make_socket(Vec2::new(10.8, -132.2), Vec2::new(-0.707, 0.707), 12.0, SurfaceType::Dirt, 1.5);
    let split_sock_e0 = make_socket(Vec2::new(10.8, -132.2), Vec2::new(-0.707, 0.707), 12.0, SurfaceType::Dirt, 1.5);
    let split_sock_e1 = make_socket(Vec2::new(10.8, -132.2), Vec2::new(-0.914, -0.406), 12.0, SurfaceType::Dirt, 1.5);

    let split_junction = RoadJunction::split(
        JunctionId(0),
        "Hilltop Joker Split",
        split_sock_in,
        vec![split_sock_e0, split_sock_e1],
        Some(GoreConfig::new(
            Vec2::new(10.8, -132.2),
            15.0,
            14.0,
            BarrierType::TireWall,
        )),
    );

    let merge_sock_i0 = make_socket(Vec2::new(-92.0, 0.9), Vec2::new(0.0, 1.0), 12.0, SurfaceType::Asphalt, 1.0);
    let merge_sock_i1 = make_socket(Vec2::new(-92.0, 0.9), Vec2::new(0.665, 0.747), 12.0, SurfaceType::Dirt, 1.0);
    let merge_sock_eg = make_socket(Vec2::new(-92.0, 0.9), Vec2::new(0.0, 1.0), 12.0, SurfaceType::Asphalt, 1.0);

    let merge_junction = RoadJunction::merge(
        JunctionId(1),
        "Hilltop Joker Merge",
        vec![merge_sock_i0, merge_sock_i1],
        merge_sock_eg,
        Some(MergeConfig {
            convergence_point: Vec2::new(-92.0, 0.9),
            merge_angle: 14.0,
            merge_length: 15.0,
        }),
    );

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

        if d1 < d0 && d1 < d3 && p_seg1.progress_distance > 1.0 && p_seg1.progress_distance < seg1.length - 1.0 {
            cp.segment_id = Some(SegmentId(1));
        } else if d3 < d0 && d3 < d1 && p_seg3.progress_distance > 1.0 {
            cp.segment_id = Some(SegmentId(3));
        } else {
            cp.segment_id = Some(SegmentId(0));
        }
    }

    let joker_cp_id = track.checkpoints.len();
    let joker_gate = LineSegment::new(
        Vec2::new(-92.4 - 6.0 * 0.7, -93.2 - 6.0 * 0.7),
        Vec2::new(-92.4 + 6.0 * 0.7, -93.2 + 6.0 * 0.7),
    );
    let joker_cp = Checkpoint::new(
        joker_cp_id,
        joker_gate,
        Vec2::new(-0.7, 0.7),
        1,
        false,
    )
    .with_segment(SegmentId(2))
    .with_joker(true)
    .with_elevation(3.5);
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

    network.validate().expect("rx_hilltop_leap TrackNetwork validation failed");

    let len_main = network.layouts[0].total_lap_length;
    let len_joker = network.layouts[1].total_lap_length;
    let delta = len_joker - len_main;
    println!("  Main length: {:.1} m, Joker length: {:.1} m, Delta: {:.1} m", len_main, len_joker, delta);

    track.network = Some(network);
    track.trim_walls_for_network();
    track.save_to_file(path).expect("failed to save rx_hilltop_leap");
    println!("  Successfully saved rx_hilltop_leap with TrackNetwork.");
}

fn build_canyon_flyer() {
    let path = "tracks/classic/rx_canyon_flyer.json";
    println!("Processing {}...", path);
    let mut track = Track::load_from_file(path).expect("failed to load rx_canyon_flyer");
    track.checkpoints.retain(|cp| !cp.is_joker);

    let wps = &track.spline.waypoints;
    // Split at wp 14 (113.1, -188.6), Merge at wp 25 (-139.1, -235.1)
    let seg0_wps = wps[0..=14].to_vec();
    let seg1_wps = wps[14..=25].to_vec();

    let seg2_wps = vec![
        TrackWaypoint::new(Vec2::new(113.1, -188.6), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(75.0, -231.8), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(30.0, -276.2), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(-20.0, -303.5), 12.0)
            .with_surface(SurfaceType::Concrete)
            .with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(-70.0, -308.5), 12.0)
            .with_surface(SurfaceType::Concrete)
            .with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(-110.0, -284.0), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(-139.1, -235.1), 12.0)
            .with_surface(SurfaceType::Dirt)
            .with_elevation(0.0),
    ];

    let mut seg3_wps = wps[25..=53].to_vec();
    seg3_wps.push(wps[0].clone());

    let seg0 = RoadSegment::new(SegmentId(0), "Start Straight & Technical Dirt", seg0_wps)
        .with_junctions(Some(SocketId::new(JunctionId(1), 0)), Some(SocketId::new(JunctionId(0), 0)));
    let seg1 = RoadSegment::new(SegmentId(1), "Canyon Gap Jump & Whoops Main", seg1_wps)
        .with_junctions(Some(SocketId::new(JunctionId(0), 0)), Some(SocketId::new(JunctionId(1), 0)));
    let seg2 = RoadSegment::new(SegmentId(2), "Canyon Rim Hairpin Joker Detour", seg2_wps)
        .with_junctions(Some(SocketId::new(JunctionId(0), 1)), Some(SocketId::new(JunctionId(1), 1)));
    let seg3 = RoadSegment::new(SegmentId(3), "West Straight & Asphalt Esses Return", seg3_wps)
        .with_junctions(Some(SocketId::new(JunctionId(1), 0)), Some(SocketId::new(JunctionId(0), 0)));

    let split_sock_in = make_socket(Vec2::new(113.1, -188.6), Vec2::new(-1.0, 0.0), 12.0, SurfaceType::Dirt, 0.0);
    let split_sock_e0 = make_socket(Vec2::new(113.1, -188.6), Vec2::new(-1.0, 0.0), 12.0, SurfaceType::Dirt, 0.0);
    let split_sock_e1 = make_socket(Vec2::new(113.1, -188.6), Vec2::new(-0.662, -0.749), 12.0, SurfaceType::Dirt, 0.0);

    let split_junction = RoadJunction::split(
        JunctionId(0),
        "Canyon Joker Split",
        split_sock_in,
        vec![split_sock_e0, split_sock_e1],
        Some(GoreConfig::new(
            Vec2::new(113.1, -188.6),
            18.0,
            15.0,
            BarrierType::TireWall,
        )),
    );

    let merge_sock_i0 = make_socket(Vec2::new(-139.1, -235.1), Vec2::new(-1.0, 0.0), 12.0, SurfaceType::Dirt, 0.0);
    let merge_sock_i1 = make_socket(Vec2::new(-139.1, -235.1), Vec2::new(-0.51, 0.86), 12.0, SurfaceType::Dirt, 0.0);
    let merge_sock_eg = make_socket(Vec2::new(-139.1, -235.1), Vec2::new(-1.0, 0.0), 12.0, SurfaceType::Dirt, 0.0);

    let merge_junction = RoadJunction::merge(
        JunctionId(1),
        "Canyon Joker Merge",
        vec![merge_sock_i0, merge_sock_i1],
        merge_sock_eg,
        Some(MergeConfig {
            convergence_point: Vec2::new(-139.1, -235.1),
            merge_angle: 16.0,
            merge_length: 16.0,
        }),
    );

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

        if d1 < d0 && d1 < d3 && p_seg1.progress_distance > 1.0 && p_seg1.progress_distance < seg1.length - 1.0 {
            cp.segment_id = Some(SegmentId(1));
        } else if d3 < d0 && d3 < d1 && p_seg3.progress_distance > 1.0 {
            cp.segment_id = Some(SegmentId(3));
        } else {
            cp.segment_id = Some(SegmentId(0));
        }
    }

    let joker_cp_id = track.checkpoints.len();
    let joker_gate = LineSegment::new(
        Vec2::new(-20.0, -303.5 - 6.0),
        Vec2::new(-20.0, -303.5 + 6.0),
    );
    let joker_cp = Checkpoint::new(
        joker_cp_id,
        joker_gate,
        Vec2::new(-1.0, 0.0),
        1,
        false,
    )
    .with_segment(SegmentId(2))
    .with_joker(true)
    .with_elevation(0.0);
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

    network.validate().expect("rx_canyon_flyer TrackNetwork validation failed");

    let len_main = network.layouts[0].total_lap_length;
    let len_joker = network.layouts[1].total_lap_length;
    let delta = len_joker - len_main;
    println!("  Main length: {:.1} m, Joker length: {:.1} m, Delta: {:.1} m", len_main, len_joker, delta);

    track.network = Some(network);
    track.trim_walls_for_network();
    track.save_to_file(path).expect("failed to save rx_canyon_flyer");
    println!("  Successfully saved rx_canyon_flyer with TrackNetwork.");
}

fn main() {
    println!("Building Classic RX Joker track networks...");
    build_quarry_sprint();
    build_hilltop_leap();
    build_canyon_flyer();
    println!("All Classic RX Joker track networks successfully built!");
}
