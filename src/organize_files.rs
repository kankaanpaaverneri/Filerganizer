use crate::directory;
use crate::directory::Directory;
use crate::file::File;
use crate::layouts::{CheckboxStates, IndexPosition};
use crate::metadata::DateType;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::ErrorKind;

#[derive(Debug, Clone)]
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
            Ok(_) => {
                selected_directory.insert_directory(new_directory, &new_directory_name);
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_date {
        // If only organize_by_date is checked
        let mut new_directory = Directory::new(None);
        match organize_files_by_date(&mut new_directory, data) {
            Ok(_) => {
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
        match organize_files_by_file_type_and_date(selected_directory, data) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_filetype {
        if let Err(error) = selected_directory.contains_unique_files_recursive(&data.files_selected)
        {
            return Err(error);
        }
        match organize_files_by_file_type(selected_directory, data) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_date {
        // Check files before inserting
        match organize_files_by_date(selected_directory, data) {
            Ok(_) => {}
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
        let mut file_type_dirs = directory::organizing::get_file_types(&data.files_selected);
        selected_directory.filter_duplicate_directories(&mut file_type_dirs);

        for (dir_name, new_dir) in file_type_dirs {
            if let Some(dir_name) = dir_name.to_str() {
                selected_directory.insert_directory(new_dir, dir_name);
            }
        }
        if let Some(file_type_dirs) = selected_directory.get_mut_directories() {
            directory::organizing::sort_files_by_file_type(
                data.files_selected,
                file_type_dirs,
                &data.checkbox_states,
                data.new_directory_name,
                data.custom_file_name,
                data.file_name_component_order,
                data.date_type,
                data.index_position,
                false,
            )?;

            // After this organize by date as well
            for (_dir_name, dir) in file_type_dirs {
                if let Some(files_by_filetype) = dir.get_mut_files().take() {
                    let new_data = OrganizingData::new(
                        files_by_filetype,
                        data.checkbox_states.clone(),
                        data.new_directory_name,
                        data.custom_file_name,
                        data.file_name_component_order,
                        Some(date_type_selected),
                        data.index_position.clone(),
                    );
                    organize_files_by_date(dir, new_data)?;
                }
            }
        }
        return Ok(());
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
) -> std::io::Result<()> {
    if let None = data.date_type {
        if data.checkbox_states.insert_date_to_file_name {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Date type not specified.",
            ));
        }
    }
    let mut file_type_dirs = directory::organizing::get_file_types(&data.files_selected);
    selected_directory.filter_duplicate_directories(&mut file_type_dirs);

    for (dir_name, new_dir) in file_type_dirs {
        if let Some(dir_name) = dir_name.to_str() {
            selected_directory.insert_directory(new_dir, dir_name);
        }
    }
    if let Some(file_type_dirs) = selected_directory.get_mut_directories() {
        directory::organizing::sort_files_by_file_type(
            data.files_selected,
            file_type_dirs,
            &data.checkbox_states,
            data.new_directory_name,
            data.custom_file_name,
            data.file_name_component_order,
            data.date_type,
            data.index_position,
            true,
        )?;
        return Ok(());
    }
    Err(std::io::Error::new(
        ErrorKind::NotFound,
        "No file type directories found",
    ))
}

fn organize_files_by_date(
    selected_directory: &mut Directory,
    data: OrganizingData,
) -> std::io::Result<()> {
    if let Some(date_type) = data.date_type {
        let mut file_date_dirs =
            directory::organizing::get_file_dates(&data.files_selected, date_type);
        selected_directory.filter_duplicate_directories(&mut file_date_dirs);

        for (dir_name, dir) in file_date_dirs {
            if let Some(dir_name) = dir_name.to_str() {
                selected_directory.insert_directory(dir, dir_name);
            }
        }
        if let Some(file_date_dirs) = selected_directory.get_mut_directories() {
            directory::organizing::sort_files_by_date(
                data.files_selected,
                file_date_dirs,
                &data.checkbox_states,
                data.new_directory_name,
                data.custom_file_name,
                data.file_name_component_order,
                date_type,
                data.index_position,
            )?;
        }
        Ok(())
    } else {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Date type not specified",
        ));
    }
}

fn rename_files(data: OrganizingData) -> std::io::Result<BTreeMap<OsString, File>> {
    if let None = data.date_type {
        if data.checkbox_states.insert_date_to_file_name {
            return Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Date type not specified",
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
