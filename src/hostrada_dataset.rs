use std::{
    collections::HashMap, path, sync::Arc, time::Instant
};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use log::{debug, warn};
use num_traits::{Float, FromPrimitive};
use netcdf::AttributeValue;

use crate::{
    config::Config, dates_and_times::parse_time, error::HostradaError, hostrada_pixel::HostradaGridPixel, hostrada_variable::HostradaVar, misc::{WithDistance, haversine},
};



/// One Dataset with its associated file.
/// The grid is an array with size 675360 containing all HostradaGridPixels.
/// This is subject to change if it takes way to long to calculate the grid each time.
pub struct HostradaDataset {
    file: netcdf::File,
    pub grid: HashMap<(u16, u16), HostradaGridPixel>,
    pub time_map: HashMap<DateTime<Utc>, f64>,
}

/// This struct can be used, if the dataset wants to be used without grid or time map. This could be the case, if only some metadata wants to be
pub struct HostradaFile {
    file: netcdf::File,
}
impl HostradaFile {
    pub fn new<P>(file_path: P) -> Result<HostradaFile, HostradaError>
    where 
        P: AsRef<std::path::Path>
    {
        let file_path = file_path.as_ref();
        let file = netcdf::open(file_path)?;

        Ok(Self { file })
    }

    pub fn file(&self) -> &netcdf::File {
        &self.file
    }

    /// Shortcut for file().path()
    pub fn path(&self) -> Result<path::PathBuf, HostradaError> {
        Ok(self.file.path()?)
    }

    /// Shortcut for the filename of the file
    pub fn file_name(&self) -> Result<std::ffi::OsString, HostradaError>{
        Ok(self.file.path()?.file_name().expect("unreachable").to_os_string())
    }

    /// Check the value of the netcdf attribute 'variable_id'. This is important for
    /// the program to know, what variable it is looking at. Example: 'tas' for 'air temperature mean'.
    /// None is returned if attribute variable_id couldn't be found, no value is present or the value is not uniformly Str(some_str).
    /// As the program will likely crash if this happens, it is recommended to put a warning somewhere if this function returns None.
    pub fn var_id(&self) -> Option<String> {
        match self.file.attribute("variable_id")?.value().ok()? {
            AttributeValue::Str(variable_id) => return Some(variable_id),
            _ => {
                return None;
            }
        }
    }

    /// Build the HostradaVar struct out of this file
    pub fn hostrada_var(&self) -> Option<HostradaVar> {
        let var_id = self.var_id()?;

        HostradaVar::from_abbr(&var_id)
    }

}
impl HostradaDataset {
    pub fn new<P>(file_path: P) -> Result<HostradaDataset, HostradaError>
    where 
        P: AsRef<std::path::Path>
    {
        let file_path = file_path.as_ref();
        let file = netcdf::open(file_path)?;
        
        let grid = calculate_grid(&file)?; 

        let time_map = calculate_time_map(&file)?;

        Ok(Self { file, grid, time_map })
    }

    /// Create HostradaDataset objects by filelist, creating a new grid for each
    pub fn from_filelist<I>(files: I) -> Result<Vec<HostradaDataset>, HostradaError>
    where 
        I: ParallelIterator::<Item: AsRef<std::path::Path>>,
    {
        warn!("No clue why this does not fail. If this still does not fail, this is good.");
        files
        .map(|path| HostradaDataset::new(path))
        .collect()
    }

    /// Creates multiple HostradaDatasets from multiple files. 
    /// The same grid is used across all results.
    /// Returns None, if a file couldn't be opened by netcdf::open or the input vector is empty.
    pub fn from_filelist_same_grids<T>(files: Vec<T>) -> Result<Vec<HostradaDataset>, HostradaError>
    where 
        T: AsRef<std::path::Path>,
    {
        let first_file = netcdf::open(files.first().unwrap())?;

        // calculate grid and time map with first file
        let grid = calculate_grid(&first_file)?;

        // return HostradaDatasets
        files
            .iter()
            .map(|file| {
                let opened_file = netcdf::open(file)?;
                let time_map = calculate_time_map(&opened_file)?;
                Ok(HostradaDataset {
                    file: opened_file,
                    grid: grid.clone(),
                    time_map: time_map,
                })
            })
            .collect()
    }

    pub fn file(&self) -> &netcdf::File {
        &self.file
    }

