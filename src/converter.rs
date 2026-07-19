use std::{
    fs,
    io::{self, BufWriter, Write}, 
    path,
};
use log::{warn};
use indicatif::{ProgressBar, ProgressStyle};
use chrono::{
    Duration,
    Local
};
use rayon::prelude::*;

use crate::{
    hostrada_variable::HostradaVar,
    hostrada_dataset::HostradaDataset,
    misc::green_spinner,
};

/// Converts all values of a file into csv format, in order time, y, x (this means that first, all x values for the first timestamp for the first y are displayed and so on).
/// The use of this function is strongly discouraged, as >700 files are created (1 for each hour).
/// Currently, x and y for each value are hard coded (x 0-719, y 0-937)
pub fn convert_all_values(files: Vec<path::PathBuf>, output_dir: &path::PathBuf, skip_nan: bool) -> anyhow::Result<()> {
    warn!("The use of this functionality is strongly discouraged! Expect >700 csv files!");
    let spinner = green_spinner();
    let datasets = HostradaDataset::from_filelist_same_grids(files)?;

    let x_y: Vec<(usize, usize)> = (0..938)
        .flat_map(|y| (0..720).map(move |x| (x, y)))
        .collect();
    spinner.finish();

    // iterate through datasets and pixels and write data to csv
    for (idx, dataset) in datasets.iter().enumerate() {
        // mk dir for each dataset
        let dataset_dir = &output_dir.join(dataset.file().path()?.file_stem().expect("This should not be possible!"));
        fs::create_dir(dataset_dir)?;

        let mut current = dataset.start_date().unwrap().0.clone();
        let end_date = dataset.end_date().unwrap().0.clone();

        // progressbar stuff
        let diff = (end_date-current).num_hours() as u64;
        let pb = converter_pb(diff);
        pb.set_message(format!("Converting dataset {}/{}", idx+1, datasets.len()));

        // extract var_id for each dataset to support different variables in same directory
        let var_id = dataset.var_id().ok_or(anyhow::anyhow!("Did not found a var id for merged file"))?;

        // actual writing and increment progressbar
        while current <= end_date {
            // setup hourly file
            let inner_output_dir = dataset_dir.join(path::PathBuf::from(&format!("{}_{}_converted.csv", var_id, current)));

            let hourly_file = fs::File::create(inner_output_dir)?;
            let mut wtr = BufWriter::new(hourly_file);
            writeln!(wtr, "{},pixel_x, pixel_y", var_id)?;

            let hourly_data  = dataset
                .file()
                .variable(&var_id)
                .unwrap()
                .get::<f32, _>((dataset.time_index(&current).unwrap(), .., ..))?;

            let scale_factor = dataset.scale_factor(&var_id);
            for (idx, val) in hourly_data.iter().enumerate() {
                // apply scale factor
                let val = if let Some(factor) = scale_factor {
                    val * factor as f32
                } else {
                    *val
                };
                if skip_nan && (val == -9999.0 as f32) {
                    continue;
                }
                writeln!(wtr, "{:.2},{},{}", val, &x_y[idx].0, &x_y[idx].1).unwrap();
            }

            current += Duration::hours(1);
            pb.inc(1); // let this stay here to reduce computing needed for pb

        }

        pb.finish_with_message(format!("Done with 1 dataset. ({:.02}s)", pb.elapsed().as_secs_f32()));

    }

    Ok(())
}


pub fn convert_pixel(files: Vec<path::PathBuf>, x: usize, y: usize, output_dir: &path::PathBuf, merge: bool) -> anyhow::Result<()> {
    let spinner = green_spinner();
    let datasets = HostradaDataset::from_filelist_same_grids(files)?;
    spinner.finish();
    let mut mode = if merge {
        let var_id = datasets.first().unwrap().var_id().ok_or(anyhow::anyhow!("Did not found a var id for merged file"))?;
        let file_name = output_dir
            .join(format!("merged_{}_pix_{}_{}_{}_merged.csv",var_id, x, y, Local::now().to_rfc3339()));
        let file = fs::File::create(file_name)?;

        let mut wtr = io::BufWriter::new(file);
        writeln!(wtr, "timestamp, {}", var_id)?;

        ConvertMode::Combined(wtr)

    } else {
        ConvertMode::Seperate()
    };

    for dataset in datasets {
        convert_dataset(dataset, x, y, &output_dir, &mut mode)?;
    }
    Ok(())
}

