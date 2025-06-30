use crate::app::filename_components;
use crate::directory::Directory;
use crate::file::File;
use crate::layouts::{CheckboxStates, IndexPosition};
use crate::metadata::DateType;
use std::collections::BTreeMap;
use std::ffi::{OsString, OsStr};
use std::io::ErrorKind;
use std::path::PathBuf;

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
    path_to_selected_directory: &PathBuf,
    files_organized: &mut BTreeMap<OsString, File>,
    new_directory_name: String,
    selected_directory: &mut Directory,
    data: OrganizingData,
) -> std::io::Result<()> {
    // If both organize_by_file_type and date are checked
    if data.checkbox_states.organize_by_filetype && data.checkbox_states.organize_by_date {
        let mut new_directory = Directory::new(None);
        match organize_files_by_file_type_and_date(path_to_selected_directory, files_organized, &mut new_directory, data) {
            Ok(_) => {
                selected_directory.insert_directory(new_directory, &new_directory_name);
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_filetype {
        let mut new_directory = Directory::new(None);
        match organize_files_by_file_type(path_to_selected_directory, files_organized, &mut new_directory, data) {
            Ok(_) => {
                selected_directory.insert_directory(new_directory, &new_directory_name);
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_date {
        // If only organize_by_date is checked
        let mut new_directory = Directory::new(None);
        let mut path_to_named_directory = PathBuf::from(&path_to_selected_directory);
        path_to_named_directory.push(&new_directory_name);
        match organize_files_by_date(path_to_selected_directory, files_organized, &mut new_directory, data) {
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
                    for (file_name, mut file) in renamed_files {
                        let mut destination_path = PathBuf::from(path_to_selected_directory);
                        destination_path.push(&new_directory_name);
                        destination_path.push(&file_name);
                        file.set_destination_path(destination_path);
                        files_organized.insert(OsString::from(&file_name), file.clone());
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
        for (filename, mut file) in data.files_selected {
            let mut destination_path = PathBuf::from(path_to_selected_directory);
            destination_path.push(&new_directory_name);
            destination_path.push(&filename);
            file.set_destination_path(destination_path);
            files_organized.insert(OsString::from(&filename), file.clone());

            new_directory.insert_file(filename, file);
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
    path_to_selected_directory: &PathBuf,
    files_organized: &mut BTreeMap<OsString, File>,
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
        match organize_files_by_file_type_and_date(path_to_selected_directory, files_organized, selected_directory, data) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_filetype {
        if let Err(error) = selected_directory.contains_unique_files_recursive(&data.files_selected)
        {
            return Err(error);
        }
        match organize_files_by_file_type(path_to_selected_directory, files_organized, selected_directory, data) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    } else if data.checkbox_states.organize_by_date {
        // Check files before inserting
        match organize_files_by_date(path_to_selected_directory, files_organized, selected_directory, data) {
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
                    for (filename, mut file) in renamed_files {
                        let mut destination_path = PathBuf::from(path_to_selected_directory);
                        destination_path.push(directory_name);
                        destination_path.push(&filename);
                        file.set_destination_path(destination_path);
                        files_organized.insert(OsString::from(&filename), file.clone());
                        selected_directory.insert_file(filename, file);
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

        for (filename, mut file) in data.files_selected {
            let mut destination_path = PathBuf::from(path_to_selected_directory);
            destination_path.push(directory_name);
            destination_path.push(&filename);
            file.set_destination_path(destination_path);
            files_organized.insert(OsString::from(&filename), file.clone());
            selected_directory.insert_file(filename, file);
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
    path_to_selected_directory: &PathBuf,
    files_organized: &mut BTreeMap<OsString, File>,
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
                SortData::build(
                    path_to_selected_directory,
                    files_organized,
                    data.files_selected,
                    file_type_dirs,
                    &data.checkbox_states,
                    data.new_directory_name,
                    data.custom_file_name,
                    data.file_name_component_order,
                    data.date_type,
                    data.index_position,
                    false,
                    false
                ))?;

            // After this organize by date as well
            for (filetype_dir_name, dir) in file_type_dirs {
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
                    let mut path_to_filetype_directory = PathBuf::from(&path_to_selected_directory);
                    path_to_filetype_directory.push(data.new_directory_name);
                    path_to_filetype_directory.push(&filetype_dir_name);
                    organize_files_by_date(&path_to_filetype_directory, files_organized, dir, new_data)?;
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
    path_to_selected_directory: &PathBuf,
    files_organized: &mut BTreeMap<OsString, File>,
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
            SortData::build(
                path_to_selected_directory,
                files_organized,
                data.files_selected,
                file_type_dirs,
                &data.checkbox_states,
                data.new_directory_name,
                data.custom_file_name,
                data.file_name_component_order,
                data.date_type,
                data.index_position,
                true,
                true,
            ))?;
        return Ok(());
    }
    Err(std::io::Error::new(
        ErrorKind::NotFound,
        "No file type directories found",
    ))
}

fn organize_files_by_date(
    path_to_selected_directory: &PathBuf,
    files_organized: &mut BTreeMap<OsString, File>,
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
                SortData::build(
                    path_to_selected_directory,
                    files_organized,
                    data.files_selected,
                    file_date_dirs,
                    &data.checkbox_states,
                    data.new_directory_name,
                    data.custom_file_name,
                    data.file_name_component_order,
                    Some(date_type),
                    data.index_position,
                    true,
                    true
                ))?;
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
                RenameData::build(
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
                )
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

pub struct SortData<'a> {
    path_to_selected_directory: &'a PathBuf,
    files_organized: &'a mut BTreeMap<OsString, File>,
    files_selected: BTreeMap<OsString, File>,
    file_type_directories: &'a mut BTreeMap<OsString, Directory>,
    checkbox_states: &'a CheckboxStates,
    new_directory_name: &'a str,
    custom_file_name: &'a str,
    file_name_component_order: &'a Vec<String>,
    date_type_selected: Option<DateType>,
    index_position: Option<IndexPosition>,
    rename: bool,
    mark_as_organized: bool
}
impl<'a> SortData<'a> {
    pub fn build(
        path_to_selected_directory: &'a PathBuf,
        files_organized: &'a mut BTreeMap<OsString, File>,
        files_selected: BTreeMap<OsString, File>,
        file_type_directories: &'a mut BTreeMap<OsString, Directory>,
        checkbox_states: &'a CheckboxStates,
        new_directory_name: &'a str,
        custom_file_name: &'a str,
        file_name_component_order: &'a Vec<String>,
        date_type_selected: Option<DateType>,
        index_position: Option<IndexPosition>,
        rename: bool,
        mark_as_organized: bool
    ) -> Self {
        Self {
            path_to_selected_directory,
            files_organized,
            files_selected,
            file_type_directories,
            checkbox_states,
            new_directory_name,
            custom_file_name,
            file_name_component_order,
            date_type_selected,
            index_position,
            rename,
            mark_as_organized
        }
    }
}
pub fn sort_files_by_file_type(
    mut sort_data: SortData
) -> std::io::Result<()> {
    for (key, file) in sort_data.files_selected {
        let file_name = convert_os_str_to_str(&key)?;
        let mut renamed_file_name = String::new();
        let file_count = get_file_count_from_dir(file_name, sort_data.file_type_directories);
        if sort_data.rename {
            rename_file_name(
                RenameData::build(
                    &mut renamed_file_name,
                    sort_data.checkbox_states,
                    sort_data.new_directory_name,
                    sort_data.custom_file_name,
                    file_count,
                    sort_data.file_name_component_order,
                    file_name,
                    &file,
                    sort_data.date_type_selected,
                    sort_data.index_position
                )
            );
        } else {
            renamed_file_name = String::from(file_name);
        }
        insert_file_to_file_type_dir(
            &renamed_file_name,
            sort_data.file_type_directories,
            sort_data.path_to_selected_directory,
            sort_data.new_directory_name,
            key,
            file,
            &mut sort_data.files_organized
        )?;
    }
    Ok(())
}

pub fn sort_files_by_date(
    mut sort_data: SortData
) -> std::io::Result<()> {
    if let Some(date_type_selected) = sort_data.date_type_selected {
        for (key, file) in sort_data.files_selected {
            let file_name = convert_os_str_to_str(&key)?;
            let formatted_date = get_formatted_date_from_file(&file, &date_type_selected)?;
            if let Some(date_dir) = sort_data.file_type_directories.get_mut(&OsString::from(&formatted_date)) {
                let mut renamed_file_name = String::new();
                let file_count = date_dir.get_file_count();
                rename_file_name(
                    RenameData::build(
                        &mut renamed_file_name,
                        sort_data.checkbox_states,
                        sort_data.new_directory_name,
                        sort_data.custom_file_name,
                        file_count,
                        sort_data.file_name_component_order,
                        file_name,
                        &file,
                        Some(date_type_selected),
                        sort_data.index_position,
                    )
                );
                insert_file_to_date_dir(
                    date_dir,
                    renamed_file_name,
                    sort_data.mark_as_organized,
                    sort_data.path_to_selected_directory,
                    formatted_date,
                    file,
                    &mut sort_data.files_organized
                )?;
            }
        }
        return Ok(());
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No date type specified")) 
}

pub struct RenameData<'a> {
    renamed_file_name: &'a mut String,
    checkbox_states: &'a CheckboxStates,
    new_directory_name: &'a str,
    custom_file_name: &'a str,
    file_count: usize,
    file_name_component_order: &'a Vec<String>,
    file_name: &'a str,
    file: &'a File,
    date_type_selected: Option<DateType>,
    index_position: Option<IndexPosition>,
}

impl<'a> RenameData<'a> {
    pub fn build(
        renamed_file_name: &'a mut String,
        checkbox_states: &'a CheckboxStates,
        new_directory_name: &'a str,
        custom_file_name: &'a str,
        file_count: usize,
        file_name_component_order: &'a Vec<String>,
        file_name: &'a str,
        file: &'a File,
        date_type_selected: Option<DateType>,
        index_position: Option<IndexPosition>
    ) -> Self {
        Self {
            renamed_file_name,
            checkbox_states,
            new_directory_name,
            custom_file_name,
            file_count,
            file_name_component_order,
            file_name,
            file,
            date_type_selected,
            index_position,
        }
    }
}



pub fn rename_file_name(
    rename_data: RenameData,
) {
    let FilenameComponents {
        mut date,
        mut directory_name,
        mut custom_name,
        mut original_name,
        mut file_type,
    } = FilenameComponents::new();
    if rename_data.checkbox_states.insert_directory_name_to_file_name {
        directory_name.push_str(rename_data.new_directory_name);
    }
    if let Some(date_type) = rename_data.date_type_selected {
        if rename_data.checkbox_states.insert_date_to_file_name {
            if let Some(metadata) = rename_data.file.get_metadata() {
                if let Some(formatted) = metadata.get_formatted_date(date_type) {
                    date.push_str(formatted.as_str());
                }
            }
        }
    }

    if !rename_data.checkbox_states.remove_original_file_name {
        original_name = get_file_name_without_file_type(rename_data.file_name);
    }

    if let Some(index_position) = rename_data.index_position {
        if rename_data.checkbox_states.add_custom_name {
            let mut file_name_index = String::new();
            let file_count_str = (rename_data.file_count + 1).to_string();

            match index_position {
                IndexPosition::Before => {
                    file_name_index.push('0');
                    file_name_index.push_str(&file_count_str);
                    file_name_index.push('_');
                    custom_name.push_str(&file_name_index);
                    custom_name.push_str(rename_data.custom_file_name);
                }
                IndexPosition::After => {
                    file_name_index.push('_');
                    file_name_index.push('0');
                    file_name_index.push_str(&file_count_str);
                    custom_name.push_str(rename_data.custom_file_name);
                    custom_name.push_str(&file_name_index);
                }
            }
        }
    }

    if let Some(file_type_ref) = get_file_type_from_file_name(rename_data.file_name) {
        file_type.push('.');
        file_type.push_str(&file_type_ref);
    }

    if rename_data.checkbox_states.remove_uppercase {
        custom_name = custom_name.as_str().to_lowercase();
        date = date.as_str().to_lowercase();
        directory_name = directory_name.as_str().to_lowercase();
        original_name = original_name.as_str().to_lowercase();
        file_type = file_type.as_str().to_lowercase();
    }

    if rename_data.checkbox_states.use_only_ascii {
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
    if let Some(last) = rename_data.file_name_component_order.last() {
        for component in rename_data.file_name_component_order {
            if *component == String::from(filename_components::DATE) {
                rename_data.renamed_file_name.push_str(date.as_str());
            } else if *component == String::from(filename_components::CUSTOM_FILE_NAME) {
                rename_data.renamed_file_name.push_str(custom_name.as_str());
            } else if *component == String::from(filename_components::DIRECTORY_NAME) {
                rename_data.renamed_file_name.push_str(directory_name.as_str());
            } else if *component == String::from(filename_components::ORIGINAL_FILENAME) {
                rename_data.renamed_file_name.push_str(original_name.as_str());
            }
            if component != last {
                rename_data.renamed_file_name.push('_');
            }
        }
        rename_data.renamed_file_name.push_str(file_type.as_str());
    }
}

pub fn get_file_type_from_file_name(file_name: &str) -> Option<String> {
    if !file_name.contains(".") {
        return None;
    }
    let splitted: Vec<_> = file_name.split(".").collect();
    if let Some(file_type) = splitted.iter().last() {
        let lower_case_file_type: String = file_type.to_lowercase();
        return Some(lower_case_file_type);
    }
    None
}

pub fn get_file_name_without_file_type(file_name: &str) -> String {
    let mut splitted: Vec<_> = file_name.split(".").collect();
    if splitted.len() > 1 {
        splitted.pop();
    } else {
        return String::from(file_name);
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
            if let Some(formatted) = metadata.get_formatted_date(date_type) {
                file_dates.insert(OsString::from(&formatted), Directory::new(None));
            }
        }
    }
    file_dates
}

fn build_destination_path(path_components: Vec<&str>) -> PathBuf {
    let mut path = PathBuf::new();
    for path_component in path_components {
        if path_component.is_empty() {
            continue;
        }
        path.push(path_component);
    }
    path
}

fn create_destination_path(path_to_selected_directory: &PathBuf, path_components: Vec<&str>, file: &mut File) {
    let path_in_rule_directory = build_destination_path(path_components);
            
    let mut destination_path = PathBuf::from(path_to_selected_directory);
    destination_path.push(path_in_rule_directory);
    file.set_destination_path(destination_path);
}
fn get_file_count_from_dir(file_name: &str, file_type_directories: &BTreeMap<OsString, Directory>) -> usize {
    let mut file_count = 0;
    if let Some(file_type) = get_file_type_from_file_name(file_name) { 
        if let Some(file_type_dir) = file_type_directories.get(&OsString::from(file_type)) {
            file_count = file_type_dir.get_file_count();
        }
    } else {
        if let Some(other_dir) = file_type_directories.get(&OsString::from("other")) {
            file_count = other_dir.get_file_count();
        }
    }
    file_count
}

fn convert_os_str_to_str(key: &OsStr) -> std::io::Result<&str> {
   if let Some(key) = key.to_str() {
        return Ok(key);
   } 
   Err(std::io::Error::new(std::io::ErrorKind::Other, "Could not parse &OsStr to &str"))
}

fn insert_file_to_file_type_dir(
    file_name: &str,
    file_type_directories: &mut BTreeMap<OsString, Directory>,
    path_to_selected_directory: &PathBuf,
    new_directory_name: &str,
    key: OsString,
    mut file: File,
    files_organized: &mut BTreeMap<OsString, File>
) -> std::io::Result<()> {
    let file_type_dir = get_file_type_dir(file_name, file_type_directories)?;
    file_type_dir.file_aready_exists_in_directory(&OsString::from(file_name))?;
    let mut file_type = String::new();
    if let Some(file_type_from_file_name) = get_file_type_from_file_name(file_name) {
        file_type.push_str(&file_type_from_file_name);
    }
    create_destination_path(path_to_selected_directory, vec![
       new_directory_name,
       &file_type,
       file_name
    ], &mut file);
    files_organized.insert(key.clone(), file.clone());
    file_type_dir.insert_file(OsString::from(file_name), file);
    Ok(())
}

fn insert_file_to_date_dir(
    dir: &mut Directory,
    renamed_file_name: String,
    mark_as_organized: bool,
    path_to_selected_directory: &PathBuf,
    formatted_date: String,
    mut file: File,
    files_organized: &mut BTreeMap<OsString, File>
) -> std::io::Result<()> {
    dir.file_aready_exists_in_directory(&OsString::from(&renamed_file_name))?;
    if mark_as_organized {
        create_destination_path(path_to_selected_directory, vec![
            &formatted_date,
            &renamed_file_name,
        ], &mut file);
        
        files_organized.insert(OsString::from(&renamed_file_name), file.clone());
    }
    dir.insert_file(OsString::from(renamed_file_name), file);
    Ok(())
}

fn get_file_type_dir<'a>(
    file_name: &'a str,
    file_type_directories: &'a mut BTreeMap<OsString, Directory>
) -> std::io::Result<&'a mut Directory> {
   if let Some(file_type) = get_file_type_from_file_name(file_name) {
        if let Some(file_type_dir) = file_type_directories.get_mut(&OsString::from(file_type)) {
            return Ok(file_type_dir);
        } 
   } 
   Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File type directory not found"))
}

fn get_formatted_date_from_file(file: &File, date_type_selected: &DateType) -> std::io::Result<String> {
    if let Some(metadata) = file.get_metadata() {
        if let Some(formatted_date) = metadata.get_formatted_date(*date_type_selected) {
            return Ok(formatted_date);
        }
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Could not get formatted date from metadata."));    
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Metadata not found."))
}
