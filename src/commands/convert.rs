use crate::{cli::{ConvertArgs}, converter::*};

use log::{warn, debug};

pub fn run(args: ConvertArgs) {
    match (args.file, args.dir) {
        (Some(file_path), None) => if args.all {
            // Get all values of one file 
            convert_all_values(vec![file_path.clone()], args.output_dir).unwrap_or_else(|err| {
                eprintln!("Failed to convert all values for {:?}: {err}", file_path);
                std::process::exit(1);
            });

        } else {
            // Get values of a defined pixel of one file
            convert_pixel(vec![file_path.clone()], args.x.unwrap(), args.y.unwrap(), args.output_dir).unwrap_or_else(|err| {
                eprintln!("Failed to convert values of a pixel for {:?}: {err}", file_path);
                std::process::exit(1);
            });
        },
        (None, Some(dir_path)) => if args.all {
            // Get all values of all files in a directory
            let paths = std::fs::read_dir(dir_path.clone()).unwrap_or_else(|err| {
                eprintln!("Failed to read directory {:?}: {err}", dir_path);
                std::process::exit(1);
            });
            let files = collect_paths(paths);

            convert_all_values(files, args.output_dir).unwrap_or_else(|err| {
                eprintln!("Failed to convert all values for directory {:?}: {err}", dir_path);
                std::process::exit(1);
            });

        } else {
            // Get all values of a defined pixel of all files in a directory
            validate_files(&dir_path).unwrap();
            let paths = std::fs::read_dir(dir_path.clone()).unwrap_or_else(|err| {
                eprintln!("Failed to read directory {:?}: {err}", dir_path);
                std::process::exit(1);
            });
            let files = collect_paths(paths);

            convert_pixel(files, args.x.unwrap(), args.y.unwrap(), args.output_dir).unwrap_or_else(|err| {
                eprintln!("Failed to convert all values of a pixel for directory {:?}: {err}", dir_path);
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