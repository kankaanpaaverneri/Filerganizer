use crate::directory;
use crate::directory::Directory;
use crate::file::File;
use crate::layouts::{CheckboxStates, IndexPosition};
use crate::metadata::DateType;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::ErrorKind;

pub struct OrganizingData<'a> {
    files_selected: BTreeMap<OsString, File>,
    checkbox_states: CheckboxStates,
    new_directory_name: &'a str,
    custom_file_name: &'a str,
    file_name_component_order: &'a Vec<String>,
    date_type: Option<DateType>,
    index_position: Option<IndexPosition>,
}

impl<'a> OrganizingData<'a> {
    pub fn new(
        files_selected: BTreeMap<OsString, File>,
        checkbox_states: CheckboxStates,
        new_directory_name: &'a str,
        custom_file_name: &'a str,
        file_name_component_order: &'a Vec<String>,
        date_type: Option<DateType>,
        index_position: Option<IndexPosition>,
    ) -> Self {
        Self {
            files_selected,
            checkbox_states,
            new_directory_name,
            custom_file_name,
            file_name_component_order,
            date_type,
            index_position,
        }
    }
}

pub fn apply_rules_for_directory(
    new_directory_name: String,
    selected_directory: &mut Directory,
    data: OrganizingData,
) -> std::io::Result<()> {
    // If both organize_by_file_type and date are checked
    if data.checkbox_states.organize_by_filetype && data.checkbox_states.organize_by_date {
        let mut new_directory = Directory::new(None);
        match organize_files_by_file_type_and_date(&mut new_directory, data) {
            Ok(_) => {
                selected_directory.insert_directory(new_directory, &new_directory_name);
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_filetype {
        let mut new_directory = Directory::new(None);
        match organize_files_by_file_type(&mut new_directory, data) {
            Ok(directories_by_file_type) => {
                new_directory.insert_directories(directories_by_file_type);
                selected_directory.insert_directory(new_directory, &new_directory_name);
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_date {
        // If only organize_by_date is checked
        let mut new_directory = Directory::new(None);
        match organize_files_by_date(&mut new_directory, data) {
            Ok(directories_by_date) => {
                new_directory.insert_directories(directories_by_date);
                selected_directory.insert_directory(new_directory, &new_directory_name);
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.insert_directory_name_to_file_name
        || data.checkbox_states.insert_date_to_file_name
        || data.checkbox_states.remove_uppercase
        || data.checkbox_states.replace_spaces_with_underscores
        || data.checkbox_states.use_only_ascii
        || data.checkbox_states.remove_original_file_name
        || data.checkbox_states.add_custom_name
    {
        let mut new_directory = Directory::new(None);
        match rename_files(data) {
            Ok(renamed_files) => match new_directory.contains_unique_files(&renamed_files) {
                Ok(_) => {
                    for (file_name, file) in renamed_files {
                        new_directory.insert_file(file_name, file);
                    }
                    selected_directory.insert_directory(new_directory, &new_directory_name);
                    return Ok(());
                }
                Err(error) => return Err(error),
            },
            Err(error) => return Err(error),
        }
    } else if !data.checkbox_states.organize_by_filetype
        && !data.checkbox_states.organize_by_filetype
        && !data.checkbox_states.insert_date_to_file_name
        && !data.checkbox_states.insert_directory_name_to_file_name
        && !data.checkbox_states.remove_uppercase
        && !data.checkbox_states.replace_spaces_with_underscores
        && !data.checkbox_states.use_only_ascii
        && !data.checkbox_states.remove_original_file_name
        && !data.checkbox_states.add_custom_name
    {
        let mut new_directory = Directory::new(None);
        for (key, value) in data.files_selected {
            new_directory.insert_file(key, value);
        }
        selected_directory.insert_directory(new_directory, &new_directory_name);
        return Ok(());
    }
    Err(std::io::Error::new(
        ErrorKind::Other,
        "Rules didn't match any case",
    ))
}

pub fn move_files_to_organized_directory(
    files_selected: BTreeMap<OsString, File>,
    selected_directory: &mut Directory,
    directory_name: &str,
    custom_file_name: &str,
    file_name_component_order: &Vec<String>,
    checkbox_states: CheckboxStates,
    date_type: Option<DateType>,
    index_position: Option<IndexPosition>,
) -> std::io::Result<()> {
    let data = OrganizingData {
        files_selected,
        checkbox_states,
        new_directory_name: directory_name,
        file_name_component_order,
        custom_file_name,
        date_type,
        index_position,
    };
    if data.checkbox_states.organize_by_filetype && data.checkbox_states.organize_by_date {
        // Check files before inserting
        match organize_files_by_file_type_and_date(selected_directory, data) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_filetype {
        // Check files before inserting
        if let Err(error) = selected_directory.contains_unique_files_recursive(&data.files_selected)
        {
            return Err(error);
        }
        match organize_files_by_file_type(selected_directory, data) {
            Ok(file_type_directories) => {
                selected_directory.insert_directories(file_type_directories);
            }
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_date {
        // Check files before inserting
        match organize_files_by_date(selected_directory, data) {
            Ok(directories_by_date) => {
                selected_directory.insert_directories(directories_by_date);
            }
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.insert_directory_name_to_file_name
        || data.checkbox_states.insert_date_to_file_name
        || data.checkbox_states.remove_uppercase
        || data.checkbox_states.replace_spaces_with_underscores
        || data.checkbox_states.use_only_ascii
        || data.checkbox_states.remove_original_file_name
        || data.checkbox_states.add_custom_name
    {
        match rename_files(data) {
            Ok(renamed_files) => match selected_directory.contains_unique_files(&renamed_files) {
                Ok(_) => {
                    for (file_name, file) in renamed_files {
                        selected_directory.insert_file(file_name, file);
                    }
                    return Ok(());
                }
                Err(error) => return Err(error),
            },
            Err(error) => return Err(error),
        }
    } else if !data.checkbox_states.organize_by_filetype
        && !data.checkbox_states.organize_by_filetype
        && !data.checkbox_states.insert_date_to_file_name
        && !data.checkbox_states.insert_directory_name_to_file_name
        && !data.checkbox_states.remove_uppercase
        && !data.checkbox_states.replace_spaces_with_underscores
        && !data.checkbox_states.use_only_ascii
        && !data.checkbox_states.remove_original_file_name
        && !data.checkbox_states.add_custom_name
    {
        if let Err(error) = selected_directory.contains_unique_files(&data.files_selected) {
            return Err(error);
        }

        for (key, value) in data.files_selected {
            selected_directory.insert_file(key, value);
        }
    } else {
        return Err(std::io::Error::new(
            ErrorKind::NotFound,
            "No selected directory found",
        ));
    }

    Ok(())
}

fn organize_files_by_file_type_and_date(
    selected_directory: &mut Directory,
    data: OrganizingData,
) -> std::io::Result<()> {
    if let Some(date_type_selected) = data.date_type {
        let mut directories_by_file_type = directory::organizing::sort_files_by_file_type(
            data.files_selected,
            &data.checkbox_states,
            data.new_directory_name,
            data.custom_file_name,
            data.file_name_component_order,
            Some(date_type_selected),
            data.index_position,
        );
        move_files_from_duplicate_directories(selected_directory, &mut directories_by_file_type)?;
        directory::remove_empty_directories(&mut directories_by_file_type);
        selected_directory.insert_directories(directories_by_file_type);

        if let Some(directories_by_file_type) = selected_directory.get_mut_directories() {
            for (_key, directory) in directories_by_file_type {
                if let Some(files) = directory.get_mut_files().take() {
                    let mut directories_by_date = directory::organizing::sort_files_by_date(
                        files,
                        &CheckboxStates::default(),
                        data.new_directory_name,
                        data.custom_file_name,
                        data.file_name_component_order,
                        date_type_selected,
                        data.index_position,
                    );
                    move_files_from_duplicate_directories(directory, &mut directories_by_date)?;
                    directory::remove_empty_directories(&mut directories_by_date);
                    directory.insert_directories(directories_by_date);
                }
            }
            return Ok(());
        }
        Err(std::io::Error::new(
            ErrorKind::Other,
            "No directories by file type found",
        ))
    } else {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Date type not specified.",
        ));
    }
}

fn organize_files_by_file_type(
    selected_directory: &mut Directory,
    data: OrganizingData,
) -> std::io::Result<BTreeMap<OsString, Directory>> {
    if let None = data.date_type {
        if data.checkbox_states.insert_date_to_file_name {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Date type not specified.",
            ));
        }
    }
    let mut file_type_directories = directory::organizing::sort_files_by_file_type(
        data.files_selected,
        &data.checkbox_states,
        data.new_directory_name,
        data.custom_file_name,
        data.file_name_component_order,
        data.date_type,
        data.index_position,
    );
    move_files_from_duplicate_directories(selected_directory, &mut file_type_directories)?;
    directory::remove_empty_directories(&mut file_type_directories);
    Ok(file_type_directories)
}

fn organize_files_by_date(
    selected_directory: &mut Directory,
    data: OrganizingData,
) -> std::io::Result<BTreeMap<OsString, Directory>> {
    if let Some(date_type) = data.date_type {
        let mut directories_by_date = directory::organizing::sort_files_by_date(
            data.files_selected,
            &data.checkbox_states,
            data.new_directory_name,
            data.custom_file_name,
            data.file_name_component_order,
            date_type,
            data.index_position,
        );
        move_files_from_duplicate_directories(selected_directory, &mut directories_by_date)?;
        directory::remove_empty_directories(&mut directories_by_date);
        Ok(directories_by_date)
    } else {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Date type not specified.",
        ));
    }
}

fn rename_files(data: OrganizingData) -> std::io::Result<BTreeMap<OsString, File>> {
    if let None = data.date_type {
        if data.checkbox_states.insert_date_to_file_name {
            return Err(std::io::Error::new(
                ErrorKind::NotFound,
                "No date type specified",
            ));
        }
    }

    // If only renaming are checked
    let mut renamed_files = BTreeMap::new();
    for (key, file) in data.files_selected {
        if let Some(file_name) = key.to_str() {
            let mut renamed_file_name = String::new();
            let file_count = renamed_files.len();
            directory::organizing::rename_file_name(
                &mut renamed_file_name,
                &data.checkbox_states,
                data.new_directory_name,
                data.custom_file_name,
                file_count,
                data.file_name_component_order,
                file_name,
                &file,
                data.date_type,
                data.index_position,
            );
            renamed_files.insert(OsString::from(renamed_file_name), file);
        }
    }

    Ok(renamed_files)
}

fn move_files_from_duplicate_directories(
    selected_directory: &mut Directory,
    new_directories: &mut BTreeMap<OsString, Directory>,
) -> std::io::Result<()> {
    if let Some(selected_directories) = selected_directory.get_mut_directories() {
        for (new_key, new_dir) in new_directories {
            if selected_directories.contains_key(new_key) {
                if let Some(files) = new_dir.get_mut_files().take() {
                    if let Some(selected_directory) = selected_directories.get_mut(new_key) {
                        // Do some checking with the files that overwriting doesn't happen
                        selected_directory.contains_unique_files(&files)?;
                        for (file_name, file) in files {
                            selected_directory.insert_file(file_name, file);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
