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

export enum InputType {
    Keyboard,
    Gamepad,
}

export class Input {
    id: string;
    label: string;
    type: InputType;
    value: number;

    constructor ({ id, label, type }) {
        this.id = id;
        this.label = label;
        this.type = type;
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

export class Keyboard extends Input {
    static keymap = {
        ' ': Button.A,
        'Escape': Button.B,
        'Shift': Button.Select,
        'Enter': Button.Start,
        'ArrowUp': Button.Up,
        'ArrowDown': Button.Down,
        'ArrowLeft': Button.Left,
        'ArrowRight': Button.Right,
    };

    constructor ({ id, label }) {
        super({ id, label, type: InputType.Keyboard });
    }
}

/**
 * Gamepad button standard order
 * https://w3c.github.io/gamepad/#remapping
 */
 export enum GamepadButton {
    A,
    B,
    X,
    Y,
    LB,
    RB,
    LT,
    RT,
    Back,
    Start,
    LeftJoystick,
    RightJoystick,
    Up,
    Down,
    Left,
    Right,
    Home,
}

export class Gamepad extends Input {
    static keymap = {
        [GamepadButton.A]: Button.A,
        [GamepadButton.B]: Button.B,
        [GamepadButton.Back]: Button.Select,
        [GamepadButton.Start]: Button.Start,
        [GamepadButton.Up]: Button.Up,
        [GamepadButton.Down]: Button.Down,
        [GamepadButton.Left]: Button.Left,
        [GamepadButton.Right]: Button.Right,
    };

    constructor ({ id, label }) {
        super({ id, label, type: InputType.Gamepad });
    }
}

export class InputMonitor extends EventTarget {
    onInput: () => void;
    private inputs: Input[];
    private rafHandle: ReturnType<typeof requestAnimationFrame>;

    constructor () {
        super();
        this.inputs = [new Keyboard({ id: 'keyboard', label: 'Keyboard' })];
    }

    get keyboard () {
        return this.inputs.find((input) => input.type === InputType.Keyboard);
    }

    get gamepads () {
        return this.inputs.filter((input) => input.type === InputType.Gamepad);
    }

    start () {
        this.pollGamepads();
        document.addEventListener('keyup', this.onKey.bind(this));
        document.addEventListener('keydown', this.onKey.bind(this));
    }

    onKey (e: KeyboardEvent) {
        if (e.key in Keyboard.keymap) {
            const previousValue = this.keyboard.value;

            e.preventDefault();
            switch (e.type) {
                case 'keydown': this.keyboard.press(Keyboard.keymap[e.key]); break;
                case 'keyup': this.keyboard.release(Keyboard.keymap[e.key]); break;
            }

            if (this.keyboard.value !== previousValue) {
                this.dispatchEvent(new CustomEvent('update', { detail: this.inputs }));
            }
        }
    }

    pollGamepads () {
        const gamepads = [...navigator.getGamepads()].filter((gamepad) => !!gamepad);
        const previousGamepads = this.gamepads.slice();
        let emitUpdate = false;

        // Remove all gamepads
        this.inputs = this.inputs.filter((input) => input.type !== InputType.Gamepad);

        for (const gamepad of gamepads) {
            const input = previousGamepads.find((input) => input.id === gamepad.index.toString()) || new Gamepad({ id: gamepad.index.toString(), label: gamepad.id });
            const previousValue = input.value;

            input.reset();
            this.inputs.push(input);

            // Button controls
            for (const key in Gamepad.keymap) {
                const button = gamepad.buttons[key];

                if (button.pressed) {
                    input.press(Gamepad.keymap[key]);
                }
            }

            // Joystick controls
            if (gamepad.axes[0] <= -0.5) {
                input.press(Button.Left);
            } else if (gamepad.axes[0] >= 0.5) {
                input.press(Button.Right);
            }

            if (gamepad.axes[1] <= -0.5) {
                input.press(Button.Up);
            } else if (gamepad.axes[1] >= 0.5) {
                input.press(Button.Down);
            }

            if (input.value !== previousValue) {
                emitUpdate = true;
            }
        }

        if (emitUpdate || this.gamepads.length !== previousGamepads.length) {
            this.dispatchEvent(new CustomEvent('update', { detail: this.inputs }));
        }

        this.rafHandle = requestAnimationFrame(this.pollGamepads.bind(this));
    }

    stop () {
        cancelAnimationFrame(this.rafHandle);
        document.removeEventListener('keyup', this.onKey.bind(this));
        document.removeEventListener('keydown', this.onKey.bind(this));
    }
}
