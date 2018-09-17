use pi::console::CONSOLE;
use pi::framebuffer::Pixel;
use pi::screen::SCREEN;

/// Starts a draw mode
pub fn draw_loop() {
    let mut pixel_cursor: Pixel = Default::default();

    loop {
        let byte = {
            let mut console = CONSOLE.lock();
            console.read_byte()
        };

        if byte == 0x1b {
            SCREEN.lock().inner().clear();
        }

        if byte == 0x60 {
            break;
        }

        if byte == 0x61 {
            pixel_cursor.position.x = pixel_cursor.position.x.wrapping_sub(1);
        }
        if byte == 0x64 {
            pixel_cursor.position.x = pixel_cursor.position.x.wrapping_add(1);
        }
        if byte == 0x73 {
            pixel_cursor.position.y = pixel_cursor.position.y.wrapping_add(1);
        }
        if byte == 0x77 {
            pixel_cursor.position.y = pixel_cursor.position.y.wrapping_sub(1);
        }
        if byte == 0x31 {
            pixel_cursor.color.red = pixel_cursor.color.red.wrapping_add(10);
        }
        if byte == 0x32 {
            pixel_cursor.color.green = pixel_cursor.color.green.wrapping_add(10);
        }
        if byte == 0x33 {
            pixel_cursor.color.blue = pixel_cursor.color.blue.wrapping_add(10);
        }

        SCREEN.lock().inner().draw_pixel(&pixel_cursor);
    }
}
