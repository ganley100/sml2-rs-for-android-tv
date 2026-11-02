use super::*;
use crate::input::Bindings;

fn device(id: u64, model: &str) -> Device {
    Device {
        id,
        name: model.into(),
        preference: preference(model, "gamepad"),
        snapshot: Snapshot::default(),
    }
}

fn pressed(button: PadButton) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.buttons[BUTTONS.iter().position(|b| *b == button).unwrap()] = true;
    snapshot
}

#[test]
fn every_button_reaches_its_existing_binding_name() {
    // Independent expected positional mapping, including menu and analog triggers.
    let expected = [
        PadButton::South,
        PadButton::East,
        PadButton::West,
        PadButton::North,
        PadButton::Up,
        PadButton::Down,
        PadButton::Left,
        PadButton::Right,
        PadButton::Start,
        PadButton::Select,
        PadButton::LeftThumb,
        PadButton::RightThumb,
        PadButton::LeftShoulder,
        PadButton::RightShoulder,
        PadButton::LeftTrigger,
        PadButton::RightTrigger,
    ];
    for (index, button) in expected.into_iter().enumerate() {
        let mut state = NativeState::default();
        let mut input = InputState::default();
        let mut d = device(1, "PS5");
        d.snapshot.buttons[index] = true;
        state.poll(&[d], [], &mut input, None, 0.3);
        assert!(input.take_press(&Button::Pad(button)), "{button:?}");
    }
}

#[test]
fn short_press_release_between_frames_is_not_lost_or_stuck() {
    let mut state = NativeState::default();
    let mut input = InputState::default();
    state.poll(
        &[device(1, "PS5")],
        [(1, pressed(PadButton::South)), (1, Snapshot::default())],
        &mut input,
        None,
        0.3,
    );
    assert!(input.sample_player(&Bindings::default()).a);
    assert!(!input.sample_player(&Bindings::default()).a);
}

#[test]
fn apple_axes_use_up_positive_and_existing_hysteresis() {
    for (left, expected) in [
        ([-1., 0.], PadButton::StickLeft),
        ([1., 0.], PadButton::StickRight),
        ([0., 1.], PadButton::StickUp),
        ([0., -1.], PadButton::StickDown),
    ] {
        let mut state = NativeState::default();
        let mut input = InputState::default();
        let mut d = device(1, "PS5");
        d.snapshot.left = left;
        state.poll(&[d], [], &mut input, None, 0.3);
        assert!(input.take_press(&Button::Pad(expected)));
    }
    let mut state = NativeState::default();
    let mut input = InputState::default();
    let mut devices = [device(1, "PS5")];
    for (value, right) in [
        (0.29, false),
        (0.31, true),
        (0.25, true),
        (0.20, false),
        (f32::NAN, false),
    ] {
        devices[0].snapshot.left[0] = value;
        state.poll(&devices, [], &mut input, None, 0.3);
        assert_eq!(input.sample_player(&Bindings::default()).right, right);
    }
}

#[test]
fn stick_taps_between_frames_are_preserved() {
    let mut state = NativeState::default();
    let mut input = InputState::default();
    state.poll(
        &[device(1, "PS5")],
        [
            (
                1,
                Snapshot {
                    left: [0., 1.],
                    ..Snapshot::default()
                },
            ),
            (1, Snapshot::default()),
        ],
        &mut input,
        None,
        0.3,
    );
    assert!(input.sample_player(&Bindings::default()).up);
    assert!(!input.sample_player(&Bindings::default()).up);
}

#[test]
fn identical_controllers_keep_explicit_selection_after_cycling() {
    let mut state = NativeState::default();
    let mut input = InputState::default();
    let devices = [device(1, "PS5"), device(2, "PS5")];
    state.poll(&devices, [], &mut input, None, 0.3);
    let saved = state.cycle(&devices, &mut input).unwrap();
    for _ in 0..3 {
        assert!(!state.poll(
            &devices,
            [
                (1, pressed(PadButton::West)),
                (2, pressed(PadButton::South))
            ],
            &mut input,
            Some(&saved),
            0.3
        ));
        let keys = input.sample_player(&Bindings::default());
        assert!(keys.a);
        assert!(!keys.b);
    }
    assert_eq!(state.selected, Some(2));
}

