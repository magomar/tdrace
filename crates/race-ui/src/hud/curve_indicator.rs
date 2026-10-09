use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_line, draw_triangle};
use serde::{Deserialize, Serialize};
use wheelbase::car::Car;
use arcade_race_core::track::checkpoint::TrackProgressTracker;
use arcade_race_core::track::curve::{angle_between_tangents, CurveApproachStatus, CurveDirection, TrackCurve};
use arcade_race_core::track::network::{JunctionId, JunctionKind, SegmentId, SocketId, TrackLayout, TrackNetwork};
use arcade_race_core::track::spline::TrackSpline;
use arcade_race_core::track::Track;

/// Available color schemes for the curve approaching & braking indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveColorScheme {
    /// Classic Traffic Signal Gradient: Green -> Yellow -> Orange -> Red (Default).
    Traffic,
    /// Arcade Synthwave: Neon Cyan -> Neon Magenta -> Laser Red.
    Synthwave,
    /// High-Contrast Minimalist: Ice White -> Amber Warning -> Pure Red.
    Contrast,
    /// Motorsport Rally Pacenotes: Color fixed by curve degree, flashing Red on critical brake.
    Rally,
}

impl Default for CurveColorScheme {
    fn default() -> Self {
        Self::Traffic
    }
}

impl CurveColorScheme {
    /// Cycles to the next color scheme in the sequence.
    pub const fn next(&self) -> Self {
        match self {
            Self::Traffic => Self::Synthwave,
            Self::Synthwave => Self::Contrast,
            Self::Contrast => Self::Rally,
            Self::Rally => Self::Traffic,
        }
    }

    /// User-facing display title for HUD toast notification.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Traffic => "TRAFFIC (GREEN-YELLOW-RED)",
            Self::Synthwave => "SYNTHWAVE (CYAN-MAGENTA-RED)",
            Self::Contrast => "HIGH CONTRAST (WHITE-AMBER-RED)",
            Self::Rally => "RALLY PACENOTES (RATING-CODED)",
        }
    }
}

/// Linearly interpolates between two macroquad colors.
pub fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        c1.r + (c2.r - c1.r) * t,
        c1.g + (c2.g - c1.g) * t,
        c1.b + (c2.b - c1.b) * t,
        c1.a + (c2.a - c1.a) * t,
    )
}

/// Computes the active foreground and border/glow colors based on color scheme, dynamic urgency, and degree.
pub fn compute_curve_colors(
    scheme: CurveColorScheme,
    urgency: f32,
    degree: u8,
    alpha: f32,
) -> (Color, Color) {
    let u = urgency.clamp(0.0, 1.0);

    let (base_col, border_col) = match scheme {
        CurveColorScheme::Traffic => {
            let green = Color::new(0.18, 0.92, 0.38, 1.0); // Vibrant Neon Green
            let yellow = Color::new(1.0, 0.84, 0.12, 1.0); // Bright Neon Yellow
            let orange = Color::new(1.0, 0.48, 0.10, 1.0); // Warning Neon Orange
            let red = Color::new(0.96, 0.16, 0.20, 1.0); // Critical Red

            let c = if u < 0.35 {
                lerp_color(green, yellow, u / 0.35)
            } else if u < 0.70 {
                lerp_color(yellow, orange, (u - 0.35) / 0.35)
            } else {
                lerp_color(orange, red, (u - 0.70) / 0.30)
            };
            (c, c)
        }
        CurveColorScheme::Synthwave => {
            let cyan = Color::new(0.20, 0.90, 1.0, 1.0); // Cyber Neon Cyan
            let magenta = Color::new(1.0, 0.25, 0.80, 1.0); // Neon Magenta
            let red = Color::new(1.0, 0.10, 0.25, 1.0); // Laser Red

            let c = if u < 0.55 {
                lerp_color(cyan, magenta, u / 0.55)
            } else {
                lerp_color(magenta, red, (u - 0.55) / 0.45)
            };
            (c, c)
        }
        CurveColorScheme::Contrast => {
            let white = Color::new(0.88, 0.95, 1.0, 1.0); // Ice White
            let amber = Color::new(1.0, 0.65, 0.05, 1.0); // Amber
            let red = Color::new(0.96, 0.16, 0.18, 1.0); // Pure Red

            let c = if u < 0.50 {
                lerp_color(white, amber, u / 0.50)
            } else {
                lerp_color(amber, red, (u - 0.50) / 0.50)
            };
            (c, c)
        }
        CurveColorScheme::Rally => {
            if u >= 0.80 {
                let red = Color::new(0.96, 0.16, 0.20, 1.0);
                (red, red)
            } else {
                let deg_col = match degree {
                    1 => Color::new(0.20, 0.85, 0.55, 1.0), // Emerald
                    2 => Color::new(0.20, 0.90, 1.0, 1.0),  // Cyan
                    3 => Color::new(1.0, 0.82, 0.15, 1.0),  // Yellow
                    4 => Color::new(1.0, 0.52, 0.12, 1.0),  // Orange
                    _ => Color::new(1.0, 0.22, 0.70, 1.0),  // Rose/Magenta
                };
                (deg_col, deg_col)
            }
        }
    };

    (
        Color::new(base_col.r, base_col.g, base_col.b, base_col.a * alpha),
        Color::new(border_col.r, border_col.g, border_col.b, border_col.a * alpha),
    )
}

