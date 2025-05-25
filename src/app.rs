use iced::widget::Container;
use iced::Task;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs::read_dir;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::directory::organizing;
use crate::directory::Directory;
use crate::file::File;
use crate::layouts::{CheckboxStates, DirectoryView, Layout};
use crate::metadata::DateType;

pub struct App {
    path: PathBuf,
    path_input: String,
    error: String,
    root: Directory,
    external_storage: BTreeSet<OsString>,
    layout: Layout,
    directory_view: DirectoryView,

    directories_selected: Vec<PathBuf>,
    files_selected: BTreeMap<OsString, File>,
    new_directory_name: String,
    checkbox_states: CheckboxStates,
    date_type_selected: Option<DateType>,
}

impl Default for App {
    fn default() -> Self {
        App {
            path: PathBuf::new(),
            path_input: String::new(),
            error: String::new(),
            root: Directory::new(None),
            external_storage: BTreeSet::new(),
            layout: Layout::Main,
            directory_view: DirectoryView::List,

            directories_selected: Vec::new(),
            files_selected: BTreeMap::new(),
            new_directory_name: String::new(),
            checkbox_states: CheckboxStates::default(),
            date_type_selected: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchLayout(Layout),
    SwitchDirectoryView(DirectoryView),
    TextInput(String),
    SearchPath,
    MoveDownDirectory(OsString),
    MoveUpDirectory,
    MoveInExternalDirectory(OsString),
    DropDownDirectory(PathBuf),

    SelectPath,
    SelectDirectory(PathBuf),
    SelectFile(PathBuf),
    SelectAllFiles,
    PutAllFilesBack,
    InputNewDirectoryName(String),
    CreateDirectoryWithSelectedFiles,
    RenameFiles,
    CheckboxToggled(bool, usize),
    DateTypeSelected(DateType),
    ExtractContentFromDirectory(PathBuf),
    ExtractAllContentFromDirectory(PathBuf),
    Back,
    Exit,
}

impl App {
    pub fn view(&self) -> Container<Message> {
        self.layout.get_layout(self)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.error.clear();
        match message {
            Message::SwitchLayout(layout) => {
                if let Err(error) = self.switch_layout(&layout) {
                    self.error = error.to_string();
                }
                Task::none()
            }
            Message::TextInput(text_input) => {
                self.path_input = text_input;
                Task::none()
            }
            Message::SearchPath => {
                if let Err(error) = self.search_path() {
                    self.error = error.to_string();
                }
                Task::none()
            }
            Message::MoveDownDirectory(directory_name) => {
                if let Err(error) = self.move_down_directory(&directory_name) {
                    self.error = error.to_string();
                }
                Task::none()
            }
            Message::MoveUpDirectory => {
                self.move_up_directory();
                Task::none()
            }
            Message::MoveInExternalDirectory(external) => {
                if let Err(error) = self.move_in_external_directory(&external) {
                    self.error = error.to_string();
                }
                Task::none()
            }
            Message::DropDownDirectory(path_to_selected_directory) => {
                if let Err(error) = self.select_drop_down_directory(&path_to_selected_directory) {
                    self.error = error.to_string();
                }
                Task::none()
            }
            Message::SwitchDirectoryView(directory_view) => match directory_view {
                DirectoryView::List => {
                    if let DirectoryView::DropDown = self.directory_view {
                        self.directory_view = directory_view;
                    }
                    Task::none()
                }
                DirectoryView::DropDown => {
                    if let DirectoryView::List = self.directory_view {
                        self.directory_view = directory_view;
                    }
                    Task::none()
                }
            },
            Message::SelectPath => match self.switch_layout(&Layout::DirectoryOrganizingLayout) {
                Ok(_) => Task::none(),
                Err(error) => {
                    self.error = error.to_string();
                    return Task::none();
                }
            },
            Message::SelectDirectory(path_to_selected_directory) => {
                if self.directories_selected.is_empty() {
                    self.insert_directory_path_to_selected(path_to_selected_directory);
                } else {
                    if let Some(last_path) = self.directories_selected.last() {
                        if are_paths_equal(last_path, &path_to_selected_directory) {
                            self.insert_directory_path_to_selected(path_to_selected_directory);
                        } else {
                            while let Some(popped) = self.directories_selected.pop() {
                                if are_paths_equal(&popped, &path_to_selected_directory) {
                                    self.directories_selected.push(popped);
                                    break;
                                }
                            }
                            self.insert_directory_path_to_selected(path_to_selected_directory);
                        }
                    }
                }

                Task::none()
            }
            Message::SelectFile(file_path) => {
                if let Some(directory) = self.root.get_mut_directory_by_path(&self.path) {
                    if let Some(files) = directory.get_mut_files() {
                        if let Some(file_name) = file_path.iter().last() {
                            if let Err(error) =
                                select_file(files, &mut self.files_selected, file_name)
                            {
                                self.error = error.to_string();
                            }
                        }
                    }
                }
                return Task::none();
            }
            Message::SelectAllFiles => {
                if let Err(error) = self.is_duplicate_files_selected() {
                    self.error = error.to_string();
                    return Task::none();
                }
                if let Some(selected_dir) = self.root.get_mut_directory_by_path(&self.path) {
                    if let Some(files) = selected_dir.get_mut_files() {
                        while let Some((key, value)) = files.pop_last() {
                            self.files_selected.insert(key, value);
                        }
                    }
                }
                return Task::none();
            }
            Message::PutAllFilesBack => {
                if let Err(error) = self.is_duplicate_files_selected() {
                    self.error = error.to_string();
                    return Task::none();
                }

                if let Some(selected_dir) = self.root.get_mut_directory_by_path(&self.path) {
                    while let Some((key, value)) = self.files_selected.pop_last() {
                        selected_dir.insert_file(key, value);
                    }
                }
                Task::none()
            }
            Message::InputNewDirectoryName(input) => {
                self.new_directory_name = input;
                Task::none()
            }
            Message::CreateDirectoryWithSelectedFiles => {
                if let Err(error) = self.is_directory_creation_valid() {
                    self.error = error.to_string();
                    return Task::none();
                }

                let mut files_selected = BTreeMap::new();
                while let Some((key, value)) = self.files_selected.pop_last() {
                    files_selected.insert(key, value);
                }
                match self.create_directory_with_selected_files(files_selected) {
                    Ok(_) => {
                        let mut path_to_directory = PathBuf::from(&self.path);
                        path_to_directory.push(&self.new_directory_name);

                        match save_directory::write_created_directory_to_save_file(
                            path_to_directory,
                            self.checkbox_states.clone(),
                            self.date_type_selected,
                        ) {
                            Ok(_) => {
                                self.new_directory_name.clear();
                            }
                            Err(error) => {
                                self.error = error.to_string();
                            }
                        }
                    }
                    Err(error) => self.error = error.to_string(),
                }

                Task::none()
            }
            Message::RenameFiles => {
                let insert_date_to_file_name = self.checkbox_states.insert_date_to_file_name;
                let remove_uppercase = self.checkbox_states.remove_uppercase;
                let replace_spaces_with_underscores =
                    self.checkbox_states.replace_spaces_with_underscores;
                let use_only_ascii = self.checkbox_states.use_only_ascii;

                if insert_date_to_file_name
                    || remove_uppercase
                    || replace_spaces_with_underscores
                    || use_only_ascii
                {
                    if insert_date_to_file_name {
                        if let Some(date_type) = self.date_type_selected {
                            let result = self.rename_files_without_directory(
                                insert_date_to_file_name,
                                remove_uppercase,
                                replace_spaces_with_underscores,
                                use_only_ascii,
                                Some(date_type),
                            );
                            if let Err(error) = result {
                                self.error = error.to_string();
                            }
                        } else {
                            self.error =
                                std::io::Error::new(ErrorKind::NotFound, "No date type specified")
                                    .to_string();
                        }
                    } else {
                        let result = self.rename_files_without_directory(
                            false,
                            remove_uppercase,
                            replace_spaces_with_underscores,
                            use_only_ascii,
                            None,
                        );
                        if let Err(error) = result {
                            self.error = error.to_string();
                        }
                    }
                } else {
                    self.error =
                        std::io::Error::new(ErrorKind::NotFound, "No rename options specified")
                            .to_string();
                }
                Task::none()
            }
            Message::CheckboxToggled(toggle, id) => {
                self.toggle_checkbox(toggle, id);
                Task::none()
            }
            Message::DateTypeSelected(date_type) => {
                self.date_type_selected = Some(date_type);
                Task::none()
            }
            Message::ExtractContentFromDirectory(mut path_to_selected_directory) => {
                let mut path_to_parent_directory = PathBuf::from(&path_to_selected_directory);
                if path_to_parent_directory.pop() {
                    match self.extract_content_from_directory(
                        &mut path_to_selected_directory,
                        &path_to_parent_directory,
                    ) {
                        Ok(_) => {
                            // Delete path from .save_file.txt
                            if let Err(error) = save_directory::remove_directory_from_file(
                                path_to_selected_directory,
                            ) {
                                self.error = error.to_string();
                            }
                        }
                        Err(error) => self.error = error.to_string(),
                    }
                }

                Task::none()
            }
            Message::ExtractAllContentFromDirectory(mut path_to_selected_directory) => {
                let mut path_to_parent_directory = PathBuf::from(&path_to_selected_directory);
                if path_to_parent_directory.pop() {
                    if let Err(error) = self.extract_all_files_from_directory(
                        &path_to_parent_directory,
                        &mut path_to_selected_directory,
                    ) {
                        self.error = error.to_string();
                    }
                }

                Task::none()
            }
            Message::Back => {
                self.init_app_data();
                if let Err(error) = self.switch_layout(&Layout::DirectorySelectionLayout) {
                    self.error = error.to_string();
                }
                Task::none()
            }
            Message::Exit => iced::exit(),
        }
    }

    pub fn get_root_directory(&self) -> &Directory {
        &self.root
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_path_input(&self) -> &str {
        self.path_input.as_str()
    }

    pub fn get_error(&self) -> &str {
        self.error.as_str()
    }

    pub fn get_external_directories(&self) -> &BTreeSet<OsString> {
        &self.external_storage
    }

    pub fn get_directory_view(&self) -> DirectoryView {
        self.directory_view.clone()
    }

    pub fn get_files_selected(&self) -> &BTreeMap<OsString, File> {
        &self.files_selected
    }

    pub fn get_directories_selected(&self) -> &Vec<PathBuf> {
        &self.directories_selected
    }

    pub fn get_new_directory_input(&self) -> &String {
        &self.new_directory_name
    }

    pub fn get_checkbox_states(&self) -> &CheckboxStates {
        &self.checkbox_states
    }

    pub fn get_date_type_selected(&self) -> Option<DateType> {
        self.date_type_selected
    }

    fn switch_layout(&mut self, layout: &Layout) -> std::io::Result<()> {
        match layout {
            Layout::DirectorySelectionLayout => match std::env::consts::OS {
                "windows" => {
                    if let Some(first) = self.get_drives_on_windows().first() {
                        let path = PathBuf::from(first);
                        for path in self.get_drives_on_windows() {
                            self.external_storage.insert(path);
                        }
                        self.insert_root_directory(&path);
                        self.update_path_input();
                    }
                    self.layout = Layout::DirectorySelectionLayout;
                    Ok(())
                }
                "macos" => {
                    let mut path = PathBuf::from("/");
                    self.insert_root_directory(&path);
                    self.write_directory_to_tree(&path)?;
                    path.push("Volumes");
                    self.write_directory_to_tree(&path)?;
                    self.get_volumes_on_macos();
                    self.write_directories_from_path(&PathBuf::from("/Users/vernerikankaanpaa"))?;
                    self.update_path_input();
                    self.layout = Layout::DirectorySelectionLayout;
                    Ok(())
                }
                _ => Ok(()),
            },
            Layout::Main => {
                self.init_app_data();
                self.layout = Layout::Main;
                Ok(())
            }
            Layout::DirectoryOrganizingLayout => {
                let mut path = PathBuf::from(&self.path);
                if let Err(error) = self.write_selected_directory_recursively(&mut path) {
                    return Err(error);
                }
                self.layout = Layout::DirectoryOrganizingLayout;
                Ok(())
            }
        }
    }

    fn init_app_data(&mut self) {
        self.directories_selected.clear();
        self.date_type_selected = None;
        self.files_selected.clear();

        self.root.clear_directory_content();
        self.root = Directory::new(None);
        self.path.clear();
        self.update_path_input();
        self.external_storage.clear();
        self.error.clear();
        self.new_directory_name.clear();
        self.checkbox_states = CheckboxStates::default();
    }

    fn search_path(&mut self) -> std::io::Result<()> {
        if self.path_input.is_empty() {
            self.path_input = String::from("/");
        }
        self.write_directories_from_path(&PathBuf::from(&self.path_input))?;
        self.path = PathBuf::from(&self.path_input);
        self.update_path_input();
        Ok(())
    }

    fn move_down_directory(&mut self, directory_name: &OsStr) -> std::io::Result<()> {
        let mut path = PathBuf::from(&self.path);
        path.push(directory_name);
        self.write_directory_to_tree(&path)?;
        self.path = path;
        self.update_path_input();
        Ok(())
    }

    fn move_up_directory(&mut self) {
        let path_before_pop = self.path.as_path().to_path_buf();
        if self.path.pop() {
            if let Some(last) = self.root.get_mut_directory_by_path(&path_before_pop) {
                last.clear_directory_content();
            }
        }
        self.update_path_input();
    }

    fn move_in_external_directory(&mut self, external: &OsStr) -> std::io::Result<()> {
        match std::env::consts::OS {
            "windows" => {
                self.update_path_prefix(external);
                self.write_directory_to_tree(&PathBuf::from(&self.path))?;
                self.update_path_input();
                Ok(())
            }
            "macos" => {
                self.path.clear();
                self.path.push("/Volumes");
                self.write_directory_to_tree(&PathBuf::from(&self.path))?;
                self.path.push(external);
                self.write_directory_to_tree(&PathBuf::from(&self.path))?;
                self.update_path_input();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn insert_directory_path_to_selected(&mut self, path: PathBuf) {
        if self.directories_selected.contains(&path) {
            while let Some(popped) = self.directories_selected.pop() {
                if path == popped {
                    break;
                }
            }
        } else {
            self.directories_selected.push(path);
        }
    }

    fn select_drop_down_directory(
        &mut self,
        path_to_selected_directory: &PathBuf,
    ) -> std::io::Result<()> {
        if path_to_selected_directory == &self.path {
            if let Some(dir) = self
                .root
                .get_mut_directory_by_path(path_to_selected_directory)
            {
                dir.clear_directory_content();
                self.path.pop();
            }
        } else {
            if path_to_selected_directory.components().count() < self.path.components().count() {
                while self.path.pop() {
                    if self.path.components().count()
                        < path_to_selected_directory.components().count()
                    {
                        break;
                    }
                }
                self.update_path_input();
                return Ok(());
            }
            self.write_directory_to_tree(path_to_selected_directory)?;
            self.path = PathBuf::from(path_to_selected_directory);
        }
        self.update_path_input();
        Ok(())
    }

    fn insert_root_directory(&mut self, path: &PathBuf) {
        let mut new_directory = Directory::new(None);
        if let Err(error) = self.root.read_path(&path, &mut new_directory) {
            self.error = error.to_string();
        }
        self.root = new_directory;
        self.path = PathBuf::from(path);
    }

    fn write_directory_to_tree(&mut self, path: &PathBuf) -> std::io::Result<()> {
        let mut new_dir = self.root.clone();
        match new_dir.get_mut_directory_by_path(&path) {
            Some(selected_directory) => {
                if let Err(error) = self.root.read_path(&path, selected_directory) {
                    return Err(error);
                }
                Ok(())
            }
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::NotFound,
                    "Directory not found",
                ));
            }
        }
    }

    fn write_directories_from_path(&mut self, path: &PathBuf) -> std::io::Result<()> {
        let mut path_stack = PathBuf::from("/");
        for component in path.iter() {
            if component == OsString::from("/") {
                continue;
            }
            path_stack.push(OsString::from(component));
            if let Err(error) = self.write_directory_to_tree(&path_stack) {
                if path_stack.pop() {
                    self.path = path_stack;
                }
                return Err(error);
            }
        }
        self.path = path.clone();
        Ok(())
    }

    fn write_selected_directory_recursively(
        &mut self,
        path_stack: &mut PathBuf,
    ) -> std::io::Result<()> {
        if let Some(directory) = self.root.get_mut_directory_by_path(path_stack) {
            let temp = directory.clone(); // Save copy of directory in case of failure
            if let Err(error) = directory.write_directories_recursive(path_stack) {
                *directory = temp;
                return Err(error);
            }
        }
        Ok(())
    }

    fn get_drives_on_windows(&self) -> Vec<OsString> {
        let mut external_storages = Vec::new();
        for letter in 'A'..'Z' {
            let formatted_drive_letter = format!("{}:/", letter);
            if let Ok(_) = read_dir(&formatted_drive_letter) {
                external_storages.push(OsString::from(formatted_drive_letter));
            }
        }
        external_storages
    }

    fn get_volumes_on_macos(&mut self) {
        if let Some(directories) = self.root.get_directories() {
            if let Some(volumes_dir) = directories.get(&OsString::from("Volumes")) {
                if let Some(volumes) = volumes_dir.get_directories() {
                    for key in volumes.keys() {
                        self.external_storage.insert(OsString::from(&key));
                    }
                }
            }
        }
    }

    fn update_path_prefix(&mut self, key: &OsStr) {
        for keys in self.external_storage.iter() {
            if keys == key {
                self.path = PathBuf::from(key);
            }
        }
    }

    fn update_path_input(&mut self) {
        if let Some(path_str) = self.path.to_str() {
            self.path_input = String::from(path_str);
        }
    }

    fn handle_checkbox_error(
        &mut self,
        error: std::io::Error,
        files_selected: BTreeMap<OsString, File>,
    ) -> std::io::Error {
        for (key, value) in files_selected {
            self.files_selected.insert(key, value);
        }
        error
    }

    fn is_duplicate_files_selected(&self) -> std::io::Result<()> {
        let selected_dir = self.root.get_directory_by_path(&self.path);
        if let Some(files) = selected_dir.get_files() {
            for key in files.keys() {
                if self.files_selected.contains_key(key) {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidInput,
                        "Duplicate file names found",
                    ));
                }
            }
        }
        Ok(())
    }

    fn is_directory_creation_valid(&self) -> std::io::Result<()> {
        if self.files_selected.is_empty() {
            return Err(std::io::Error::new(
                ErrorKind::NotFound,
                "No files selected.",
            ));
        }
        if self.new_directory_name.is_empty() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Directory name not specified.",
            ));
        }
        Ok(())
    }

    fn create_directory_with_selected_files(
        &mut self,
        files_selected: BTreeMap<OsString, File>,
    ) -> std::io::Result<()> {
        if let Some(selected_directory) = self.root.get_mut_directory_by_path(&self.path) {
            if let Some(directories) = selected_directory.get_directories() {
                if !organizing::is_directory_name_unique(&self.new_directory_name, directories) {
                    return Err(self.handle_checkbox_error(
                        std::io::Error::new(
                            ErrorKind::AlreadyExists,
                            "Directory name already exists.",
                        ),
                        files_selected,
                    ));
                }
            }

            let CheckboxStates {
                organize_by_filetype,
                organize_by_date,
                insert_date_to_file_name,
                insert_directory_name_to_file_name,
                remove_uppercase,
                replace_spaces_with_underscores,
                use_only_ascii,
            } = self.checkbox_states;

            // In case of an error, put files_selected back to self
            let temp_files_selected = files_selected.clone();

            let data = OrganizingData {
                files_selected,
                insert_directory_name_to_file_name,
                insert_date_to_file_name,
                remove_uppercase,
                replace_spaces_with_underscores,
                use_only_ascii,
                new_directory_name: &self.new_directory_name,
                date_type: self.date_type_selected,
            };

            // Write directory path and checkbox states to a file

            // If both organize_by_file_type and date are checked
            if organize_by_filetype && organize_by_date {
                match organize_files_by_file_type_and_date(data) {
                    Ok(directories_by_file_type_and_date) => {
                        selected_directory.insert_new_sub_directory(
                            &self.new_directory_name,
                            directories_by_file_type_and_date,
                        );
                        return Ok(());
                    }
                    Err(error) => {
                        return Err(self.handle_checkbox_error(error, temp_files_selected))
                    }
                }
            } else if organize_by_filetype {
                match organize_by_file_type(data) {
                    Ok(directories_by_file_type) => {
                        selected_directory.insert_new_sub_directory(
                            &self.new_directory_name,
                            directories_by_file_type,
                        );
                        return Ok(());
                    }
                    Err(error) => {
                        return Err(self.handle_checkbox_error(error, temp_files_selected))
                    }
                }
            } else if organize_by_date {
                // If only organize_by_date is checked
                match organize_to_directories_by_date(data) {
                    Ok(directories_by_date) => {
                        selected_directory.insert_new_sub_directory(
                            &self.new_directory_name,
                            directories_by_date,
                        );
                        return Ok(());
                    }
                    Err(error) => {
                        return Err(self.handle_checkbox_error(error, temp_files_selected))
                    }
                }
            } else if insert_directory_name_to_file_name
                || insert_date_to_file_name
                || remove_uppercase
                || replace_spaces_with_underscores
                || use_only_ascii
            {
                match rename_and_organize_to_directory(data) {
                    Ok(new_directory) => {
                        selected_directory
                            .insert_directory(new_directory, &self.new_directory_name);
                        return Ok(());
                    }
                    Err(error) => {
                        return Err(self.handle_checkbox_error(error, temp_files_selected))
                    }
                }
            } else if !organize_by_filetype
                && !organize_by_filetype
                && !insert_date_to_file_name
                && !insert_directory_name_to_file_name
                && !remove_uppercase
                && !replace_spaces_with_underscores
                && !use_only_ascii
            {
                // If none are checked
                let mut new_directory = Directory::new(None);
                for (key, value) in data.files_selected {
                    new_directory.insert_file(key, value);
                }
                selected_directory.insert_directory(new_directory, &self.new_directory_name);
                return Ok(());
            }
            return Ok(());
        }
        Err(std::io::Error::new(
            ErrorKind::NotFound,
            "No directory found with specified path",
        ))
    }

    fn rename_files_without_directory(
        &mut self,
        insert_date_to_file_name: bool,
        remove_uppercase: bool,
        replace_spaces_with_underscores: bool,
        use_only_ascii: bool,
        date_type: Option<DateType>,
    ) -> std::io::Result<()> {
        if let Some(selected_dir) = self.root.get_mut_directory_by_path(&self.path) {
            while let Some((key, value)) = self.files_selected.pop_last() {
                if let Some(file_name) = key.to_str() {
                    let mut renamed_file_name = String::new();
                    organizing::rename_file_name(
                        &mut renamed_file_name,
                        insert_date_to_file_name,
                        false,
                        remove_uppercase,
                        replace_spaces_with_underscores,
                        use_only_ascii,
                        &self.new_directory_name,
                        file_name,
                        &value,
                        date_type,
                    );
                    selected_dir.insert_file(OsString::from(renamed_file_name), value);
                }
            }
            return Ok(());
        }
        Err(std::io::Error::new(
            ErrorKind::NotFound,
            "No directory found in specified path",
        ))
    }

    fn toggle_checkbox(&mut self, toggle: bool, id: usize) {
        match id {
            1 => {
                self.checkbox_states.organize_by_filetype = toggle;
            }
            2 => {
                self.checkbox_states.organize_by_date = toggle;
            }
            3 => {
                self.checkbox_states.insert_date_to_file_name = toggle;
            }
            4 => {
                self.checkbox_states.insert_directory_name_to_file_name = toggle;
            }
            5 => {
                self.checkbox_states.remove_uppercase = toggle;
            }
            6 => {
                self.checkbox_states.replace_spaces_with_underscores = toggle;
            }
            7 => {
                self.checkbox_states.use_only_ascii = toggle;
            }
            _ => {}
        }
    }

    fn extract_content_from_directory(
        &mut self,
        path_to_selected_directory: &mut PathBuf,
        path_to_parent_directory: &PathBuf,
    ) -> std::io::Result<()> {
        let selected_dir = self.root.get_directory_by_path(&path_to_selected_directory);
        let parent_dir = self.root.get_directory_by_path(&path_to_parent_directory);

        if directories_have_duplicate_directories(parent_dir, selected_dir)
            || directories_have_duplicate_files(parent_dir, selected_dir)
        {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "No duplicates allowed in same directory",
            ));
        }
        let mut files_holder = BTreeMap::new();
        let mut directories_holder = BTreeMap::new();
        self.insert_content_from_selected(
            &mut directories_holder,
            &mut files_holder,
            &path_to_selected_directory,
        )?;

        self.place_files_to_parent_directory(
            directories_holder,
            files_holder,
            path_to_parent_directory,
        )?;

        if let Some(last) = path_to_selected_directory.iter().last() {
            self.remove_directories_from_extracted_dir(last, path_to_parent_directory)?;
            self.directories_selected.pop();
        }

        Ok(())
    }

    fn insert_content_from_selected(
        &mut self,
        directories_holder: &mut BTreeMap<OsString, Directory>,
        files_holder: &mut BTreeMap<OsString, File>,
        path_to_selected_directory: &PathBuf,
    ) -> std::io::Result<()> {
        match self
            .root
            .get_mut_directory_by_path(path_to_selected_directory)
        {
            Some(selected_dir) => {
                if let Some(files) = selected_dir.get_mut_files().take() {
                    for (key, value) in files {
                        files_holder.insert(key, value);
                    }
                }
                if let Some(directories) = selected_dir.get_mut_directories().take() {
                    for (key, value) in directories {
                        directories_holder.insert(key, value);
                    }
                }
                Ok(())
            }
            None => Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Selected directory path didn't have results",
            )),
        }
    }

    fn place_files_to_parent_directory(
        &mut self,
        directories_holder: BTreeMap<OsString, Directory>,
        files_holder: BTreeMap<OsString, File>,
        path_to_parent_directory: &PathBuf,
    ) -> std::io::Result<()> {
        match self
            .root
            .get_mut_directory_by_path(&path_to_parent_directory)
        {
            Some(parent_dir) => {
                for (file_name, file) in files_holder {
                    parent_dir.insert_file(file_name, file);
                }
                for (dir_name, directory) in directories_holder {
                    if let Some(directory_name_str) = dir_name.to_str() {
                        parent_dir.insert_directory(directory, directory_name_str);
                    }
                }
                Ok(())
            }
            None => Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Parent directory not found",
            )),
        }
    }

    fn remove_directories_from_extracted_dir(
        &mut self,
        selected_directory_name: &OsStr,
        path_to_parent_directory: &PathBuf,
    ) -> std::io::Result<()> {
        match self
            .root
            .get_mut_directory_by_path(path_to_parent_directory)
        {
            Some(parent_dir) => {
                parent_dir.remove_sub_directory(selected_directory_name);
                Ok(())
            }
            None => Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Parent directory not found",
            )),
        }
    }

    fn extract_all_files_from_directory(
        &mut self,
        path_to_parent_directory: &PathBuf,
        path_to_selected_directory: &mut PathBuf,
    ) -> std::io::Result<()> {
        let mut files_holder = BTreeMap::new();
        if let Some(parent_dir) = self
            .root
            .get_mut_directory_by_path(path_to_parent_directory)
        {
            if let Some(files) = parent_dir.get_mut_files().take() {
                for (key, value) in files {
                    files_holder.insert(key, value);
                }
            }
        }
        let mut error_container = None;
        if let Some(selected_dir) = self
            .root
            .get_mut_directory_by_path(&path_to_selected_directory)
        {
            if let Err(error) =
                selected_dir.get_files_recursive(&mut files_holder, path_to_selected_directory)
            {
                error_container = Some(error);
            }
        }
        if let Some(parent_dir) = self
            .root
            .get_mut_directory_by_path(&path_to_parent_directory)
        {
            for (key, value) in files_holder {
                parent_dir.insert_file(key, value);
            }
        }

        match error_container {
            Some(error) => Err(error),
            None => {
                if let Some(parent_dir) = self
                    .root
                    .get_mut_directory_by_path(&path_to_parent_directory)
                {
                    if let Some(last) = path_to_selected_directory.iter().last() {
                        parent_dir.remove_sub_directory(last);
                        self.directories_selected.clear();
                        return Ok(());
                    } else {
                        return Err(std::io::Error::new(
                            ErrorKind::NotFound,
                            "Not selected_directory found",
                        ));
                    }
                }
                return Err(std::io::Error::new(
                    ErrorKind::NotFound,
                    "Parent directory not found",
                ));
            }
        }
    }
}

