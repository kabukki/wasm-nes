import { Emulator } from '../backend/pkg';

class Memoizable {
    protected memoize (name: string, getter: () => unknown) {
        Object.defineProperty(this, name, {
            get () {
                try {
                    Object.defineProperty(this, name, {
                        value: getter(),
                        writable: false,
                    });
                    return this[name];
                } catch (err) {
                    console.warn(`Failed to get value of ${name}`, err.message);
                    return null;
                }
            },
            configurable: true,
        });    
    }
}

class DebugBus extends Memoizable {
    constructor (private vm: Emulator) {
        super();
        this.memoize('ram', () => vm.debug_bus_ram());
        this.memoize('stack', () => vm.debug_bus_stack());
        this.memoize('dma', () => vm.debug_bus_dma());
    }

    at (address: number) {
        return this.vm.debug_bus_at(address);
    }
}

class DebugCartridge extends Memoizable {
    constructor (vm: Emulator) {
        super();
        this.memoize('ines',            () => vm.debug_cartridge_ines());
        this.memoize('patternTables',   () => vm.debug_cartridge_pattern_tables());
        this.memoize('prgCurrent',      () => vm.debug_cartridge_prg_current());
        this.memoize('prgCapacity',     () => vm.debug_cartridge_prg_capacity());
        this.memoize('chrCurrent',      () => vm.debug_cartridge_chr_current());
        this.memoize('chrCapacity',     () => vm.debug_cartridge_chr_capacity());
    }
}

class DebugDisassembly extends Memoizable {
    constructor (private vm: Emulator) {
        super();
        this.memoize('total', () => vm.debug_disassembly_total());
    }

    at (address) {
        return this.vm.debug_disassembly_at(address);
    }

    address_to_index (address) {
        return this.vm.debug_disassembly_address_to_index(address);
    }

    index_to_address (index) {
        return this.vm.debug_disassembly_index_to_address(index);
    }
}

class DebugCpu extends Memoizable {
    constructor (vm: Emulator) {
        super();
        this.memoize('clock', () => vm.debug_cpu_clock());
        this.memoize('pc', () => vm.debug_cpu_pc());
        this.memoize('sp', () => vm.debug_cpu_sp());
        this.memoize('a', () => vm.debug_cpu_a());
        this.memoize('x', () => vm.debug_cpu_x());
        this.memoize('y', () => vm.debug_cpu_y());
        this.memoize('status', () => vm.debug_cpu_status());
        this.memoize('cycles', () => vm.debug_cpu_cycles());
        this.memoize('interrupt', () => vm.debug_cpu_interrupt());
    }
}

class DebugPpu extends Memoizable {
    constructor (vm: Emulator) {
        super();
        this.memoize('clock', () => vm.debug_ppu_clock());
        this.memoize('ctrl', () => vm.debug_ppu_ctrl());
        this.memoize('mask', () => vm.debug_ppu_mask());
        this.memoize('status', () => vm.debug_ppu_status());
        this.memoize('dot', () => vm.debug_ppu_dot());
        this.memoize('scanline', () => vm.debug_ppu_scanline());
        this.memoize('frame', () => vm.debug_ppu_frame());
        this.memoize('oam', () => vm.debug_ppu_oam());
        this.memoize('palettes', () => vm.debug_ppu_palettes());
        this.memoize('palette', () => vm.debug_ppu_palette());
        this.memoize('nametables', () => vm.debug_ppu_nametables());
    }
}

class DebugApu extends Memoizable {
    constructor (vm: Emulator) {
        super();
        this.memoize('clock', () => vm.debug_apu_clock());
    }
}

export class Debug extends Memoizable {
    constructor (vm: Emulator) {
        super();
        this.memoize('apu', () => new DebugApu(vm));
        this.memoize('bus', () => new DebugBus(vm));
        this.memoize('cartridge', () => new DebugCartridge(vm));
        this.memoize('clock', () => vm.debug_clock());
        this.memoize('cpu', () => new DebugCpu(vm));
        this.memoize('ppu', () => new DebugPpu(vm));
        this.memoize('disassembly', () => new DebugDisassembly(vm));
        this.memoize('frame', () => vm.debug_ppu_frame());
        this.memoize('oam', () => vm.debug_ppu_oam());
        this.memoize('palettes', () => vm.debug_ppu_palettes());
        this.memoize('palette', () => vm.debug_ppu_palette());
        this.memoize('nametables', () => vm.debug_ppu_nametables());
        this.memoize('input', () => vm.debug_input());
    }
}
