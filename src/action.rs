use std::{
    collections::HashMap,
    process::{Command, Stdio},
};

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, EventType,
};

use crate::{
    config::{Action, ActionType, Config, WhenCondition},
    device::{DeviceId, DeviceInput},
    input::{Input, InputState},
};

pub struct ActionExecutor {
    actions: HashMap<DeviceId, Vec<Action>>,
    virtual_device: VirtualDevice,
    keys: AttributeSet<evdev::Key>,
    rel_axis: AttributeSet<evdev::RelativeAxisType>,
}
impl ActionExecutor {
    pub fn from_config(config: Config) -> Self {
        let actions = config
            .devices
            .into_iter()
            .map(|dev| (dev.accessor.to_string(), dev.actions))
            .collect();

        let keys = Self::keys_from_actions(&actions);
        let rel_axis = Self::rel_axis_from_actions(&actions);

        let virtual_device = Self::new_virtual_device(&keys, &rel_axis);

        Self {
            actions,
            virtual_device,
            keys,
            rel_axis,
        }
    }

    fn keys_from_actions(actions: &HashMap<DeviceId, Vec<Action>>) -> AttributeSet<evdev::Key> {
        let mut keys = AttributeSet::<evdev::Key>::new();

        let binds = actions
            .iter()
            .flat_map(|(_, actions)| actions.iter())
            .filter_map(|action| match action.action {
                ActionType::Bind { to, when: _ } => Some(to),
                _ => None,
            });

        for bind in binds {
            let key: evdev::Key = match bind {
                Input::Key(key) => key.into(),
                Input::Btn(btn) => btn.into(),
                _ => continue,
            };

            keys.insert(key);
        }

        keys
    }

    fn rel_axis_from_actions(
        actions: &HashMap<DeviceId, Vec<Action>>,
    ) -> AttributeSet<evdev::RelativeAxisType> {
        let mut keys = AttributeSet::<evdev::RelativeAxisType>::new();

        let binds = actions
            .iter()
            .flat_map(|(_, actions)| actions.iter())
            .filter_map(|action| match action.action {
                ActionType::Bind { to, when: _ } => Some(to),
                _ => None,
            });

        for bind in binds {
            let key: evdev::RelativeAxisType = match bind {
                Input::RelAxis(axis) => axis.into(),
                _ => continue,
            };

            keys.insert(key);
        }

        keys
    }

    pub fn new_virtual_device(
        keys: &AttributeSet<evdev::Key>,
        rel_axis: &AttributeSet<evdev::RelativeAxisType>,
    ) -> VirtualDevice {
        VirtualDeviceBuilder::new()
            .unwrap()
            .name("CoMB Vitual Device")
            .with_keys(&keys)
            .unwrap()
            .with_relative_axes(&rel_axis)
            .unwrap()
            .build()
            .unwrap()
    }

    pub fn update_config(&mut self, config: Config) {
        self.actions = config
            .devices
            .into_iter()
            .map(|dev| (dev.accessor.to_string(), dev.actions))
            .collect();

        let keys = Self::keys_from_actions(&self.actions);
        let rel_axis = Self::rel_axis_from_actions(&self.actions);

        let mut update_virtual_device = false;

        if !keys.iter().all(|key| self.keys.contains(key)) {
            self.keys = keys;
            update_virtual_device = true;
        }

        if !rel_axis.iter().all(|axis| self.rel_axis.contains(axis)) {
            self.rel_axis = rel_axis;
            update_virtual_device = true;
        }

        if update_virtual_device {
            self.virtual_device = Self::new_virtual_device(&self.keys, &self.rel_axis);
        }
    }

    pub fn handle_input(&mut self, input: DeviceInput) {
        let Some(actions) = self.actions.get(input.device()) else {
            return;
        };

        let actions = actions
            .iter()
            .filter(|action| action.bind == input.input_event().input());

        let input_state = input.input_event().state();
        let input = input.input_event().input();

        for action in actions {
            match action.action {
                ActionType::Hook { when, ref cmd } => {
                    if when.test(input_state) {
                        Self::execute_hook(cmd);
                    }
                }
                ActionType::Print { when, ref print } => {
                    if when.test(input_state) {
                        Self::execute_print(print);
                    }
                }
                ActionType::Bind { when, to } => {
                    if input.is_toggle() && !to.is_toggle() {
                        unimplemented!();
                    }

                    Self::execute_bind(&mut self.virtual_device, input, to, when, input_state)
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

    fn execute_bind(
        virtual_device: &mut VirtualDevice,
        input: Input,
        to: Input,
        when: Option<WhenCondition>,
        state: i32,
    ) {
        let state = if when.map(|when| when.test(state)).unwrap_or(true) {
            if !input.is_toggle() && to.is_toggle() {
                InputState::Pressed.as_i32()
            } else {
                state
            }
        } else {
            0
        };

        let (type_, key): (_, u16) = match to {
            Input::Key(key) => (EventType::KEY, Into::<evdev::Key>::into(key).0),
            Input::Btn(btn) => (EventType::KEY, Into::<evdev::Key>::into(btn).0),
            Input::RelAxis(axis) => (
                EventType::RELATIVE,
                Into::<evdev::RelativeAxisType>::into(axis).0,
            ),
            Input::AbsAxis(axis) => (
                EventType::ABSOLUTE,
                Into::<evdev::AbsoluteAxisType>::into(axis).0,
            ),
        };

        let event = evdev::InputEvent::new(type_, key, state);
        let _ = virtual_device.emit(&[event]);
    }
}
