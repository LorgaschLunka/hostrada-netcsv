use std::{fs, io::{self, BufWriter, Write}, path, time::Instant};
use log::{warn};
use indicatif::{ProgressBar, ProgressStyle};
use chrono::Duration;

use crate::{
    hostrada_dataset::HostradaDataset,
    error::HostradaError,
    misc::green_spinner,
};

/// Converts all values of a file into csv format, in order time, y, x (this means that first, all x values for the first timestamp for the first y are displayed and so on).
/// The use of this function is strongly discouraged, as >700 files are created (1 for each hour).
/// Currently, x and y for each value are hard coded (x 0-719, y 0-937)
pub fn convert_all_values(files: Vec<path::PathBuf>, output_dir: path::PathBuf) -> Result<(), HostradaError> {
    warn!("The use of this functionality is strongly discouraged! Expect >700 csv files!");
    let spinner = green_spinner();
    let (datasets, _, var_id) = setup(files, None, None)?;

    let mut dataset_count = 1;

    let x_y: Vec<(usize, usize)> = (0..938)
        .flat_map(|y| (0..720).map(move |x| (x, y)))
        .collect();

    spinner.finish();
    // iterate through datasets and pixels and write data to csv
    for dataset in &datasets {
        let mut current = dataset.start_date().unwrap().0.clone();
        let end_date = dataset.end_date().unwrap().0.clone();

        // progressbar stuff
        let diff = (end_date-current).num_hours();
        
        let pb = ProgressBar::new(diff as u64);

        pb.set_style(ProgressStyle::with_template(
            "{msg} {spinner:.green} [{bar:40.green}] {percent}% [{eta}]"
        ).unwrap().progress_chars("->o"));

        pb.set_message(format!("Converting dataset {}/{}", dataset_count, datasets.len()));

        pb.enable_steady_tick(std::time::Duration::from_millis(100));



        let start = Instant::now();
        // actual writing and increment progressbar
        while current <= end_date {
            // setup hourly file
            let mut inner_output_dir = output_dir.clone();
            let hourly_filename = path::PathBuf::from(&format!("{}_{}_converted.csv", var_id, current));
            inner_output_dir.push(hourly_filename);

            let hourly_file = fs::File::create(inner_output_dir)?;
            let mut wtr = BufWriter::new(hourly_file);
            writeln!(wtr, "{},pixel_x, pixel_y", var_id)?;


            // let inner_start = Instant::now();

            let hourly_data  = dataset.file().variable(&var_id).unwrap().get::<f32, _>((dataset.time_index(&current).unwrap(), .., ..))?;

            let scale_factor = dataset.scale_factor(&var_id);
            for (idx, val) in hourly_data.iter().enumerate() {
                // apply scale factor
                // if let Some(factor) = scale_factor {
                //     let val = val * factor as f32;
                //     writeln!(wtr, "{},{}", current, val).unwrap();
                // } else {
                //     writeln!(wtr, "{},{}", current, val).unwrap();
                // };

                // KRASSER

                let val = if let Some(factor) = scale_factor {
                    val * factor as f32
                } else {
                    *val
                };

                writeln!(wtr, "{:.2},{},{}", val, &x_y[idx].0, &x_y[idx].1).unwrap();
            }
            // println!("One date iteration: {:?}", inner_start.elapsed());


            current += Duration::hours(1);
            pb.inc(1); // let this stay here to reduce computing needed for pb

        }

        println!("Time for one dataset: {:?}", start.elapsed());
        pb.finish();
        dataset_count += 1;

    }

    Ok(())
}

pub fn convert_pixel(files: Vec<path::PathBuf>, x: usize, y: usize, mut output_dir: path::PathBuf) -> Result<(), HostradaError> {
    let spinner = green_spinner();
    let (datasets, wtr, var_id) = setup(files, Some((x, y)), Some(&mut output_dir))?;
    let mut wtr = wtr.unwrap();
    writeln!(wtr, "timestamp,{}", var_id)?;
    spinner.finish();
    // iterate through datasets and write data to csv
    for dataset in datasets {
        let mut current = dataset.start_date().unwrap().0.clone();
        let end_date = dataset.end_date().unwrap().0.clone();
        
        // progressbar stuff
        let diff = (end_date-current).num_hours();
        
        let pb = ProgressBar::new(diff as u64);
        pb.set_style(ProgressStyle::with_template(
            "{spinner:.green} [{bar:40.green}] {percent}% [{eta}] {msg}"
        ).unwrap().progress_chars("->o"));

        pb.set_message(format!("Converting pixel ({}/{}) {:?}...", x, y, dataset.file().path().unwrap()));

        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let start = Instant::now();
        // actual writing and increment progressbar
        while current <= end_date{

            let val = dataset.value_at(&var_id, &current, x, y).unwrap();

            // also fix floating point error
            writeln!(wtr, "{},{:.2}", current, val)?;

            current += Duration::hours(1);
            pb.inc(1);
        }

        pb.finish_with_message(format!("Done. ({:.02}s)", start.elapsed().as_secs_f32()));
    }

    Ok(())
}

fn setup(files: Vec<path::PathBuf>, x_y: Option<(usize, usize)>, output_dir: Option<&mut path::PathBuf>) -> Result<(Vec<HostradaDataset>, Option<io::BufWriter<fs::File>>, String), HostradaError> {
    // can also be 1 dataset in single file mode
    let datasets = HostradaDataset::from_filelist_same_grids(files)?;
    
    // use first dataset to extract variable_id for output filename
    let first = datasets.first().unwrap();
    let var_id = first.var_id().unwrap_or_else(|| {
        eprintln!("File {:?} does not seem to have any variable id. Is this a hostrada file? If everything should work, this is a bug. If this is indeed not a hostrada hourly netcdf file, this is expected behavior.", first.file().path());
        std::process::exit(1);
    });


    if let Some(dir) = output_dir {

        // use last dataset for file name
        let last = datasets.last().unwrap();
        let output_filename = if let Some(coords) = x_y {
            path::PathBuf::from(&format!("{}_Pix_{}_{}_{:?}-{:?}_converted.csv", var_id, coords.0, coords.1, first.start_date().unwrap().0, last.end_date().unwrap().0))
        } else {
            return Err(std::io::Error::other("Internal Error: did not pass x_y while passing output_dir.").into());
        };
        dir.push(output_filename);
        let file = fs::File::create(dir)?;
        let wtr = io::BufWriter::new(file);
        return Ok((datasets, Some(wtr), var_id))
    };

    Ok((datasets, None, var_id))


}
