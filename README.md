# Nintendo Entertainment System

> The Nintendo Entertainment System (NES) is an 8-bit third-generation home video game console produced by Nintendo. Nintendo first released it in Japan as the Family Computer, commonly known as the Famicom, in 1983. The NES, a remodelled version, was released internationally in the following years.

## Technical specifications

- **Central Processing Unit**: Ricoh 2A03 (NTSC) vs 2A07 (PAL) chip
    - CPU based on MOS 6502
    - Pseudo-**Audio Processing Unit** capabilities
    - 3 general-purpose registers
    - Little-endian
    - 16-bit address bus
- **Pixel Processing Unit**: Ricoh 2C02
    - Memory: 16KiB VRAM

Cartridge ROM is accessed by the CPU through a **Memory Management Controller** (MMC) aka mapper, used to determine which bank to load into memory.

## Development

### Toolchain

The emulator is written in Rust and compiled into a WebAssembly module through wasm-pack and uses wasm-bindgen to ease interoperability with the JavaScript environment. A custom JavaScript file wraps the produced package for convenience when consuming it in JavaScript.

```
.rs ---[wasm-pack]---> .wasm <--> JS wrapper <--- JS
```

The emitted JS wrapper is distributed as an ES Module.

### Compiling a test program

You'll need a 6502 assembler & linker such as [cc65](https://github.com/cc65/cc65).

```bash
cl65 roms/test.s -C roms/test.cfg -o roms/test.bin
```

## Usage

Before creating the emulator, you need to call the `init` function which will correctly instantiate and setup the WebAssembly module.

```js
import init, { Emulator } from '@kabukki/wasm-nes';

init().then(() => {
    const emulator = new Emulator();

    document.getElementById('input').addEventListener('change', async (e) => {
        const buffer = await e.target.files[0]?.arrayBuffer();
        emulator.load(new Uint8Array(buffer));
        emulator.start();
    });
}).catch(console.error);
```

## Resources

### Reference

- http://wiki.nesdev.com
- http://nesdev.com/NESDoc.pdf
- https://en.wikipedia.org/wiki/MOS_Technology_6502
- https://www.copetti.org/writings/consoles/nes

#### Opcodes

- https://www.masswerk.at/6502/6502_instruction_set.html
- http://www.obelisk.me.uk/6502/reference.html

### Examples & tutorials

- https://github.com/koute/pinky
- https://github.com/gianlucag/mos6502
- https://skilldrick.github.io/easy6502/
- https://github.com/GarettCooper/emulator_6502
- https://github.com/GarettCooper/gc_nes_emulator
- https://github.com/daniel5151/ANESE
- https://famicom.party/book
- https://bugzmanov.github.io/nes_ebook

### ROMs & Tests

- https://github.com/christopherpow/nes-test-roms
- https://github.com/koute/pinky/tree/master/mos6502/roms
- https://github.com/koute/pinky/tree/master/nes-testsuite/roms
- https://github.com/Klaus2m5/6502_65C02_functional_tests
- http://wiki.nesdev.com/w/index.php/Emulator_tests
- https://www.qmtpro.com/~nes/
    - https://www.qmtpro.com/~nes/misc
    - https://www.qmtpro.com/~nes/misc/nestest.txt

### Talks

- https://www.youtube.com/watch?v=fWqBmmPQP40
- https://www.youtube.com/watch?v=DMcx9DAHrZQ

### Writing assembly for the platform

- https://timcheeseman.com/nesdev
- https://www.cc65.org/doc/ld65-5.html
