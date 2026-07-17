mod commands;
mod cli;
mod config;
mod converter;
mod pixel_calculator;
mod downloader;
mod hostrada_dataset;
mod hostrada_pixel;
mod misc;
mod error;
mod dates_and_times;

use env_logger::Env;
use clap::Parser;

use crate::{
    cli::{Cli, Commands},
    commands::{convert, download, pixel, origin},
};


fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Convert(args) => convert::run(args),
        Commands::Pixel(args) => pixel::run(args),
        Commands::Download(args) => download::run(args),
        Commands::Origin(args) => origin::run(args),
    }

}



