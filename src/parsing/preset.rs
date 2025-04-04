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
        let extensions: Vec<&str> = splitted.last().unwrap().trim().split_whitespace().collect();
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
        println!("Could not find any valid preset in {}", path.display());
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
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if extensions.is_empty() {
        return Err(format!(
            "Preset \"{}\" does not contain any extensions in {}",
            preset,
            path.display()
        )
        .into());
    }

    Ok(extensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{io::Write, path::Path};
    use tempfile::NamedTempFile;

    fn create_config_file(data: &str) -> Result<NamedTempFile, Box<dyn Error>> {
        let file = NamedTempFile::new()?;
        write!(&file, "{}", data)?;
        Ok(file)
    }

    #[test]
    fn parse_unknown_config_should_err() -> Result<(), Box<dyn Error>> {
        let presets = "preset java=jar class\npreset c=o a so out";
        create_config_file(presets)?;

        let preset = "out".to_string();
        let config_path = Path::new("-unknown file-").to_path_buf();

        let res = parse(&preset, &config_path);

        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn parse_unknown_preset_should_err() -> Result<(), Box<dyn Error>> {
        let presets = "preset java=jar class\npreset c=o a so out";
        let file = create_config_file(presets)?;

        let preset = "out".to_string();
        let config_path = file.path().to_path_buf();

        let res = parse(&preset, &config_path);

        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn parse_preset_no_extension_should_err() -> Result<(), Box<dyn Error>> {
        let presets = "preset java=jar class\npreset c=";
        let file = create_config_file(presets)?;

        let preset = "c".to_string();
        let config_path = file.path().to_path_buf();

        let res = parse(&preset, &config_path);

        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn parse_preset_with_one_extension() -> Result<(), Box<dyn Error>> {
        let presets = "preset java=class\npreset c=o a so out";
        let file = create_config_file(presets)?;

        let preset = "java".to_string();
        let config_path = file.path().to_path_buf();

        let res = parse(&preset, &config_path)?;

        assert_eq!(1, res.len());

        Ok(())
    }

    #[test]
    fn parse_preset_with_multiple_extension() -> Result<(), Box<dyn Error>> {
        let presets = "preset java=jar class\npreset c=o a so out";
        let file = create_config_file(presets)?;

        let preset = "c".to_string();
        let config_path = file.path().to_path_buf();

        let res = parse(&preset, &config_path)?;

        assert_eq!(4, res.len());

        Ok(())
    }
}
