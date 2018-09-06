use std::fmt;
use volatile::prelude::*;
use volatile::{Volatile};
use console::kprintln;
use propertytag::{send_tags, PropertyTag, PropertyId};
use stack_vec::StackVec;

// Many thanks to 
// https://elinux.org/RPi_Framebuffer
// and
// https://github.com/raspberrypi/firmware/wiki/Mailbox
// for info on address locations, etc

pub struct Framebuffer {
    pub size: usize,
    pub buffer: &'static mut[Volatile<u8>]
}

impl Framebuffer {
    pub fn new() -> Result<Framebuffer, ()> {

        let physical_width_height_tag = PropertyTag::new(PropertyId::SetPhysicalWidthHeight);
        let virtual_width_height_tag = PropertyTag::new(PropertyId::SetVirtualWidthHeight);
        let set_depth_tag = PropertyTag::new(PropertyId::SetDepth);
        let allocate_buffer_tag = PropertyTag::new(PropertyId::AllocateBuffer);

        kprintln!("Allocate Buffer Property Tag:");

        let mut tag_backing: [PropertyTag; 8] = [Default::default(); 8];
        let mut tags = StackVec::new(&mut tag_backing);
        tags.push(physical_width_height_tag)?;
        tags.push(virtual_width_height_tag)?;
        tags.push(set_depth_tag)?;
        tags.push(allocate_buffer_tag)?;
        let tags = send_tags(&mut tags);

        let allocate_buffer_tag = tags.pop().unwrap();
        let set_depth_tag = tags.pop().unwrap();
        let virtual_width_height_tag = tags.pop().unwrap();
        let physical_width_height_tag = tags.pop().unwrap();

        kprintln!("physical_width_height_tag data: {:x?}", &physical_width_height_tag.data[..]);
        kprintln!("virtual_width_height_tag data: {:x?}", &virtual_width_height_tag.data[..]);
        kprintln!("set_depth_tag data: {:x?}", &set_depth_tag.data[..]);
        kprintln!("allocate_buffer_tag data: {:x?}", &allocate_buffer_tag.data[..]);


        let fb_base_addr = (allocate_buffer_tag.data[0] - 0xc0000000) as *mut Volatile<u8>;
        let size = allocate_buffer_tag.data[1] as usize;

        kprintln!("fb_base_addr = {:x}", fb_base_addr as u32);
        kprintln!("size = {:x}", size);

        let buffer = unsafe { ::core::slice::from_raw_parts_mut(fb_base_addr, size) };

        Ok(Framebuffer {
            size,
            buffer,
        })

    }
}

impl fmt::Display for Framebuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Size: {}, pixel 1: ({}, {}, {})", self.size, self.buffer[0].read(), self.buffer[1].read(), self.buffer[2].read())
    }
}
