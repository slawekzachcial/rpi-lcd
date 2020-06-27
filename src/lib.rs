use gpio_cdev::*;
use std::{thread, time};
use std::ops::BitOr;
use std::convert::TryInto;

fn delay_micros(micros: u64) {
    thread::sleep(time::Duration::from_micros(micros));
}

const DATA_PINS: usize = 8;

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
    CursorShiftLeft = 0x10,
    FunctionSet = 0x20,
    SetCGRamAddress = 0x40,
    SetDDRamAddress = 0x80,
}

impl Command {
    fn clear_display() -> u8 {
        Command::ClearDisplay as u8
    }

    fn function_set(df: &DisplayFunction) -> u8 {
        Command::FunctionSet as u8 | df.mode as u8 | df.lines as u8 | df.char_size as u8
    }

    fn display_control(dc: &DisplayControl) -> u8 {
        Command::DisplayControl as u8 | dc.display as u8 | dc.cursor as u8 | dc.blink as u8
    }

    fn entry_mode_set(dm: &DisplayMode) -> u8 {
        Command::EntryModeSet as u8 | dm.entry_mode as u8 | dm.entry_shift_mode as u8
    }

    fn set_ddram_address(address: u8) -> u8 {
        Command::SetDDRamAddress as u8 | address
    }
}

// impl BitOr<&DisplayFunction> for Command {
//     type Output = u8;

//     fn bitor(self, rhs: &DisplayFunction) -> u8 {
//         self as u8 | rhs.mode as u8 | rhs.lines as u8 | rhs.char_size as u8
//     }
// }

// impl BitOr<&DisplayControl> for Command {
//     type Output = u8;

//     fn bitor(self, rhs: &DisplayControl) -> u8 {
//         self as u8 | rhs.display as u8 | rhs.cursor as u8 | rhs.blink as u8
//     }
// }

// impl BitOr<&DisplayMode> for Command {
//     type Output = u8;

//     fn bitor(self, rhs: &DisplayMode) -> u8 {
//         self as u8 | rhs.entry_mode as u8 | rhs.entry_shift_mode as u8
//     }
// }

// impl BitOr<u8> for Command {
//     type Output = u8;

//     fn bitor(self, rhs: u8) -> u8 {
//         self as u8 | rhs
//     }
// }

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

#[derive(Debug)]
enum MoveDirection {
    Right = 0x04,
    Left = 0x00,
}

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

#[derive(Debug)]
pub struct Pins {
    pub rs: GpioPin,
    pub rw: Option<GpioPin>,
    pub enable: GpioPin,
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

pub struct LCD {
    pins: LineHandles,
    display_function: DisplayFunction,
    display_control: DisplayControl,
    display_mode: DisplayMode,
    row_offsets: [u8; 4],
    num_lines: u8,
}

impl LCD {
    pub fn new(pins: Pins) -> Result<LCD, errors::Error> {
        let mut display_function = DisplayFunction {
            mode: Mode::Bits4,
            lines: Lines::Lines1,
            char_size: CharSize::Dots5x8,
        };

        if pins.data[4] != GpioPin::NONE {
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
            // self.command(Command::FunctionSet | &self.display_function);
            self.command(Command::function_set(&self.display_function));
            delay_micros(4500);

            // second try
            // self.command(Command::FunctionSet | &self.display_function);
            self.command(Command::function_set(&self.display_function));
            delay_micros(150);

            // third go
            // self.command(Command::FunctionSet | &self.display_function);
            self.command(Command::function_set(&self.display_function));
        }

        // finally, set # lines, font size, etc.
        // self.command(Command::FunctionSet | &self.display_function);
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
        // self.command(Command::EntryModeSet | &self.display_mode);
        self.command(Command::entry_mode_set(&self.display_mode));
    }

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

        // self.command(Command::SetDDRamAddress | (col + self.row_offsets[row as usize]));
        self.command(Command::set_ddram_address(col + self.row_offsets[row as usize]));
    }

    pub fn print(&self, msg: &str) {
        eprintln!("Printing: {}", msg);

        msg.as_bytes().iter().for_each(|b| {
            self.write(*b);
        });
    }

    fn clear(&self) {
        // self.command(Command::ClearDisplay as u8);
        self.command(Command::clear_display());
        delay_micros(2000);
    }

    fn display(&mut self) {
        self.display_control.display = DisplayState::On;
        // self.command(Command::DisplayControl | &self.display_control);
        self.command(Command::display_control(&self.display_control));
    }

    fn set_row_offsets(&mut self, row1: u8, row2: u8, row3: u8, row4: u8) {
        self.row_offsets[0] = row1;
        self.row_offsets[1] = row2;
        self.row_offsets[2] = row3;
        self.row_offsets[3] = row4;
    }

    fn command(&self, value: u8) {
        self.send(value, GpioPinSignal::Low);
    }

    fn write(&self, value: u8) {
        self.send(value, GpioPinSignal::High);
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
        self.pins.data[0..4]
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
