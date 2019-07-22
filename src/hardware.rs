use cortex_m::asm::delay;

use asm_delay::AsmDelay;

use stm32f1xx_hal::device::TIM2;
use stm32f1xx_hal::gpio::gpioa::{PA0, PA1, PA2, PA3, PA4, PA5};
use stm32f1xx_hal::gpio::gpiob::{
    PB0, PB1, PB10, PB12, PB13, PB14, PB15, PB2, PB3, PB4, PB5, PB6, PB7, PB8, PB9,
};
use stm32f1xx_hal::gpio::gpioc::{PC0, PC1, PC10, PC11, PC12, PC13, PC14, PC15, PC2, PC8, PC9};

use stm32f1xx_hal::gpio::{Floating, Input, OpenDrain, Output, PushPull};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::qei::Qei;

pub struct ButtonMatrix {
    pins: ButtonMatrixPins,
    rows: [u8; 11],
    delay: AsmDelay,
}

impl ButtonMatrix {
    pub fn new(pins: ButtonMatrixPins, delay: AsmDelay) -> Self {
        ButtonMatrix {
            pins,
            rows: [0; 11],
            delay,
        }
    }

    pub fn get_rows(self) -> [u8; 11] {
        self.rows
    }

    pub fn read(&mut self) {
        for i in 0..11 {
            match i {
                0 => self.pins.col1.set_low(),
                1 => self.pins.col2.set_low(),
                2 => self.pins.col3.set_low(),
                3 => self.pins.col4.set_low(),
                4 => self.pins.col5.set_low(),
                5 => self.pins.col6.set_low(),
                6 => self.pins.col7.set_low(),
                7 => self.pins.col8.set_low(),
                8 => self.pins.col9.set_low(),
                9 => self.pins.col10.set_low(),
                10 => self.pins.col11.set_low(),
                _ => panic!("This should never happen"),
            };
            self.delay.delay_us(1_u32);

            let mut row: u8 = 0;
            row |= (self.pins.row1.is_high() as u8) << 0;
            row |= (self.pins.row2.is_high() as u8) << 1;
            row |= (self.pins.row3.is_high() as u8) << 2;
            row |= (self.pins.row4.is_high() as u8) << 3;
            row |= (self.pins.row5.is_high() as u8) << 4;
            row |= (self.pins.row6.is_high() as u8) << 5;
            row |= (self.pins.row7.is_high() as u8) << 6;
            row |= (self.pins.row8.is_high() as u8) << 7;

            self.rows[i] = row;

            match i {
                0 => self.pins.col1.set_high(),
                1 => self.pins.col2.set_high(),
                2 => self.pins.col3.set_high(),
                3 => self.pins.col4.set_high(),
                4 => self.pins.col5.set_high(),
                5 => self.pins.col6.set_high(),
                6 => self.pins.col7.set_high(),
                7 => self.pins.col8.set_high(),
                8 => self.pins.col9.set_high(),
                9 => self.pins.col10.set_high(),
                10 => self.pins.col11.set_high(),
                _ => panic!("This should never happen"),
            };
        }
    }
}

pub struct ButtonMatrixPins {
    pub row1: PC8<Input<Floating>>,
    pub row2: PC9<Input<Floating>>,
    pub row3: PC10<Input<Floating>>,
    pub row4: PC11<Input<Floating>>,
    pub row5: PC12<Input<Floating>>,
    pub row6: PC13<Input<Floating>>,
    pub row7: PC14<Input<Floating>>,
    pub row8: PC15<Input<Floating>>,
    pub col1: PB0<Output<OpenDrain>>,
    pub col2: PB1<Output<OpenDrain>>,
    pub col3: PB2<Output<OpenDrain>>,
    pub col4: PB3<Output<OpenDrain>>,
    pub col5: PB4<Output<OpenDrain>>,
    pub col6: PB5<Output<OpenDrain>>,
    pub col7: PB6<Output<OpenDrain>>,
    pub col8: PB7<Output<OpenDrain>>,
    pub col9: PB8<Output<OpenDrain>>,
    pub col10: PB9<Output<OpenDrain>>,
    pub col11: PB10<Output<OpenDrain>>,
}

