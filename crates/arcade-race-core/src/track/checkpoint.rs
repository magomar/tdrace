use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::geometry::LineSegment;
use super::network::{SegmentId, TrackNetwork};
use super::spline::{SplineProjection, TrackSpline};
use wheelbase::Car;

/// Directional crossing result when testing car trajectory across a checkpoint gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointCrossResult {
    /// Car crossed the gate in the correct forward racing direction.
    Forward,
    /// Car crossed the gate backwards (wrong way).
    Backward,
}

/// A checkpoint gate spanning across the track to track progression, lap times, and sectors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique sequential checkpoint index [0..N-1].
    pub id: usize,
    /// Physical 2D gate line segment across the track.
    pub gate: LineSegment,
    /// Expected forward crossing unit vector.
    pub direction: Vec2,
    /// Sector index (e.g. 0, 1, 2 for a 3-sector track).
    pub sector: usize,
    /// True if this checkpoint serves as the Start / Finish timing line.
    pub is_finish_line: bool,
    /// True if this checkpoint marks the pit lane entrance.
    pub is_pit_entry: bool,
    /// True if this checkpoint marks the pit lane exit.
    pub is_pit_exit: bool,
    /// Arc-length distance along the spline in meters.
    pub target_distance: f32,
    /// Track surface elevation at this checkpoint in meters (default: 0.0).
    #[serde(default)]
    pub elevation: f32,
    /// Optional road segment ID where this checkpoint is located in a TrackNetwork.
    #[serde(default)]
    pub segment_id: Option<SegmentId>,
    /// True if this checkpoint marks a joker lap detour branch.
    #[serde(default)]
    pub is_joker: bool,
}

impl Checkpoint {
    pub fn new(
        id: usize,
        gate: LineSegment,
        direction: Vec2,
        sector: usize,
        is_finish_line: bool,
    ) -> Self {
        Self {
            id,
            gate,
            direction: direction.normalize_or_zero(),
            sector,
            is_finish_line,
            is_pit_entry: false,
            is_pit_exit: false,
            target_distance: 0.0,
            elevation: 0.0,
            segment_id: None,
            is_joker: false,
        }
    }

    pub const fn with_segment(mut self, segment_id: SegmentId) -> Self {
        self.segment_id = Some(segment_id);
        self
    }

    pub const fn with_joker(mut self, is_joker: bool) -> Self {
        self.is_joker = is_joker;
        self
    }

    pub fn with_pit_flags(mut self, is_entry: bool, is_exit: bool) -> Self {
        self.is_pit_entry = is_entry;
        self.is_pit_exit = is_exit;
        self
    }

    pub fn with_target_distance(mut self, distance: f32) -> Self {
        self.target_distance = distance;
        self
    }

    pub fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    /// Tests if a trajectory segment from `prev_pos` to `curr_pos` crossed this checkpoint gate.
    pub fn test_crossing(&self, prev_pos: Vec2, curr_pos: Vec2) -> Option<CheckpointCrossResult> {
        let trajectory = LineSegment::new(prev_pos, curr_pos);
        if self.gate.intersect_segment(&trajectory).is_some() {
            let movement = curr_pos - prev_pos;
            let dot = movement.dot(self.direction);
            if dot > 0.0 {
                Some(CheckpointCrossResult::Forward)
            } else {
                Some(CheckpointCrossResult::Backward)
            }
        } else {
            None
        }
    }
}

/// Comprehensive real-time tracker for lap progress, timing, sector splits, and race rule violations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackProgressTracker {
    /// Current lap number (0 = warmup / out-lap, 1 = first flying lap, etc.).
    pub current_lap: u32,
    /// Elapsed time in seconds on the current lap.
    pub lap_time: f32,
    /// Personal best lap time in seconds (if any valid lap completed).
    pub best_lap_time: Option<f32>,
    /// Most recently completed lap time in seconds.
    pub last_lap_time: Option<f32>,
    /// Current timing sector index (0-indexed).
    pub current_sector: usize,
    /// Elapsed times in each sector for current lap.
    pub sector_times: Vec<f32>,
    /// Best sector times achieved across all completed laps.
    pub best_sector_times: Vec<Option<f32>>,
    /// Sector times from the most recently completed lap.
    #[serde(default)]
    pub last_lap_sector_times: Vec<f32>,
    /// ID of the last validated checkpoint crossed in sequence.
    pub last_checkpoint_idx: usize,
    /// ID of the expected next sequential checkpoint to cross.
    pub next_checkpoint_idx: usize,
    /// Number of distinct sequential checkpoints passed in current lap.
    pub checkpoints_passed_this_lap: usize,
    /// Total number of timing checkpoints on track.
    pub total_checkpoints: usize,
    /// Cumulative arc-length distance along the centerline spline (meters).
    pub progress_distance: f32,
    /// Normalized progress along the circuit [0.0, 1.0).
    pub normalized_progress: f32,
    /// Total cumulative distance driven by car across entire session (meters).
    pub total_distance_travelled: f32,
    /// Whether the car is currently driving the wrong way down the track.
    pub is_wrong_way: bool,
    /// Cumulative duration driving wrong way in seconds.
    pub wrong_way_timer: f32,
    /// Whether the car is currently completely off-track.
    pub is_off_track: bool,
    /// Cumulative duration off-track in seconds.
    pub off_track_timer: f32,
    /// Whether the car is currently navigating the pit lane.
    pub in_pit_lane: bool,
    /// Total count of completed pit stops.
    pub pit_stops: u32,
    /// Whether a new lap was completed on the most recent step.
    pub lap_completed: bool,
    /// Previous step world position of the car.
    pub last_position: Option<Vec2>,
    /// Optional multi-route progress tracker for branching track networks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multi_route: Option<MultiRouteProgressTracker>,
}