    /// Check the value of the netcdf attribute 'variable_id'. This is important for
    /// the program to know, what variable it is looking at. Example: 'tas' for 'air temperature mean'.
    /// None is returned if attribute variable_id couldn't be found, no value is present or the value is not uniformly Str(some_str).
    /// As the program will likely crash if this happens, it is recommended to put a warning somewhere if this function returns None.
    pub fn var_id(&self) -> Option<String> {
        match self.file.attribute("variable_id")?.value().ok()? {
            AttributeValue::Str(variable_id) => return Some(variable_id),
            _ => {
                return None;
            }
        }
    }

    /// Return the earliest date in the dataset and its corresponding days since origin.
    /// ## Panics
    /// Panics if partial_cmp returns None, which only happens if the time_map is empty which should absolutely not happen.
    pub fn start_date(&self) -> Option<(&DateTime<Utc>, &f64)> {
        self
            .time_map
            .iter()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).expect("Couldn't sort dates."))
    }

    /// Return the latest date in the dataset and its corresponding days since origin.
    /// ## Panics
    /// Panics if partial_cmp returns None, which only happens if the time_map is empty which should absolutely not happen.
    pub fn end_date(&self) -> Option<(&DateTime<Utc>, &f64)> {
        self
            .time_map
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("Couldn't sort dates."))
    }

    /// Get HostradaGridPixel by x and y index.
    pub fn pixel_at(&self, x: u16, y: u16) -> Option<&HostradaGridPixel> { 
        self.grid
            .get(&(x, y))
    }

    /// Check if given latitude and longitude is inside the hostrada grid.
    /// Calculates nearest pixel (with the fast function) and if the distance is > 500m, the requested coord is not contained
    /// Also returns distance to nearest pixel.
    pub fn contains_coord(&self, lat: f64, lon: f64) -> WithDistance<bool> {
        let ret = self.nearest_pixel_at_coord_approx(lat, lon);
        let dist = ret.distance;
        if dist > 500.0 {
            return WithDistance { value: false, distance: dist };
        }
        WithDistance { value: true, distance: dist}
    }

    /// Check if given pixel x and y are inside the hostrada grid.
    pub fn contains_pixel(&self, x: usize, y: usize) -> Result<bool, HostradaError> {
        if (0..=self.max_x()?).contains(&x) && (0..=self.max_y()?).contains(&y) {
            return Ok(true);
        }
        Ok(false)
    }

    /// Calculate the nearest pixel of given coordinates.
    /// This is calculated via haversine distance to the center of the pixels. Smallest distance = nearest pixel.
    pub fn nearest_pixel_at_coord(&self, lat: f64, lon: f64) -> WithDistance<&HostradaGridPixel> {
        // Nicht optimiert
        
        let mut min_dist: (&(u16, u16), f64) = (&(100, 100), 100000000.0); // arbitrary values to ensure that the first shadow is swapping the values 
        for (key, pixel) in &self.grid {
        
                let d = haversine((pixel.lat, pixel.lon), (lat, lon), 6371000.0f64);

                min_dist = if d < min_dist.1 { (key, d) } else { min_dist };
           };

        let pixel = self.grid.get(min_dist.0).unwrap();

        WithDistance { value: pixel, distance: min_dist.1 }
    }


    /// Calculates the nearest pixel for a coord with an optimisation: Start with large quadrants, then get smaller to reduce amount of haversine computations.
    /// Is an approximation, because the curvature of the earth could lead to a misidentified pixel. This should not be that critical or not even happening at all.
    pub fn nearest_pixel_at_coord_approx(&self, lat: f64, lon: f64) -> WithDistance<&HostradaGridPixel> {

        // max x and y for current square
        let mut max_x = 719.0;
        let mut max_y = 937.0;

        // bottom left corner of current square
        let mut base_x = 0.0;
        let mut base_y = 0.0;

        // do this loop 8 times to reduce eligible space where the pixel could be 
        for _i in 0..8 { 
            // calculate distances to each quadrant center 
            let q1_center = self.pixel_at((base_x + 0.75*max_x) as u16, (base_y + 0.75*max_y) as u16).unwrap();
            let q2_center = self.pixel_at((base_x + 0.25*max_x) as u16, (base_y + 0.75*max_y) as u16).unwrap();
            let q3_center = self.pixel_at((base_x + 0.25*max_x) as u16, (base_y + 0.25*max_y) as u16).unwrap();
            let q4_center = self.pixel_at((base_x + 0.75*max_x) as u16, (base_y + 0.25*max_y) as u16).unwrap();

            let q1_dist = haversine((q1_center.lat, q1_center.lon), (lat, lon), 6371000.0);
            let q2_dist = haversine((q2_center.lat, q2_center.lon), (lat, lon), 6371000.0);
            let q3_dist = haversine((q3_center.lat, q3_center.lon), (lat, lon), 6371000.0);
            let q4_dist = haversine((q4_center.lat, q4_center.lon), (lat, lon), 6371000.0);

            let candidates = [
                ("q1", q1_dist),
                ("q2", q2_dist),
                ("q3", q3_dist),
                ("q4", q4_dist),
            ];
            
            let (quad, _) = candidates.iter()
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap();
            
            // check which quadrant the searched point is in and adjust base_x and base_y accordingly
            (base_x, base_y) = match *quad { 
                "q1" => (base_x + max_x/2.0, base_y + max_y/2.0),
                "q2" => (base_x, base_y + max_y/2.0),
                "q3" => (base_x, base_y),
                "q4" => (base_x + max_x/2.0, base_y),
                _ => panic!("Should have a quadrant to locate the searched coords in.")
            };

            max_x = max_x/2.0;
            max_y = max_y/2.0;
        }

        // residual square should be a square of around 3 pixels size -> search distance to all 9 pixels -> win

        let mut min_dist: ((u16, u16), f64) = ((100, 100), 1000000.0); // arbitrary values to ensure that the first shadow is swapping the values 
        for x in base_x as u16..=(base_x+max_x) as u16 {
            for y in base_y as u16..=(base_y+max_y) as u16{

                let pixel = self.pixel_at(x, y).unwrap();
                let d = haversine((pixel.lat, pixel.lon), (lat, lon), 6371000.0);

                let key_x = x.clone();
                let key_y = y.clone();
                min_dist = if d < min_dist.1 { ((key_x, key_y), d) } else { min_dist };
            }
        }

        let nearest_pixel = self.pixel_at(min_dist.0.0, min_dist.0.1).unwrap();

        WithDistance { value: nearest_pixel, distance: min_dist.1 }
    }


    /// Get the index of a timestamp in the time hashmap
    pub fn time_index(&self, timestamp: &DateTime<Utc>) -> Option<usize> {

        let time0 = self.file().variable("time")?.get_value::<f64, _>([0]).ok()?;

        let time = self.time_map.get(timestamp)?;

        let diff = ((time - time0) * 24.0).round();
        if diff >= 0.0 {
            return Some(diff as usize)
        } else {
            return None
        };

        
    }


    /// returns the value of the variable at timestamp of the the pixel defined by x and y coordinates.
    /// Timestamp must only be accurate by hour, as only then the timestamp is included in the dataset.
    /// Applies the scale factor, if there is any.
    pub fn value_at(&self, var_name: &str, timestamp: &DateTime<Utc>, x: usize, y: usize) -> Option<f32> {

        let var = self.file.variable(var_name)?;
        
        let time_index = self.time_index(timestamp)?;

        let val = var.get_value::<f32, _>((time_index, y, x)).ok()?;

        // check for scale factor
        self.apply_scale_factor(val, var_name)
    }

    pub fn scale_factor(&self, var_name: &str) -> Option<f64> {
        if let Some(attr) = self.file()
            .variable(var_name)
            .unwrap()
            .attribute("scale_factor")
        {
            if let Ok(attr_value) = attr.value() {
                let scale_factor = match attr_value {
                    AttributeValue::Double(v) => Some(v),
                    AttributeValue::Float(v) => f64::from_f32(v),
                    AttributeValue::Int(v) => f64::from_i32(v),
                    _ => None,
                };
                return scale_factor;
            }
        }
        None
    }

    pub fn apply_scale_factor<T>(&self, val: T, var_name: &str) -> Option<T>
    where 
        T: Float + FromPrimitive,
    {
        if let Some(attr) = self.file()
            .variable(var_name)?
            .attribute("scale_factor")
        {
            if let Ok(attr_value) = attr.value() {
                let scale_factor = match attr_value {
                    AttributeValue::Double(v) => T::from_f64(v),
                    AttributeValue::Float(v) => T::from_f32(v),
                    AttributeValue::Int(v) => T::from_i32(v),
                    _ => None,
                };
        
                if let Some(scale_factor) = scale_factor {
                    return Some(val * scale_factor);
                }
            }
        }
        
        Some(val)
    }

    /// Returns the maximum value that x can be. This will always be 719, as this is the logic of the hostrada data; however, this method reduces hard coded values.
    /// ## Errors
    /// - fails if the x variable cannot be found in the netcdf file
    pub fn max_x(&self) -> Result<usize, HostradaError> {
        let len  = self
            .file()
            .variable("X")
            .ok_or(HostradaError::VarNotFound { var: "X".to_string() })?
            .len();
        Ok(len - 1) // - 1 because it starts at 0)
    }

    /// Returns the maximum value that y can be. This will always be 937, as this is the logic of the hostrada data; however, this method reduces hard coded values.
    /// ## Errors
    /// - fails if the y variable cannot be found in the netcdf file
    pub fn max_y(&self) -> Result<usize, HostradaError> {
        let len  = self
            .file()
            .variable("Y")
            .ok_or(HostradaError::VarNotFound { var: "Y".to_string() })?
            .len();
        Ok(len - 1) // - 1 because it starts at 0)
    }

    pub fn origin(&self) -> Result<String, HostradaError> {
        let attr_val = self
            .file()
            .variable("time").ok_or(HostradaError::VarNotFound { var: "time".to_string() })?
            .attribute("units").ok_or(HostradaError::AttrNotFound { attr: "units".to_string() })?
            .value()?;

        if let AttributeValue::Str(value) = attr_val {
            return Ok(value.split_whitespace().last().unwrap().to_owned());
        }
        
        Err(HostradaError::AttrNotFound { attr: "ANY ATTRIBUTE TYPE".to_string() })

    }
}

