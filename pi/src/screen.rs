use std::io;
use std::fmt;

use framebuffer::{Color, Position, Pixel, Framebuffer};
use character_set::{TELETEXT, ascii_to_glyph};
use mutex::Mutex;

/// A global singleton allowing read/write access to the screen.
pub struct Screen {
    inner: Option<Framebuffer>,
    position: Position,
    // TODO: make methods for this
    pub color: Color,
}

impl Screen {
    /// Creates a new instance of `Screen`.
    const fn new() -> Screen {
        Screen { 
            inner: None,
            position: Position {
                x: 0,
                y: 0,
            },
            color: Color {
                red: 0x00,
                green: 0xff,
                blue: 0x00,
            }
        }
    }

    /// Initializes the screen if it's not already initialized.
    #[inline]
    fn initialize(&mut self) {
        if self.inner.is_none() {
            self.inner = Some(Framebuffer::new().expect("error creating new framebuffer"));
        }
    }

    /// Returns a mutable borrow to the inner `Framebuffer`, initializing it as
    /// needed.
    pub fn inner(&mut self) -> &mut Framebuffer {
        self.initialize();
        self.inner.as_mut().unwrap()
    }

    pub fn clear(&mut self) {
        self.inner().clear();
        self.position = Default::default();
    }

    pub fn draw_char(&mut self, c: u8) {
        if c == 0x0d {
            self.position.y = self.position.y.wrapping_add(10);
            let default_position: Position = Default::default();
            self.position.x = default_position.x;
            return;
        }

        match ascii_to_glyph(c) {
            Some(glyph) => {

                let mut pixel = Pixel {
                    position: self.position,
                    color: self.color,
                };

                for row in glyph.iter() {
                    for i in 0..5 {
                        if row << i & 0b10000u8 != 0u8 {
                            self.inner().draw_pixel(&Pixel {
                                position: pixel.position,
                                color: pixel.color
                            });
                        }
                        pixel.position.x = pixel.position.x.wrapping_add(1);
                    }
                    pixel.position.x = self.position.x;
                    pixel.position.y = pixel.position.y.wrapping_add(1);
                }

                self.position.x = self.position.x.wrapping_add(6);

                if self.position.x > self.inner().width {
                    self.position.x = 0;
                    self.position.y = self.position.y.wrapping_add(10);
                }
            },
            None => {}
        }
    }

    pub fn draw_string(&mut self, s: &str) {
        for c in s.bytes() {
            self.draw_char(c);
        }
    }
}

// impl io::Write for Screen {
//     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//         self.inner().write_str(buf)
//     }

//     fn flush(&mut self) -> io::Result<()> {
//         Ok(())
//     }
// }

// impl fmt::Write for Screen {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         self.inner().write_str(s)
//     }
// }

/// Global `SCREEN` singleton.
pub static SCREEN: Mutex<Screen> = Mutex::new(Screen::new());
