use crate::ppu::palette::PALETTE;

pub struct Tilemap {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u8>,
}

impl Tilemap {
    pub fn new (width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * 8 * height * 8 * 4],
        }
    }

    /**
     * Write a pixel at given coordinates.
     */
    pub fn write (&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        let n = x + (8 * self.width * y);

        self.buffer[4 * n + 0] = color.0;
        self.buffer[4 * n + 1] = color.1;
        self.buffer[4 * n + 2] = color.2;
        self.buffer[4 * n + 3] = 255;
    }

    /**
     * Write a 8x8 tile at given coordinates, applying the given palette.
     */
    pub fn write_tile (&mut self, x: usize, y: usize, tile: &[u8], palette: &[u8]) {
        for tile_y in 0..8 {
            for tile_x in 0..8 {
                self.write(
                    8 * x + tile_x,
                    8 * y + tile_y,
                    PALETTE[palette[tile[tile_x + tile_y * 8] as usize] as usize],
                );
            }
        }
    }
}