pub struct Encoders {
    qei: Qei<TIM2, (PA0<Input<Floating>>, PA1<Input<Floating>>)>,
    pins: EncoderPins,
    positions: [i16; 8],
    current_encoder: usize,
    last_count: u16,
}

impl Encoders {
    pub fn new(
        qei: Qei<TIM2, (PA0<Input<Floating>>, PA1<Input<Floating>>)>,
        pins: EncoderPins,
    ) -> Self {
        Encoders {
            qei,
            pins,
            positions: [0; 8],
            current_encoder: 0,
            last_count: 0,
        }
    }

    pub fn get_positions(self) -> [i16; 8] {
        self.positions
    }

    pub fn next_encoder(&mut self) -> bool {
        let current_count = self.qei.count();
        let last_count = self.last_count;
        let diff = current_count.wrapping_sub(last_count) as i16;

        self.positions[self.current_encoder] += diff;

        self.current_encoder += 1;
        if self.current_encoder == 8 {
            self.current_encoder = 0;
        }

        if self.current_encoder & 1 << 0 == 0 {
            self.pins.a0.set_low();
        } else {
            self.pins.a0.set_high();
        }
        if self.current_encoder & 1 << 1 == 0 {
            self.pins.a1.set_low();
        } else {
            self.pins.a1.set_high();
        }
        if self.current_encoder & 1 << 2 == 0 {
            self.pins.a2.set_low();
        } else {
            self.pins.a2.set_high();
        }

        self.last_count = self.qei.count();

        diff != 0
    }
}

pub struct EncoderPins {
    pub a0: PC0<Output<PushPull>>,
    pub a1: PC1<Output<PushPull>>,
    pub a2: PC2<Output<PushPull>>,
}

pub struct Leds {
    pins: LedPins,
    banks: [u32; 8],
    current_bank: usize,
}

impl Leds {
    pub fn new(pins: LedPins) -> Self {
        Leds {
            pins,
            banks: [0; 8],
            current_bank: 0,
        }
    }

    pub fn get_bank_value(&self, bank: usize) -> u32 {
        self.banks[bank]
    }

    pub fn set_bank_value(&mut self, bank: usize, value: u32) {
        self.banks[bank] = value;
    }

    pub fn write_next_bank(&mut self) {
        self.current_bank += 1;
        if self.current_bank == 8 {
            self.current_bank = 0;
        }

        self.pins.hs_en_l.set_high();
        self.pins.ls_en_l.set_high();

        if self.current_bank & 1 << 0 == 0 {
            self.pins.hs_a0.set_low();
        } else {
            self.pins.hs_a0.set_high();
        }
        if self.current_bank & 1 << 1 == 0 {
            self.pins.hs_a1.set_low();
        } else {
            self.pins.hs_a1.set_high();
        }
        if self.current_bank & 1 << 2 == 0 {
            self.pins.hs_a2.set_low();
        } else {
            self.pins.hs_a2.set_high();
        }

        for i in 0..32 {
            if self.banks[self.current_bank] & (1 << i) == 0 {
                self.pins.ls_dai.set_low();
            } else {
                self.pins.ls_dai.set_high();
            }
            delay(4);
            self.pins.ls_dck.set_high();
            delay(4);
            self.pins.ls_dck.set_low();
        }
        delay(4);
        self.pins.ls_lat.set_high();
        delay(4);
        self.pins.ls_lat.set_low();
        delay(4);

        self.pins.ls_en_l.set_low();
        self.pins.hs_en_l.set_low();
    }
}

pub struct LedPins {
    pub hs_en_l: PA2<Output<PushPull>>,
    pub hs_a0: PA3<Output<PushPull>>,
    pub hs_a1: PA4<Output<PushPull>>,
    pub hs_a2: PA5<Output<PushPull>>,
    pub ls_en_l: PB12<Output<PushPull>>,
    pub ls_dai: PB15<Output<PushPull>>,
    pub ls_dck: PB14<Output<PushPull>>,
    pub ls_lat: PB13<Output<PushPull>>,
}