impl TrackProgressTracker {
    /// Creates a new progress tracker initialized for a track with `num_checkpoints` and `num_sectors`.
    pub fn new(num_checkpoints: usize, num_sectors: usize) -> Self {
        let sectors = num_sectors.max(1);
        Self {
            current_lap: 1,
            lap_time: 0.0,
            best_lap_time: None,
            last_lap_time: None,
            current_sector: 0,
            sector_times: vec![0.0; sectors],
            best_sector_times: vec![None; sectors],
            last_lap_sector_times: vec![0.0; sectors],
            last_checkpoint_idx: 0,
            next_checkpoint_idx: 0,
            checkpoints_passed_this_lap: 0,
            total_checkpoints: num_checkpoints.max(1),
            progress_distance: 0.0,
            normalized_progress: 0.0,
            total_distance_travelled: 0.0,
            is_wrong_way: false,
            wrong_way_timer: 0.0,
            is_off_track: false,
            off_track_timer: 0.0,
            in_pit_lane: false,
            pit_stops: 0,
            lap_completed: false,
            last_position: None,
            multi_route: None,
        }
    }

    /// Resets all race tracking state to initial conditions at the starting line.
    pub fn reset(&mut self) {
        let sectors = self.sector_times.len();
        self.current_lap = 1;
        self.lap_time = 0.0;
        self.best_lap_time = None;
        self.last_lap_time = None;
        self.current_sector = 0;
        self.sector_times = vec![0.0; sectors];
        self.best_sector_times = vec![None; sectors];
        self.last_lap_sector_times = vec![0.0; sectors];
        self.last_checkpoint_idx = 0;
        self.next_checkpoint_idx = 0;
        self.checkpoints_passed_this_lap = 0;
        self.progress_distance = 0.0;
        self.normalized_progress = 0.0;
        self.total_distance_travelled = 0.0;
        self.is_wrong_way = false;
        self.wrong_way_timer = 0.0;
        self.is_off_track = false;
        self.off_track_timer = 0.0;
        self.in_pit_lane = false;
        self.pit_stops = 0;
        self.lap_completed = false;
        self.last_position = None;
        if let Some(multi) = &mut self.multi_route {
            multi.reset_state();
        }
    }

    /// Updates race progression given the car's state, track spline, checkpoints, and timestep.
    pub fn update(
        &mut self,
        car: &Car,
        spline: &TrackSpline,
        checkpoints: &[Checkpoint],
        dt: f32,
    ) {
        self.lap_completed = false;
        let car_pos = car.state.position;

        // 1. Advance timing clocks
        self.lap_time += dt;
        if self.current_sector < self.sector_times.len() {
            self.sector_times[self.current_sector] += dt;
        }
        self.total_distance_travelled += car.state.speed * dt;

        // 2. Project onto spline centerline with continuity constraint
        let proj = if self.last_position.is_some() && self.total_distance_travelled > 0.0 {
            spline.project_point_continuity(car_pos, self.progress_distance, 50.0)
        } else {
            spline.project_point(car_pos)
        };
        self.progress_distance = proj.progress_distance;
        self.normalized_progress = proj.normalized_progress;

        // 3. Wrong-way detection via spline tangent alignment
        let car_fwd = car.forward_vector();
        let alignment = car_fwd.dot(proj.tangent);
        let facing_wrong_way = alignment < -0.25;

        if facing_wrong_way {
            self.is_wrong_way = true;
            self.wrong_way_timer += dt;
        } else {
            self.is_wrong_way = false;
            self.wrong_way_timer = 0.0;
        }

        // 4. Off-track detection: check if car center and wheels are outside track ribbon & curbs
        if !proj.is_on_track && !proj.is_on_curb {
            self.is_off_track = true;
            self.off_track_timer += dt;
        } else {
            self.is_off_track = false;
            self.off_track_timer = 0.0;
        }

        // 5. Checkpoint gate crossing detection
        if let Some(prev_pos) = self.last_position {
            for (idx, cp) in checkpoints.iter().enumerate() {
                if let Some(cross_res) = cp.test_crossing(prev_pos, car_pos) {
                    match cross_res {
                        CheckpointCrossResult::Forward => {
                            if cp.is_pit_entry {
                                self.in_pit_lane = true;
                            } else if cp.is_pit_exit {
                                if self.in_pit_lane {
                                    self.pit_stops += 1;
                                }
                                self.in_pit_lane = false;
                            } else if idx == self.next_checkpoint_idx {
                                self.handle_forward_checkpoint_pass(idx, cp, checkpoints.len());
                            } else if idx > self.next_checkpoint_idx && (idx - self.next_checkpoint_idx) <= 3 {
                                // Forward sequence jump within current lap (allowing fast motion or shortcuts)
                                self.handle_forward_checkpoint_pass(idx, cp, checkpoints.len());
                            } else if idx == 0 && self.next_checkpoint_idx >= checkpoints.len().saturating_sub(2) {
                                // Final sector crossing to finish line
                                self.handle_forward_checkpoint_pass(idx, cp, checkpoints.len());
                            }
                        }
                        CheckpointCrossResult::Backward => {
                            // Backward crossing on any gate triggers immediate wrong way warning
                            self.is_wrong_way = true;
                            self.wrong_way_timer += dt;
                        }
                    }
                }
            }
        }

        self.last_position = Some(car_pos);
    }

