#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_rp::i2c::{I2c, InterruptHandler as I2CInterruptHandler, Config as I2cConfig};
use embassy_rp::peripherals::I2C1;
use embassy_rp::bind_interrupts;
use {defmt_rtt as _, panic_probe as _};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Text, Baseline},
};
use heapless::String;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use embassy_rp::gpio::{Level, Output, Input, Pull};
mod keypad;
mod input;
mod eval;

use crate::keypad::{scan_keypad_4x3, scan_keypad_4x4};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => I2CInterruptHandler<I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut buzzer = Output::new(p.PIN_14, Level::Low);
    let sda = p.PIN_6;
    let scl = p.PIN_7;
    let i2c = I2c::new_async(p.I2C1, scl, sda, Irqs, I2cConfig::default());
    let mut display = Ssd1306::new(
        I2CDisplayInterface::new(i2c),
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();
    display.init().unwrap();
    let style = MonoTextStyleBuilder::new().font(&FONT_6X10).text_color(BinaryColor::On).build();

    let mut cols_4x4 = [
        Output::new(p.PIN_10, Level::High),
        Output::new(p.PIN_11, Level::High),
        Output::new(p.PIN_12, Level::High),
        Output::new(p.PIN_13, Level::High),
    ];
    let rows_4x4 = [
        Input::new(p.PIN_2, Pull::Up),
        Input::new(p.PIN_3, Pull::Up),
        Input::new(p.PIN_4, Pull::Up),
        Input::new(p.PIN_5, Pull::Up),
    ];
    let mut cols_4x3 = [
        Output::new(p.PIN_18, Level::High),
        Output::new(p.PIN_16, Level::High),
        Output::new(p.PIN_20, Level::High),
    ];
    let rows_4x3 = [
        Input::new(p.PIN_17, Pull::Up),
        Input::new(p.PIN_22, Pull::Up),
        Input::new(p.PIN_21, Pull::Up),
        Input::new(p.PIN_19, Pull::Up),
    ];
    let mut expr: String<64> = String::new();
    let mut fresh = true;
    let mut last_answer: Option<f64> = None;
    loop {
        scan_keypad_4x4(&mut cols_4x4, &rows_4x4, &mut expr, &mut fresh, &mut last_answer, &mut buzzer).await;
        scan_keypad_4x3(&mut cols_4x3, &rows_4x3, &mut expr, &mut fresh, &mut last_answer, &mut buzzer).await;

        display.clear(BinaryColor::Off).unwrap();
        let max_chars = 20;
        let line_height = 10;
        let s = expr.as_str();
        let mut start = 0;
        let len = s.len();
        for line in 0..4 {
            if start >= len { break; }
            let mut end = (start + max_chars).min(len);
            if end < len && s.chars().nth(end).unwrap().is_digit(10) {
                if let Some(pos) = s[start..end].rfind(|c: char| !c.is_digit(10)) {
                    end = start + pos + 1;
                }
            }
            let chunk = &s[start..end];
            Text::with_baseline(
                chunk,
                Point::new(0, (line * line_height) as i32),
                style,
                Baseline::Top,
            )
            .draw(&mut display)
            .unwrap();
            start = end;
        }
        display.flush().unwrap();
        Timer::after(Duration::from_millis(100)).await;
    }
}
