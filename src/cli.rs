use anyhow::{
    ensure,
};
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

impl Cli {
    /// Validates the user input. If this function returns an error, the user input is faultsy or something went wrong validating the input.
    pub fn validate(&self) -> anyhow::Result<()> {
        match &self.command {
            Commands::Convert(args) => Cli::validate_convert(args),
            Commands::Pixel(args) => Cli::validate_pixel(args),
            Commands::Download(args) => Cli::validate_download(args),
            Commands::Origin(args) => Cli::validate_origin(args),
        }
    }

    /// Validates convert command input
    /// ## Errors
    /// - if the directorys or files are not valid directorys or netcdf files
    fn validate_convert(args: &ConvertArgs) -> anyhow::Result<()> {
        ensure!(args.output_dir.is_dir(), "Invalid output directory: {}", args.output_dir.display());

        if let Some(dir) = &args.dir {
            ensure!(dir.is_dir(), "Invalid input directory: {}", dir.display());
        };

        if let Some(file) = &args.file {
            ensure!(file.is_file(), "Invalid input file: {}", file.display());

            ensure!(
                file.extension().and_then(|v| v.to_str()) == Some("nc"),
                "File {} does not seem to be a netcdf file",
                file.display()
            );
        };
        Ok(())
    }

    /// Validates pixel command input.
    /// ## Errors
    /// - if the ref_file is not a valid netcdf file
    /// - if lat/lon are higher/lower -90/90. Technically, this could be specified further by focussing to German area coordinates. But not for now.
    fn validate_pixel(args: &PixelArgs) -> anyhow::Result<()> {
        ensure!(args.ref_file.is_file(), "Invalid reference file: {}", args.ref_file.display());
        
        ensure!(
            args.ref_file.extension().and_then(|v| v.to_str()) == Some("nc"),
            "File {} does not seem to be a netcdf file",
            args.ref_file.display()
        );

        ensure!(
            !(args.lat.clamp(-90.0, 90.0).abs() == 90.0) && !(args.lon.clamp(-90.0, 90.0).abs() == 90.0),
            "Out of bounds latitude or longitude: {}, {}",
            args.lat, args.lon
        );

        Ok(())
    }

    /// Validates download command input.
    /// ## Errors
    /// - if the install directory is not a valid directory
    fn validate_download(args: &DownloadArgs) -> anyhow::Result<()> {
        ensure!(args.install_dir.is_dir(), "Invalid install directory: {}", args.install_dir.display());
        Ok(())
    }

    /// Validates origin command input.
    /// ## Errors
    /// - if the file to get the origin from is not a valid netcdf file
    fn validate_origin(args: &OriginArgs) -> anyhow::Result<()> {
        ensure!(args.file_path.is_file(), "Invalid file to get origin from: {}", args.file_path.display());

        ensure!(
            args.file_path.extension().and_then(|v| v.to_str()) == Some("nc"),
            "File {} does not seem to be a netcdf file",
            args.file_path.display()
        );

        Ok(())
    }
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