    fn handle_forward_checkpoint_pass(
        &mut self,
        idx: usize,
        cp: &Checkpoint,
        total_cps: usize,
    ) {
        self.last_checkpoint_idx = idx;
        self.next_checkpoint_idx = (idx + 1) % total_cps;
        self.checkpoints_passed_this_lap += 1;

        // Pit lane triggers
        if cp.is_pit_entry {
            self.in_pit_lane = true;
        }
        if cp.is_pit_exit {
            if self.in_pit_lane {
                self.pit_stops += 1;
            }
            self.in_pit_lane = false;
        }

        // Sector split tracking
        if cp.sector != self.current_sector {
            let completed_sector = self.current_sector;
            let time = self.sector_times[completed_sector];
            if self.best_sector_times[completed_sector].map_or(true, |b| time < b) {
                self.best_sector_times[completed_sector] = Some(time);
            }
            self.current_sector = cp.sector;
        }

        // Finish line crossing check
        if cp.is_finish_line {
            // Anti-cheat requirement: Must have passed at least 70% of total checkpoints to count a valid lap
            let min_required_cps = (total_cps * 7) / 10;
            if self.checkpoints_passed_this_lap >= min_required_cps {
                // Complete current sector time
                let last_sec = self.current_sector;
                let sec_time = self.sector_times[last_sec];
                if self.best_sector_times[last_sec].map_or(true, |b| sec_time < b) {
                    self.best_sector_times[last_sec] = Some(sec_time);
                }

                // Lap complete!
                let finished_lap_time = self.lap_time;
                self.last_lap_time = Some(finished_lap_time);
                if self.best_lap_time.map_or(true, |b| finished_lap_time < b) {
                    self.best_lap_time = Some(finished_lap_time);
                }

                self.current_lap += 1;
                self.lap_time = 0.0;
                self.checkpoints_passed_this_lap = 0;
                self.lap_completed = true;

                // Save completed lap sector times before resetting for next lap
                self.last_lap_sector_times = self.sector_times.clone();

                // Reset current lap sector times
                for s in self.sector_times.iter_mut() {
                    *s = 0.0;
                }
                self.current_sector = 0;
            }
        }
    }

    /// Updates race progression on a multi-branch track network.
    pub fn update_network(
        &mut self,
        car: &Car,
        network: &TrackNetwork,
        checkpoints: &[Checkpoint],
        dt: f32,
    ) {
        let multi = self.multi_route.get_or_insert_with(|| {
            MultiRouteProgressTracker::from_network(network, None, self.sector_times.len())
        });
        multi.update(car, network, checkpoints, dt);

        // Synchronize core tracker metrics
        self.current_lap = multi.current_lap;
        self.lap_time = multi.lap_time;
        self.best_lap_time = multi.best_lap_time;
        self.last_lap_time = multi.last_lap_time;
        self.current_sector = multi.current_sector;
        self.sector_times = multi.sector_times.clone();
        self.best_sector_times = multi.best_sector_times.clone();
        self.last_lap_sector_times = multi.last_lap_sector_times.clone();
        self.last_checkpoint_idx = multi.last_checkpoint_id;
        self.next_checkpoint_idx = multi.next_checkpoint_index;
        self.checkpoints_passed_this_lap = multi.checkpoints_passed_this_lap;
        self.progress_distance = multi.layout_distance;
        self.normalized_progress = multi.layout_progress;
        self.total_distance_travelled = multi.total_distance_travelled;
        self.is_wrong_way = multi.is_wrong_way;
        self.wrong_way_timer = multi.wrong_way_timer;
        self.is_off_track = multi.is_off_track;
        self.off_track_timer = multi.off_track_timer;
        self.in_pit_lane = multi.in_pit_lane;
        self.pit_stops = multi.pit_stops;
        self.lap_completed = multi.lap_completed;
        self.last_position = multi.last_position;
    }
}

/// Comprehensive progression tracker supporting multi-branch track networks, alternative layouts, and Joker laps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiRouteProgressTracker {
    /// Currently active or dynamically detected layout ID.
    pub active_layout_id: String,
    /// Current road segment ID the car is traversing.
    pub current_segment_id: SegmentId,
    /// Index of next expected checkpoint in the active layout's sequence.
    pub next_checkpoint_index: usize,
    /// Number of joker laps completed by this driver.
    pub joker_laps_completed: u32,
    /// Whether the car is currently navigating a branch detour off the default trunk.
    pub is_in_branch: bool,
    /// Normalized progress along the currently active layout [0.0, 1.0).
    pub layout_progress: f32,
    /// Cumulative arc-length distance along the currently active layout in meters.
    pub layout_distance: f32,
    /// Arc-length progress distance along the current segment in meters.
    pub segment_progress_distance: f32,

    // Timing & Lap Progression
    /// Current lap number (1 = first lap, etc.).
    pub current_lap: u32,
    /// Elapsed time in seconds on current lap.
    pub lap_time: f32,
    /// Personal best lap time in seconds.
    pub best_lap_time: Option<f32>,
    /// Most recently completed lap time in seconds.
    pub last_lap_time: Option<f32>,
    /// Current timing sector index (0-indexed).
    pub current_sector: usize,
    /// Elapsed times in each sector for current lap.
    pub sector_times: Vec<f32>,
    /// Best sector times achieved across all completed laps.
    pub best_sector_times: Vec<Option<f32>>,
    /// Sector times from the most recently completed lap.
    #[serde(default)]
    pub last_lap_sector_times: Vec<f32>,
    /// ID of the last validated checkpoint crossed.
    pub last_checkpoint_id: usize,
    /// Number of distinct checkpoints passed in current lap.
    pub checkpoints_passed_this_lap: usize,
    /// Total cumulative distance travelled by car across session in meters.
    pub total_distance_travelled: f32,
    /// Whether a new lap was completed on the most recent step.
    pub lap_completed: bool,
    /// Whether the current lap has taken a joker lap detour.
    pub is_joker_lap: bool,

    // Penalties & Status
    /// Whether the car is currently driving the wrong way down its current segment.
    pub is_wrong_way: bool,
    /// Cumulative duration driving wrong way in seconds.
    pub wrong_way_timer: f32,
    /// Whether the car is currently off track.
    pub is_off_track: bool,
    /// Cumulative duration off track in seconds.
    pub off_track_timer: f32,
    /// Whether the car is navigating the pit lane.
    pub in_pit_lane: bool,
    /// Total count of completed pit stops.
    pub pit_stops: u32,
    /// Previous step world position of the car.
    pub last_position: Option<Vec2>,
}

