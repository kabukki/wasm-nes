use crate::{cartridge, util::Tilemap};

#[derive(serde::Serialize)]
pub struct Cartridge {
    pub ines: cartridge::InesHeader,
    pub ram: Vec<u8>,
    pub rom: Vec<u8>,
    pub pattern_tables: Vec<Vec<u8>>,
}

impl cartridge::Cartridge {
    pub fn get_tile (&self, n: usize) -> Vec<u8> {
        let mut tile = Vec::with_capacity(8 * 8);

        for tile_y in 0..8 {
            let (hi, lo) = (self.chr[n * 16 + tile_y + 8], self.chr[n * 16 + tile_y]);
    
            for tile_x in 0..8 {
                let (hi, lo) = (hi >> (7 - tile_x) & 1, lo >> (7 - tile_x) & 1);
                tile.push(hi << 1 | lo);
            }
        }

        tile
    }

    /**
     * Get the contents of the CHR-ROM pattern tables.
     * Pattern tables contain background graphics (right) and sprite graphics (left)
     * https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
     */
    pub fn get_pattern_tables (&self) -> Vec<Vec<u8>> {
        let mut pt = vec![]; // Vec::with_capacity(2);

        let mut map = Tilemap::new(16, 16);
        let palette = [8, 24, 40, 56];
    
        for n in 0..256 {
            let x = n % 16;
            let y = n / 16;

            let tile = self.get_tile(n);
            map.write_tile(x, y, tile.as_slice(), &palette);
        }
    
        pt.push(map.buffer);

        pt
    }

    pub fn get_pattern_tables_bg (&self) -> Vec<u8> {
        let mut map = Tilemap::new(16, 16);
        let palette = [8, 24, 40, 56];
    
        for n in 256..512 {
            let x = n % 16;
            let y = n / 16;

            let tile = self.get_tile(n);
            map.write_tile(x, y, tile.as_slice(), &palette);
        }
    
        map.buffer
    }
}

// impl Probe<Debug> for Cartridge {
//     fn get_debug (&self) -> Debug {
//         Debug {
//             // ines: self.ines.to_owned(),
//             ram: self.prg_ram.to_vec(),
//             rom: self.prg_rom.to_vec(),
//             // pattern_tables: self.get_pattern_tables(),
//         }
//     }
// }
