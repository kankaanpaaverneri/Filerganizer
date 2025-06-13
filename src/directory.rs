use crate::file::File;
use crate::metadata::Metadata;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs::{DirEntry, ReadDir};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Directory {
    directories: Option<BTreeMap<OsString, Directory>>,
    files: Option<BTreeMap<OsString, File>>,
    metadata: Option<Metadata>,
}

impl Directory {
    pub fn new(metadata: Option<Metadata>) -> Self {
        Directory {
            directories: None,
            files: None,
            metadata,
        }
    }

    pub fn insert_file(&mut self, file_name: OsString, file: File) {
        if let Some(mut files) = self.files.take() {
            files.insert(file_name, file);
            self.files = Some(files);
        } else {
            let mut files = BTreeMap::new();
            files.insert(file_name, file);
            let _result = self.files.insert(files);
        }
    }

    pub fn insert_directory(&mut self, new_directory: Directory, directory_name: &str) {
        if let Some(mut directories) = self.directories.take() {
            directories.insert(OsString::from(directory_name), new_directory);
            self.directories = Some(directories);
        } else {
            let mut new_directories = BTreeMap::new();
            new_directories.insert(OsString::from(directory_name), new_directory);
            let _result = self.directories.insert(new_directories);
        }
    }

    pub fn get_files_recursive(
        &mut self,
        files_holder: &mut BTreeMap<OsString, File>,
        path_to_selected_directory: &mut PathBuf,
    ) -> std::io::Result<()> {
        self.contains_unique_files(files_holder)?;
        if let Some(files) = self.files.take() {
            for (key, value) in files {
                files_holder.insert(key, value);
            }
        }

        if let Some(directories) = &mut self.directories {
            for (key, directory) in directories {
                path_to_selected_directory.push(key);
                directory.get_files_recursive(files_holder, path_to_selected_directory)?;
                path_to_selected_directory.pop();
            }
        }
        self.clear_directory_content();

        Ok(())
    }