impl MultiRouteProgressTracker {
    /// Creates a new multi-route tracker for a specific layout and starting segment.
    pub fn new(
        active_layout_id: impl Into<String>,
        start_segment_id: SegmentId,
        num_sectors: usize,
    ) -> Self {
        let sectors = num_sectors.max(1);
        Self {
            active_layout_id: active_layout_id.into(),
            current_segment_id: start_segment_id,
            next_checkpoint_index: 0,
            joker_laps_completed: 0,
            is_in_branch: false,
            layout_progress: 0.0,
            layout_distance: 0.0,
            segment_progress_distance: 0.0,
            current_lap: 1,
            lap_time: 0.0,
            best_lap_time: None,
            last_lap_time: None,
            current_sector: 0,
            sector_times: vec![0.0; sectors],
            best_sector_times: vec![None; sectors],
            last_lap_sector_times: vec![0.0; sectors],
            last_checkpoint_id: 0,
            checkpoints_passed_this_lap: 0,
            total_distance_travelled: 0.0,
            lap_completed: false,
            is_joker_lap: false,
            is_wrong_way: false,
            wrong_way_timer: 0.0,
            is_off_track: false,
            off_track_timer: 0.0,
            in_pit_lane: false,
            pit_stops: 0,
            last_position: None,
        }
    }

    /// Initializes a tracker from a `TrackNetwork` automatically selecting initial layout and starting segment.
    pub fn from_network(
        network: &TrackNetwork,
        initial_layout_id: Option<&str>,
        num_sectors: usize,
    ) -> Self {
        let layout = network.active_or_default_layout(initial_layout_id);
        let layout_id = layout.map_or("main", |l| l.id.as_str());
        let start_seg = layout
            .and_then(|l| l.segment_sequence.first().copied())
            .unwrap_or_else(|| network.segments.first().map_or(SegmentId(0), |s| s.id));
        Self::new(layout_id, start_seg, num_sectors)
    }

    /// Resets all race tracking state to initial conditions.
    pub fn reset_state(&mut self) {
        let sectors = self.sector_times.len();
        self.current_lap = 1;
        self.lap_time = 0.0;
        self.best_lap_time = None;
        self.last_lap_time = None;
        self.current_sector = 0;
        self.sector_times = vec![0.0; sectors];
        self.best_sector_times = vec![None; sectors];
        self.last_lap_sector_times = vec![0.0; sectors];
        self.last_checkpoint_id = 0;
        self.next_checkpoint_index = 0;
        self.checkpoints_passed_this_lap = 0;
        self.layout_distance = 0.0;
        self.layout_progress = 0.0;
        self.segment_progress_distance = 0.0;
        self.total_distance_travelled = 0.0;
        self.is_wrong_way = false;
        self.wrong_way_timer = 0.0;
        self.is_off_track = false;
        self.off_track_timer = 0.0;
        self.in_pit_lane = false;
        self.pit_stops = 0;
        self.lap_completed = false;
        self.is_joker_lap = false;
        self.is_in_branch = false;
        self.last_position = None;
    }

    /// Explicitly switches the active layout.
    pub fn set_layout(&mut self, layout_id: &str, network: &TrackNetwork) {
        if let Some(layout) = network.get_layout(layout_id) {
            self.active_layout_id = layout.id.clone();
            if let Some(&first_seg) = layout.segment_sequence.first() {
                self.current_segment_id = first_seg;
            }
            if let Some(default_layout) = network.get_layout(&network.default_layout_id) {
                self.is_in_branch = !default_layout.segment_sequence.contains(&self.current_segment_id);
            }
        }
    }

