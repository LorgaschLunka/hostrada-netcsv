*Transparency notice: Written by AI and checked for correctness.*

# hostrada-netcsv

A command-line utility for downloading and converting **HOSTRADA NetCDF** datasets from the German Weather Service (DWD) into CSV files (see https://opendata.dwd.de/climate_environment/CDC/grids_germany/hourly/hostrada/). Currently only for linux.

## Features

* Convert single NetCDF files or entire directories to CSV
* Extract data for a single HOSTRADA grid pixel
* Convert the complete HOSTRADA grid for a file or directory
* Conveniently download official HOSTRADA datasets directly from the DWD Open Data archive
* Resolve latitude/longitude coordinates to the corresponding HOSTRADA grid pixel

---

## The HOSTRADA Grid

HOSTRADA data is organized on a regular **1 km × 1 km grid** covering Germany. The grid consists of approximately **720 × 938** cells. Each cell contains data calculated via statistical models.

Throughout this project, these grid cells are referred to as **pixels**. Each pixel is identified by its integer **X** and **Y** coordinates within the grid. The `pixel` command can be used to determine the corresponding pixel for a given latitude and longitude (decimal degrees).

---

## Installation

### Prerequisites

This project depends on the native **NetCDF C library**. See build/dependencies of https://github.com/georust/netcdf.

Ubuntu/Debian:

```bash
sudo apt install libnetcdf-dev
```

Arch Linux:

```bash
sudo pacman -S netcdf
```

Note, that netCDF 4.9.3 was used developing and testing the tool. It is known that older versions of the native library (e.g. 3.x on stable Ubuntu repositories) potentially produce the desired result, but do so much slower and with some error messages. I did not test for any versions, so it probably is best to just use 4.9.3 or similar versions.

The build also requires `pkg-config`.

---

## Usage

```
hostrada-netcsv <COMMAND>
```

### Convert NetCDF to CSV

Convert a single file:

```bash
hostrada-netcsv convert --file input.nc output_dir X Y
```

Convert all NetCDF files in a directory:

```bash
hostrada-netcsv convert --dir input_directory output_dir X Y
```

Convert the complete HOSTRADA grid:

```bash
hostrada-netcsv convert --dir input_directory output_dir --all
```

Useful options:

| Option       | Description                                                                                                                                               |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--all`      | Convert every grid cell. Produces very large output (potentially >10 GB).                                                                                 |
| `--skip-nan` | While converting every grid cell, skip invalid HOSTRADA values (`-9999`) to significantly reduce output size.                                             |
| `--merge`    | Merge multiple converted files into a single CSV. Only supported for single-pixel conversions. Input directory should contain files of one variable only. |

---

### Find the pixel for coordinates

```bash
hostrada-netcsv pixel reference.nc LATITUDE LONGITUDE
```

Returns the HOSTRADA grid coordinates (`X`, `Y`) corresponding to the provided geographic coordinates.

Coordinates farther than approximately **735 m** from the nearest HOSTRADA grid cell are considered outside the dataset.

Keep in mind, that coordinates must be provided in decimal degrees.

Example:

```bash
$ hostrada-netcsv pixel ./data/input/clt_1hr_HOSTRADA-v1-0_BE_gn_2020010100-2020013123.nc 48.0829 11.7519

Found a pixel!

The nearest pixel to coordinates 48.0829, 11.7519 is pixel (456, 138) with center coordinates 48.081, 11.756.
Estimated distance to pixel center: 373.50 m.
```

---

### Download HOSTRADA data

```bash
hostrada-netcsv download <VARIABLE> <START_MONTH> <END_MONTH> <INSTALL_DIR>
```
`START_MONTH` is inclusive, while `END_MONTH` is exclusive.

Example:

```bash
hostrada-netcsv download air-temperature-mean 2023-01 2023-06 ./data
```
This would download 5 months (January, February, March, April, May).

Downloads data directly from the official DWD Open Data archive: https://opendata.dwd.de/climate_environment/CDC/grids_germany/hourly/hostrada/.

Supported variables include:

* air-temperature-mean
* cloud-cover
* dew-point
* humidity-mixing-ration
* humidity-relative
* pressure-sealevel
* pressure-surface
* radiation-downwelling
* urban-heat-island-intensity
* wind-direction
* wind-speed

---

## Notes

* **Do not rename downloaded HOSTRADA files.** The converter relies on the original filenames.
* Invalid values in the HOSTRADA dataset are represented by `-9999`.
* Converting the full grid creates one output file per hourly timestep and may require substantial disk space.

---

## Performance

Conversion takes some time. Expect 2-10s per file when converting a single pixel.

---

## License

See the repository license for details.

