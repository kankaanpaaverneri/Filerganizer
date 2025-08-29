use crate::app::filename_components;
use crate::app_util;
use crate::{layouts::CheckboxStates, metadata::DateType};
use std::{
    io::{ErrorKind, Read, Write},
    path::PathBuf,
};

const CSV_FILE_HEADER: &str = "path, organize_by_file_type, organize_by_date, convert_uppercase_to_lowercase, replace_character, use_only_ascii, insert_directory_name_to_file_name, insert_date_to_file_name, remove_original_file_name, add_custom_name, date_type, component_order\n";

pub const SAVE_FILE_NAME: &str = ".save_file.csv";

fn get_save_file_location(home_directory_path: &PathBuf, save_file_name: &str) -> PathBuf {
    let mut path_to_save_file = PathBuf::from(home_directory_path);
    path_to_save_file.push(save_file_name);
    path_to_save_file
}

pub fn write_created_directory_to_save_file(
    home_directory_path: &PathBuf,
    directory_path: PathBuf,
    checkbox_states: CheckboxStates,
    date_type: Option<DateType>,
    order_of_filename_components: &Vec<String>,
    custom_filename: &str,
) -> std::io::Result<()> {
    match std::fs::File::options()
        .append(true)
        .open(get_save_file_location(home_directory_path, SAVE_FILE_NAME))
    {
        Ok(mut file) => {
            // Append to existing file
            let dir_path = app_util::convert_path_to_str(&directory_path)?;
            let mut new_directory_data = String::new();
            write_directory_data_to_string(
                &mut new_directory_data,
                dir_path,
                checkbox_states,
                date_type,
                order_of_filename_components,
                custom_filename,
            );
            file.write(new_directory_data.as_bytes())?;
        }
        Err(_) => {
            // Create new file
            let mut save_file = create_save_file(home_directory_path, SAVE_FILE_NAME)?;
            let dir_path = app_util::convert_path_to_str(&directory_path)?;
            let mut file_content = String::from(CSV_FILE_HEADER);
            write_directory_data_to_string(
                &mut file_content,
                dir_path,
                checkbox_states,
                date_type,
                order_of_filename_components,
                custom_filename,
            );
            save_file.write(file_content.as_bytes())?;
        }
    }
    Ok(())
}

pub fn remove_directory_from_file(
    home_directory_path: &PathBuf,
    path_to_extracted_dir: &PathBuf,
) -> std::io::Result<()> {
    let read_result = match std::fs::File::options()
        .read(true)
        .open(get_save_file_location(home_directory_path, SAVE_FILE_NAME))
    {
        Ok(mut file) => {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;

            // Filter file content
            let filtered = filter_path_from_file_content(&mut buffer, path_to_extracted_dir);
            let mut updated_file_content = String::new();
            for line in filtered {
                updated_file_content.push_str(line);
                updated_file_content.push('\n');
            }
            Ok(updated_file_content)
        }
        Err(error) => Err(error),
    };
    let updated_file_content = read_result?;
    let mut file = std::fs::File::options()
        .truncate(true)
        .write(true)
        .open(get_save_file_location(home_directory_path, SAVE_FILE_NAME))?;
    file.set_len(0)?;
    file.write(updated_file_content.as_bytes())?;
    Ok(())
}

pub fn read_directory_rules_from_file(
    home_directory_path: &PathBuf,
    directory_path: &PathBuf,
) -> std::io::Result<(CheckboxStates, Option<DateType>, Vec<String>, String)> {
    match std::fs::File::options()
        .read(true)
        .open(get_save_file_location(home_directory_path, SAVE_FILE_NAME))
    {
        Ok(mut file) => {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            if let Some(list_of_rules) = parse_file_result(buffer.as_str(), directory_path) {
                let checkbox_states = parse_rules(&list_of_rules);
                let date_type = parse_date_type(&list_of_rules);
                let order_of_filename_components = parse_filename_components(&list_of_rules);
                let custom_filename = parse_custom_filename(&list_of_rules);
                return Ok((
                    checkbox_states,
                    date_type,
                    order_of_filename_components,
                    custom_filename,
                ));
            }
            Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Cannot select a non organized directory",
            ))
        }
        Err(error) => Err(error),
    }
}