/// Calculates a hashmap mapping a chrono::Datetime<Utc> to the respective timestamp in the dataset
fn calculate_time_map(file: &netcdf::File) -> Result<HashMap<DateTime<Utc>, f64>, HostradaError> {
    let config: Config = match Config::load() {
        Ok(conf) => conf,
        Err(e) => {
            eprintln!("Couldn't build internal time hashmap due to config_err: {e}");
            std::process::exit(1);
        },
    };
    let time_vals = file
        .variable("time")
        .ok_or(HostradaError::VarNotFound { var: "time".to_string() })?
        .get::<f64,_>(..)?;

    let parsed= time_vals
        .into_iter()
        .map(|val| {
            parse_time(&config.origin, val)
                .map(|time| (time, val))
    })
    .collect::<Result<HashMap<_, _>, _>>();

    return parsed.map_err(|e| HostradaError::ParseError { context: " all the values for the time grid".to_string(), e })
}


/// Calculates a hashmap mapping all coordinates to all HostradaGridPixels of the Hostrada Grid.
fn calculate_grid(file: &netcdf::File) -> Result<HashMap<(u16, u16), HostradaGridPixel>, HostradaError> {

    let start = Instant::now();
    let lat_vals = file
        .variable("lat")
        .ok_or(HostradaError::VarNotFound { var: "lat".to_string() })?
        .get::<f64, _>(..)?; // 4 decimal places so should be precise to ~11m

    let lon_vals = file
        .variable("lon")
        .ok_or(HostradaError::VarNotFound { var:"lon".to_string() })?
        .get::<f64, _>(..)?; // 4 decimal places so should be precise to ~11m
        
    
    debug!("Reading all lat and lon vals for grid took {:?}", start.elapsed());

    let start = Instant::now();

    let ny = lat_vals.shape()[0];
    let nx = lat_vals.shape()[1];

    // MULTITHREADING 
    let lat_vals = Arc::new(lat_vals);
    let lon_vals = Arc::new(lon_vals);

    let grid: HashMap<(u16, u16), HostradaGridPixel> =
        (0..ny)
            .into_par_iter()
            .flat_map_iter({
                move |y| {
                (0..nx).map({
                    let cloned_lat_vals = Arc::clone(&lat_vals);
                    let cloned_lon_vals = Arc::clone(&lon_vals);

                    move |x| {
                    (
                        (x as u16, y as u16),
                        HostradaGridPixel {
                            lat: cloned_lat_vals[[y, x]],
                            lon: cloned_lon_vals[[y, x]],
                            y: y as u16, 
                            x: x as u16,
                        },
                    )
                }})
            }}).collect();

    debug!("Calculating hashmap of all pixels took {:?}", start.elapsed());
    
    Ok(grid)
}
