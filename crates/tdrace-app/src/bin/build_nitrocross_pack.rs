//! Builds the 7 official North American Rallycross / Nitrocross (Tier 6) circuits from
//! real-world OpenStreetMap survey data (`assets/osm/*.osm`) and laser-scanned iRacing RX
//! multi-layer SVG track maps:
//! - `utah_umc_rx` (`way/343798146` + `way/343798140` — Utah Motorsports Campus 112-ft Mega Gap Jump & Dual X-Joker Laps)
//! - `phoenix_rx` (`iRacing 306` / `#472` — Phoenix Raceway Rallycross, Avondale, Arizona, `1,352 m`)
//! - `daytona_rx` (`iRacing 293` — Daytona International Speedway Long Rallycross, Daytona Beach, Florida, `1,529 m`)
//! - `sonoma_rx` (`iRacing 312` — Sonoma Raceway Rallycross, Sonoma, California, `1,352 m`)
//! - `atlanta_rx` (`iRacing 323` — Atlanta Motor Speedway Long Rallycross, Hampton, Georgia, `1,320 m`)
//! - `charlotte_rx` (`iRacing 558` — Charlotte Motor Speedway Rallycross Long, Concord, North Carolina, `1,159 m`)
//! - `iowa_rx` (`iRacing 295` — Iowa Speedway Rallycross, Newton, Iowa, `1,046 m`)

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
use tdrace_core::track::scenery::{
    Building, BuildingStyle, Grandstand, GrandstandStyle, Rock, RockType, Tree, TreeType,
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
    road_width: f32,
    mega_half_len: f32,
    mega_gap_len: f32,
    mega_height: f32,
    mega_angle_deg: f32,
    mega_launch_speed: f32,
    mega_wp_pair: (usize, usize),
    minor_wp_pair: (usize, usize),
    chute_side: ChuteSide,
    gore_side: f32,
    mega_jump_name: &'static str,
    minor_jump_name: &'static str,
    joker_name: &'static str,
}

