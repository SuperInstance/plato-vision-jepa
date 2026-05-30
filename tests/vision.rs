use plato_vision_jepa::*;

#[test]
fn test_compute_frame_diff_identical() {
    let h = [128u8; 256];
    let diff = compute_frame_diff(&h, &h);
    assert!(diff < 1e-10, "identical histograms should have ~0 diff, got {diff}");
}

#[test]
fn test_compute_frame_diff_maximal() {
    let mut h1 = [0u8; 256];
    let mut h2 = [0u8; 256];
    h1[0] = 255;
    h2[255] = 255;
    let diff = compute_frame_diff(&h1, &h2);
    assert!(diff > 0.9, "maximally different histograms should be near 1.0, got {diff}");
}

#[test]
fn test_compute_frame_diff_symmetric() {
    let mut h1 = [0u8; 256];
    let mut h2 = [0u8; 256];
    h1[0] = 200;
    h1[1] = 50;
    h2[0] = 50;
    h2[1] = 200;
    let diff12 = compute_frame_diff(&h1, &h2);
    let diff21 = compute_frame_diff(&h2, &h1);
    assert!((diff12 - diff21).abs() < 1e-10, "should be symmetric");
}

#[test]
fn test_is_significant_change() {
    assert!(!is_significant_change(0.01, 0.05));
    assert!(is_significant_change(0.10, 0.05));
    assert!(!is_significant_change(0.05, 0.05));
}

#[test]
fn test_extract_region_states_empty() {
    let grid: Vec<Vec<u8>> = vec![];
    let states = extract_region_states(&grid);
    assert_eq!(states, [0.0; 4]);
}

#[test]
fn test_extract_region_states_uniform() {
    let grid = vec![vec![128u8; 4]; 4];
    let states = extract_region_states(&grid);
    for &s in &states {
        let expected = 128.0 / 255.0;
        assert!((s - expected).abs() < 0.01, "expected {expected}, got {s}");
    }
}

#[test]
fn test_extract_region_states_asymmetric() {
    let grid = vec![vec![255, 0], vec![0, 0]];
    let states = extract_region_states(&grid);
    assert!((states[0] - 1.0).abs() < 0.01, "TL should be ~1.0");
    assert!(states[1] < 0.01, "TR should be ~0.0");
    assert!(states[2] < 0.01, "BL should be ~0.0");
    assert!(states[3] < 0.01, "BR should be ~0.0");
}

#[test]
fn test_compute_motion_vector_empty() {
    assert_eq!(compute_motion_vector(&[], &[]), (0.0, 0.0));
    assert_eq!(compute_motion_vector(&[(1.0, 2.0)], &[]), (0.0, 0.0));
}

#[test]
fn test_compute_motion_vector_basic() {
    let prev = [(0.0, 0.0), (1.0, 1.0)];
    let curr = [(1.0, 0.0), (2.0, 1.0)];
    let (dx, dy) = compute_motion_vector(&prev, &curr);
    assert!((dx - 1.0).abs() < 1e-6);
    assert!((dy - 0.0).abs() < 1e-6);
}

#[test]
fn test_compute_motion_vector_uneven_lengths() {
    let prev = [(0.0, 0.0)];
    let curr = [(2.0, 3.0), (4.0, 5.0)];
    let (dx, dy) = compute_motion_vector(&prev, &curr);
    assert!((dx - 2.0).abs() < 1e-6);
    assert!((dy - 3.0).abs() < 1e-6);
}

#[test]
fn test_vision_state_roundtrip() {
    let state = RoomVisionState {
        brightness: 0.7,
        motion_level: 0.3,
        occupancy: 2.0,
        anomaly_score: 0.1,
        region_states: [0.1, 0.2, 0.3, 0.4],
        temporal_patterns: [0.5, 0.6, 0.7, 0.8],
        reserved: [0.0; 4],
    };
    let v = state.to_vector();
    let restored = RoomVisionState::from_vector(&v);
    assert!((restored.brightness - 0.7).abs() < 1e-6);
    assert!((restored.motion_level - 0.3).abs() < 1e-6);
    assert!((restored.occupancy - 2.0).abs() < 1e-6);
    assert_eq!(restored.region_states, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(restored.temporal_patterns, [0.5, 0.6, 0.7, 0.8]);
}

#[test]
fn test_vision_state_to_tile() {
    let state = RoomVisionState {
        brightness: 0.5,
        motion_level: 0.2,
        occupancy: 3.0,
        anomaly_score: 0.8,
        region_states: [0.0; 4],
        temporal_patterns: [0.0; 4],
        reserved: [0.0; 4],
    };
    let tile = vision_state_to_tile(&state);
    assert!((tile.brightness - 0.5).abs() < 1e-6);
    assert!((tile.motion_level - 0.2).abs() < 1e-6);
    assert_eq!(tile.object_count, 3);
    assert_eq!(tile.anomalies_detected, 1);
}

#[test]
fn test_deadband_first_frame() {
    let mut db = VisionDeadband::new(0.05);
    let h = [128u8; 256];
    assert!(db.should_process(&h), "first frame should always process");
}

#[test]
fn test_deadband_no_change() {
    let mut db = VisionDeadband::new(0.05);
    let h = [128u8; 256];
    assert!(db.should_process(&h));
    assert!(!db.should_process(&h), "identical frame should not reprocess");
}

#[test]
fn test_deadband_with_change() {
    let mut db = VisionDeadband::new(0.05);
    let h1 = [128u8; 256];
    let h2 = [0u8; 256];
    // Completely different histogram — all zeros vs all 128s
    assert!(db.should_process(&h1));
    assert!(db.should_process(&h2), "changed frame should process");
}
