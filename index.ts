import GameStats from 'game-stats';

import init, { Nes, set_panic_hook, set_log, fingerprint } from './pkg';
import { Audio } from './audio';

interface Options {
    debugRate: number;
    onError?: (err: Error) => void;
    onDebug?: (info: any) => void;
    onSave?: (save: Save) => void;
}

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

export enum Status {
    Idle,
    Running,
    Crashed,
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

export class Emulator {
    private vm: Nes;
    private rafHandle: ReturnType<typeof requestAnimationFrame>;
    private saveHandle: ReturnType<typeof setInterval>;
    private debugHandle: ReturnType<typeof setInterval>;
    private inputs: Uint8Array;
    private stats: GameStats;
    private audio: Audio;
    public status: Status;
    public canvas: HTMLCanvasElement;
    public rom: Rom;

    constructor (canvas: HTMLCanvasElement, rom: Rom) {
        this.vm = Nes.new(rom.buffer);
        this.inputs = new Uint8Array([0, 0]);
        this.stats = new GameStats();
        this.status = Status.Idle;
        this.canvas = canvas;
        this.rom = rom;
        this.audio = new Audio();
    }

    start ({ onError, onDebug, onSave }: Options) {
        const context = this.canvas.getContext('2d');
        const framebuffer = new Uint8ClampedArray(4 * this.canvas.width * this.canvas.height);
        const rafCallback = (timestamp) => {
            try {
                this.vm.update_controllers(this.inputs);
                this.vm.cycle_until_frame();
                this.vm.get_framebuffer((framebuffer as unknown) as Uint8Array);
                const audio = this.vm.get_audio(this.audio.sampleRate);
                this.audio.queue(audio);
                context.putImageData(new ImageData(framebuffer, this.canvas.width, this.canvas.height), 0, 0);
                this.rafHandle = requestAnimationFrame(rafCallback);
                this.stats.record(timestamp);
            } catch (err) {
                onError?.(err);
                this.stop(err);
            }
        };

        if (onSave) {
            this.saveHandle = setInterval(() => {
                onSave(this.getSave());
            }, 1000);
        }

        if (onDebug) {
            this.debugHandle = setInterval(() => {
                onDebug({
                    stats: this.stats.stats(),
                    ram: this.vm.get_ram(),
                    oam: this.vm.get_oam(),
                    // ram_nametables: this.vm.get_nametable_ram(),
                    // ram_cartridge: this.vm.get_cartridge_ram(),    
                    patternTables: this.vm.get_pattern_tables(),
                    palettes: this.vm.get_palettes(),
                    palette: this.vm.get_palette(),
                });
            }, 1000);
        }

        this.rafHandle = requestAnimationFrame(rafCallback);
        this.audio.start();
        this.status = Status.Running;
    }

    stop (error?: Error) {
        this.audio.stop();
        clearInterval(this.saveHandle);
        clearInterval(this.debugHandle);
        cancelAnimationFrame(this.rafHandle);
        this.status = error ? Status.Crashed : Status.Idle;
    }

    reset () {
        this.vm.reset();
    }

    input (index: number, input: number) {
        if (index >= 0 && index < this.inputs.length) {
            this.inputs[index] = input;
        }
    }

    getSave (): Save {
        return {
            name: this.rom.name,
            date: new Date(),
            data: this.vm.get_cartridge_ram(),
            thumbnail: this.canvas.toDataURL(),
        };
    }

    loadSave (save: Save) {
        this.vm.set_cartridge_ram(save.data);
    }
}

export default async function () {
    const wasm = await init();
    
    set_panic_hook();
    set_log();

    return wasm;
}
