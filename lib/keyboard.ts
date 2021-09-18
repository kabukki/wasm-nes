import { Input, Button } from './input';

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

    constructor ({ onUpdate }) {
        super(onUpdate);
        document.addEventListener('keyup', this.onKey.bind(this));
        document.addEventListener('keydown', this.onKey.bind(this));
    }

    onKey (e: KeyboardEvent) {
        if (e.key in Keyboard.keymap) {
            e.preventDefault();
            switch (e.type) {
                case 'keydown': this.press(Keyboard.keymap[e.key]); break;
                case 'keyup': this.release(Keyboard.keymap[e.key]); break;
            }
            this.onUpdate(this.value);
        }
    }

    clear () {
        document.removeEventListener('keyup', this.onKey.bind(this));
        document.removeEventListener('keydown', this.onKey.bind(this));
    }
}
