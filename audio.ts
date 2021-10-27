export class Audio {
    private ctx: AudioContext;
    private buffer: Float32Array;
    private time: number;
    private playHandle: ReturnType<typeof setInterval>;
    private gain: GainNode;

    constructor () {
        this.ctx = new AudioContext();
        this.buffer = new Float32Array();
        this.time = this.ctx.currentTime;
        this.gain = this.ctx.createGain();
        this.gain.gain.value = 1;
        this.gain.connect(this.ctx.destination)
    }

    get sampleRate () {
        return this.ctx.sampleRate;
    }

    start () {
        this.playHandle = setInterval(this.flush.bind(this), 1000);
        this.ctx.resume();
    }

    stop () {
        clearInterval(this.playHandle);
        this.ctx.suspend();
    }

    /**
     * Append samples to the buffer
     */
    queue (chunk: Float32Array) {
        const buffer = new Float32Array(this.buffer.length + chunk.length);
        buffer.set(this.buffer, 0);
        buffer.set(chunk, this.buffer.length);
        this.buffer = buffer;
    }

    /**
     * Play audio in buffer and flush it
     */
    private flush () {
        const source = this.ctx.createBufferSource();
        source.buffer = this.ctx.createBuffer(1, this.buffer.length, this.ctx.sampleRate);
        source.buffer.copyToChannel(this.buffer, 0);

        // Catch up if needed
        if (this.time < this.ctx.currentTime) {
            this.time = this.ctx.currentTime;
        }

        source.connect(this.gain);
        source.start(this.time);
        // console.log(`Time: ${this.time}, context time: ${this.ctx.currentTime}, playing for ${source.buffer.duration}`);

        this.time += source.buffer.duration;
        this.buffer = new Float32Array()
    }
}
