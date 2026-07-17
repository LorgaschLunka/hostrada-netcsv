use crate::{
    error::HostradaError,
    hostrada_dataset::HostradaDataset,
    hostrada_pixel::HostradaGridPixel,
    misc::WithDistance,
};
use std::path;

pub fn pixel(ref_file: path::PathBuf, lat: f64, lon: f64) -> Result<WithDistance<HostradaGridPixel>, HostradaError> {
    let dataset = HostradaDataset::new(ref_file)?;

    let contains = dataset.contains_coord(lat, lon);
    if !contains.value {
        let nearest = dataset.nearest_pixel_at_coord(lat, lon).value;
        return Err(
            HostradaError::OutOfBounds {
            lat,
            lon,
            dist: contains.distance,
            nearest_pixel: *nearest,
            }
        );
    }

    let res = dataset.nearest_pixel_at_coord_approx(lat, lon);
    let pixel = res.value.to_owned();
    Ok(WithDistance { value: pixel, distance: res.distance })
    // dataset.nearest_pixel_at_coord(lat, lon)
}