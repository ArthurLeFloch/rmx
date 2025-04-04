use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn show(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let Ok(data) = fs::read_to_string(path) else {
        return Err(format!("Could not read {}", path.display()).into());
    };
    println!("Parsing presets in {}...", path.display());

    let lines = data.lines().filter_map(|s| s.strip_prefix("preset "));

    let mut found = false;

    for line in lines {
        let splitted: Vec<&str> = line.trim().split("=").collect();
        if splitted.len() != 2 {
            eprintln!("Preset line \"{}\" not formatted correctly", line);
            continue;
        }

        let preset = splitted.first().unwrap();
        let extensions: Vec<&str> = splitted.last().unwrap().trim().split(" ").collect();
        if extensions.is_empty() {
            eprintln!("Preset \"{}\" does not contain any extensions", preset);
            continue;
        }

        print!("rmx --preset {preset}: \tRemoves ");
        let n = extensions.len();
        for i in 0..n - 1 {
            print!("*.{}, ", extensions[i]);
        }
        println!("*.{}", extensions[n - 1]);

        found = true;
    }

    if !found {
        println!("Could not find any preset in {}", path.display());
    }

    Ok(())
}

// Only supported in linux filesystems
pub fn parse(preset: &String, path: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    let Ok(data) = fs::read_to_string(path) else {
        return Err(format!("Could not read {}", path.display()).into());
    };

    let prefix = format!("preset {preset}");
    let Some(line) = data.lines().find(|s| s.starts_with(&prefix)) else {
        return Err(format!("Could not find preset \"{}\" in {}", preset, path.display()).into());
    };

    let splitted: Vec<&str> = line.split("=").collect();

    if splitted.len() != 2 {
        return Err(format!(
            "Preset \"{}\" not formatted correctly in {}",
            preset,
            path.display()
        )
        .into());
    }

    let extensions: Vec<String> = splitted
        .last()
        .unwrap()
        .trim()
        .split(" ")
        .map(|s| s.to_string())
        .collect();

    if extensions.is_empty() {
        return Err(format!(
            "Preset \"{}\" with no extensions in {}",
            preset,
            path.display()
        )
        .into());
    }

    Ok(extensions)
}
