use std::num::ParseFloatError;

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

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error(transparent)]
    RequestErr(#[from] reqwest::Error),
    
    #[error(transparent)]
    MetadataToStrErr(#[from] reqwest::header::ToStrError),

    #[error(transparent)]
    ParseFloatErr(#[from] ParseFloatError),

    #[error("I/O error while accessing {path}: {source}")]
    IOErr {
        source: std::io::Error,
        path: std::path::PathBuf,
    },

    #[error(transparent)]
    ReaderWriterErr(#[from] std::io::Error),

    #[error(transparent)]
    ConfigError(#[from] ConfigError),
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Could not determine user config directory")]
    DirNotFound,

    #[error(transparent)]
    IOErr(#[from] std::io::Error),

    #[error("toml error with {path}: {source}")]
    TomlErr {
        source: toml::de::Error,
        path: std::path::PathBuf,
    },

    #[error("invalid config: value {val} for {var} {msg}")]
    InvalidConfig {
        val: String,
        var: String,
        msg: String,
    }
}