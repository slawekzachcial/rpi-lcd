//! The `rpi-lcd` crate allows Raspberry Pi to control LiquidCrystal displays (LCDs) based on the
//! Hitachi HD44780 (or a compatible) chipset, which is found on most text-based LCDs. The library
//! works with in either 4- or 8-bit mode (i.e. using 4 or 8 data lines in addition to the `rs`,
//! `enable`, and, optionally, the `rw` control lines).
//!
//! The crate is a Rust port of [LiquidCrystal](https://github.com/arduino-libraries/LiquidCrystal)
//! Arduino library. The library API documentation has also been copied and adapted accordingly.
//!
//! # Examples
//!
//! The following example shows how to print "Hello World!" assuming Raspberry Pi
//! has been connected to the LCD display using the referenced below GPIO pins.
//!
//! ```rust,no_run
//! use gpio_cdev::errors;
//! use rpi_lcd::{CharSize, GpioPin::*, Pins, LCD};
//!
//! fn do_main() -> Result<(), errors::Error> {
//!
//!     let mut lcd = LCD::new(Pins {
//!         rs: P26,
//!         rw: None,
//!         enable: P19,
//!         data: [NONE, NONE, NONE, NONE, P13, P06, P05, P11],
//!     })?;
//!
//!     lcd.begin(16, 2, CharSize::Dots5x8);
//!     lcd.print("Hello,  World!");
//! }
//!
//! fn main() {
//!     match do_main() {
//!         Ok(()) => {}
//!         Err(e) => {
//!             eprintln!("Error {:?}", e);
//!         }
//!     }
//! }
//! ```

use gpio_cdev::*;
use std::{thread, time};
use std::convert::TryInto;

fn delay_micros(micros: u64) {
    thread::sleep(time::Duration::from_micros(micros));
}

const DATA_PINS: usize = 8;

/// Raspberry Pi GPIO pin references
///
/// The values of this enum are used to indicate in [Pins](struct.Pins.html] which GPIO pin is
/// connected to which LCD pin. Use `GpioPin::NONE` for LCD data pins 0 to 3 to indicate the LCD
/// works in 4-bit mode.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum GpioPin {
    NONE = -1,
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
    P4 = 4,
    P5 = 5,
    P6 = 6,
    P7 = 7,
    P8 = 8,
    P9 = 9,
    P10 = 10,
    P11 = 11,
    P12 = 12,
    P13 = 13,
    P14 = 14,
    P15 = 15,
    P16 = 16,
    P17 = 17,
    P18 = 18,
    P19 = 19,
    P20 = 20,
    P21 = 21,
    P22 = 22,
    P23 = 23,
    P24 = 24,
    P25 = 25,
    P26 = 26,
    P27 = 27,
}

impl GpioPin {

    fn line_handle(&self, chip: &mut Chip, consumer: &str) -> Result<LineHandle, errors::Error> {
        Ok(chip.get_line(*self as u32)?.request(LineRequestFlags::OUTPUT, 1, consumer)?)
    }
}

trait OutputPin {
    fn write(&self, value: GpioPinSignal);
}

impl OutputPin for LineHandle {
    fn write(&self, value: GpioPinSignal) {
        self.set_value(value as u8).unwrap();
    }
}

#[derive(Debug)]
enum GpioPinSignal {
    High = 0x01,
    Low = 0x00,
}

impl GpioPinSignal {
    fn from(value: u8) -> Self {
        match value {
            0 => GpioPinSignal::Low,
            1 => GpioPinSignal::High,
            _ => panic!("Invalid signal value: {:?}", value),
        }
    }
}

#[derive(Debug)]
enum Command {
    ClearDisplay = 0x01,
    ReturnHome = 0x02,
    EntryModeSet = 0x04,
    DisplayControl = 0x08,
    CursorShift = 0x10,
    FunctionSet = 0x20,
    SetCGRamAddress = 0x40,
    SetDDRamAddress = 0x80,
}

impl Command {
    fn clear_display() -> u8 {
        Command::ClearDisplay as u8
    }

