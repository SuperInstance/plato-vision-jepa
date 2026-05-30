//! # plato-vision-jepa
//!
//! Vision JEPA for the PLATO nervous system. Processes camera frames into
//! structured room state vectors suitable for downstream nervous-system tiles.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Structured room state produced by the vision JEPA from a single frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionTile {
    pub id: Uuid,
    pub brightness: f32,
    pub occupancy: f32,
    pub motion_level: f32,
    pub object_count: u32,
    pub anomalies_detected: u32,
    pub timestamp: u64,
}

/// Deadband filter — only pass frames whose histogram diff exceeds a threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionDeadband {
    pub threshold: f64,
    pub last_histogram: Option<Vec<u8>>,
}

impl Default for VisionDeadband {
    fn default() -> Self {
        Self {
            threshold: 0.05,
            last_histogram: None,
        }
    }
}

impl VisionDeadband {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            last_histogram: None,
        }
    }

    /// Returns `true` if the new histogram represents a significant change.
    pub fn should_process(&mut self, histogram: &[u8; 256]) -> bool {
        let significant = match self.last_histogram {
            None => true,
            Some(ref prev) => {
                let prev_arr: [u8; 256] = prev.clone().try_into().unwrap_or([0u8; 256]);
                is_significant_change(compute_frame_diff(&prev_arr, histogram), self.threshold)
            }
        };
        if significant {
            self.last_histogram = Some(histogram.to_vec());
        }
        significant
    }
}

/// 16-dimensional vision state vector for a room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomVisionState {
    pub brightness: f32,
    pub motion_level: f32,
    pub occupancy: f32,
    pub anomaly_score: f32,
    pub region_states: [f32; 4],
    pub temporal_patterns: [f32; 4],
    pub reserved: [f32; 4],
}

impl Default for RoomVisionState {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            motion_level: 0.0,
            occupancy: 0.0,
            anomaly_score: 0.0,
            region_states: [0.0; 4],
            temporal_patterns: [0.0; 4],
            reserved: [0.0; 4],
        }
    }
}

impl RoomVisionState {
    pub fn to_vector(&self) -> [f32; 16] {
        let mut v = [0.0f32; 16];
        v[0] = self.brightness;
        v[1] = self.motion_level;
        v[2] = self.occupancy;
        v[3] = self.anomaly_score;
        v[4..8].copy_from_slice(&self.region_states);
        v[8..12].copy_from_slice(&self.temporal_patterns);
        v[12..16].copy_from_slice(&self.reserved);
        v
    }

    pub fn from_vector(v: &[f32; 16]) -> Self {
        let mut region = [0.0f32; 4];
        let mut temporal = [0.0f32; 4];
        let mut reserved = [0.0f32; 4];
        region.copy_from_slice(&v[4..8]);
        temporal.copy_from_slice(&v[8..12]);
        reserved.copy_from_slice(&v[12..16]);
        Self {
            brightness: v[0],
            motion_level: v[1],
            occupancy: v[2],
            anomaly_score: v[3],
            region_states: region,
            temporal_patterns: temporal,
            reserved,
        }
    }
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Compute histogram intersection distance between two frames.
/// Returns a value in [0, 1] where 0 = identical, 1 = maximally different.
pub fn compute_frame_diff(prev_histogram: &[u8; 256], curr_histogram: &[u8; 256]) -> f64 {
    let mut intersection: f64 = 0.0;
    let mut prev_total: f64 = 0.0;
    let mut curr_total: f64 = 0.0;

    for i in 0..256 {
        let p = prev_histogram[i] as f64;
        let c = curr_histogram[i] as f64;
        intersection += p.min(c);
        prev_total += p;
        curr_total += c;
    }

    let total = prev_total.max(curr_total);
    if total == 0.0 {
        return 0.0;
    }

    1.0 - (intersection / total)
}

/// Determine whether a frame diff exceeds the significance threshold.
pub fn is_significant_change(diff: f64, threshold: f64) -> bool {
    diff > threshold
}

/// Divide an image grid into 4 quadrants and return average intensity of each
/// normalized to [0, 1].
pub fn extract_region_states(grid: &[Vec<u8>]) -> [f32; 4] {
    if grid.is_empty() {
        return [0.0; 4];
    }

    let rows = grid.len();
    let mid_row = rows / 2;
    let mut sums = [0.0f64; 4];
    let mut counts = [0usize; 4];

    for (r, row) in grid.iter().enumerate() {
        let cols = row.len();
        if cols == 0 {
            continue;
        }
        let mid_col = cols / 2;
        for (c, &val) in row.iter().enumerate() {
            let quadrant = match (r < mid_row, c < mid_col) {
                (true, true) => 0,
                (true, false) => 1,
                (false, true) => 2,
                (false, false) => 3,
            };
            sums[quadrant] += val as f64;
            counts[quadrant] += 1;
        }
    }

    let mut result = [0.0f32; 4];
    for i in 0..4 {
        if counts[i] > 0 {
            result[i] = (sums[i] / counts[i] as f64 / 255.0) as f32;
        }
    }
    result
}

/// Compute average motion vector from tracked point positions.
pub fn compute_motion_vector(
    prev_positions: &[(f32, f32)],
    curr_positions: &[(f32, f32)],
) -> (f32, f32) {
    if prev_positions.is_empty() || curr_positions.is_empty() {
        return (0.0, 0.0);
    }

    let n = prev_positions.len().min(curr_positions.len());
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;

    for i in 0..n {
        dx += curr_positions[i].0 - prev_positions[i].0;
        dy += curr_positions[i].1 - prev_positions[i].1;
    }

    (dx / n as f32, dy / n as f32)
}

/// Convert a RoomVisionState into a VisionTile.
pub fn vision_state_to_tile(state: &RoomVisionState) -> VisionTile {
    VisionTile {
        id: Uuid::new_v4(),
        brightness: state.brightness,
        occupancy: state.occupancy,
        motion_level: state.motion_level,
        object_count: state.occupancy.round() as u32,
        anomalies_detected: if state.anomaly_score > 0.5 { 1 } else { 0 },
        timestamp: 0,
    }
}
