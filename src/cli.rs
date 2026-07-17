use clap::{Parser,
    Subcommand,
    ArgGroup,
    Args,
    ValueEnum,
};

use crate::{
    config::Config, dates_and_times::YearMonth, error::ConfigError,
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

#[derive(Debug, Clone, ValueEnum)]
pub enum HostradaVar {
    AirTemperatureMean, // tas
    CloudCover, // clt
    DewPoint, // tdew
    HumidityMixingRation, // mixr
    HumidityRelative, // hurs
    PressureSealevel, // psl
    PressureSurface, // ps
    RadiationDownwelling, // rsds
    UrbanHeatIslandIntensity, // uhi
    WindDirection, // sfcWind_direction
    WindSpeed, // sfcWind
}

impl HostradaVar {
    /// Returns the variable name as it appears in the filenames on the dwd server (also, most of this abbreviations do match the variable names in the netcdf files; i did not check that as of now)
    pub fn abbr(&self) -> &str {
        match self {
            HostradaVar::AirTemperatureMean => "tas",
            HostradaVar::CloudCover => "clt",
            HostradaVar::DewPoint => "tdew",
            HostradaVar::HumidityMixingRation => "mixr",
            HostradaVar::HumidityRelative => "hurs",
            HostradaVar::PressureSealevel => "psl",
            HostradaVar::PressureSurface => "ps",
            HostradaVar::RadiationDownwelling => "rsds",
            HostradaVar::UrbanHeatIslandIntensity => "uhi",
            HostradaVar::WindDirection => "sfcWind_direction",
            HostradaVar::WindSpeed => "sfcWind",
        }
    }

    /// Returns the link of the variable. This link looks like this: https://opendata.dwd.de/climate_environment/CDC/grids_germany/hourly/hostrada/VAR_NAME/ where 
    /// VAR_NAME is something like 'air_temperature_mean'
    /// # Errors
    /// - As the base link is defined by the config file, this method propagates any ConfigErrors.
    pub fn link(&self) -> Result<String, ConfigError> {
        let config = Config::load()?;
        let mut base_link = config.base_link;
        
        let var_name_link = match self {
            HostradaVar::AirTemperatureMean => "air_temperature_mean",
            HostradaVar::CloudCover => "cloud_cover",
            HostradaVar::DewPoint => "dew_point",
            HostradaVar::HumidityMixingRation => "humidity_mixing_ratio",
            HostradaVar::HumidityRelative => "humidity_relative",
            HostradaVar::PressureSealevel => "pressure_sealevel",
            HostradaVar::PressureSurface => "pressure_surface",
            HostradaVar::RadiationDownwelling => "radiation_downwelling",
            HostradaVar::UrbanHeatIslandIntensity => "urban_heat_island_intensity",
            HostradaVar::WindDirection => "wind_direction",
            HostradaVar::WindSpeed => "wind_speed",
        };

        base_link.push_str(var_name_link);
        base_link.push('/');

        Ok(base_link)
    }
}

#[derive(Args, Debug)]
pub struct OriginArgs {
    /// File to read in and get origin from (= timestamp used as a base to depict time (days-since) in hostrada netcdf files)
    pub file_path: std::path::PathBuf,
}