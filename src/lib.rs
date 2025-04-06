use std::error::Error;

use std::fs::{self, DirEntry, FileType};
use std::io::{self, Write};
use std::path::PathBuf;

pub use crate::parsing::arguments;
pub use crate::parsing::arguments::{CollectOptions, DeleteOptions};
mod parsing;

// Returns the largest extension (i.e. using file.tar.gz will return tar.gz)
fn get_fileext(filename: &String) -> Option<&str> {
    let parts = filename.split_once(".")?;
    if parts.0.is_empty() {
        // In case file is hidden, like ".file.lock", split again to get ".lock"
        return Some(parts.1.split_once(".")?.1);
    }
    Some(parts.1)
}

fn get_filetype(entry: &DirEntry) -> Result<FileType, Box<dyn Error>> {
    entry
        .file_type()
        .map_err(|e| format!("Couldn't extract filetype from {:?}: {}", entry.path(), e).into())
}

fn get_filename(entry: &DirEntry) -> Result<String, Box<dyn Error>> {
    match entry.file_name().to_str() {
        Some(s) => Ok(String::from(s)),
        None => Err(format!("Couldn't extract filename from {:?}", entry).into()),
    }
}

fn collect_matching_files_rec(
    options: &CollectOptions,
    keep: &dyn Fn(&str) -> bool,
    path: &PathBuf,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    // Storing PathBuf instead of DirEntry avoids keeping the directories' file descriptors opened
    // thus avoiding a "Too many open files" error
    let mut acc: Vec<PathBuf> = Vec::new();
    let mut directories: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry.map_err(|e| format!("Error while retrieving file data: {:?}", e))?;

        let filepath = entry.path();
        let filename = get_filename(&entry)?;
        let filetype = get_filetype(&entry)?;

        if !options.all && filename.starts_with('.') {
            continue;
        }

        if options.recurse && filetype.is_dir() {
            directories.push(filepath);
            continue;
        }

        if !filetype.is_file() || !get_fileext(&filename).is_some_and(|f| keep(&f)) {
            continue;
        };

        if options.list {
            println!("{}", filepath.to_string_lossy());
        }

        acc.push(filepath);
    }

    for p in directories.iter() {
        acc.extend(collect_matching_files_rec(options, keep, p)?);
    }

    Ok(acc)
}

// Assume extensions are valid
pub fn collect_matching_files(
    extensions: &Vec<String>,
    path: &PathBuf,
    options: &CollectOptions,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let dotted: Vec<String> = extensions.iter().map(|s| format!(".{s}")).collect();
    let keep = |file_ext: &str| {
        let dotted_fil_ext = format!(".{file_ext}");
        options.invert != dotted.iter().any(|e| dotted_fil_ext.ends_with(e))
    };

    collect_matching_files_rec(options, &keep, path)
}

fn prompt_for_confirmation(files: &[PathBuf]) -> Result<bool, Box<dyn Error>> {
    print!(
        "Do you really want to delete {} file(s)? [Y/n] ",
        files.len()
    );
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;

    Ok(matches!(buf.trim(), "" | "y" | "Y"))
}

