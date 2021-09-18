import { Input, Button } from './input';

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
    index: number;
    gamepadConnected: boolean;
    onGamepad: (connected: boolean) => void;
    private rafHandle: ReturnType<typeof requestAnimationFrame>;
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

    constructor ({ index, onUpdate, onGamepad }) {
        super(onUpdate);
        this.index = index;
        this.gamepadConnected = false;
        this.onGamepad = onGamepad;
        this.poll();
    }

    poll () {
        const gamepad = [...navigator.getGamepads()].filter((gamepad) => !!gamepad)[this.index];
        
        this.reset();

        if (this.gamepadConnected !== !!gamepad) {
            this.gamepadConnected = !!gamepad;
            this.onGamepad(this.gamepadConnected);
        }

        if (gamepad) {
            // Button controls
            for (const key in Gamepad.keymap) {
                const button = gamepad.buttons[key];

                if (button.pressed) {
                    this.press(Gamepad.keymap[key]);
                }
            }

            // Joystick controls
            if (gamepad.axes[0] <= -0.5) {
                this.press(Button.Left);
            } else if (gamepad.axes[0] >= 0.5) {
                this.press(Button.Right);
            }

            if (gamepad.axes[1] <= -0.5) {
                this.press(Button.Up);
            } else if (gamepad.axes[1] >= 0.5) {
                this.press(Button.Down);
            }
        }
        
        this.onUpdate(this.value);
        this.rafHandle = requestAnimationFrame(this.poll.bind(this));
    }

    clear () {
        cancelAnimationFrame(this.rafHandle);
    }
}