/// Time to curve entry (seconds) at which the indicator starts to fade in.
pub const CURVE_INDICATOR_FADE_START_S: f32 = 4.5;
/// Time to curve entry (seconds) at which the indicator is fully visible.
pub const CURVE_INDICATOR_FADE_END_S: f32 = 4.0;
/// Speed floor (m/s) for the time-to-entry estimate, so a slow or stopped car
/// still sees the turn at `CURVE_INDICATOR_FADE_START_S * 15 m/s` = 67.5 m.
pub const CURVE_INDICATOR_MIN_ETA_SPEED: f32 = 15.0;

/// Estimated time (seconds) until the car reaches `distance_to_entry` at `speed_mps`.
#[inline]
pub fn compute_curve_eta(distance_to_entry: f32, speed_mps: f32) -> f32 {
    distance_to_entry / speed_mps.max(CURVE_INDICATOR_MIN_ETA_SPEED)
}

/// Curve search distance (meters) that covers the whole fade-in window at `speed_mps`.
#[inline]
pub fn curve_indicator_lookahead(speed_mps: f32) -> f32 {
    CURVE_INDICATOR_FADE_START_S * speed_mps.max(CURVE_INDICATOR_MIN_ETA_SPEED)
}

/// Calculates the display opacity for the curve indicator based on time to curve entry and apex traversal.
pub fn compute_indicator_alpha(
    distance_to_entry: f32,
    distance_to_apex: f32,
    is_inside_curve: bool,
    speed_mps: f32,
) -> f32 {
    // Smooth fade in on approach (4.5 s -> 4.0 s before entry), so faster cars get earlier warning
    let approach_alpha = if is_inside_curve {
        1.0
    } else {
        let eta = compute_curve_eta(distance_to_entry, speed_mps);
        ((CURVE_INDICATOR_FADE_START_S - eta)
            / (CURVE_INDICATOR_FADE_START_S - CURVE_INDICATOR_FADE_END_S))
            .clamp(0.0, 1.0)
    };

    // Quick fade away after the apex / inflexion point of the curve has been traversed (< 0.0)
    let apex_fade_alpha = if distance_to_apex < 0.0 {
        let past_apex = -distance_to_apex;
        // Fades smoothly to zero across 10 meters past apex
        (1.0 - past_apex / 10.0).clamp(0.0, 1.0)
    } else {
        1.0
    };

    approach_alpha * apex_fade_alpha
}

/// Unit vector from the player car toward the side of the curve, in the car's own frame
/// (the car's left or right, not the screen's).
#[inline]
fn curve_side_vector(player_car: &Car, direction: CurveDirection) -> Vec2 {
    match direction {
        CurveDirection::Left => -player_car.right_vector(),
        CurveDirection::Right => player_car.right_vector(),
    }
}

/// Distance (world units) from the car center to the inner edge of the curve indicator:
/// half the car's track width, plus 0.5 m for the body, plus a 10 px gap on screen.
#[inline]
pub fn curve_indicator_inner_clearance(player_car: &Car, current_zoom: f32) -> f32 {
    player_car.config.track_width * 0.5 + 0.5 + 10.0 / current_zoom.max(0.5)
}

