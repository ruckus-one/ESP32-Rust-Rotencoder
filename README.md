## Learning how to program `esp32c3` with Rust & IDF bindings

Using this little clone with typical a 20 impulses rotary encoder. Esp32C3 does not have a PCNT and I could not make it work with interrupts, so I decided to implement it using hardware timer & basic gray code read from two GPIOs. 

The result & feedback is then transferrd onto an SSD1306 driven 128x64 blue/yellow OLED.

![ESP32 C3 Super Mini](assets/image20240507_200600381.jpg)
![ESP32 C3 Demo](assets/sample.gif)
![ESP32_C3_wokwi](assets/rotendocer_diag.png)

- Generated from https://github.com/esp-rs/esp-idf-template