    /// Updates race progression given the car's state, track network, checkpoints, and timestep.
    pub fn update(
        &mut self,
        car: &Car,
        network: &TrackNetwork,
        checkpoints: &[Checkpoint],
        dt: f32,
    ) {
        self.lap_completed = false;
        let car_pos = car.state.position;

        // 1. Advance timing clocks
        self.lap_time += dt;
        if self.current_sector < self.sector_times.len() {
            self.sector_times[self.current_sector] += dt;
        }
        self.total_distance_travelled += car.state.speed * dt;

        // 2. Checkpoint crossing detection
        if let Some(prev_pos) = self.last_position {
            for cp in checkpoints {
                if let Some(cross_res) = cp.test_crossing(prev_pos, car_pos) {
                    match cross_res {
                        CheckpointCrossResult::Backward => {
                            self.is_wrong_way = true;
                            self.wrong_way_timer += dt;
                        }
                        CheckpointCrossResult::Forward => {
                            if cp.is_pit_entry {
                                self.in_pit_lane = true;
                            } else if cp.is_pit_exit {
                                if self.in_pit_lane {
                                    self.pit_stops += 1;
                                }
                                self.in_pit_lane = false;
                            } else {
                                self.process_forward_crossing(cp, network, checkpoints);
                            }
                        }
                    }
                }
            }
        }

        // 3. Spline projection & segment continuity
        let mut curr_seg = network.get_segment(self.current_segment_id);
        if curr_seg.is_none() {
            if let Some(closest) = network.segments.iter().min_by(|a, b| {
                let da = (a.project_point(car_pos).closest_point - car_pos).length_squared();
                let db = (b.project_point(car_pos).closest_point - car_pos).length_squared();
                da.partial_cmp(&db).unwrap()
            }) {
                self.current_segment_id = closest.id;
                curr_seg = Some(closest);
            }
        }

        let mut proj = if let Some(seg) = curr_seg {
            if self.last_position.is_some() && self.total_distance_travelled > 0.0 {
                seg.project_point_continuity(car_pos, self.segment_progress_distance, 60.0)
            } else {
                seg.project_point(car_pos)
            }
        } else {
            SplineProjection {
                closest_point: car_pos,
                distance_to_spline: 0.0,
                lateral_offset: 0.0,
                progress_distance: 0.0,
                normalized_progress: 0.0,
                tangent: Vec2::X,
                normal: Vec2::Y,
                track_width: 10.0,
                left_curb: false,
                right_curb: false,
                is_on_track: true,
                is_on_curb: false,
                base_surface: wheelbase::SurfaceType::Asphalt,
                elevation: 0.0,
                bank_angle: 0.0,
            }
        };

        // 4. Downstream segment transition
        if let Some(seg) = curr_seg {
            let near_end = proj.progress_distance >= (seg.length - 25.0).max(0.0) || !proj.is_on_track;
            if near_end {
                let mut best_cand: Option<(SegmentId, SplineProjection)> = None;
                let mut min_cand_dist = f32::INFINITY;

                let mut candidate_ids = Vec::new();
                if let Some(exit_sock) = seg.exit_junction {
                    for s in &network.segments {
                        if let Some(entry_sock) = s.entry_junction {
                            if entry_sock.junction_id == exit_sock.junction_id {
                                candidate_ids.push(s.id);
                            }
                        }
                    }
                }

                if let Some(layout) = network.active_or_default_layout(Some(&self.active_layout_id)) {
                    if let Some(pos) = layout.segment_sequence.iter().position(|&sid| sid == seg.id) {
                        let next_sid = layout.segment_sequence[(pos + 1) % layout.segment_sequence.len()];
                        if !candidate_ids.contains(&next_sid) {
                            candidate_ids.push(next_sid);
                        }
                    }
                }

                for &cand_id in &candidate_ids {
                    if cand_id == seg.id {
                        continue;
                    }
                    if let Some(cand) = network.get_segment(cand_id) {
                        let cand_proj = cand.project_point_continuity(car_pos, 0.0, 50.0);
                        let is_forward = car.forward_vector().dot(cand_proj.tangent) > -0.25;
                        if cand_proj.progress_distance <= 50.0 && is_forward {
                            let half_w = cand.sample_at_distance(cand_proj.progress_distance).width * 0.5 + 4.0;
                            if cand_proj.distance_to_spline <= half_w && cand_proj.distance_to_spline < min_cand_dist {
                                min_cand_dist = cand_proj.distance_to_spline;
                                best_cand = Some((cand_id, cand_proj));
                            }
                        }
                    }
                }

                if let Some((new_seg_id, new_proj)) = best_cand {
                    self.current_segment_id = new_seg_id;
                    proj = new_proj;
                }
            }
        }

        self.segment_progress_distance = proj.progress_distance;

        // 5. Layout distance and normalized progress
        if let Some(layout) = network.active_or_default_layout(Some(&self.active_layout_id)) {
            if let Some(seg_idx) = layout.segment_sequence.iter().position(|&sid| sid == self.current_segment_id) {
                let dist_before: f32 = layout.segment_sequence[0..seg_idx]
                    .iter()
                    .filter_map(|&sid| network.get_segment(sid))
                    .map(|s| s.length)
                    .sum();
                let total_len: f32 = layout.segment_sequence
                    .iter()
                    .filter_map(|&sid| network.get_segment(sid))
                    .map(|s| s.length)
                    .sum();
                let total = if total_len > 1.0 { total_len } else { layout.total_lap_length.max(1.0) };
                self.layout_distance = (dist_before + self.segment_progress_distance).clamp(0.0, total);
                self.layout_progress = (self.layout_distance / total).clamp(0.0, 0.999999);
            } else {
                self.layout_distance = self.segment_progress_distance;
                self.layout_progress = proj.normalized_progress;
            }
        } else {
            self.layout_distance = self.segment_progress_distance;
            self.layout_progress = proj.normalized_progress;
        }

        // 6. Branch status
        if let Some(default_layout) = network.get_layout(&network.default_layout_id) {
            self.is_in_branch = !default_layout.segment_sequence.contains(&self.current_segment_id);
        } else {
            self.is_in_branch = false;
        }

        // 7. Wrong-way detection via current segment tangent
        let car_fwd = car.forward_vector();
        let alignment = car_fwd.dot(proj.tangent);
        let facing_wrong_way = alignment < -0.25;
        if facing_wrong_way {
            self.is_wrong_way = true;
            self.wrong_way_timer += dt;
        } else {
            self.is_wrong_way = false;
            self.wrong_way_timer = 0.0;
        }

        // 8. Off-track detection
        if !proj.is_on_track && !proj.is_on_curb {
            let on_any_segment = network.segments.iter().any(|s| {
                let p = s.project_point(car_pos);
                p.is_on_track || p.is_on_curb
            });
            if !on_any_segment {
                self.is_off_track = true;
                self.off_track_timer += dt;
            } else {
                self.is_off_track = false;
                self.off_track_timer = 0.0;
            }
        } else {
            self.is_off_track = false;
            self.off_track_timer = 0.0;
        }

        self.last_position = Some(car_pos);
    }

