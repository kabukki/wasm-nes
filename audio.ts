export class Audio {
    private ctx: AudioContext;
    private oscillator: OscillatorNode;

    constructor () {
        this.ctx = new AudioContext();
        this.oscillator = null;
    }
    
    play (chunk) {
        const canvas = document.createElement('canvas');
        canvas.height = 200;
        canvas.width = 2048;5
        canvas.style.width = '100%';
        document.body.prepend(canvas);

        const map = 0b0000_1111;
        const buffer = this.ctx.createBuffer(1, 5 * this.ctx.sampleRate, this.ctx.sampleRate);
        buffer.copyToChannel(Float32Array.from({ length: buffer.length }, (_, n) => 2 * ((map >> (n % 8)) & 1) - 1), 0);
        console.log(buffer.getChannelData(0));
        
        // console.log(sample);
        
        const node = this.ctx.createBufferSource();
        node.buffer = buffer;

        const oscillator = this.ctx.createOscillator();
        // (x, y) = (cos(angle), sin(angle));
        // temps t, freq f, (x, y) = (cos(2pi.f.t), cos(2pi.f.t)). A = sin(2pi.f.t + phase)
        // A (amplitude aka volume, cos): [offset, f, overtone], B (sin) [-, f, overtone]
        const wave = this.ctx.createPeriodicWave([0, 1], [0, 0]);
        oscillator.setPeriodicWave(wave);

        const analyzer = this.ctx.createAnalyser();

        // Graph
        // node.connect(analyzer);
        oscillator.connect(analyzer);
        // analyzer.connect(this.ctx.destination);

        // oscillator.start();
        
        const data = new Float32Array(analyzer.fftSize);
        const ctx = canvas.getContext('2d');
        ctx.strokeStyle = 'black';
        
        const draw = () => {
            let x = 0;
            analyzer.getFloatTimeDomainData(data);
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            ctx.beginPath();

            for (const td of data) {
                const y = 100 + td * 10;
                if (x === 0) {
                    ctx.moveTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
                x += canvas.width / analyzer.fftSize;
            }
            
            ctx.stroke();
            requestAnimationFrame(draw);
        };

        // setTimeout(draw, 500);
        draw();

        // if (!this.oscillator) {
        //     this.oscillator = this.ctx.createOscillator();
        //     this.oscillator.connect(this.ctx.destination);
            // this.oscillator.type = '';
        //     this.oscillator.frequency.value = 440;
        //     this.oscillator.start();
        // }
    }
    
    pause () {
        if (this.oscillator) {
            this.oscillator.stop();
            this.oscillator = null;
        }
    }
}
