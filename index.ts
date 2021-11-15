import GameStats from 'game-stats';
import throttle from 'lodash.throttle';

import init, { Nes, set_panic_hook, set_log, fingerprint } from './pkg';
import { Audio } from './audio';

export enum Button {
    None    = 0b0000_0000,
    A       = 0b0000_0001,
    B       = 0b0000_0010,
    Select  = 0b0000_0100,
    Start   = 0b0000_1000,
    Up      = 0b0001_0000,
    Down    = 0b0010_0000,
    Left    = 0b0100_0000,
    Right   = 0b1000_0000,
}

export interface Save {
    name: string;
    date: Date;
    data: Uint8Array;
    thumbnail: string;
}

export interface Rom {
    name: string;
    buffer: Uint8Array;
    fingerprint: string;
}

export async function getRom (file: File) {
    const buffer = new Uint8Array(await file?.arrayBuffer());

    return {
        name: file.name,
        buffer,
        fingerprint: fingerprint(buffer),
    } as Rom;
}

export class Emulator extends EventTarget {
    private vm: Nes;
    private rafHandle: ReturnType<typeof requestAnimationFrame>;
    private inputs: Uint8Array;
    private stats: GameStats;
    private audio: Audio;
    private context: CanvasRenderingContext2D;
    private framebuffer: Uint8ClampedArray;
    public canvas: HTMLCanvasElement;
    public rom: Rom;

    constructor (canvas: HTMLCanvasElement, rom: Rom) {
        super();
        this.audio = new Audio();
        this.canvas = canvas;
        this.context = this.canvas.getContext('2d');
        this.framebuffer = new Uint8ClampedArray(4 * this.canvas.width * this.canvas.height);
        this.rom = rom;
        this.inputs = new Uint8Array([0, 0]);
        this.stats = new GameStats();
        this.vm = Nes.new(rom.buffer, this.audio.sampleRate);
        this.save = throttle(this.save.bind(this), 2000);
        this.debug = throttle(this.debug.bind(this), 1000);
    }

    async start () {
        await this.audio.init();
        const rafCallback = (timestamp) => {
            this.rafCallback(timestamp);
            this.save();
            this.debug();
            this.rafHandle = requestAnimationFrame(rafCallback);
        };

        this.rafHandle = requestAnimationFrame(rafCallback);
        this.audio.start();
    }

    step () {
        requestAnimationFrame((timestamp) => {
            this.rafCallback(timestamp);
            this.save();
            this.debug();
        });
    }

    stop (error?: Error) {
        this.audio.stop();
        cancelAnimationFrame(this.rafHandle);
        if (error) {
            this.dispatchEvent(new CustomEvent('error', {
                detail: {
                    error,
                },
            }));
        }
    }

    reset () {
        this.vm.reset();
    }

    input (index: number, input: number) {
        if (index >= 0 && index < this.inputs.length) {
            this.inputs[index] = input;
        }
    }

    loadSave (save: Save) {
        this.vm.set_cartridge_ram(save.data);
    }

    private save () {
        const save = {
            name: this.rom.name,
            date: new Date(),
            data: this.vm.get_cartridge_ram(),
            thumbnail: this.canvas.toDataURL(),
        };

        this.dispatchEvent(new CustomEvent('save', {
            detail: {
                save,
            },
        }));
    }

    private rafCallback (timestamp) {
        try {
            this.vm.update_controllers(this.inputs);
            this.vm.cycle_until_frame();
            this.vm.get_framebuffer((this.framebuffer as unknown) as Uint8Array);
            this.audio.queue(this.vm.get_audio());
            this.context.putImageData(new ImageData(this.framebuffer, this.canvas.width, this.canvas.height), 0, 0);
            this.stats.record(timestamp);
        } catch (err) {
            this.stop(err);
        }
    }

    private debug () {
        const stats = this.stats.stats();
        const audio = this.audio.debug();
        const debug = this.vm.get_debug();

        this.dispatchEvent(new CustomEvent('debug', {
            detail: {
                audio,
                performance: {
                    fps: stats.fpsAverage,
                    delta: stats.deltaAverage,
                    frame: stats.frame,
                    timestamp: stats.timestamp,
                },
                time: debug.time,
                cartridge: debug.cartridge,
                ppu: debug.ppu,
            },
        }));
    }
}

export default async function () {
    const wasm = await init();
    
    set_panic_hook();
    set_log();

    return wasm;
}
