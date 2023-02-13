use std::{
    collections::HashMap,
    process::{Command, Stdio},
};

use crate::{
    config::{Action, ActionType, Config},
    device::{DeviceId, DeviceInput},
};

pub struct ActionExecutor {
    actions: HashMap<DeviceId, Vec<Action>>,
}
impl ActionExecutor {
    pub fn from_config(config: Config) -> Self {
        let actions = config
            .devices
            .into_iter()
            .map(|dev| (dev.accessor.to_string(), dev.actions))
            .collect();

        Self { actions }
    }

    pub fn update_config(&mut self, config: Config) {
        self.actions = config
            .devices
            .into_iter()
            .map(|dev| (dev.accessor.to_string(), dev.actions))
            .collect();
    }

    pub fn handle_input(&self, input: DeviceInput) {
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
                ActionType::Bind { to: _ } => unimplemented!(),
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
}
