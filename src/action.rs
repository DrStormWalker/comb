use std::io;

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, Key,
};

pub struct ActionExecutor {
    virtual_device: VirtualDevice,
}
impl ActionExecutor {
    pub fn new() -> io::Result<Self> {
        let mut keys = AttributeSet::<Key>::new();
        keys.insert(Key::KEY_PLAYPAUSE);

        let virtual_device = VirtualDeviceBuilder::new()?
            .name("Comb virtual device")
            .with_keys(&keys)?
            .build()?;

        Ok(Self { virtual_device })
    }
}

pub enum Action {
    // Bind(Vec<InputEvent>),
    // Type(String),
}
impl Action {
    // pub fn execute(&self, executor: &mut ActionExecutor) -> io::Result<()> {
    //     match self {
    //         Self::InputEvents(input_events) => executor.virtual_device.emit(
    //             &[
    //                 input_events
    //                     .iter()
    //                     .map(|event| event.as_raw())
    //                     .intersperse_with(|| {
    //                         evdev::InputEvent::new(
    //                             EventType::SYNCHRONIZATION,
    //                             Synchronization::SYN_REPORT.0,
    //                             0,
    //                         )
    //                     })
    //                     .collect::<Vec<evdev::InputEvent>>()
    //                     .as_slice(),
    //                 &[evdev::InputEvent::new(
    //                     EventType::SYNCHRONIZATION,
    //                     Synchronization::SYN_REPORT.0,
    //                     0,
    //                 )],
    //             ]
    //             .concat(),
    //         ),
    //     }
    // }
}
