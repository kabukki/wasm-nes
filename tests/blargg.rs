use wasm_nes::nes::Nes;

macro_rules! run {
    ($path:expr) => {
        match crate::run(include_bytes!($path)) {
            Ok (_) => {
                println!("Passed !");
            },
            Err ((code, message)) => {
                println!("{}", message);
                panic!("Failed ({:02X})", code);
            },
        }
    };
}

fn read_string (nes: &mut Nes) -> String {
    let mut bytes: Vec<u8> = vec![];

    for address in 0x6004.. {
        let byte = nes.read(address);

        if byte > 0 {
            bytes.push(byte);
        } else {
            break;
        }
    }

    String::from_utf8(bytes).unwrap()
}

fn run (rom: &[u8]) -> Result<(), (u8, String)> {
    let mut nes = Nes::new(rom.to_vec(), 48_000.0);

    loop {
        nes.cycle_until_frame();

        // Test magic numbers
        if &[nes.read(0x6001), nes.read(0x6002), nes.read(0x6003)] == &[0xDE, 0xB0, 0x61] {
            match nes.read(0x6000) {
                0x00 => {
                    return Ok(())
                },
                0x80 => {}, // Running
                0x81 => {
                    unimplemented!("Reset");
                },
                result => {
                    return Err((result, read_string(&mut nes)))
                },
            }
        }
    }
}

mod ppu {
    mod ppu_vbl_nmi {
        #[test]
        fn vbl_basics () {
           run!("roms/ppu/ppu_vbl_nmi/rom_singles/01-vbl_basics.nes");
        }

        #[test]
        fn vbl_set_time () {
           run!("roms/ppu/ppu_vbl_nmi/rom_singles/02-vbl_set_time.nes");
        }

        #[test]
        fn vbl_clear_time () {
           run!("roms/ppu/ppu_vbl_nmi/rom_singles/03-vbl_clear_time.nes");
        }

        #[test]
        fn nmi_control () {
           run!("roms/ppu/ppu_vbl_nmi/rom_singles/04-nmi_control.nes");
        }

        #[test]
        fn nmi_timing () {
           run!("roms/ppu/ppu_vbl_nmi/rom_singles/05-nmi_timing.nes");
        }

        #[test]
        fn suppression () {
           run!("roms/ppu/ppu_vbl_nmi/rom_singles/06-suppression.nes");
        }

        #[test]
        fn nmi_on_timing () {
           run!("roms/ppu/ppu_vbl_nmi/rom_singles/07-nmi_on_timing.nes");
        }

        #[test]
        fn nmi_off_timing () {
           run!("roms/ppu/ppu_vbl_nmi/rom_singles/08-nmi_off_timing.nes");
        }

        #[test]
        fn even_odd_frames () {
           run!("roms/ppu/ppu_vbl_nmi/rom_singles/09-even_odd_frames.nes");
        }

        #[test]
        fn even_odd_timing () {
           run!("roms/ppu/ppu_vbl_nmi/rom_singles/10-even_odd_timing.nes");
        }
    }

    mod ppu_sprite_overflow {
        #[test]
        fn basics () {
           run!("roms/ppu/ppu_sprite_overflow/rom_singles/01-basics.nes");
        }

        #[test]
        fn details () {
           run!("roms/ppu/ppu_sprite_overflow/rom_singles/02-details.nes");
        }

        #[test]
        fn timing () {
           run!("roms/ppu/ppu_sprite_overflow/rom_singles/03-timing.nes");
        }

        #[test]
        fn obscure () {
           run!("roms/ppu/ppu_sprite_overflow/rom_singles/04-obscure.nes");
        }

        #[test]
        fn emulator () {
           run!("roms/ppu/ppu_sprite_overflow/rom_singles/05-emulator.nes");
        }
    }
}
