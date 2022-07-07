import React, { createContext, useState, useContext, useEffect, useCallback, useReducer } from 'react';
import Statistics from 'game-stats/lib/interfaces/Statistics';

import wasm, { emulator_read } from '../backend/pkg/index_bg.wasm';
import init, { Emulator, set_panic_hook, set_logger, Button } from '../backend/pkg';
import { useAudio, useAnimationFrame } from './hooks';

export enum Status {
    NONE,
    IDLE,
    RUNNING,
    ERROR,
}

//     save () {
//         return {
//             name: this.rom.name,
//             date: new Date(),
//             data: this.vm.get_cartridge_ram(),
//             thumbnail: this.video.screenshot(),
//         } as Save;
//     }

//     load (save: Save) {
//         this.vm.set_cartridge_ram(save.data);
//     }




interface IEmulatorContext {
    frame: ImageData;
    audio: ReturnType<typeof useAudio>;
    // debug: ReturnType<Emulator['get_debug']>; 
        // emulator: ReturnType<Emulator['get_debug']>;
        // performance: Statistics;
        // logs: object[];
    emulator: Emulator,
    performance: Statistics;
    error?: Error;
    status: Status;
    input (key: Button, state: boolean): void;
    create (rom: Uint8Array): void;
    cycleCpu (): void;
    cyclePpu (): void;
    cycleFrame (): void;
    start (): void;
    reset (): void;
    stop (error?: Error): void;
    destroy (): void;
    // toggleDebug (enabled: boolean): void;
}

const EmulatorContext = createContext<IEmulatorContext>(null);
const logs = [];

export const EmulatorProvider = ({ children }) => {
    const raf = useAnimationFrame();
    const audio = useAudio();
    const [frame, setFrame] = useState(() => new ImageData(256, 240));
    const [emulator, setEmulator] = useState(null);
    const [error, setError] = useState(null);
    const [status, setStatus] = useState(Status.NONE);
    const [, forceRender] = useReducer((x) => !x, true);
    // const [saves, setSaves] = useState(() => []);

    // const onSave = ({ detail: save }) => {
    //     // db.save(emulator.rom.fingerprint, save);
    //     console.log('save');
    // };

    const wrapCycle = (cycle) => () => {
        try {
            //         this.vm.update_controllers(this.inputs);
            cycle();
            //         this.audio.queue(this.vm.get_audio());
            setFrame(new ImageData(emulator.ppu_framebuffer, 256, 240));
            // setFrame(new ImageData(new Uint8ClampedArray(emulator.get_framebuffer()), 256, 240));
        } catch (err) {
            stop(err);
            // At this point, we can't get any debug info, but we can still provide the logs to help understand the error.
            // setDebug((previous) => ({ ...previous, logs }));
        } finally {
            forceRender();
        }
    };

    const cycleCpu = useCallback(wrapCycle(emulator?.cycle.bind(emulator)), [emulator]);
    const cyclePpu = useCallback(wrapCycle(emulator?.cycle_until_ppu.bind(emulator)), [emulator]);
    const cycleFrame = useCallback(wrapCycle(emulator?.cycle_until_frame.bind(emulator)), [emulator]);

    const create = async (rom: Uint8Array) => {
        try {
            setEmulator(Emulator.new(rom, 0));
        } catch (err) {
            console.error(err);
        }
    };

    const start = useCallback(() => {
        raf.start(cycleFrame);
        audio.start();
        setStatus(Status.RUNNING);
    }, [emulator]);

    const reset = useCallback(() => {
        emulator.reset();
    }, [emulator]);

    const stop = useCallback((error?: Error) => {
        audio.stop();
        raf.stop();
        if (error && error instanceof Error) {
            console.error(error);
            setStatus(Status.ERROR);
        } else {
            setStatus(Status.IDLE);
        }
    }, [emulator]);

    const destroy = useCallback(() => {
        setEmulator(null);
    }, [emulator]);

    const input = useCallback((key: Button, state: boolean) => {
        emulator?.update_key(key, state);
        //     input (index: number, input: number) {
        //         if (index >= 0 && index < this.inputs.length) {
        //             this.inputs[index] = input;
        //         }
        //     }
    }, [emulator]);

    // Auto-start emulator on load
    useEffect(() => {
        if (emulator) {
            start();
            return stop;
        } else {
            setStatus(Status.NONE);
            setError(null);
        }
    }, [emulator]);
    
    // Initialize
    useEffect(() => {
        (async () => {
            const instance = await init(wasm);

            // await this.audio.init();
            set_panic_hook((message) => setError(new Error(message)));
            set_logger((log) => logs.push(log));
            // db.getAll().then(setSaves).catch(setError);

            return instance;
        })().then((instance) => console.log(`NES initialized`, instance)).catch(console.error);
    }, []);

    // console.log(emulator);
    
    return (
        <EmulatorContext.Provider value={{
            frame,
            audio,
            // debug,
            emulator,
            performance: raf.stats.stats(),
            error,
            status,
            create,
            cycleCpu,
            cyclePpu,
            cycleFrame,
            start,
            reset,
            stop,
            destroy,
            input,
        }}>
            {children}
        </EmulatorContext.Provider>
    );
};


export const useLifecycle = () => {
    const { create, cycleCpu, cyclePpu, cycleFrame, start, reset, stop, destroy, error, status } = useContext(EmulatorContext);

    return {
        create,
        cycleCpu,
        cyclePpu,
        cycleFrame,
        start,
        reset,
        stop,
        destroy,
        error,
        status,
    };
};

export const useIO = () => {
    const { input, frame, audio } = useContext(EmulatorContext);

    return {
        frame,
        audio,
        input,
    };
};

export const useDebug = () => {
    const { emulator, performance } = useContext(EmulatorContext);
    
    return {
        logs,
        emulator: emulator ? {
            ram: emulator.ram,
            cpu: emulator.cpu,
            ppu: {
                nametables: emulator.ppu_nametables,
            },
        } : null,
        performance, //: debug?.performance,
    };
};

export {
    CpuStatusFlag,
    PpuCtrlFlag,
    PpuMaskFlag,
    PpuStatusFlag,
    Button,
} from '../backend/pkg';
