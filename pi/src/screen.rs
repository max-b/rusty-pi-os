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
    pub hue: f32,
}

/// Global `SCREEN` singleton.
pub static SCREEN: Mutex<Screen> = Mutex::new(Screen::new());

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
            },
            hue: 0f32,
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
        self.draw_char_scale(c, 1);
    }

    pub fn draw_char_scale(&mut self, c: u8, scale: usize) {
        if c == 0x0d || c == 0x0a {
            self.position.y = self.position.y.wrapping_add(10 * scale);
            let default_position: Position = Default::default();
            self.position.x = default_position.x;
            return;
        }

        self.hue += 0.05f32;
        if self.hue > 1f32 {
            self.hue -= 1f32;
        }

        match ascii_to_glyph(c) {
            Some(glyph) => {

                let mut pixel = Pixel {
                    position: self.position,
                    color: Screen::hsl_to_rgb(self.hue, 1f32, 0.5f32),
                };

                if pixel.position.y >= self.inner().height - 10 * scale {
                    self.position.y = 0;
                }

                for row in glyph.iter() {
                    for i in 0..5 {
                        if row << i & 0b10000u8 != 0u8 {
                            for j in 0..scale {
                                for k in 0..scale {
                                    self.inner().draw_pixel(&Pixel {
                                        position: Position {
                                            x: pixel.position.x + j,
                                            y: pixel.position.y + k,
                                        },
                                        color: pixel.color
                                    });
                                }
                            }
                        } else {
                            for j in 0..scale {
                                for k in 0..scale {
                                    self.inner().draw_pixel(&Pixel {
                                        position: Position {
                                            x: pixel.position.x + j,
                                            y: pixel.position.y + k,
                                        },
                                        color: Default::default()
                                    });
                                }
                            }
                        }
                        pixel.position.x = pixel.position.x.wrapping_add(scale);
                    }
                    pixel.position.x = self.position.x;
                    pixel.position.y = pixel.position.y.wrapping_add(scale);
                }

                self.position.x = self.position.x.wrapping_add(6 * scale);

                if self.position.x > self.inner().width {
                    self.position.x = 0;
                    self.position.y = self.position.y.wrapping_add(10 * scale);
                }
            },
            None => {}
        }
    }

    pub fn draw_string_scale(&mut self, s: &str, scale: usize) {
        for c in s.bytes() {
            self.draw_char_scale(c, scale);
        }
    }

    pub fn draw_string(&mut self, s: &str) {
        self.draw_string_scale(s, 1);
    }

    pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
        let mut r = 0f32;
        let mut g = 0f32;
        let mut b = 0f32;

        if (s == 0f32) {
            r = l;
            g = l;
            b = l; // achromatic
        } else {
            let q: f32 = if l < 0.5f32 { l * (1f32 + s) } else { l + s - l * s };
            let p: f32 = 2f32 * l - q;
            r = Screen::hue_to_rgb(p, q, h + 1f32/3f32);
            g = Screen::hue_to_rgb(p, q, h);
            b = Screen::hue_to_rgb(p, q, h - 1f32/3f32);
        }

        Color {
            red: (r * 255f32) as u8,
            green: (g * 255f32) as u8,
            blue: (b * 255f32) as u8,
        }
    }

    pub fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
        if (t < 0f32) {
            t += 1f32;
        }
        if (t > 1f32) {
            t -= 1f32;
        }
        if (t < 1f32 / 6f32) {
            return p + (q - p) * 6f32 * t;
        }
        if (t < 1f32 / 2f32) {
            return q;
        }
        if (t < 2f32 / 3f32) {
            return p + (q - p) * (2f32 /3f32 - t) * 6f32;
        }
        return p;
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
