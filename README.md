# 🕹 Nintendo Entertainment System

A **NES** emulator written in Rust compiled to WebAssemly for usage on the web.

> The Nintendo Entertainment System (NES) is an 8-bit third-generation home video game console produced by Nintendo. Nintendo first released it in Japan as the Family Computer, commonly known as the Famicom, in 1983. The NES, a remodelled version, was released internationally in the following years.

## Overview

- ✅ **CPU**: all official opcodes # Central Processing Unit (Ricoh 2A03) 
- ✅ **PPU**: Pixel Processing Unit
- 🚧 **APU**: Audio Processing Unit: Pulse, ~~triangle~~, ~~noise~~, ~~DMC~~.
- ✅ **Input**: Controller input
- ✅ **Mappers**: `NROM`, `MMC1`, `UxROM`, `003`, `CNROM`, `AxROM`, `GxROM`.
- ✅ **Save states**: game saves via cartridge RAM

### Timing

The emulator synchronizes to video with the [requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame) function, which usually matches the refresh rate of the display.
- CPU runs at 500Hz
- Timers run at 60Hz

At every repaint, enough emulator cycles are run to simulate that the duration for one frame has passed. Given an ideal refresh rate of 60FPS, that is 1/60s.

### Known limitations

The emulator currently lacks in the following areas:
- Open bus behaviour is missing
- Precise PPU timing
- Some sprites are not displayed correctly

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

### `useDebug`

This hook provides various information regarding emulation status.

- `logs` emulator logs (nestest compliant) produced through Rust's [log](https://crates.io/crates/log) facade.
- `emulator` emulator state
- `performance` measures of browser frame performance

## Tests

<img src="https://badgen.net/badge/cpu/70%25/green" />
<img src="https://badgen.net/badge/ppu/24%25/yellow" />
<img src="https://badgen.net/badge/apu/17%25/orange" />
<img src="https://badgen.net/badge/mappers/-/grey" />

Emulation accuracy is tested thanks to test ROMs taken from https://wiki.nesdev.com/w/index.php/Emulator_tests (available [here](https://github.com/christopherpow/nes-test-roms)), and inspired from http://tasvideos.org/EmulatorResources/NESAccuracyTests.html.
Here is the summary of results, you can find details below.

| Component | Passed    | Total     | %         |
|-----------|-----------|-----------|-----------|
| CPU       | 21        | 30        | 70%       |
| PPU       | 10        | 41        | 24%       |
| APU       | 3         | 18        | 17%       |
| Mappers   | -         | -         | -         |
| **Total** | **34**    | **89**    | **38%**   |

### CPU

| Test                                          | Status                |
|-----------------------------------------------|-----------------------|
| `branch_timing_tests/branch_basics`           | ✅ Passed             |
| `branch_timing_tests/backward_branch`         | ✅ Passed             |
| `branch_timing_tests/forward_branch`          | ✅ Passed             |
| `cpu_dummy_reads`                             | ❌ Failed             |
| `cpu_dummy_writes/cpu_dummy_writes_oam`       | ❌ Failed             |
| `cpu_dummy_writes/cpu_dummy_writes_ppumem`    | ❌ Failed             |
| `cpu_exec_space/test_cpu_exec_space_apu`      | ❌ Failed             |
| `cpu_exec_space/test_cpu_exec_space_ppuio`    | ❌ Failed             |
| `cpu_interrupts_v2`                           | ❌ Failed             |
| `cpu_reset/ram_after_reset`                   | ✅ Passed             |
| `cpu_reset/registers`                         | ✅ Passed             |
| `cpu_timing_test6`                            | ✅ Passed             |
| `instr_misc`                                  | ❌ Failed             |
| `instr_test_v5/basics`                        | ✅ Passed             |
| `instr_test_v5/implied`                       | ✅ Passed             |
| `instr_test_v5/immediate`                     | ⚠️ Official only      |
| `instr_test_v5/zero_page`                     | ⚠️ Official only      |
| `instr_test_v5/zp_xy`                         | ⚠️ Official only      |
| `instr_test_v5/absolute`                      | ⚠️ Official only      |
| `instr_test_v5/abs_xy`                        | ⚠️ Official only      |
| `instr_test_v5/ind_x`                         | ⚠️ Official only      |
| `instr_test_v5/ind_y`                         | ⚠️ Official only      |
| `instr_test_v5/branches`                      | ✅ Passed             |
| `instr_test_v5/stack`                         | ✅ Passed             |
| `instr_test_v5/jmp_jsr`                       | ✅ Passed             |
| `instr_test_v5/rts`                           | ✅ Passed             |
| `instr_test_v5/rti`                           | ✅ Passed             |
| `instr_test_v5/brk`                           | ❌ Failed             |
| `instr_test_v5/special`                       | ❌ Failed             |
| `nestest`                                     | ⚠️ Official only      |

