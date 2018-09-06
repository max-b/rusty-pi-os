use volatile::prelude::*;
use volatile::{Volatile, WriteVolatile, ReadVolatile, Reserved};
use console::kprintln;
use mailbox::{Mailbox, Channel};
use propertytag::{u32_from_buffer, u32_to_buffer, PropertyTag, PropertyId};

// Many thanks to 
// https://elinux.org/RPi_Framebuffer
// and
// https://github.com/raspberrypi/firmware/wiki/Mailbox
// for info on address locations, etc

#[derive(Default, Debug)]
#[repr(C)]
pub struct FramebufferData {
    width: u32,
    height: u32,
    virtual_width: u32,
    virtual_height: u32,
    pitch: u32,
    depth: u32,
    x_offset: u32,
    y_offset: u32,
    pointer: u32,
    size: u32,
}

#[repr(align(16))] // TODO: Check this is padding correctly
pub struct BufferWrapper {
    buffer: [u8; (1 << 14)],
}

static mut TAG_BUFFER: BufferWrapper = BufferWrapper {
    buffer: [0x00; (1 << 14)]
};

pub struct Framebuffer {
    pub size: usize,
    pub buffer: &'static mut [u8]
}

impl Framebuffer {
    pub fn new() -> Framebuffer {
        let mut mailbox = Mailbox::new(Channel::PropertyTagsARMTOVC);

        let physical_width_height_tag = PropertyTag::new(PropertyId::SetPhysicalWidthHeight);
        let virtual_width_height_tag = PropertyTag::new(PropertyId::SetVirtualWidthHeight);
        let set_depth_tag = PropertyTag::new(PropertyId::SetDepth);
        let allocate_buffer_tag = PropertyTag::new(PropertyId::AllocateBuffer);

        kprintln!("Allocate Buffer Property Tag:");

        let mut buf = [0u8; 300];
        kprintln!("{:?}", allocate_buffer_tag.to_bytes(&mut buf));
        kprintln!("{:?}", &buf[..]);

        let allocate_buffer_tag = PropertyTag::from_bytes(&buf);
        kprintln!("Property tag id: {:?}", allocate_buffer_tag.id);
        kprintln!("Property tag byte length: {:?}", allocate_buffer_tag.byte_length);
        kprintln!("Property tag data: {:?}", &allocate_buffer_tag.data[..]);

        unsafe {
            kprintln!("Into unsafe territory....");
            kprintln!("Empty buffer: {:?}", &TAG_BUFFER.buffer[..300]);
            let mut index = 8;
            let tags = [physical_width_height_tag, virtual_width_height_tag, set_depth_tag, allocate_buffer_tag];
            for tag in tags.iter() {
                index += tag.to_bytes(&mut TAG_BUFFER.buffer[index..]);
                kprintln!("Added tag {}", index);
            }
            u32_to_buffer(0, &mut TAG_BUFFER.buffer[index..]); // End tag
            index += 4;
            u32_to_buffer(index as u32, &mut TAG_BUFFER.buffer); // Size
            u32_to_buffer(0, &mut TAG_BUFFER.buffer[4..]); // Request code

            kprintln!("{:?}", &TAG_BUFFER.buffer[..300]);
            mailbox.send(((&mut TAG_BUFFER.buffer as *mut [u8]) as *mut u8) as u32);

            let mailbox_response = mailbox.receive().expect("Error in mailbox reception");

            kprintln!("Received {:x} from mailbox in framebuffer", mailbox_response);

            kprintln!("{:?}", &TAG_BUFFER.buffer[..300]);

            let mut bytes_read = 0;
            let buffer_size = u32_from_buffer(&TAG_BUFFER.buffer[bytes_read..]);
            bytes_read += 4;
            let response = u32_from_buffer(&TAG_BUFFER.buffer[bytes_read..]);
            bytes_read += 4;

            let physical_width_height_tag = PropertyTag::from_bytes(&TAG_BUFFER.buffer[bytes_read..]);
            bytes_read += physical_width_height_tag.length();

            let virtual_width_height_tag = PropertyTag::from_bytes(&TAG_BUFFER.buffer[bytes_read..]);
            bytes_read += virtual_width_height_tag.length();

            let set_depth_tag = PropertyTag::from_bytes(&TAG_BUFFER.buffer[bytes_read..]);
            bytes_read += set_depth_tag.length();

            let allocate_buffer_tag = PropertyTag::from_bytes(&TAG_BUFFER.buffer[bytes_read..]);
            bytes_read += allocate_buffer_tag.length();

            kprintln!("physical_width_height_tag data: {:?}", &physical_width_height_tag.data[..]);
            kprintln!("virtual_width_height_tag data: {:?}", &virtual_width_height_tag.data[..]);
            kprintln!("set_depth_tag data: {:?}", &set_depth_tag.data[..]);
            kprintln!("allocate_buffer_tag data: {:?}", &allocate_buffer_tag.data[..]);


            let fb_base_addr = (allocate_buffer_tag.data[0] - 0xc0000000) as *mut u8;
            let size = allocate_buffer_tag.data[1] as usize;

            kprintln!("fb_base_addr = {:x}", fb_base_addr as u32);
            kprintln!("size = {:x}", size);

            let buffer = ::core::slice::from_raw_parts_mut(fb_base_addr, size);

            Framebuffer {
                size,
                buffer,
            }
        }

    }
}