    fn process_forward_crossing(
        &mut self,
        cp: &Checkpoint,
        network: &TrackNetwork,
        all_cps: &[Checkpoint],
    ) {
        let active_layout = network.active_or_default_layout(Some(&self.active_layout_id));
        let active_cps: &[usize] = active_layout.map(|l| l.checkpoint_ids.as_slice()).unwrap_or(&[]);

        // 1. Check if checkpoint matches current active layout
        if !active_cps.is_empty() {
            if let Some(pos) = active_cps.iter().position(|&id| id == cp.id) {
                let is_expected = pos == self.next_checkpoint_index;
                let is_small_jump = pos > self.next_checkpoint_index && (pos - self.next_checkpoint_index) <= 3;
                let is_finish_crossing = pos == 0 && self.next_checkpoint_index >= active_cps.len().saturating_sub(2);

                if is_expected || is_small_jump || is_finish_crossing {
                    let layout_id = active_layout.map(|l| l.id.clone()).unwrap_or_default();
                    self.handle_forward_checkpoint_pass(cp.id, cp, active_cps.len(), &layout_id, network);
                    return;
                }
            }
        }

        // 2. Check if checkpoint belongs to an alternative layout (Dynamic Branch Detection)
        for other_layout in &network.layouts {
            if other_layout.id == self.active_layout_id {
                continue;
            }
            if let Some(pos) = other_layout.checkpoint_ids.iter().position(|&id| id == cp.id) {
                let valid_switch = if let Some(last_pos) = other_layout.checkpoint_ids.iter().position(|&id| id == self.last_checkpoint_id) {
                    pos > last_pos && (pos - last_pos) <= 3
                } else {
                    pos <= 2 || cp.is_joker
                };

                if valid_switch {
                    self.active_layout_id = other_layout.id.clone();
                    if other_layout.id.to_lowercase().contains("joker") || cp.is_joker {
                        self.is_joker_lap = true;
                    }
                    if let Some(seg_id) = cp.segment_id {
                        self.current_segment_id = seg_id;
                    }
                    if let Some(default_layout) = network.get_layout(&network.default_layout_id) {
                        self.is_in_branch = !default_layout.segment_sequence.contains(&self.current_segment_id);
                    }
                    self.handle_forward_checkpoint_pass(
                        cp.id,
                        cp,
                        other_layout.checkpoint_ids.len(),
                        &other_layout.id,
                        network,
                    );
                    return;
                }
            }
        }

        // 3. Fallback when active layout has no checkpoint list (e.g. legacy single spline)
        if active_cps.is_empty() {
            let total = all_cps.len();
            let is_expected = cp.id == self.next_checkpoint_index;
            let is_jump = cp.id > self.next_checkpoint_index && (cp.id - self.next_checkpoint_index) <= 3;
            let is_finish = cp.id == 0 && self.next_checkpoint_index >= total.saturating_sub(2);
            if is_expected || is_jump || is_finish {
                let layout_id = self.active_layout_id.clone();
                self.handle_forward_checkpoint_pass(cp.id, cp, total, &layout_id, network);
            }
        }
    }