pub fn read_save_file_content(
    home_directory_path: &PathBuf,
    directory_path: &PathBuf,
    save_file_name: &str,
) -> std::io::Result<()> {
    let mut file = std::fs::File::options()
        .read(true)
        .open(get_save_file_location(home_directory_path, save_file_name))?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    let dir_path = app_util::convert_path_to_str(directory_path)?;
    for line in file_content.lines() {
        if let Some((path, _checkbox_states)) = line.split_once(",") {
            if path == dir_path {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Similar path already exists.",
                ));
            }
        }
    }
    Ok(())
}

fn parse_rules(list_of_rules: &Vec<&str>) -> CheckboxStates {
    let mut checkbox_states = CheckboxStates::default();
    let mut checkbox_states_array: [&mut bool; 9] = [
        &mut checkbox_states.organize_by_filetype,
        &mut checkbox_states.organize_by_date,
        &mut checkbox_states.convert_uppercase_to_lowercase,
        &mut checkbox_states.replace_character,
        &mut checkbox_states.use_only_ascii,
        &mut checkbox_states.insert_directory_name_to_file_name,
        &mut checkbox_states.insert_date_to_file_name,
        &mut checkbox_states.remove_original_file_name,
        &mut checkbox_states.add_custom_name,
    ];
    for (i, checkbox) in checkbox_states_array.iter_mut().enumerate() {
        if list_of_rules[i + 1] == "1" {
            **checkbox = true;
        }
    }
    checkbox_states
}

fn parse_date_type(list_of_rules: &Vec<&str>) -> Option<DateType> {
    if list_of_rules.len() < 11 {
        return None;
    }
    let date_type = list_of_rules[10];
    return match date_type {
        "Created" => Some(DateType::Created),
        "Accessed" => Some(DateType::Accessed),
        "Modified" => Some(DateType::Modified),
        _ => return None,
    };
}

fn parse_filename_components(list_of_rules: &Vec<&str>) -> Vec<String> {
    let mut order_of_filename_components = Vec::new();
    for rule in list_of_rules {
        let component = match *rule {
            "directory_name" => filename_components::DIRECTORY_NAME,
            "date" => filename_components::DATE,
            "custom_file_name" => filename_components::CUSTOM_FILE_NAME,
            "original_filename" => filename_components::ORIGINAL_FILENAME,
            _ => "",
        };
        if !component.is_empty() {
            order_of_filename_components.push(String::from(component));
        }
    }
    order_of_filename_components
}

fn parse_custom_filename(list_of_rules: &Vec<&str>) -> String {
    let mut custom_filename = String::new();
    if let Some(last) = list_of_rules.last() {
        custom_filename.push_str(last);
    }
    custom_filename
}

fn parse_file_result<'a>(buffer: &'a str, path: &'a PathBuf) -> Option<Vec<&'a str>> {
    let line = buffer.lines().find(|line| {
        if let Some(path) = path.to_str() {
            if line.contains(path) {
                return true;
            }
        }
        false
    });
    if let Some(line) = line {
        let directory_rules: Vec<&'a str> = line.split(",").collect();
        return Some(directory_rules);
    }

    None
}

pub fn create_save_file(
    home_directory_path: &PathBuf,
    save_file_name: &str,
) -> std::io::Result<std::fs::File> {
    match std::fs::File::create(get_save_file_location(home_directory_path, save_file_name)) {
        Ok(file) => Ok(file),
        Err(error) => Err(error),
    }
}

fn write_directory_data_to_string(
    file_content: &mut String,
    dir_path: &str,
    checkbox_states: CheckboxStates,
    date_type: Option<DateType>,
    order_of_filename_components: &Vec<String>,
    custom_filename: &str,
) {
    file_content.push_str(dir_path);
    file_content.push_str(",");
    write_value_to_file_content(file_content, checkbox_states.organize_by_filetype);
    write_value_to_file_content(file_content, checkbox_states.organize_by_date);
    write_value_to_file_content(file_content, checkbox_states.convert_uppercase_to_lowercase);
    write_value_to_file_content(file_content, checkbox_states.replace_character);
    write_value_to_file_content(file_content, checkbox_states.use_only_ascii);
    write_value_to_file_content(
        file_content,
        checkbox_states.insert_directory_name_to_file_name,
    );
    write_value_to_file_content(file_content, checkbox_states.insert_date_to_file_name);
    write_value_to_file_content(file_content, checkbox_states.remove_original_file_name);
    write_value_to_file_content(file_content, checkbox_states.add_custom_name);

    if let Some(date_type) = date_type {
        match date_type {
            DateType::Created => file_content.push_str("Created"),
            DateType::Accessed => file_content.push_str("Accessed"),
            DateType::Modified => file_content.push_str("Modified"),
        }
    } else {
        file_content.push_str("None");
    }
    write_order_of_filename_components(file_content, order_of_filename_components);
    if order_of_filename_components.contains(&String::from(filename_components::CUSTOM_FILE_NAME)) {
        file_content.push_str(",");
        file_content.push_str(custom_filename);
    }
    file_content.push_str("\n");
}

