use clap::ValueEnum;

use crate::config::Config;

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

    /// Returns the HostradaVar enum from the abbreviation (e.g. "tas" => HostradaVar::AirTemperatureMean)
    pub fn from_abbr(abbr: &str) -> Option<Self> {
        match abbr{
            "tas" => Some(HostradaVar::AirTemperatureMean),
            "clt" => Some(HostradaVar::CloudCover),
            "tdew" => Some(HostradaVar::DewPoint),
            "mixr" => Some(HostradaVar::HumidityMixingRation),
            "hurs" => Some(HostradaVar::HumidityRelative),
            "psl" => Some(HostradaVar::PressureSealevel),
            "ps" => Some(HostradaVar::PressureSurface),
            "rsds" => Some(HostradaVar::RadiationDownwelling),
            "uhi" => Some(HostradaVar::UrbanHeatIslandIntensity),
            "sfcWind_direction" => Some(HostradaVar::WindDirection),
            "sfcWind" => Some(HostradaVar::WindSpeed),
            _ => None,
        }   
    }
    /// Returns the link of the variable. This link looks like this: https://opendata.dwd.de/climate_environment/CDC/grids_germany/hourly/hostrada/VAR_NAME/ where 
    /// VAR_NAME is something like 'air_temperature_mean'
    /// # Errors
    /// - As the base link is defined by the config file, this method propagates any errors from the config.
    pub fn link(&self) -> anyhow::Result<String> {
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

impl std::fmt::Display for HostradaVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current_variant = match self {
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
        write!(f, "{current_variant}")
    }
}