/// Track length (meters) drawn before the curve entry and after its exit in the pacenote icon.
const PACENOTE_LEAD_M: f32 = 12.0;
/// Number of centerline points in the pacenote icon.
const PACENOTE_SAMPLES: usize = 24;

/// Builds the pacenote icon line: the curve's own centerline, from `PACENOTE_LEAD_M` before
/// entry to `PACENOTE_LEAD_M` after exit, scaled so its longer side is `size` and centered on `center`.
///
/// The icon keeps the track's world orientation, so it matches the road on screen.
pub fn compute_pacenote_polyline(
    spline: &TrackSpline,
    curve: &TrackCurve,
    center: Vec2,
    size: f32,
) -> Vec<Vec2> {
    let mut span = curve.exit_distance - curve.entry_distance;
    if span < 0.0 {
        span += spline.total_length();
    }
    let start = curve.entry_distance - PACENOTE_LEAD_M;
    let length = span + 2.0 * PACENOTE_LEAD_M;

    let raw: Vec<Vec2> = (0..PACENOTE_SAMPLES)
        .map(|i| {
            let t = i as f32 / (PACENOTE_SAMPLES - 1) as f32;
            spline.sample_at_distance(start + length * t).point
        })
        .collect();

    let (min, max) = raw
        .iter()
        .fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    let extent = (max - min).max_element().max(1e-3);
    let mid = (min + max) * 0.5;
    raw.iter().map(|p| center + (*p - mid) * (size / extent)).collect()
}

/// Renders the rally pacenote curve icon beside the player car.
///
/// A dark disc holds a drawing of the upcoming curve's shape with an arrowhead at the exit,
/// colored by the curve color scheme and urgency.
pub fn render_curve_pacenote(
    player_car: &Car,
    status: &CurveApproachStatus,
    spline: &TrackSpline,
    scheme: CurveColorScheme,
    current_zoom: f32,
    anim_time: f32,
    scale: f32,
    brightness: f32,
) {
    let base_alpha = compute_indicator_alpha(
        status.distance_to_entry,
        status.distance_to_apex,
        status.is_inside_curve,
        player_car.state.speed,
    );
    let alpha = (base_alpha * brightness).clamp(0.0, 1.0);

    if alpha <= 0.02 {
        return;
    }

    let degree = status.curve.degree.clamp(1, 5);
    let (color, _border_color) = compute_curve_colors(scheme, status.urgency, degree, alpha);

    // Pulse scale when in critical braking envelope
    let pulse_scale = if status.urgency >= 0.85 {
        1.0 + (anim_time * 9.0).sin().abs() * 0.10
    } else {
        1.0
    };

    let zoom = current_zoom.max(0.5);
    let size = (34.0 * pulse_scale * scale) / zoom;
    let plate_radius = size * 0.92;
    let thickness = (4.0 * pulse_scale * scale) / zoom;
    let outline = thickness + 3.0 / zoom;

    // Disc sits on the car's own left or right side
    let side = curve_side_vector(player_car, status.curve.direction);
    let origin = player_car.state.position;
    let center = origin + side * (curve_indicator_inner_clearance(player_car, zoom) + plate_radius);

    let points = compute_pacenote_polyline(spline, &status.curve, center, size);
    let n = points.len();
    if n < 2 {
        return;
    }

    let plate_col = Color::new(0.02, 0.03, 0.06, (0.55 * alpha).min(1.0));
    let shadow_col = Color::new(0.0, 0.0, 0.0, (0.80 * alpha).min(1.0));

    draw_circle(center.x, center.y, plate_radius, plate_col);

    // Arrowhead at the exit, pointing along the last segment
    let dir = (points[n - 1] - points[n - 2]).normalize_or_zero();
    let perp = Vec2::new(-dir.y, dir.x);
    let head_len = size * 0.28;
    let head_w = size * 0.18;
    let tip = points[n - 1] + dir * head_len * 0.6;
    let base = points[n - 1] - dir * head_len * 0.4;
    let pad = 1.5 / zoom;

    // 1. Dark outline under the line and arrowhead
    for pair in points.windows(2) {
        draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, outline, shadow_col);
    }
    for p in &points {
        draw_circle(p.x, p.y, outline * 0.5, shadow_col);
    }
    let (t_o, l_o, r_o) = (
        tip + dir * pad * 2.0,
        base - dir * pad + perp * (head_w + pad * 1.5),
        base - dir * pad - perp * (head_w + pad * 1.5),
    );
    draw_triangle(mq_vec(t_o), mq_vec(l_o), mq_vec(r_o), shadow_col);

    // 2. Colored centerline with round joins, then the arrowhead
    for pair in points.windows(2) {
        draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, thickness, color);
    }
    for p in &points {
        draw_circle(p.x, p.y, thickness * 0.5, color);
    }
    draw_triangle(mq_vec(tip), mq_vec(base + perp * head_w), mq_vec(base - perp * head_w), color);
}

