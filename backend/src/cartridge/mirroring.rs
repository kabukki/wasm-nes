use wasm_bindgen::prelude::wasm_bindgen;
use std::fmt;
use serde::Serialize;

#[wasm_bindgen]
#[derive(Debug, PartialEq, Copy, Clone, Serialize)]
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
