use super::*;
use crate::input::{Button, PadButton};

#[test]
fn framework_snapshot_maps_physical_buttons_and_axes() {
    objc2::rc::autoreleasepool(|_| {
        // SAFETY: Apple's writable snapshot controllers need no paired hardware.
        // All objects are created, used, and released on this test thread.
        unsafe {
            let controller = GCController::controllerWithExtendedGamepad();
            let pad = controller.extendedGamepad().unwrap();
            let expected = [
                Some(pad.buttonA()),
                Some(pad.buttonB()),
                Some(pad.buttonX()),
                Some(pad.buttonY()),
                Some(pad.dpad().up()),
                Some(pad.dpad().down()),
                Some(pad.dpad().left()),
                Some(pad.dpad().right()),
                Some(pad.buttonMenu()),
                pad.buttonOptions(),
                pad.leftThumbstickButton(),
                pad.rightThumbstickButton(),
                Some(pad.leftShoulder()),
                Some(pad.rightShoulder()),
                Some(pad.leftTrigger()),
                Some(pad.rightTrigger()),
            ];
            for (index, button) in expected.into_iter().enumerate() {
                if let Some(button) = button {
                    button.setValue(1.);
                    let mut wanted = [false; 16];
                    wanted[index] = true;
                    assert_eq!(snapshot(&pad).buttons, wanted, "input {index}");
                    button.setValue(0.);
                } else {
                    assert!(!snapshot(&pad).buttons[index]);
                }
            }
            pad.leftThumbstick().setValueForXAxis_yAxis(-1., 1.);
            assert_eq!(snapshot(&pad).left, [-1., 1.]);
        }
    });
}

#[test]
fn callback_values_preserve_taps_even_when_live_profile_is_already_released() {
    objc2::rc::autoreleasepool(|_| {
        // SAFETY: Snapshot controller with owned handlers, invoked synchronously using
        // their documented argument ABI. No device or main-runloop timing is required.
        unsafe {
            let controller = GCController::controllerWithExtendedGamepad();
            let pad = controller.extendedGamepad().unwrap();
            let (tx, rx) = mpsc::channel();
            bind_inputs(&pad, 7, &tx);
            let button = pad.buttonA();
            let callback = &*button.pressedChangedHandler();
            callback.call((NonNull::from(&*button), 1., Bool::YES));
            callback.call((NonNull::from(&*button), 0., Bool::NO));
            let axis = pad.leftThumbstick().yAxis();
            let callback = &*axis.valueChangedHandler();
            callback.call((NonNull::from(&*axis), 1.));
            callback.call((NonNull::from(&*axis), 0.));
            let events: Vec<_> = rx
                .try_iter()
                .map(|message| match message {
                    Message::Input(id, snapshot) => (id, snapshot),
                    _ => panic!("unexpected lifecycle message"),
                })
                .collect();
            assert_eq!(events.len(), 4);
            let mut state = NativeState::default();
            let mut input = InputState::default();
            state.poll(
                &[Device {
                    id: 7,
                    name: "snapshot".into(),
                    preference: "snapshot".into(),
                    snapshot: snapshot(&pad),
                }],
                events,
                &mut input,
                None,
                0.3,
            );
            assert!(input.take_press(&Button::Pad(PadButton::South)));
            assert!(input.take_press(&Button::Pad(PadButton::StickUp)));
            let device = AppleDevice {
                controller,
                profile: pad.clone(),
                device: Device {
                    id: 7,
                    name: "snapshot".into(),
                    preference: "snapshot".into(),
                    snapshot: Snapshot::default(),
                },
            };
            drop(device);
            assert!(button.pressedChangedHandler().is_null());
            assert!(axis.valueChangedHandler().is_null());
        }
    });
}
