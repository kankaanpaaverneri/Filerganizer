use std::ffi::OsString;
use std::collections::BTreeMap;
use crate::file::File;
use std::path::PathBuf;
use std::fs;

pub fn move_files_organized(files_organized: &BTreeMap<OsString, File>) -> std::io::Result<()> {
    for file in files_organized.values() {
        if let Some(metadata) = file.get_metadata() {
            if let Some(destination_path) = metadata.get_destination_path() {
                create_missing_directories(PathBuf::from(&destination_path))?;
                if let Some(origin_path) = metadata.get_origin_path() {
                    fs::rename(origin_path, destination_path)?;
                }
            }
        }
    }
    Ok(()) 
}

fn create_missing_directories(destination_path: PathBuf) -> std::io::Result<()> {
    let mut search_path = PathBuf::new();
    for (i, component) in destination_path.components().enumerate() {
        if i == destination_path.components().count() - 1 {
            break;
        }
        search_path.push(component);
        let exists = fs::exists(&search_path)?; 
        if !exists {
            fs::create_dir(&search_path)?;
        }
    }
    Ok(())
}
