//! Spec 103: stamps a walled launch chute onto a circuit.
//!
//! A launch chute is a spur that runs beside the last stretch of the lap, ends in a rigid rear barrier, holds the
//! packed starting grid and merges into the circuit shortly before the start/finish line through a `Merge`
//! junction. The cars drive it once, at the start (`TrackLayout::entry_segment`), and never on laps 2+.
//!
//! [`Track::stamp_launch_chute`] is the single builder behind the Track Studio stamp tool and the rollout to the
//! official Autocross and Rallycross circuits: it splits the loop at the merge waypoint, adds the chute segment and
//! the junction, sets the entry segment of every layout, puts the grid on the pad, and regenerates the walls.

use std::fmt;

use glam::Vec2;
use wheelbase::SurfaceType;

use super::geometry::{BarrierType, Obstacle, ObstacleShape, SpawnPose, WallBarrier};
use super::network::{
    JunctionId, LaunchChuteConfig, MergeConfig, RoadJunction, RoadSegment, SegmentId, SocketId, SplineSocket,
};
use super::presets::{generate_packed_launch_grid, PackedGridPattern};
use super::spline::{SplineSample, TrackWaypoint};
use super::validation::{validate_launch_chute, ValidationSeverity};
use super::Track;

/// Pad width range the inspector offers (m).
pub const PAD_WIDTH_RANGE: (f32, f32) = (14.0, 18.0);
/// Pad length range (m): rear wall to the end of the straight part, before the merge ramp.
pub const PAD_LENGTH_RANGE: (f32, f32) = (35.0, 50.0);
/// How far before the start/finish line the chute may merge (m): the cars must not cross the line while merging,
/// and the pad should stay near the finish straight.
pub const MERGE_BEFORE_FINISH_RANGE: (f32, f32) = (10.0, 250.0);
/// Length over which the chute converges onto the circuit, ending tangent to it at the merge waypoint (m).
pub(super) const RAMP_LENGTH_M: f32 = 55.0;
/// Extra distance between the chute's road edge and the circuit's wall line, besides the two wall gaps (m).
const TRENCH_M: f32 = 2.0;
/// Distance between rows of the packed grid (m): longer than any car, so a row never overlaps the one before.
const ROW_SPACING_M: f32 = 6.5;
/// Free pad behind the last grid row (m).
const REAR_MARGIN_M: f32 = 4.0;
/// Distance between the waypoints of the chute (m).
const WAYPOINT_STEP_M: f32 = 10.0;

/// Side of the circuit the chute lies on, looking along the direction of travel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChuteSide {
    Left,
    Right,
}

/// What a launch chute looks like and where it merges (the Track Studio inspector edits these).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaunchChuteSpec {
    /// Index of the loop waypoint (`Track::spline.waypoints`) at which the chute merges.
    pub merge_waypoint: usize,
    pub side: ChuteSide,
    /// Launch pad width (m), within [`PAD_WIDTH_RANGE`].
    pub pad_width: f32,
    /// Launch pad length (m), within [`PAD_LENGTH_RANGE`].
    pub pad_length: f32,
    pub pattern: PackedGridPattern,
    /// Pad surface: `Concrete` or `Asphalt`.
    pub surface: SurfaceType,
}

impl LaunchChuteSpec {
    /// A 16 m x 40 m concrete pad with the Autocross 5-3 grid.
    pub fn new(merge_waypoint: usize, side: ChuteSide) -> Self {
        Self {
            merge_waypoint,
            side,
            pad_width: 16.0,
            pad_length: 40.0,
            pattern: PackedGridPattern::AutocrossFiveThree,
            surface: SurfaceType::Concrete,
        }
    }
}

/// Why a launch chute could not be stamped.
#[derive(Debug, Clone, PartialEq)]
pub struct LaunchChuteError(pub String);

impl fmt::Display for LaunchChuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for LaunchChuteError {}

