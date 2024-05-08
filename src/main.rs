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
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder}, pixelcolor::BinaryColor, prelude::*, primitives::{Arc, Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle}, text::{Baseline, Text}
};
use crossbeam_channel::{bounded, Receiver, Sender};

struct Rotencoder<T1: InputPin + OutputPin, T2: InputPin + OutputPin> {
    clk: T1,
    dt: T2,
}

impl<T1: InputPin + OutputPin, T2: InputPin + OutputPin> Rotencoder<T1, T2> {
    fn start_thread(self, tx: Sender<i32>) {
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
                            tx.send(counter).unwrap();
                            // println!("counter is {:?}", counter);
                        } else {
                            if button_2.get_level() == Level::Low {
                                counter += 1;
                            } else {
                                counter -= 1;
                            }
                            // println!("counter is {:?}", counter);
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

    let (tx, rx) = bounded::<i32>(8);

    encoder.start_thread(tx);

    thread::Builder::new()
        .stack_size(4000)
        .spawn(move || {

            let i2c = peripherals.i2c0;
            let scl = peripherals.pins.gpio3;
            let sda = peripherals.pins.gpio2;

            let config = I2cConfig::new().baudrate(400.kHz().into());
            let i2c = I2cDriver::new(i2c, sda, scl, &config).unwrap();

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
                .fill_color(BinaryColor::Off)
                .build();

            let on = PrimitiveStyleBuilder::new()
                .stroke_width(1)
                .stroke_color(BinaryColor::On)
                .build();

            let mut i = 0;
            let mut dir = 1;

            loop {
                match rx.try_recv() {

                    Ok(counter) => {
                        // let foo = format!("foo -> {:?}", counter);
                        // let bar = foo.len() as u32;
                        // let mut baz = FONT_6X10.character_size;
                        // baz.width = bar * (baz.width + 3);

                        // Rectangle::new(Point::new(30, 32), baz)
                        //     .into_styled(off)
                        //     .draw(&mut display)
                        //     .unwrap();

                        // Text::with_baseline(foo.as_str(), Point::new(30, 32), text_style, Baseline::Top)
                        //     .draw(&mut display)
                        //     .unwrap();
                        
                        Circle::new(Point::new(64-20, 25), 30 + 2*5)
                            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                            .draw(&mut display)
                            .unwrap();

                        Arc::new(Point::new(64-15, 30), 30, 0.0.deg(), ((counter*8) as f32).deg())
                            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 5))
                            .draw(&mut display)
                            .unwrap();

                        display.flush().unwrap();
                        // println!("{}", format!("foo -> {}", counter).as_str());

                    },
                    Err(_) => {},
                }

                FreeRtos::delay_ms(1);
            }
        })
        .unwrap();

    loop { }
    // anyhow::Ok(())
}