use esp_idf_hal::gpio::{InputPin, Level, OutputPin};
use esp_idf_svc::{hal::gpio::{PinDriver, Pull}, timer::EspTimer};
use std::{sync::atomic::{ AtomicI32, Ordering }, thread::{self, JoinHandle}, time::Duration};


use esp_idf_svc::timer::EspTimerService;
use std::sync::Arc;

pub mod rotencoder {
    use super::*;

    pub struct Rotencoder<T1: InputPin + OutputPin, T2: InputPin + OutputPin> {
        clk: T1,
        dt: T2,
        counter: Arc<AtomicI32>,
    }

    impl<T1: InputPin + OutputPin, T2: InputPin + OutputPin> Rotencoder<T1, T2> {
        pub fn with_counter(clk: T1, dt: T2, counter: Arc<AtomicI32>) -> Self {
            Self { clk, dt, counter }
        }

        pub fn start_thread(self) -> JoinHandle<EspTimer<'static>> {
            let timer_service = EspTimerService::new().unwrap();

            return thread::Builder::new()
                .stack_size(2000)
                .spawn(move || {
                    let mut button_1 = PinDriver::input(self.clk).unwrap();
                    button_1.set_pull(Pull::Up).unwrap();
                    let mut button_2 = PinDriver::input(self.dt).unwrap();
                    button_2.set_pull(Pull::Up).unwrap();

                    let callback_timer = {
                        let mut prev = 0;

                        timer_service.timer(move || {
                            let a = button_1.get_level();
                            let b = button_2.get_level();
                            let curr = Rotencoder::<T1, T2>::graycode_to_binary(a, b);
                            let diff = prev - curr;

                            if diff == -1 || diff == 3 {
                                self.counter.fetch_sub(1, Ordering::SeqCst);
                                prev = curr;
                            } else if diff == 1 || diff == -3 {
                                self.counter.fetch_add(1, Ordering::SeqCst);
                                prev = curr;
                            }
                        })
                        .unwrap()
                    };
                    callback_timer.every(Duration::from_micros(244)).unwrap();
                    return callback_timer;
                })
                .unwrap()
        }

        fn graycode_to_binary(a: Level, b: Level) -> i8 {
            if a == Level::Low && b == Level::Low {
                return 0
            } else if a == Level::Low && b == Level::High {
                return 1
            } else if a == Level::High && b == Level::High {
                return 2
            }
        
            return 3
        }
    }
}
