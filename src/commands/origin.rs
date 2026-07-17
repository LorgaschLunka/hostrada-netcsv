use owo_colors::OwoColorize;

use crate::{cli::OriginArgs,
    dates_and_times::fast_origin,
};



pub fn run(args: OriginArgs) {
    match fast_origin(args.file_path) {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!("{} {e}", "Failed to get origin:\n╰─▶".red().bold());
            std::process::exit(1);
        },
    };  
}