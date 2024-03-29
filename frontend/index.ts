import GameStats from 'game-stats';

import wasm from '../backend/pkg/index_bg.wasm';
import init, { Button, Emulator, set_panic_hook } from '../backend/pkg';
import { Debug } from './debug';
import { Logs } from './logs';
import { Audio } from './audio';

export enum Status {
    IDLE,
    RUNNING,
    ERROR,
}

export class Nes {
    static VIDEO_WIDTH = 256;
    static VIDEO_HEIGHT = 240;
    
    canvas: HTMLCanvasElement;
    error: Error;
    logs: Logs;
    memory: WebAssembly.Memory;
    debug: Debug;
    audio: Audio;
    onCycle?: () => void;
    onStatus?: () => void;

    #vm: Emulator;
    #rafHandle: ReturnType<typeof requestAnimationFrame>;
    #stats: GameStats;

    static async new (rom) {
        const { memory } = await init(wasm);
        return new Nes(rom, memory);
    }

    private constructor (rom, memory) {
        this.logs = new Logs();
        this.memory = memory;
        this.audio = new Audio();
        this.#vm = Emulator.new(rom, this.audio.sampleRate);
        this.#stats = new GameStats({ historyLimit: 100 });

        set_panic_hook((message) => this.stop(new Error(message)));
        // db.getAll().then(setSaves).catch(setError);
    }

    start () {
        const rafCallback = (timestamp) => {
            this.cycleUntil('frame');
            this.#stats.record(timestamp);
            // Don't run another frame if it has been canceled in the mean time
            if (this.#rafHandle) {
                this.#rafHandle = requestAnimationFrame(rafCallback);
            }
        };

        this.#rafHandle = requestAnimationFrame(rafCallback);
        this.audio.start();
        this.onStatus?.();
    }

    stop (error?: Error) {
        this.audio.stop();
        cancelAnimationFrame(this.#rafHandle);
        this.#rafHandle = null;

        if (error && error instanceof Error) {
            console.error(error);
            this.error = error;
        }

        this.onStatus?.();
    }

    reset () {
        this.#vm.reset();
    }

    private cycle (fn) {
        try {
            // this.vm.update_controllers(this.inputs);
            fn();
            this.debug = new Debug(this.#vm);
            this.render();
            this.audio.analyze();
            this.audio.queue(this.#vm.get_audio());
        } catch (err) {
            // Don't call stop() here, because the original error will already be caught by the panic hook
            console.error(err);
        } finally {
            this.onCycle?.();
        }
    }

    cycleUntil (duration) {
        switch (duration) {
            case 'tick': this.cycle(this.#vm.cycle.bind(this.#vm)); break;
            case 'cpu': this.cycle(this.#vm.cycle_until_cpu.bind(this.#vm)); break;
            case 'ppu': this.cycle(this.#vm.cycle_until_ppu.bind(this.#vm)); break;
            case 'scanline': this.cycle(this.#vm.cycle_until_scanline.bind(this.#vm)); break;
            case 'frame': this.cycle(this.#vm.cycle_until_frame.bind(this.#vm)); break;
            default: console.warn('Unknown cycle duration');
        }
    }

    private render () {
        this.canvas?.getContext('2d').putImageData(new ImageData(this.#vm.get_framebuffer(), Nes.VIDEO_WIDTH, Nes.VIDEO_HEIGHT), 0, 0);
    }

    input (player: number, button: Button, pressed: boolean) {
        this.#vm.update_controller(player, button, pressed);
    }

    get status () {
        if (this.error) {
            return Status.ERROR;
        } else if (this.#rafHandle) {
            return Status.RUNNING;
        } else {
            return Status.IDLE;
        }
    }

    get performance () {
        return this.#stats.stats();
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
