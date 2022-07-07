/**
 * https://wiki.nesdev.com/w/index.php/CPU_interrupts
 */

pub const INTERRUPT_LATENCY: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub enum Interrupt {
    NMI     = 0xFFFA, // Non-maskable interrupt
    IRQ     = 0xFFFE, // Maskable interrupt
    RESET   = 0xFFFC, // System reset
}
