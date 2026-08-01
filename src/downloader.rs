use std::{fs, io::{Read, Write}, path};
use anyhow::Context;
use crate::{
    dates_and_times::YearMonth, hostrada_variable::HostradaVar,
};

/// Handles the download of exaktly one file identified by variable and date, using the supplied client
pub fn download_file(variable: &HostradaVar, date: YearMonth, mut install_dir: path::PathBuf, client: &reqwest::blocking::Client) -> anyhow::Result<()> {
    let mut download_link = variable.link()
        .with_context(|| format!("Failed to extract download link for variable {variable}"))?;

    let filename = format!("{}_1hr_HOSTRADA-v1-0_BE_gn_{}{:02}0100-{}{:02}{:02}23.nc", variable.abbr(), date.year, date.month, date.year, date.month, date.days_in_month());
    
    download_link.push_str(&filename);

    let mut response = client
        .get(&download_link)
        .send()?
        .error_for_status()?;

    let size = response.content_length().ok_or(anyhow::anyhow!("No filesize in requested online file."))?;
      
    install_dir.push(&filename);
    
    let mut active_file = ActiveFile::new(install_dir);
    let mut file = fs::File::create(&active_file.path)
        .with_context(|| format!("Could not create file {}", active_file.path.display()))?;

    let start_download = std::time::Instant::now();
    // io::copy but as a self written loop with specified chunk sizes
    const BUF_SIZE: usize = 1024 * 64;
    let mut download_buffer= [0u8; BUF_SIZE]; // 64 kb to write to buffer
    let mut total_written = 0;
    let pb = download_pb(size);
    let init_msg = format!("Downloading {} ({:.02}mb)...", &filename, (size as f64/1000000.0));
    pb.set_message(init_msg.clone());

    loop {
        let bytes = response.read(&mut download_buffer).with_context(|| format!("while streaming to {}. Could be a network error", active_file.path.display()))?;
    
        if bytes == 0 {
            // break if response is empty
            break
        }

        // write all bytes that have been returned from response.read
        file.write_all(&download_buffer[..bytes])?;

        total_written += bytes;
        pb.set_message(format!("{} [{:.02} mb]", init_msg, total_written as f32/1_000_000.0));
        pb.inc(bytes as u64);
    }

    let download_elapsed = start_download.elapsed().as_secs_f32();

    active_file.complete();

    pb.finish_with_message(format!("Downloading {}...Done ({:.01} mb/s)", &filename, (size as f32/download_elapsed)/1000000.0));
    
    Ok(())
}


// Local helper to create pb with an empty message
fn download_pb(len: u64) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new(len);
    pb.set_style(
        indicatif::ProgressStyle::with_template("{spinner:.green} {msg} [{bar:40.green}]").unwrap()
        .progress_chars("->O")
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(70));

    pb
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