    fn handle_forward_checkpoint_pass(
        &mut self,
        cp_id: usize,
        cp: &Checkpoint,
        total_cps: usize,
        _active_layout_id: &str,
        network: &TrackNetwork,
    ) {
        self.last_checkpoint_id = cp_id;
        self.checkpoints_passed_this_lap += 1;

        if let Some(layout) = network.active_or_default_layout(Some(&self.active_layout_id)) {
            if !layout.checkpoint_ids.is_empty() {
                if let Some(pos) = layout.checkpoint_ids.iter().position(|&id| id == cp_id) {
                    self.next_checkpoint_index = (pos + 1) % layout.checkpoint_ids.len();
                } else {
                    self.next_checkpoint_index = (self.next_checkpoint_index + 1) % layout.checkpoint_ids.len();
                }
            } else {
                self.next_checkpoint_index = (cp_id + 1) % total_cps.max(1);
            }
        } else {
            self.next_checkpoint_index = (cp_id + 1) % total_cps.max(1);
        }

        if cp.is_joker {
            self.is_joker_lap = true;
        }

        if let Some(seg_id) = cp.segment_id {
            self.current_segment_id = seg_id;
        }

        // Sector split tracking
        if cp.sector != self.current_sector {
            let completed_sector = self.current_sector;
            let time = self.sector_times[completed_sector];
            if self.best_sector_times[completed_sector].map_or(true, |b| time < b) {
                self.best_sector_times[completed_sector] = Some(time);
            }
            self.current_sector = cp.sector;
        }

        // Finish line crossing check
        if cp.is_finish_line {
            let min_required = (total_cps.max(1) * 7) / 10;
            if self.checkpoints_passed_this_lap >= min_required {
                let last_sec = self.current_sector;
                let sec_time = self.sector_times[last_sec];
                if self.best_sector_times[last_sec].map_or(true, |b| sec_time < b) {
                    self.best_sector_times[last_sec] = Some(sec_time);
                }

                let finished_lap_time = self.lap_time;
                self.last_lap_time = Some(finished_lap_time);
                if self.best_lap_time.map_or(true, |b| finished_lap_time < b) {
                    self.best_lap_time = Some(finished_lap_time);
                }

                if self.is_joker_lap {
                    self.joker_laps_completed += 1;
                }

                self.current_lap += 1;
                self.lap_time = 0.0;
                self.checkpoints_passed_this_lap = 0;
                self.lap_completed = true;
                self.is_joker_lap = false;
                self.is_in_branch = false;

                // Reset layout back to default for the next lap's strategic decision
                self.active_layout_id = network.default_layout_id.clone();
                self.next_checkpoint_index = 1 % total_cps.max(1);

                self.last_lap_sector_times = self.sector_times.clone();
                for s in self.sector_times.iter_mut() {
                    *s = 0.0;
                }
                self.current_sector = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wheelbase::CarConfig;

    #[test]
    fn test_checkpoint_crossing() {
        let cp = Checkpoint::new(
            0,
            LineSegment::new(Vec2::new(0.0, -10.0), Vec2::new(0.0, 10.0)),
            Vec2::new(1.0, 0.0),
            0,
            true,
        );

        let cross_fwd = cp.test_crossing(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0));
        assert_eq!(cross_fwd, Some(CheckpointCrossResult::Forward));

        let cross_bwd = cp.test_crossing(Vec2::new(1.0, 0.0), Vec2::new(-1.0, 0.0));
        assert_eq!(cross_bwd, Some(CheckpointCrossResult::Backward));

        let no_cross = cp.test_crossing(Vec2::new(5.0, 0.0), Vec2::new(10.0, 0.0));
        assert_eq!(no_cross, None);
    }

    #[test]
    fn test_lap_counting_and_sequence_enforcement() {
        let waypoints = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 100.0),
        ];
        let spline = TrackSpline::from_points(&waypoints, 12.0, true);

        let checkpoints = vec![
            Checkpoint::new(
                0,
                LineSegment::new(Vec2::new(0.0, -10.0), Vec2::new(0.0, 10.0)),
                Vec2::new(1.0, 0.0),
                0,
                true,
            ),
            Checkpoint::new(
                1,
                LineSegment::new(Vec2::new(90.0, 50.0), Vec2::new(110.0, 50.0)),
                Vec2::new(0.0, 1.0),
                1,
                false,
            ),
            Checkpoint::new(
                2,
                LineSegment::new(Vec2::new(50.0, 90.0), Vec2::new(50.0, 110.0)),
                Vec2::new(-1.0, 0.0),
                2,
                false,
            ),
            Checkpoint::new(
                3,
                LineSegment::new(Vec2::new(-10.0, 50.0), Vec2::new(10.0, 50.0)),
                Vec2::new(0.0, -1.0),
                2,
                false,
            ),
        ];

        let mut tracker = TrackProgressTracker::new(checkpoints.len(), 3);
        let mut car = Car::new(CarConfig::sports_car());

        // Step 1: Start just before CP0
        car.state.position = Vec2::new(-2.0, 0.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);

        // Step 2: Cross CP0 (start line)
        car.state.position = Vec2::new(2.0, 0.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);
        assert_eq!(tracker.last_checkpoint_idx, 0);
        assert_eq!(tracker.next_checkpoint_idx, 1);

        // Cheating attempt: directly cross finish line again without crossing CPs 1, 2, 3
        car.state.position = Vec2::new(-2.0, 0.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);
        car.state.position = Vec2::new(2.0, 0.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);
        // Lap should not complete because checkpoints passed < required threshold
        assert_eq!(tracker.current_lap, 1);
        assert!(!tracker.lap_completed);

        // Proper sequence: Cross CP1
        car.state.position = Vec2::new(100.0, 48.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);
        car.state.position = Vec2::new(100.0, 52.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);
        assert_eq!(tracker.last_checkpoint_idx, 1);
        assert_eq!(tracker.next_checkpoint_idx, 2);

        // Cross CP2
        car.state.position = Vec2::new(52.0, 100.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);
        car.state.position = Vec2::new(48.0, 100.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);
        assert_eq!(tracker.last_checkpoint_idx, 2);
        assert_eq!(tracker.next_checkpoint_idx, 3);

        // Cross CP3
        car.state.position = Vec2::new(0.0, 52.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);
        car.state.position = Vec2::new(0.0, 48.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);
        assert_eq!(tracker.last_checkpoint_idx, 3);
        assert_eq!(tracker.next_checkpoint_idx, 0);

        // Cross Finish line CP0
        car.state.position = Vec2::new(-2.0, 0.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);
        car.state.position = Vec2::new(2.0, 0.0);
        tracker.update(&car, &spline, &checkpoints, 0.016);

        assert!(tracker.lap_completed);
        assert_eq!(tracker.current_lap, 2);
        assert!(tracker.best_lap_time.is_some());
    }

