pub const INTERRUPT_LATENCY: u8 = 7;

pub enum Interrupt {
    /**
     * Maskable interrupt
     */
    IRQ = 0xFFFE,

    /**
     * Non-maskable interrupt
     */
    NMI = 0xFFFA,

    /**
     * System reset
     */
    RESET = 0xFFFC,
}
