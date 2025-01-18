use crate::range_check::{range_check, OutOfRangeError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct NitsRelativeCarCount(i32); // 負の値が前方とする

impl NitsRelativeCarCount {
    pub fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn get_channel_number(
        &self,
        car_count_front: u32,
        car_count_back: u32,
    ) -> Result<u32, OutOfRangeError<i32>> {
        let c = self.0;
        range_check(&(-15..=15), c)?;
        range_check(&(0..=15), car_count_front as i32)?;
        range_check(&(0..=15), car_count_back as i32)?;

        if c < 0 {
            Ok(1 + car_count_front - c.unsigned_abs())
        } else if c > 0 {
            Ok(31 + c.unsigned_abs() - car_count_back)
        } else {
            Ok(16)
        }
    }
}

impl std::fmt::Display for NitsRelativeCarCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 < 0 {
            write!(f, "{} Front", (-self.0).to_string())
        } else if self.0 > 0 {
            write!(f, "{} Back", self.0.to_string())
        } else {
            write!(f, "Self")
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct NitsCommandType(u8);

impl std::fmt::Display for NitsCommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:02x}", self.0)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct NitsCommand(u32);

impl NitsCommand {
    pub fn new(value: u32) -> Self {
        Self(value)
    }
    pub fn command_type(&self) -> NitsCommandType {
        NitsCommandType((self.0 >> 24 & 0xFF).try_into().unwrap())
    }
    pub fn payload(&self) -> u32 {
        self.0 & 0xFFFFFF
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NitsTick {
    commonline: NitsCommand,
    commands: BTreeMap<NitsRelativeCarCount, NitsCommand>,
}

impl NitsTick {
    pub fn new(commonline: NitsCommand) -> Self {
        Self {
            commonline,
            commands: BTreeMap::new(),
        }
    }
    pub fn add_command(&mut self, sender: NitsRelativeCarCount, command: NitsCommand) {
        self.commands.insert(sender, command);
    }
    pub fn commonline(&self) -> &NitsCommand {
        &self.commonline
    }
    pub fn commands(&self) -> &BTreeMap<NitsRelativeCarCount, NitsCommand> {
        &self.commands
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NitsSender {
    Command(NitsRelativeCarCount),
    CommonLine,
}

impl std::fmt::Display for NitsSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(sender) => write!(f, "{}", sender.to_string()),
            Self::CommonLine => write!(f, "Common Line"),
        }
    }
}
