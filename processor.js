import { RingBuffer } from 'ringbuf.js';

class Processor extends AudioWorkletProcessor {
    constructor(options) {
        super(options);
        this.interleaved = new Float32Array(128);
        this.buffer = new RingBuffer(options.processorOptions.queue, Float32Array);
    }

    process (inputs, outputs) {
        const read = this.buffer.pop(this.interleaved);

        for (let n = 0; n < read; n++) {
            outputs[0][0][n] = this.interleaved[n];
        }

        return true;
    }
}

registerProcessor('processor', Processor);