pub fn delete_files(
    files: &[PathBuf],
    delete_options: &DeleteOptions,
) -> Result<(), Box<dyn Error>> {
    if files.is_empty() {
        println!("No matching file.");
        return Ok(());
    }

    if delete_options.dry_run {
        return Ok(());
    }

    if !delete_options.force && !prompt_for_confirmation(&files)? {
        println!("Cancelled file deletion.");
        return Ok(());
    }

    println!("Deleting files...");
    for file in files {
        fs::remove_file(file)?;
    }
    println!("Done!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::os::unix::fs::symlink;
    use tempfile::{TempDir, tempdir};

    // Creates a directory as follow:
    //
    // <temp_dir>/
    // ├── .hidden_folder/
    // │   └── hidden.txt
    // ├── subfolder1/
    // │   ├── subfolder2/
    // │   │   ├── backup.bak
    // │   │   ├── data.dat
    // │   │   └── sub2.txt
    // │   ├── sub1.log
    // │   └── sub1.txt
    // ├── .hidden.txt
    // ├── data.dat
    // ├── file.tar.gz
    // ├── other.md.gz
    // ├── root.log
    // └── root.txt
    //
    fn create_temp_folder() -> TempDir {
        let temp_dir = tempdir().unwrap();

        // Create files in the root directory
        File::create(temp_dir.path().join("root.txt")).unwrap();
        File::create(temp_dir.path().join("root.log")).unwrap();
        File::create(temp_dir.path().join("data.dat")).unwrap();
        File::create(temp_dir.path().join(".hidden.txt")).unwrap();
        File::create(temp_dir.path().join("file.tar.gz")).unwrap();
        File::create(temp_dir.path().join("other.md.gz")).unwrap();

        // Create subfolder1
        let subfolder1 = temp_dir.path().join("subfolder1");
        fs::create_dir(&subfolder1).unwrap();

        // Create files in subfolder1
        File::create(subfolder1.join("sub1.txt")).unwrap();
        File::create(subfolder1.join("sub1.log")).unwrap();

        // Create subfolder2 inside subfolder1
        let subfolder2 = subfolder1.join("subfolder2");
        fs::create_dir(&subfolder2).unwrap();

        // Create files in subfolder2
        File::create(subfolder2.join("sub2.txt")).unwrap();
        File::create(subfolder2.join("backup.bak")).unwrap();
        File::create(subfolder2.join("data.dat")).unwrap();

        // Create a hidden subfolder
        let hidden_folder = temp_dir.path().join(".hidden_folder");
        fs::create_dir(&hidden_folder).unwrap();

        // Create a file in the hidden folder
        File::create(hidden_folder.join("hidden.txt")).unwrap();

        temp_dir
    }

    #[test]
    fn get_fileext_normal() -> Result<(), Box<dyn Error>> {
        let filename = "file.txt".to_string();

        let extension = get_fileext(&filename).unwrap();

        assert_eq!("txt", extension);

        Ok(())
    }

    #[test]
    fn get_fileext_hidden_file() -> Result<(), Box<dyn Error>> {
        let filename = ".file.tar.gz".to_string();

        let extension = get_fileext(&filename).unwrap();

        assert_eq!("tar.gz", extension);

        Ok(())
    }

    #[test]
    fn collect_without_match() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["json".to_string()];
        let options = CollectOptions {
            all: false,
            list: false,
            recurse: false,
            invert: false,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 0);

        Ok(())
    }

    #[test]
    fn collect_with_extension_suffix_should_fail() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["g".to_string()]; // For root.log
        let options = CollectOptions {
            all: false,
            list: false,
            recurse: false,
            invert: false,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 0);

        Ok(())
    }

    #[test]
    fn collect_one_extension() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["txt".to_string()];
        let options = CollectOptions {
            all: false,
            list: false,
            recurse: false,
            invert: false,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 1);
        assert_eq!(path_buf.join("root.txt"), files[0]);

        Ok(())
    }

    #[test]
    fn collect_reverse() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["txt".to_string()];
        let options = CollectOptions {
            all: false,
            list: false,
            recurse: false,
            invert: true,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 4);
        assert!(files.contains(&path_buf.join("root.log")));
        assert!(files.contains(&path_buf.join("data.dat")));
        assert!(files.contains(&path_buf.join("file.tar.gz")));
        assert!(files.contains(&path_buf.join("other.md.gz")));

        Ok(())
    }

    #[test]
    fn collect_with_hidden_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["txt".to_string()];
        let options = CollectOptions {
            all: true,
            list: false,
            recurse: false,
            invert: false,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 2);
        assert!(files.contains(&path_buf.join("root.txt")));
        assert!(files.contains(&path_buf.join(".hidden.txt")));

        Ok(())
    }

    #[test]
    fn collect_reverse_with_hidden_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["dat".to_string()];
        let options = CollectOptions {
            all: true,
            list: false,
            recurse: false,
            invert: true,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 5);
        assert!(files.contains(&path_buf.join("root.txt")));
        assert!(files.contains(&path_buf.join("root.log")));
        assert!(files.contains(&path_buf.join(".hidden.txt")));
        assert!(files.contains(&path_buf.join("file.tar.gz")));
        assert!(files.contains(&path_buf.join("other.md.gz")));

        Ok(())
    }

    #[test]
    fn collect_multiple_extension() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["dat".to_string(), "txt".to_string()];
        let options = CollectOptions {
            all: true,
            list: false,
            recurse: false,
            invert: true,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 3);
        assert!(files.contains(&path_buf.join("root.log")));
        assert!(files.contains(&path_buf.join("file.tar.gz")));
        assert!(files.contains(&path_buf.join("other.md.gz")));

        Ok(())
    }

    #[test]
    fn collect_recursive() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["txt".to_string()];
        let options = CollectOptions {
            all: false,
            list: false,
            recurse: true,
            invert: false,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 3);
        assert!(files.contains(&path_buf.join("root.txt")));
        assert!(files.contains(&path_buf.join("subfolder1").join("sub1.txt")));
        assert!(
            files.contains(
                &path_buf
                    .join("subfolder1")
                    .join("subfolder2")
                    .join("sub2.txt")
            )
        );

        Ok(())
    }

    #[test]
    fn collect_should_not_traverse_symlinks() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();

        let other_temp_dir = tempdir().unwrap();
        let path_buf = other_temp_dir.path().to_path_buf();

        let link = path_buf.join("link");

        // Creating a symbolic link to the real directory
        symlink(&temp_dir, &link)?;

        // Verify that the symlink works by checking that we can see files through it
        let to_temp_dir = fs::read_link(&link).unwrap();
        let file_count = fs::read_dir(&to_temp_dir)?.count();
        assert_eq!(file_count, 8);

        let extensions = vec!["txt".to_string()];
        let options = CollectOptions {
            all: true,
            list: false,
            recurse: true,
            invert: false,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 0);

        Ok(())
    }

    #[test]
    fn collect_multi_dot_extension_with_extension_start() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["tar".to_string()];
        let options = CollectOptions {
            all: true,
            list: false,
            recurse: false,
            invert: false,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 0);

        Ok(())
    }

    #[test]
    fn collect_multi_dot_extension_with_complete_extension() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["tar.gz".to_string()];
        let options = CollectOptions {
            all: true,
            list: false,
            recurse: false,
            invert: false,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 1);
        assert!(files.contains(&path_buf.join("file.tar.gz")));

        Ok(())
    }

    #[test]
    fn collect_multi_dot_extension_with_last_part() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let extensions = vec!["gz".to_string()];
        let options = CollectOptions {
            all: true,
            list: false,
            recurse: false,
            invert: false,
        };

        let files = collect_matching_files(&extensions, &path_buf, &options)?;

        assert_eq!(files.len(), 2);
        assert!(files.contains(&path_buf.join("file.tar.gz")));
        assert!(files.contains(&path_buf.join("other.md.gz")));

        Ok(())
    }

    #[test]
    fn delete_one_file() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let options = DeleteOptions {
            dry_run: false,
            force: true,
        };

        let mut files: Vec<PathBuf> = Vec::new();
        let file = path_buf.join("root.txt");
        files.push(file.clone());

        assert!(file.exists());

        delete_files(&files, &options)?;

        assert!(!file.exists());

        Ok(())
    }

    #[test]
    fn delete_multiple_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let options = DeleteOptions {
            dry_run: false,
            force: true,
        };

        let mut files: Vec<PathBuf> = Vec::new();
        let first_file = path_buf.join("root.txt");
        let second_file = path_buf
            .join("subfolder1")
            .join("subfolder2")
            .join("sub2.txt");
        files.push(first_file.clone());
        files.push(second_file.clone());

        assert!(first_file.exists());
        assert!(second_file.exists());

        delete_files(&files, &options)?;

        assert!(!first_file.exists());
        assert!(!second_file.exists());

        Ok(())
    }

    #[test]
    fn delete_dry_run() -> Result<(), Box<dyn Error>> {
        let temp_dir = create_temp_folder();
        let path_buf = temp_dir.path().to_path_buf();

        let options = DeleteOptions {
            dry_run: true,
            force: true,
        };

        let mut files: Vec<PathBuf> = Vec::new();
        let first_file = path_buf.join("root.txt");
        let second_file = path_buf
            .join("subfolder1")
            .join("subfolder2")
            .join("sub2.txt");
        files.push(first_file.clone());
        files.push(second_file.clone());

        assert!(first_file.exists());
        assert!(second_file.exists());

        delete_files(&files, &options)?;

        assert!(first_file.exists());
        assert!(second_file.exists());

        Ok(())
    }
}