fn select_file(
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

fn directories_have_duplicate_directories(
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

fn directories_have_duplicate_files(parent_dir: &Directory, selected_dir: &Directory) -> bool {
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

fn are_paths_equal(path1: &PathBuf, path2: &PathBuf) -> bool {
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

pub struct OrganizingData<'a> {
    pub files_selected: BTreeMap<OsString, File>,
    pub insert_directory_name_to_file_name: bool,
    pub insert_date_to_file_name: bool,
    pub remove_uppercase: bool,
    pub replace_spaces_with_underscores: bool,
    pub use_only_ascii: bool,
    pub new_directory_name: &'a str,
    pub date_type: Option<DateType>,
}

fn organize_files_by_file_type_and_date(
    data: OrganizingData,
) -> std::io::Result<BTreeMap<OsString, Directory>> {
    if let Some(date_type_selected) = data.date_type {
        let mut directories_by_file_type = organizing::sort_files_by_file_type(
            data.files_selected,
            data.insert_directory_name_to_file_name,
            data.insert_date_to_file_name,
            data.remove_uppercase,
            data.replace_spaces_with_underscores,
            data.use_only_ascii,
            data.new_directory_name,
            Some(date_type_selected),
        );

        for (_, directory) in &mut directories_by_file_type {
            if let Some(files) = directory.get_mut_files().take() {
                let directories_by_date = organizing::sort_files_by_date(
                    files,
                    false,
                    false,
                    data.remove_uppercase,
                    data.replace_spaces_with_underscores,
                    data.use_only_ascii,
                    data.new_directory_name,
                    date_type_selected,
                );
                directory.insert_directories(directories_by_date);
            }
        }
        Ok(directories_by_file_type)
    } else {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Date type not specified.",
        ));
    }
}

fn organize_by_file_type(data: OrganizingData) -> std::io::Result<BTreeMap<OsString, Directory>> {
    if let None = data.date_type {
        if data.insert_date_to_file_name {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Date type not specified.",
            ));
        }
    }
    let file_type_directories = organizing::sort_files_by_file_type(
        data.files_selected,
        data.insert_directory_name_to_file_name,
        data.insert_date_to_file_name,
        data.remove_uppercase,
        data.replace_spaces_with_underscores,
        data.use_only_ascii,
        data.new_directory_name,
        data.date_type,
    );
    Ok(file_type_directories)
}

