use assert_cmd::Command;
use predicates::prelude::*;
use std::error::Error;
use std::fs::{self, File};
use tempfile::{self, TempDir};

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
// ├── root.log
// └── root.txt
//

fn create_temp_folder() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create files in the root directory
    File::create(temp_dir.path().join("root.txt")).unwrap();
    File::create(temp_dir.path().join("root.log")).unwrap();
    File::create(temp_dir.path().join("data.dat")).unwrap();
    File::create(temp_dir.path().join(".hidden.txt")).unwrap();

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
fn it_no_extension_should_fail() -> Result<(), Box<dyn Error>> {
    let temp_dir = create_temp_folder();
    let path_buf = temp_dir.path().to_path_buf();

    Command::cargo_bin("rmx")?
        .arg("-n")
        .arg("-p")
        .arg(path_buf.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("<EXTENSIONS>"));

    Ok(())
}

#[test]
fn it_no_path_should_fail() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("rmx")?
        .arg("-n")
        .arg("-p")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--path <PATH>"));

    Ok(())
}

#[test]
fn it_default_behavior_no_delete() -> Result<(), Box<dyn Error>> {
    let temp_dir = create_temp_folder();
    let path_buf = temp_dir.path().to_path_buf();

    let expected_file = path_buf.clone().join("root.txt");
    let expected_file_str = expected_file.to_str().unwrap();

    assert!(expected_file.exists());

    Command::cargo_bin("rmx")?
        .arg("-n")
        .arg("-l")
        .arg("-p")
        .arg(path_buf.to_str().unwrap())
        .arg("txt")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(expected_file_str)
                .and(predicate::str::contains("Do you really want to delete").not()),
        );

    assert!(expected_file.exists());

    Ok(())
}

#[test]
fn it_default_behavior() -> Result<(), Box<dyn Error>> {
    let temp_dir = create_temp_folder();
    let path_buf = temp_dir.path().to_path_buf();

    let expected_file = path_buf.clone().join("root.txt");
    let control_file = path_buf.clone().join("root.log");
    // Control file does not end with .txt, so it should not be deleted

    assert!(expected_file.exists());
    assert!(control_file.exists());

    Command::cargo_bin("rmx")?
        .arg("-p")
        .arg(path_buf.to_str().unwrap())
        .arg("txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Do you really want to delete 1 "));

    assert!(!expected_file.exists());
    assert!(control_file.exists());

    Ok(())
}

#[test]
fn it_recursive() -> Result<(), Box<dyn Error>> {
    let temp_dir = create_temp_folder();
    let path_buf = temp_dir.path().to_path_buf();

    let files = [
        path_buf.clone().join("root.txt"),
        path_buf.clone().join("subfolder1").join("sub1.txt"),
        path_buf
            .clone()
            .join("subfolder1")
            .join("subfolder2")
            .join("sub2.txt"),
    ];
    let hidden_file = path_buf.clone().join(".hidden.txt");
    // Hidden files are not enabled here, so should not be deleted
    let control_file = path_buf.clone().join("root.log");
    // Control file does not end with .txt, so it should not be deleted

    assert!(files.iter().all(|f| f.exists()));
    assert!(hidden_file.exists());
    assert!(control_file.exists());

    Command::cargo_bin("rmx")?
        .arg("-r")
        .arg("-p")
        .arg(path_buf.to_str().unwrap())
        .arg("txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Do you really want to delete 3 "));

    assert!(files.iter().all(|f| !f.exists()));
    assert!(hidden_file.exists());
    assert!(control_file.exists());

    Ok(())
}

#[test]
fn it_recursive_with_hidden_files() -> Result<(), Box<dyn Error>> {
    let temp_dir = create_temp_folder();
    let path_buf = temp_dir.path().to_path_buf();

    let files = [
        path_buf.clone().join("root.txt"),
        path_buf.clone().join("subfolder1").join("sub1.txt"),
        path_buf
            .clone()
            .join("subfolder1")
            .join("subfolder2")
            .join("sub2.txt"),
        path_buf.clone().join(".hidden.txt"),
        path_buf.clone().join(".hidden_folder").join("hidden.txt"),
    ];

    let control_file = path_buf.clone().join("root.log");
    // Control file does not end with .txt, so it should not be deleted

    assert!(files.iter().all(|f| f.exists()));
    assert!(control_file.exists());

    Command::cargo_bin("rmx")?
        .arg("-r")
        .arg("-a")
        .arg("-p")
        .arg(path_buf.to_str().unwrap())
        .arg("txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Do you really want to delete 5 "));

    assert!(files.iter().all(|f| !f.exists()));
    assert!(control_file.exists());

    Ok(())
}

