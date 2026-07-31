use owo_colors::OwoColorize;
use crate::{
    cli::PixelArgs,
    pixel_calculator::*,
    misc::green_spinner,
};

pub fn run(args: PixelArgs) {
    let spinner = green_spinner();
    match pixel(args.ref_file, args.lat, args.lon) {
        Ok(pix) => {
            spinner.finish();
            println!("{}The nearest pixel to coordinates {}, {} is pixel ({}, {}) with center coordinates {:.03}, {:.03}. Estimated distance to pixel center: {:.2}m.",
                "Found a pixel! :D\n".green().bold(),
                args.lat,
                args.lon,
                pix.value.x.green().bold(),
                pix.value.y.green().bold(),
                pix.value.lat,
                pix.value.lon,
                pix.distance
            )
        },
        Err(e) => {
            spinner.finish();
            eprintln!("{} {e}", "Error calculating nearest pixel:\n╰─▶".red().bold())
        },
    }
}