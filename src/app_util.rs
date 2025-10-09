use crate::directory::Directory;
use crate::file::File;
use crate::layouts::CheckboxStates;
use crate::metadata::DateType;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::PathBuf;

pub fn is_duplicate_files_in_files_selected(
    root_dir: &Directory,
    files_selected: &BTreeMap<OsString, File>,
    path: &PathBuf,
) -> std::io::Result<()> {
    let selected_dir = root_dir.get_directory_by_path(path);
    if let Some(files) = selected_dir.get_files() {
        for key in files.keys() {
            if files_selected.contains_key(key) {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "Duplicate file found in files selected and directory.",
                ));
            }
        }
    }
    Ok(())
}

pub fn is_duplicate_files_in_directory_selection(
    files_selected: &BTreeMap<OsString, File>,
    original_files_selected: &BTreeMap<OsString, File>,
) -> std::io::Result<()> {
    for key in original_files_selected.keys() {
        if files_selected.contains_key(key) {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Duplicate file name found in directory and files selected",
            ));
        }
    }
    Ok(())
}

pub fn convert_os_str_to_str(key: &OsStr) -> std::io::Result<&str> {
    if let Some(key) = key.to_str() {
        return Ok(key);
    }
    Err(std::io::Error::new(
        ErrorKind::Other,
        "Could not parse &OsStr to &str",
    ))
}

pub fn convert_path_to_str<'a>(path: &'a PathBuf) -> std::io::Result<&'a str> {
    if let Some(path) = path.to_str() {
        return Ok(path);
    }
    Err(std::io::Error::new(
        ErrorKind::Other,
        "Coult not parse PathBuf to &str",
    ))
}

pub fn just_rename_checked(checkbox_states: &CheckboxStates) -> bool {
    if checkbox_states.insert_directory_name_to_file_name
        || checkbox_states.insert_date_to_file_name
        || checkbox_states.convert_uppercase_to_lowercase
        || checkbox_states.replace_character
        || checkbox_states.use_only_ascii
        || checkbox_states.remove_original_file_name
        || checkbox_states.add_custom_name
    {
        return true;
    }
    return false;
}

pub fn get_date_type(date_type: Option<DateType>) -> std::io::Result<DateType> {
    if let Some(date_type) = date_type {
        return Ok(date_type);
    }
    Err(std::io::Error::new(
        ErrorKind::NotFound,
        "Date type not specified.",
    ))
}

pub fn is_substring(needle: &str, haystack: &str) -> usize {
    let mut score = 0;
    let mut iterator = needle.chars();
    for hay in haystack.chars() {
        if let Some(next) = iterator.next() {
            if hay == next {
                score += 1;
            } else {
                return score;
            }
        } else {
            break;
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_just_rename_checked() {
        let checkbox_states =
            CheckboxStates::new(false, false, true, true, true, true, true, true, true);
        assert_eq!(just_rename_checked(&checkbox_states), true);
        let checkbox_states =
            CheckboxStates::new(true, true, false, false, false, false, false, false, false);
        assert_eq!(just_rename_checked(&checkbox_states), false);
    }
}
