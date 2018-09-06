use volatile::prelude::*;
use volatile::{Volatile};
use console::kprintln;
use stack_vec::StackVec;
use mailbox::{Mailbox, Channel};

#[repr(align(16))]
pub struct TagBufferStorage {
    buffer: [u8; (1 << 10)],
}

static mut TAG_BUFFER_STORAGE: TagBufferStorage = TagBufferStorage {
    buffer: [0x00u8; (1 << 10)]
};

#[repr(align(16))]
pub struct TagBuffer {
    buffer: [Volatile<u8>; (1 << 10)],
}

// From https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface
#[derive(Debug)]
#[repr(u32)]
pub enum PropertyId {
    GetFirmwareRevision = 0x00000001,
    GetBoardModel = 0x00010001,
    GetBoardRevision = 0x00010002,
    GetBoardMac = 0x00010003,
    GetBoardSerial = 0x00010004,
    GetArmMemory = 0x00010005,
    GetVcMemory = 0x00010006,
    GetClocks = 0x00010007,
    GetCommandline = 0x00050001,
    GetDmachannels = 0x00060001,
    GetPowerstate = 0x00020001,
    GetTiming = 0x00020002,
    SetPowerState = 0x00028001,
    GetClockState = 0x00030001,
    SetClockState = 0x00038001,
    GetClockRate = 0x00030002,
    SetClockRate = 0x00038002,
    GetMaxClockRate = 0x00030004,
    GetMinClockRate = 0x00030007,
    GetTurbo = 0x00030009,
    SetTurbo = 0x00038009,
    GetVoltage = 0x00030003,
    SetVoltage = 0x00038003,
    GetMaxVoltage = 0x00030005,
    GetMinVoltage = 0x00030008,
    GetTemperature = 0x00030006,
    GetMaxtemperature = 0x0003000a,
    AllocateMemory = 0x0003000c,
    LockMemory = 0x0003000d,
    UnlockMemory = 0x0003000e,
    ReleaseMemory = 0x0003000f,
    ExecuteCode = 0x00030010,
    GetDispmanxResourceMemHandle = 0x00030014,
    GetEdidBlock = 0x00030020,
    AllocateBuffer = 0x00040001,
    ReleaseBuffer = 0x00048001,
    BlankScreen = 0x00040002,
    GetPhysicalWidthHeight = 0x00040003,
    TestPhysicalWidthHeight = 0x00044003,
    SetPhysicalWidthHeight = 0x00048003,
    GetVirtualWidthHeight = 0x00040004,
    TestVirtualWidthHeight = 0x00044004,
    SetVirtualWidthHeight = 0x00048004,
    GetDepth = 0x00040005,
    TestDepth = 0x00044005,
    SetDepth = 0x00048005,
    GetPixelOrder = 0x00040006,
    TestPixelOrder = 0x00044006,
    SetPixelOrder = 0x00048006,
    GetAlphaMode = 0x00040007,
    TestAlphaMode = 0x00044007,
    SetAlphaMode = 0x00048007,
    GetPitch = 0x00040008,
    GetVirtualOffset = 0x00040009,
    TestVirtualOffset = 0x00044009,
    SetVirtualOffset = 0x00048009,
    GetOverscan = 0x0004000a,
    TestOverscan = 0x0004400a,
    SetOverscan = 0x0004800a,
    GetPalette = 0x0004000b,
    TestPalette = 0x0004400b,
    SetPalette = 0x0004800b,
    SetCursorInfo = 0x00008010,
    SetCursorState = 0x00008011,
    Unimplemented = 0x00,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PropertyTag {
    pub id: u32,
    pub byte_length: u32,
    pub tag_type: u32,
    pub data: [u32; 32]
}

pub fn u32_to_buffer(i: u32, buffer: &mut [Volatile<u8>]) {
    buffer[3].write((i >> 24) as u8);
    buffer[2].write((i >> 16) as u8);
    buffer[1].write((i >> 8) as u8);
    buffer[0].write(i as u8);
}

pub fn u32_from_buffer(buffer: &[Volatile<u8>]) -> u32 {
    ((buffer[3].read() as u32) << 24) | ((buffer[2].read() as u32) << 16) | ((buffer[1].read() as u32) << 8) | (buffer[0].read() as u32)
}

impl Default for PropertyTag {
    fn default() -> PropertyTag {
        PropertyTag {
            id: 0,
            byte_length: 0,
            tag_type: 0,
            data: [0; 32]
        }
    }
}

impl PropertyTag {
    pub fn new(property_id: PropertyId) -> PropertyTag {
        match property_id {
            PropertyId::AllocateBuffer => {
                let mut data = [0x00; 32];
                data[0] = 1;
                PropertyTag {
                    id: property_id as u32,
                    byte_length: 8,
                    tag_type: 0x00, // Request
                    data
                }
            },
            PropertyId::SetPhysicalWidthHeight => {
                let mut data = [0x00; 32];
                data[0] = 320;
                data[1] = 240;
                PropertyTag {
                    id: property_id as u32,
                    byte_length: 8,
                    tag_type: 0x00, // Request
                    data
                }
            },
            PropertyId::SetVirtualWidthHeight => {
                let mut data = [0x00; 32];
                data[0] = 320;
                data[1] = 240;
                PropertyTag {
                    id: property_id as u32,
                    byte_length: 8,
                    tag_type: 0x00, // Request
                    data
                }
            },
            PropertyId::SetDepth => {
                let mut data = [0x00; 32];
                data[0] = 24;
                PropertyTag {
                    id: property_id as u32,
                    byte_length: 4,
                    tag_type: 0x00, // Request
                    data
                }
            },
            PropertyId::GetPhysicalWidthHeight => {
                let data = [0x00; 32];
                PropertyTag {
                    id: property_id as u32,
                    byte_length: 8,
                    tag_type: 0x00, // Request
                    data
                }
            },
            PropertyId::GetPitch => {
                let data = [0x00; 32];
                PropertyTag {
                    id: property_id as u32,
                    byte_length: 4,
                    tag_type: 0x00, // Request
                    data
                }
            },
            PropertyId::GetDepth => {
                let data = [0x00; 32];
                PropertyTag {
                    id: property_id as u32,
                    byte_length: 4,
                    tag_type: 0x00, // Request
                    data
                }
            },
            _ => { // TODO: Unimplemented!
                PropertyTag {
                    id: PropertyId::Unimplemented as u32,
                    byte_length: 0,
                    tag_type: 0x00, // Request
                    data:[0x00; 32]
                }
            }
        }
    }

