use volatile::prelude::*;
use volatile::{WriteVolatile, ReadVolatile};
use common::IO_BASE;
use console::kprintln;
use timer::spin_sleep_ms;

// Many thanks to 
// https://elinux.org/RPi_Framebuffer
// and
// https://github.com/raspberrypi/firmware/wiki/Mailbox
// for info on address locations, etc

const MAILBOX_BASE: usize = IO_BASE + 0xB880;

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Status {
    MailFull = (1 << 31),
    MailEmpty = (1 << 30),
}

// Mailbox Channels
// Channel Number 	Description
// 0 	Power management interface
// 1 	Framebuffer
// 2 	Virtual UART
// 3 	VCHIQ interface
// 4 	LEDs interface
// 5 	Buttons interface
// 6 	Touch screen interface 
// 7    None/Reserved 
// 8    Property Tags ARM to VC
// 8    Property Tags VC to ARM
// TODO: This probably makes more sense as a bitfield
// because it's only represented in 4 bits
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Channel {
    PowerManagement = 0,
    Framebuffer = 1,
    VirtualUART = 2,
    VCHIQ = 3,
    LED = 4,
    Buttons = 5,
    TouchScreen = 6,
    _Reserved = 7,
    PropertyTagsARMTOVC = 8,
    PropertyTagsVCTOARM = 9,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
// 0x00 	MAIL0_READ 	The read register for mailbox 0 
// 0x10 	MAIL0_PEEK 	Read from the mailbox without removing data from it.
// 0x14 	MAIL0_SENDER 	Sender ID (bottom 2 bits only)
// 0x18 	MAIL0_STATUS 	The status register for mailbox 0
// 0x1C 	MAIL0_CONFIG 	The configuration register for mailbox 0
// 0x20 	MAIL0_WRITE 	The write register for mailbox 0 (this is actually the read register for mailbox 1). 
    MAIL0_READ: ReadVolatile<u32>,
    r0: ReadVolatile<u32>,
    r1: ReadVolatile<u32>,
    r2: ReadVolatile<u32>,
    MAIL0_PEEK: ReadVolatile<u32>,
    MAIL0_SENDER: ReadVolatile<u32>,
    MAIL0_STATUS: ReadVolatile<u32>,
    MAIL0_CONFIG: WriteVolatile<u32>,
    MAIL0_WRITE: WriteVolatile<u32>,
}

pub struct Mailbox {
    channel: Channel,
    registers: &'static mut Registers,
}

impl Mailbox {
    pub fn new(channel: Channel) -> Mailbox {

        let registers = unsafe {
            &mut *(MAILBOX_BASE as *mut Registers)
        };

        let mb = Mailbox {
            channel,
            registers
        };
        Mailbox::print_state(&mb);
        mb
    }

    fn memory_barrier() {
        unsafe {
            asm!("DSB SY" :::: "volatile");
            asm!("DMB SY" :::: "volatile");
        }
    }

    fn print_state(&self) {
        kprintln!("MAIL0_PEEK = {:x}", self.registers.MAIL0_PEEK.read());
        kprintln!("MAIL0_SENDER = {:x}", self.registers.MAIL0_SENDER.read());
        kprintln!("MAIL0_STATUS = {:x}", self.registers.MAIL0_STATUS.read());
    }

    // status is the status to block until
    fn block_while_status(&self, status: Status) -> Result<(), ()> {
        loop {
            self.print_state();
            Mailbox::memory_barrier();
            if !self.registers.MAIL0_STATUS.has_mask(status as u32) {
                break;
            }
            kprintln!("Blocking on status: {:?}({:x}) from mailbox", status, status as u32);
            spin_sleep_ms(1000);
        }
        Ok(())
    }

    pub fn receive(&self) -> Result<(u32), ()> {
        Mailbox::memory_barrier();
        loop {
            self.block_while_status(Status::MailEmpty)?;

            Mailbox::memory_barrier();

            let received = self.registers.MAIL0_READ.read();
            kprintln!("Received {:x} from mailbox", received);

            if received & 0xf == self.channel as u32 {
                return Ok(received & !0xf);
            }
            spin_sleep_ms(1000);
        }
    }

    pub fn send(&mut self, data: u32) -> Result<(), ()> {
        self.block_while_status(Status::MailFull)?;

        kprintln!("Sending {:x} to mailbox on channel {:?}", data, self.channel);

        unsafe {
            kprintln!("Contents of data: {:?}", &(*(data as *mut [u8; 300]))[..]);
        }
        self.registers.MAIL0_WRITE.write((self.channel as u32) | data);

        kprintln!("Sent {:x}", (self.channel as u32) | data);
        Mailbox::memory_barrier();
        Ok(())
    }
}