/// Converts to macroquad's own `Vec2` (a different glam version) for `draw_triangle`.
#[inline]
fn mq_vec(v: Vec2) -> macroquad::math::Vec2 {
    macroquad::math::Vec2::new(v.x, v.y)
}

// ---------------------------------------------------------------------------------------------------------------
// Bifurcation pacenote (spec 085): a fork badge before a joker split or a pit lane entry
// ---------------------------------------------------------------------------------------------------------------

/// Status of an upcoming track split or bifurcation.
#[derive(Debug, Clone, PartialEq)]
pub struct BifurcationApproachStatus {
    /// Distance along the route from the car to the split (m). Negative once the car is past it.
    pub distance_to_split: f32,
    /// Angle between the two branches (degrees).
    pub divergence_angle: f32,
    pub branch_left_is_tactical: bool,
    pub branch_right_is_tactical: bool,
    /// True when the branch the car should take is the left one.
    pub recommended_branch_left: bool,
    pub is_pit_entry: bool,
    pub is_joker_split: bool,
}

impl BifurcationApproachStatus {
    /// True when the car should take the tactical branch (the joker road, or the pit lane).
    pub fn recommends_tactical(&self) -> bool {
        self.recommended_branch_left == self.branch_left_is_tactical
    }
}

/// Distance along a branch road (m) where its direction is measured.
const BIFURCATION_BRANCH_PROBE_M: f32 = 20.0;
/// How far (m) a segment end may lie from a split's ingress socket for the split to count as ahead.
const SPLIT_ANCHOR_TOLERANCE_M: f32 = 3.0;
/// Segments the route walk crosses at most before it gives up.
const BIFURCATION_MAX_SEGMENT_HOPS: usize = 6;
/// The fork graphic spreads its branches by the real divergence, kept inside this range (degrees).
const FORK_MIN_SPREAD_DEG: f32 = 14.0;
const FORK_MAX_SPREAD_DEG: f32 = 35.0;

/// Display opacity of the bifurcation badge: fades in 4.5 s to 4.0 s before the split and out over 10 m past it.
pub fn compute_bifurcation_alpha(status: &BifurcationApproachStatus, speed_mps: f32) -> f32 {
    compute_indicator_alpha(status.distance_to_split, status.distance_to_split, false, speed_mps)
}

/// Finds the next joker split or pit lane entry ahead of the car within the badge's fade-in window.
///
/// `joker_wanted` is true while the driver still owes a joker lap. `pit_wanted` is true while a pit stop is
/// advised. The result only drives the HUD: nothing in the simulation reads it.
pub fn upcoming_bifurcation(
    track: &Track,
    tracker: &TrackProgressTracker,
    speed_mps: f32,
    joker_wanted: bool,
    pit_wanted: bool,
) -> Option<BifurcationApproachStatus> {
    let lookahead = curve_indicator_lookahead(speed_mps);
    let joker = upcoming_joker_split(track, tracker, lookahead, joker_wanted);
    let pit = upcoming_pit_entry(track, tracker, lookahead, pit_wanted);
    match (joker, pit) {
        (Some(j), Some(p)) => Some(if p.distance_to_split.abs() < j.distance_to_split.abs() { p } else { j }),
        (j, p) => j.or(p),
    }
}

/// The layout that contains `segment`: `preferred` when it does, else the first layout that does.
fn layout_with_segment<'a>(network: &'a TrackNetwork, preferred: &str, segment: SegmentId) -> Option<&'a TrackLayout> {
    let has = |l: &TrackLayout| l.entry_segment == Some(segment) || l.segment_sequence.contains(&segment);
    network.get_layout(preferred).filter(|l| has(l)).or_else(|| network.layouts.iter().find(|l| has(l)))
}

