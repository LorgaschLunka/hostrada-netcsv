use thiserror;

use crate::hostrada_pixel::HostradaGridPixel;

/// Error that should be used if anything requested is out of the hostrada grid span
#[derive(Debug, thiserror::Error)]
pub enum HostradaError {
    #[error("Coordinates {lat}, {lon} are out of hostrada grid bounds: {dist:.2}m to nearest pixel ({}, {}) at {:.4}, {:.4}", nearest_pixel.x, nearest_pixel.y, nearest_pixel.lat, nearest_pixel.lon)]
    OutOfBounds {
        lat: f64,
        lon: f64,
        dist: f64, // distance to nearest hostrada pixel
        nearest_pixel: HostradaGridPixel,
    },

    #[error(transparent)]
    NetcdfIo(#[from] netcdf::Error),

    #[error(transparent)]
    StdIo(#[from] std::io::Error),

    #[error("Unable to find variable {var} in the netcdf file")]
    VarNotFound {
        var: String,
    },

    #[error("Unable to find attribute {attr} in the netcdf file")]
    AttrNotFound {
        attr: String,
    },

    #[error("Error parsing{context}: {e}")]
    ParseError {
        context: String,
        e: chrono::ParseError,
    }
}
