use std::error::Error;
use std::path::PathBuf;
use std::process;

use rmx::arguments::Args;
use rmx::{self, CollectOptions, DeleteOptions};

fn run(
    extensions: &Vec<String>,
    path: &PathBuf,
    options: &(CollectOptions, DeleteOptions),
) -> Result<(), Box<dyn Error>> {
    let to_delete = rmx::collect_matching_files(extensions, path, &options.0)?;
    rmx::delete_files(&to_delete, &options.1)?;

    Ok(())
}

fn main() {
    let args = Args::parse().unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });

    let Some(args) = args else {
        // Happens with --help / --version
        process::exit(0);
    };

    let extensions = args.get_extensions().unwrap_or_else(|e| {
        eprintln!("Error while collecting extensions: {e}");
        process::exit(1);
    });

    let path = args.get_path().unwrap_or_else(|e| {
        eprintln!("Error while getting path: {e}");
        process::exit(1);
    });

    let options = args.get_options();

    if let Err(e) = run(&extensions, &path, &options) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