const VENUES: &[NitroVenueSpec] = &[
    NitroVenueSpec {
        slug: "utah_umc_rx",
        name: "Utah Motorsports Campus Nitro RX",
        description: "Flagship high-desert Nitrocross stadium circuit on the Utah Motorsports Campus East Course featuring the 112-foot Mega Gap Jump over the X-crossing dual Joker Laps, Talladega banked S-berms, Wind Up tarmac turn, and parallel North Straight dirt chicane.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Utah_Motorsports_Campus",
        osm_url: "https://www.openstreetmap.org/way/343798146",
        tag: "NITROCROSS UTAH 130FT MEGA GAP",
        default_surface: SurfaceType::Dirt,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        road_width: 14.0,
        mega_half_len: 30.0,
        mega_gap_len: 34.0,
        mega_height: 2.85,
        mega_angle_deg: 7.9,
        mega_launch_speed: 3.0,
        mega_wp_pair: (10, 17),
        minor_wp_pair: (24, 25),
        chute_side: ChuteSide::Right,
        gore_side: 1.0,
        mega_jump_name: "Utah UMC RX 130-ft Desert Mega Gap Jump",
        minor_jump_name: "Utah UMC RX Infield Step-Up Tabletop",
        joker_name: "Utah UMC Underpass X-Joker 1 (North-West to South-Bowl)",
    },
    NitroVenueSpec {
        slug: "phoenix_rx",
        name: "Phoenix Raceway Rallycross",
        description: "Sonoran Desert speedway-infield rallycross circuit in Avondale, Arizona combining the fast Phoenix Raceway dogleg tarmac with a technical Turn 2-4 dirt hairpin complex, southern Joker Lap corridor, and high-speed western dirt jump chute.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Phoenix_Raceway",
        osm_url: "https://www.openstreetmap.org/way/28895817",
        tag: "AMERICAN RX PHOENIX DESERT",
        default_surface: SurfaceType::Dirt,
        runoff_surface: Some(SurfaceType::PackedSand),
        primary_surface: SurfaceType::Dirt,
        road_width: 13.8,
        mega_half_len: 24.0,
        mega_gap_len: 24.0,
        mega_height: 2.60,
        mega_angle_deg: 7.6,
        mega_launch_speed: 2.8,
        mega_wp_pair: (30, 31),
        minor_wp_pair: (18, 19),
        chute_side: ChuteSide::Right,
        gore_side: 1.0,
        mega_jump_name: "Phoenix RX Turn 8 Desert Stadium Gap Jump",
        minor_jump_name: "Phoenix RX Infield Step-Up Tabletop",
        joker_name: "Phoenix RX Southern Dirt Joker Detour",
    },
    NitroVenueSpec {
        slug: "daytona_rx",
        name: "Daytona International Speedway RX",
        description: "Full-length 0.95-mile Daytona International Speedway infield rallycross circuit featuring the high-speed road course tarmac launch, tightening Turn 3-5 dirt horseshoe, long southeastern stadium jump straightaway, and West hairpin Joker Lap.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Daytona_International_Speedway",
        osm_url: "https://www.openstreetmap.org/way/39869745",
        tag: "AMERICAN RX DAYTONA LONG",
        default_surface: SurfaceType::Grass,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        road_width: 14.0,
        mega_half_len: 25.0,
        mega_gap_len: 25.0,
        mega_height: 2.65,
        mega_angle_deg: 7.7,
        mega_launch_speed: 2.9,
        mega_wp_pair: (10, 11),
        minor_wp_pair: (24, 25),
        chute_side: ChuteSide::Right,
        gore_side: -1.0,
        mega_jump_name: "Daytona RX Infield Horseshoe Gap Jump",
        minor_jump_name: "Daytona RX Backstretch Dirt Tabletop",
        joker_name: "Daytona RX Turn 9 Outer Loop Joker Detour",
    },
    NitroVenueSpec {
        slug: "sonoma_rx",
        name: "Sonoma Raceway Rallycross",
        description: "Hilly Wine Country stadium rallycross course in Sonoma, California featuring a sweeping 180-degree western dirt bowl, steep uphill tarmac climb to Turn 1, plunging downhill dirt jump chute, and Turn 2 Joker Lap split.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Sonoma_Raceway",
        osm_url: "https://www.openstreetmap.org/way/32919446",
        tag: "AMERICAN RX SONOMA HILLSIDE",
        default_surface: SurfaceType::Dirt,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        road_width: 13.8,
        mega_half_len: 24.0,
        mega_gap_len: 23.5,
        mega_height: 2.60,
        mega_angle_deg: 7.6,
        mega_launch_speed: 2.8,
        mega_wp_pair: (9, 10),
        minor_wp_pair: (22, 23),
        chute_side: ChuteSide::Right,
        gore_side: 1.0,
        mega_jump_name: "Sonoma RX Hillside Dirt Gap Jump",
        minor_jump_name: "Sonoma RX Turn 1 Downhill Tabletop",
        joker_name: "Sonoma RX Turn 2 Outer Corner Joker Detour",
    },
    NitroVenueSpec {
        slug: "atlanta_rx",
        name: "Atlanta Motor Speedway RX",
        description: "Full-length 0.82-mile Atlanta Motor Speedway rallycross circuit in Hampton, Georgia combining the fast frontstretch tarmac with the banked Turn 3 eastern clay bowl, deep southern Turn 4-5 hairpin Joker Lap, and 220-meter central dirt jump straightaway.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Atlanta_Motor_Speedway",
        osm_url: "https://www.openstreetmap.org/way/24788504",
        tag: "AMERICAN RX ATLANTA GEORGIA CLAY",
        default_surface: SurfaceType::Grass,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        road_width: 14.0,
        mega_half_len: 26.0,
        mega_gap_len: 26.5,
        mega_height: 2.70,
        mega_angle_deg: 7.7,
        mega_launch_speed: 2.9,
        mega_wp_pair: (15, 16),
        minor_wp_pair: (18, 19),
        chute_side: ChuteSide::Right,
        gore_side: -1.0,
        mega_jump_name: "Atlanta RX Georgia Red-Clay Gap Jump",
        minor_jump_name: "Atlanta RX Infield Rhythm Tabletop",
        joker_name: "Atlanta RX Turn 4-5 Hairpin Joker Detour",
    },
    NitroVenueSpec {
        slug: "charlotte_rx",
        name: "Charlotte Motor Speedway RX",
        description: "Compact 0.72-mile stadium rallycross circuit at Charlotte Motor Speedway in Concord, North Carolina featuring a fast tarmac west chute, flat-top northern gravel straight, plunging central dirt jump straightaway, and sweeping southeastern kidney bowl Joker Lap.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Charlotte_Motor_Speedway",
        osm_url: "https://www.openstreetmap.org/way/31245901",
        tag: "AMERICAN RX CHARLOTTE ROVAL",
        default_surface: SurfaceType::Grass,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        road_width: 13.8,
        mega_half_len: 24.0,
        mega_gap_len: 23.5,
        mega_height: 2.60,
        mega_angle_deg: 7.6,
        mega_launch_speed: 2.8,
        mega_wp_pair: (12, 13),
        minor_wp_pair: (7, 8),
        chute_side: ChuteSide::Right,
        gore_side: -1.0,
        mega_jump_name: "Charlotte RX Infield Stadium Gap Jump",
        minor_jump_name: "Charlotte RX North Straight Tabletop",
        joker_name: "Charlotte RX Southern Bowl Joker Detour",
    },
    NitroVenueSpec {
        slug: "iowa_rx",
        name: "Iowa Speedway Rallycross",
        description: "High-banked 0.65-mile bullring speedway rallycross circuit in Newton, Iowa combining the 14-degree banked Turn 1-2 oval asphalt with a deep infield dirt keyhole switchback, northern perimeter Joker Lap, and sweeping western dirt chute.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Iowa_Speedway",
        osm_url: "https://www.openstreetmap.org/way/126950311",
        tag: "AMERICAN RX IOWA BULLRING",
        default_surface: SurfaceType::Grass,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        road_width: 13.8,
        mega_half_len: 23.0,
        mega_gap_len: 22.5,
        mega_height: 2.55,
        mega_angle_deg: 7.5,
        mega_launch_speed: 2.8,
        mega_wp_pair: (7, 8),
        minor_wp_pair: (12, 13),
        chute_side: ChuteSide::Left,
        gore_side: -1.0,
        mega_jump_name: "Iowa RX Infield Keyhole Gap Jump",
        minor_jump_name: "Iowa RX Turn 5 Switchback Tabletop",
        joker_name: "Iowa RX Northern Perimeter Joker Detour",
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

/// Builds a straight 8-waypoint `Seg 1` corridor from `p_split` to `p_merge` with `cross_idx = split_idx + 3`
/// placed at 45% along the straightaway so `span_in >= 89m` and `span_out >= 109m`.
fn push_mega_straight(
    raw: &mut Vec<(Vec2, SurfaceType, f32, f32)>,
    p_split: Vec2,
    p_merge: Vec2,
    surf: SurfaceType,
) -> (usize, usize, usize) {
    let split_idx = raw.len();
    let fracs = [0.0, 0.15, 0.30, 0.45, 0.59, 0.73, 0.87, 1.0];
    for &f in &fracs {
        let pt = p_split.lerp(p_merge, f);
        raw.push((pt, surf, 0.0, 0.0));
    }
    let cross_idx = split_idx + 3;
    let merge_idx = split_idx + 7;
    (split_idx, cross_idx, merge_idx)
}

const UTAH_SPLIT1_IDX: usize = 5;

fn build_utah_umc_rx() -> (
    Vec<(Vec2, SurfaceType, f32, f32)>,
    usize,
    usize,
    usize,
    Vec<(Vec2, SurfaceType, f32, f32)>,
) {
    use SurfaceType::{Asphalt, Dirt};
    let mut raw = vec![
        // 0..=5: North Straight Start/Finish (heading West -X along y = 72m, way/343798146) into East One (way/343798140)
        (Vec2::new(-140.0, 72.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-186.0, 72.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-230.0, 70.0), Asphalt, 0.0, -6.0),
        (Vec2::new(-262.0, 52.0), Dirt, 0.0, -12.0),
        (Vec2::new(-272.0, 26.0), Dirt, 0.0, -14.0),
        (Vec2::new(-252.0, 10.0), Dirt, 0.0, 0.0), // wp 5 (UTAH_SPLIT1_IDX): Joker 1 splits left/NE onto upper straight
        // 6..=9: East One Outer Dirt Chicane (Seg 4 on Main Track)
        (Vec2::new(-228.0, -8.0), Dirt, 0.0, -12.0),
        (Vec2::new(-202.0, -8.0), Dirt, 0.0, 10.0),
        (Vec2::new(-180.0, 4.0), Dirt, 0.0, 8.0),
        (Vec2::new(-162.0, 6.0), Dirt, 0.0, 0.0),
    ];
    // 10..=17: 112-ft (34.0m Open Chasm) Mega Gap Jump Straightaway (Seg 1 on Main Track, heading East +X along y = 6m)
    let (split_idx, cross_idx, merge_idx) = push_mega_straight(
        &mut raw,
        Vec2::new(-144.0, 6.0),
        Vec2::new(76.0, 6.0),
        Dirt,
    );
    // 18..=41: Seg 3 — Eastern "Talladega" Banked Dirt S-Berms -> South Straight Tabletop -> Wind Up Asphalt -> Release Dirt Chicane -> North Parallel Dirt Section
    raw.extend_from_slice(&[
        (Vec2::new(104.0, 2.0), Dirt, 1.2, -16.0),
        (Vec2::new(118.0, -18.0), Dirt, 2.4, -22.0),
        (Vec2::new(100.0, -38.0), Dirt, 1.5, -16.0),
        (Vec2::new(66.0, -38.0), Dirt, 0.5, 0.0),
        (Vec2::new(34.0, -36.0), Dirt, 0.8, 12.0),
        (Vec2::new(16.0, -52.0), Dirt, 1.6, 18.0),
        // Long Southern Dirt Straight (wp 24..=26, collinear along SE heading) with Step-Up Tabletop Jump on (24, 25)
        (Vec2::new(26.0, -72.0), Dirt, 0.0, 0.0),
        (Vec2::new(50.0, -96.0), Dirt, 0.0, 0.0),
        (Vec2::new(74.0, -120.0), Dirt, 0.0, 0.0),
        (Vec2::new(98.0, -130.0), Dirt, 0.8, 14.0),
        (Vec2::new(118.0, -114.0), Dirt, 0.4, 10.0),
        (Vec2::new(120.0, -90.0), Asphalt, 0.0, -10.0),
        (Vec2::new(139.0, -64.0), Asphalt, 0.0, -8.0),
        (Vec2::new(172.0, -40.0), Asphalt, 0.0, 0.0),
        (Vec2::new(186.0, -8.0), Dirt, 1.0, 12.0),
        (Vec2::new(178.0, 24.0), Dirt, 1.2, 10.0),
        (Vec2::new(156.0, 50.0), Dirt, 0.6, 8.0),
        (Vec2::new(128.0, 66.0), Asphalt, 0.0, 6.0),
        // North Straight Added Parallel Dirt Section (Main Track, bypassing blocked asphalt straight)
        (Vec2::new(100.0, 64.0), Asphalt, 0.0, 0.0),
        (Vec2::new(74.0, 54.0), Dirt, 0.0, 0.0),
        (Vec2::new(40.0, 54.0), Dirt, 0.0, 0.0),
        (Vec2::new(12.0, 66.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-20.0, 72.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-56.0, 72.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-92.0, 72.0), Asphalt, 0.0, 0.0),
    ]);
    (raw, split_idx, cross_idx, merge_idx, Vec::new())
}

fn build_phoenix_rx() -> (
    Vec<(Vec2, SurfaceType, f32, f32)>,
    usize,
    usize,
    usize,
    Vec<(Vec2, SurfaceType, f32, f32)>,
) {
    use SurfaceType::{Asphalt, Dirt};
    let raw = vec![
        // 0..=6: Phoenix Raceway Dogleg Start/Finish & Turn 1 Sweeper (Asphalt)
        (Vec2::new(12.0, 80.0), Asphalt, 0.0, 0.0),
        (Vec2::new(54.0, 84.0), Asphalt, 0.0, 0.0),
        (Vec2::new(96.0, 88.0), Asphalt, 0.0, -4.0),
        (Vec2::new(142.0, 91.0), Asphalt, 0.5, -8.0),
        (Vec2::new(188.0, 91.0), Asphalt, 0.8, -10.0),
        (Vec2::new(230.0, 78.0), Asphalt, 0.5, -8.0),
        (Vec2::new(262.0, 52.0), Asphalt, 0.0, -6.0),
        // 7..=14: Turn 2-3 Technical Infield Dirt Hairpin Chicane & Turn 4 Southern Bowl
        (Vec2::new(280.0, 22.0), Dirt, 0.0, -10.0),
        (Vec2::new(280.0, -8.0), Dirt, 0.0, -14.0),
        (Vec2::new(260.0, -34.0), Dirt, 0.0, -8.0),
        (Vec2::new(232.0, -50.0), Dirt, 0.0, 8.0),
        (Vec2::new(204.0, -72.0), Dirt, 0.0, -10.0),
        (Vec2::new(174.0, -85.0), Dirt, 0.0, -14.0),
        (Vec2::new(140.0, -84.0), Dirt, 0.0, -10.0),
        (Vec2::new(106.0, -72.0), Dirt, 0.0, 0.0), // wp 14 (split_idx): Joker splits left onto Southern [Joker] corridor
        // 15..=22: Seg 1 — Turn 5 Infield Flick & Collinear Central Straight (minor_wp_pair = (18, 19))
        (Vec2::new(68.0, -44.0), Dirt, 0.0, 8.0),
        (Vec2::new(34.0, -12.0), Dirt, 0.0, -8.0),
        (Vec2::new(0.0, -3.0), Dirt, 0.0, 0.0),
        (Vec2::new(-38.0, -3.0), Dirt, 0.0, 0.0),
        (Vec2::new(-76.0, -3.0), Dirt, 0.0, 0.0),
        (Vec2::new(-114.0, -3.0), Dirt, 0.0, 0.0),
        (Vec2::new(-148.0, -10.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-178.0, -28.0), Dirt, 0.0, 0.0), // wp 22 (merge_idx): Turn 6 merge
        // 23..=28: Western Turn 7-8 Banked Dirt Hairpin Bowl
        (Vec2::new(-206.0, -56.0), Dirt, 0.0, -8.0),
        (Vec2::new(-228.0, -86.0), Dirt, 0.5, -14.0),
        (Vec2::new(-256.0, -93.0), Dirt, 1.0, -16.0),
        (Vec2::new(-278.0, -74.0), Dirt, 1.0, -16.0),
        (Vec2::new(-282.0, -44.0), Dirt, 0.5, -12.0),
        (Vec2::new(-268.0, -12.0), Dirt, 0.0, 0.0),
        // 29..=32: Collinear 135m NE Dirt Mega Jump Straightaway (mega_wp_pair = (30, 31))
        (Vec2::new(-244.0, 4.0), Dirt, 0.0, 0.0),
        (Vec2::new(-204.0, 24.0), Dirt, 0.0, 0.0),
        (Vec2::new(-164.0, 44.0), Dirt, 0.0, 0.0),
        (Vec2::new(-124.0, 64.0), Dirt, 0.0, 0.0),
        // 33..=35: Turn 9 Dogleg Asphalt Rejoin into Start/Finish
        (Vec2::new(-82.0, 70.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-38.0, 75.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-12.0, 78.0), Asphalt, 0.0, 0.0),
    ];
    let joker_interior = vec![
        (Vec2::new(62.0, -64.0), Dirt, 0.0, 0.0),
        (Vec2::new(20.0, -76.0), Dirt, 0.0, -6.0),
        (Vec2::new(-28.0, -86.0), Dirt, 0.0, -10.0),
        (Vec2::new(-76.0, -82.0), Dirt, 0.0, -10.0),
        (Vec2::new(-118.0, -58.0), Dirt, 0.0, 6.0),
        (Vec2::new(-148.0, -32.0), Dirt, 0.0, 0.0),
    ];
    (raw, 14, 18, 22, joker_interior)
}

fn build_daytona_rx() -> (
    Vec<(Vec2, SurfaceType, f32, f32)>,
    usize,
    usize,
    usize,
    Vec<(Vec2, SurfaceType, f32, f32)>,
) {
    use SurfaceType::{Asphalt, Dirt};
    let raw = vec![
        // 0..=6: Daytona Infield Start/Finish & Turn 1-2 Tarmac Section
        (Vec2::new(-196.0, 126.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-158.0, 128.0), Asphalt, 0.0, -8.0),
        (Vec2::new(-126.0, 108.0), Asphalt, 0.0, -10.0),
        (Vec2::new(-94.0, 88.0), Asphalt, 0.0, 8.0),
        (Vec2::new(-48.0, 84.0), Asphalt, 0.0, 0.0),
        (Vec2::new(0.0, 83.0), Asphalt, 0.0, 0.0),
        (Vec2::new(48.0, 85.0), Asphalt, 0.0, 0.0),
        // 7..=9: Turn 3-4-5 Dirt Horseshoe
        (Vec2::new(92.0, 100.0), Dirt, 0.0, 10.0),
        (Vec2::new(136.0, 127.0), Dirt, 0.8, -14.0),
        (Vec2::new(178.0, 118.0), Dirt, 0.5, -12.0),
        // 10..=13: Collinear 140m SE Dirt Mega Jump Straightaway (mega_wp_pair = (10, 11))
        (Vec2::new(192.0, 84.0), Dirt, 0.0, 0.0),
        (Vec2::new(206.0, 38.0), Dirt, 0.0, 0.0),
        (Vec2::new(220.0, -8.0), Dirt, 0.0, 0.0),
        (Vec2::new(234.0, -54.0), Dirt, 0.0, 0.0),
        // 14..=19: Turn 6-7 SE Dirt Hairpin Bowl
        (Vec2::new(272.0, -76.0), Dirt, 0.5, 10.0),
        (Vec2::new(308.0, -96.0), Dirt, 1.2, -16.0),
        (Vec2::new(302.0, -118.0), Dirt, 1.2, -16.0),
        (Vec2::new(262.0, -129.0), Dirt, 0.8, -14.0),
        (Vec2::new(222.0, -118.0), Dirt, 0.4, -10.0),
        (Vec2::new(186.0, -82.0), Dirt, 0.0, 0.0),
        // 20..=23: NW Return Chute & Turn 8 Kink
        (Vec2::new(146.0, -44.0), Dirt, 0.0, 0.0),
        (Vec2::new(98.0, -18.0), Dirt, 0.0, 6.0),
        (Vec2::new(42.0, -9.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-12.0, 12.0), Asphalt, 0.0, 8.0),
        // 24..=28: Collinear 184m Westbound Dirt Tabletop Straightaway (minor_wp_pair = (24, 25))
        (Vec2::new(-56.0, 32.0), Dirt, 0.0, 0.0),
        (Vec2::new(-102.0, 32.0), Dirt, 0.0, 0.0),
        (Vec2::new(-148.0, 32.0), Dirt, 0.0, 0.0),
        (Vec2::new(-194.0, 32.0), Dirt, 0.0, 0.0),
        (Vec2::new(-240.0, 34.0), Dirt, 0.0, 0.0), // wp 28 (split_idx): Turn 9 / [Joker] split
        // 29..=31: Seg 1 — Inner Daytona Turn 9 Cut
        (Vec2::new(-268.0, 60.0), Dirt, 0.0, 12.0),
        (Vec2::new(-262.0, 98.0), Dirt, 0.0, 10.0),
        (Vec2::new(-228.0, 122.0), Asphalt, 0.0, 0.0), // wp 31 (merge_idx): Start/Finish straight merge
    ];
    let joker_interior = vec![
        (Vec2::new(-306.0, 48.0), Dirt, 0.0, 14.0),
        (Vec2::new(-332.0, 76.0), Dirt, 0.5, 16.0),
        (Vec2::new(-318.0, 108.0), Dirt, 0.5, 14.0),
        (Vec2::new(-282.0, 122.0), Dirt, 0.0, 10.0),
    ];
    (raw, 28, 29, 31, joker_interior)
}

fn build_sonoma_rx() -> (
    Vec<(Vec2, SurfaceType, f32, f32)>,
    usize,
    usize,
    usize,
    Vec<(Vec2, SurfaceType, f32, f32)>,
) {
    use SurfaceType::{Asphalt, Dirt};
    let raw = vec![
        // 0..=3: Bottom Start/Finish Straight (Asphalt, heading West -X)
        (Vec2::new(-48.0, -115.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-94.0, -115.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-140.0, -113.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-186.0, -109.0), Asphalt, 0.0, 6.0),
        // 4..=8: Turn 3 Sweeping 180-deg Western Dirt Bowl
        (Vec2::new(-226.0, -92.0), Dirt, 0.8, 14.0),
        (Vec2::new(-252.0, -56.0), Dirt, 1.5, 16.0),
        (Vec2::new(-252.0, -12.0), Dirt, 1.8, 16.0),
        (Vec2::new(-228.0, 24.0), Dirt, 1.2, 14.0),
        (Vec2::new(-186.0, 37.0), Dirt, 0.6, 10.0),
        // 9..=12: Collinear 126m SE Dirt Mega Jump Straightaway (mega_wp_pair = (9, 10))
        (Vec2::new(-144.0, 28.0), Dirt, 0.0, 0.0),
        (Vec2::new(-104.0, 13.0), Dirt, 0.0, 0.0),
        (Vec2::new(-64.0, -2.0), Dirt, 0.0, 0.0),
        (Vec2::new(-24.0, -17.0), Dirt, 0.0, 0.0),
        // 13..=20: Turn 4-5 Hillside Climb to Turn 1 Summit Hairpin (+6.5m)
        (Vec2::new(18.0, -15.0), Asphalt, 1.0, -10.0),
        (Vec2::new(58.0, 2.0), Asphalt, 2.2, -12.0),
        (Vec2::new(94.0, 35.0), Asphalt, 3.8, -8.0),
        (Vec2::new(128.0, 80.0), Asphalt, 5.2, 0.0),
        (Vec2::new(164.0, 116.0), Asphalt, 6.5, 12.0),
        (Vec2::new(208.0, 110.0), Asphalt, 6.0, 10.0),
        (Vec2::new(244.0, 98.0), Dirt, 5.0, 14.0),
        (Vec2::new(252.0, 74.0), Dirt, 3.5, 14.0),
        // 21..=24: Collinear 114m N-to-S Downhill Dirt Chute (minor_wp_pair = (22, 23))
        (Vec2::new(225.0, 48.0), Dirt, 1.5, 0.0),
        (Vec2::new(225.0, 14.0), Dirt, 0.0, 0.0),
        (Vec2::new(225.0, -20.0), Dirt, 0.0, 0.0),
        (Vec2::new(225.0, -54.0), Dirt, 0.0, 0.0), // wp 24 (split_idx): Turn 2 / [Joker] split
        // 25..=28: Seg 1 — Diagonal Sonoma [Joker] Cut
        (Vec2::new(192.0, -76.0), Dirt, 0.0, 8.0),
        (Vec2::new(136.0, -90.0), Dirt, 0.0, 0.0),
        (Vec2::new(72.0, -102.0), Dirt, 0.0, 0.0),
        (Vec2::new(8.0, -115.0), Asphalt, 0.0, 0.0), // wp 28 (merge_idx): Bottom straight merge
    ];
    let joker_interior = vec![
        (Vec2::new(220.0, -98.0), Dirt, 0.0, 12.0),
        (Vec2::new(192.0, -122.0), Dirt, 0.0, 14.0),
        (Vec2::new(146.0, -124.0), Dirt, 0.0, 8.0),
        (Vec2::new(96.0, -121.0), Dirt, 0.0, 0.0),
        (Vec2::new(50.0, -118.0), Dirt, 0.0, 0.0),
    ];
    (raw, 24, 26, 28, joker_interior)
}

fn build_atlanta_rx() -> (
    Vec<(Vec2, SurfaceType, f32, f32)>,
    usize,
    usize,
    usize,
    Vec<(Vec2, SurfaceType, f32, f32)>,
) {
    use SurfaceType::{Asphalt, Dirt};
    let raw = vec![
        // 0..=5: Atlanta Frontstretch/Pit Straight & Turn 1-2 (Asphalt, heading East +X)
        (Vec2::new(22.0, 107.0), Asphalt, 0.0, 0.0),
        (Vec2::new(68.0, 102.0), Asphalt, 0.0, 0.0),
        (Vec2::new(115.0, 96.0), Asphalt, 0.0, 0.0),
        (Vec2::new(162.0, 88.0), Asphalt, 0.0, -8.0),
        (Vec2::new(196.0, 66.0), Asphalt, 0.5, -12.0),
        (Vec2::new(214.0, 36.0), Dirt, 0.5, 8.0),
        // 6..=9: Turn 3 Eastern Georgia Red-Clay Bowl
        (Vec2::new(242.0, 10.0), Dirt, 1.2, -14.0),
        (Vec2::new(250.0, -22.0), Dirt, 1.5, -16.0),
        (Vec2::new(232.0, -48.0), Dirt, 1.0, -12.0),
        (Vec2::new(196.0, -57.0), Dirt, 0.0, 0.0), // wp 9 (split_idx): Turn 4 / [Joker] split
        // 10..=13: Seg 1 — Diagonal Cut to Turn 6
        (Vec2::new(158.0, -44.0), Dirt, 0.0, 0.0),
        (Vec2::new(124.0, -22.0), Dirt, 0.0, 0.0),
        (Vec2::new(96.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(68.0, 18.0), Dirt, 0.0, 0.0), // wp 13 (merge_idx): Turn 6 merge
        // 14..=20: Collinear 240m Central E-to-W Dirt Jump Straightaway (mega_wp_pair = (15, 16), minor_wp_pair = (18, 19))
        (Vec2::new(34.0, 28.4), Dirt, 0.0, 0.0),
        (Vec2::new(-6.0, 28.4), Dirt, 0.0, 0.0),
        (Vec2::new(-46.0, 28.4), Dirt, 0.0, 0.0),
        (Vec2::new(-86.0, 28.4), Dirt, 0.0, 0.0),
        (Vec2::new(-126.0, 28.4), Dirt, 0.0, 0.0),
        (Vec2::new(-166.0, 28.4), Dirt, 0.0, 0.0),
        (Vec2::new(-206.0, 28.4), Dirt, 0.0, 0.0),
        // 21..=28: Turn 7-8 Western Oval Turnaround & Frontstretch Return
        (Vec2::new(-238.0, 46.0), Asphalt, 0.8, -14.0),
        (Vec2::new(-251.0, 80.0), Asphalt, 1.5, -16.0),
        (Vec2::new(-240.0, 116.0), Asphalt, 1.2, -14.0),
        (Vec2::new(-208.0, 132.0), Asphalt, 0.5, -8.0),
        (Vec2::new(-162.0, 127.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-114.0, 121.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-66.0, 115.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-18.0, 110.0), Asphalt, 0.0, 0.0),
    ];
    let joker_interior = vec![
        (Vec2::new(162.0, -68.0), Dirt, 0.0, -10.0),
        (Vec2::new(134.0, -78.0), Dirt, 0.0, -14.0),
        (Vec2::new(108.0, -64.0), Dirt, 0.0, -12.0),
        (Vec2::new(92.0, -32.0), Dirt, 0.0, -8.0),
    ];
    (raw, 9, 11, 13, joker_interior)
}

fn build_charlotte_rx() -> (
    Vec<(Vec2, SurfaceType, f32, f32)>,
    usize,
    usize,
    usize,
    Vec<(Vec2, SurfaceType, f32, f32)>,
) {
    use SurfaceType::{Asphalt, Dirt};
    let raw = vec![
        // 0..=5: Diagonal Start/Finish Straight & West Vertical Tarmac Chute
        (Vec2::new(-82.0, -10.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-126.0, -1.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-164.0, 10.0), Asphalt, 0.0, -10.0),
        (Vec2::new(-186.0, 40.0), Asphalt, 0.0, -12.0),
        (Vec2::new(-188.0, 80.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-182.0, 118.0), Asphalt, 0.0, -12.0),
        // 6..=9: Collinear 120m North Flat-Top Dirt Straight (minor_wp_pair = (7, 8))
        (Vec2::new(-152.0, 133.0), Dirt, 0.0, 0.0),
        (Vec2::new(-112.0, 133.0), Dirt, 0.0, 0.0),
        (Vec2::new(-72.0, 133.0), Dirt, 0.0, 0.0),
        (Vec2::new(-32.0, 133.0), Dirt, 0.0, 0.0),
        // 10..=15: Collinear 165m Central N-to-S Dirt Mega Jump Straightaway (mega_wp_pair = (12, 13))
        (Vec2::new(-8.0, 116.0), Dirt, 0.0, -8.0),
        (Vec2::new(2.0, 76.0), Dirt, 0.0, 0.0),
        (Vec2::new(12.0, 36.0), Dirt, 0.0, 0.0),
        (Vec2::new(22.0, -4.0), Dirt, 0.0, 0.0),
        (Vec2::new(32.0, -44.0), Dirt, 0.0, 0.0),
        (Vec2::new(48.0, -74.0), Dirt, 0.0, 12.0),
        // 16..=22: SE Kidney Dirt Switchback & Outer Banked Bowl
        (Vec2::new(82.0, -88.0), Dirt, 0.5, 14.0),
        (Vec2::new(116.0, -64.0), Dirt, 1.0, 10.0),
        (Vec2::new(142.0, -30.0), Dirt, 1.5, -12.0),
        (Vec2::new(170.0, -18.0), Dirt, 2.0, -16.0),
        (Vec2::new(188.0, -40.0), Dirt, 2.0, -16.0),
        (Vec2::new(180.0, -76.0), Dirt, 1.2, -14.0),
        (Vec2::new(146.0, -108.0), Dirt, 0.0, 0.0), // wp 22 (split_idx): Southern Joker split
        // 23..=27: Seg 1 — Inner Kidney Exit Chute
        (Vec2::new(102.0, -126.0), Dirt, 0.0, -10.0),
        (Vec2::new(56.0, -130.0), Dirt, 0.0, -12.0),
        (Vec2::new(28.0, -96.0), Dirt, 0.0, -8.0),
        (Vec2::new(8.0, -48.0), Asphalt, 0.0, 8.0),
        (Vec2::new(-38.0, -19.0), Asphalt, 0.0, 0.0), // wp 27 (merge_idx): Start/Finish straight merge
    ];
    let joker_interior = vec![
        (Vec2::new(96.0, -152.0), Dirt, 0.0, -12.0),
        (Vec2::new(46.0, -164.0), Dirt, 0.0, -14.0),
        (Vec2::new(4.0, -150.0), Dirt, 0.0, -14.0),
        (Vec2::new(-10.0, -110.0), Dirt, 0.0, 0.0),
        (Vec2::new(-16.0, -66.0), Dirt, 0.0, 8.0),
    ];
    (raw, 22, 24, 27, joker_interior)
}

fn build_iowa_rx() -> (
    Vec<(Vec2, SurfaceType, f32, f32)>,
    usize,
    usize,
    usize,
    Vec<(Vec2, SurfaceType, f32, f32)>,
) {
    use SurfaceType::{Asphalt, Dirt};
    let raw = vec![
        // 0..=5: Iowa Speedway Turn 1-2 Banked Oval Asphalt (heading NE -> N)
        (Vec2::new(70.0, -26.0), Asphalt, 0.0, 10.0),
        (Vec2::new(100.0, 2.0), Asphalt, 1.0, 14.0),
        (Vec2::new(122.0, 32.0), Asphalt, 1.8, 14.0),
        (Vec2::new(131.0, 58.0), Asphalt, 1.8, 14.0),
        (Vec2::new(114.0, 84.0), Asphalt, 1.2, 12.0),
        (Vec2::new(84.0, 92.0), Dirt, 0.0, 0.0),
        // 6..=9: Turn 3 Infield Entry & Collinear NE-to-SW Dirt Mega Jump Straightaway (mega_wp_pair = (7, 8))
        (Vec2::new(52.0, 74.0), Dirt, 0.0, 0.0),
        (Vec2::new(20.0, 50.0), Dirt, 0.0, 0.0),
        (Vec2::new(-12.0, 26.0), Dirt, 0.0, 0.0),
        (Vec2::new(-44.0, 2.0), Dirt, 0.0, 0.0),
        (Vec2::new(-72.0, -16.0), Dirt, 0.0, -12.0),
        // 11..=14: Turn 5 Hairpin & Collinear SW-to-NE Infield Dirt Tabletop Climb (minor_wp_pair = (12, 13))
        (Vec2::new(-94.0, -4.0), Dirt, 0.0, -16.0),
        (Vec2::new(-90.0, 22.0), Dirt, 0.0, 0.0),
        (Vec2::new(-72.0, 50.0), Dirt, 0.0, 0.0),
        (Vec2::new(-54.0, 78.0), Dirt, 0.0, 0.0), // wp 14 (split_idx): Northern [Joker] split
        // 15..=18: Seg 1 — Inner Turn 6 Apex
        (Vec2::new(-44.0, 104.0), Dirt, 0.6, 12.0),
        (Vec2::new(-62.0, 116.0), Dirt, 0.8, 14.0),
        (Vec2::new(-88.0, 104.0), Dirt, 0.8, 12.0),
        (Vec2::new(-112.0, 72.0), Dirt, 0.5, 10.0), // wp 18 (merge_idx): Turn 6 exit merge
        // 19..=24: Turn 7-8 Sweeping Western Dirt & Southern Oval Rejoin
        (Vec2::new(-130.0, 24.0), Dirt, 0.0, 10.0),
        (Vec2::new(-128.0, -26.0), Dirt, 0.0, 12.0),
        (Vec2::new(-110.0, -72.0), Dirt, 0.0, 12.0),
        (Vec2::new(-72.0, -110.0), Asphalt, 0.5, 14.0),
        (Vec2::new(-18.0, -118.0), Asphalt, 0.5, 14.0),
        (Vec2::new(32.0, -68.0), Asphalt, 0.0, 10.0),
    ];
    let joker_interior = vec![
        (Vec2::new(-18.0, 120.0), Dirt, 0.0, 10.0),
        (Vec2::new(-32.0, 144.0), Dirt, 0.5, 14.0),
        (Vec2::new(-64.0, 144.0), Dirt, 0.5, 14.0),
        (Vec2::new(-94.0, 120.0), Dirt, 0.0, 10.0),
    ];
    (raw, 14, 16, 18, joker_interior)
}

fn build_main_waypoints(
    spec: &NitroVenueSpec,
) -> (
    Vec<TrackWaypoint>,
    usize,
    usize,
    usize,
    Vec<(Vec2, SurfaceType, f32, f32)>,
) {
    let (raw_pts, split_idx, cross_idx, merge_idx, joker_interior) = match spec.slug {
        "utah_umc_rx" => build_utah_umc_rx(),
        "phoenix_rx" => build_phoenix_rx(),
        "daytona_rx" => build_daytona_rx(),
        "sonoma_rx" => build_sonoma_rx(),
        "atlanta_rx" => build_atlanta_rx(),
        "charlotte_rx" => build_charlotte_rx(),
        "iowa_rx" => build_iowa_rx(),
        other => panic!("unknown Nitrocross/American RX venue slug: {}", other),
    };

    let wps: Vec<TrackWaypoint> = raw_pts
        .into_iter()
        .map(|(pt, surf, elev, bank)| {
            let mut wp = TrackWaypoint::new(pt, spec.road_width)
                .with_surface(surf)
                .with_elevation(elev)
                .with_bank_angle(bank);
            wp.left_runoff_surface = spec.runoff_surface;
            wp.right_runoff_surface = spec.runoff_surface;
            wp
        })
        .collect();

    (wps, split_idx, cross_idx, merge_idx, joker_interior)
}

fn assemble_network(
    spec: &NitroVenueSpec,
    track: &Track,
    split_idx: usize,
    merge_idx: usize,
    joker_interior: &[(Vec2, SurfaceType, f32, f32)],
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
        elevation: wps[split_idx].elevation,
        bank_angle: wps[split_idx].bank_angle,
        surface: wps[split_idx].surface.unwrap_or(spec.primary_surface),
    };
    let merge_socket = SplineSocket {
        point: wps[merge_idx].point,
        tangent: m_sample.tangent,
        normal: m_sample.normal,
        width: wps[merge_idx].width,
        elevation: wps[merge_idx].elevation,
        bank_angle: wps[merge_idx].bank_angle,
        surface: wps[merge_idx].surface.unwrap_or(spec.primary_surface),
    };

    let p_split = wps[split_idx].point;
    let p_merge = wps[merge_idx].point;
    let mut joker_wps = Vec::with_capacity(joker_interior.len() + 4);
    let mut wp_start = TrackWaypoint::new(p_split, spec.road_width)
        .with_surface(split_socket.surface)
        .with_elevation(split_socket.elevation)
        .with_bank_angle(split_socket.bank_angle);
    wp_start.left_runoff_surface = spec.runoff_surface;
    wp_start.right_runoff_surface = spec.runoff_surface;
    joker_wps.push(wp_start);

    let mut wp_lead_in = TrackWaypoint::new(p_split + split_socket.tangent * 14.0, 13.0)
        .with_surface(spec.primary_surface)
        .with_elevation(0.0)
        .with_bank_angle(0.0);
    wp_lead_in.left_runoff_surface = spec.runoff_surface;
    wp_lead_in.right_runoff_surface = spec.runoff_surface;
    joker_wps.push(wp_lead_in);

    for &(pt, surf, elev, bank) in joker_interior {
        let mut wp = TrackWaypoint::new(pt, 13.0)
            .with_surface(surf)
            .with_elevation(elev)
            .with_bank_angle(bank);
        wp.left_runoff_surface = spec.runoff_surface;
        wp.right_runoff_surface = spec.runoff_surface;
        joker_wps.push(wp);
    }

    let mut wp_lead_out = TrackWaypoint::new(p_merge - merge_socket.tangent * 14.0, 13.0)
        .with_surface(spec.primary_surface)
        .with_elevation(0.0)
        .with_bank_angle(0.0);
    wp_lead_out.left_runoff_surface = spec.runoff_surface;
    wp_lead_out.right_runoff_surface = spec.runoff_surface;
    joker_wps.push(wp_lead_out);

    let mut wp_end = TrackWaypoint::new(p_merge, spec.road_width)
        .with_surface(merge_socket.surface)
        .with_elevation(merge_socket.elevation)
        .with_bank_angle(merge_socket.bank_angle);
    wp_end.left_runoff_surface = spec.runoff_surface;
    wp_end.right_runoff_surface = spec.runoff_surface;
    joker_wps.push(wp_end);

    let nose_center = wps[split_idx].point
        + s_sample.tangent * 14.0
        + s_sample.normal * spec.gore_side * (spec.road_width * 0.5 + 2.5);
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

    let mut seg0 = RoadSegment::new(SegmentId(0), "Start / Finish & Technical Sector", seg0_wps);
    seg0.entry_junction = Some(SocketId::new(JunctionId(1), 0));
    seg0.exit_junction = Some(SocketId::new(JunctionId(0), 0));
    seg0.recompute_samples(None, Some(&split_socket));

    let mut seg1 = RoadSegment::new(SegmentId(1), "Main Circuit Sector", seg1_wps);
    seg1.entry_junction = Some(SocketId::new(JunctionId(0), 0));
    seg1.exit_junction = Some(SocketId::new(JunctionId(1), 0));
    seg1.recompute_samples(Some(&split_socket), Some(&merge_socket));

    let mut seg2 = RoadSegment::new(SegmentId(2), spec.joker_name, joker_wps);
    seg2.entry_junction = Some(SocketId::new(JunctionId(0), 1));
    seg2.exit_junction = Some(SocketId::new(JunctionId(1), 1));
    seg2.recompute_samples(Some(&split_socket), Some(&merge_socket));

    let mut seg3 = RoadSegment::new(SegmentId(3), "Final Bowl & Return Chute", seg3_wps);
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

fn build_utah_x_joker_network(
    spec: &NitroVenueSpec,
    track: &Track,
    split1_idx: usize,
    split2_idx: usize,
    cross_idx: usize,
    merge_idx: usize,
) -> TrackNetwork {
    let wps = &track.spline.waypoints;
    let seg0_wps = wps[0..=split1_idx].to_vec();
    let seg4_wps = wps[split1_idx..=split2_idx].to_vec();
    let seg1_wps = wps[split2_idx..=merge_idx].to_vec();
    let mut seg3_wps = wps[merge_idx..].to_vec();
    seg3_wps.push(wps[0].clone());

    let s1_dist = track.spline.project_point(wps[split1_idx].point).progress_distance;
    let s2_dist = track.spline.project_point(wps[split2_idx].point).progress_distance;
    let m_dist = track.spline.project_point(wps[merge_idx].point).progress_distance;
    let s1_sample = track.spline.sample_at_distance(s1_dist);
    let s2_sample = track.spline.sample_at_distance(s2_dist);
    let m_sample = track.spline.sample_at_distance(m_dist);

    let split1_socket = SplineSocket {
        point: wps[split1_idx].point,
        tangent: s1_sample.tangent,
        normal: s1_sample.normal,
        width: wps[split1_idx].width,
        elevation: 0.0,
        bank_angle: 0.0,
        surface: SurfaceType::Dirt,
    };
    let split2_socket = SplineSocket {
        point: wps[split2_idx].point,
        tangent: s2_sample.tangent,
        normal: s2_sample.normal,
        width: wps[split2_idx].width,
        elevation: 0.0,
        bank_angle: 0.0,
        surface: SurfaceType::Dirt,
    };
    let merge_socket = SplineSocket {
        point: wps[merge_idx].point,
        tangent: m_sample.tangent,
        normal: m_sample.normal,
        width: wps[merge_idx].width,
        elevation: 0.0,
        bank_angle: 0.0,
        surface: SurfaceType::Dirt,
    };

    let nose1_center =
        wps[split1_idx].point + s1_sample.tangent * 14.0 - s1_sample.normal * (spec.road_width * 0.5 + 2.5);
    let split1_junction = RoadJunction {
        id: JunctionId(0),
        name: "utah_umc_rx East One Joker 1 Split".to_string(),
        kind: JunctionKind::Split {
            ingress_socket: split1_socket,
            egress_sockets: vec![split1_socket, split1_socket],
            gore_config: Some(GoreConfig {
                apex_point: wps[split1_idx].point,
                divergence_angle: 14.0,
                gore_length: 14.0,
                nose_barrier: WallBarrier {
                    segment: LineSegment::new(
                        nose1_center - s1_sample.normal * 0.8,
                        nose1_center + s1_sample.normal * 0.8,
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
        name: "utah_umc_rx Central Plaza X-Joker Merge".to_string(),
        kind: JunctionKind::Merge {
            ingress_sockets: vec![merge_socket, merge_socket, merge_socket],
            egress_socket: merge_socket,
            merge_config: Some(MergeConfig {
                convergence_point: wps[merge_idx].point,
                merge_angle: 14.0,
                merge_length: 15.0,
            }),
        },
    };

    let nose2_center =
        wps[split2_idx].point + s2_sample.tangent * 14.0 + s2_sample.normal * (spec.road_width * 0.5 + 2.5);
    let split2_junction = RoadJunction {
        id: JunctionId(2),
        name: "utah_umc_rx Mega Jump Y-Fork Joker 2 Split".to_string(),
        kind: JunctionKind::Split {
            ingress_socket: split2_socket,
            egress_sockets: vec![split2_socket, split2_socket],
            gore_config: Some(GoreConfig {
                apex_point: wps[split2_idx].point,
                divergence_angle: 14.0,
                gore_length: 14.0,
                nose_barrier: WallBarrier {
                    segment: LineSegment::new(
                        nose2_center - s2_sample.normal * 0.8,
                        nose2_center + s2_sample.normal * 0.8,
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

    let p_cross = wps[cross_idx].point;
    let p_split1 = wps[split1_idx].point;
    let p_split2 = wps[split2_idx].point;
    let p_merge = wps[merge_idx].point;
    let joker1_pts = [
        p_split1,
        p_split1 + split1_socket.tangent * 15.0,
        Vec2::new(-214.0, 14.0),
        Vec2::new(-182.0, 36.0),
        Vec2::new(-138.0, 44.0),
        Vec2::new(-94.0, 40.0),
        Vec2::new(-64.0, 25.0),
        Vec2::new(-52.0, 13.0),
        p_cross,
        Vec2::new(-38.0, -1.0),
        Vec2::new(-26.0, -13.0),
        Vec2::new(-2.0, -28.0),
        Vec2::new(26.0, -22.0),
        Vec2::new(48.0, 0.0),
        p_merge - merge_socket.tangent * 14.0,
        p_merge,
    ];
    let n1 = joker1_pts.len();
    let joker1_wps: Vec<TrackWaypoint> = joker1_pts
        .into_iter()
        .enumerate()
        .map(|(i, pt)| {
            let width = if i == 0 || i + 1 == n1 { spec.road_width } else { 13.0 };
            TrackWaypoint::new(pt, width)
                .with_surface(SurfaceType::Dirt)
                .with_elevation(0.0)
                .with_bank_angle(0.0)
        })
        .collect();

    let joker2_pts = [
        p_split2,
        p_split2 + split2_socket.tangent * 14.0,
        Vec2::new(-108.0, -8.0),
        Vec2::new(-84.0, -22.0),
        Vec2::new(-64.0, -14.0),
        Vec2::new(-52.0, -1.0),
        p_cross,
        Vec2::new(-38.0, 13.0),
        Vec2::new(-26.0, 25.0),
        Vec2::new(0.0, 34.0),
        Vec2::new(26.0, 26.0),
        Vec2::new(48.0, 10.0),
        p_merge - merge_socket.tangent * 14.0,
        p_merge,
    ];
    let n2 = joker2_pts.len();
    let joker2_wps: Vec<TrackWaypoint> = joker2_pts
        .into_iter()
        .enumerate()
        .map(|(i, pt)| {
            let width = if i == 0 || i + 1 == n2 { spec.road_width } else { 13.0 };
            TrackWaypoint::new(pt, width)
                .with_surface(SurfaceType::Dirt)
                .with_elevation(0.0)
                .with_bank_angle(0.0)
        })
        .collect();

    let mut seg0 = RoadSegment::new(SegmentId(0), "Start / Finish & East One Entry", seg0_wps);
    seg0.entry_junction = Some(SocketId::new(JunctionId(1), 0));
    seg0.exit_junction = Some(SocketId::new(JunctionId(0), 0));
    seg0.recompute_samples(None, Some(&split1_socket));

    let mut seg4 = RoadSegment::new(SegmentId(4), "East One Outer Dirt Chicane", seg4_wps);
    seg4.entry_junction = Some(SocketId::new(JunctionId(0), 0));
    seg4.exit_junction = Some(SocketId::new(JunctionId(2), 0));
    seg4.recompute_samples(Some(&split1_socket), Some(&split2_socket));

    let mut seg1 = RoadSegment::new(SegmentId(1), "130-ft Mega Jump Straightaway", seg1_wps);
    seg1.entry_junction = Some(SocketId::new(JunctionId(2), 0));
    seg1.exit_junction = Some(SocketId::new(JunctionId(1), 0));
    seg1.recompute_samples(Some(&split2_socket), Some(&merge_socket));

    let mut seg2 = RoadSegment::new(SegmentId(2), spec.joker_name, joker1_wps);
    seg2.entry_junction = Some(SocketId::new(JunctionId(0), 1));
    seg2.exit_junction = Some(SocketId::new(JunctionId(1), 1));
    seg2.recompute_samples(Some(&split1_socket), Some(&merge_socket));

    let mut seg5 = RoadSegment::new(
        SegmentId(5),
        "Utah UMC Underpass X-Joker 2 (South-Corner to North-Loop)",
        joker2_wps,
    );
    seg5.entry_junction = Some(SocketId::new(JunctionId(2), 1));
    seg5.exit_junction = Some(SocketId::new(JunctionId(1), 2));
    seg5.recompute_samples(Some(&split2_socket), Some(&merge_socket));

    let mut seg3 = RoadSegment::new(
        SegmentId(3),
        "Talladega S-Berms, Wind Up & North Dirt Chicane",
        seg3_wps,
    );
    seg3.entry_junction = Some(SocketId::new(JunctionId(1), 0));
    seg3.exit_junction = Some(SocketId::new(JunctionId(0), 0));
    seg3.recompute_samples(Some(&merge_socket), None);

    let main_len = seg0.length + seg4.length + seg1.length + seg3.length;
    let joker1_len = seg0.length + seg2.length + seg3.length;
    let joker2_len = seg0.length + seg4.length + seg5.length + seg3.length;

    let mut net = TrackNetwork {
        junctions: vec![split1_junction, merge_junction, split2_junction],
        segments: vec![seg0, seg4, seg1, seg2, seg5, seg3],
        layouts: vec![
            TrackLayout {
                id: "main".to_string(),
                display_name: "Standard Circuit (Over 130-ft Mega Jump)".to_string(),
                entry_segment: None,
                segment_sequence: vec![SegmentId(0), SegmentId(4), SegmentId(1), SegmentId(3)],
                is_closed: true,
                total_lap_length: main_len,
                start_finish_segment: SegmentId(0),
                checkpoint_ids: Vec::new(),
            },
            TrackLayout {
                id: "joker".to_string(),
                display_name: "Joker Lap 1 (North-West to South-Bowl X-Underpass)".to_string(),
                entry_segment: None,
                segment_sequence: vec![SegmentId(0), SegmentId(2), SegmentId(3)],
                is_closed: true,
                total_lap_length: joker1_len,
                start_finish_segment: SegmentId(0),
                checkpoint_ids: Vec::new(),
            },
            TrackLayout {
                id: "joker_2".to_string(),
                display_name: "Joker Lap 2 (South-Corner to North-Loop X-Underpass)".to_string(),
                entry_segment: None,
                segment_sequence: vec![SegmentId(0), SegmentId(4), SegmentId(5), SegmentId(3)],
                is_closed: true,
                total_lap_length: joker2_len,
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

fn is_clear_of_roads(track: &Track, pt: Vec2, min_clearance: f32) -> bool {
    let proj = track.spline.project_point(pt);
    if proj.distance_to_spline < proj.track_width * 0.5 + min_clearance {
        return false;
    }
    if let Some(ref net) = track.network {
        for seg in &net.segments {
            let p = seg.project_point(pt);
            if p.distance_to_spline < p.track_width * 0.5 + min_clearance {
                return false;
            }
        }
        if let Some(ref chute) = net.launch_chute {
            for slot in &chute.grid_slots {
                if slot.position.distance(pt) < min_clearance + 10.0 {
                    return false;
                }
            }
        }
    }
    true
}

fn populate_venue_scenery(spec: &NitroVenueSpec, track: &mut Track) {
    let mut grandstands = Vec::new();
    let mut buildings = Vec::new();
    let mut trees = Vec::new();
    let mut rocks = Vec::new();

    match spec.slug {
        "utah_umc_rx" => {
            grandstands.push(
                Grandstand::new(
                    1,
                    Vec2::new(195.0, -80.0),
                    75.0,
                    14.0,
                    std::f32::consts::FRAC_PI_4 * 3.0,
                )
                .with_style(GrandstandStyle::CoveredStadium),
            );
            grandstands.push(
                Grandstand::new(2, Vec2::new(-40.0, 108.0), 90.0, 14.0, std::f32::consts::PI)
                    .with_style(GrandstandStyle::OpenBleachers),
            );
            buildings.push(Building::new(
                1,
                Vec2::new(40.0, 108.0),
                Vec2::new(36.0, 16.0),
                0.0,
                BuildingStyle::PitGarage,
            ));
            for (i, pos) in [
                Vec2::new(-310.0, 40.0),
                Vec2::new(-290.0, -50.0),
                Vec2::new(-30.0, -85.0),
                Vec2::new(165.0, -145.0),
                Vec2::new(230.0, 15.0),
            ]
            .into_iter()
            .enumerate()
            {
                rocks.push(Rock::new(i + 1, pos, RockType::Sandstone).with_scale(1.4));
            }
            for (i, pos) in [
                Vec2::new(-220.0, -55.0),
                Vec2::new(-85.0, -70.0),
                Vec2::new(50.0, -180.0),
                Vec2::new(225.0, 65.0),
            ]
            .into_iter()
            .enumerate()
            {
                trees.push(Tree::new(i + 1, pos, TreeType::Bush).with_scale(1.2));
            }
        }
        "phoenix_rx" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(80.0, 130.0), 110.0, 16.0, std::f32::consts::PI)
                    .with_style(GrandstandStyle::CoveredStadium),
            );
            for (i, pos) in [
                Vec2::new(-40.0, -130.0),
                Vec2::new(40.0, -130.0),
                Vec2::new(220.0, -130.0),
                Vec2::new(-320.0, -60.0),
            ]
            .into_iter()
            .enumerate()
            {
                let kind = if i % 2 == 0 { TreeType::Cactus } else { TreeType::Palm };
                trees.push(Tree::new(i + 1, pos, kind).with_scale(1.25));
            }
            for (i, pos) in [Vec2::new(-325.0, -10.0), Vec2::new(325.0, 10.0)]
                .into_iter()
                .enumerate()
            {
                rocks.push(Rock::new(i + 1, pos, RockType::Sandstone).with_scale(1.3));
            }
        }
        "daytona_rx" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(-40.0, 135.0), 120.0, 16.0, std::f32::consts::PI)
                    .with_style(GrandstandStyle::CoveredStadium),
            );
            buildings.push(Building::new(
                1,
                Vec2::new(40.0, 40.0),
                Vec2::new(32.0, 14.0),
                0.0,
                BuildingStyle::PitGarage,
            ));
            for (i, pos) in [
                Vec2::new(-120.0, -25.0),
                Vec2::new(-40.0, -35.0),
                Vec2::new(110.0, -85.0),
                Vec2::new(355.0, -90.0),
            ]
            .into_iter()
            .enumerate()
            {
                trees.push(Tree::new(i + 1, pos, TreeType::Palm).with_scale(1.25));
            }
        }
        "sonoma_rx" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(-110.0, -158.0), 100.0, 14.0, 0.0)
                    .with_style(GrandstandStyle::HillsideBleachers),
            );
            for (i, pos) in [
                Vec2::new(-180.0, -35.0),
                Vec2::new(-60.0, 65.0),
                Vec2::new(30.0, 65.0),
                Vec2::new(285.0, 20.0),
            ]
            .into_iter()
            .enumerate()
            {
                trees.push(Tree::new(i + 1, pos, TreeType::Oak).with_scale(1.2));
            }
        }
        "atlanta_rx" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(40.0, 152.0), 120.0, 16.0, std::f32::consts::PI)
                    .with_style(GrandstandStyle::CoveredStadium),
            );
            for (i, pos) in [
                Vec2::new(-80.0, -25.0),
                Vec2::new(-10.0, -25.0),
                Vec2::new(-150.0, -25.0),
                Vec2::new(290.0, 0.0),
            ]
            .into_iter()
            .enumerate()
            {
                trees.push(Tree::new(i + 1, pos, TreeType::Pine).with_scale(1.2));
            }
        }
        "charlotte_rx" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(-90.0, 175.0), 105.0, 16.0, std::f32::consts::PI)
                    .with_style(GrandstandStyle::CoveredStadium),
            );
            for (i, pos) in [
                Vec2::new(-95.0, 65.0),
                Vec2::new(-55.0, 65.0),
                Vec2::new(230.0, -50.0),
                Vec2::new(-230.0, 65.0),
            ]
            .into_iter()
            .enumerate()
            {
                trees.push(Tree::new(i + 1, pos, TreeType::Oak).with_scale(1.2));
            }
        }
        "iowa_rx" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(165.0, 10.0), 95.0, 16.0, std::f32::consts::FRAC_PI_2)
                    .with_style(GrandstandStyle::CoveredStadium),
            );
            for (i, pos) in [
                Vec2::new(-175.0, 20.0),
                Vec2::new(-165.0, -50.0),
                Vec2::new(15.0, 185.0),
                Vec2::new(-85.0, 170.0),
            ]
            .into_iter()
            .enumerate()
            {
                trees.push(Tree::new(i + 1, pos, TreeType::Oak).with_scale(1.2));
            }
        }
        _ => {}
    }

    track.geometry.grandstands = grandstands
        .into_iter()
        .filter(|g| is_clear_of_roads(track, g.center, 22.0))
        .collect();
    track.geometry.buildings = buildings
        .into_iter()
        .filter(|b| is_clear_of_roads(track, b.center, 22.0))
        .collect();
    track.geometry.trees = trees
        .into_iter()
        .filter(|t| is_clear_of_roads(track, t.position, 14.0))
        .collect();
    track.geometry.rocks = rocks
        .into_iter()
        .filter(|r| is_clear_of_roads(track, r.position, 14.0))
        .collect();
}

