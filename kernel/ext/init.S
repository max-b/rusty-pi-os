#define EL0 0b00
#define EL1 0b01
#define EL2 0b10
#define EL3 0b11

.section .text.init

.global _start
_start:
    // read cpu affinity, start core 0, halt rest
    mrs     x1, MPIDR_EL1
    and     x1, x1, #3
    cbz     x1, setup

halt:
    // core affinity != 0, halt it
    wfe
    b       halt

setup:
    // store the desired EL1 stack pointer in x1
    adr     x1, _start

    // read the current exception level into x0 (ref: C5.2.1)
    mrs     x0, CurrentEL
    and     x0, x0, #0b1100
    lsr     x0, x0, #2

switch_to_el2:
    // switch to EL2 if we're in EL3. otherwise switch to EL1
    cmp     x0, EL3
    bne     switch_to_el1

    // set-up SCR_EL3 (bits 0, 4, 5, 7, 8, 10) (A53: 4.3.42)
    mov     x2, #0x5b1
    msr     SCR_EL3, x2

    // set-up SPSR and PL switch! (bits 0, 3, 6, 7, 8, 9) (ref: C5.2.20)
    mov     x2, #0x3c9
    msr     SPSR_EL3, x2
    adr     x2, switch_to_el1
    msr     ELR_EL3, x2
    eret

switch_to_el1:
    // switch to EL1 if we're not already in EL1. otherwise continue with start
    cmp     x0, EL1
    beq     set_stack

    // set the stack-pointer for EL1
    msr     SP_EL1, x1

    // enable CNTP for EL1/EL0 (ref: D7.5.2, D7.5.13)
    // NOTE: This doesn't actually enable the counter stream.
    mrs     x0, CNTHCTL_EL2
    orr     x0, x0, #0b11
    msr     CNTHCTL_EL2, x0
    msr     CNTVOFF_EL2, xzr

    // enable AArch64 in EL1 (A53: 4.3.36)
    mov     x0, #(1 << 31)      // Enable AArch64 for EL1
    orr     x0, x0, #(1 << 1)   // RES1 on A-53
    msr     HCR_EL2, x0
    mrs     x0, HCR_EL2

    // enable floating point and SVE (SIMD) (A53: 4.3.38, 4.3.34)
    msr     CPTR_EL2, xzr     // don't trap accessing SVE registers
    mrs     x0, CPACR_EL1
    orr     x0, x0, #(0b11 << 20)
    msr     CPACR_EL1, x0

    // Set SCTLR to known state (RES1: 11, 20, 22, 23, 28, 29) (A53: 4.3.30)
    mov     x2, #0x0800
    movk    x2, #0x30d0, lsl #16
    msr     SCTLR_EL1, x2

    // set up exception handlers
    adr     x7, _vectors
    msr     VBAR_EL1, x7

    // change execution level to EL1 (ref: C5.2.19)
    mov     x2, #0x3c5
    msr     SPSR_EL2, x2
    adr     x3, set_stack
    msr     ELR_EL2, x3
    eret

set_stack:
    // set the current stack pointer
    mov     sp, x1

zero_bss:
    // load the start address and number of bytes in BSS section
    ldr     x1, =__bss_start
    ldr     x2, =__bss_length

zero_bss_loop:
    // zero out the BSS section, 64-bits at a time
    cbz     x2, go_kmain
    str     xzr, [x1], #8
    sub     x2, x2, #8
    cbnz    x2, zero_bss_loop

go_kmain:
    // jump to kmain, which shouldn't return. halt if it does
    bl      kmain
    b       halt

