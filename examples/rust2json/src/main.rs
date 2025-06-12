use std::{
    env, fs,
    io::{self, BufWriter, Write as _},
    path::Path,
    time::SystemTime,
};

use syn_serde::json;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_dir = env::args_os()
        .nth(1)
        .map(|s| s.into_string().unwrap_or_else(|_| {
            eprintln!("Invalid input directory path");
            std::process::exit(1);
        }))
        .unwrap_or_else(|| {
            println!("Usage: rust2json <input_directory>");
            std::process::exit(1);
        });

    for entry in WalkDir::new(&input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let input_path = entry.path();
        let output_path = input_path.with_extension("rs.json");

        // Skip if output exists and is newer than input
        if let Ok(output_meta) = fs::metadata(&output_path) {
            if let Ok(input_meta) = fs::metadata(input_path) {
                if let (Ok(input_time), Ok(output_time)) = (
                    input_meta.modified(),
                    output_meta.modified(),
                ) {
                    if output_time >= input_time {
                        continue;
                    }
                }
            }
        }

        println!("Processing: {:?}", input_path);

        let code = match fs::read_to_string(input_path) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("Error reading {:?}: {}", input_path, e);
                continue;
            }
        };

        let syntax = match syn::parse_file(&code) {
            Ok(syntax) => syntax,
            Err(e) => {
                eprintln!("Error parsing {:?}: {}", input_path, e);
                continue;
            }
        };

        let buf = json::to_string_pretty(&syntax);
        if let Err(e) = fs::write(&output_path, buf) {
            eprintln!("Error writing {:?}: {}", output_path, e);
            continue;
        }
    }

    Ok(())
}