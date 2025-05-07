use iced::widget::Container;
use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fs::read_dir;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::directory::Directory;
use crate::layouts::{DirectoryView, Layout};

pub struct App {
    path: PathBuf,
    path_input: String,
    error: String,
    root: Directory,
    external_storage: BTreeSet<OsString>,
    layout: Layout,
    directory_view: DirectoryView,
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
    Exit,
}

impl App {
    pub fn view(&self) -> Container<Message> {
        self.layout.get_layout(self)
    }

    pub fn update(&mut self, message: Message) {
        self.error.clear();
        match message {
            Message::SwitchLayout(layout) => {
                if let Err(error) = self.switch_layout(layout) {
                    self.error = error.to_string();
                }
            }
            Message::TextInput(text_input) => self.path_input = text_input,
            Message::SearchPath => {
                if let Err(error) = self.search_path() {
                    self.error = error.to_string();
                }
            }
            Message::MoveDownDirectory(directory_name) => {
                if let Err(error) = self.move_down_directory(&directory_name) {
                    self.error = error.to_string();
                }
            }
            Message::MoveUpDirectory => self.move_up_directory(),
            Message::MoveInExternalDirectory(external) => {
                if let Err(error) = self.move_in_external_directory(&external) {
                    self.error = error.to_string();
                }
            }
            Message::DropDownDirectory(directory_name) => {
                if let Err(error) = self.select_drop_down_directory(&directory_name) {
                    self.error = error.to_string();
                }
            }
            Message::SwitchDirectoryView(directory_view) => match directory_view {
                DirectoryView::List => {
                    if let DirectoryView::DropDown = self.directory_view {
                        self.directory_view = directory_view;
                    }
                }
                DirectoryView::DropDown => {
                    if let DirectoryView::List = self.directory_view {
                        self.directory_view = directory_view;
                    }
                }
            },
            Message::SelectPath => {
                println!("Selected path: {:?}", self.path);
            }
            Message::Exit => std::process::exit(0),
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

    fn switch_layout(&mut self, layout: Layout) -> std::io::Result<()> {
        match layout {
            Layout::DirectoryExploringLayout => match std::env::consts::OS {
                "windows" => {
                    if let Some(first) = self.get_drives_on_windows().first() {
                        let path = PathBuf::from(first);
                        for path in self.get_drives_on_windows() {
                            self.external_storage.insert(path);
                        }
                        self.insert_root_directory(&path);
                        self.update_path_input();
                    }
                    self.layout = Layout::DirectoryExploringLayout;
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
                    self.layout = Layout::DirectoryExploringLayout;
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

    fn select_drop_down_directory(
        &mut self,
        path_to_selected_directory: &PathBuf,
    ) -> std::io::Result<()> {
        if let Some(last) = path_to_selected_directory.iter().last() {
            if self.is_directory_name_in_path(last) {
                if self.are_paths_equal(path_to_selected_directory) {
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

    fn are_paths_equal(&self, path_to_selected_directory: &PathBuf) -> bool {
        let mut components = path_to_selected_directory.components();
        for current_path in self.path.iter() {
            if let Some(component) = components.next() {
                if component.as_os_str() != current_path {
                    return false;
                }
            }
        }
        true
    }

    fn drop_down_directory(
        &mut self,
        path_to_selected_directory: &PathBuf,
        last: &OsStr,
    ) -> std::io::Result<()> {
        if !self.are_paths_equal(path_to_selected_directory) {
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