fn organize_to_directories_by_date(
    data: OrganizingData,
) -> std::io::Result<BTreeMap<OsString, Directory>> {
    if let Some(date_type) = data.date_type {
        let directories_by_date = organizing::sort_files_by_date(
            data.files_selected.clone(),
            data.insert_directory_name_to_file_name,
            data.insert_date_to_file_name,
            data.remove_uppercase,
            data.replace_spaces_with_underscores,
            data.use_only_ascii,
            data.new_directory_name,
            date_type,
        );
        Ok(directories_by_date)
    } else {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Date type not specified.",
        ));
    }
}

fn rename_and_organize_to_directory(data: OrganizingData) -> std::io::Result<Directory> {
    if let None = data.date_type {
        if data.insert_date_to_file_name {
            return Err(std::io::Error::new(
                ErrorKind::NotFound,
                "No date type specified",
            ));
        }
    }
    // If only renaming are checked
    let mut new_directory = Directory::new(None);
    for (key, file) in data.files_selected {
        if let Some(file_name) = key.to_str() {
            let mut renamed_file_name = String::new();
            organizing::rename_file_name(
                &mut renamed_file_name,
                data.insert_date_to_file_name,
                data.insert_directory_name_to_file_name,
                data.remove_uppercase,
                data.replace_spaces_with_underscores,
                data.use_only_ascii,
                data.new_directory_name,
                file_name,
                &file,
                data.date_type,
            );
            new_directory.insert_file(OsString::from(renamed_file_name), file);
        }
    }
    Ok(new_directory)
}

