use std::{fs, io, path};
use log::warn;
use crate::{
    cli::HostradaVar,
    dates_and_times::YearMonth,
    error::DownloadError,
    misc::green_spinner,
};

/// Handles the download of exaktly one file identified by variable and date, using the supplied client
pub fn download_file(variable: &HostradaVar, date: YearMonth, install_dir: &path::PathBuf, client: &reqwest::blocking::Client) -> anyhow::Result<()> {
    let spinner = green_spinner();

    let mut download_link = variable.link()?;
    let filename = format!("{}_1hr_HOSTRADA-v1-0_BE_gn_{}{:02}0100-{}{:02}{:02}23.nc", variable.abbr(), date.year, date.month, date.year, date.month, date.days_in_month());
    
    download_link.push_str(&filename);

    let mut response = client
        .get(&download_link)
        .send()?
        .error_for_status()?;

    if let Some(size) = response.content_length() {
        spinner.set_message(format!("Downloading {} ({:.02}mb)...", &filename, (size as f64/1000000.0)));
    } else {
        warn!("Unable to get filesize. This could be a sign, that the file is corrupted. Could also be fine.");
        spinner.set_message(format!("Downloading {} (Unknown)...", &filename));
    }
    
    let mut inner_install_dir = install_dir.clone();
    inner_install_dir.push(&filename);
    
    let mut file = fs::File::create(&inner_install_dir)
        .map_err(|e| DownloadError::IOErr {
            source: e,
            path: inner_install_dir,
        })?;

    io::copy(&mut response, &mut file)
        .map_err(DownloadError::ReaderWriterErr)?;

    spinner.finish_with_message(format!("Downloading {}...Done", &filename));

    Ok(())
}
