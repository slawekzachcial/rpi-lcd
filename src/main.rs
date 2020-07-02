use gpio_cdev::errors;
use lcd::{CharSize, GpioPin::*, Pins, LCD};
use std::{thread, time};

fn do_main() -> Result<(), errors::Error> {
    let mut lcd = LCD::new(Pins {
        rs: P7,
        rw: None,
        enable: P8,
        data: [P9, P10, P11, P12, NONE, NONE, NONE, NONE],
    })?;

    lcd.begin(16, 2, CharSize::Dots5x8);

    lcd.set_cursor(0, 0);
    lcd.print("Hello, ...");

    delay_millis(500);

    for _ in 0..16 {
        lcd.scroll_display_right();
        delay_millis(200);
    }

    lcd.clear();

    lcd.set_cursor(6, 1);
    lcd.print("... world!");

    delay_millis(500);

    for _ in 0..16 {
        lcd.scroll_display_left();
        delay_millis(250);
    }

    lcd.clear();
    lcd.print("turning off ...");
    delay_millis(2000);
    lcd.no_display();
    delay_millis(2000);
    lcd.clear();
    lcd.print("turned on");
    lcd.display();
    delay_millis(2000);

    lcd.clear();
    lcd.cursor();
    delay_millis(2000);
    lcd.print("cursor ");
    delay_millis(1000);
    lcd.no_blink();
    delay_millis(1000);
    lcd.blink();
    delay_millis(2000);
    lcd.no_cursor();
    delay_millis(1000);
    lcd.no_blink();

    lcd.clear();
    lcd.set_cursor(15, 0);
    lcd.right_to_left();
    lcd.print("right to left");
    delay_millis(3000);

    lcd.left_to_right();
    lcd.clear();
    // lcd.set_cursor(16, 0);
    // lcd.autoscroll();
    // for c in "The quick brown fox jumps over the lazy dog".chars() {
    //     lcd.print(format!("{}", c).as_str());
    //     delay_millis(300);
    // }

    let smiley = [
        0b00000u8,
        0b10001u8,
        0b10001u8,
        0b00000u8,
        0b10001u8,
        0b01110u8,
        0b00000u8,
        0b00000u8,
    ];
    let big_dot = [
        0b00000u8,
        0b01110u8,
        0b11111u8,
        0b11111u8,
        0b11111u8,
        0b01110u8,
        0b00000u8,
        0b00000u8,
    ];

    lcd.create_char(0, smiley);
    lcd.create_char(1, big_dot);
    lcd.clear();
    lcd.write(0);
    lcd.set_cursor(3, 1);
    lcd.write(0);
    lcd.set_cursor(5, 0);
    lcd.write(1);
    delay_millis(30000);

    lcd.clear();
    lcd.print("The End");
    delay_millis(2000);
    lcd.clear();
    Ok(())
}

fn delay_millis(millis: u64) {
    thread::sleep(time::Duration::from_millis(millis));
}

fn main() {
    match do_main() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error {:?}", e);
        }
    }
}
