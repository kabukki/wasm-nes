import { useEffect, useRef, useState } from 'react';
import GameStats from 'game-stats';

/**
 * oscillator -> gain -> analyzer -> destination
 */
export const useAudio = (type: OscillatorType = 'sine', frequency = 440) => {
    const raf = useAnimationFrame();
    const [context] = useState(() => new AudioContext());
    const [gain] = useState(() => {
        const gain = context.createGain();
        gain.gain.value = 1;
        return gain;
    });
    const [oscillator] = useState(() => {
        const oscillator = context.createOscillator();
        oscillator.type = type;
        oscillator.frequency.value = frequency;
        return oscillator;
    });
    const [analyzer] = useState(() => {
        const analyzer = context.createAnalyser();
        analyzer.minDecibels = -100;
        analyzer.maxDecibels = 0;
        analyzer.smoothingTimeConstant = 0;
        return analyzer;
    });
    const [data] = useState(() => ({
        timeDomain: new Uint8Array(analyzer.fftSize),
        frequency: new Uint8Array(analyzer.frequencyBinCount),
    }));

    const analyze = () => {
        analyzer.getByteFrequencyData(data.frequency);
        analyzer.getByteTimeDomainData(data.timeDomain);
    };
    
    useEffect(() => {
        gain.connect(analyzer);
        oscillator.start();
    }, []);

    return {
        baseType: oscillator.type,
        sampleRate: context.sampleRate,
        data,
        start () {
            raf.start(analyze);
            analyzer.connect(context.destination);
        },
        stop () {
            raf.stop();
            analyzer.disconnect();
        },
        play () {
            analyze();
            return oscillator.connect(gain);
        },
        pause () {
            analyze();
            return oscillator.disconnect();
        },
        volume (volume) {
            gain.gain.value = volume;
        },
        type (type) {
            oscillator.type = type;
        },
        frequency (frequency) {
            oscillator.frequency.value = frequency;
        },
    };
};

export const useAnimationFrame = () => {
    const stats = useRef(new GameStats({ historyLimit: 100 }));
    const handle = useRef<ReturnType<typeof requestAnimationFrame>>();

    return {
        stats: stats.current,
        start (callback) {
            const rafCallback = (timestamp) => {
                callback();
                stats.current.record(timestamp);
                // Don't run another frame if it has been canceled in the mean time
                if (handle.current) {
                    handle.current = requestAnimationFrame(rafCallback);
                }
            };
        
            handle.current = requestAnimationFrame(rafCallback);
        },
        stop () {
            cancelAnimationFrame(handle.current);
            handle.current = null;
        },
    };
};