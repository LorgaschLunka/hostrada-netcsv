mod commands;
mod cli;
mod config;
mod converter;
mod pixel_calculator;
mod downloader;
mod hostrada_dataset;
mod hostrada_pixel;
mod hostrada_variable;
mod misc;
mod error;
mod dates_and_times;

use env_logger::Env;
use clap::Parser;
use owo_colors::OwoColorize;

use crate::{
    cli::{Cli, Commands},
    commands::{convert, download, origin, pixel},
    config::Config,
};


fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    if let Err(e) = Config::create_dir() {
        eprintln!("{} {e}", "Error with config file:\n╰─▶".red().bold());
        std::process::exit(1);
    }

    let cli = Cli::parse();
    match cli.validate() {
        Err(e) => {
            eprintln!("CLI Error: {e}");
            std::process::exit(1);
        },
        Ok(()) => ()
    }

    match cli.command {
        Commands::Convert(args) => convert::run(args),
        Commands::Pixel(args) => pixel::run(args),
        Commands::Download(args) => download::run(args),
        Commands::Origin(args) => origin::run(args),
    }

}



