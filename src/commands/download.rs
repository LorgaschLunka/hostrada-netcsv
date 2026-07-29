use owo_colors::OwoColorize;
use std::time::Instant;

use crate::{
    cli::DownloadArgs,
    downloader::*,
    dates_and_times::readable_dur,
};

pub fn run(mut args: DownloadArgs) {
    let client = reqwest::blocking::Client::new();
    let months = args.start_month.range_to(&args.end_month);

    let start = Instant::now();
    for month in &months {
        let mut inner = args.install_dir.clone();
        if let Err(e) = download_file(&args.variable, *month, inner, &client) {
            eprintln!("{} {e:?}", "Failed to download file:\n╰─▶".red().bold());
            std::process::exit(1);
        }
    }
    
    println!("{} downloading {} {} file(s) ({} to {}) in {}", "Finished".green().bold(), months.len(), args.variable.abbr(), args.start_month, args.end_month, readable_dur(start.elapsed()))
}