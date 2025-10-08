use crate::directory::Directory;
use crate::file::File;
use crate::layouts::CheckboxStates;
use crate::metadata::DateType;
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

pub fn select_files_in_boundary(
    in_file_boundaries: bool,
    files_selected: &mut BTreeMap<OsString, File>,
    files_unselected: &mut BTreeMap<OsString, File>,
    key: &OsStr,
    value: File,
) {
    if in_file_boundaries {
        files_selected.insert(OsString::from(key), value);
    } else {
        files_unselected.insert(OsString::from(key), value);
    }
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
    use crate::metadata::Metadata;

    #[test]
    fn test_just_rename_checked() {
        let checkbox_states =
            CheckboxStates::new(false, false, true, true, true, true, true, true, true);
        assert_eq!(just_rename_checked(&checkbox_states), true);
        let checkbox_states =
            CheckboxStates::new(true, true, false, false, false, false, false, false, false);
        assert_eq!(just_rename_checked(&checkbox_states), false);
    }

    fn create_dummy_files() -> BTreeMap<OsString, File> {
        let mut files = BTreeMap::new();
        files.insert(OsString::from("file1.txt"), File::new(Metadata::new()));
        files.insert(OsString::from("file2.txt"), File::new(Metadata::new()));
        files.insert(OsString::from("file3.txt"), File::new(Metadata::new()));
        files.insert(OsString::from("file4.txt"), File::new(Metadata::new()));
        files
    }

    fn create_dummy_files_selected() -> BTreeMap<OsString, File> {
        let mut files_selected = BTreeMap::new();
        files_selected.insert(OsString::from("image0.jpg"), File::new(Metadata::new()));
        files_selected.insert(OsString::from("image1.jpg"), File::new(Metadata::new()));
        files_selected.insert(OsString::from("image2.jpg"), File::new(Metadata::new()));
        files_selected.insert(OsString::from("image3.jpg"), File::new(Metadata::new()));
        files_selected
    }

    #[test]
    fn test_select_file() {
        let mut files = create_dummy_files();
        let mut files_selected = create_dummy_files_selected();
        match select_file(
            &mut files,
            &mut files_selected,
            &OsString::from("file3.txt"),
        ) {
            Ok(()) => {
                assert!(files_selected.contains_key(&OsString::from("file3.txt")));
            }
            Err(error) => {
                panic!("Error in select file: {}", error);
            }
        }
        files.insert(OsString::from("file3.txt"), File::new(Metadata::new()));
        match select_file(
            &mut files,
            &mut files_selected,
            &OsString::from("file3.txt"),
        ) {
            Ok(()) => {
                panic!("File should not have been able to select");
            }
            Err(error) => {
                assert_eq!(error.to_string(), String::from("Duplicate file name found"))
            }
        }
    }

    #[test]
    fn test_are_paths_equal() {
        let path1 = PathBuf::from("/home/verneri/screen_record");
        let path2 = PathBuf::from("/home/verneri/rust");
        assert_eq!(are_paths_equal(&path1, &path2), false);
        let path3 = PathBuf::from("/home/verneri/screen_record");
        assert_eq!(are_paths_equal(&path1, &path3), true);
    }

    fn create_dummy_directory() -> Directory {
        let mut directory = Directory::new(None);
        directory.insert_file(OsString::from("file1.txt"), File::new(Metadata::new()));
        directory.insert_file(OsString::from("file2.txt"), File::new(Metadata::new()));
        directory.insert_file(OsString::from("file3.txt"), File::new(Metadata::new()));
        directory.insert_file(OsString::from("file4.txt"), File::new(Metadata::new()));
        directory
    }

    #[test]
    fn test_directories_have_duplicate_files() {
        let dir1 = create_dummy_directory();
        let dir2 = create_dummy_directory();
        assert_eq!(directories_have_duplicate_files(&dir1, &dir2), true);
        let mut dir3 = Directory::new(None);
        dir3.insert_file(OsString::from("image.jpg"), File::new(Metadata::new()));
        assert_eq!(directories_have_duplicate_files(&dir1, &dir3), false);
    }

    fn create_dummy_directory_with_directories() -> Directory {
        let mut directory = Directory::new(None);
        directory.insert_directory(Directory::new(None), "content");
        directory.insert_directory(Directory::new(None), "other");
        directory
    }

    #[test]
    fn test_directories_have_duplicate_directories() {
        let dir1 = create_dummy_directory_with_directories();
        let dir2 = create_dummy_directory_with_directories();
        assert_eq!(directories_have_duplicate_directories(&dir1, &dir2), true);
        let mut dir3 = Directory::new(None);
        dir3.insert_directory(Directory::new(None), "new_dir");
        assert_eq!(directories_have_duplicate_directories(&dir1, &dir3), false);
    }
}
