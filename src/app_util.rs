use crate::directory::Directory;
use crate::file::File;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::PathBuf;

pub fn directories_have_duplicate_directories(
    parent_dir: &Directory,
    selected_dir: &Directory,
) -> bool {
    if let Some(selected_directories) = selected_dir.get_directories() {
        if let Some(parent_directories) = parent_dir.get_directories() {
            for key in selected_directories.keys() {
                if parent_directories.contains_key(key) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn directories_have_duplicate_files(parent_dir: &Directory, selected_dir: &Directory) -> bool {
    if let Some(selected_files) = selected_dir.get_files() {
        if let Some(parent_files) = parent_dir.get_files() {
            for key in selected_files.keys() {
                if parent_files.contains_key(key) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn are_paths_equal(path1: &PathBuf, path2: &PathBuf) -> bool {
    let mut components = path2.components();
    for current_path in path1.iter() {
        if let Some(component) = components.next() {
            if component.as_os_str() != current_path {
                return false;
            }
        }
    }
    true
}

pub fn select_file(
    files: &mut BTreeMap<OsString, File>,
    files_selected: &mut BTreeMap<OsString, File>,
    file_name: &OsStr,
) -> std::io::Result<()> {
    if files_selected.contains_key(file_name) {
        if files.contains_key(file_name) {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Duplicate file name found",
            ));
        }
        if let Some((key, value)) = files_selected.remove_entry(file_name) {
            files.insert(key, value);
        }
    } else {
        if let Some((key, value)) = files.remove_entry(file_name) {
            files_selected.insert(key, value);
        }
    }
    Ok(())
}
