use crate::app::{FilenameComponents, ReplacableSelection};
use crate::app_util;
use crate::directory::Directory;
use crate::file::File;
use crate::layouts::{CheckboxStates, IndexPosition, ReplaceWith, Replaceable};
use crate::metadata::DateType;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct OrganizingData<'a> {
    files_selected: BTreeMap<OsString, File>,
    checkbox_states: &'a CheckboxStates,
    replaceables: &'a Vec<ReplacableSelection>,
    directory_name: &'a str,
    custom_file_name: &'a str,
    file_name_component_order: &'a Vec<FilenameComponents>,
    date_type: Option<DateType>,
    index_position: Option<IndexPosition>,
}

impl<'a> OrganizingData<'a> {
    pub fn new(
        files_selected: BTreeMap<OsString, File>,
        checkbox_states: &'a CheckboxStates,
        replaceables: &'a Vec<ReplacableSelection>,
        directory_name: &'a str,
        custom_file_name: &'a str,
        file_name_component_order: &'a Vec<FilenameComponents>,
        date_type: Option<DateType>,
        index_position: Option<IndexPosition>,
    ) -> Self {
        Self {
            files_selected,
            checkbox_states,
            replaceables,
            directory_name,
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
    let mut new_directory = Directory::new(None);
    if data.checkbox_states.organize_by_filetype && data.checkbox_states.organize_by_date {
        organize_files_by_file_type_and_date(
            path_to_selected_directory,
            files_organized,
            &mut new_directory,
            data,
        )?;
        selected_directory.insert_directory(new_directory, &new_directory_name);
    } else if data.checkbox_states.organize_by_filetype {
        organize_files_by_file_type(
            path_to_selected_directory,
            files_organized,
            &mut new_directory,
            data,
        )?;
        selected_directory.insert_directory(new_directory, &new_directory_name);
    } else if data.checkbox_states.organize_by_date {
        let mut path_to_named_directory = PathBuf::from(&path_to_selected_directory);
        path_to_named_directory.push(&new_directory_name);
        organize_files_by_date(
            path_to_selected_directory,
            files_organized,
            &mut new_directory,
            data,
        )?;
        selected_directory.insert_directory(new_directory, &new_directory_name);
    } else if app_util::just_rename_checked(&data.checkbox_states) {
        rename_files(
            data,
            &mut new_directory,
            files_organized,
            path_to_selected_directory,
        )?;
        selected_directory.insert_directory(new_directory, &new_directory_name);
    } else {
        for (key, mut file) in data.files_selected {
            let file_name = app_util::convert_os_str_to_str(&key)?;
            create_destination_path(
                path_to_selected_directory,
                vec![&new_directory_name, &file_name],
                &mut file,
            );
            files_organized.insert(OsString::from(&file_name), file.clone());
            new_directory.insert_file(key, file);
        }
        selected_directory.insert_directory(new_directory, &new_directory_name);
    }
    Ok(())
}

pub fn move_files_to_organized_directory(
    path_to_selected_directory: &PathBuf,
    files_organized: &mut BTreeMap<OsString, File>,
    selected_directory: &mut Directory,
    data: OrganizingData,
) -> std::io::Result<()> {
    if data.checkbox_states.organize_by_filetype && data.checkbox_states.organize_by_date {
        organize_files_by_file_type_and_date(
            path_to_selected_directory,
            files_organized,
            selected_directory,
            data,
        )?;
    } else if data.checkbox_states.organize_by_filetype {
        organize_files_by_file_type(
            path_to_selected_directory,
            files_organized,
            selected_directory,
            data,
        )?;
    } else if data.checkbox_states.organize_by_date {
        organize_files_by_date(
            path_to_selected_directory,
            files_organized,
            selected_directory,
            data,
        )?;
    } else if app_util::just_rename_checked(&data.checkbox_states) {
        rename_files(
            data,
            selected_directory,
            files_organized,
            path_to_selected_directory,
        )?;
    } else {
        selected_directory.contains_unique_files(&data.files_selected)?;
        for (key, mut file) in data.files_selected {
            let file_name = app_util::convert_os_str_to_str(&key)?;
            create_destination_path(
                path_to_selected_directory,
                vec![&data.directory_name, file_name],
                &mut file,
            );
            files_organized.insert(OsString::from(&file_name), file.clone());
            selected_directory.insert_file(key, file);
        }
    }
    Ok(())
}

fn organize_files_by_file_type_and_date(
    path_to_selected_directory: &PathBuf,
    files_organized: &mut BTreeMap<OsString, File>,
    selected_directory: &mut Directory,
    data: OrganizingData,
) -> std::io::Result<()> {
    let date_type_selected = app_util::get_date_type(data.date_type)?;
    let mut file_type_dirs = get_file_types(&data.files_selected);
    selected_directory.filter_duplicate_directories(&mut file_type_dirs);
    selected_directory.insert_new_directories(file_type_dirs);

    if let Some(file_type_dirs) = selected_directory.get_mut_directories() {
        sort_files_by_file_type(SortData::build(
            path_to_selected_directory,
            files_organized,
            data.files_selected,
            file_type_dirs,
            &data.checkbox_states,
            data.replaceables,
            data.directory_name,
            data.custom_file_name,
            data.file_name_component_order,
            data.date_type,
            data.index_position,
            false,
            false,
        ))?;

        // After this organize by date as well
        for (filetype_dir_name, dir) in file_type_dirs {
            if let Some(files_by_filetype) = dir.get_mut_files().take() {
                let new_data = OrganizingData::new(
                    files_by_filetype,
                    &data.checkbox_states,
                    data.replaceables,
                    data.directory_name,
                    data.custom_file_name,
                    data.file_name_component_order,
                    Some(date_type_selected),
                    data.index_position.clone(),
                );
                let mut path_to_filetype_directory = PathBuf::from(&path_to_selected_directory);
                path_to_filetype_directory.push(data.directory_name);
                path_to_filetype_directory.push(&filetype_dir_name);
                organize_files_by_date(
                    &path_to_filetype_directory,
                    files_organized,
                    dir,
                    new_data,
                )?;
            }
        }
    }
    return Ok(());
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
    selected_directory.insert_new_directories(file_type_dirs);

    if let Some(file_type_dirs) = selected_directory.get_mut_directories() {
        sort_files_by_file_type(SortData::build(
            path_to_selected_directory,
            files_organized,
            data.files_selected,
            file_type_dirs,
            &data.checkbox_states,
            data.replaceables,
            data.directory_name,
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
    let date_type = app_util::get_date_type(data.date_type)?;
    let mut file_date_dirs = create_file_dates(&data.files_selected, date_type);
    selected_directory.filter_duplicate_directories(&mut file_date_dirs);
    selected_directory.insert_new_directories(file_date_dirs);
    if let Some(file_date_dirs) = selected_directory.get_mut_directories() {
        sort_files_by_date(SortData::build(
            path_to_selected_directory,
            files_organized,
            data.files_selected,
            file_date_dirs,
            &data.checkbox_states,
            data.replaceables,
            data.directory_name,
            data.custom_file_name,
            data.file_name_component_order,
            Some(date_type),
            data.index_position,
            true,
            true,
        ))?;
    }
    Ok(())
}

fn rename_files(
    data: OrganizingData,
    directory: &mut Directory,
    files_organized: &mut BTreeMap<OsString, File>,
    path_to_selected_directory: &PathBuf,
) -> std::io::Result<()> {
    if let None = data.date_type {
        if data.checkbox_states.insert_date_to_file_name {
            return Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Date type not specified",
            ));
        }
    }
    for (key, file) in data.files_selected {
        if let Some(file_name) = key.to_str() {
            let mut renamed_file_name = String::new();
            let file_count = directory.get_file_count();
            rename_file_name(RenameData::build(
                &mut renamed_file_name,
                &data.checkbox_states,
                data.replaceables,
                data.directory_name,
                data.custom_file_name,
                file_count,
                data.file_name_component_order,
                file_name,
                &file,
                data.date_type,
                data.index_position,
            ));
            insert_renamed_files_to_dir(
                &renamed_file_name,
                file,
                path_to_selected_directory,
                directory,
                data.directory_name,
                files_organized,
            )?;
        }
    }

    Ok(())
}

fn insert_renamed_files_to_dir(
    renamed_file_name: &str,
    mut file: File,
    path_to_selected_directory: &PathBuf,
    directory: &mut Directory,
    directory_name: &str,
    files_organized: &mut BTreeMap<OsString, File>,
) -> std::io::Result<()> {
    directory.file_already_exists_in_directory(&OsString::from(renamed_file_name))?;
    create_destination_path(
        path_to_selected_directory,
        vec![directory_name, renamed_file_name],
        &mut file,
    );
    files_organized.insert(OsString::from(&renamed_file_name), file.clone());
    directory.insert_file(OsString::from(renamed_file_name), file);
    Ok(())
}

#[derive(Debug)]
pub struct SortData<'a> {
    path_to_selected_directory: &'a PathBuf,
    files_organized: &'a mut BTreeMap<OsString, File>,
    files_selected: BTreeMap<OsString, File>,
    file_type_directories: &'a mut BTreeMap<OsString, Directory>,
    checkbox_states: &'a CheckboxStates,
    replaceables: &'a Vec<ReplacableSelection>,
    new_directory_name: &'a str,
    custom_file_name: &'a str,
    file_name_component_order: &'a Vec<FilenameComponents>,
    date_type_selected: Option<DateType>,
    index_position: Option<IndexPosition>,
    rename: bool,
    mark_as_organized: bool,
}
impl<'a> SortData<'a> {
    pub fn build(
        path_to_selected_directory: &'a PathBuf,
        files_organized: &'a mut BTreeMap<OsString, File>,
        files_selected: BTreeMap<OsString, File>,
        file_type_directories: &'a mut BTreeMap<OsString, Directory>,
        checkbox_states: &'a CheckboxStates,
        replaceables: &'a Vec<ReplacableSelection>,
        new_directory_name: &'a str,
        custom_file_name: &'a str,
        file_name_component_order: &'a Vec<FilenameComponents>,
        date_type_selected: Option<DateType>,
        index_position: Option<IndexPosition>,
        rename: bool,
        mark_as_organized: bool,
    ) -> Self {
        Self {
            path_to_selected_directory,
            files_organized,
            files_selected,
            file_type_directories,
            checkbox_states,
            replaceables,
            new_directory_name,
            custom_file_name,
            file_name_component_order,
            date_type_selected,
            index_position,
            rename,
            mark_as_organized,
        }
    }
}
pub fn sort_files_by_file_type(mut sort_data: SortData) -> std::io::Result<()> {
    for (key, file) in sort_data.files_selected {
        let file_name = app_util::convert_os_str_to_str(&key)?;
        let mut renamed_file_name = String::new();
        let file_count = get_file_count_from_dir(file_name, sort_data.file_type_directories);
        if sort_data.rename {
            rename_file_name(RenameData::build(
                &mut renamed_file_name,
                sort_data.checkbox_states,
                sort_data.replaceables,
                sort_data.new_directory_name,
                sort_data.custom_file_name,
                file_count,
                sort_data.file_name_component_order,
                file_name,
                &file,
                sort_data.date_type_selected,
                sort_data.index_position,
            ));
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
            &mut sort_data.files_organized,
            sort_data.mark_as_organized,
        )?;
    }
    Ok(())
}

pub fn sort_files_by_date(mut sort_data: SortData) -> std::io::Result<()> {
    let date_type = app_util::get_date_type(sort_data.date_type_selected)?;
    for (key, file) in sort_data.files_selected {
        let file_name = app_util::convert_os_str_to_str(&key)?;
        let formatted_date = get_formatted_date_from_file(&file, &date_type)?;
        if let Some(date_dir) = sort_data
            .file_type_directories
            .get_mut(&OsString::from(&formatted_date))
        {
            let mut renamed_file_name = String::new();
            let file_count = date_dir.get_file_count();
            rename_file_name(RenameData::build(
                &mut renamed_file_name,
                sort_data.checkbox_states,
                sort_data.replaceables,
                sort_data.new_directory_name,
                sort_data.custom_file_name,
                file_count,
                sort_data.file_name_component_order,
                file_name,
                &file,
                Some(date_type),
                sort_data.index_position,
            ));
            let mut directory_name = Some(sort_data.new_directory_name);
            if sort_data.checkbox_states.organize_by_filetype
                && sort_data.checkbox_states.organize_by_date
            {
                directory_name = None;
            }
            insert_file_to_date_dir(
                directory_name,
                date_dir,
                renamed_file_name,
                sort_data.mark_as_organized,
                sort_data.path_to_selected_directory,
                formatted_date,
                file,
                &mut sort_data.files_organized,
            )?;
        }
    }
    return Ok(());
}

#[derive(Debug)]
pub struct RenameData<'a> {
    renamed_file_name: &'a mut String,
    checkbox_states: &'a CheckboxStates,
    replaceables: &'a Vec<ReplacableSelection>,
    new_directory_name: &'a str,
    custom_file_name: &'a str,
    file_count: usize,
    file_name_component_order: &'a Vec<FilenameComponents>,
    file_name: &'a str,
    file: &'a File,
    date_type_selected: Option<DateType>,
    index_position: Option<IndexPosition>,
}

impl<'a> RenameData<'a> {
    pub fn build(
        renamed_file_name: &'a mut String,
        checkbox_states: &'a CheckboxStates,
        replaceables: &'a Vec<ReplacableSelection>,
        new_directory_name: &'a str,
        custom_file_name: &'a str,
        file_count: usize,
        file_name_component_order: &'a Vec<FilenameComponents>,
        file_name: &'a str,
        file: &'a File,
        date_type_selected: Option<DateType>,
        index_position: Option<IndexPosition>,
    ) -> Self {
        Self {
            renamed_file_name,
            checkbox_states,
            replaceables,
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

struct FilenameComponentString {
    date: String,
    directory_name: String,
    custom_name: String,
    original_name: String,
    file_type: String,
}

impl FilenameComponentString {
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

pub fn rename_file_name(rename_data: RenameData) {
    let FilenameComponentString {
        mut date,
        mut directory_name,
        mut custom_name,
        mut original_name,
        mut file_type,
    } = FilenameComponentString::new();
    if rename_data
        .checkbox_states
        .insert_directory_name_to_file_name
    {
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

    if rename_data.checkbox_states.convert_uppercase_to_lowercase {
        custom_name = custom_name.as_str().to_lowercase();
        date = date.as_str().to_lowercase();
        directory_name = directory_name.as_str().to_lowercase();
        original_name = original_name.as_str().to_lowercase();
        file_type = file_type.as_str().to_lowercase();
    }

    if rename_data.checkbox_states.replace_character {
        replace_characters_by_rules(
            &mut custom_name,
            &mut directory_name,
            &mut original_name,
            &mut file_type,
            rename_data.replaceables,
        );
    }

    if rename_data.checkbox_states.use_only_ascii {
        if !custom_name.is_ascii() {
            custom_name = replace_non_ascii(custom_name);
        }
        if !date.is_ascii() {
            date = replace_non_ascii(date);
        }

        if !directory_name.is_ascii() {
            directory_name = replace_non_ascii(directory_name);
        }

        if !original_name.is_ascii() {
            original_name = replace_non_ascii(original_name);
        }
    }

    let size = rename_data.file_name_component_order.len();

    for (i, component) in rename_data.file_name_component_order.iter().enumerate() {
        match component {
            FilenameComponents::Date => rename_data.renamed_file_name.push_str(date.as_str()),
            FilenameComponents::DirectoryName => rename_data
                .renamed_file_name
                .push_str(directory_name.as_str()),
            FilenameComponents::CustomFilename => {
                rename_data.renamed_file_name.push_str(custom_name.as_str())
            }
            FilenameComponents::OriginalFilename => rename_data
                .renamed_file_name
                .push_str(original_name.as_str()),
        }
        if i < (size - 1) {
            rename_data.renamed_file_name.push('_');
        }
    }
    rename_data.renamed_file_name.push_str(file_type.as_str());
}

fn replace_characters_by_rules(
    custom_name: &mut String,
    directory_name: &mut String,
    original_name: &mut String,
    file_type: &mut String,
    replaceables: &Vec<ReplacableSelection>,
) {
    for replaceable in replaceables {
        if let Some(replace) = replaceable.get_replaceable_selected() {
            if let Some(replace_with) = replaceable.get_replace_with_selected() {
                replace_character_with(custom_name, replace, replace_with);
                replace_character_with(directory_name, replace, replace_with);
                replace_character_with(original_name, replace, replace_with);
                replace_character_with(file_type, replace, replace_with);
            }
        }
    }
}

pub fn replace_character_with(
    text_component: &mut String,
    replace: Replaceable,
    replace_with: ReplaceWith,
) {
    let replace_character = match replace {
        Replaceable::Dash => "-",
        Replaceable::Space => " ",
        Replaceable::Comma => ",",
    };
    let replace_with_character = match replace_with {
        ReplaceWith::Nothing => "",
        ReplaceWith::Underscore => "_",
    };
    *text_component = text_component
        .as_str()
        .replace(replace_character, replace_with_character);
}

pub fn get_file_type_from_file_name(file_name: &str) -> Option<String> {
    if !file_name.contains(".") || file_name.starts_with(".") || file_name.ends_with(".") {
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

pub fn replace_non_ascii(text: String) -> String {
    let mut replaced = String::new();
    for character in text.chars() {
        let mut changed_character = character;
        if character == 'ä' {
            changed_character = 'a';
        }
        if character == 'Ä' {
            changed_character = 'A';
        }
        if character == 'ö' {
            changed_character = 'o';
        }
        if character == 'Ö' {
            changed_character = 'O';
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
            if !file_name.contains(".") || file_name.starts_with(".") || file_name.ends_with(".") {
                file_types.insert(OsString::from("other"), Directory::new(None));
                continue;
            }
            if let Some(file_type) = splitted.last() {
                let lower_case_file_type = file_type.to_lowercase();
                file_types.insert(OsString::from(&lower_case_file_type), Directory::new(None));
            }
        }
    }
    file_types
}

pub fn create_file_dates(
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

pub fn create_destination_path(
    path_to_selected_directory: &PathBuf,
    path_components: Vec<&str>,
    file: &mut File,
) {
    let path_in_rule_directory = build_destination_path(path_components);

    let mut destination_path = PathBuf::from(path_to_selected_directory);
    destination_path.push(path_in_rule_directory);
    file.set_destination_path(destination_path);
}

fn get_file_count_from_dir(
    file_name: &str,
    file_type_directories: &BTreeMap<OsString, Directory>,
) -> usize {
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

fn insert_file_to_file_type_dir(
    file_name: &str,
    file_type_directories: &mut BTreeMap<OsString, Directory>,
    path_to_selected_directory: &PathBuf,
    new_directory_name: &str,
    key: OsString,
    mut file: File,
    files_organized: &mut BTreeMap<OsString, File>,
    mark_as_organized: bool,
) -> std::io::Result<()> {
    let file_type_dir = get_file_type_dir(file_name, file_type_directories)?;
    file_type_dir.file_already_exists_in_directory(&OsString::from(file_name))?;
    let mut file_type = String::new();
    if let Some(file_type_from_file_name) = get_file_type_from_file_name(file_name) {
        file_type.push_str(&file_type_from_file_name);
    } else {
        file_type.push_str("other");
    }
    if mark_as_organized {
        create_destination_path(
            path_to_selected_directory,
            vec![new_directory_name, &file_type, file_name],
            &mut file,
        );
        files_organized.insert(key.clone(), file.clone());
    }

    file_type_dir.insert_file(OsString::from(file_name), file);
    Ok(())
}

fn insert_file_to_date_dir(
    new_directory_name: Option<&str>,
    dir: &mut Directory,
    renamed_file_name: String,
    mark_as_organized: bool,
    path_to_selected_directory: &PathBuf,
    formatted_date: String,
    mut file: File,
    files_organized: &mut BTreeMap<OsString, File>,
) -> std::io::Result<()> {
    dir.file_already_exists_in_directory(&OsString::from(&renamed_file_name))?;
    if mark_as_organized {
        if let Some(new_directory_name) = new_directory_name {
            create_destination_path(
                path_to_selected_directory,
                vec![new_directory_name, &formatted_date, &renamed_file_name],
                &mut file,
            );
        } else {
            create_destination_path(
                path_to_selected_directory,
                vec![&formatted_date, &renamed_file_name],
                &mut file,
            );
        }
        files_organized.insert(OsString::from(&renamed_file_name), file.clone());
    }
    dir.insert_file(OsString::from(renamed_file_name), file);
    Ok(())
}

fn get_file_type_dir<'a>(
    file_name: &'a str,
    file_type_directories: &'a mut BTreeMap<OsString, Directory>,
) -> std::io::Result<&'a mut Directory> {
    if let Some(file_type) = get_file_type_from_file_name(file_name) {
        if let Some(file_type_dir) = file_type_directories.get_mut(&OsString::from(file_type)) {
            return Ok(file_type_dir);
        }
    } else {
        if let Some(other_dir) = file_type_directories.get_mut(&OsString::from("other")) {
            return Ok(other_dir);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "File type directory not found",
    ))
}

fn get_formatted_date_from_file(
    file: &File,
    date_type_selected: &DateType,
) -> std::io::Result<String> {
    if let Some(metadata) = file.get_metadata() {
        if let Some(formatted_date) = metadata.get_formatted_date(*date_type_selected) {
            return Ok(formatted_date);
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not get formatted date from metadata.",
        ));
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Metadata not found.",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::Metadata;
    use std::time::SystemTime;

    #[test]
    fn test_get_formatted_date_from_file() {
        let result = get_formatted_date_from_file(
            &File::new(Metadata::build(
                Some(OsString::from("text.txt")),
                Some(SystemTime::UNIX_EPOCH),
                Some(SystemTime::UNIX_EPOCH),
                Some(SystemTime::UNIX_EPOCH),
                Some(10.5),
                false,
                Some(PathBuf::new()),
                Some(PathBuf::new()),
            )),
            &DateType::Created,
        );
        if let Ok(result) = result {
            let formatted = convert_system_time_to_string(SystemTime::UNIX_EPOCH);
            assert_eq!(result, formatted);
        } else {
            panic!("Result was not Ok");
        }
    }

    fn create_dummy_file_type_directories() -> BTreeMap<OsString, Directory> {
        let mut file_type_directories = BTreeMap::new();
        let mut txt_directory = Directory::new(None);
        txt_directory.insert_file(OsString::from("text1.txt"), File::new(Metadata::new()));
        txt_directory.insert_file(OsString::from("text2.txt"), File::new(Metadata::new()));
        file_type_directories.insert(OsString::from("txt"), txt_directory);
        file_type_directories.insert(OsString::from("jpg"), Directory::new(None));
        file_type_directories.insert(OsString::from("pdf"), Directory::new(None));
        file_type_directories.insert(OsString::from("png"), Directory::new(None));
        let mut other_directory = Directory::new(None);
        other_directory.insert_file(OsString::from("file"), File::new(Metadata::new()));
        file_type_directories.insert(OsString::from("other"), other_directory);
        file_type_directories
    }

    #[test]
    fn test_get_file_type_dir() {
        let mut file_type_directories = create_dummy_file_type_directories();
        match get_file_type_dir("text.txt", &mut file_type_directories) {
            Ok(file_type_dir) => {
                if let Some(name) = file_type_dir.get_name() {
                    assert_eq!(OsString::from("txt"), name);
                }
            }
            Err(error) => panic!("{}", error),
        }
        match get_file_type_dir("text", &mut file_type_directories) {
            Ok(file_type_dir) => {
                if let Some(name) = file_type_dir.get_name() {
                    assert_eq!(OsString::from("other"), name);
                }
            }
            Err(error) => panic!("{}", error),
        }
    }

    #[test]
    fn test_get_file_count_from_dir() {
        let file_type_directories = create_dummy_file_type_directories();
        let txt_file_count = get_file_count_from_dir("text.txt", &file_type_directories);
        assert_eq!(2, txt_file_count);
        let jpg_file_count = get_file_count_from_dir("image.jpg", &file_type_directories);
        assert_eq!(0, jpg_file_count);
        let other_file_count = get_file_count_from_dir("justfile", &file_type_directories);
        assert_eq!(1, other_file_count);
    }

    #[test]
    fn test_build_destination_path() {
        let path = build_destination_path(vec!["/", "home", "verneri", "filerganizer_test"]);
        assert_eq!(path, PathBuf::from("/home/verneri/filerganizer_test"));
    }

    fn create_dummy_files_selected() -> BTreeMap<OsString, File> {
        let mut files_selected = BTreeMap::new();
        files_selected.insert(
            OsString::from("file.txt"),
            File::new(Metadata::build(
                Some(OsString::from("file.txt")),
                Some(SystemTime::UNIX_EPOCH),
                Some(SystemTime::UNIX_EPOCH),
                Some(SystemTime::UNIX_EPOCH),
                Some(500.0),
                false,
                Some(PathBuf::from("")),
                Some(PathBuf::from("")),
            )),
        );
        files_selected.insert(OsString::from("file2.txt"), File::new(Metadata::new()));
        files_selected.insert(OsString::from("image.jpg"), File::new(Metadata::new()));
        files_selected.insert(
            OsString::from("description.pdf"),
            File::new(Metadata::new()),
        );
        files_selected.insert(OsString::from("file3.txt"), File::new(Metadata::new()));
        files_selected
    }

    fn convert_system_time_to_string(time: SystemTime) -> String {
        use chrono::{DateTime, Local};
        let date_time: DateTime<Local> = DateTime::<Local>::from(time);
        date_time.format("%Y%m%d").to_string()
    }

    #[test]
    fn test_create_file_dates() {
        let files_selected = create_dummy_files_selected();
        let file_date_dirs = create_file_dates(&files_selected, DateType::Created);
        for key in file_date_dirs.keys() {
            assert_eq!(
                &OsString::from(convert_system_time_to_string(SystemTime::UNIX_EPOCH)),
                key
            )
        }
    }
    #[test]
    fn test_get_file_types() {
        let files_selected = create_dummy_files_selected();
        let file_types = get_file_types(&files_selected);
        let test_file_types: [OsString; 3] = [
            OsString::from("jpg"),
            OsString::from("pdf"),
            OsString::from("txt"),
        ];
        let mut i = 0;
        for key in file_types.keys() {
            assert_eq!(key, &test_file_types[i]);
            i += 1;
        }
    }

    #[test]
    fn test_is_directory_name_unique() {
        let directories = create_dummy_file_type_directories();
        assert_eq!(false, is_directory_name_unique("txt", &directories));
        assert_eq!(true, is_directory_name_unique("html", &directories));
    }

    #[test]
    fn test_replace_non_ascii() {
        let result = replace_non_ascii(String::from("Ääni"));
        assert_eq!(String::from("Aani"), result);
    }

    #[test]
    fn test_get_file_name_without_file_type() {
        let without_filetype = get_file_name_without_file_type("filename_01.txt");
        assert_eq!(String::from("filename_01"), without_filetype);
        let without_filetype = get_file_name_without_file_type("filename");
        assert_eq!(String::from("filename"), without_filetype);
    }

    #[test]
    fn test_get_file_type_from_file_name() {
        if let Some(file_type) = get_file_type_from_file_name("text.txt") {
            assert_eq!(file_type, String::from("txt"))
        } else {
            panic!("Could not get filetype from a filename!");
        }
        if let Some(_file_type) = get_file_type_from_file_name("file") {
            panic!("filetype extension was not in filename. Should have returned None.");
        }
    }
}
