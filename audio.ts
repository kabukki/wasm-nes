import { RingBuffer } from 'ringbuf.js';

export class Audio {
    private ctx: AudioContext;
    private gain: GainNode;
    private buffer: RingBuffer;
 
    constructor () {
        this.ctx = new AudioContext();
        this.gain = this.ctx.createGain();
        this.gain.gain.value = 1;
        this.gain.connect(this.ctx.destination);
    }

    get sampleRate () {
        return this.ctx.sampleRate;
    }

    get bufferSize () {
        const length = 100; // ms
        return length * this.ctx.sampleRate / 1000;
    }

    async init () {
        if (!this.buffer) {
            await this.ctx.audioWorklet.addModule(new URL('processor.js', import.meta.url).href);

            const queue = RingBuffer.getStorageForCapacity(this.bufferSize, Float32Array);
            const node = new AudioWorkletNode(this.ctx, 'processor', {
                processorOptions: {
                    queue,
                },
            });

            node.connect(this.gain);
            this.buffer = new RingBuffer(queue, Float32Array);
        }
    }

    start () {
        this.ctx.resume();
    }

    stop () {
        this.ctx.suspend();
    }

    queue (chunk: Float32Array) {
        this.buffer.push(chunk);
    }

    debug () {
        return {
            sampleRate: this.sampleRate,
            capacity: this.buffer.capacity - 1, // Todo update package to fix this and call instead
            readable: this.buffer.available_read(),
            writable: this.buffer.available_write(),
            full: this.buffer.full(),
        };
    }
}