    fn return_home() -> u8 {
        Command::ReturnHome as u8
    }

    fn entry_mode_set(dm: &DisplayMode) -> u8 {
        Command::EntryModeSet as u8 | dm.entry_mode as u8 | dm.entry_shift_mode as u8
    }

    fn display_control(dc: &DisplayControl) -> u8 {
        Command::DisplayControl as u8 | dc.display as u8 | dc.cursor as u8 | dc.blink as u8
    }

    fn cursor_shift(what: &MoveControl, direction: &MoveDirection) -> u8 {
        Command::CursorShift as u8 | what.as_u8() | direction.as_u8()
    }

    fn function_set(df: &DisplayFunction) -> u8 {
        Command::FunctionSet as u8 | df.mode as u8 | df.lines as u8 | df.char_size as u8
    }

    fn set_ddram_address(address: u8) -> u8 {
        Command::SetDDRamAddress as u8 | address
    }

    fn set_cgram_address(address: u8) -> u8 {
        eprintln!("address: {:08b}", address);
        Command::SetCGRamAddress as u8 | address
    }
}

#[derive(Debug, Clone, Copy)]
enum DisplayEntryMode {
    Right = 0x00,
    Left = 0x02,
}

#[derive(Debug, Clone, Copy)]
enum DisplayEntryShiftMode {
    Increment = 0x01,
    Decrement = 0x00,
}

#[derive(Debug, Clone, Copy)]
enum DisplayState {
    On = 0x04,
    Off = 0x00,
}

#[derive(Debug, Clone, Copy)]
enum CursorState {
    On = 0x02,
    Off = 0x00,
}

#[derive(Debug, Clone, Copy)]
enum BlinkState {
    On = 0x01,
    Off = 0x00,
}

#[derive(Debug)]
enum MoveControl {
    Display = 0x08,
    Cursor = 0x00,
}

impl MoveControl {
    fn as_u8(&self) -> u8 {
        match self {
            MoveControl::Display => MoveControl::Display as u8,
            MoveControl::Cursor => MoveControl::Cursor as u8,
        }
    }
}


#[derive(Debug)]
enum MoveDirection {
    Right = 0x04,
    Left = 0x00,
}

impl MoveDirection {
    fn as_u8(&self) -> u8 {
        match self {
            MoveDirection::Right => MoveDirection::Right as u8,
            MoveDirection::Left => MoveDirection::Left as u8,
        }
    }
}