context_save:
    sub     SP, SP, #8
    str     x29, [SP, #-8]!

    sub     SP, SP, #32
    stp     x25, x26, [SP]
    stp     x27, x28, [SP, #16]

    sub     SP, SP, #32
    stp     x21, x22, [SP]
    stp     x23, x24, [SP, #16]

    sub     SP, SP, #32
    stp     x17, x18, [SP]
    stp     x19, x20, [SP, #16]

    sub     SP, SP, #32
    stp     x13, x14, [SP]
    stp     x15, x16, [SP, #16]

    sub     SP, SP, #32
    stp     x9, x10, [SP]
    stp     x11, x12, [SP, #16]

    sub     SP, SP, #32
    stp     x5, x6, [SP]
    stp     x7, x8, [SP, #16]

    sub     SP, SP, #32
    stp     x1, x2, [SP]
    stp     x3, x4, [SP, #16]

    sub     SP, SP, #32
    stp     q30, q31, [SP]

    sub     SP, SP, #64
    stp     q26, q27, [SP]
    stp     q28, q29, [SP, #32]

    sub     SP, SP, #64
    stp     q22, q23, [SP]
    stp     q24, q25, [SP, #32]

    sub     SP, SP, #64
    stp     q18, q19, [SP]
    stp     q20, q21, [SP, #32]

    sub     SP, SP, #64
    stp     q14, q15, [SP]
    stp     q16, q17, [SP, #32]

    sub     SP, SP, #64
    stp     q10, q11, [SP]
    stp     q12, q13, [SP, #32]

    sub     SP, SP, #64
    stp     q6, q7, [SP]
    stp     q8, q9, [SP, #32]

    sub     SP, SP, #64
    stp     q2, q3, [SP]
    stp     q4, q5, [SP, #32]

    sub     SP, SP, #32
    stp     q0, q1, [SP]

    mrs     x4, TPIDR_EL0
    mrs     x5, SP_EL0
    mrs     x6, SPSR_EL1
    mrs     x7, ELR_EL1

    sub     SP, SP, #32
    stp     x5, x4, [SP, #16]
    stp     x7, x6, [SP]

    // r0/x0 holds `Info`
    // r1/x1 needs to hold esr
    // r2/x2 needs to hold trap frame (what we've just pushed)
    mrs     x1, ESR_EL1
    mov     x2, SP

    // by branch link jumping here we're going to
    // clobber x30, so let's save that to the stack as well
    str     x30, [SP, #-16]!
    bl      handle_exception

.global context_restore
context_restore:

    // first pop our saved x30/lr
    ldr     x30, [SP], #16

    ldp     x8, x9, [SP]
    ldp     x10, x11, [SP, #16]
    add     SP, SP, #32

    msr     ELR_EL1, x8
    msr     SPSR_EL1, x9

    msr     SP_EL0, x10
    msr     TPIDR_EL0, x11

    ldp     q0, q1, [SP]
    ldp     q2, q3, [SP, #32]
    add     SP, SP, #64

    ldp     q4, q5, [SP]
    ldp     q6, q7, [SP, #32]
    add     SP, SP, #64

    ldp     q8, q9, [SP]
    ldp     q10, q11, [SP, #32]
    add     SP, SP, #64

    ldp     q12, q13, [SP]
    ldp     q14, q15, [SP, #32]
    add     SP, SP, #64

    ldp     q16, q17, [SP]
    ldp     q18, q19, [SP, #32]
    add     SP, SP, #64

    ldp     q20, q21, [SP]
    ldp     q22, q23, [SP, #32]
    add     SP, SP, #64

    ldp     q24, q25, [SP]
    ldp     q26, q27, [SP, #32]
    add     SP, SP, #64

    ldp     q28, q29, [SP]
    ldp     q30, q31, [SP, #32]
    add     SP, SP, #64

    ldp     x1, x2, [SP]
    ldp     x3, x4, [SP, #16]
    add     SP, SP, #32

    ldp     x5, x6, [SP]
    ldp     x7, x8, [SP, #16]
    add     SP, SP, #32

    ldp     x9, x10, [SP]
    ldp     x11, x12, [SP, #16]
    add     SP, SP, #32

    ldp     x13, x14, [SP]
    ldp     x15, x16, [SP, #16]
    add     SP, SP, #32

    ldp     x17, x18, [SP]
    ldp     x19, x20, [SP, #16]
    add     SP, SP, #32

    ldp     x21, x22, [SP]
    ldp     x23, x24, [SP, #16]
    add     SP, SP, #32

    ldp     x25, x26, [SP]
    ldp     x27, x28, [SP, #16]
    add     SP, SP, #32

    ldr     x29, [SP]
    add     SP, SP, #16

    ret

#define HANDLER(source, kind) \
    .align 7; \
    stp     x30, x0, [SP, #-16]!; \
    mov     x0, ##source; \
    movk    x0, ##kind, LSL #16; \
    bl      context_save; \
    ldp     x30, x0, [SP], #16; \
    eret

.align 11
_vectors:
    // From Armv8 - A Programmer Guide 10.4

    // Source = CurrentSpEl0; Kind = Synchronous
    HANDLER(0x0000, 0x0000)

    // Source = CurrentSpEl0; Kind = IRQ
    HANDLER(0x0000, 0x0001)

    // Source = CurrentSpEl0; Kind = FIQ
    HANDLER(0x0000, 0x0010)

    // Source = CurrentSpEl0; Kind = SError
    HANDLER(0x0000, 0x0011)

    // Source = CurrentSpElx; Kind = Synchronous
    HANDLER(0x0001, 0x0000)

    // Source = CurrentSpElx; Kind = IRQ
    HANDLER(0x0001, 0x0001)

    // Source = CurrentSpElx; Kind = FIQ
    HANDLER(0x0001, 0x0010)

    // Source = CurrentSpElx; Kind = SError
    HANDLER(0x0001, 0x0011)

    // Source = LowerAArch64; Kind = Synchronous
    HANDLER(0x0010, 0x0000)

    // Source = LowerAArch64; Kind = IRQ
    HANDLER(0x0010, 0x0001)

    // Source = LowerAArch64; Kind = FIQ
    HANDLER(0x0010, 0x0010)

    // Source = LowerAArch64; Kind = SError
    HANDLER(0x0010, 0x0011)

    // Source = LowerAArch32; Kind = Synchronous
    HANDLER(0x0011, 0x0000)

    // Source = LowerAArch32; Kind = IRQ
    HANDLER(0x0011, 0x0001)

    // Source = LowerAArch32; Kind = FIQ
    HANDLER(0x0011, 0x0010)

    // Source = LowerAArch32; Kind = SError
    HANDLER(0x0011, 0x0011)