mod save_directory {
    use crate::{layouts::CheckboxStates, metadata::DateType};
    use std::{
        io::{Read, Write},
        path::PathBuf,
    };
    const SAVE_FILE_LOCATION: &str =
        "/Users/vernerikankaanpaa/RustProjects/filerganizer/.save_file.txt";
    pub fn write_created_directory_to_save_file(
        path: PathBuf,
        checkbox_states: CheckboxStates,
        date_type: Option<DateType>,
    ) -> std::io::Result<()> {
        match std::fs::File::options()
            .append(true)
            .open(SAVE_FILE_LOCATION)
        {
            Ok(mut file) => {
                // Add to existing file
                if let Some(dir_path) = path.to_str() {
                    let mut file_content = String::new();
                    write_directory_data_to_string(
                        &mut file_content,
                        dir_path,
                        checkbox_states,
                        date_type,
                    );
                    file.write(file_content.as_bytes())?;
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Could not convert path to string.",
                    ));
                }
            }
            Err(_) => {
                // Create file
                let mut save_file = create_save_file()?;
                if let Some(dir_path) = path.to_str() {
                    let mut file_content = String::new();
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

    pub fn remove_directory_from_file(path_to_extracted_dir: PathBuf) -> std::io::Result<()> {
        match std::fs::File::options().read(true).open(SAVE_FILE_LOCATION) {
            Ok(mut file) => {
                let mut file_content = String::new();
                file.read_to_string(&mut file_content)?;
                for line in file_content.lines() {
                    let line_path = PathBuf::from(line);
                    if line_path == path_to_extracted_dir {
                        println!("Line found: {line}");
                    }
                }
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn create_save_file() -> std::io::Result<std::fs::File> {
        match std::fs::File::create(SAVE_FILE_LOCATION) {
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
        file_content.push_str("\n");

        let formatted = format!(
            "organize_by_file_type: {}\n",
            checkbox_states.organize_by_filetype
        );
        file_content.push_str(&formatted);

        let formatted = format!("organize_by_date: {}\n", checkbox_states.organize_by_date);
        file_content.push_str(&formatted);

        let formatted = format!(
            "insert_date_to_file_name: {}\n",
            checkbox_states.insert_date_to_file_name
        );
        file_content.push_str(&formatted);

        let formatted = format!(
            "insert_directory_name_to_file_name: {}\n",
            checkbox_states.insert_directory_name_to_file_name
        );
        file_content.push_str(&formatted);

        let formatted = format!("remove_uppercase: {}\n", checkbox_states.remove_uppercase);
        file_content.push_str(&formatted);

        let formatted = format!(
            "replace_spaces_with_underscores: {}\n",
            checkbox_states.replace_spaces_with_underscores
        );
        file_content.push_str(&formatted);

        let formatted = format!("use_only_ascii: {}\n", checkbox_states.use_only_ascii);
        file_content.push_str(&formatted);

        if let Some(date_type) = date_type {
            match date_type {
                DateType::Created => {
                    let formatted = String::from("date_type: Created\n");
                    file_content.push_str(&formatted);
                }
                DateType::Accessed => {
                    let formatted = String::from("date_type: Accessed\n");
                    file_content.push_str(&formatted);
                }
                DateType::Modified => {
                    let formatted = String::from("date_type: Modified\n");
                    file_content.push_str(&formatted);
                }
            }
        } else {
            let formatted = String::from("date_type: None\n");
            file_content.push_str(&formatted);
        }
    }
}