#[test]
fn different_model_cycling_and_preferred_arrival_work() {
    let mut state = NativeState::default();
    let mut input = InputState::default();
    let devices = [device(1, "PS5"), device(2, "Xbox")];
    state.poll(&devices, [], &mut input, None, 0.3);
    let saved = state.cycle(&devices, &mut input).unwrap();
    state.poll(&devices, [], &mut input, Some(&saved), 0.3);
    assert_eq!(state.selected, Some(2));
    state.poll(&devices[..1], [], &mut input, Some(&saved), 0.3);
    assert_eq!(state.selected, Some(1));
    assert!(state.poll(&devices, [], &mut input, Some(&saved), 0.3));
    assert_eq!(state.selected, Some(2));
}

#[test]
fn legacy_preferences_fall_back_without_resetting_bindings() {
    let mut state = NativeState::default();
    let mut input = InputState::default();
    let devices = [device(1, "PS5")];
    assert!(!state.poll(&devices, [], &mut input, Some("030000004c050000"), 0.3));
    assert_eq!(state.name(&devices), "PS5");
    assert_eq!(
        state.cycle(&devices, &mut input),
        Some(preference("PS5", "gamepad"))
    );
    assert_ne!(preference("a:b", "c"), preference("a", "b:c"));
}

#[test]
fn disconnect_clears_pad_and_hysteresis_but_preserves_keyboard() {
    let mut state = NativeState::default();
    let mut input = InputState::default();
    let mut old = device(1, "PS5");
    old.snapshot = pressed(PadButton::South);
    old.snapshot.left = [1., 0.];
    state.poll(&[old], [], &mut input, None, 0.3);
    input.event(Button::key("ArrowLeft"), true);
    assert!(state.poll(&[], [], &mut input, None, 0.3));
    let keys = input.sample_player(&Bindings::default());
    assert!(keys.left);
    assert!(!keys.a && !keys.right);
    input.event(Button::key("ArrowLeft"), false);
    let mut new = device(2, "PS5");
    new.snapshot.left = [0.25, 0.];
    assert!(!state.poll(
        &[new],
        [(1, pressed(PadButton::South))],
        &mut input,
        None,
        0.3
    ));
    let keys = input.sample_player(&Bindings::default());
    assert!(!keys.a && !keys.right);
}

#[test]
fn usb_to_bluetooth_in_one_frame_rejects_old_events_and_accepts_new() {
    let mut state = NativeState::default();
    let mut input = InputState::default();
    state.poll(&[device(1, "PS5")], [], &mut input, None, 0.3);
    assert!(state.poll(
        &[device(2, "PS5")],
        [
            (1, pressed(PadButton::South)),
            (2, pressed(PadButton::West))
        ],
        &mut input,
        None,
        0.3
    ));
    let keys = input.sample_player(&Bindings::default());
    assert!(!keys.a && keys.b);
}

#[test]
fn focus_suppression_requires_release_before_next_press() {
    let mut state = NativeState::default();
    let mut input = InputState::default();
    let mut devices = [device(1, "PS5")];
    devices[0].snapshot = pressed(PadButton::South);
    state.poll(&devices, [], &mut input, None, 0.3);
    input.suppress_until_release();
    state.poll(&devices, [], &mut input, None, 0.3);
    assert!(!input.sample_player(&Bindings::default()).a);
    devices[0].snapshot = Snapshot::default();
    state.poll(&devices, [], &mut input, None, 0.3);
    devices[0].snapshot = pressed(PadButton::South);
    state.poll(&devices, [], &mut input, None, 0.3);
    assert!(input.sample_player(&Bindings::default()).a);
}