#[test]
fn it_recursive_reverse_with_hidden_files() -> Result<(), Box<dyn Error>> {
    let temp_dir = create_temp_folder();
    let path_buf = temp_dir.path().to_path_buf();

    let files = [
        path_buf.clone().join("root.log"),
        path_buf.clone().join("data.dat"),
        path_buf.clone().join("subfolder1").join("sub1.log"),
        path_buf
            .clone()
            .join("subfolder1")
            .join("subfolder2")
            .join("data.dat"),
        path_buf
            .clone()
            .join("subfolder1")
            .join("subfolder2")
            .join("backup.bak"),
    ];

    let first_control_file = path_buf.clone().join("root.txt");
    let second_control_file = path_buf.clone().join(".hidden.txt");
    // Control files end with .txt, so should not be deleted

    assert!(files.iter().all(|f| f.exists()));
    assert!(first_control_file.exists());
    assert!(second_control_file.exists());

    Command::cargo_bin("rmx")?
        .arg("-i")
        .arg("-r")
        .arg("-a")
        .arg("-p")
        .arg(path_buf.to_str().unwrap())
        .arg("txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Do you really want to delete 5 "));

    assert!(files.iter().all(|f| !f.exists()));
    assert!(first_control_file.exists());
    assert!(second_control_file.exists());

    Ok(())
}

#[test]
fn it_list() -> Result<(), Box<dyn Error>> {
    let temp_dir = create_temp_folder();
    let path_buf = temp_dir.path().to_path_buf();

    let files = [
        path_buf.clone().join("root.txt"),
        path_buf.clone().join("subfolder1").join("sub1.txt"),
        path_buf
            .clone()
            .join("subfolder1")
            .join("subfolder2")
            .join("sub2.txt"),
        path_buf.clone().join(".hidden.txt"),
        path_buf.clone().join(".hidden_folder").join("hidden.txt"),
    ];

    let control_file = path_buf.clone().join("root.log");
    // Control file does not end with .txt, so it should not be deleted

    assert!(files.iter().all(|f| f.exists()));
    assert!(control_file.exists());

    let test_filename = path_buf.clone().join(".hidden_folder").join("hidden.txt");
    let test_filename = test_filename.to_str().unwrap();

    Command::cargo_bin("rmx")?
        .arg("-l")
        .arg("-n")
        .arg("-r")
        .arg("-a")
        .arg("-p")
        .arg(path_buf.to_str().unwrap())
        .arg("txt")
        .assert()
        .success()
        .stdout(predicate::str::contains(test_filename));

    assert!(files.iter().all(|f| f.exists()));
    assert!(control_file.exists());

    Ok(())
}

#[test]
fn it_no_matching_file_should_success() -> Result<(), Box<dyn Error>> {
    let temp_dir = create_temp_folder();
    let path_buf = temp_dir.path().to_path_buf();

    let control_file = path_buf.clone().join("root.log");
    assert!(control_file.exists());

    Command::cargo_bin("rmx")?
        .arg("-l")
        .arg("-n")
        .arg("-r")
        .arg("-a")
        .arg("-p")
        .arg(path_buf.to_str().unwrap())
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching file"));

    assert!(control_file.exists());

    Ok(())
}

#[test]
fn it_using_help_should_success() -> Result<(), Box<dyn Error>> {
    let temp_dir = create_temp_folder();
    let path_buf = temp_dir.path().to_path_buf();

    let control_file = path_buf.clone().join("root.log");
    assert!(control_file.exists());

    Command::cargo_bin("rmx")?.arg("-h").assert().success();

    assert!(control_file.exists());

    Ok(())
}
