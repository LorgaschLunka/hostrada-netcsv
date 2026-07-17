use serde::{Serialize, Deserialize};

/// Represents one pixel on the hostrada grid containing it's x and y coords on the grid and the latitude and longitude of it's center 
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HostradaGridPixel {
    pub lat: f64,
    pub lon: f64,
    pub x: u16,
    pub y: u16,
}
