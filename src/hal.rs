use cortex_m::asm::delay;
use num_enum::TryFromPrimitive;

use crate::hardware::{Bank, LedsState};

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum ButtonType {
    Pad { x: u8, y: u8 },
    Master(u8),
    Arrow(Direction),
    Mode(ModeType),
    Parameter(ParameterType),
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum Direction {
    Up = 0,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum ModeType {
    Clip = 0,
    Mode1,
    Mode2,
    Set,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, defmt::Format)]
#[repr(u8)]
pub enum ParameterType {
    Volume = 0,
    SendA,
    SendB,
    Pan,
    Control1,
    Control2,
    Control3,
    Control4,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum ButtonEventEdge {
    PosEdge,
    NegEdge,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct ButtonEvent {
    pub btn: ButtonType,
    pub event: ButtonEventEdge,
}

pub static PARAMETER_TYPES: [ParameterType; 8] = [
    ParameterType::Volume,
    ParameterType::SendA,
    ParameterType::SendB,
    ParameterType::Pan,
    ParameterType::Control1,
    ParameterType::Control2,
    ParameterType::Control3,
    ParameterType::Control4,
];

pub static DIRECTION_TYPES: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

pub static MODE_TYPES: [ModeType; 4] = [
    ModeType::Clip,
    ModeType::Mode1,
    ModeType::Mode2,
    ModeType::Set,
];

impl ButtonEvent {
    pub fn new(row: u8, col: u8, event: ButtonEventEdge) -> ButtonEvent {
        let btn = match col {
            0..=7 => {
                let x = row;
                let y = col;

                ButtonType::Pad { x, y }
            }
            8 => ButtonType::Master(row + 1),
            9 => match row {
                0..=3 => ButtonType::Arrow(DIRECTION_TYPES[row as usize]),
                4..=7 => ButtonType::Mode(MODE_TYPES[row as usize - 4]),
                _ => unreachable!(),
            },
            10 => ButtonType::Parameter(PARAMETER_TYPES[row as usize]),
            _ => unreachable!(),
        };

        ButtonEvent { btn, event }
    }
}
#[derive(Clone, Copy, defmt::Format)]
pub struct LedEvent {
    btn: ButtonType,
    event: LedEventType,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum LedEventType {
    Switch(bool),
    SwitchColor(PadColor),
}

pub const COLOR_BLACK: PadColor = PadColor { r: 0, g: 0, b: 0 };
pub const COLOR_WHITE: PadColor = PadColor { r: 3, g: 3, b: 3 };
pub const COLOR_YELLOW: PadColor = PadColor { r: 3, g: 3, b: 0 };
pub const COLOR_AQUA: PadColor = PadColor { r: 0, g: 3, b: 3 };
pub const COLOR_PURPLE: PadColor = PadColor { r: 3, g: 0, b: 3 };
pub const COLOR_BLUE: PadColor = PadColor { r: 0, g: 0, b: 3 };
pub const COLOR_GREEN: PadColor = PadColor { r: 0, g: 3, b: 0 };
pub const COLOR_RED: PadColor = PadColor { r: 3, g: 0, b: 0 };

/// Describes a pad's color using RGB values in the range of [0, 3].
///
/// The time is divided into 4 slots. The values tell how many of the slots the corresponding LED
/// should be turned on.
///
/// Time slots have different lengths to implement some kind of gamma correction.
#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct PadColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Binary RGB value.
///
/// Indicates which of the LEDs should be on.
#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct Rgb {
    pub r: bool,
    pub g: bool,
    pub b: bool,
}

impl PadColor {
    pub fn new(r: u8, g: u8, b: u8) -> Option<PadColor> {
        if r < 4 && g < 4 && b < 4 {
            Some(PadColor { r, g, b })
        } else {
            None
        }
    }

    /// The 2-bit values for the color components are encoded into the lowest 6 bits.
    pub fn from_value(v: u8) -> PadColor {
        PadColor::new(v & 0b11, (v & (0b11 << 2)) >> 2, (v & (0b11 << 4)) >> 4).unwrap()
    }

    /// Calculates which LEDs should be on in each of the 4 cycles.
    pub fn as_rgb(&self) -> [Rgb; 4] {
        let mut rgb = [Rgb {
            r: false,
            g: false,
            b: false,
        }; 4];

        if self.r >= 1 {
            rgb[0].r = true;
        }
        if self.r >= 2 {
            rgb[1].r = true;
        }
        if self.r >= 3 {
            rgb[2].r = true;
        }

        if self.g >= 1 {
            rgb[0].g = true;
        }
        if self.g >= 2 {
            rgb[1].g = true;
        }
        if self.g >= 3 {
            rgb[2].g = true;
        }

        if self.b >= 1 {
            rgb[0].b = true;
        }
        if self.b >= 2 {
            rgb[1].b = true;
        }
        if self.b >= 3 {
            rgb[2].b = true;
        }

        rgb
    }
}

impl LedEvent {
    pub fn new(btn: ButtonType, event: LedEventType) -> LedEvent {
        LedEvent { btn, event }
    }

    pub fn apply_to_leds_state(&self, state: LedsState) -> LedsState {
        let mut new_banks = state;

        match (self.event, self.btn) {
            (LedEventType::Switch(s), ButtonType::Master(i)) => {
                let bank = Bank::C1 as usize;
                let bit = 32 - i;

                for new_bank in new_banks.0.iter_mut() {
                    if s {
                        new_bank[bank] |= 1 << bit;
                    } else {
                        new_bank[bank] &= !(1 << bit);
                    }
                }
            }
            (LedEventType::Switch(s), ButtonType::Arrow(d)) => {
                let bank = Bank::C0 as usize;

                let bit = 31 - d as u8;

                for new_bank in new_banks.0.iter_mut() {
                    if s {
                        new_bank[bank] |= 1 << bit;
                    } else {
                        new_bank[bank] &= !(1 << bit);
                    }
                }
            }
            (LedEventType::Switch(s), ButtonType::Mode(m)) => {
                let bank = Bank::C0 as usize;
                let bit = 27 - m as u8;

                for new_bank in new_banks.0.iter_mut() {
                    if s {
                        new_bank[bank] |= 1 << bit;
                    } else {
                        new_bank[bank] &= !(1 << bit);
                    }
                }
            }
            (LedEventType::Switch(s), ButtonType::Parameter(p)) => {
                let bank = Bank::C0 as usize;
                let bit = 23 - p as u8;

                for new_bank in new_banks.0.iter_mut() {
                    if s {
                        new_bank[bank] |= 1 << bit;
                    } else {
                        new_bank[bank] &= !(1 << bit);
                    }
                }
            }
            (LedEventType::SwitchColor(color), ButtonType::Pad { x, y }) => {
                let (bank_r, bank_g, bank_b) = if y < 4 {
                    (Bank::R0 as usize, Bank::G0 as usize, Bank::B0 as usize)
                } else {
                    (Bank::R1 as usize, Bank::G1 as usize, Bank::B1 as usize)
                };

                let bit = 31 - (((y % 4) * 8) + x);

                for (new_bank, Rgb { r, g, b }) in
                    new_banks.0.iter_mut().zip(color.as_rgb().into_iter())
                {
                    if r {
                        new_bank[bank_r] |= 1 << bit;
                    } else {
                        new_bank[bank_r] &= !(1 << bit);
                    }

                    if g {
                        new_bank[bank_g] |= 1 << bit;
                    } else {
                        new_bank[bank_g] &= !(1 << bit);
                    }

                    if b {
                        new_bank[bank_b] |= 1 << bit;
                    } else {
                        new_bank[bank_b] &= !(1 << bit);
                    }
                }
            }
            _ => unreachable!(),
        };

        new_banks
    }
}

pub fn delay_us(us: u32) {
    const SYSCLK_MHZ: u32 = 72;

    delay(SYSCLK_MHZ * us);
}