/// Segment that follows `segment` on `layout`, wrapping round a closed loop.
fn next_segment_on_layout(network: &TrackNetwork, layout: &TrackLayout, segment: SegmentId) -> Option<SegmentId> {
    if layout.entry_segment == Some(segment) {
        return network.entry_continuation_segment(layout);
    }
    let i = layout.segment_sequence.iter().position(|&s| s == segment)?;
    layout.segment_sequence.get(i + 1).copied().or_else(|| layout.is_closed.then(|| layout.segment_sequence[0]))
}

fn upcoming_joker_split(
    track: &Track,
    tracker: &TrackProgressTracker,
    lookahead: f32,
    joker_wanted: bool,
) -> Option<BifurcationApproachStatus> {
    let network = track.network.as_ref()?;
    let multi = tracker.multi_route.as_ref()?;
    let layout = layout_with_segment(network, &multi.active_layout_id, multi.current_segment_id)?;

    // Just past a split the badge fades out over the first metres of the branch.
    let mut segment = network.get_segment(multi.current_segment_id)?;
    let mut progress = multi.segment_progress_distance;
    if let Some(entry) = segment.entry_junction {
        if progress < 10.0 {
            let start = segment.samples.first()?.point;
            if let Some(status) = joker_split_status(network, entry.junction_id, start, -progress, joker_wanted) {
                return Some(status);
            }
        }
    }

    let mut travelled = 0.0;
    for _ in 0..BIFURCATION_MAX_SEGMENT_HOPS {
        let distance = travelled + (segment.length - progress).max(0.0);
        if distance > lookahead {
            return None;
        }
        if let Some(exit) = segment.exit_junction {
            let end = segment.samples.last()?.point;
            if let Some(status) = joker_split_status(network, exit.junction_id, end, distance, joker_wanted) {
                return Some(status);
            }
        }
        travelled = distance;
        progress = 0.0;
        segment = network.get_segment(next_segment_on_layout(network, layout, segment.id)?)?;
    }
    None
}

/// Status for the split `junction_id` when it is a two-way split with exactly one joker branch.
/// `at` is the segment end (or start) the junction is linked from: some circuits link the loop closure to the
/// split junction too, so the split only counts where its ingress socket really is.
fn joker_split_status(
    network: &TrackNetwork,
    junction_id: JunctionId,
    at: Vec2,
    distance_to_split: f32,
    joker_wanted: bool,
) -> Option<BifurcationApproachStatus> {
    let JunctionKind::Split { ingress_socket, egress_sockets, .. } = &network.get_junction(junction_id)?.kind else {
        return None;
    };
    if egress_sockets.len() != 2 || ingress_socket.point.distance(at) > SPLIT_ANCHOR_TOLERANCE_M {
        return None;
    }
    let joker = network.get_layout("joker")?;
    let main = network.get_layout(&network.default_layout_id).filter(|l| l.id != joker.id)?;

    let mut branch_angle = [0.0f32; 2];
    let mut is_joker_branch = [false; 2];
    for i in 0..2 {
        let seg = network.segments.iter().find(|s| s.entry_junction == Some(SocketId::new(junction_id, i)))?;
        if seg.samples.len() < 2 {
            return None;
        }
        is_joker_branch[i] = joker.segment_sequence.contains(&seg.id) && !main.segment_sequence.contains(&seg.id);
        let probe = seg.sample_at_distance(BIFURCATION_BRANCH_PROBE_M.min(seg.length)).point;
        let dir = (probe - ingress_socket.point).normalize_or_zero();
        branch_angle[i] = angle_between_tangents(ingress_socket.tangent, dir).to_degrees();
    }
    // Exactly one branch belongs to the joker route.
    let tactical = match is_joker_branch {
        [true, false] => 0,
        [false, true] => 1,
        _ => return None,
    };
    let left = usize::from(branch_angle[1] > branch_angle[0]);
    let recommended = if joker_wanted { tactical } else { 1 - tactical };
    Some(BifurcationApproachStatus {
        distance_to_split,
        divergence_angle: (branch_angle[0] - branch_angle[1]).abs(),
        branch_left_is_tactical: tactical == left,
        branch_right_is_tactical: tactical != left,
        recommended_branch_left: recommended == left,
        is_pit_entry: false,
        is_joker_split: true,
    })
}

