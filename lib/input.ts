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

export abstract class Input {
    value: number;
    onUpdate: (input: number) => void;

    constructor (onUpdate) {
        this.onUpdate = onUpdate;
        this.reset();
    }

    press (button: Button) {
        this.value |= button;
    }

    release (button: Button) {
        this.value &= ~button;
    }

    reset () {
        this.value = Button.None;
    }
}
