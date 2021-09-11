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
    clockRate: number;
    frameRate: number;
    debugRate: number;
    onError?: (err: Error) => void;
    onDebug?: (info: any) => void;
    onDisplay?: (framebuffer: Uint8ClampedArray) => void;
}

export class Emulator {
    private vm: Nes;
    private callback: FrameRequestCallback;

    constructor () {
        this.vm = Nes.new();
        this.callback = null;
    }

    load (rom: Uint8Array) {
        this.vm.load(rom);
    }

    start ({
        onError,
        onDebug,
        onDisplay,
    }: Options) {
        let last;
        let fps;
        
        this.callback = (timestamp) => {
            try {
                const elapsed = (timestamp - last) / 1000;
                const frame = this.vm.frame();
                
                fps = Math.round(1 / elapsed);
                last = timestamp;

                onDisplay(this.vm.get_framebuffer());
                onDebug({ frame, fps });
        
                requestAnimationFrame(this.callback);
            } catch (err) {
                onError?.(err);
                this.callback = null;
            }
        };
      
        requestAnimationFrame(this.callback);
    }

    stop () {
        this.callback = null;
    }

    cycle () {
        return this.vm.frame();
    }
    
    display () {
        return this.vm.get_framebuffer();
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