### PPU

| Test                                          | Status    |
|-----------------------------------------------|-----------|
| `blargg_ppu_tests_2005.09.15b/palette_ram`    | ✅ Passed |
| `blargg_ppu_tests_2005.09.15b/sprite_ram`     | ✅ Passed |
| `blargg_ppu_tests_2005.09.15b/vbl_clear_time` | ❌ Failed |
| `blargg_ppu_tests_2005.09.15b/vram_access`    | ❌ Failed |
| `nmi_sync/demo_ntsc`                          | ❌ Failed |
| `oam_read`                                    | ✅ Passed |
| `oam_stress`                                  | ❌ Failed |
| `oamtest3`                                    | ❌ Failed |
| `ppu_open_bus`                                | ❌ Decay not implemented  |
| `ppu_read_buffer`                             | ❌ Failed |
| `ppu_sprite_hit/basics`                       | ✅ Passed |
| `ppu_sprite_hit/alignment`                    | ❌ Failed |
| `ppu_sprite_hit/corners`                      | ❌ Failed |
| `ppu_sprite_hit/flip`                         | ❌ Failed |
| `ppu_sprite_hit/left_clip`                    | ❌ Failed |
| `ppu_sprite_hit/right_edge`                   | ❌ Failed |
| `ppu_sprite_hit/screen_bottom`                | ❌ Failed |
| `ppu_sprite_hit/double_height`                | ✅ Passed |
| `ppu_sprite_hit/timing`                       | ❌ Failed |
| `ppu_sprite_hit/timing_order`                 | ❌ Failed |
| `ppu_sprite_overflow/basics`                  | ❌ Failed |
| `ppu_sprite_overflow/details`                 | ✅ Passed |
| `ppu_sprite_overflow/timing`                  | ❌ Failed |
| `ppu_sprite_overflow/obscure`                 | ❌ Failed |
| `ppu_sprite_overflow/emulator`                | ❌ Failed |
| `ppu_vbl_nmi/vbl_basics`                      | ✅ Passed |
| `ppu_vbl_nmi/vbl_set_time`                    | ❌ Failed |
| `ppu_vbl_nmi/vbl_clear_time`                  | ✅ Passed |
| `ppu_vbl_nmi/nmi_control`                     | ❌ Failed |
| `ppu_vbl_nmi/nmi_timing`                      | ❌ Failed |
| `ppu_vbl_nmi/suppression`                     | ❌ Failed |
| `ppu_vbl_nmi/nmi_on_timing`                   | ❌ Failed |
| `ppu_vbl_nmi/nmi_off_timing`                  | ❌ Failed |
| `ppu_vbl_nmi/even_odd_frames`                 | ✅ Passed |
| `ppu_vbl_nmi/even_odd_timing`                 | ❌ Failed |
| `sprdma_and_dmc_dma`                          | -         |
| `sprite_overflow_tests/basics`                | ❌ Failed |
| `sprite_overflow_tests/details`               | ✅ Passed |
| `sprite_overflow_tests/timing`                | ❌ Failed |
| `sprite_overflow_tests/obscure`               | ❌ Failed |
| `sprite_overflow_tests/emulator`              | ❌ Failed |

### APU

| Test                          | Status    |
|-------------------------------|-----------|
| `apu_mixer/dmc`               | ❌ Failed |
| `apu_mixer/noise`             | ❌ Failed |
| `apu_mixer/square`            | ❌ Failed |
| `apu_mixer/triangle`          | ❌ Failed |
| `apu_reset/4015_cleared`      | ❌ Failed |
| `apu_reset/4017_timing`       | ❌ Failed |
| `apu_reset/4017_written`      | ✅ Passed |
| `apu_reset/irq_flag_cleared`  | ✅ Passed |
| `apu_reset/len_ctrs_enabled`  | ❌ Failed |
| `apu_reset/works_immediately` | ❌ Failed |
| `apu_test/len_ctr`            | ❌ Failed |
| `apu_test/len_table`          | ❌ Failed |
| `apu_test/irq_flag`           | ✅ Passed |
| `apu_test/jitter`             | ❌ Failed |
| `apu_test/len_timing`         | ❌ Failed |
| `apu_test/irq_flag_timing`    | ❌ Failed |
| `apu_test/dmc_basics`         | ❌ Failed |
| `apu_test/dmc_rates`          | ❌ Failed |
| ...                           | ...       |

### Mappers

| Test                  | Status    |
|-----------------------|-----------|
| `Holy Mapperel`       | -         |
| ...                   | ...       |

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

### Examples / tutorials / inspiration

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

### Audio

- https://jackschaedler.github.io/circles-sines-signals/dft_introduction.html
- https://www.ams.jhu.edu/dan-mathofmusic/sound-waves/
- https://pudding.cool/2018/02/waveforms/
- https://web.dev/audio-scheduling/
