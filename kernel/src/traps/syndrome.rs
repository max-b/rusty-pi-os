#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8)
}

impl From<u32> for Fault {
    fn from(val: u32) -> Fault {
        match (val as u8 & 0b111111) >> 2 {
            0b0000 => Fault::AddressSize,
            0b0001 => Fault::Translation,
            0b0010 => Fault::AccessFlag,
            0b0011 => Fault::Permission,
            0b1000 => Fault::Alignment,
            0b1100 => Fault::TlbConflict,
            _ => Fault::Other(val as u8 & 0b111111)
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    McrMrc,
    McrrMrrc,
    LdcStc,
    SimdFp,
    Vmrs,
    Mrrc,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort {
        kind: Fault,
        level: u8,
    },
    PCAlignmentFault,
    DataAbort {
        kind: Fault,
        level: u8
    },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32)
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Syndrome {
        use self::Syndrome::*;

        match esr >> 26 { // TODO: check if I need to mask high bits?
            0b000000 => Unknown,
            0b000001 => WfiWfe,
            0b000011 => McrMrc,
            0b000100 => McrrMrrc,
            0b000101 => McrMrc,
            0b000110 => LdcStc,
            0b000111 => SimdFp,
            0b001000 => Vmrs,
            0b001100 => Mrrc,
            0b001110 => IllegalExecutionState,
            0b010001 => Svc(esr as u16), // TODO: check that this is correct encoding
            0b010010 => Hvc(esr as u16),
            0b010011 => Smc(esr as u16),
            0b010101 => Svc(esr as u16),
            0b010110 => Hvc(esr as u16),
            0b010111 => Smc(esr as u16),
            0b011000 => MsrMrsSystem,
            // TODO: What is `IMPLEMENTATION DEFINED` for 0b011111??
            0b100000 => InstructionAbort {
                kind: Fault::from(esr),
                level: esr as u8 & 0b00000011
            },
            0b100001 => InstructionAbort {
                kind: Fault::from(esr),
                level: esr as u8 & 0b00000011
            },
            0b100010 => PCAlignmentFault,
            0b100100 => DataAbort {
                kind: Fault::from(esr),
                level: esr as u8 & 0b00000011
            },
            0b100101 => DataAbort {
                kind: Fault::from(esr),
                level: esr as u8 & 0b00000011
            },
            0b100110 => SpAlignmentFault,
            0b101000 => TrappedFpu,
            0b101100 => TrappedFpu,
            0b101111 => SError,
            0b110000 => Breakpoint,
            0b110001 => Breakpoint,
            0b110010 => Step,
            0b110011 => Step,
            0b110100 => Watchpoint,
            0b110101 => Watchpoint,
            0b111000 => Brk(esr as u16),
            0b111100 => Brk(esr as u16),
            _ => Other(esr),
        }
    }
}
