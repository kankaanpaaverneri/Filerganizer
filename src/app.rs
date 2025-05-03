use iced::widget::Container;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::read_dir;
use std::path::PathBuf;

use crate::directory::Directory;
use crate::layouts::Layout;

pub struct App {
    path: PathBuf,
    path_input: String,
    error: String,
    root: Directory,
    external_storage: HashMap<OsString, Directory>,
    layout: Layout,
}


impl Default for App {
    fn default() -> Self {
        App {
            path: PathBuf::new(),
            path_input: String::new(),
            error: String::new(),
            root: Directory::new(None),
            external_storage: HashMap::new(),
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
                self.write_directory_to_tree(&mut path);
            }
            Message::MoveUpDirectory => {
                let path_before_pop = self.path.as_path().to_path_buf();
                if self.path.pop() {
                    if let Some(last) = self.root.get_mut_directory_by_path(&path_before_pop) {
                        last.clear_directory_content();
                    }
                }
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

    fn switch_layout(&mut self, layout: Layout) {
        self.layout = layout;
        match self.layout {
            Layout::DirectoryExploringLayout => {
                if let Some(first) = self.get_drive_paths().first() {
                    let mut new_directory = Directory::new(None);
                    let path = PathBuf::from(first);
                    if let Err(error) = self.root.read_path(&path, &mut new_directory) {
                        self.error = error.to_string();
                    }
                    self.root = new_directory;
                    self.path = path;
                    self.write_directories_from_path();
                }
            }
            Layout::Main => {
                self.root.clear_directory_content();
                self.root = Directory::new(None);
                self.path.clear();
            }
        }
    }

    fn write_directory_to_tree(&mut self, path: &mut PathBuf) {
        let mut new_dir = self.root.clone();
        match new_dir.get_mut_directory_by_path(&path) {
            Some(selected_directory) => {
                if let Err(error) = self.root.read_path(&path, selected_directory) {
                    self.error = error.to_string();
                    return;
                }

                self.path = PathBuf::from(path.as_os_str());
            }
            None => self.error = String::from("Directory not found"),
        }
    }

    fn write_directories_from_path(&mut self) {
        if let Some(first) = self.get_drive_paths().first(){
            let mut path_stack = PathBuf::from(&first);
            for (i, path_directory) in PathBuf::from(first).iter().enumerate() {
                if i == 0 {
                    continue;
                }
                path_stack.push(path_directory);
                self.write_directory_to_tree(&mut PathBuf::from(&path_stack));
            }
            
        }
    }

    fn get_drive_paths(&self) -> Vec<String> {
        match std::env::consts::OS {
            "windows" => {
                self.get_drives()
            },
            _ => Vec::with_capacity(0)
        }
    }

    fn get_drives(&self) -> Vec<String> {
        let mut external_storages = Vec::new();
        for letter in 'A'..'Z' {
            let formatted_drive_letter = format!("{}:/", letter);
            if let Ok(_) = read_dir(&formatted_drive_letter) {
                
                external_storages.push(formatted_drive_letter);
            }
        }
        external_storages
    }
}
