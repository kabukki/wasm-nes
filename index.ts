import init, { Nes, set_panic_hook, set_log } from './pkg';

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
    frameRate: number;
    onError?: (err: Error) => void;
    onCycle?: (framebuffer: any) => void;
}

export class Emulator {
    private vm: Nes;
    private interval: ReturnType<typeof setInterval>;

    constructor () {
        this.vm = Nes.new();
        this.interval = null;
    }

    load (rom: Uint8Array) {
        this.vm.load(rom);
    }

    start ({
        clockSpeed = 1000 / 5369,
        frameRate = 1000 / 30,
        onError,
        onCycle,
    }: Options) {
        this.interval = setInterval(() => {
            try {
                onCycle(this.cycle());
            } catch (err) {
                this.stop();
                onError?.(err);
            }
        }, frameRate);
    }

    stop () {
        window.clearInterval(this.interval);
    }

    cycle () {
        const frame = this.vm.cycle();

        return {
            framebuffer: this.vm.get_framebuffer(),
            nametables: [
                this.vm.get_nametable(0),
                this.vm.get_nametable(1),
                this.vm.get_nametable(2),
                this.vm.get_nametable(3),
            ],
            frame,
        };
    }

    debug () {
        return {
            nametables_ram: this.vm.get_nametable_ram(),
            patternTables: this.vm.get_pattern_tables(),
            palettes: this.vm.get_palettes(),
            palette: this.vm.get_palette(),
        }; 
    }
}

export default function () {
    return init().then(set_panic_hook).then(set_log);
}
