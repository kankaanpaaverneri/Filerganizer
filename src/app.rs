use iced::widget::Container;
use iced::Task;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs::read_dir;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::directory::Directory;
use crate::file::File;
use crate::layouts::{CheckboxStates, DirectoryView, Layout};

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
    SelectFile(OsString, File),
    InputNewDirectoryName(String),
    CreateDirectoryWithSelectedFiles,
    CheckboxToggled(bool, usize),
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
            Message::SelectPath => {
                if let Err(error) = self.switch_layout(&Layout::DirectoryOrganizingLayout) {
                    self.error = error.to_string();
                }
                Task::none()
            }
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
            Message::SelectFile(file_name, file) => {
                if let Some(_) = self.files_selected.get(&file_name) {
                    self.files_selected.remove(&file_name);
                } else {
                    self.files_selected.insert(file_name, file);
                }
                Task::none()
            }

            Message::InputNewDirectoryName(input) => {
                self.new_directory_name = input;
                Task::none()
            }
            Message::CreateDirectoryWithSelectedFiles => {
                if self.files_selected.is_empty() {
                    return Task::none();
                }
                if let Some(selected_directory) = self.root.get_mut_directory_by_path(&self.path) {
                    // Copy selected files to new sub directory
                    selected_directory.insert_new_sub_directory(
                        &self.new_directory_name,
                        self.files_selected.clone(),
                    );
                    // Remove old files from this directory
                    if let Some(files) = selected_directory.get_mut_files() {
                        for (key, _file) in &self.files_selected {
                            if files.contains_key(key) {
                                files.remove(key);
                            }
                        }
                    }
                    self.files_selected.clear(); // Clear selection
                    self.new_directory_name.clear(); // Clear input field
                }

                Task::none()
            }
            Message::CheckboxToggled(toggle, id) => match id {
                1 => {
                    self.checkbox_states.organize_by_filetype = toggle;
                    return Task::none();
                }
                2 => {
                    self.checkbox_states.organize_by_date = toggle;
                    return Task::none();
                }
                3 => {
                    self.checkbox_states.insert_date_to_file_name = toggle;
                    return Task::none();
                }
                4 => {
                    self.checkbox_states.insert_directory_name_to_file_name = toggle;
                    return Task::none();
                }
                _ => Task::none(),
            },
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
                self.root.clear_directory_content();
                self.root = Directory::new(None);
                self.path.clear();
                self.path_input.clear();
                self.external_storage.clear();
                self.layout = Layout::Main;
                Ok(())
            }
            Layout::DirectoryOrganizingLayout => {
                let mut path = PathBuf::from(&self.path);
                if let Err(error) = self.write_selected_directory_recursively(&mut path) {
                    self.error = error.to_string();
                }
                self.layout = Layout::DirectoryOrganizingLayout;
                Ok(())
            }
        }
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
        if let Some(last) = path_to_selected_directory.iter().last() {
            if self.is_directory_name_in_path(last) {
                if are_paths_equal(&self.path, path_to_selected_directory) {
                    self.clear_directories_by_path(last);
                } else {
                    self.drop_down_directory(path_to_selected_directory, last)?;
                }
            } else {
                self.drop_down_directory(path_to_selected_directory, last)?;
            }
        }
        Ok(())
    }

    fn drop_down_directory(
        &mut self,
        path_to_selected_directory: &PathBuf,
        last: &OsStr,
    ) -> std::io::Result<()> {
        if !are_paths_equal(&self.path, path_to_selected_directory) {
            self.path = PathBuf::from(path_to_selected_directory);
        } else {
            self.path.push(last);
        }
        if let Err(error) = self.write_directory_to_tree(&PathBuf::from(&self.path)) {
            if let ErrorKind::NotFound = error.kind() {
                self.find_directory_from_parents(last)?;
                self.update_path_input();
                return Ok(());
            }
            self.path.pop();
            return Err(error);
        }
        self.update_path_input();
        Ok(())
    }

    fn find_directory_from_parents(&mut self, directory_name: &OsStr) -> std::io::Result<()> {
        let mut path_stack = PathBuf::new();
        let original_path = PathBuf::from(&self.path);
        for path_directory in &original_path {
            path_stack.push(path_directory);
            if let Some(current_directory) = self.root.get_mut_directory_by_path(&path_stack) {
                if let Some(sub_directories) = current_directory.get_directories() {
                    if let Some(_) = sub_directories.get(directory_name) {
                        path_stack.push(directory_name);
                        self.write_directory_to_tree(&PathBuf::from(&path_stack))?;
                        self.path = PathBuf::from(&path_stack);
                        self.update_path_input();
                    }
                }
            }
        }
        Ok(())
    }

    fn clear_directories_by_path(&mut self, selected_directory: &OsStr) {
        while let Some(last_directory) = self.root.get_mut_directory_by_path(&self.path) {
            last_directory.clear_directory_content();
            if let Some(last) = self.path.iter().last() {
                if last == selected_directory {
                    self.path.pop();
                    self.update_path_input();
                    break;
                }
            }
            self.path.pop();
            self.update_path_input();
        }
    }

    fn is_directory_name_in_path(&self, directory_name: &OsStr) -> bool {
        for path_directory in &self.path {
            if directory_name == path_directory {
                return true;
            }
        }
        false
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
            if let Err(err) = directory.write_directories_recursive(path_stack) {
                return Err(err);
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