/// Handles conversion of a single dataset. When in Seperate mode, creates its own file and writer. When in Combined mode, the wtr held by ConvertMode::Combined(wtr) is used.
fn convert_dataset(dataset: HostradaDataset, x: usize, y: usize, output_dir: &path::PathBuf, mode: &mut ConvertMode) -> anyhow::Result<()> {
    let mut current = dataset.start_date().unwrap().0.clone();
    let end_date = dataset.end_date().unwrap().0.clone();
    let diff = (end_date-current).num_hours() as u64;

    let path = dataset.file().path()?;

    let var_id = dataset.var_id()
    .ok_or(anyhow::anyhow!("Failed to get variable id for {}", path.display()))?;

    // illegal stuff to make this work without code duplication
    let mut local_wtr;
    let wtr = match mode {
        ConvertMode::Seperate() => {
            let res_file_name = output_dir
                .clone()
                .join(format!("pix_{}_{}_{}.csv", x, y, path.file_stem().unwrap().display()));
            let file = fs::File::create(res_file_name)?;
            local_wtr = io::BufWriter::new(file);

            writeln!(local_wtr, "timestamp, {}", var_id)?;


            &mut local_wtr
        },
        ConvertMode::Combined(wtr) => wtr,
    };
    
    let pb = converter_pb(diff);
    pb.set_message(format!("Converting pixel ({}/{}) {:?}...", x, y, path));

    while current <= end_date {

        let val = dataset.value_at(&var_id, &current, x, y).unwrap_or(-8888.0);

        // also fix floating point error
        writeln!(wtr, "{},{:.2}", current, val)?;

        current += Duration::hours(1);
        pb.inc(1);
    }

    pb.finish_with_message(format!("Done. ({:.02}s)", pb.elapsed().as_secs_f32()));

    Ok(())

}

fn converter_pb(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{bar:40.green}] {percent}% [{eta}] {msg}").unwrap()
        .progress_chars("->o")
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(75));

    pb
}

pub enum ConvertMode {
    Seperate(),
    Combined(BufWriter<fs::File>),
}

/// Compares the files in the directory with their size on the hostrada server.
/// Does not return an error if they don't match, but returns the path of the entry that does not match.
pub fn validate_files(dir: &path::PathBuf) -> anyhow::Result<Option<path::PathBuf>> {
    
    let files: Vec<fs::DirEntry> = fs::read_dir(dir)?
        .collect::<Result<_, _>>()?;
    
    let client = reqwest::blocking::Client::new();
    let res = files
        .into_par_iter()
        .map(|entry| compare_file_size(&entry, &client)
            .map(|result| (entry, result))
        )
        .collect::<Result<Vec<_>, _>>()?;
    
    for tup in &res {
        if !tup.1 {
            return Ok(Some(tup.0.path()));
        }
    }

    Ok(None)
}

/// Compares the filesize of a given DirEntry with the filesize on the dwd hostrada server of the respective file
fn compare_file_size(entry: &fs::DirEntry, client: &reqwest::blocking::Client) -> anyhow::Result<bool> {
    let file_name = entry.file_name();
    let file_name = file_name.to_str().unwrap();

    let var = file_name.split_once("_").unwrap().0;
    let var = HostradaVar::from_abbr(var).unwrap();

    let mut link = var.link()?;
    link.push_str(file_name);

    let response = client
        .get(&link)
        .send()?
        .error_for_status()?;

    let exp_content_length = response.content_length().ok_or(anyhow::anyhow!("Could not get content length"))?;

    let rl_content_length = entry.metadata()?.len();
    Ok(exp_content_length == rl_content_length)
}