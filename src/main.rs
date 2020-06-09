use lcd::{LCD, GpioPins};

fn main() {
    let lcd = LCD::new(GpioPins {rs: 7, rw: None, enable: 8, d0123: [9, 10, 11, 12], d4567: None });

    lcd.set_cursor(0, 0);
    lcd.print("Hello, ...");
    lcd.set_cursor(1, 1);
    lcd.print("... world!");
}
