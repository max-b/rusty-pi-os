/// The address where I/O peripherals are mapped to.
pub const IO_BASE: usize = 0x3F000000;

/// Power management addresses
pub const ARM_POWER_MANAGEMENT_BASE: usize = IO_BASE + 0x100000;
pub const ARM_POWER_MANAGEMENT_RSTC: usize = ARM_POWER_MANAGEMENT_BASE + 0x1C;
pub const ARM_POWER_MANAGEMENT_WDOG: usize = ARM_POWER_MANAGEMENT_BASE + 0x24;
pub const ARM_POWER_MANAGEMENT_PASSWD: u32 = 0x5A << 24;
pub const ARM_POWER_MANAGEMENT_FULL_RESET: u32 = 0x20;

/// Generates `pub enums` with no variants for each `ident` passed in.
pub macro states($($name:ident),*) {
    $(
        /// A possible state.
        #[doc(hidden)]
        pub enum $name {  }
    )*
}
