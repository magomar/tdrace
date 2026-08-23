use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::geometry::LineSegment;
use super::spline::TrackSpline;
use crate::physics::car::Car;

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
        }
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
                            } else if (idx + checkpoints.len() - self.next_checkpoint_idx) % checkpoints.len() <= 2 {
                                // Close enough in sequence (in case of fast motion across adjacent gates)
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

                // Reset current lap sector times
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
    use crate::physics::config::CarConfig;

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
}
