use std::{fs, io, path};
use anyhow::Context;
use log::warn;
use crate::{
    dates_and_times::YearMonth, hostrada_variable::HostradaVar, misc::green_spinner,
};

/// Handles the download of exaktly one file identified by variable and date, using the supplied client
pub fn download_file(variable: &HostradaVar, date: YearMonth, install_dir: &path::PathBuf, client: &reqwest::blocking::Client) -> anyhow::Result<()> {
    let spinner = green_spinner();

    let mut download_link = variable.link()
        .with_context(|| format!("Failed to extract download link for variable {variable}"))?;

    let filename = format!("{}_1hr_HOSTRADA-v1-0_BE_gn_{}{:02}0100-{}{:02}{:02}23.nc", variable.abbr(), date.year, date.month, date.year, date.month, date.days_in_month());
    
    download_link.push_str(&filename);

    let mut response = client
        .get(&download_link)
        .send()?
        .error_for_status()?;

    let size = if let Some(size) = response.content_length() {
        spinner.set_message(format!("Downloading {} ({:.02}mb)...", &filename, (size as f64/1000000.0)));
        Some(size)
    } else {
        warn!("Unable to get filesize. This could be a sign, that the file is corrupted. Could also be fine.");
        spinner.set_message(format!("Downloading {} (Unknown)...", &filename));
        None
    };
    
    let mut inner_install_dir = install_dir.clone();
    inner_install_dir.push(&filename);
    
    let mut active_file = ActiveFile::new(inner_install_dir);
    let mut file = fs::File::create(&active_file.path)
        .with_context(|| format!("Could not create file {}", active_file.path.display()))?;

    let start_download = std::time::Instant::now();
    io::copy(&mut response, &mut file)
        .with_context(|| format!("while streaming to {}. Could be a network error", active_file.path.display()))?;
    let download_elapsed = start_download.elapsed().as_secs_f32();

    active_file.complete();

    if let Some(size) = size {
        spinner.finish_with_message(format!("Downloading {}...Done ({:.02} mb/s)", &filename, (size as f32/download_elapsed)/1000000.0));
    } else {
        spinner.finish_with_message(format!("Downloading {}...Done", &filename));
    }
    
    Ok(())
}


/// Little helper struct for the file that is currently written to. Implements drop to be dropped if anything goes wrong without the file being completed.
struct ActiveFile {
    path: path::PathBuf,
    completed: bool,
}

impl ActiveFile {
    /// Create a new, uncompleted active file
    pub fn new(path: path::PathBuf) -> Self {
        Self { path, completed: false }
    }

    /// Complete this file (i.e. set completed = true)
    pub fn complete(&mut self) {
        self.completed = true;
    }

    // pub fn completed(&self) -> bool {
    //     self.completed
    // }
}

impl Drop for ActiveFile {
    fn drop(&mut self) {
        if !self.completed {
            let _ = fs::remove_file(&self.path);
        }       
    }
}