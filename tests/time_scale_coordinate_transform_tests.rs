use chart_rs::ChartError;
use chart_rs::core::TimeIndexCoordinateSpace;

#[test]
fn index_to_coordinate_and_back_roundtrip_matches_lwc_formula() {
    let space = TimeIndexCoordinateSpace {
        base_index: 200.0,
        right_offset_bars: 2.5,
        bar_spacing_px: 6.0,
        width_px: 1000.0,
    };

    let logical_index = 194.25;
    let x = space
        .index_to_coordinate(logical_index)
        .expect("index to coordinate");
    let recovered = space
        .coordinate_to_logical_index(x)
        .expect("coordinate to logical");

    assert!((recovered - logical_index).abs() <= 1e-12);
}

#[test]
fn coordinate_to_index_ceil_uses_ceil_semantics() {
    let space = TimeIndexCoordinateSpace {
        base_index: 50.0,
        right_offset_bars: 0.0,
        bar_spacing_px: 8.0,
        width_px: 800.0,
    };

    let x = space
        .index_to_coordinate(12.2)
        .expect("index to coordinate");
    let discrete = space
        .coordinate_to_index_ceil(x)
        .expect("coordinate to discrete index");
    assert_eq!(discrete, 13);
}

#[test]
fn pan_right_offset_by_pixels_matches_delta_bars() {
    let space = TimeIndexCoordinateSpace {
        base_index: 120.0,
        right_offset_bars: 1.25,
        bar_spacing_px: 5.0,
        width_px: 900.0,
    };

    let updated = space
        .pan_right_offset_by_pixels(20.0)
        .expect("pan right offset");
    assert!((updated - 5.25).abs() <= 1e-12);
}

#[test]
fn anchor_preserving_zoom_solver_keeps_anchor_coordinate_stable() {
    let old_space = TimeIndexCoordinateSpace {
        base_index: 300.0,
        right_offset_bars: -1.5,
        bar_spacing_px: 7.0,
        width_px: 1200.0,
    };
    let anchor_index = 287.75;
    let anchor_x_before = old_space
        .index_to_coordinate(anchor_index)
        .expect("old anchor coordinate");

    let new_space = TimeIndexCoordinateSpace {
        bar_spacing_px: 11.0,
        ..old_space
    };
    let new_right_offset = new_space
        .solve_right_offset_for_anchor_preserving_zoom(
            old_space.bar_spacing_px,
            old_space.right_offset_bars,
            anchor_index,
        )
        .expect("solve right offset");

    let solved_space = TimeIndexCoordinateSpace {
        right_offset_bars: new_right_offset,
        ..new_space
    };
    let anchor_x_after = solved_space
        .index_to_coordinate(anchor_index)
        .expect("new anchor coordinate");

    assert!((anchor_x_after - anchor_x_before).abs() <= 1e-9);
}

#[test]
fn invalid_spacing_is_rejected() {
    let space = TimeIndexCoordinateSpace {
        base_index: 0.0,
        right_offset_bars: 0.0,
        bar_spacing_px: 0.0,
        width_px: 1000.0,
    };

    let err = space
        .index_to_coordinate(0.0)
        .expect_err("zero spacing must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn nearest_filled_slot_prefers_closest_sparse_index() {
    let space = TimeIndexCoordinateSpace {
        base_index: 0.0,
        right_offset_bars: 0.0,
        bar_spacing_px: 10.0,
        width_px: 1000.0,
    };
    let sparse = [0.0, 5.0, 10.0, 20.0];
    let x = space.index_to_coordinate(8.9).expect("index to coordinate");

    let slot = space
        .coordinate_to_nearest_filled_slot(x, sparse.len(), |idx| sparse[idx])
        .expect("nearest sparse slot")
        .expect("slot present");
    assert_eq!(slot, 2);
}

#[test]
fn nearest_filled_slot_uses_upper_slot_on_equal_distance() {
    let space = TimeIndexCoordinateSpace {
        base_index: 0.0,
        right_offset_bars: 0.0,
        bar_spacing_px: 10.0,
        width_px: 1000.0,
    };
    let sparse = [10.0, 14.0];
    let x = space
        .index_to_coordinate(12.0)
        .expect("index to coordinate");

    let slot = space
        .coordinate_to_nearest_filled_slot(x, sparse.len(), |idx| sparse[idx])
        .expect("nearest sparse slot")
        .expect("slot present");
    assert_eq!(slot, 1);
}