fn err<T>(message: impl Into<String>) -> Result<T, LaunchChuteError> {
    Err(LaunchChuteError(message.into()))
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Arc distance along the loop from waypoint `index` to the start/finish line (m).
fn merge_distance_to_finish(track: &Track, index: usize) -> Result<(f32, f32), LaunchChuteError> {
    let Some(finish) = track.finish_line_checkpoint() else {
        return err("the circuit has no start/finish line");
    };
    let Some(waypoint) = track.spline.waypoints.get(index) else {
        return err(format!("waypoint {} does not exist", index + 1));
    };
    let loop_len = track.spline.total_length();
    let merge_dist = track.spline.project_point(waypoint.point).progress_distance;
    let finish_dist = track.spline.project_point((finish.gate.start + finish.gate.end) * 0.5).progress_distance;
    Ok((merge_dist, (finish_dist - merge_dist).rem_euclid(loop_len)))
}

/// Rough radius of an obstacle around its centre (m).
fn obstacle_reach(obstacle: &Obstacle) -> (Vec2, f32) {
    match &obstacle.shape {
        ObstacleShape::Circle { center, radius } => (*center, *radius),
        ObstacleShape::Box { center, half_extents, .. } => (*center, half_extents.length()),
        ObstacleShape::Polygon { vertices } => {
            let center = vertices.iter().copied().sum::<Vec2>() / vertices.len().max(1) as f32;
            (center, vertices.iter().map(|v| v.distance(center)).fold(0.0, f32::max))
        }
    }
}

impl Track {
    /// The launch chute of this circuit, if it has one.
    pub fn launch_chute(&self) -> Option<&LaunchChuteConfig> {
        self.network.as_ref()?.launch_chute.as_ref()
    }

    /// Stamps a launch chute on the circuit, or leaves the circuit unchanged and says why not.
    ///
    /// The chute merges at `spec.merge_waypoint`, which must lie [`MERGE_BEFORE_FINISH_RANGE`] metres before the
    /// start/finish line so that lap 1 is the chute plus one lap. The circuit must be a closed loop with a finish
    /// line and no chute yet; use [`Track::remove_launch_chute`] first to replace one.
    pub fn stamp_launch_chute(&mut self, spec: &LaunchChuteSpec) -> Result<(), LaunchChuteError> {
        let mut stamped = self.clone();
        stamped.stamp(spec)?;
        let errors: Vec<String> = validate_launch_chute(&stamped)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .map(|d| d.message)
            .collect();
        if !errors.is_empty() {
            return err(format!("the launch chute does not fit: {}", errors.join("; ")));
        }
        *self = stamped;
        Ok(())
    }

    /// Stamps a chute at the best waypoint for `template` (its `merge_waypoint` and `side` are ignored): the
    /// candidates before the finish line are tried nearest to 70 m first, on both sides. Returns the spec used.
    pub fn place_launch_chute(&mut self, template: &LaunchChuteSpec) -> Result<LaunchChuteSpec, LaunchChuteError> {
        let mut candidates: Vec<(usize, f32)> = (0..self.spline.waypoints.len())
            .filter_map(|k| {
                let (_, to_finish) = merge_distance_to_finish(self, k).ok()?;
                (MERGE_BEFORE_FINISH_RANGE.0..=MERGE_BEFORE_FINISH_RANGE.1).contains(&to_finish).then_some((k, to_finish))
            })
            .collect();
        candidates.sort_by(|a, b| (a.1 - 70.0).abs().total_cmp(&(b.1 - 70.0).abs()));

        let mut last_error = LaunchChuteError("no waypoint lies before the start/finish line".to_string());
        for (k, _) in candidates {
            for side in [ChuteSide::Right, ChuteSide::Left] {
                let spec = LaunchChuteSpec { merge_waypoint: k, side, ..*template };
                match self.stamp_launch_chute(&spec) {
                    Ok(()) => return Ok(spec),
                    Err(e) => last_error = e,
                }
            }
        }
        Err(last_error)
    }

    /// Removes the launch chute: the loop segments around the merge are joined again, the entry segments and the
    /// chute grid go, the walls are rebuilt, and the standard grid is placed. A circuit that only had a network
    /// for the chute goes back to having none. Returns false when there was no chute.
    pub fn remove_launch_chute(&mut self) -> bool {
        let Some(net) = self.network.as_mut() else { return false };
        let Some(chute) = net.launch_chute.take() else { return false };

        let merge = chute.merge_junction_id;
        let before = net.segments.iter().position(|s| s.exit_junction == Some(SocketId::new(merge, 0)));
        let after = net.segments.iter().position(|s| s.entry_junction == Some(SocketId::new(merge, 0)));
        if let (Some(before), Some(after)) = (before, after) {
            let joined = net.segments[after].clone();
            let (kept, removed) = (net.segments[before].id, joined.id);
            let seg = &mut net.segments[before];
            let offset = seg.length;
            seg.samples.extend(joined.samples.iter().skip(1).cloned().map(|mut s| {
                s.distance += offset;
                s
            }));
            seg.waypoints.extend(joined.waypoints.iter().skip(1).cloned());
            seg.length = offset + joined.length;
            seg.exit_junction = joined.exit_junction;
            for layout in &mut net.layouts {
                layout.segment_sequence.retain(|&s| s != removed);
            }
            for cp in &mut self.checkpoints {
                if cp.segment_id == Some(removed) {
                    cp.segment_id = Some(kept);
                }
            }
            net.segments.retain(|s| s.id != removed);
        }
        net.segments.retain(|s| s.id != chute.segment_id);
        net.junctions.retain(|j| j.id != merge);
        for layout in &mut net.layouts {
            layout.entry_segment = None;
        }

        if net.segments.len() == 1 && net.layouts.len() == 1 && net.junctions.is_empty() {
            self.network = None;
            for cp in &mut self.checkpoints {
                cp.segment_id = None;
            }
        }
        let (offset, barrier_type) = (self.effective_barrier_offset(), self.dominant_barrier_type().unwrap_or(BarrierType::TireWall));
        self.rebuild_geometry(offset, barrier_type);
        self.grid_positions.clear();
        self.auto_generate_grid_default();
        true
    }

    fn stamp(&mut self, spec: &LaunchChuteSpec) -> Result<(), LaunchChuteError> {
        if self.launch_chute().is_some() {
            return err("the circuit already has a launch chute");
        }
        if !self.spline.closed || self.spline.samples.len() < 2 {
            return err("launch chutes need a closed circuit");
        }
        if !(PAD_WIDTH_RANGE.0..=PAD_WIDTH_RANGE.1).contains(&spec.pad_width) {
            return err(format!("pad width must be {:.0}-{:.0} m", PAD_WIDTH_RANGE.0, PAD_WIDTH_RANGE.1));
        }
        if !(PAD_LENGTH_RANGE.0..=PAD_LENGTH_RANGE.1).contains(&spec.pad_length) {
            return err(format!("pad length must be {:.0}-{:.0} m", PAD_LENGTH_RANGE.0, PAD_LENGTH_RANGE.1));
        }
        if !matches!(spec.surface, SurfaceType::Concrete | SurfaceType::Asphalt) {
            return err("the pad surface must be Concrete or Asphalt");
        }
        let (merge_dist, to_finish) = merge_distance_to_finish(self, spec.merge_waypoint)?;
        if !(MERGE_BEFORE_FINISH_RANGE.0..=MERGE_BEFORE_FINISH_RANGE.1).contains(&to_finish) {
            return err(format!(
                "the chute must merge {:.0}-{:.0} m before the start/finish line (waypoint {} is {:.0} m before it)",
                MERGE_BEFORE_FINISH_RANGE.0,
                MERGE_BEFORE_FINISH_RANGE.1,
                spec.merge_waypoint + 1,
                to_finish
            ));
        }

        let loop_len = self.spline.total_length();
        let barrier_offset = self.effective_barrier_offset();
        let barrier_type = self.dominant_barrier_type().unwrap_or(BarrierType::TireWall);
        let merge_waypoint = self.spline.waypoints[spec.merge_waypoint].clone();
        let merge_sample = self.spline.sample_at_distance(merge_dist);

        // The centreline: the circuit's own line, offset sideways, over the pad, and eased back onto it over the
        // ramp so that it arrives tangent to the circuit at the merge waypoint.
        let sign = match spec.side {
            ChuteSide::Left => 1.0,
            ChuteSide::Right => -1.0,
        };
        let total = RAMP_LENGTH_M + spec.pad_length;
        let steps = (total / WAYPOINT_STEP_M).ceil() as usize;
        let mut waypoints = Vec::with_capacity(steps + 1);
        for j in 0..=steps {
            let u = total * (1.0 - j as f32 / steps as f32); // metres upstream of the merge
            let loop_sample = self.spline.sample_at_distance(merge_dist - u);
            let blend = smoothstep(u / RAMP_LENGTH_M);
            let curb = if (sign > 0.0 && loop_sample.left_curb) || (sign < 0.0 && loop_sample.right_curb) { 1.35 } else { 0.0 };
            let lateral = blend * (loop_sample.width * 0.5 + curb + 2.0 * barrier_offset + TRENCH_M + spec.pad_width * 0.5);
            let width = loop_sample.width + (spec.pad_width - loop_sample.width) * blend;
            let point = if j == steps { merge_waypoint.point } else { loop_sample.point + loop_sample.normal * (sign * lateral) };
            let mut waypoint = TrackWaypoint::new(point, width);
            waypoint.surface = Some(if blend >= 0.5 { spec.surface } else { loop_sample.surface });
            waypoint.elevation = loop_sample.elevation;
            waypoints.push(waypoint);
        }

        // Ids and the merge junction.
        let was_plain = self.network.is_none();
        let net = self.ensure_network();
        let next_segment = net.segments.iter().map(|s| s.id.0 + 1).max().unwrap_or(0);
        let (after_id, chute_id) = (SegmentId(next_segment), SegmentId(next_segment + 1));
        let merge_id = JunctionId(net.junctions.iter().map(|j| j.id.0 + 1).max().unwrap_or(0));

        let socket = SplineSocket::new(merge_waypoint.point, merge_sample.tangent, merge_waypoint.width)
            .with_surface(merge_sample.surface)
            .with_elevation(merge_sample.elevation);
        let mut chute = RoadSegment::new(chute_id, "Launch Chute", waypoints)
            .with_junctions(None, Some(SocketId::new(merge_id, 1)));
        chute.recompute_samples(None, Some(&socket));

        // Split the loop segment that holds the merge waypoint in two.
        let Some(loop_idx) = net.segments.iter().position(|s| {
            s.waypoints.len() > 2
                && s.waypoints[1..s.waypoints.len() - 1].iter().any(|w| (w.point - merge_waypoint.point).length() < 0.05)
                && net.layouts.iter().all(|l| l.segment_sequence.contains(&s.id))
        }) else {
            return err("the merge waypoint must lie inside a segment that every layout drives");
        };
        let split_at = net.segments[loop_idx].waypoints.iter().position(|w| (w.point - merge_waypoint.point).length() < 0.05).unwrap();
        let split_sample = net.segments[loop_idx]
            .samples
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.point.distance_squared(merge_waypoint.point).total_cmp(&b.1.point.distance_squared(merge_waypoint.point)))
            .map(|(i, _)| i)
            .unwrap();
        let first = &net.segments[loop_idx];
        let mut before = RoadSegment { waypoints: first.waypoints[..=split_at].to_vec(), samples: first.samples[..=split_sample].to_vec(), ..first.clone() };
        before.length = before.samples.last().map_or(0.0, |s| s.distance);
        before.exit_junction = Some(SocketId::new(merge_id, 0));
        let origin = first.samples[split_sample].distance;
        let mut after = RoadSegment {
            id: after_id,
            name: format!("{} (after chute merge)", first.name),
            waypoints: first.waypoints[split_at..].to_vec(),
            samples: first.samples[split_sample..]
                .iter()
                .map(|s| SplineSample { distance: s.distance - origin, ..*s })
                .collect(),
            entry_junction: Some(SocketId::new(merge_id, 0)),
            ..first.clone()
        };
        if after.waypoints.len() < 2 || before.waypoints.len() < 2 {
            return err("the merge waypoint cannot be the first or last of its segment");
        }
        if was_plain {
            // The one segment of a plain circuit holds the closed loop without its closing point: the last
            // segment ends on the first waypoint, as the baked RX networks do.
            after.waypoints.push(before.waypoints[0].clone());
            after.samples.push(SplineSample { distance: loop_len - origin, ..before.samples[0] });
        }
        after.length = after.samples.last().map_or(0.0, |s| s.distance);
        net.segments[loop_idx] = before;
        net.segments.push(after);

        let main_id = net.segments[loop_idx].id;
        for layout in &mut net.layouts {
            if let Some(pos) = layout.segment_sequence.iter().position(|&s| s == main_id) {
                layout.segment_sequence.insert(pos + 1, after_id);
            }
            layout.entry_segment = Some(chute_id);
        }
        net.junctions.push(RoadJunction::merge(
            merge_id,
            "Launch Chute Merge",
            vec![socket, socket],
            socket,
            Some(MergeConfig { convergence_point: merge_waypoint.point, merge_angle: 0.0, merge_length: RAMP_LENGTH_M }),
        ));

        // Pack the grid at the rear of the pad: the straight layout of the pattern, moved onto the chute's
        // centreline. The rear row stands REAR_MARGIN_M from the rear barrier.
        let rows = spec.pattern.row_counts().len() as f32;
        let s_front = REAR_MARGIN_M + (rows - 1.0) * ROW_SPACING_M;
        let spline = chute.to_spline();
        let grid_slots: Vec<SpawnPose> =
            generate_packed_launch_grid(Vec2::ZERO, Vec2::X, spec.pad_width, spec.pattern, ROW_SPACING_M)
                .into_iter()
                .map(|slot| {
                    let on_line = spline.sample_at_distance(s_front + slot.position.x);
                    let pos = on_line.point + on_line.normal * slot.position.y;
                    SpawnPose::new(pos, on_line.tangent.y.atan2(on_line.tangent.x), slot.grid_slot)
                })
                .collect();

        // Not on a branch of the baked network yet: the chute is not in any layout's sequence.
        net.segments.push(chute);
        net.launch_chute = Some(LaunchChuteConfig {
            segment_id: chute_id,
            terminal_barrier: WallBarrier::new(Vec2::ZERO, Vec2::ZERO, barrier_type),
            side_barriers: Vec::new(),
            merge_junction_id: merge_id,
            grid_slots: grid_slots.clone(),
            surface: spec.surface,
            pad_width: spec.pad_width,
        });

        // Every checkpoint belongs to the segment it lies on: after the merge waypoint means the `after` segment.
        self.assign_checkpoint_segments(main_id, after_id, merge_dist);

        self.grid_positions = grid_slots;
        self.trim_walls_for_network();
        self.generate_network_walls();
        if let Some(net) = self.network.as_mut() {
            net.recompute_composite_splines();
        }
        self.reject_obstacles_on_chute(spec)
    }

    /// Gives the checkpoints their segment: those in the split segment before the merge keep it, the others get
    /// `after`. A circuit that had no network has no ids yet: all checkpoints are placed on the loop's two parts.
    fn assign_checkpoint_segments(&mut self, main: SegmentId, after: SegmentId, merge_dist: f32) {
        let had_ids = self.checkpoints.iter().any(|cp| cp.segment_id.is_some());
        let spline = &self.spline;
        for cp in &mut self.checkpoints {
            if cp.is_finish_line {
                cp.segment_id.get_or_insert(main);
                continue;
            }
            if had_ids && cp.segment_id != Some(main) {
                continue;
            }
            let progress = spline.project_point((cp.gate.start + cp.gate.end) * 0.5).progress_distance;
            cp.segment_id = Some(if progress > merge_dist { after } else { main });
        }
    }

    /// A tree, rock or grandstand on the chute would stop the cars: the stamp is refused then.
    fn reject_obstacles_on_chute(&self, spec: &LaunchChuteSpec) -> Result<(), LaunchChuteError> {
        let Some(chute) = self.launch_chute().and_then(|c| self.network.as_ref()?.get_segment(c.segment_id)) else {
            return Ok(());
        };
        let reach = spec.pad_width * 0.5 + 1.0;
        for obstacle in self.geometry.all_obstacles_with_scenery() {
            let (center, radius) = obstacle_reach(obstacle);
            if chute.samples.iter().any(|s| s.point.distance(center) < reach + radius) {
                return err(format!("obstacle '{}' stands on the chute", obstacle.name));
            }
        }
        Ok(())
    }
}
