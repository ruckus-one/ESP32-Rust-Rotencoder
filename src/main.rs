use anyhow::Result;
use esp_idf_hal::{delay::FreeRtos, gpio::{InputPin, Level, OutputPin}};
use esp_idf_svc::hal::{
    gpio::{PinDriver, Pull},
    peripherals::Peripherals,
};
use std::thread;

use esp_idf_hal::i2c::*;
use esp_idf_hal::prelude::*;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
    primitives::{Circle, PrimitiveStyleBuilder}
};

struct Rotencoder<T1: InputPin + OutputPin, T2: InputPin + OutputPin> {
    clk: T1,
    dt: T2,
}

impl<T1: InputPin + OutputPin, T2: InputPin + OutputPin> Rotencoder<T1, T2> {
    fn start_thread(self) {
        let _t1: thread::JoinHandle<_> = thread::Builder::new()
            .stack_size(2000)
            .spawn(move || {
                
                let mut dt_last = Level::Low;
                let mut counter: i32 = 0;
            
                let mut button_1 = PinDriver::input(self.clk).unwrap();
                button_1.set_pull(Pull::Up).unwrap();
                let mut button_2 = PinDriver::input(self.dt).unwrap();
                button_2.set_pull(Pull::Up).unwrap();

                loop {
                    if button_1.get_level() != dt_last {
                        dt_last = button_1.get_level();
                        
                        if dt_last == Level::High {
                            if button_2.get_level() == Level::High {
                                counter += 1;
                            } else {
                                counter -= 1;
                            }
                            println!("counter is {:?}", counter);
                        } else {
                            if button_2.get_level() == Level::Low {
                                counter += 1;
                            } else {
                                counter -= 1;
                            }
                        }
                        FreeRtos::delay_ms(1);
                    }
                }

            })
            .unwrap();
    }
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take()?;

    let encoder = Rotencoder {
        clk: peripherals.pins.gpio0,
        dt: peripherals.pins.gpio1,
    };

    encoder.start_thread();

    let i2c = peripherals.i2c0;
    let scl = peripherals.pins.gpio4;
    let sda = peripherals.pins.gpio5;

    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let off = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(BinaryColor::Off)
        .build();

    let on = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(BinaryColor::On)
        .build();

    let mut i = 0;
    let mut dir = 1;

    Text::with_baseline("Hello world!", Point::new(30, 0), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    Text::with_baseline("Hello Rust!!", Point::new(30, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    loop {
        Circle::new(Point::new(i, 40), 16)
            .into_styled(off)
            .draw(&mut display)
            .unwrap();

        if i > 100 { 
            dir = -1; 
        } 

        if i == 0 {
            dir = 1;
        }

        i += dir << 4;

        Circle::new(Point::new(i, 40), 16)
            .into_styled(on)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();

        FreeRtos::delay_ms(50);
    }

    // anyhow::Ok(())
}