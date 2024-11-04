use evdev::{uinput::{VirtualDevice, VirtualDeviceBuilder}, AttributeSet, Key, RelativeAxisType};

pub fn make_virtual_devices() -> std::io::Result<(VirtualDevice, VirtualDevice)> {
    let keys = AttributeSet::from_iter([
        Key::KEY_A,
        Key::KEY_B,
        Key::KEY_C,
        Key::KEY_D,
        Key::KEY_E,
    ]);

    let keyboard_device = VirtualDeviceBuilder::new()?
        .name("Fake KVM Keyboard")
        .with_keys(&keys)?
        .build()
        .expect("Could not build virtual keyboard device");

    let mouse_device = VirtualDeviceBuilder::new()?
        .name("Fake KVM Mouse")
        .with_relative_axes(&AttributeSet::from_iter([
            RelativeAxisType::REL_X,
            RelativeAxisType::REL_Y,
            RelativeAxisType::REL_WHEEL,
            RelativeAxisType::REL_HWHEEL,
        ]))?
        .build()
        .expect("Could not build virtual keyboard device");

    Ok((keyboard_device, mouse_device))
}