fn build_venue(spec: &NitroVenueSpec) -> (String, Track) {
    let (wps, split_idx, cross_idx, merge_idx, joker_interior) = build_main_waypoints(spec);
    let spline = TrackSpline::new(wps.clone(), true);
    let (inner_walls, outer_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    // Primary MegaRamp:
    // On utah_umc_rx, centered at wps[cross_idx] on Seg 1 so the open chasm sits directly over the X-crossing Jokers.
    // On the 6 iRacing American RX venues, placed at the midpoint of spec.mega_wp_pair on a collinear dirt straightaway.
    let (p_mega, t_mega) = if spec.slug == "utah_umc_rx" {
        let p_cross = wps[cross_idx].point;
        let t_cross = (wps[merge_idx].point - wps[split_idx].point).normalize_or_zero();
        let rem = 2.0 * spec.mega_half_len - spec.mega_gap_len;
        (p_cross + t_cross * (0.04 * rem), t_cross)
    } else {
        let (g0, g1) = spec.mega_wp_pair;
        let p = (wps[g0].point + wps[g1].point) * 0.5;
        let t = (wps[g1].point - wps[g0].point).normalize_or_zero();
        (p, t)
    };
    let angle_mega = t_mega.y.atan2(t_mega.x);
    let mut mega_ramp = JumpRamp::new(
        1,
        SurfaceShape::OrientedBox {
            center: p_mega,
            half_extents: Vec2::new(spec.mega_half_len, 4.6),
            angle: angle_mega,
        },
        t_mega,
        spec.mega_launch_speed,
        spec.mega_angle_deg,
        spec.mega_height,
        spec.mega_jump_name,
    )
    .with_kind(RampKind::MegaRamp)
    .with_gap_length(spec.mega_gap_len);
    mega_ramp.surface = SurfaceType::Dirt;

    // Secondary tabletop jump on a collinear dirt straightaway
    let (m0, m1) = spec.minor_wp_pair;
    let p_minor = (wps[m0].point + wps[m1].point) * 0.5;
    let t_minor = (wps[m1].point - wps[m0].point).normalize_or_zero();
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

    let mut checkpoints = generate_checkpoints(&base_track.spline, 24, 3);
    let net = if spec.slug == "utah_umc_rx" {
        let mut net = build_utah_x_joker_network(
            spec,
            &base_track,
            UTAH_SPLIT1_IDX,
            split_idx,
            cross_idx,
            merge_idx,
        );
        let s1_dist = base_track.spline.project_point(wps[UTAH_SPLIT1_IDX].point).progress_distance;
        let s2_dist = base_track.spline.project_point(wps[split_idx].point).progress_distance;
        let m_dist = base_track.spline.project_point(wps[merge_idx].point).progress_distance;
        let mut main_cp_ids = Vec::new();
        let mut joker1_cp_ids = Vec::new();
        let mut joker2_cp_ids = Vec::new();
        let joker1_cp_id = checkpoints.len();
        let joker2_cp_id = checkpoints.len() + 1;
        let mut inserted_joker1_cp = false;
        let mut inserted_joker2_cp = false;

        for cp in &mut checkpoints {
            let mid = (cp.gate.start + cp.gate.end) * 0.5;
            let d = base_track.spline.project_point(mid).progress_distance;
            main_cp_ids.push(cp.id);
            if d <= s1_dist {
                cp.segment_id = Some(SegmentId(0));
                joker1_cp_ids.push(cp.id);
                joker2_cp_ids.push(cp.id);
            } else if d <= s2_dist {
                cp.segment_id = Some(SegmentId(4));
                joker2_cp_ids.push(cp.id);
                if !inserted_joker1_cp {
                    joker1_cp_ids.push(joker1_cp_id);
                    inserted_joker1_cp = true;
                }
            } else if d < m_dist {
                cp.segment_id = Some(SegmentId(1));
                if !inserted_joker1_cp {
                    joker1_cp_ids.push(joker1_cp_id);
                    inserted_joker1_cp = true;
                }
                if !inserted_joker2_cp {
                    joker2_cp_ids.push(joker2_cp_id);
                    inserted_joker2_cp = true;
                }
            } else {
                cp.segment_id = Some(SegmentId(3));
                joker1_cp_ids.push(cp.id);
                joker2_cp_ids.push(cp.id);
            }
        }

        let seg2 = net.get_segment(SegmentId(2)).unwrap();
        let j1_sample = seg2.sample_at_distance(seg2.length * 0.28);
        let j1_half_w = j1_sample.width * 0.5 + 1.5;
        let mut j1_cp = Checkpoint::new(
            joker1_cp_id,
            LineSegment::new(
                j1_sample.point + j1_sample.normal * j1_half_w,
                j1_sample.point - j1_sample.normal * j1_half_w,
            ),
            j1_sample.tangent,
            1,
            false,
        );
        j1_cp.is_joker = true;
        j1_cp.segment_id = Some(SegmentId(2));
        checkpoints.push(j1_cp);

        let seg5 = net.get_segment(SegmentId(5)).unwrap();
        let j2_sample = seg5.sample_at_distance(seg5.length * 0.25);
        let j2_half_w = j2_sample.width * 0.5 + 1.5;
        let mut j2_cp = Checkpoint::new(
            joker2_cp_id,
            LineSegment::new(
                j2_sample.point + j2_sample.normal * j2_half_w,
                j2_sample.point - j2_sample.normal * j2_half_w,
            ),
            j2_sample.tangent,
            1,
            false,
        );
        j2_cp.is_joker = true;
        j2_cp.segment_id = Some(SegmentId(5));
        checkpoints.push(j2_cp);

        net.layouts[0].checkpoint_ids = main_cp_ids;
        net.layouts[1].checkpoint_ids = joker1_cp_ids;
        net.layouts[2].checkpoint_ids = joker2_cp_ids;
        net
    } else {
        let mut net = assemble_network(spec, &base_track, split_idx, merge_idx, &joker_interior);

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
            if d > s_dist + 4.0 && d < m_dist - 4.0 {
                cp.segment_id = Some(SegmentId(1));
                if !inserted_joker_cp {
                    joker_cp_ids.push(joker_cp_id);
                    inserted_joker_cp = true;
                }
            } else if d <= s_dist + 4.0 {
                cp.segment_id = Some(SegmentId(0));
                joker_cp_ids.push(cp.id);
            } else {
                if !inserted_joker_cp {
                    joker_cp_ids.push(joker_cp_id);
                    inserted_joker_cp = true;
                }
                cp.segment_id = Some(SegmentId(3));
                joker_cp_ids.push(cp.id);
            }
        }

        let seg2 = net.get_segment(SegmentId(2)).unwrap();
        let j_sample = seg2.sample_at_distance(seg2.length * 0.50);
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
        net
    };

    base_track.checkpoints = checkpoints;
    base_track.network = Some(net);
    base_track.rebuild_geometry(3.5, BarrierType::TireWall);

    // Stamp the 6-car Rallycross launch chute
    let mut chute_template = LaunchChuteSpec::new(0, spec.chute_side);
    chute_template.pattern = PackedGridPattern::RallycrossTwoTwoTwo;
    chute_template.surface = SurfaceType::Asphalt;
    let chute_spec = base_track
        .place_launch_chute(&chute_template)
        .unwrap_or_else(|e| panic!("{}: failed to place launch chute: {}", spec.slug, e));

    // Populate authentic venue scenery after network and launch chute are placed
    populate_venue_scenery(spec, &mut base_track);

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
    if spec.slug == "utah_umc_rx" {
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
        "Built {:24} -> len={:6.1}m, mega_gap={:4.1}m ({:5.1}ft), mega_len={:4.1}m ({:5.1}ft), joker_cost={:+.2}s, min_r={:4.1}m, underpass_pts={:2}, chute_wp={} ({:?}), breakdown={}",
        spec.slug,
        base_track.spline.total_length(),
        mega_ramp.effective_gap_length(),
        mega_ramp.effective_gap_length() * 3.28084,
        mega_ramp.length(),
        mega_ramp.length() * 3.28084,
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
