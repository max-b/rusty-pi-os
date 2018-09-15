use std::io;
use pi::timer;
use pi::mutex::Mutex;
use fat32::traits::BlockDevice;

extern "C" {
    /// A global representing the last SD controller error that occured.
    static sd_err: i64;

    /// Initializes the SD card controller.
    ///
    /// Returns 0 if initialization is successful. If initialization fails,
    /// returns -1 if a timeout occured, or -2 if an error sending commands to
    /// the SD controller occured.
    fn sd_init() -> i32;

    /// Reads sector `n` (512 bytes) from the SD card and writes it to `buffer`.
    /// It is undefined behavior if `buffer` does not point to at least 512
    /// bytes of memory.
    ///
    /// On success, returns the number of bytes read: a positive number.
    ///
    /// On error, returns 0. The true error code is stored in the `sd_err`
    /// global. `sd_err` will be set to -1 if a timeout occured or -2 if an
    /// error sending commands to the SD controller occured. Other error codes
    /// are also possible but defined only as being less than zero.
    fn sd_readsector(n: i32, buffer: *mut u8) -> i32;
}

#[no_mangle]
pub fn wait_micros(us: u32) {
    timer::spin_sleep_us(us.into());
}

#[derive(Debug)]
pub enum Error {
    Timeout(i64),
    CommandError(i64),
    UnknownReturnCode(i64),
}

pub static SD_ERR: Mutex<&i64> = unsafe { Mutex::new(&sd_err) };

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

impl Sd {
    // /// Initializes the SD card controller and returns a handle to it.
    pub fn new() -> Result<Sd, Error> {
        match unsafe { sd_init() } {
            0 => Ok(Sd),
            -1 => Err(Error::Timeout(**SD_ERR.lock())),
            -2 => Err(Error::CommandError(**SD_ERR.lock())),
            _ => Err(Error::UnknownReturnCode(**SD_ERR.lock())),
        }
    }
}

impl BlockDevice for Sd {
    /// Reads sector `n` from the SD card into `buf`. On success, the number of
    /// bytes read is returned.
    ///
    /// # Errors
    ///
    /// An I/O error of kind `InvalidInput` is returned if `buf.len() < 512` or
    /// `n > 2^31 - 1` (the maximum value for an `i32`).
    ///
    /// An error of kind `TimedOut` is returned if a timeout occurs while
    /// reading from the SD card.
    ///
    /// An error of kind `Other` is returned for all other errors.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 512 || n > (1 << 32) - 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buf.len() or n is out of bounds"));
        }

        let ret = unsafe { sd_readsector(n as i32, buf as *mut [u8] as *mut u8) };

        if ret > 0 {
            Ok(ret as usize)
        } else {
            match **SD_ERR.lock() {
                _ if ret == -1 => Err(io::Error::new(io::ErrorKind::TimedOut, "timed out in read_sector")),
                _ if ret == -2 => Err(io::Error::new(io::ErrorKind::Other, "error sending commands to the sd controller")),
                _ => Err(io::Error::new(io::ErrorKind::Other, "unknown error in read_sector")),
            }
        }
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("SD card and file system are read only")
    }
}
