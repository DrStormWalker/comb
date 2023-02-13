use std::{
    collections::HashMap,
    process::{Command, Stdio},
};

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, EventType,
};

use crate::{
    config::{Action, ActionType, Config},
    device::{DeviceId, DeviceInput},
    input::{Input, InputState},
};

pub struct ActionExecutor {
    actions: HashMap<DeviceId, Vec<Action>>,
    virtual_device: VirtualDevice,
}
impl ActionExecutor {
    pub fn from_config(config: Config) -> Self {
        let actions = config
            .devices
            .into_iter()
            .map(|dev| (dev.accessor.to_string(), dev.actions))
            .collect();

        let virtual_device = VirtualDeviceBuilder::new()
            .unwrap()
            .name("CoMB Vitual Device")
            .with_keys(&Self::keys_from_actions(&actions))
            .unwrap()
            .build()
            .unwrap();

        Self {
            actions,
            virtual_device,
        }
    }

    fn keys_from_actions(actions: &HashMap<DeviceId, Vec<Action>>) -> AttributeSet<evdev::Key> {
        let mut keys = AttributeSet::<evdev::Key>::new();

        let binds = actions
            .iter()
            .flat_map(|(_, actions)| actions.iter())
            .filter_map(|action| match action.action {
                ActionType::Bind { to } => Some(to),
                _ => None,
            });

        for bind in binds {
            let key: evdev::Key = match bind {
                Input::Key(key) => key.into(),
                Input::Btn(btn) => btn.into(),
            };

            keys.insert(key);
        }

        keys
    }

    pub fn update_config(&mut self, config: Config) {
        self.actions = config
            .devices
            .into_iter()
            .map(|dev| (dev.accessor.to_string(), dev.actions))
            .collect();
    }

    pub fn handle_input(&mut self, input: DeviceInput) {
        let Some(actions) = self.actions.get(input.device()) else {
            return;
        };

        let actions = actions
            .iter()
            .filter(|action| action.bind == input.input_event().input());

        let input_state = input.input_event().state();

        for action in actions {
            match action.action {
                ActionType::Hook { on, ref cmd } => {
                    if on == input_state {
                        Self::execute_hook(cmd);
                    }
                }
                ActionType::Print { on, ref print } => {
                    if on == input_state {
                        Self::execute_print(print);
                    }
                }
                ActionType::Bind { to } => {
                    Self::execute_bind(&mut self.virtual_device, to, input.input_event().state())
                }
            }
        }
    }

    fn execute_hook(cmd: &str) {
        let _ = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn();
    }

    fn execute_print(_print: &str) {
        unimplemented!()
    }

    fn execute_bind(virtual_device: &mut VirtualDevice, to: Input, state: InputState) {
        let key: evdev::Key = match to {
            Input::Key(key) => key.into(),
            Input::Btn(btn) => btn.into(),
        };

        let event = evdev::InputEvent::new(EventType::KEY, key.0, state.as_i32());
        let _ = virtual_device.emit(&[event]);
    }
}
