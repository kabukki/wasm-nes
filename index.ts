import { Emulator, Rom, Save, Video2D, AudioPCM } from '@kabukki/emukit';

import init, { Nes as VM, set_panic_hook, set_log } from './pkg';

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

export interface Options {
    canvas: HTMLCanvasElement;
    rom: Rom;
}

export class Nes extends Emulator<AudioPCM, Video2D> {
    public readonly rom: Rom;
    private vm: any;
    private inputs: Uint8Array;

    constructor ({ canvas, rom }: Options) {
        super(new Video2D(canvas), new AudioPCM(100));
        this.rom = rom;
        this.inputs = new Uint8Array([0, 0]);
        this.vm = VM.new(rom.buffer, this.audio.sampleRate);
    }

    async init () {
        await this.audio.init();
    }

    loadSave (save: Save) {
        this.vm.set_cartridge_ram(save.data);
    }

    input (index: number, input: number) {
        if (index >= 0 && index < this.inputs.length) {
            this.inputs[index] = input;
        }
    }

    cycle () {
        this.vm.update_controllers(this.inputs);
        this.vm.cycle_until_frame();
        this.vm.get_framebuffer((this.video.framebuffer as unknown) as Uint8Array);
        this.video.paint();
        this.audio.queue(this.vm.get_audio());
        this.emit('frame', this.debugFrame());
    }

    reset () {
        this.vm.reset();
    }

    save () {
        return {
            name: this.rom.name,
            date: new Date(),
            data: this.vm.get_cartridge_ram(),
            thumbnail: this.video.screenshot(),
        } as Save;
    }

    debugFrame () {
        const audio = this.audio.debug();
        const stats = this.stats.stats();
        const time = this.vm.get_debug_time();

        return {
            audio,
            performance: {
                fps: stats.fpsAverage || stats.fps,
                delta: stats.deltaAverage || stats.delta,
                frame: stats.frame,
                timestamp: stats.timestamp,
            },
            time,
        }
    }

    debug () {
        const debug = this.vm.get_debug();

        return {
            cartridge: debug.cartridge,
            ppu: debug.ppu,
        };
    }
}

export default async function () {
    const wasm = await init();
    
    set_panic_hook();
    set_log();

    return wasm;
}