fn write_order_of_filename_components(
    file_content: &mut String,
    order_of_filename_components: &Vec<String>,
) {
    for component in order_of_filename_components {
        match component.as_str() {
            filename_components::DATE => file_content.push_str(",date"),
            filename_components::ORIGINAL_FILENAME => file_content.push_str(",original_filename"),
            filename_components::DIRECTORY_NAME => file_content.push_str(",directory_name"),
            filename_components::CUSTOM_FILE_NAME => file_content.push_str(",custom_file_name"),
            _ => {}
        }
    }
}

fn write_value_to_file_content(file_content: &mut String, value: bool) {
    if value {
        file_content.push_str("1,");
    } else {
        file_content.push_str("0,");
    }
}

fn filter_path_from_file_content<'a>(
    buffer: &'a mut String,
    path_to_remove: &'a PathBuf,
) -> Vec<&'a str> {
    buffer
        .lines()
        .filter_map(|line| {
            if let Some((path, _rest)) = line.split_once(",") {
                if &PathBuf::from(path) == path_to_remove {
                    return None;
                }
            }
            Some(line)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_path_from_file_content() {
        let mut file_content = String::from(CSV_FILE_HEADER);
        file_content.push_str("/home/verneri/screen_record/records,0,0,1,1,1,1,1,1,1,Created\n");
        file_content.push_str("/home/verneri/screen_record/template,1,1,1,1,1,1,1,1,1,Modified\n");

        let path_to_remove = PathBuf::from("/home/verneri/screen_record/template");

        let filtered = filter_path_from_file_content(&mut file_content, &path_to_remove);
        let csv_file_header = String::from(CSV_FILE_HEADER);
        let replaced = csv_file_header.replace("\n", "");
        let expected_file_content = vec![
            &replaced,
            "/home/verneri/screen_record/records,0,0,1,1,1,1,1,1,1,Created",
        ];
        assert_eq!(expected_file_content, filtered);

        let second_path_to_remove = PathBuf::from("/home/verneri/screen_record/records");

        let second_filtered =
            filter_path_from_file_content(&mut file_content, &second_path_to_remove);
        let second_expected_file_content = vec![
            &replaced,
            "/home/verneri/screen_record/template,1,1,1,1,1,1,1,1,1,Modified",
        ];
        assert_eq!(second_expected_file_content, second_filtered);
    }

    #[test]
    fn test_parse_file_result() {
        let mut buffer = String::from(CSV_FILE_HEADER);
        buffer.push_str("/home/verneri/screen_record/records,0,0,1,1,1,1,1,1,1,Created\n");
        buffer.push_str("/home/verneri/screen_record/template,1,1,1,1,1,1,1,1,1,Modified\n");
        let path = PathBuf::from("/home/verneri/screen_record/template");
        if let Some(result) = parse_file_result(&buffer, &path) {
            assert_eq!(
                result,
                vec![
                    "/home/verneri/screen_record/template",
                    "1",
                    "1",
                    "1",
                    "1",
                    "1",
                    "1",
                    "1",
                    "1",
                    "1",
                    "Modified"
                ]
            );
        } else {
            panic!("Could not parse file result");
        }
    }

    #[test]
    fn test_parse_date_type() {
        let list_of_rules = vec![
            "/home/verneri/screen_record/template",
            "1",
            "1",
            "1",
            "1",
            "1",
            "1",
            "1",
            "1",
            "1",
            "Modified",
        ];
        if let Some(date_type) = parse_date_type(&list_of_rules) {
            assert_eq!(date_type, DateType::Modified);
        } else {
            panic!("Could not parse date type");
        }
    }

    #[test]
    fn test_parse_rules() {
        let list_of_rules = vec![
            "/home/verneri/screen_record/template",
            "0",
            "0",
            "1",
            "1",
            "1",
            "1",
            "1",
            "0",
            "0",
            "Modified",
        ];

        assert_eq!(
            parse_rules(&list_of_rules),
            CheckboxStates::new(false, false, true, true, true, true, true, false, false)
        );
    }
}
