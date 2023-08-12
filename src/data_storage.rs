use models::Project;
use std::{
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use owo_colors::OwoColorize;

use crate::models;

pub enum PicoPath {
    NotFound(gix_discover::upwards::Error),
    FoundNotInit(PathBuf),
    Found(PathBuf),
}

pub fn check_pico_dir() -> PicoPath {
    let git_location = gix_discover::upwards(Path::new("."));

    let git_location = match git_location {
        Ok(git_path) => git_path.0,
        Err(err) => {
            return PicoPath::NotFound(err);
        }
    };

    let (_, github_folder_dir) = git_location.into_repository_and_work_tree_directories();

    let pico_json_path = github_folder_dir.unwrap().join(".pico");
    if pico_json_path.exists() {
        PicoPath::Found(pico_json_path)
    } else {
        PicoPath::FoundNotInit(pico_json_path)
    }
}

pub(crate) fn store_project(p: &Project) -> Result<(), std::io::Error> {
    let pico = check_pico_dir();

    let path = match pico {
        PicoPath::NotFound(err) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Unable to store .pico file: {}", err),
            ))
        }
        PicoPath::FoundNotInit(path) | PicoPath::Found(path) => Some(path),
    };
    let mut file = File::create(path.unwrap())?;
    let serialized = serde_json::to_string(p)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

pub(crate) fn load_project() -> Result<Project, std::io::Error> {
    let pico = check_pico_dir();

    let path = match pico {
        PicoPath::NotFound(err) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Unable to load project, .pico file could not be found: {}",
                    err
                ),
            ));
        }
        PicoPath::Found(path) => Some(path),
        PicoPath::FoundNotInit(path) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Unable to load project, .pico file could not be found: {:?} - {} ",
                    path,
                    "try running `pico init`".green(),
                ),
            ));
        }
    };

    let file = File::open(path.unwrap())?;
    let rdr = BufReader::new(file);
    let p: Project = serde_json::from_reader(rdr)?;
    Ok(p)
}
