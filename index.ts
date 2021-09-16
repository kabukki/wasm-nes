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

export enum Button {
    A       = 0b0000_0001,
    B       = 0b0000_0010,
    Select  = 0b0000_0100,
    Start   = 0b0000_1000,
    Up      = 0b0001_0000,
    Down    = 0b0010_0000,
    Left    = 0b0100_0000,
    Right   = 0b1000_0000,
}

export enum ButtonIndex {
    A,
    B,
    X,
    Y,
    LB,
    RB,
    LT,
    RT,
    Back,
    Start,
    LeftJoystick,
    RightJoystick,
    Up,
    Down,
    Left,
    Right,
    Home,
}

const keyMap = {
    keyboard: {
        ' ': Button.A,
        'Escape': Button.B,
        'Shift': Button.Select,
        'Enter': Button.Start,
        'ArrowUp': Button.Up,
        'ArrowDown': Button.Down,
        'ArrowLeft': Button.Left,
        'ArrowRight': Button.Right,
    },
    gamepad: {
        [ButtonIndex.A]: Button.A,
        [ButtonIndex.B]: Button.B,
        [ButtonIndex.Back]: Button.Select,
        [ButtonIndex.Start]: Button.Start,
        [ButtonIndex.Up]: Button.Up,
        [ButtonIndex.Down]: Button.Down,
        [ButtonIndex.Left]: Button.Left,
        [ButtonIndex.Right]: Button.Right,
    },
};

export class Emulator {
    private vm: Nes;
    private rafHandle: ReturnType<typeof requestAnimationFrame>;
    private debugHandle: ReturnType<typeof setInterval>;
    private inputs: Uint8Array;
    private stats: GameStats;
    public status: Status;

    constructor () {
        this.vm = Nes.new();
        this.inputs = new Uint8Array([0, 0]);
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
                const gamepads = [...navigator.getGamepads()].filter((gamepad) => !!gamepad).slice(0, 2);
                this.inputs.fill(0);

                for (const [index, gamepad] of gamepads.entries()) {
                    this.inputs[index] = 0;

                    // Button controls
                    for (const key in keyMap.gamepad) {
                        const button = gamepad.buttons[key];

                        if (button.pressed) {
                            this.inputs[index] |= keyMap.gamepad[key];
                        }
                    }

                    // Joystick controls
                    if (gamepad.axes[index] <= -0.5) {
                        this.inputs[index] |= Button.Left;
                    } else if (gamepad.axes[index] >= 0.5) {
                        this.inputs[index] |= Button.Right;
                    }

                    if (gamepad.axes[1] <= -0.5) {
                        this.inputs[index] |= Button.Up;
                    } else if (gamepad.axes[1] >= 0.5) {
                        this.inputs[index] |= Button.Down;
                    }
                }
                
                this.vm.update_controllers(this.inputs);
                this.vm.frame();
                context.putImageData(new ImageData(this.vm.get_framebuffer(), 32 * 8, 30 * 8), 0, 0);
                this.rafHandle = requestAnimationFrame(rafCallback);
                this.stats.record(timestamp);
            } catch (err) {
                onError?.(err);
                this.stop(err);
            }
        };

        document.addEventListener('keydown', this.onKeydown.bind(this));
        document.addEventListener('keyup', this.onKeyup.bind(this));
        this.debugHandle = setInterval(() => {
            onDebug({
                stats: this.stats.stats(),
                // ram: this.vm.get_ram(),
                // ram_nametables: this.vm.get_nametable_ram(),
                // ram_cartridge: this.vm.get_cartridge_ram(),    
                // patternTables: this.vm.get_pattern_tables(),
                // palettes: this.vm.get_palettes(),
                // palette: this.vm.get_palette(),
                // nametables: [
                //     this.vm.get_nametable(0),
                //     this.vm.get_nametable(1),
                //     this.vm.get_nametable(2),
                //     this.vm.get_nametable(3),
                // ],
            });
        }, 500);
        this.rafHandle = requestAnimationFrame(rafCallback);
        this.status = Status.Running;
    }

    stop (error?: Error) {
        document.removeEventListener('keydown', this.onKeydown.bind(this));
        document.removeEventListener('keyup', this.onKeyup.bind(this));
        clearInterval(this.debugHandle);
        cancelAnimationFrame(this.rafHandle);
        this.status = error ? Status.Crashed : Status.Idle;
    }

    onKeydown (e: KeyboardEvent) {
        if (e.key in keyMap.keyboard) {
            this.inputs[0] |= keyMap.keyboard[e.key];
            e.preventDefault();
        }
    }

    onKeyup (e: KeyboardEvent) {
        if (e.key in keyMap.keyboard) {
            this.inputs[0] &= ~keyMap.keyboard[e.key];
            e.preventDefault();
        }
    }
}

export default function () {
    return init().then(set_panic_hook).then(set_log);
}
