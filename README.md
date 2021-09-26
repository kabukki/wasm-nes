# 🕹 Nintendo Entertainment System

A **NES** emulator written in Rust compiled to WebAssemly for usage on the web.

> The Nintendo Entertainment System (NES) is an 8-bit third-generation home video game console produced by Nintendo. Nintendo first released it in Japan as the Family Computer, commonly known as the Famicom, in 1983. The NES, a remodelled version, was released internationally in the following years.

## Current implementation status:

- ✅ Central Processing Unit (Ricoh 2A03) 
- ✅ Pixel Processing Unit
- ❌ Audio Processing Unit 
- ✅ Input devices (keyboard & controller)
- 🚧 Cartridge [mappers](https://wiki.nesdev.com/w/index.php/Mapper): NROM

### Well-known tests

Based on https://wiki.nesdev.com/w/index.php/Emulator_tests

#### CPU

| Test                  | Status    |
|-----------------------|-----------|
| `branch_timing_tests` | Passed ✅ |
| `cpu_dummy_reads`     | TBD ❓    |
| `cpu_dummy_writes`    | TBD ❓    |
| `cpu_exec_space`      | TBD ❓    |
| `cpu_flag_concurrency`| TBD ❓    |
| `cpu_interrupts_v2`   | Failed ❌ (APU missing)   |
| `cpu_reset`           | TBD ❓    |
| `cpu_timing_test6`    | Passed ✅ |
| `instr_misc`          | Passed ✅ |
| `instr_test_v5`       | Passed ✅ |
| `nestest`             | Passed ✅ |
| `ram_retain`          | TBD ❓    |

#### PPU

| Test                              | Status    |
|-----------------------------------|-----------|
| `blargg_ppu_tests_2005.09.15b`    | TBD ❓    |
| `nmi_sync`                        | TBD ❓    |
| `ntsc_torture`                    | TBD ❓    |
| `oam_read`                        | TBD ❓    |
| `oam_stress`                      | TBD ❓    |
| `oamtest3`                        | TBD ❓    |
| `ppu_open_bus`                    | TBD ❓    |
| `ppu_read_buffer`                 | TBD ❓    |
| `ppu_sprite_hit`                  | TBD ❓    |
| `ppu_sprite_overflow`             | TBD ❓    |
| `ppu_vbl_nmi`                     | TBD ❓    |
| `sprdma_and_dmc_dma`              | TBD ❓    |
| `tvpassfail`                      | TBD ❓    |

#### APU

| Test                  | Status    |
|-----------------------|-----------|

#### Mappers

| Test                  | Status    |
|-----------------------|-----------|

#### Inputs

| Test                  | Status    |
|-----------------------|-----------|

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

- https://www.masswerk.at/6502/6502_instruction_set.html (contains errors for instruction timing)
- http://www.obelisk.me.uk/6502/reference.html
- https://www.nesdev.com/6502_cpu.txt

### Examples & tutorials

- https://github.com/gianlucag/mos6502
- https://github.com/GarettCooper/gc_nes_emulator
- https://skilldrick.github.io/easy6502/
- https://famicom.party/book
- https://bugzmanov.github.io/nes_ebook
- http://emudev.de/
- https://austinmorlan.com/posts/nes_rendering_overview/
- https://wiki.nesdev.com/w/index.php/The_frame_and_NMIs

### ROMs & Tests

- http://wiki.nesdev.com/w/index.php/Emulator_tests
- https://github.com/christopherpow/nes-test-roms
- https://github.com/koute/pinky/tree/master/mos6502/roms
- https://github.com/koute/pinky/tree/master/nes-testsuite/roms
- https://github.com/Klaus2m5/6502_65C02_functional_tests
- https://www.qmtpro.com/~nes/
    - https://www.qmtpro.com/~nes/misc
    - https://www.qmtpro.com/~nes/misc/nestest.txt
- https://github.com/bbbradsmith/nes-audio-tests
- http://bootgod.dyndns.org:7777/search.php?ines=1&group=groupid

### Videos & talks

- https://www.youtube.com/watch?v=fWqBmmPQP40
- https://www.youtube.com/playlist?list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf

### Assembly

- https://timcheeseman.com/nesdev
- https://www.cc65.org/doc/ld65-5.html