fn upcoming_pit_entry(
    track: &Track,
    tracker: &TrackProgressTracker,
    lookahead: f32,
    pit_wanted: bool,
) -> Option<BifurcationApproachStatus> {
    let lane = track.pit_lane.as_ref()?;
    let main = &track.spline;
    if tracker.in_pit_lane || lane.spline.samples.len() < 2 || main.samples.len() < 2 {
        return None;
    }
    let fallback;
    let junctions = match &track.pit_lane_junctions {
        Some(j) => j,
        None => {
            fallback = track.compute_pit_lane_junctions();
            fallback.as_ref()?
        }
    };

    // Distance along the main spline to the apex where the pit road leaves the track.
    let total = main.total_length();
    let mut distance = main.project_point(junctions.p_apex).progress_distance - tracker.progress_distance;
    if main.closed && total > 1.0 {
        distance = (distance + total * 0.5).rem_euclid(total) - total * 0.5;
    }
    if !(-10.0..=lookahead).contains(&distance) {
        return None;
    }

    // Side and spread of the pit road: from where it leaves the track edge to the free end of its first road.
    let free_end = (lane.entry_gate.start + lane.entry_gate.end) * 0.5;
    let chord = (free_end - junctions.te_start).normalize_or_zero();
    let track_dir = main.project_point(junctions.te_start).tangent;
    let angle = angle_between_tangents(track_dir, chord).to_degrees();
    let pit_left = angle > 0.0;
    Some(BifurcationApproachStatus {
        distance_to_split: distance,
        divergence_angle: angle.abs(),
        branch_left_is_tactical: pit_left,
        branch_right_is_tactical: !pit_left,
        recommended_branch_left: if pit_wanted { pit_left } else { !pit_left },
        is_pit_entry: true,
        is_joker_split: false,
    })
}

/// Tactical-branch accent colors: neon gold for the joker road, neon cyan for the pit lane.
const JOKER_ACCENT: (f32, f32, f32) = (1.0, 0.78, 0.10);
const PIT_ACCENT: (f32, f32, f32) = (0.0, 0.90, 1.0);

/// Centre of the badge disc: on the side of the tactical branch, with the same clearance as the curve pacenote.
pub fn bifurcation_disc_center(player_car: &Car, status: &BifurcationApproachStatus, current_zoom: f32, scale: f32) -> Vec2 {
    let zoom = current_zoom.max(0.5);
    let plate_radius = 34.0 * scale / zoom * 0.92;
    let side = if status.branch_left_is_tactical { -player_car.right_vector() } else { player_car.right_vector() };
    player_car.state.position + side * (curve_indicator_inner_clearance(player_car, zoom) + plate_radius)
}

/// Points of the fork icon, in the car's own frame so the icon matches the road ahead on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForkPoints {
    /// Bottom of the incoming road.
    pub stem_start: Vec2,
    /// Point where the road splits.
    pub junction: Vec2,
    pub left_tip: Vec2,
    pub right_tip: Vec2,
}

/// Lays the fork icon out around `center`, `size` being the disc's unit size. The branches spread by
/// `divergence_deg`, kept between `FORK_MIN_SPREAD_DEG` and `FORK_MAX_SPREAD_DEG` so the icon stays readable.
pub fn compute_fork_points(forward: Vec2, left: Vec2, center: Vec2, size: f32, divergence_deg: f32) -> ForkPoints {
    let half = (divergence_deg * 0.5).clamp(FORK_MIN_SPREAD_DEG, FORK_MAX_SPREAD_DEG).to_radians();
    let dir = |a: f32| forward * a.cos() + left * a.sin();
    let junction = center - forward * (size * 0.18);
    ForkPoints {
        stem_start: junction - forward * (size * 0.52),
        junction,
        left_tip: junction + dir(half) * (size * 0.66),
        right_tip: junction + dir(-half) * (size * 0.66),
    }
}

