import wasm, { emulator_read } from '../backend/pkg/index_bg.wasm';
import init, { Emulator, set_panic_hook, set_logger, Button } from '../backend/pkg';
import GameStats from 'game-stats';

export enum Status {
    IDLE,
    RUNNING,
    ERROR,
}

export class Nes {
    public static VIDEO_WIDTH = 256;
    public static VIDEO_HEIGHT = 240;

    private vm = null;
    private rafHandle: ReturnType<typeof requestAnimationFrame>;
    private stats: GameStats;
    public canvas: HTMLCanvasElement;
    public logs = [];
    public error: Error;
    public onCycle?: () => void;
    public onStatus?: () => void;
    private dbg = null;

    constructor (rom) {
        this.vm = Emulator.new(rom, 0);
        this.stats = new GameStats({ historyLimit: 100 });

        // await this.audio.init();
        set_panic_hook((message) => this.stop(new Error(message)));
        set_logger((log) => {
            console.log(log.text);
            
            // this.logs.push(log);
        });
        // db.getAll().then(setSaves).catch(setError);
    }

    static async init () {
        return init(wasm);
    }

    start () {
        const rafCallback = (timestamp) => {
            this.cycleFrame();
            this.stats.record(timestamp);
            // this.emitSave();
            // this.emitDebug();
            this.rafHandle = requestAnimationFrame(rafCallback);
        };

        this.rafHandle = requestAnimationFrame(rafCallback);
        // this.audio.start();
        this.onStatus?.();
    }

    stop (error?: Error) {
        // audio.stop();
        cancelAnimationFrame(this.rafHandle);
        this.rafHandle = null;

        if (error && error instanceof Error) {
            console.error(error);
            this.error = error;
        }

        this.onStatus?.();
    }

    reset () {
        this.vm.reset();
    }

    cycle (fn) {
        try {
            // this.vm.update_controllers(this.inputs);
            fn();

            if (this.dbg) {
                this.dbg = null;
            }

            this.render();
            // this.audio.queue(this.vm.get_audio());
        } catch (err) {
            this.stop(err);
        } finally {
            this.onCycle?.();
        }
    }

    cycleCpu () {
        this.cycle(this.vm.cycle.bind(this.vm));
    }

    cyclePpu () {
        this.cycle(this.vm.cycle_until_ppu.bind(this.vm));
    }

    cycleFrame () {
        this.cycle(this.vm.cycle_until_frame.bind(this.vm));
    }

    private render () {
        this.canvas?.getContext('2d').putImageData(new ImageData(this.vm.get_framebuffer(), 256, 240), 0, 0);
    }

    get status () {
        if (this.error) {
            return Status.ERROR;
        } else if (this.rafHandle) {
            return Status.RUNNING;
        } else {
            return Status.IDLE;
        }
    }

    get performance () {
        return this.stats.stats();
    }

    get debug () {
        // Lazily get debug info, at most once per cycle
        if (!this.dbg) {
            this.dbg = this.vm.get_debug();
        }

        return this.dbg;
    }
}

export {
    Button,
    CpuStatusFlag,
    PpuCtrlFlag,
    PpuMaskFlag,
    PpuStatusFlag,
    SpriteAttribute,
} from '../backend/pkg';
