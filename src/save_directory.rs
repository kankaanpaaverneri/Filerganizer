use crate::{layouts::CheckboxStates, metadata::DateType};
use std::{
    io::{ErrorKind, Read, Write},
    path::PathBuf,
};

fn get_save_file_location(home_directory_path: &PathBuf) -> PathBuf {
    let mut path_to_save_file = PathBuf::from(home_directory_path);
    path_to_save_file.push(".save_file.csv");
    path_to_save_file
}

pub fn write_created_directory_to_save_file(
    home_directory_path: &PathBuf,
    directory_path: PathBuf,
    checkbox_states: CheckboxStates,
    date_type: Option<DateType>,
) -> std::io::Result<()> {
    
    match std::fs::File::options()
        .append(true)
        .open(get_save_file_location(home_directory_path))
    {
        Ok(mut file) => {
            if let Some(dir_path) = directory_path.to_str() {
                let mut new_directory_data = String::new();
                write_directory_data_to_string(
                    &mut new_directory_data,
                    dir_path,
                    checkbox_states,
                    date_type,
                );
                file.write(new_directory_data.as_bytes())?;
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not convert path to string.",
                ));
            }
        }
        Err(_) => {
            // Create file
            let mut save_file = create_save_file(home_directory_path)?;
            if let Some(dir_path) = directory_path.to_str() {
                let mut file_content = String::from("path, organize_by_file_type, organize_by_date, insert_date_to_file_name, insert_directory_name_to_file_name, remove_uppercase, replace_spaces_with_underscores, use_only_ascii, date_type\n");
                write_directory_data_to_string(
                    &mut file_content,
                    dir_path,
                    checkbox_states,
                    date_type,
                );
                save_file.write(file_content.as_bytes())?;
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not convert path to string.",
                ));
            }
        }
    }
    Ok(())
}

pub fn remove_directory_from_file(home_directory_path: &PathBuf, path_to_extracted_dir: PathBuf) -> std::io::Result<()> {
    let read_result = match std::fs::File::options().read(true).open(get_save_file_location(home_directory_path)) {
        Ok(mut file) => {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;

            let filtered: Vec<&str> = buffer
                .lines()
                .filter_map(|line| {
                    if let Some((path, _rest)) = line.split_once(",") {
                        if PathBuf::from(path) == path_to_extracted_dir {
                            return None;
                        }
                    }

                    Some(line)
                })
                .collect();
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
        .open(get_save_file_location(home_directory_path))?;
    file.set_len(0)?;
    file.write(updated_file_content.as_bytes())?;
    Ok(())
}

pub fn read_directory_rules_from_file(
    home_directory_path: &PathBuf,
    directory_path: &PathBuf,
) -> std::io::Result<(CheckboxStates, Option<DateType>)> {
    match std::fs::File::options().read(true).open(get_save_file_location(home_directory_path)) {
        Ok(mut file) => {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            if let Some(list_of_rules) = parse_file_result(buffer.as_str(), directory_path) {
                let checkbox_states = parse_rules(&list_of_rules);
                let date_type = parse_date_type(&list_of_rules);
                return Ok((checkbox_states, date_type));
            }
            Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Cannot select a non organized directory",
            ))
        }
        Err(error) => Err(error),
    }
}

fn parse_rules(list_of_rules: &Vec<&str>) -> CheckboxStates {
    let mut checkbox_states = CheckboxStates::default();
    if list_of_rules[1] == "1" {
        checkbox_states.organize_by_filetype = true;
    }
    if list_of_rules[2] == "1" {
        checkbox_states.organize_by_date = true;
    }

    if list_of_rules[3] == "1" {
        checkbox_states.insert_date_to_file_name = true;
    }

    if list_of_rules[4] == "1" {
        checkbox_states.insert_directory_name_to_file_name = true;
    }

    if list_of_rules[5] == "1" {
        checkbox_states.remove_uppercase = true;
    }

    if list_of_rules[6] == "1" {
        checkbox_states.replace_spaces_with_underscores = true;
    }

    if list_of_rules[7] == "1" {
        checkbox_states.use_only_ascii = true;
    }
    checkbox_states
}

fn parse_date_type(list_of_rules: &Vec<&str>) -> Option<DateType> {
    if let Some(date_type) = list_of_rules.last() {
        return match *date_type {
            "Created" => Some(DateType::Created),
            "Accessed" => Some(DateType::Accessed),
            "Modified" => Some(DateType::Modified),
            _ => return None,
        };
    }
    None
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

fn create_save_file(home_directory_path: &PathBuf) -> std::io::Result<std::fs::File> {
    match std::fs::File::create(get_save_file_location(home_directory_path)) {
        Ok(file) => Ok(file),
        Err(error) => Err(error),
    }
}

fn write_directory_data_to_string(
    file_content: &mut String,
    dir_path: &str,
    checkbox_states: CheckboxStates,
    date_type: Option<DateType>,
) {
    file_content.push_str(dir_path);
    file_content.push_str(",");
    if checkbox_states.organize_by_filetype {
        file_content.push_str("1,");
    } else {
        file_content.push_str("0,");
    }

    if checkbox_states.organize_by_date {
        file_content.push_str("1,");
    } else {
        file_content.push_str("0,");
    }

    if checkbox_states.insert_date_to_file_name {
        file_content.push_str("1,");
    } else {
        file_content.push_str("0,");
    }

    if checkbox_states.insert_directory_name_to_file_name {
        file_content.push_str("1,");
    } else {
        file_content.push_str("0,");
    }

    if checkbox_states.remove_uppercase {
        file_content.push_str("1,");
    } else {
        file_content.push_str("0,");
    }

    if checkbox_states.replace_spaces_with_underscores {
        file_content.push_str("1,");
    } else {
        file_content.push_str("0,");
    }

    if checkbox_states.use_only_ascii {
        file_content.push_str("1,");
    } else {
        file_content.push_str("0,");
    }

    if let Some(date_type) = date_type {
        match date_type {
            DateType::Created => file_content.push_str("Created\n"),
            DateType::Accessed => file_content.push_str("Accessed\n"),
            DateType::Modified => file_content.push_str("Modified\n"),
        }
    } else {
        file_content.push_str("None\n");
    }
}
