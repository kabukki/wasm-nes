pub struct Timing {
    pub name: String,
    pub master: f64,
    pub ppu: f64,
    pub cpu: f64,
}

pub const NTSC: Timing = Timing {
    name: "NTSC",
    master: 21_477_272.0,
    ppu:    21_477_272.0 / 4.0,
    cpu:    21_477_272.0 / 12.0,
};


enum Mode { ntsc, pal }

impl Mode {
    fn value (&self) -> f64 {
        21_477_272.0
    }
}
