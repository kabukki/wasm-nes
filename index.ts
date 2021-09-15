import GameStats from 'game-stats';

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
    debugRate: number;
    canvas: HTMLCanvasElement;
    onError?: (err: Error) => void;
    onDebug?: (info: any) => void;
}

export enum Status {
    Idle,
    Running,
    Crashed,
}

export class Emulator {
    private vm: Nes;
    private rafHandle: ReturnType<typeof requestAnimationFrame>;
    private debugHandle: ReturnType<typeof setInterval>;
    private stats: GameStats;
    public status: Status;

    constructor () {
        this.vm = Nes.new();
        this.stats = new GameStats();
        this.status = Status.Idle;
    }

    load (rom: Uint8Array) {
        this.vm.load(rom);
    }

    start ({
        canvas,
        onError,
        onDebug,
    }: Options) {
        const context = canvas.getContext('2d', { alpha: false });
        const rafCallback = (timestamp) => {
            try {
                this.vm.frame();
                context.putImageData(new ImageData(this.vm.get_framebuffer(), 32 * 8, 30 * 8), 0, 0);
                this.rafHandle = requestAnimationFrame(rafCallback);
                this.stats.record(timestamp);
            } catch (err) {
                onError?.(err);
                this.stop(err);
            }
        };

        this.debugHandle = setInterval(() => {
            onDebug({
                stats: this.stats.stats(),
                ram: this.vm.get_ram(),
                palettes: this.vm.get_palettes(),
            });
        }, 500);
        this.rafHandle = requestAnimationFrame(rafCallback);
        this.status = Status.Running;
    }

    stop (error?: Error) {
        clearInterval(this.debugHandle);
        cancelAnimationFrame(this.rafHandle);
        this.status = error ? Status.Crashed : Status.Idle;
    }

    debug () {
        return {
            nametables: [
                this.vm.get_nametable(0),
                this.vm.get_nametable(1),
                this.vm.get_nametable(2),
                this.vm.get_nametable(3),
            ],
            ram: this.vm.get_ram(),
            ram_nametables: this.vm.get_nametable_ram(),
            ram_cartridge: this.vm.get_cartridge_ram(),
            patternTables: this.vm.get_pattern_tables(),
            palettes: this.vm.get_palettes(),
            palette: this.vm.get_palette(),
        }; 
    }
}

export default function () {
    return init().then(set_panic_hook).then(set_log);
}
