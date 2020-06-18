use lcd::{CharSize, GpioPin::*, Pins, LCD};

fn main() {
    let mut lcd = LCD::new(Pins {
        rs: P7,
        rw: None,
        enable: P8,
        data: [P9, P10, P11, P12, NONE, NONE, NONE, NONE],
    });

    lcd.begin(16, 2, CharSize::Dots5x8);
    lcd.set_cursor(0, 0);
    lcd.print("Hello, ...");
    lcd.set_cursor(1, 1);
    lcd.print("... world!");
}