/// Size of the LCD character
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CharSize {
    Dots5x8 = 0x00,
    Dots5x10 = 0x04,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Mode {
    Bits4 = 0x00,
    Bits8 = 0x10,
}

#[derive(Debug, Clone, Copy)]
enum Lines {
    Lines1 = 0x00,
    Lines2 = 0x08,
}

/// LCD pins
///
/// To inidicate that LCD works in 4-bit mode use `GpioPin::NONE` as the first
/// 4 items of `data` pins array.
#[derive(Debug)]
pub struct Pins {
    /// GPIO pin connected to LCD RS pin
    pub rs: GpioPin,

    /// GPIO pin connected to LCD RW pin; may be set to `None` if LCD RW pin is not used
    pub rw: Option<GpioPin>,

    /// GPIO pin connected to LCD ENABLE pin
    pub enable: GpioPin,

    /// GPIO pins connected to LCD DATA pins d0 to d7
    ///
    /// Set the first 4 items of this array to `GpioPin::NONE` to indicate LCD
    /// is working in 4-bit mode.
    pub data: [GpioPin; DATA_PINS],
}

struct LineHandles {
    rs: LineHandle,
    rw: Option<LineHandle>,
    enable: LineHandle,
    data: [Option<LineHandle>; DATA_PINS],
}

#[derive(Debug)]
struct DisplayFunction {
    mode: Mode,
    lines: Lines,
    char_size: CharSize,
}

#[derive(Debug)]
struct DisplayControl {
    display: DisplayState,
    cursor: CursorState,
    blink: BlinkState,
}

#[derive(Debug)]
struct DisplayMode {
    entry_mode: DisplayEntryMode,
    entry_shift_mode: DisplayEntryShiftMode,
}

/// LCD display main struct
pub struct LCD {
    pins: LineHandles,
    display_function: DisplayFunction,
    display_control: DisplayControl,
    display_mode: DisplayMode,
    row_offsets: [u8; 4],
    num_lines: u8,
}

impl LCD {

    /// Creates a variable of type LCD. The display can be controlled using 4 or 8 data
    /// lines. If the former, set the `Pins.data` 0 to 3 array items to `GpioPin::NONE`
    /// and leave those lines unconnected. The RW pin can be tied to ground instead of connected to
    /// a pin on the Raspberry; if so, set the `Pins.rw` to `None`. See [Pins](struct.Pins.html)
    /// for detailed parameters description.
    ///
    /// # Examples
    /// ```rust,no_run
    /// let mut lcd = LCD::new(Pins {
    ///     rs: P26,
    ///     rw: None,
    ///     enable: P19,
    ///     data: [NONE, NONE, NONE, NONE, P13, P06, P05, P11],
    /// })?;
    /// ```
    pub fn new(pins: Pins) -> Result<LCD, errors::Error> {
        let mut display_function = DisplayFunction {
            mode: Mode::Bits4,
            lines: Lines::Lines1,
            char_size: CharSize::Dots5x8,
        };

        if pins.data[0] != GpioPin::NONE {
            display_function.mode = Mode::Bits8;
        }

        let display_control = DisplayControl {
            display: DisplayState::On,
            cursor: CursorState::Off,
            blink: BlinkState::Off,
        };

        let display_mode = DisplayMode {
            entry_mode: DisplayEntryMode::Left,
            entry_shift_mode: DisplayEntryShiftMode::Decrement,
        };

        let mut chip = Chip::new("/dev/gpiochip0")?;

        let mut data_pins: [Option<LineHandle>; DATA_PINS] = Default::default();
        for i in 0..DATA_PINS {
            if pins.data[i] != GpioPin::NONE {
                let line = pins.data[i].line_handle(&mut chip, format!("data{}", i).as_str()).unwrap();
                data_pins[i] = Some(line);
            }
        }
        let pins = LineHandles {
            rs: pins.rs.line_handle(&mut chip, "rs")?,
            rw: pins.rw.map(|p| { p.line_handle(&mut chip, "rw").unwrap() }),
            enable: pins.enable.line_handle(&mut chip, "enable")?,
            data: data_pins,
        };

        Ok(LCD {
            pins,
            display_function,
            display_control,
            display_mode,
            row_offsets: [0x00; 4],
            num_lines: 1,
        })
    }

    /// Initializes the interface to the LCD screen, and specifies the dimensions (width and
    /// height) of the display. `begin()` needs to be called before any other LCD library commands.
    /// `cols` is the number of characters per line, `lines` is the number of lines,
    /// `char_size` is the size of the character matrix.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # let mut lcd = LCD::new(Pins {
    /// #     rs: P26,
    /// #     rw: None,
    /// #     enable: P19,
    /// #     data: [NONE, NONE, NONE, NONE, P13, P06, P05, P11],
    /// # })?;
    /// #
    /// lcd.begin(16, 2, CharSize::Dots5x8);
    /// ```
    pub fn begin(&mut self, cols: u8, lines: u8, char_size: CharSize) {
        if lines > 1 {
            self.display_function.lines = Lines::Lines2;
        }

        self.num_lines = lines;

        self.set_row_offsets(0x00, 0x40, 0x00 + cols, 0x40 + cols);

        if char_size != CharSize::Dots5x8 && lines == 1 {
            self.display_function.char_size = CharSize::Dots5x10;
        }

        // SEE PAGE 45/46 FOR INITIALIZATION SPECIFICATION!
        // according to datasheet, we need at least 40ms after power rises above 2.7V
        // before sending commands. Arduino can turn on way before 4.5V so we'll wait 50
        // TODO: Is the wait time for RPi different from Arduino?
        delay_micros(50000);
        self.pins.rs.write(GpioPinSignal::Low);
        self.pins.enable.write(GpioPinSignal::Low);
        if let Some(rw_pin) = &self.pins.rw {
            rw_pin.write(GpioPinSignal::Low);
        }

        // put the LCD into 4 bit or 8 bit mode
        if self.display_function.mode == Mode::Bits4 {
            // this is according to the hitachi HD44780 datasheet
            // figure 24, pg 46

            // we start in 8bit mode, try to set 4 bit mode
            self.write_4_bits(0x03);
            delay_micros(45000);

            // second try
            self.write_4_bits(0x03);
            delay_micros(4500); // wait min 4.1ms

            // third go!
            self.write_4_bits(0x03);
            delay_micros(150);

            // finally, set to 4-bit interface
            self.write_4_bits(0x02);
        } else {
            // this is according to the hitachi HD44780 datasheet
            // page 45 figure 23

            // Send function set command sequence
            self.command(Command::function_set(&self.display_function));
            delay_micros(4500);

            // second try
            self.command(Command::function_set(&self.display_function));
            delay_micros(150);

            // third go
            self.command(Command::function_set(&self.display_function));
        }

        // finally, set # lines, font size, etc.
        self.command(Command::function_set(&self.display_function));

        // turn the display on with no cursor or blinking default
        self.display_control.display = DisplayState::On;
        self.display_control.cursor = CursorState::Off;
        self.display_control.blink = BlinkState::Off;
        self.display();

        // clear it off
        self.clear();

        // Initialize to default text direction (for romance languages)
        self.display_mode.entry_mode = DisplayEntryMode::Left;
        self.display_mode.entry_shift_mode = DisplayEntryShiftMode::Decrement;

        // set the entry mode
        self.command(Command::entry_mode_set(&self.display_mode));
    }

    /// Position the LCD cursor
    ///
    /// That is, set the location at which subsequent text written to the LCD will be displayed.
    /// `col` is the column at which to position the cursor (with 0 being the first column)
    /// `row` is the row at which to position the cursor (with 0 being the first row).
    ///
    /// # Examples
    ///
    /// To position the cursor at the first column of the second line:
    ///
    /// ```rust,no_run
    /// # let mut lcd = LCD::new(Pins {
    /// #     rs: P26,
    /// #     rw: None,
    /// #     enable: P19,
    /// #     data: [NONE, NONE, NONE, NONE, P13, P06, P05, P11],
    /// # })?;
    /// #
    /// # lcd.begin(16, 2, CharSize::Dots5x8);
    /// lcd.set_cursor(0, 1);
    /// ```
    pub fn set_cursor(&self, col: u8, row: u8) {
        eprintln!("Settings cursor to: {},{}", col, row);

        let mut row = row;
        let max_rows = self.row_offsets.len().try_into().unwrap();

        if row >= max_rows {
            row = max_rows - 1;
        }

        if row >= self.num_lines {
            row = self.num_lines - 1;
        }

        self.command(Command::set_ddram_address(col + self.row_offsets[row as usize]));
    }

    /// Print text to the LCD
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # let mut lcd = LCD::new(Pins {
    /// #     rs: P26,
    /// #     rw: None,
    /// #     enable: P19,
    /// #     data: [NONE, NONE, NONE, NONE, P13, P06, P05, P11],
    /// # })?;
    /// #
    /// # lcd.begin(16, 2, CharSize::Dots5x8);
    /// lcd.print("Hello,  World!");
    /// ```
    pub fn print(&self, msg: &str) {
        eprintln!("Printing: {}", msg);

        msg.as_bytes().iter().for_each(|b| {
            self.write(*b);
        });
    }

    /// Clear the LCD screen and position the cursor in the upper-left corner
    pub fn clear(&self) {
        self.command(Command::clear_display());
        delay_micros(2000);
    }

    /// Position the cursor in the upper-left of the LCD
    ///
    /// That is, use that location in outputting subsequent text to the display. To also clear the
    /// display, use the [clear()](#method.clear) function instead.
    pub fn home(&self) {
        self.command(Command::return_home());
        delay_micros(2000);
    }

    /// Turn off the LCD display, without losing the text currently shown on it
    ///
    /// See also [display()](#method.display).
    pub fn no_display(&mut self) {
        self.display_control.display = DisplayState::Off;
        self.command(Command::display_control(&self.display_control));
    }

    /// Turn on the LCD display, after it's been turned off with [no_display()](#method.no_display)
    ///
    /// This will restore the text (and cursor) that was on the display.
    pub fn display(&mut self) {
        self.display_control.display = DisplayState::On;
        self.command(Command::display_control(&self.display_control));
    }

    /// Hide the LCD cursor
    ///
    /// See also [cursor](#method.cursor).
    pub fn no_cursor(&mut self) {
        self.display_control.cursor = CursorState::Off;
        self.command(Command::display_control(&self.display_control));
    }

    /// Display the LCD cursor: an underscore (line) at the position to which the next character
    /// will be written
    ///
    /// See also [no_cursor](#method.no_cursor).
    pub fn cursor(&mut self) {
        self.display_control.cursor = CursorState::On;
        self.command(Command::display_control(&self.display_control));
    }

    /// Turn off the blinking LCD cursor
    ///
    /// See also [blink()](#method.blink).
    pub fn no_blink(&mut self) {
        self.display_control.blink = BlinkState::Off;
        self.command(Command::display_control(&self.display_control));
    }

    /// Display the blinking LCD cursor
    ///
    /// If used in combination with the [cursor()](#method.cursor) function, the
    /// result will depend on the particular display.
    ///
    /// See also [no_blink()](#method.no_blink).
    pub fn blink(&mut self) {
        self.display_control.blink = BlinkState::On;
        self.command(Command::display_control(&self.display_control));
    }

    /// Scroll the contents of the display (text and cursor) one space to the left
    ///
    /// See also [scroll_display_right()](#method.scroll_display_right).
    pub fn scroll_display_left(&self) {
        self.command(Command::cursor_shift(&MoveControl::Display, &MoveDirection::Left));
    }

    /// Scroll the contents of the display (text and cursor) one space to the right
    ///
    /// See also [scroll_display_left](#method.scroll_display_left).
    pub fn scroll_display_right(&self) {
        self.command(Command::cursor_shift(&MoveControl::Display, &MoveDirection::Right));
    }

    /// Set the direction for text written to the LCD to left-to-right, the default
    ///
    /// This means that subsequent characters written to the display will go from left to right,
    /// but does not affect previously-output text.
    ///
    /// See also [right_to_left()](#method.right_to_left).
    pub fn left_to_right(&mut self) {
        self.display_mode.entry_mode = DisplayEntryMode::Left;
        self.command(Command::entry_mode_set(&self.display_mode));
    }

    /// Set the direction for text written to the LCD to right-to-left (the default is
    /// left-to-right)
    ///
    /// This means that subsequent characters written to the display will go from right to left,
    /// but does not affect previously-output text.
    ///
    /// See also [left-to-right()](#method.left_to_right).
    pub fn right_to_left(&mut self) {
        self.display_mode.entry_mode = DisplayEntryMode::Right;
        self.command(Command::entry_mode_set(&self.display_mode));
    }

    /// Turn on automatic scrolling of the LCD
    ///
    /// This causes each character output to the display to push previous characters over by one
    /// space. If the current text direction is left-to-right (the default), the display scrolls to
    /// the left; if the current direction is right-to-left, the display scrolls to the right. This
    /// has the effect of outputting each new character to the same location on the LCD.
    ///
    /// See also [no_autscroll()](#method.no_autscroll).
    pub fn autoscroll(&mut self) {
        self.display_mode.entry_shift_mode = DisplayEntryShiftMode::Increment;
        self.command(Command::entry_mode_set(&self.display_mode));
    }

    /// Turn off automatic scrolling of the LCD
    ///
    /// See also [autoscroll()](#method.autoscroll).
    pub fn no_autscroll(&mut self) {
        self.display_mode.entry_shift_mode = DisplayEntryShiftMode::Decrement;
        self.command(Command::entry_mode_set(&self.display_mode));
    }

    /// Create a custom character (glyph) for use on the LCD
    ///
    /// Up to eight characters of 5x8 pixels are supported (numbered 0 to 7). The appearance of
    /// each custom character is specified by an array of eight bytes, one for each row. The five
    /// least significant bits of each byte determine the pixels in that row. To display a custom
    /// character on the screen, [write()](#method.write) its number.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # let mut lcd = LCD::new(Pins {
    /// #     rs: P26,
    /// #     rw: None,
    /// #     enable: P19,
    /// #     data: [NONE, NONE, NONE, NONE, P13, P06, P05, P11],
    /// # })?;
    /// #
    /// # lcd.begin(16, 2, CharSize::Dots5x8);
    /// #
    ///
    /// let smiley = [
    ///     0b00000u8,
    ///     0b10001u8,
    ///     0b10001u8,
    ///     0b00000u8,
    ///     0b10001u8,
    ///     0b01110u8,
    ///     0b00000u8,
    ///     0b00000u8,
    /// ];
    /// let big_dot = [
    ///     0b00000u8,
    ///     0b01110u8,
    ///     0b11111u8,
    ///     0b11111u8,
    ///     0b11111u8,
    ///     0b01110u8,
    ///     0b00000u8,
    ///     0b00000u8,
    /// ];
    /// lcd.create_char(0, smiley);
    /// lcd.create_char(1, big_dot);
    /// lcd.write(0);
    /// lcd.set_cursor(3, 1);
    /// lcd.write(1);
    /// ```
    pub fn create_char(&self, location: u8, charmap: [u8; 8]) {
        let location = location & 0x7;
        self.command(Command::set_cgram_address(location << 3));
        charmap.iter().for_each(|b| {
            eprintln!("{:05b}", *b);
            self.write(*b);
        });
    }

    /// Write a character to the LCD
    pub fn write(&self, value: u8) {
        self.send(value, GpioPinSignal::High);
    }

    fn set_row_offsets(&mut self, row1: u8, row2: u8, row3: u8, row4: u8) {
        self.row_offsets[0] = row1;
        self.row_offsets[1] = row2;
        self.row_offsets[2] = row3;
        self.row_offsets[3] = row4;
    }

    fn command(&self, value: u8) {
        eprintln!("command: {:08b}", value);
        self.send(value, GpioPinSignal::Low);
    }

    fn send(&self, value: u8, signal: GpioPinSignal) {
        self.pins.rs.write(signal);

        if let Some(rw_pin) = &self.pins.rw {
            rw_pin.write(GpioPinSignal::Low);
        }

        if self.display_function.mode == Mode::Bits8 {
            self.write_8_bits(value);
        } else {
            self.write_4_bits(value >> 4);
            self.write_4_bits(value);
        }
    }

    fn pulse_enable(&self) {
        self.pins.enable.write(GpioPinSignal::Low);
        delay_micros(1);
        self.pins.enable.write(GpioPinSignal::High);
        delay_micros(1);
        self.pins.enable.write(GpioPinSignal::Low);
        delay_micros(100);
    }

    fn write_4_bits(&self, value: u8) {
        self.pins.data[4..8]
            .iter()
            .enumerate()
            .for_each(|(i, pin)| {
                pin.as_ref().unwrap().write(GpioPinSignal::from((value >> i) & 0x01));
            });

        self.pulse_enable();
    }

    fn write_8_bits(&self, value: u8) {
        self.pins.data.iter().enumerate().for_each(|(i, pin)| {
            pin.as_ref().unwrap().write(GpioPinSignal::from((value >> i) & 0x01));
        });

        self.pulse_enable();
    }
}
