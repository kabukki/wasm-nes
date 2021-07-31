// https://wiki.nesdev.com/w/index.php/CPU_interrupts

pub const INTERRUPT_LATENCY: u8 = 7;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interrupt {
    /**
     * Non-maskable interrupt
     */
    NMI = 0xFFFA,

    /**
     * Maskable interrupt
     */
    IRQ = 0xFFFE,

    /**
     * System reset
     */
    RESET = 0xFFFC,
}
