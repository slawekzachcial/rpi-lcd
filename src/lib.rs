// use gpio_cdev::*;

#[derive(Debug)]
pub struct GpioPins {
    pub rs: u8,
    pub rw: Option<u8>,
    pub enable: u8,
    pub d0123: [u8; 4],
    pub d4567: Option<[u8; 4]>,
}

#[derive(Debug)]
pub struct LCD {
    pins: GpioPins,
}

impl LCD {
    pub fn new(pins: GpioPins) -> LCD {
        LCD { pins }
    }

    pub fn set_cursor(&self, col: u8, row: u8) {
        println!("[{:?}] Settings cursor to: {},{}", self, col, row);
    }

    pub fn print(&self, msg: &str) {
        println!("[{:?}] Printing: {}", self, msg);
    }
}