    #[test]
    fn test_multi_route_dynamic_joker_lap_and_wrong_way() {
        use crate::track::network::{RoadSegment, SegmentId, TrackLayout, TrackNetwork};
        use crate::track::spline::TrackWaypoint;

        // Construct a network with:
        // Seg 0: (0,0) -> (50,0) (Trunk entry)
        // Seg 1: (50,0) -> (100, -20) -> (150, 0) (Main branch)
        // Seg 2: (50,0) -> (100, 30) -> (150, 0) (Joker branch)
        // Seg 3: (150,0) -> (200, 0) -> (200, -50) -> (0, -50) -> (0, 0) (Common return)
        let seg0 = RoadSegment::new(
            SegmentId(0),
            "Trunk Entry",
            vec![
                TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
                TrackWaypoint::new(Vec2::new(50.0, 0.0), 12.0),
            ],
        );

        let seg1 = RoadSegment::new(
            SegmentId(1),
            "Main Branch",
            vec![
                TrackWaypoint::new(Vec2::new(50.0, 0.0), 12.0),
                TrackWaypoint::new(Vec2::new(100.0, -20.0), 12.0),
                TrackWaypoint::new(Vec2::new(150.0, 0.0), 12.0),
            ],
        );

        let seg2 = RoadSegment::new(
            SegmentId(2),
            "Joker Branch",
            vec![
                TrackWaypoint::new(Vec2::new(50.0, 0.0), 12.0),
                TrackWaypoint::new(Vec2::new(100.0, 30.0), 12.0),
                TrackWaypoint::new(Vec2::new(150.0, 0.0), 12.0),
            ],
        );

        let seg3 = RoadSegment::new(
            SegmentId(3),
            "Common Return",
            vec![
                TrackWaypoint::new(Vec2::new(150.0, 0.0), 12.0),
                TrackWaypoint::new(Vec2::new(200.0, 0.0), 12.0),
                TrackWaypoint::new(Vec2::new(200.0, -50.0), 12.0),
                TrackWaypoint::new(Vec2::new(0.0, -50.0), 12.0),
                TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
            ],
        );

        let layout_main = TrackLayout::new(
            "main",
            "Main GP",
            vec![SegmentId(0), SegmentId(1), SegmentId(3)],
            SegmentId(0),
        )
        .with_checkpoints(vec![0, 1, 2, 4]);

        let layout_joker = TrackLayout::new(
            "joker",
            "Joker Lap",
            vec![SegmentId(0), SegmentId(2), SegmentId(3)],
            SegmentId(0),
        )
        .with_checkpoints(vec![0, 1, 3, 4]);

        let network = TrackNetwork {
            junctions: Vec::new(),
            segments: vec![seg0, seg1, seg2, seg3],
            layouts: vec![layout_main, layout_joker],
            default_layout_id: "main".to_string(),
        };

        // Checkpoints:
        // CP 0: Finish line at (0, 0)
        // CP 1: Pre-split at (30, 0)
        // CP 2: Main branch at (100, -20)
        // CP 3: Joker branch at (100, 30) (is_joker: true)
        // CP 4: Post-merge at (180, 0)
        let checkpoints = vec![
            Checkpoint::new(
                0,
                LineSegment::new(Vec2::new(0.0, -10.0), Vec2::new(0.0, 10.0)),
                Vec2::new(1.0, 0.0),
                0,
                true,
            ),
            Checkpoint::new(
                1,
                LineSegment::new(Vec2::new(30.0, -10.0), Vec2::new(30.0, 10.0)),
                Vec2::new(1.0, 0.0),
                0,
                false,
            ),
            Checkpoint::new(
                2,
                LineSegment::new(Vec2::new(100.0, -30.0), Vec2::new(100.0, -10.0)),
                Vec2::new(1.0, 0.0),
                1,
                false,
            )
            .with_segment(SegmentId(1)),
            Checkpoint::new(
                3,
                LineSegment::new(Vec2::new(100.0, 20.0), Vec2::new(100.0, 40.0)),
                Vec2::new(1.0, 0.0),
                1,
                false,
            )
            .with_segment(SegmentId(2))
            .with_joker(true),
            Checkpoint::new(
                4,
                LineSegment::new(Vec2::new(180.0, -10.0), Vec2::new(180.0, 10.0)),
                Vec2::new(1.0, 0.0),
                2,
                false,
            )
            .with_segment(SegmentId(3)),
        ];

        let mut tracker = MultiRouteProgressTracker::from_network(&network, None, 3);
        let mut car = Car::new(CarConfig::sports_car());

        // Step 1: Start just before CP0
        car.state.position = Vec2::new(-2.0, 0.0);
        tracker.update(&car, &network, &checkpoints, 0.016);

        // Step 2: Cross CP0 (start line)
        car.state.position = Vec2::new(2.0, 0.0);
        tracker.update(&car, &network, &checkpoints, 0.016);
        assert_eq!(tracker.last_checkpoint_id, 0);
        assert_eq!(tracker.next_checkpoint_index, 1);
        assert_eq!(tracker.active_layout_id, "main");

        // Step 3: Cross CP1 (pre-split)
        car.state.position = Vec2::new(28.0, 0.0);
        tracker.update(&car, &network, &checkpoints, 0.016);
        car.state.position = Vec2::new(32.0, 0.0);
        tracker.update(&car, &network, &checkpoints, 0.016);
        assert_eq!(tracker.last_checkpoint_id, 1);
        assert_eq!(tracker.next_checkpoint_index, 2);

        // Step 4: Car steers into the JOKER branch and crosses CP 3!
        car.state.position = Vec2::new(98.0, 30.0);
        tracker.update(&car, &network, &checkpoints, 0.016);
        car.state.position = Vec2::new(102.0, 30.0);
        tracker.update(&car, &network, &checkpoints, 0.016);

        // Dynamic Branch Detection Verification:
        assert_eq!(tracker.last_checkpoint_id, 3);
        assert_eq!(tracker.active_layout_id, "joker");
        assert!(tracker.is_joker_lap);
        assert!(tracker.is_in_branch);
        assert_eq!(tracker.current_segment_id, SegmentId(2));

        // Step 5: Test wrong-way detection on the Joker branch
        // Turn car 180 degrees backwards
        car.state.angle = std::f32::consts::PI;
        tracker.update(&car, &network, &checkpoints, 0.016);
        assert!(tracker.is_wrong_way);
        assert!(tracker.wrong_way_timer > 0.0);

        // Restore forward heading
        car.state.angle = 0.0;
        tracker.update(&car, &network, &checkpoints, 0.016);
        assert!(!tracker.is_wrong_way);

        // Step 6: Cross CP4 (post-merge on Segment 3)
        car.state.position = Vec2::new(178.0, 0.0);
        tracker.update(&car, &network, &checkpoints, 0.016);
        car.state.position = Vec2::new(182.0, 0.0);
        tracker.update(&car, &network, &checkpoints, 0.016);
        assert_eq!(tracker.last_checkpoint_id, 4);
        assert_eq!(tracker.next_checkpoint_index, 0); // expecting finish line next
        assert!(!tracker.is_in_branch); // merged back onto common trunk

        // Step 7: Drive around return loop and cross Finish Line CP0
        car.state.position = Vec2::new(-2.0, 0.0);
        tracker.update(&car, &network, &checkpoints, 0.016);
        car.state.position = Vec2::new(2.0, 0.0);
        tracker.update(&car, &network, &checkpoints, 0.016);

        // Lap completion & Joker count verification:
        assert!(tracker.lap_completed);
        assert_eq!(tracker.current_lap, 2);
        assert_eq!(tracker.joker_laps_completed, 1);
        assert!(!tracker.is_joker_lap); // reset for fresh lap 2
        assert_eq!(tracker.active_layout_id, "main"); // reset to default for next decision
    }
}
