use draw::draw_loop;
use fat32::traits::{self, Dir, Entry};
use fs::FileSystem;
use pi::common::{
    ARM_POWER_MANAGEMENT_FULL_RESET, ARM_POWER_MANAGEMENT_PASSWD, ARM_POWER_MANAGEMENT_RSTC,
    ARM_POWER_MANAGEMENT_WDOG,
};
use pi::console::{kprint, kprintln, CONSOLE};
use pi::raccoon::RACCOON_STRING;
use pi::screen::SCREEN;
use stack_vec::StackVec;
use std::io::Read;
use std::str;
use volatile::prelude::*;
use volatile::WriteVolatile;

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();
const BOOTLOADER_START_ADDR: usize = 0x4000000;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }

    fn process(&self) {
        match self.path() {
            "echo" => {
                let mut iter = self.args.iter();
                iter.next(); // skip over path
                for arg in iter {
                    kprint!("{} ", arg);
                }
                kprintln!("");
            }
            "draw" => {
                draw_loop();
            }
            "raccoon" => {
                SCREEN.lock().draw_string(&RACCOON_STRING);
                SCREEN.lock().draw_char(0x0d);
            }
            "cat" => {
                let mut iter = self.args.iter();
                iter.next(); // skip over path
                for arg in iter {
                    match traits::FileSystem::open_file(&FILE_SYSTEM, arg) {
                        Ok(mut file) => {
                            let mut buf = vec![0; 100];
                            match file.read(&mut buf[..]) {
                                Ok(_bytes_read) => {
                                    SCREEN.lock().draw_string(&file.metadata.name);
                                    SCREEN.lock().draw_char(0x0d);
                                    SCREEN
                                        .lock()
                                        .draw_string(&String::from_utf8_lossy(&buf[..]));
                                    SCREEN.lock().draw_char(0x0d);
                                }
                                Err(error) => {
                                    kprintln!(
                                        "Error reading file {}: {:#?}",
                                        &file.metadata.name,
                                        error
                                    );
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
            "ls" => {
                let mut iter = self.args.iter();
                iter.next(); // skip over path
                for arg in iter {
                    match traits::FileSystem::open_dir(&FILE_SYSTEM, arg) {
                        Ok(dir) => {
                            for entry in dir.entries().expect("iter") {
                                SCREEN.lock().draw_string(&entry.name());
                                SCREEN.lock().draw_char(0x0d);
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
            "clear" => {
                SCREEN.lock().clear();
            }
            "print" => {
                let mut iter = self.args.iter();
                iter.next();
                if let Some(scale) = iter.next() {
                    for arg in iter {
                        SCREEN
                            .lock()
                            .draw_string_scale(&arg, scale.parse::<usize>().unwrap_or(1));
                        SCREEN
                            .lock()
                            .draw_char_scale(0x20, scale.parse::<usize>().unwrap_or(1));
                    }
                    SCREEN
                        .lock()
                        .draw_char_scale(0x0d, scale.parse::<usize>().unwrap_or(1));
                }
            }
            "reboot" => unsafe {
                let watchdog_register: &mut WriteVolatile<u32> =
                    &mut *(ARM_POWER_MANAGEMENT_WDOG as *mut WriteVolatile<u32>);
                let reset_register: &mut WriteVolatile<u32> =
                    &mut *(ARM_POWER_MANAGEMENT_RSTC as *mut WriteVolatile<u32>);
                watchdog_register.write(ARM_POWER_MANAGEMENT_PASSWD | 1);
                reset_register.write(ARM_POWER_MANAGEMENT_PASSWD | ARM_POWER_MANAGEMENT_FULL_RESET);
            },
            "help" => unsafe {
                asm!("br $0" : : "r"(BOOTLOADER_START_ADDR as usize));
            },
            _ => {
                kprintln!("unknown command: {}", self.path());
            }
        }
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) {
    let mut raw_buffer = [0u8; 512];
    let mut buffer = StackVec::new(&mut raw_buffer);
    let parsed_cmd: [&str; 64] = [""; 64];

    SCREEN.lock().clear();
    SCREEN.lock().draw_string_scale(&"WELCOME TO MaxOS,5", 5);
    SCREEN.lock().draw_char_scale(0x0d, 5);
    loop {
        kprint!("{}", prefix);

        // read until a full command (+ newline) has been written
        loop {
            let byte = CONSOLE.lock().read_byte();
            if byte == b'\n' || byte == b'\r' {
                break;
            }
            // don't automatically process the instruction when the max
            // length is reached. instead, wait until newline
            if buffer.is_full() {
                continue;
            }
            if byte == 8 || byte == 127 {
                // backspace
                if !buffer.is_empty() {
                    kprint!("{} {}", byte as char, byte as char);
                    buffer.pop();
                }
            } else {
                kprint!("{}", byte as char);
                buffer.push(byte).expect("buffer is full!");
            }
        }

        kprintln!("");
        if let Ok(s) = str::from_utf8(&buffer.as_slice()) {
            match Command::parse(s, &mut { parsed_cmd }) {
                Ok(cmd) => {
                    cmd.process();
                }
                Err(Error::TooManyArgs) => {
                    kprintln!("error: too many arguments");
                }
                Err(Error::Empty) => {}
            };
        } else {
            kprint!("{}", 7); // sound bell for unrecognized character
        }

        buffer.truncate(0);
    }
}