    pub fn to_bytes(&self, buffer: &mut [Volatile<u8>]) -> usize {
        let mut num_written = 0;
        u32_to_buffer(self.id, &mut buffer[..]);
        num_written += 4;
        u32_to_buffer(self.byte_length, &mut buffer[num_written..]);
        num_written += 4;
        u32_to_buffer(self.tag_type, &mut buffer[num_written..]);
        num_written += 4;
        for i in 0..self.byte_length/4 {
            u32_to_buffer(self.data[i as usize], &mut buffer[num_written..]);
            num_written += 4;
        }
        num_written
    }

    pub fn length(&self) -> usize {
        self.byte_length as usize + 0x0c
    }

    pub fn from_bytes(buffer: &[Volatile<u8>]) -> Self {
        let mut num_read = 0;
        let id = u32_from_buffer(&buffer[num_read..]);
        num_read += 4;
        let byte_length = u32_from_buffer(&buffer[num_read..]);
        num_read += 4;
        let tag_type = u32_from_buffer(&buffer[num_read..]);
        num_read += 4;

        let mut data = [0x00; 32];
        for i in 0..byte_length / 4 {
            data[i as usize] =  u32_from_buffer(&buffer[num_read..]);
            num_read += 4;
        }

        PropertyTag {
            id,
            byte_length,
            tag_type,
            data
        }
    }
}

pub fn send_tags<'a>(tags: &'a mut StackVec<'a, PropertyTag>) -> &'a mut StackVec<'a, PropertyTag> {

    let mut mailbox = Mailbox::new(Channel::PropertyTagsARMTOVC);

    let tag_buffer: &mut TagBuffer = unsafe { &mut *(((&mut TAG_BUFFER_STORAGE.buffer as *mut [u8]) as *mut u8) as *mut TagBuffer)};

    kprintln!("tag_buffer.buffer[0] = {:?}", tag_buffer.buffer[0].read());
    kprintln!("tag_buffer.buffer[1] = {:?}", tag_buffer.buffer[1].read());
    kprintln!("tag_buffer.buffer[2] = {:?}", tag_buffer.buffer[2].read());
    kprintln!("Totally safe territory...");
    kprintln!("Empty buffer: {:x?}", &tag_buffer.buffer[..300]);

    let mut index = 8;
    for tag in tags.iter() {
        index += tag.to_bytes(&mut tag_buffer.buffer[index..]);
        kprintln!("Added tag {}", index);
    }

    u32_to_buffer(0, &mut tag_buffer.buffer[index..]); // End tag
    index += 4;
    u32_to_buffer(index as u32, &mut tag_buffer.buffer); // Size
    u32_to_buffer(0, &mut tag_buffer.buffer[4..]); // Request code

    kprintln!("{:x?}", &tag_buffer.buffer[..300]);
    mailbox.send((((&mut tag_buffer.buffer as *mut [Volatile<u8>]) as *mut [u8]) as *mut u8) as u32).expect("Error sending tags in mailbox");

    let mailbox_response = mailbox.receive().expect("Error in mailbox reception");

    kprintln!("Received {:x} from mailbox in framebuffer", mailbox_response);

    kprintln!("{:x?}", &tag_buffer.buffer[..300]);

    let mut bytes_read = 0;
    let buffer_size = u32_from_buffer(&tag_buffer.buffer[bytes_read..]);
    bytes_read += 4;
    let _response = u32_from_buffer(&tag_buffer.buffer[bytes_read..]);
    bytes_read += 4;

    tags.truncate(0);

    while bytes_read < buffer_size as usize {
        let tag = PropertyTag::from_bytes(&tag_buffer.buffer[bytes_read..]);
        kprintln!("Received tag: {:x?}", &tag);
        tags.push(tag);
        bytes_read += tag.length();
    }

    tags.pop();

    tags
}
