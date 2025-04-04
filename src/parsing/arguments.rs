use clap::Parser;
use clap::error::ErrorKind as ClapErrorKind;

use std::error::Error;
use std::path::PathBuf;

use crate::parsing::preset;

// (Linux only)
fn default_config_path() -> PathBuf {
    PathBuf::from("/etc/rmx/rmx.conf")
}

#[derive(Parser, Debug)]
#[command(
    name = "rmx",
    version,
    about,
    long_about = "Rust CLI to delete files based on their extension"
)]
pub struct Args {
    /// List of file extensions, like `o a so`, without dots
    #[arg(num_args(1..), required_unless_present="preset", required_unless_present="presets", conflicts_with="preset", conflicts_with="presets")]
    extensions: Vec<String>,

    /// Directory in which to delete files
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Include hidden files, and files in hidden folders
    #[arg(short, long, default_value_t = false)]
    all: bool,

    /// Remove confirmation prompt
    #[arg(short, long, default_value_t = false)]
    force: bool,

    /// Print matching files (slower), does not block deletion, overriden by -n/--dry-run
    #[arg(short, long, default_value_t = false)]
    list: bool,

    /// Do not perform deletion, enables --list
    #[arg(short = 'n', long, default_value_t = false)]
    dry_run: bool,

    /// Whether the files should be deleted recursively or not
    #[arg(short, long, default_value_t = false)]
    recurse: bool,

    /// Invert selection: keep given extensions, and delete other files
    #[arg(short, long, default_value_t = false)]
    invert: bool,

    /// (Linux only) Load a preset of extension, saved in the config file (see --config), cannot be used with other extensions
    #[arg(long)]
    preset: Option<String>,

    /// (Linux only) Show available presets, cannot be used with other extensions
    #[arg(long)]
    presets: bool,

    /// (Linux only) File location for presets (see --preset/--presets)
    #[arg(long, default_value_os_t = default_config_path())]
    config: PathBuf,
}

pub struct CollectOptions {
    pub all: bool,
    pub list: bool,
    pub recurse: bool,
    pub invert: bool,
}

pub struct DeleteOptions {
    pub force: bool,
    pub dry_run: bool,
}

impl Args {
    // After parse is called, .path and .extensions can be safely called
    pub fn parse() -> Result<Option<Args>, Box<dyn Error>> {
        let args = match Args::try_parse() {
            Ok(args) => Some(args),
            Err(err) => {
                if err.kind() == ClapErrorKind::DisplayHelp
                    || err.kind() == ClapErrorKind::DisplayVersion
                {
                    err.print()?;
                    return Ok(None);
                } else {
                    return Err(err.into());
                }
            }
        };

        let Some(mut args) = args else {
            return Ok(None);
        };

        args.path = match &args.path {
            Some(p) => Some(p.clone()),
            None => Some(std::env::current_dir()?),
        };

        if args.presets {
            preset::show(&args.config)?;
            return Ok(None);
        }

        if args.dry_run {
            args.list = true;
        }

        Ok(Some(args))
    }

    pub fn get_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(match &self.path {
            Some(p) => p.clone(),
            None => std::env::current_dir()?,
        })
    }

    fn raw_get_extensions(&self) -> Result<Vec<String>, Box<dyn Error>> {
        if let Some(p) = &self.preset {
            return preset::parse(p, &self.config);
        }

        Ok(self.extensions.clone())
    }

    pub fn get_extensions(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let extensions = self.raw_get_extensions()?;
        if !are_extensions_valid(&extensions) {
            return Err("Invalid extensions.".into());
        }
        Ok(extensions)
    }

    pub fn get_options(&self) -> (CollectOptions, DeleteOptions) {
        (
            CollectOptions {
                all: self.all,
                list: self.list,
                recurse: self.recurse,
                invert: self.invert,
            },
            DeleteOptions {
                force: self.force,
                dry_run: self.dry_run,
            },
        )
    }
}

fn are_extensions_valid(extensions: &Vec<String>) -> bool {
    extensions
        .iter()
        .all(|ext| ext.bytes().all(|b| (b'a'..=b'z').contains(&b)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_one_valid_extension() {
        let mut extensions = Vec::new();
        extensions.push("a".to_string());

        assert!(are_extensions_valid(&extensions));
    }

    #[test]
    fn check_two_valid_extension() {
        let mut extensions: Vec<String> = Vec::new();
        extensions.push("a".to_string());
        extensions.push("b".to_string());

        assert!(are_extensions_valid(&extensions));
    }

    #[test]
    fn check_star_extension() {
        let mut extensions: Vec<String> = Vec::new();
        extensions.push("a".to_string());
        extensions.push("*".to_string());

        assert!(!are_extensions_valid(&extensions));
    }

    #[test]
    fn check_invalid_extension() {
        let mut extensions: Vec<String> = Vec::new();
        extensions.push(".".to_string());
        extensions.push("b".to_string());

        assert!(!are_extensions_valid(&extensions));
    }
}
