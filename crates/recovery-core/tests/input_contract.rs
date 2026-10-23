use recovery_core::{
    ButtonEdges, Control, GameStatus, InputFrame, RecoveredCore, TickError, game_status,
};

#[test]
fn held_button_has_one_press_edge_then_no_repeat() {
    let neutral = InputFrame::default();
    let held = neutral.with_down(Control::Confirm, true);

    assert!(ButtonEdges::between(neutral, held).just_pressed(Control::Confirm));
    assert!(!ButtonEdges::between(held, held).just_pressed(Control::Confirm));
    assert!(!ButtonEdges::between(held, held).just_released(Control::Confirm));
}

#[test]
fn release_edge_is_separate_from_press_edge() {
    let held = InputFrame::default().with_down(Control::Cancel, true);
    let neutral = InputFrame::default();
    let edges = ButtonEdges::between(held, neutral);

    assert!(!edges.just_pressed(Control::Cancel));
    assert!(edges.just_released(Control::Cancel));
}

#[test]
fn signed_axes_preserve_extremes_and_game_status_stays_unknown() {
    let frame = InputFrame::new(0, i16::MIN, i16::MAX);
    assert_eq!((frame.axis_x, frame.axis_y), (i16::MIN, i16::MAX));
    assert_eq!(game_status(), GameStatus::Unknown);
}

#[test]
fn fixed_tick_seam_reports_not_recovered() {
    let mut core = RecoveredCore;
    assert_eq!(
        core.tick(InputFrame::default(), ButtonEdges::default()),
        Err(TickError::NotRecovered)
    );
}
