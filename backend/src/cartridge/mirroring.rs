use std::fmt;

#[derive(Debug, PartialEq, Copy, Clone, serde::Serialize)]
pub enum Mirroring {
    OneScreenLower,
    OneScreenUpper,
    Horizontal,
    Vertical,
    FourScreen,
}

impl fmt::Display for Mirroring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mirroring::OneScreenLower   => write!(f, "One screen (lower)"),
            Mirroring::OneScreenUpper   => write!(f, "One screen (upper)"),
            Mirroring::Horizontal       => write!(f, "Horizontal"),
            Mirroring::Vertical         => write!(f, "Vertical"),
            Mirroring::FourScreen       => write!(f, "4-screen"),
        }
    }
}