/// Renders the bifurcation pacenote badge beside the car.
///
/// A dark disc holds a fork: the incoming road splits into a left and a right branch. The recommended branch
/// is drawn bright and ends in an arrowhead, the other one dim. The tactical branch (joker road or pit lane)
/// carries its own accent color and is drawn thicker; it pulses while it is the recommended one.
pub fn render_bifurcation_pacenote(
    player_car: &Car,
    status: &BifurcationApproachStatus,
    scheme: CurveColorScheme,
    current_zoom: f32,
    anim_time: f32,
    scale: f32,
    brightness: f32,
) {
    let alpha = (compute_bifurcation_alpha(status, player_car.state.speed) * brightness).clamp(0.0, 1.0);
    if alpha <= 0.02 {
        return;
    }

    let zoom = current_zoom.max(0.5);
    let pulse = if status.recommends_tactical() { 1.0 + (anim_time * 6.0).sin().abs() * 0.08 } else { 1.0 };
    let size = 34.0 * scale / zoom;
    let plate_radius = size * 0.92;
    let thickness = 4.0 * scale / zoom;
    let outline = thickness + 3.0 / zoom;

    let center = bifurcation_disc_center(player_car, status, zoom, scale);
    let forward = player_car.forward_vector();
    let left = -player_car.right_vector();
    let fork = compute_fork_points(forward, left, center, size * pulse, status.divergence_angle);

    let (standard, _) = compute_curve_colors(scheme, 0.0, 1, alpha);
    let accent = if status.is_pit_entry { PIT_ACCENT } else { JOKER_ACCENT };
    let tactical = Color::new(accent.0, accent.1, accent.2, alpha);
    let stem_col = Color::new(0.80, 0.86, 0.95, alpha);
    let plate_col = Color::new(0.02, 0.03, 0.06, (0.55 * alpha).min(1.0));
    let shadow_col = Color::new(0.0, 0.0, 0.0, (0.80 * alpha).min(1.0));
    let dim = |c: Color| Color::new(c.r, c.g, c.b, c.a * 0.6);

    draw_circle(center.x, center.y, plate_radius, plate_col);

    // Branches: (tip, is the tactical one, is the left one).
    let branches = [
        (fork.left_tip, status.branch_left_is_tactical, true),
        (fork.right_tip, status.branch_right_is_tactical, false),
    ];
    let branch_style = |is_left_branch: bool, is_tactical: bool| {
        let recommended = status.recommended_branch_left == is_left_branch;
        let base = if is_tactical { tactical } else { standard };
        let color = if recommended { base } else { dim(base) };
        let width = if is_tactical { thickness * 1.3 } else { thickness };
        (color, width, recommended)
    };

    // 1. Dark outline under the whole fork
    draw_line(fork.stem_start.x, fork.stem_start.y, fork.junction.x, fork.junction.y, outline, shadow_col);
    for (tip, is_tactical, is_left_branch) in branches {
        let (_, width, _) = branch_style(is_left_branch, is_tactical);
        draw_line(fork.junction.x, fork.junction.y, tip.x, tip.y, width + 3.0 / zoom, shadow_col);
        draw_circle(tip.x, tip.y, (width + 3.0 / zoom) * 0.5, shadow_col);
    }

    // 2. Stem, then the dim branch, then the recommended one on top
    draw_line(fork.stem_start.x, fork.stem_start.y, fork.junction.x, fork.junction.y, thickness, stem_col);
    draw_circle(fork.junction.x, fork.junction.y, thickness * 0.5, stem_col);
    let mut ordered = branches.to_vec();
    ordered.sort_by_key(|&(_, is_tactical, is_left_branch)| branch_style(is_left_branch, is_tactical).2);
    for (tip, is_tactical, is_left_branch) in ordered {
        let (color, width, recommended) = branch_style(is_left_branch, is_tactical);
        draw_line(fork.junction.x, fork.junction.y, tip.x, tip.y, width, color);
        draw_circle(tip.x, tip.y, width * 0.5, color);
        if recommended {
            let dir = (tip - fork.junction).normalize_or_zero();
            let perp = Vec2::new(-dir.y, dir.x);
            let head_len = size * 0.26;
            let head_w = size * 0.17;
            let nose = tip + dir * head_len * 0.6;
            let base = tip - dir * head_len * 0.4;
            draw_triangle(mq_vec(nose + dir * (1.5 / zoom)), mq_vec(base + perp * (head_w + 1.5 / zoom)), mq_vec(base - perp * (head_w + 1.5 / zoom)), shadow_col);
            draw_triangle(mq_vec(nose), mq_vec(base + perp * head_w), mq_vec(base - perp * head_w), color);
        }
    }
}
