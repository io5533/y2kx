use std::{
    thread::sleep,
    time::Duration,
};

use y2kx::{Y2kxFile, Command, Action};

use crate::backend::{Keyboard, KeyboardBackend};
use crate::cli::ButtonSetting;

pub struct Player {
    keyboard: Keyboard,
}

impl Player {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            keyboard: Keyboard::new()?,
        })
    }
    /// track_id: 1..=255
    pub fn play(&mut self, file: &Y2kxFile, track_id: u8, setting: ButtonSetting) -> std::io::Result<()> {
        let mut shift = false; // match setting { ButtonSetting::A => PitchUp1, ButtonSetting::B => PitchUp7 }
        let mut z = false; // match setting { ButtonSetting::A => PitchUp7, ButtonSetting::B => PitchUp12 }
        let mut x = false; // match setting { ButtonSetting::A => PitchUp12, ButtonSetting::B => PitchUp1 }
        let mut up = false;
        let mut down = false;
        let mut left = false;
        let mut right = false;
        if file.track_count() < track_id {
            return Err(std::io::Error::other("track_id must be 1..=255"));
        }

        for i in 0..file.keyframe_count() {
            let keyframe: (u32, &[Command]) = file.keyframe(i).map_err(std::io::Error::other)?;
            sleep(Duration::from_millis(keyframe.0 as u64));

            for command in keyframe.1 {
                let command = command.into_u16().to_be_bytes();
                if command[0] != track_id {
                    continue;
                }
                match Action::try_from(command[1]).map_err(std::io::Error::other)? {
                    Action::PitchUp1 =>
                        match setting {
                            ButtonSetting::A => {
                                shift = !shift;
                                if shift {
                                    self.keyboard.key_down(Keyboard::SHIFT)?;
                                } else {
                                    self.keyboard.key_up(Keyboard::SHIFT)?;
                                }
                            },
                            ButtonSetting::B => {
                                x = !x;
                                if x {
                                    self.keyboard.key_down(Keyboard::X)?;
                                } else {
                                    self.keyboard.key_up(Keyboard::X)?;
                                }
                            }
                        },
                    Action::PitchUp7 =>
                        match setting {
                            ButtonSetting::A => {
                                z = !z;
                                if z {
                                    self.keyboard.key_down(Keyboard::Z)?;
                                } else {
                                    self.keyboard.key_up(Keyboard::Z)?;
                                }
                            },
                            ButtonSetting::B => {
                                shift = !shift;
                                if shift {
                                    self.keyboard.key_down(Keyboard::SHIFT)?;
                                } else {
                                    self.keyboard.key_up(Keyboard::SHIFT)?;
                                }
                            }
                        },
                    Action::PitchUp12 =>
                        match setting {
                            ButtonSetting::A => {
                                x = !x;
                                if x {
                                    self.keyboard.key_down(Keyboard::X)?;
                                } else {
                                    self.keyboard.key_up(Keyboard::X)?;
                                }
                            },
                            ButtonSetting::B => {
                                z = !z;
                                if z {
                                    self.keyboard.key_down(Keyboard::Z)?;
                                } else {
                                    self.keyboard.key_up(Keyboard::Z)?;
                                }
                            }
                        },
                    Action::Up => {
                        up = !up;
                        if up {
                            self.keyboard.key_down(Keyboard::UP)?;
                        } else {
                            self.keyboard.key_up(Keyboard::UP)?;
                        }
                    },
                    Action::Down => {
                        down = !down;
                        if down {
                            self.keyboard.key_down(Keyboard::DOWN)?;
                        } else {
                            self.keyboard.key_up(Keyboard::DOWN)?;
                        }
                    },
                    Action::Left => {
                        left = !left;
                        if left {
                            self.keyboard.key_down(Keyboard::LEFT)?;
                        } else {
                            self.keyboard.key_up(Keyboard::LEFT)?;
                        }
                    },
                    Action::Right => {
                        right = !right;
                        if right {
                            self.keyboard.key_down(Keyboard::RIGHT)?;
                        } else {
                            self.keyboard.key_up(Keyboard::RIGHT)?;
                        }
                    },
                }
            }
        }

        Ok(())
    }
}