    pub fn contains_unique_files(
        &self,
        files_holder: &BTreeMap<OsString, File>,
    ) -> std::io::Result<()> {
        if let Some(files) = &self.files {
            for key in files.keys() {
                if files_holder.contains_key(key) {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        "Duplicate files found in directory",
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn contains_unique_files_recursive(
        &self,
        files_holder: &BTreeMap<OsString, File>,
    ) -> std::io::Result<()> {
        if let Some(directories) = self.get_directories() {
            for (_key, dir) in directories {
                if let Some(files) = dir.get_files() {
                    for file_name in files.keys() {
                        if files_holder.contains_key(file_name) {
                            return Err(std::io::Error::new(
                                ErrorKind::InvalidData,
                                "Duplicate files detected in directories",
                            ));
                        }
                    }
                }
                dir.contains_unique_files_recursive(files_holder)?;
            }
        }
        Ok(())
    }

    pub fn read_path(
        &mut self,
        path: &PathBuf,
        new_directory: &mut Directory,
    ) -> std::io::Result<()> {
        match std::fs::read_dir(path) {
            Ok(read_dir) => {
                let metadata = self.read_parent(path);
                let mut directories = BTreeMap::new();
                let mut files = BTreeMap::new();
                insert_entries(&mut directories, &mut files, read_dir);

                if let None = self.directories {
                    new_directory.directories = Some(directories);
                    new_directory.files = Some(files);
                    new_directory.metadata = metadata;
                } else {
                    if let Some(last_dir) = self.get_mut_directory_by_path(path) {
                        last_dir.directories = Some(directories);
                        last_dir.files = Some(files);
                        last_dir.metadata = metadata;
                    }
                }
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn get_mut_directory_by_path(&mut self, path: &PathBuf) -> Option<&mut Directory> {
        let mut current_directory = self;
        if let Ok(striped_path) = remove_prefix_from_path(path) {
            for path_directory in striped_path {
                if let Some(sub_directories) = &mut current_directory.directories {
                    if let Some(sub_directory) = sub_directories.get_mut(path_directory) {
                        current_directory = sub_directory;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            return Some(current_directory);
        }
        None
    }

    pub fn get_directory_by_path(&self, path: &PathBuf) -> &Directory {
        let mut current_directory = self;
        for path_directory in path {
            if let Some(ref sub_directories) = current_directory.directories {
                if let Some(sub_directory) = sub_directories.get(path_directory) {
                    current_directory = sub_directory;
                }
            }
        }
        current_directory
    }

    pub fn get_file_count(&self) -> usize {
        if let Some(files) = &self.files {
            return files.len();
        }
        0
    }

    pub fn clear_directory_content(&mut self) {
        if let Some(directories) = self.directories.as_mut() {
            directories.clear();
        }
        if let Some(files) = self.files.as_mut() {
            files.clear();
        }
        self.directories = None;
        self.files = None;
    }

    pub fn write_directories_recursive(&mut self, path: &mut PathBuf) -> std::io::Result<()> {
        if let Some(mut sub_directories) = self.directories.take() {
            let mut new_directories = BTreeMap::new();
            for (name, mut directory) in sub_directories {
                path.push(&name);
                let mut new_sub_directory = Directory::new(None);
                directory.read_path(path, &mut new_sub_directory)?;
                new_sub_directory.write_directories_recursive(path)?;
                path.pop();
                new_directories.insert(OsString::from(&name), new_sub_directory);
            }
            sub_directories = new_directories;
            self.directories = Some(sub_directories);
        }
        Ok(())
    }

    pub fn remove_sub_directory(&mut self, directory_name: &OsStr) {
        if let Some(directories) = &mut self.directories {
            directories.remove(directory_name);
        }
    }

    pub fn filter_duplicate_directories(&self, directories: &mut BTreeMap<OsString, Directory>) {
        if let Some(selected_dirs) = self.get_directories() {
            *directories = directories
                .iter()
                .filter_map(|(key, dir)| {
                    if selected_dirs.contains_key(key) {
                        return None;
                    }
                    return Some((OsString::from(key.as_os_str()), (*dir).clone()));
                })
                .collect();
        }
    }

    pub fn file_aready_exists_in_directory(&self, filename: &OsStr) -> std::io::Result<()> {
        if let Some(files) = &self.files {
            for key in files.keys() {
                if key == filename {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Filename already exists in directory",
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn get_directories(&self) -> &Option<BTreeMap<OsString, Directory>> {
        &self.directories
    }

    pub fn get_mut_directories(&mut self) -> &mut Option<BTreeMap<OsString, Directory>> {
        &mut self.directories
    }

    pub fn get_files(&self) -> &Option<BTreeMap<OsString, File>> {
        &self.files
    }

    pub fn get_mut_files(&mut self) -> &mut Option<BTreeMap<OsString, File>> {
        &mut self.files
    }

    pub fn get_metadata(&self) -> &Option<Metadata> {
        &self.metadata
    }

    fn read_parent(&self, path: &PathBuf) -> Option<Metadata> {
        if let Some(last) = path.iter().last() {
            let parent_path: PathBuf = path
                .iter()
                .filter_map(|directory| {
                    if directory == last {
                        return None;
                    }
                    Some(directory)
                })
                .collect();
            if !parent_path.as_os_str().is_empty() {
                if let Ok(metadata) = read_parent_entry(&parent_path, last) {
                    return metadata;
                }
            }
        }
        None
    }
}

fn read_parent_entry(path: &PathBuf, last_directory: &OsStr) -> std::io::Result<Option<Metadata>> {
    match std::fs::read_dir(path) {
        Ok(read_dir) => {
            for entry in read_dir {
                if let Some(ok_entry) = entry.ok() {
                    if ok_entry.file_name() == last_directory {
                        if let Some(parent) = write_directory_entry(&ok_entry) {
                            return Ok(parent.get_metadata().clone());
                        }
                    }
                }
            }
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn insert_entries(
    directories: &mut BTreeMap<OsString, Directory>,
    files: &mut BTreeMap<OsString, File>,
    read_dir: ReadDir,
) {
    for entry in read_dir {
        if let Some(ok_entry) = entry.ok() {
            let file_name = ok_entry.file_name();

            if let Some(directory) = write_directory_entry(&ok_entry) {
                directories.insert(OsString::from(file_name.as_os_str()), directory);
            }
            if let Some(file) = write_file_entry(&ok_entry) {
                files.insert(OsString::from(file_name.as_os_str()), file);
            }
        }
    }
}

fn write_directory_entry(entry: &DirEntry) -> Option<Directory> {
    match entry.metadata() {
        Ok(metadata) => {
            let created = metadata.created().ok().take();
            let accessed = metadata.accessed().ok().take();
            let modified = metadata.modified().ok().take();
            let readonly = metadata.permissions().readonly();
            if metadata.is_dir() {
                return Some(Directory::new(Some(Metadata::build(
                    Some(entry.file_name()),
                    created,
                    accessed,
                    modified,
                    None,
                    readonly,
                ))));
            }
            None
        }
        _ => None,
    }
}

fn write_file_entry(entry: &DirEntry) -> Option<File> {
    match entry.metadata() {
        Ok(metadata) => {
            let created = metadata.created().ok().take();
            let accessed = metadata.accessed().ok().take();
            let modified = metadata.modified().ok().take();
            let size = metadata.len() as f64;
            let readonly = metadata.permissions().readonly();
            if metadata.is_file() {
                return Some(File::new(Metadata::build(
                    Some(entry.file_name()),
                    created,
                    accessed,
                    modified,
                    Some(size),
                    readonly,
                )));
            }
            None
        }
        _ => None,
    }
}

fn remove_prefix_from_path(path: &PathBuf) -> Result<&Path, std::path::StripPrefixError> {
    match std::env::consts::OS {
        "windows" => path.strip_prefix(identify_prefix(path)),
        "macos" => path.strip_prefix(OsString::from("/")),
        _ => path.strip_prefix(OsString::from("/")),
    }
}

fn identify_prefix(path: &PathBuf) -> String {
    let first_two_components: Vec<_> = path
        .iter()
        .take(2)
        .filter_map(|component| {
            if let Some(element) = component.to_str() {
                return Some(element);
            }
            None
        })
        .collect();
    first_two_components.join("/")
}

pub mod system_dir {
    use std::path::PathBuf;
    pub fn get_home_directory() -> Option<PathBuf> {
        let environment_var = match std::env::consts::OS {
            "windows" => std::env::var_os("USERPROFILE"),
            "macos" | "linux" => std::env::var_os("HOME"),
            _ => None,
        };

        if let Some(key) = environment_var {
            return Some(PathBuf::from(key));
        }
        None
    }
}

pub mod organizing {
    use crate::app::filename_components;
    use crate::directory::Directory;
    use crate::file::File;
    use crate::layouts::{CheckboxStates, IndexPosition};
    use crate::metadata::DateType;
    use std::collections::BTreeMap;
    use std::ffi::OsString;

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
                            file_type_dir.file_aready_exists_in_directory(&OsString::from(
                                &renamed_file_name,
                            ))?;
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
                        if let Some(dir) = file_date_directories.get_mut(&OsString::from(formatted))
                        {
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
                            dir.file_aready_exists_in_directory(&OsString::from(
                                &renamed_file_name,
                            ))?;
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
                if checkbox_states.replace_spaces_with_underscores && component != last {
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

    pub fn get_file_types(
        files_selected: &BTreeMap<OsString, File>,
    ) -> BTreeMap<OsString, Directory> {
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
}
