use clap::{Parser,
    Subcommand,
    ArgGroup,
    Args,
};

use crate::{
    hostrada_variable::HostradaVar,
    dates_and_times::YearMonth,
};

/// convert: Take a file or directory and convert all hostrada netcdf files found to a csv file
/// pixel: calculate x and y of the pixel containing the requested coordinates, using a reference file
#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Convert(ConvertArgs),
    Pixel(PixelArgs),
    Download(DownloadArgs),
    Origin(OriginArgs),
}

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["file", "dir"]) // bezieht sich auf die Namen der fields
))]
pub struct ConvertArgs {
    /// convert full hostrada grid (CAUTION: May take a long time, will produce really large data (up to >10 GB) which is very inefficient for csv. It is highly suggested to select pixels of interest and only convert data for those pixels)
    #[arg(short, long)]
    pub all: bool,

    /// convert file mode
    #[arg(short = 'f', long)]
    pub file: Option<std::path::PathBuf>,

    /// convert all files in directory mode
    #[arg(short = 'd', long)]
    pub dir: Option<std::path::PathBuf>,

    /// result directory; converted results will be collected here
    pub output_dir: std::path::PathBuf,

    /// x value of pixel to be converted (use command 'pixel' to get pixel x and y for given coordinates)
    #[arg(required_unless_present = "all")]
    pub x: Option<usize>,

    /// y value of pixel to be converted (use command 'pixel' to get pixel x and y for given coordinates)
    #[arg(required_unless_present = "all")]
    pub y: Option<usize>,
}

#[derive(Args, Debug)]
pub struct PixelArgs {
    /// hostrada netcdf file to reference while searching for pixel
    pub ref_file: std::path::PathBuf,

    /// latitude of requested coordinate
    pub lat: f64,

    /// longitude of requested coordinate
    pub lon: f64,
}

#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// variable to download
    pub variable: HostradaVar,

    /// inclusive: first month to download (format YYYY-MM)
    pub start_month: YearMonth, 

    /// exclusive: last month to download (format YYYY-MM)
    pub end_month: YearMonth,

    /// directory to install to
    pub install_dir: std::path::PathBuf,
}

#[derive(Args, Debug)]
pub struct OriginArgs {
    /// File to read in and get origin from (= timestamp used as a base to depict time (days-since) in hostrada netcdf files)
    pub file_path: std::path::PathBuf,
}
