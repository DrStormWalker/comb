use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    device::DeviceAccessor,
    input::{Input, InputState},
};

use super::util::display_from_str;

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    #[serde(flatten)]
    pub accessor: DeviceAccessor,
    #[serde(default)]
    pub actions: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    #[serde(with = "display_from_str")]
    pub bind: Input,
    #[serde(flatten)]
    pub action: ActionType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ActionType {
    Hook {
        #[serde(default = "WhenCondition::pressed")]
        when: WhenCondition,
        cmd: String,
    },
    Bind {
        #[serde(with = "display_from_str")]
        to: Input,
        #[serde(default)]
        when: Option<WhenCondition>,
    },
    Print {
        #[serde(default = "WhenCondition::pressed")]
        when: WhenCondition,
        print: String,
    },
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WhenCondition {
    InputState(#[serde(with = "display_from_str")] InputState),
    Condition(#[serde(with = "display_from_str")] Condition),
}
impl WhenCondition {
    pub fn pressed() -> Self {
        Self::InputState(InputState::Pressed)
    }

    pub fn test(&self, value: i32) -> bool {
        match self {
            // if let guards are unstable
            // (issue #51114 https://github.com/rust-lang/rust/issues/51114)
            Self::InputState(state) if let Some(value) = InputState::from_i32(value) => state == &value,
            Self::Condition(condition) => condition.test(value),
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ConditionOp {
    Lt,
    Gt,
    LtEq,
    GtEq,
    Eq,
    Neq,
}
impl FromStr for ConditionOp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<" => Ok(Self::Lt),
            "<=" => Ok(Self::LtEq),
            ">" => Ok(Self::Gt),
            ">=" => Ok(Self::GtEq),
            "=" | "==" => Ok(Self::Eq),
            "!=" => Ok(Self::Neq),
            _ => Err(()),
        }
    }
}
impl Display for ConditionOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lt => write!(f, "<"),
            Self::LtEq => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::GtEq => write!(f, ">="),
            Self::Eq => write!(f, "="),
            Self::Neq => write!(f, "!="),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Condition(ConditionOp, i32);
impl Condition {
    pub fn test(&self, v: i32) -> bool {
        match self.0 {
            ConditionOp::Lt => v < self.1,
            ConditionOp::LtEq => v <= self.1,
            ConditionOp::Gt => v > self.1,
            ConditionOp::GtEq => v >= self.1,
            ConditionOp::Eq => v == self.1,
            ConditionOp::Neq => v != self.1,
        }
    }
}
impl FromStr for Condition {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = s.chars();

        let (op, operand) = if chars.skip(1).next().ok_or(())? == '=' {
            (&s[..2], s[2..].trim())
        } else {
            (&s[..1], s[1..].trim())
        };

        let op = op.parse()?;
        let operand = operand.parse().map_err(|_| ())?;

        Ok(Self(op, operand))
    }
}
impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}
