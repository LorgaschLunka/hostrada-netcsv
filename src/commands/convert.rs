use crate::{cli::{ConvertArgs}, converter::*};

use log::{warn, debug};
use owo_colors::OwoColorize;

pub fn run(args: ConvertArgs) {
    match (args.file, args.dir) {
        (Some(file_path), None) => if args.all {
            // Get all values of one file 
            convert_all_values(vec![file_path.clone()], &args.output_dir, args.skip_nan).unwrap_or_else(|err| {
                eprintln!("{} {:?}{} {err}", "Failed to convert all values for".red().bold(), file_path, ":\n╰─▶".red().bold());
                std::process::exit(1);
            });

        } else {
            // Get values of a defined pixel of one file
            convert_pixel(vec![file_path.clone()], args.x.unwrap(), args.y.unwrap(), &args.output_dir, args.merge).unwrap_or_else(|err| {
                eprintln!("{} {:?}{} {err}", "Failed to convert values for a pixel for".red().bold(), file_path, ":\n╰─▶".red().bold());
                std::process::exit(1);
            });
        },
        (None, Some(dir_path)) => if args.all {
            // Get all values of all files in a directory
            match validate_files(&dir_path) {
                Ok(opt) => {
                    if let Some(path) = opt {
                        warn!("File {} does not match hostrada server file size", path.display());
                        std::process::exit(1);

                    }
                },
                Err(e) => {
                    warn!("Error validating local files matching hostrada server files: {}", e);
                    std::process::exit(1);
                },
            }
            let paths = std::fs::read_dir(dir_path.clone()).unwrap_or_else(|err| {
                eprintln!("Failed to read directory {:?}: {err}", dir_path);
                std::process::exit(1);
            });
            let files = collect_paths(paths);

            convert_all_values(files, &args.output_dir, args.skip_nan).unwrap_or_else(|err| {
                eprintln!("{} {:?}{} {err}", "Failed to convert all values for".red().bold(), dir_path, ":\n╰─▶".red().bold());
                std::process::exit(1);
            });

        } else {
            // Get all values of a defined pixel of all files in a directory
            match validate_files(&dir_path) {
                Ok(opt) => {
                    if let Some(path) = opt {
                        warn!("File {} does not match hostrada server file size", path.display());
                        std::process::exit(1);

                    }
                },
                Err(e) => {
                    warn!("Error validating local files matching hostrada server files: {}", e);
                    std::process::exit(1);
                },
            }
            let paths = std::fs::read_dir(dir_path.clone()).unwrap_or_else(|err| {
                eprintln!("Failed to read directory {:?}: {err}", dir_path);
                std::process::exit(1);
            });
            let files = collect_paths(paths);

            convert_pixel(files, args.x.unwrap(), args.y.unwrap(), &args.output_dir, args.merge).unwrap_or_else(|err| {
                eprintln!("{} {:?}{} {err}", "Failed to convert values for a pixel for".red().bold(), dir_path, ":\n╰─▶".red().bold());
                std::process::exit(1);
            });

        }

        _ => unreachable!(),
    }
    
}

fn collect_paths<I>(iter: I) -> Vec<std::path::PathBuf>
where
    I: Iterator<Item = std::io::Result<std::fs::DirEntry>>,
{
    iter.filter_map(|res| match res {
        Ok(entry) => {
            debug!("Adding {:?}", entry.file_name());
            Some(entry.path())
        }
        Err(e) => {
            warn!("Skipping file: {e}");
            None
        }
    })
    .collect()
}