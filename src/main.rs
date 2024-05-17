use anyhow::Result;
use esp_idf_svc::hal::
    peripherals::Peripherals
;
use std::{sync::{atomic::{ AtomicI32, Ordering }, Mutex}, thread, time::Duration};

use esp_idf_hal::i2c::*;
use esp_idf_hal::prelude::*;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyleBuilder}, pixelcolor::BinaryColor, prelude::*, primitives::{Arc as Arch, Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle}, text::{Baseline, Text}
};
use std::sync::Arc;

use rotencoder::rotencoder::Rotencoder;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take()?;

    let counter = Arc::new(AtomicI32::new(0));

    let _rotencoder_handle = {
        let counter = counter.clone();

        let encoder = Rotencoder::with_callback(
            peripherals.pins.gpio7,
            peripherals.pins.gpio21,
            Arc::new(Mutex::new(move |delta: i8| {

                match delta {
                    1 => counter.fetch_add(1, Ordering::SeqCst),
                    -1 => counter.fetch_sub(1, Ordering::SeqCst),
                    _ => 0_i32,
                };
            }))
        );

        encoder.start_thread()
    };

    let _oled_handle = thread::Builder::new()
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
                    .font(&FONT_8X13)
                    .text_color(BinaryColor::On)
                    .build();

                let off = PrimitiveStyleBuilder::new()
                    .stroke_width(1)
                    .stroke_color(BinaryColor::Off)
                    .fill_color(BinaryColor::Off)
                    .build();

                loop {
                    let position = counter.load(Ordering::SeqCst);
                    let foo = format!("{}", position);
                    let bar = foo.len() as u32;
                    let mut baz = FONT_8X13.character_size;
                    baz.width = bar * (baz.width + 3);
            
                    Circle::new(Point::new(64-20, 15), 40 + 2*5)
                        .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                        .draw(&mut display)
                        .unwrap();
            
                    Arch::new(Point::new(64-15, 20), 40, 0.0.deg(), ((position*4) as f32).deg())
                        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 5))
                        .draw(&mut display)
                        .unwrap();
            
            
                    Rectangle::new(Point::new(64-10, 30), baz)
                        .into_styled(off)
                        .draw(&mut display)
                        .unwrap();
            
                    Text::with_baseline(foo.as_str(), Point::new(64-10, 30), text_style, Baseline::Top)
                        .draw(&mut display)
                        .unwrap();
            
            
                    match display.flush() {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error flushing: {:?}", e);
                        }
                    }
            
                    thread::sleep(Duration::from_millis(20));
                }
            })
            .unwrap();

    loop {
        thread::sleep(Duration::from_millis(20));
    }
}
