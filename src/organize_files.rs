use crate::app::filename_components;
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
        let mut file_type_dirs = get_file_types(&data.files_selected);
        selected_directory.filter_duplicate_directories(&mut file_type_dirs);

        for (dir_name, new_dir) in file_type_dirs {
            if let Some(dir_name) = dir_name.to_str() {
                selected_directory.insert_directory(new_dir, dir_name);
            }
        }
        if let Some(file_type_dirs) = selected_directory.get_mut_directories() {
            sort_files_by_file_type(
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
    let mut file_type_dirs = get_file_types(&data.files_selected);
    selected_directory.filter_duplicate_directories(&mut file_type_dirs);

    for (dir_name, new_dir) in file_type_dirs {
        if let Some(dir_name) = dir_name.to_str() {
            selected_directory.insert_directory(new_dir, dir_name);
        }
    }
    if let Some(file_type_dirs) = selected_directory.get_mut_directories() {
        sort_files_by_file_type(
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
        let mut file_date_dirs = get_file_dates(&data.files_selected, date_type);
        selected_directory.filter_duplicate_directories(&mut file_date_dirs);

        for (dir_name, dir) in file_date_dirs {
            if let Some(dir_name) = dir_name.to_str() {
                selected_directory.insert_directory(dir, dir_name);
            }
        }
        if let Some(file_date_dirs) = selected_directory.get_mut_directories() {
            sort_files_by_date(
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
            rename_file_name(
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

struct FilenameComponents {
    date: String,
    directory_name: String,
    custom_name: String,
    original_name: String,
    file_type: String,
}

impl FilenameComponents {
    pub fn new() -> Self {
        Self {
            date: String::new(),
            directory_name: String::new(),
            custom_name: String::new(),
            original_name: String::new(),
            file_type: String::new(),
        }
    }
}

pub fn sort_files_by_file_type(
    files_selected: BTreeMap<OsString, File>,
    file_type_directories: &mut BTreeMap<OsString, Directory>,
    checkbox_states: &CheckboxStates,
    new_directory_name: &str,
    custom_file_name: &str,
    file_name_component_order: &Vec<String>,
    date_type_selected: Option<DateType>,
    index_position: Option<IndexPosition>,
    rename: bool,
) -> std::io::Result<()> {
    for (key, file) in files_selected {
        if let Some(file_name) = key.to_str() {
            let splitted: Vec<_> = file_name.split(".").collect();
            if let Some(file_type) = splitted.last() {
                let lower_case_file_type = file_type.to_lowercase();
                if let Some(file_type_dir) =
                    file_type_directories.get_mut(&OsString::from(&lower_case_file_type))
                {
                    if rename {
                        let mut renamed_file_name = String::new();
                        let file_count = file_type_dir.get_file_count();
                        rename_file_name(
                            &mut renamed_file_name,
                            checkbox_states,
                            new_directory_name,
                            custom_file_name,
                            file_count,
                            file_name_component_order,
                            file_name,
                            &file,
                            date_type_selected,
                            index_position,
                        );
                        file_type_dir
                            .file_aready_exists_in_directory(&OsString::from(&renamed_file_name))?;
                        file_type_dir.insert_file(OsString::from(renamed_file_name), file);
                    } else {
                        file_type_dir.file_aready_exists_in_directory(&key)?;
                        file_type_dir.insert_file(key, file);
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn sort_files_by_date(
    files_selected: BTreeMap<OsString, File>,
    file_date_directories: &mut BTreeMap<OsString, Directory>,
    checkbox_states: &CheckboxStates,
    new_directory_name: &str,
    custom_file_name: &str,
    file_name_component_order: &Vec<String>,
    date_type_selected: DateType,
    index_position: Option<IndexPosition>,
) -> std::io::Result<()> {
    for (key, file) in files_selected {
        if let Some(file_name) = key.to_str() {
            if let Some(metadata) = file.get_metadata() {
                if let Some(formatted) = metadata.get_formated_date(date_type_selected) {
                    if let Some(dir) = file_date_directories.get_mut(&OsString::from(formatted)) {
                        let mut renamed_file_name = String::new();
                        let file_count = dir.get_file_count();
                        rename_file_name(
                            &mut renamed_file_name,
                            checkbox_states,
                            new_directory_name,
                            custom_file_name,
                            file_count,
                            file_name_component_order,
                            file_name,
                            &file,
                            Some(date_type_selected),
                            index_position,
                        );
                        dir.file_aready_exists_in_directory(&OsString::from(&renamed_file_name))?;
                        dir.insert_file(OsString::from(renamed_file_name), file);
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn rename_file_name(
    renamed_file_name: &mut String,
    checkbox_states: &CheckboxStates,
    new_directory_name: &str,
    custom_file_name: &str,
    file_count: usize,
    file_name_component_order: &Vec<String>,
    file_name: &str,
    file: &File,
    date_type_selected: Option<DateType>,
    index_position: Option<IndexPosition>,
) {
    let FilenameComponents {
        mut date,
        mut directory_name,
        mut custom_name,
        mut original_name,
        mut file_type,
    } = FilenameComponents::new();
    if checkbox_states.insert_directory_name_to_file_name {
        directory_name.push_str(new_directory_name);
    }
    if let Some(date_type) = date_type_selected {
        if checkbox_states.insert_date_to_file_name {
            if let Some(metadata) = file.get_metadata() {
                if let Some(formatted) = metadata.get_formated_date(date_type) {
                    date.push_str(formatted.as_str());
                }
            }
        }
    }

    if !checkbox_states.remove_original_file_name {
        original_name = get_file_name_without_file_type(file_name);
    }

    if let Some(index_position) = index_position {
        if checkbox_states.add_custom_name {
            let mut file_name_index = String::new();
            let file_count_str = (file_count + 1).to_string();

            match index_position {
                IndexPosition::Before => {
                    file_name_index.push('0');
                    file_name_index.push_str(&file_count_str);
                    file_name_index.push('_');
                    custom_name.push_str(&file_name_index);
                    custom_name.push_str(custom_file_name);
                }
                IndexPosition::After => {
                    file_name_index.push('_');
                    file_name_index.push('0');
                    file_name_index.push_str(&file_count_str);
                    custom_name.push_str(custom_file_name);
                    custom_name.push_str(&file_name_index);
                }
            }
        }
    }

    if let Some(file_type_ref) = get_file_type_from_file_name(file_name) {
        file_type.push('.');
        file_type.push_str(file_type_ref);
    }

    if checkbox_states.remove_uppercase {
        custom_name = custom_name.as_str().to_lowercase();
        date = date.as_str().to_lowercase();
        directory_name = directory_name.as_str().to_lowercase();
        original_name = original_name.as_str().to_lowercase();
        file_type = file_type.as_str().to_lowercase();
    }

    if checkbox_states.use_only_ascii {
        if !custom_name.is_ascii() {
            custom_name = replace_non_ascii(custom_name.clone());
        }
        if !date.is_ascii() {
            date = replace_non_ascii(date.clone());
        }

        if !directory_name.is_ascii() {
            directory_name = replace_non_ascii(directory_name.clone());
        }

        if !original_name.is_ascii() {
            original_name = replace_non_ascii(original_name.clone());
        }
    }
    if let Some(last) = file_name_component_order.last() {
        for component in file_name_component_order {
            if *component == String::from(filename_components::DATE) {
                renamed_file_name.push_str(date.as_str());
            } else if *component == String::from(filename_components::CUSTOM_FILE_NAME) {
                renamed_file_name.push_str(custom_name.as_str());
            } else if *component == String::from(filename_components::DIRECTORY_NAME) {
                renamed_file_name.push_str(directory_name.as_str());
            } else if *component == String::from(filename_components::ORIGINAL_FILENAME) {
                renamed_file_name.push_str(original_name.as_str());
            }
            if component != last {
                renamed_file_name.push('_');
            }
        }
        renamed_file_name.push_str(file_type.as_str());
    }
}

pub fn get_file_type_from_file_name(file_name: &str) -> Option<&str> {
    let splitted: Vec<_> = file_name.split(".").collect();
    if let Some(file_type) = splitted.iter().last() {
        return Some(*file_type);
    }
    None
}

pub fn get_file_name_without_file_type(file_name: &str) -> String {
    let mut splitted: Vec<_> = file_name.split(".").collect();
    if splitted.len() > 1 {
        splitted.pop();
    }

    splitted.concat()
}

fn replace_non_ascii(text: String) -> String {
    let mut replaced = String::new();
    for character in text.chars() {
        let mut changed_character = character;
        if character == 'ä' {
            changed_character = 'a';
        }
        if character == 'ö' {
            changed_character = 'o';
        }
        if !changed_character.is_ascii() {
            continue;
        }

        replaced.push(changed_character);
    }
    replaced
}

pub fn is_directory_name_unique(
    new_directory_name: &str,
    directories: &BTreeMap<OsString, Directory>,
) -> bool {
    for key in directories.keys() {
        if OsString::from(new_directory_name) == *key {
            return false;
        }
    }
    true
}

pub fn get_file_types(files_selected: &BTreeMap<OsString, File>) -> BTreeMap<OsString, Directory> {
    let mut file_types: BTreeMap<OsString, Directory> = BTreeMap::new();
    for key in files_selected.keys() {
        if let Some(file_name) = key.to_str() {
            let file_name = String::from(file_name);
            let splitted: Vec<_> = file_name.split(".").collect();
            if let Some(file_type) = splitted.last() {
                let lower_case_file_type = file_type.to_lowercase();
                file_types.insert(OsString::from(&lower_case_file_type), Directory::new(None));
            }
        }
    }
    file_types
}

pub fn get_file_dates(
    files_selected: &BTreeMap<OsString, File>,
    date_type: DateType,
) -> BTreeMap<OsString, Directory> {
    let mut file_dates: BTreeMap<OsString, Directory> = BTreeMap::new();
    for (_key, file) in files_selected {
        if let Some(metadata) = file.get_metadata() {
            if let Some(formatted) = metadata.get_formated_date(date_type) {
                file_dates.insert(OsString::from(&formatted), Directory::new(None));
            }
        }
    }
    file_dates
}
