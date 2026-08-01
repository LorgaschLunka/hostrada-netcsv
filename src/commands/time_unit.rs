use owo_colors::OwoColorize;

use crate::{cli::TimeUnitArgs,
    dates_and_times::fast_time_unit,
};



pub fn run(args: TimeUnitArgs) {
    match fast_time_unit(&args.file_path) {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!("{} {e:?}", "Failed to get time unit:\n╰─▶".red().bold());
            std::process::exit(1);
        },
    };  
}