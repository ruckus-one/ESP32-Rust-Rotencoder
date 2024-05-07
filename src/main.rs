use anyhow::Result;
use esp_idf_hal::{delay::FreeRtos, gpio::{InputPin, Level, OutputPin}};
use esp_idf_svc::hal::{
    gpio::{PinDriver, Pull},
    peripherals::Peripherals,
};
use std::thread;

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

    Ok(())
}