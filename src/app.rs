use iced::widget::Container;
use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fs::read_dir;
use std::path::PathBuf;

use crate::directory::Directory;
use crate::layouts::Layout;

pub struct App {
    path: PathBuf,
    path_input: String,
    error: String,
    root: Directory,
    external_storage: BTreeSet<OsString>,
    layout: Layout,
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
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchLayout(Layout),
    TextInput(String),
    MoveDownDirectory(OsString),
    MoveUpDirectory,
    MoveInExternalDirectory(OsString),
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
                self.switch_layout(layout);
            }
            Message::TextInput(text_input) => {
                self.path_input = text_input;
            }
            Message::MoveDownDirectory(directory_name) => {
                let mut path = self.path.as_path().to_path_buf();
                
                path.push(directory_name.as_os_str());
                self.write_directory_to_tree(&path);
                self.path = path;
                
            }
            Message::MoveUpDirectory => {
                let path_before_pop = self.path.as_path().to_path_buf();
                if self.path.pop() {
                    if let Some(last) = self.root.get_mut_directory_by_path(&path_before_pop) {
                        last.clear_directory_content();
                    }
                }
            }
            Message::MoveInExternalDirectory(external) => match std::env::consts::OS {
                "windows" => {
                    self.update_path_prefix(&external);
                    self.write_directory_to_tree(&PathBuf::from(&self.path));
                }
                "macos" => {
                    self.path.clear();
                    self.path.push("/Volumes");
                    self.write_directory_to_tree(&PathBuf::from(&self.path));
                    self.path.push(&external);
                    self.write_directory_to_tree(&PathBuf::from(&self.path));
                }
                _ => {}
            },
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

    fn switch_layout(&mut self, layout: Layout) {
        self.layout = layout;
        match self.layout {
            Layout::DirectoryExploringLayout => match std::env::consts::OS {
                "windows" => {
                    
                    if let Some(first) = self.get_drives_on_windows().first() {
                        let path = PathBuf::from(first);
                        for path in self.get_drives_on_windows() {
                            self.external_storage.insert(path);
                        }
                        self.insert_root_directory(&path);
                        
                    }
                }
                _ => {
                    let path = PathBuf::from("/");
                    self.insert_root_directory(&path);
                    self.write_directory_to_tree(&path);
                    self.get_volumes_on_macos();
                }
            },
            Layout::Main => {
                self.root.clear_directory_content();
                self.root = Directory::new(None);
                self.path.clear();
                self.external_storage.clear();
            }
        }
    }

    fn insert_root_directory(&mut self, path: &PathBuf) {
        let mut new_directory = Directory::new(None);
        if let Err(error) = self.root.read_path(&path, &mut new_directory) {
            self.error = error.to_string();
        }
        self.root = new_directory;
        self.path = PathBuf::from(path);
    }

    fn write_directory_to_tree(&mut self, path: &PathBuf) {
        let mut new_dir = self.root.clone();
        match new_dir.get_mut_directory_by_path(&path) {
            Some(selected_directory) => {
                if let Err(error) = self.root.read_path(&path, selected_directory) {
                    self.error = error.to_string();
                    return;
                }
            }
            None => self.error = String::from("Directory not found"),
        }
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
}
