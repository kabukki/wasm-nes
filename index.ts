import init, { Nes, set_panic_hook } from './pkg';

/**
 * The NES's master clock frequency is 21.477272 Mhz.
 * The PPU divides it by 4, hence runs at 5.369318 Mhz (3x CPU).
 * The CPU divides it by 12, hence runs at 1.7897727 Mhz.
 * The APU divides it by 89490, hence runs at 239.996335 Hz.
 * Since 12 / 4 = 3 there are 3 PPU clocks per 1 CPU clock.
 * Since 89490 / 12 = 7457.5 there are 7457.5 CPU clocks per 1 APU clock.
 * https://wiki.nesdev.com/w/index.php/Cycle_reference_chart
 */

interface Options {
    clockSpeed: number;
    onError?: (err: Error) => void;
}

export class Emulator {
    private vm: Nes;
    private interval: ReturnType<typeof setInterval>;

    constructor () {
        this.vm = Nes.new();
    }

    load (rom: Uint8Array) {
        this.vm.load(rom);
    }

    start ({
        clockSpeed = 1000 / 5369,
        onError,
    }: Options) {
        this.interval = setInterval(() => {
            try {
                this.vm.cycle();
            } catch (err) {
                this.stop();
                onError?.(err);
            }
        }, clockSpeed);
    }

    stop () {
        clearInterval(this.interval);
    }
}

export default function () {
    return init().then(set_panic_hook);
}

export { Cartridge } from './pkg';
