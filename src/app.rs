use iced::widget::Container;
use iced::Task;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs::read_dir;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::usize;

use crate::app_util::convert_os_str_to_str;
use crate::directory::Directory;
use crate::file::File;
use crate::filesystem;
use crate::layouts::{
    CheckboxStates, DirectoryView, FileSelectedLocation, IndexPosition, Layout, ReplaceWith,
    Replaceable,
};
use crate::metadata::DateType;
use crate::organize_files;
use crate::save_directory;
use crate::save_directory::SAVE_FILE_NAME;
use crate::{app_util, directory};

pub struct App {
    home_directory_path: PathBuf,
    path: PathBuf,
    path_input: String,
    path_input_id: iced::widget::text_input::Id,
    error: String,
    root: Directory,
    external_storage: BTreeSet<OsString>,
    layout: Layout,
    directory_view: DirectoryView,

    directories_selected: HashSet<PathBuf>,
    directory_selected: Option<PathBuf>,
    selected_directory_rules: Option<SelectedDirectoryRules>,

    multiple_selection: MultipleSelection,
    files_selected: BTreeMap<OsString, File>,
    new_directory_name: String,
    checkbox_states: CheckboxStates,
    replaceable_options: Vec<Replaceable>,
    replace_with_options: [ReplaceWith; 2],
    replaceables: Vec<ReplacableSelection>,
    date_type_selected: Option<DateType>,
    filename_input: String,
    order_of_filename_components: Vec<String>,
    index_position: Option<IndexPosition>,
    files_organized: BTreeMap<OsString, File>,
    files_have_been_organized: bool,
}

#[derive(Debug)]
pub struct SelectedDirectoryRules {
    checkbox_states: CheckboxStates,
    replaceables: Vec<ReplacableSelection>,
    date_type_selected: Option<DateType>,
    order_of_filename_components: Vec<String>,
    index_position: Option<IndexPosition>,
    filename_input: String,
}

impl SelectedDirectoryRules {
    fn from(
        checkbox_states: CheckboxStates,
        replaceables: Vec<ReplacableSelection>,
        date_type_selected: Option<DateType>,
        order_of_filename_components: Vec<String>,
        index_position: Option<IndexPosition>,
        filename_input: String,
    ) -> Self {
        Self {
            checkbox_states,
            replaceables,
            date_type_selected,
            order_of_filename_components,
            index_position,
            filename_input,
        }
    }

    pub fn get_checkbox_states(&self) -> &CheckboxStates {
        &self.checkbox_states
    }

    pub fn get_replaceables(&self) -> &Vec<ReplacableSelection> {
        &self.replaceables
    }

    pub fn get_date_type_selected(&self) -> &Option<DateType> {
        &self.date_type_selected
    }

    pub fn get_order_of_filename_components(&self) -> &Vec<String> {
        &self.order_of_filename_components
    }

    pub fn get_index_position(&self) -> &Option<IndexPosition> {
        &self.index_position
    }

    pub fn get_custom_filename(&self) -> &str {
        &self.filename_input.as_str()
    }
}

#[derive(Debug)]
pub struct ReplacableSelection {
    replaceable_selected: Option<Replaceable>,
    replace_with_selected: Option<ReplaceWith>,
}

impl ReplacableSelection {
    pub fn new() -> Self {
        Self {
            replace_with_selected: Some(ReplaceWith::Nothing),
            replaceable_selected: None,
        }
    }

    pub fn from(replaceable: Option<Replaceable>, replace_with: Option<ReplaceWith>) -> Self {
        Self {
            replaceable_selected: replaceable,
            replace_with_selected: replace_with,
        }
    }

    pub fn get_replaceable_selected(&self) -> Option<Replaceable> {
        self.replaceable_selected
    }

    pub fn get_replace_with_selected(&self) -> Option<ReplaceWith> {
        self.replace_with_selected
    }
}

struct MultipleSelection {
    file_name: String,
    file_index: usize,
}

impl MultipleSelection {
    fn new() -> Self {
        MultipleSelection {
            file_name: String::new(),
            file_index: 0,
        }
    }
}

pub mod filename_components {
    pub const DATE: &str = "Date";
    pub const ORIGINAL_FILENAME: &str = "Original filename";
    pub const DIRECTORY_NAME: &str = "Directory name";
    pub const CUSTOM_FILE_NAME: &str = "Custom filename";
}

impl Default for App {
    fn default() -> Self {
        App {
            home_directory_path: PathBuf::default(),
            path: PathBuf::new(),
            path_input: String::new(),
            path_input_id: iced::widget::text_input::Id::unique(),
            error: String::new(),
            root: Directory::new(None),
            external_storage: BTreeSet::new(),
            layout: Layout::Main,
            directory_view: DirectoryView::List,

            directories_selected: HashSet::new(),
            directory_selected: None,
            selected_directory_rules: None,
            multiple_selection: MultipleSelection::new(),
            files_selected: BTreeMap::new(),
            new_directory_name: String::new(),
            checkbox_states: CheckboxStates::default(),
            replaceable_options: vec![Replaceable::Dash, Replaceable::Space, Replaceable::Comma],
            replace_with_options: [ReplaceWith::Underscore, ReplaceWith::Nothing],
            replaceables: Vec::new(),
            date_type_selected: None,
            filename_input: String::new(),
            order_of_filename_components: Vec::new(),
            index_position: None,
            files_organized: BTreeMap::new(),
            files_have_been_organized: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchLayout(Layout),
    SwitchDirectoryView(DirectoryView),
    TextInput(String),
    SearchPath(bool),
    MoveInExternalDirectory(OsString),
    DropDownDirectory(PathBuf),

    SelectPath,
    SelectDirectory(PathBuf),
    SelectFile(FileSelectedLocation),
    SelectMultipleFiles(usize, FileSelectedLocation),
    InputNewDirectoryName(String),
    CreateDirectoryWithSelectedFiles,
    RenameFiles,
    CheckboxToggled(bool, usize),
    SelectReplaceable(Replaceable, usize),
    SelectReplaceWith(ReplaceWith, usize),
    AddNewReplaceable,
    RemoveReplaceable(usize),
    DateTypeSelected(DateType),
    InsertFilesToSelectedDirectory,
    SwapFileNameComponents(usize),
    FilenameInput(String),
    IndexPositionSelected(IndexPosition),
    Commit,
    TabKeyPressed,
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
                self.init_app_data();
                if let Err(error) = self.switch_layout(&layout) {
                    self.error = error.to_string();
                }
                Task::none()
            }
            Message::TextInput(text_input) => {
                self.path_input = text_input;
                Task::none()
            }
            Message::SearchPath(is_submit) => {
                if let Err(error) = self.search_path() {
                    self.error = error.to_string();
                }
                if is_submit {
                    self.directories_selected.insert(self.path.clone());
                    if let Err(error) = self.switch_layout(&Layout::DirectoryOrganizingLayout) {
                        self.error = error.to_string();
                    }
                }
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
                Ok(_) => {
                    self.directories_selected.insert(self.path.clone());
                    Task::none()
                }
                Err(error) => {
                    self.error = error.to_string();
                    return Task::none();
                }
            },

            Message::SelectDirectory(path_to_directory) => {
                self.selected_directory_rules = None;
                match self.directory_selected {
                    Some(ref current_selected) => {
                        if *current_selected == path_to_directory {
                            self.directory_selected = None;
                        } else {
                            self.directory_selected = Some(path_to_directory);
                        }
                    }
                    None => self.directory_selected = Some(path_to_directory),
                }
                if let Some(ref current_selected) = self.directory_selected {
                    match save_directory::read_directory_rules_from_file(
                        &self.home_directory_path,
                        &current_selected,
                    ) {
                        Ok((
                            checkbox_states,
                            date_type,
                            index_position,
                            replaceables,
                            order_of_filename_components,
                            custom_filename,
                        )) => {
                            self.selected_directory_rules = Some(SelectedDirectoryRules::from(
                                checkbox_states,
                                replaceables,
                                date_type,
                                order_of_filename_components,
                                index_position,
                                custom_filename,
                            ));
                        }
                        Err(_) => {}
                    }
                }
                return Task::none();
            }
            Message::SelectFile(file_selected_location) => {
                match file_selected_location {
                    FileSelectedLocation::FromDirectory(path_to_file) => {
                        let mut path_to_dir = PathBuf::from(&path_to_file);
                        path_to_dir.pop();
                        if let Some(directory) = self.root.get_mut_directory_by_path(&path_to_dir) {
                            if let Some(files) = directory.get_mut_files() {
                                if let Some(file_name) = path_to_file.iter().last() {
                                    if self.files_selected.contains_key(file_name) {
                                        self.error = std::io::Error::new(
                                            ErrorKind::InvalidData,
                                            "Duplicate file name found in files selected.",
                                        )
                                        .to_string();
                                        return Task::none();
                                    }
                                    if let Some((key, value)) = files.remove_entry(file_name) {
                                        self.files_selected.insert(key, value);
                                    }
                                }
                            }
                        }
                    }
                    FileSelectedLocation::FromFilesSelected(origin_path) => {
                        let mut origin_dir_path = PathBuf::from(&origin_path);
                        origin_dir_path.pop();
                        if let Some(origin_directory) =
                            self.root.get_mut_directory_by_path(&origin_dir_path)
                        {
                            if let Some(files) = origin_directory.get_mut_files() {
                                if let Some(file_name) = origin_path.iter().last() {
                                    if files.contains_key(file_name) {
                                        self.error = std::io::Error::new(
                                            ErrorKind::InvalidData,
                                            "Duplicate file name found in files origin directory",
                                        )
                                        .to_string();
                                        return Task::none();
                                    }
                                    if let Some((key, value)) =
                                        self.files_selected.remove_entry(file_name)
                                    {
                                        files.insert(key, value);
                                    }
                                }
                            }
                        }
                    }
                }
                return Task::none();
            }
            Message::SelectMultipleFiles(file_index, file_location) => {
                match file_location {
                    FileSelectedLocation::FromDirectory(path_to_file) => {
                        // Logic for multiple selecting from directory tree
                        if let Some(last) = path_to_file.iter().last() {
                            if let Ok(last_str) = convert_os_str_to_str(last) {
                                let mut path_to_directory = PathBuf::from(&path_to_file);
                                path_to_directory.pop();
                                if let Err(error) = self.select_multiple_files_from_directories(
                                    last_str,
                                    file_index,
                                    &path_to_directory,
                                ) {
                                    self.error = error.to_string();
                                }
                            }
                        }
                    }
                    FileSelectedLocation::FromFilesSelected(origin_path) => {
                        // Logic for multiple selecting from files_selected
                        if let Some(last) = origin_path.iter().last() {
                            if let Ok(last_str) = convert_os_str_to_str(last) {
                                let mut path_to_original_directory = PathBuf::from(&origin_path);
                                path_to_original_directory.pop();
                                if let Err(error) = self.select_multiple_files_from_files_selected(
                                    last_str,
                                    file_index,
                                    &path_to_original_directory,
                                ) {
                                    self.error = error.to_string();
                                }
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::InputNewDirectoryName(input) => {
                self.new_directory_name = input;
                Task::none()
            }
            Message::CreateDirectoryWithSelectedFiles => {
                self.rename_directory_name_based_on_rules();
                if let Err(error) = self.is_directory_creation_valid(SAVE_FILE_NAME) {
                    self.error = error.to_string();
                    return Task::none();
                }

                let mut files_selected = BTreeMap::new();
                while let Some((key, value)) = self.files_selected.pop_last() {
                    files_selected.insert(key, value);
                }

                match self.create_directory_with_selected_files(files_selected) {
                    Ok(_) => {
                        // Refresh the directories in layouts
                    }

                    Err(error) => self.error = error.to_string(),
                }
                let mut path = PathBuf::from(&self.path);
                path.push(&self.new_directory_name);

                self.add_directories_recursive_to_directories_selected(&path);

                Task::none()
            }
            Message::RenameFiles => {
                if !app_util::just_rename_checked(&self.checkbox_states) {
                    self.error =
                        std::io::Error::new(ErrorKind::NotFound, "No rename options specified")
                            .to_string();
                }
                if self.checkbox_states.insert_directory_name_to_file_name {
                    self.error = std::io::Error::new(
                        ErrorKind::Other,
                        "Cannot insert directory name if just renaming files",
                    )
                    .to_string();
                    return Task::none();
                }
                if !self.checkbox_states.insert_date_to_file_name {
                    let result = self.rename_files_without_directory(
                        CheckboxStates::new(
                            false,
                            false,
                            self.checkbox_states.insert_date_to_file_name,
                            false,
                            self.checkbox_states.convert_uppercase_to_lowercase,
                            self.checkbox_states.replace_character,
                            self.checkbox_states.use_only_ascii,
                            self.checkbox_states.remove_original_file_name,
                            self.checkbox_states.add_custom_name,
                        ),
                        None,
                    );
                    if let Err(error) = result {
                        self.error = error.to_string();
                    }
                }
                if let Some(date_type) = self.date_type_selected {
                    let result = self.rename_files_without_directory(
                        CheckboxStates::new(
                            false,
                            false,
                            self.checkbox_states.insert_date_to_file_name,
                            false,
                            self.checkbox_states.convert_uppercase_to_lowercase,
                            self.checkbox_states.replace_character,
                            self.checkbox_states.use_only_ascii,
                            self.checkbox_states.remove_original_file_name,
                            self.checkbox_states.add_custom_name,
                        ),
                        Some(date_type),
                    );
                    if let Err(error) = result {
                        self.error = error.to_string();
                    }
                } else {
                    self.error = std::io::Error::new(ErrorKind::NotFound, "No date type specified")
                        .to_string();
                }
                Task::none()
            }
            Message::CheckboxToggled(toggle, id) => {
                self.toggle_checkbox(toggle, id);
                Task::none()
            }
            Message::SelectReplaceable(replaceable, index) => {
                let previous_selected = self.replaceables[index].replaceable_selected;
                self.replaceables[index].replaceable_selected = Some(replaceable);
                self.replaceable_options = self
                    .replaceable_options
                    .iter()
                    .filter_map(|option| {
                        if *option == replaceable {
                            return None;
                        }
                        Some(option.clone())
                    })
                    .collect();
                if let Some(previous_selected) = previous_selected {
                    self.replaceable_options.push(previous_selected);
                }
                Task::none()
            }
            Message::SelectReplaceWith(replace_with, index) => {
                self.replaceables[index].replace_with_selected = Some(replace_with);
                Task::none()
            }
            Message::AddNewReplaceable => {
                self.replaceables.push(ReplacableSelection::new());
                Task::none()
            }
            Message::RemoveReplaceable(index) => {
                let removed = self.replaceables.remove(index);
                if let Some(selected) = removed.get_replaceable_selected() {
                    self.replaceable_options.push(selected);
                }
                if self.replaceables.is_empty() {
                    self.checkbox_states.replace_character = false;
                }
                Task::none()
            }
            Message::DateTypeSelected(date_type) => {
                self.date_type_selected = Some(date_type);
                Task::none()
            }
            Message::InsertFilesToSelectedDirectory => {
                if let Err(error) = self.insert_files_to_selected_dir() {
                    self.error = error.to_string();
                }
                Task::none()
            }
            Message::SwapFileNameComponents(index) => {
                self.swap_filename_components(index);
                Task::none()
            }
            Message::FilenameInput(input) => {
                self.filename_input = input;
                Task::none()
            }
            Message::IndexPositionSelected(index_position) => {
                match index_position {
                    IndexPosition::Before => self.index_position = Some(IndexPosition::Before),
                    IndexPosition::After => self.index_position = Some(IndexPosition::After),
                }
                return Task::none();
            }
            Message::Commit => {
                if let Err(error) = filesystem::move_files_organized(&self.files_organized) {
                    self.error = error.to_string();
                }
                let mut path_to_directory = PathBuf::from(&self.path);
                path_to_directory.push(&self.new_directory_name);

                match save_directory::write_created_directory_to_save_file(
                    &self.home_directory_path,
                    path_to_directory,
                    self.checkbox_states.clone(),
                    &self.replaceables,
                    self.date_type_selected,
                    self.index_position,
                    &self.order_of_filename_components,
                    &self.filename_input,
                ) {
                    Ok(_) => {
                        self.new_directory_name.clear();
                    }
                    Err(error) => {
                        self.error = error.to_string();
                    }
                }
                self.files_organized.clear();
                self.files_have_been_organized = true;
                self.init_app_data();
                if let Err(error) = self.switch_layout(&Layout::Main) {
                    self.error = error.to_string();
                }
                return Task::none();
            }
            Message::TabKeyPressed => {
                match self.search_directories_from_path() {
                    Ok(new_path) => {
                        self.path_input = new_path;
                        if let Err(error) = self.search_path() {
                            self.error = error.to_string();
                        }
                    }
                    Err(error) => self.error = error.to_string(),
                }
                self.update_path_input();
                iced::widget::text_input::move_cursor_to_end::<Message>(self.path_input_id.clone())
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

    pub fn get_path_input_id(&self) -> iced::widget::text_input::Id {
        self.path_input_id.clone()
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

    pub fn get_new_directory_input(&self) -> &String {
        &self.new_directory_name
    }

    pub fn get_checkbox_states(&self) -> &CheckboxStates {
        &self.checkbox_states
    }

    pub fn get_date_type_selected(&self) -> Option<DateType> {
        self.date_type_selected
    }

    pub fn get_directory_selected(&self) -> &Option<PathBuf> {
        &self.directory_selected
    }

    pub fn get_filename_input(&self) -> &String {
        &self.filename_input
    }

    pub fn get_order_of_filename_components(&self) -> &Vec<String> {
        &self.order_of_filename_components
    }

    pub fn get_index_position(&self) -> Option<IndexPosition> {
        self.index_position
    }

    pub fn get_files_have_been_organized(&self) -> bool {
        self.files_have_been_organized
    }

    pub fn get_replaceable_options(&self) -> Vec<Replaceable> {
        self.replaceable_options.to_owned()
    }

    pub fn get_files_organized(&self) -> &BTreeMap<OsString, File> {
        &self.files_organized
    }

    pub fn get_replace_with_options(&self) -> [ReplaceWith; 2] {
        self.replace_with_options
    }

    pub fn get_replaceables(&self) -> &Vec<ReplacableSelection> {
        &self.replaceables
    }

    pub fn get_selected_directory_rules(&self) -> &Option<SelectedDirectoryRules> {
        &self.selected_directory_rules
    }

    fn switch_layout(&mut self, layout: &Layout) -> std::io::Result<()> {
        match layout {
            Layout::DirectorySelectionLayout => {
                self.files_have_been_organized = false;
                match std::env::consts::OS {
                    "windows" => {
                        if let Err(error) = self.switch_layout_windows() {
                            self.error = error.to_string();
                        }
                        self.layout = Layout::DirectorySelectionLayout;
                        Ok(())
                    }
                    "macos" => {
                        if let Err(error) = self.switch_layout_macos() {
                            self.error = error.to_string();
                        }
                        self.layout = Layout::DirectorySelectionLayout;
                        Ok(())
                    }
                    "linux" => {
                        if let Err(error) = self.switch_layout_linux() {
                            self.error = error.to_string();
                        }
                        self.layout = Layout::DirectorySelectionLayout;
                        Ok(())
                    }
                    _ => Err(std::io::Error::new(
                        ErrorKind::Other,
                        "Operating system not supported",
                    )),
                }
            }
            Layout::Main => {
                self.init_app_data();
                self.layout = Layout::Main;
                Ok(())
            }
            Layout::DirectoryOrganizingLayout => {
                self.layout = Layout::DirectoryOrganizingLayout;
                Ok(())
            }
        }
    }

    fn switch_layout_windows(&mut self) -> std::io::Result<()> {
        if let Some(first) = self.get_drives_on_windows().first() {
            let path = PathBuf::from(first);
            for path in self.get_drives_on_windows() {
                self.external_storage.insert(path);
            }
            self.insert_root_directory(&path);
            self.write_home_directory()?;
            self.update_path_input();
            return Ok(());
        }
        Err(std::io::Error::new(
            ErrorKind::NotFound,
            "Could not get drives on Windows",
        ))
    }

    fn switch_layout_macos(&mut self) -> std::io::Result<()> {
        let mut path = PathBuf::from("/");
        self.insert_root_directory(&path);
        self.write_directory_to_tree(&path)?;
        path.push("Volumes");
        self.write_directory_to_tree(&path)?;
        self.get_volumes_on_macos();
        self.write_home_directory()?;
        self.update_path_input();
        Ok(())
    }

    fn switch_layout_linux(&mut self) -> std::io::Result<()> {
        let mut path = PathBuf::from("/");
        self.insert_root_directory(&path);
        self.write_directory_to_tree(&path)?;
        path.push("run");
        self.write_directory_to_tree(&path)?;
        path.push("media");
        self.write_directory_to_tree(&path)?;
        self.get_volumes_on_linux();
        self.write_home_directory()?;
        self.update_path_input();
        Ok(())
    }

    fn write_home_directory(&mut self) -> std::io::Result<()> {
        match directory::system_dir::get_home_directory() {
            Some(home_path) => {
                self.home_directory_path = home_path;
                self.write_directories_from_path(&PathBuf::from(&self.home_directory_path))?;
                return Ok(());
            }
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::NotFound,
                    "Could not find home directory",
                ));
            }
        }
    }

    fn init_app_data(&mut self) {
        self.order_of_filename_components =
            vec![String::from(filename_components::ORIGINAL_FILENAME)];
        self.directories_selected.clear();
        self.directory_selected = None;
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
            "linux" => {
                self.path.clear();
                self.path.push("/run");
                self.directories_selected.clear();
                if !self.directories_selected.contains(&PathBuf::from("/run")) {
                    self.write_directory_to_tree(&PathBuf::from(&self.path))?;
                }
                self.path.push("media");
                if !self.directories_selected.contains(&PathBuf::from("media")) {
                    self.write_directory_to_tree(&PathBuf::from(&self.path))?;
                }
                self.path.push(external);
                if !self.directories_selected.contains(&PathBuf::from(external)) {
                    self.write_directory_to_tree(&PathBuf::from(&self.path))?;
                }
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
        if path_to_selected_directory == &self.path {
            // If paths are equal then close
            self.path.pop();
            self.update_path_input();
        } else {
            // If path_to_selected_directory_has less components than current path
            if path_to_selected_directory.components().count() < self.path.components().count() {
                // Remove components from current path until current path is component count is less than selected
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
            // After that examine if directory has content
            // Check directories selected
            if !self
                .directories_selected
                .contains(path_to_selected_directory)
            {
                self.write_directory_to_tree(&path_to_selected_directory)?;
                self.directories_selected
                    .insert(path_to_selected_directory.to_owned());
            }

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
                self.directories_selected.insert(path.to_owned());
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

    fn add_directories_recursive_to_directories_selected(&mut self, path_to_directory: &PathBuf) {
        if let Some(directory) = self.root.get_mut_directory_by_path(path_to_directory) {
            self.directories_selected
                .insert(PathBuf::from(path_to_directory));
            let paths = directory.get_directory_paths_recursive(path_to_directory);
            for key in paths {
                self.directories_selected.insert(key);
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

    // This one requires fixing
    fn get_volumes_on_linux(&mut self) {
        if let Some(directories) = self.root.get_directories() {
            if let Some(run) = directories.get(&OsString::from("run")) {
                if let Some(run_sub_dirs) = run.get_directories() {
                    if let Some(media) = run_sub_dirs.get(&OsString::from("media")) {
                        if let Some(media_sub_dirs) = media.get_directories() {
                            for key in media_sub_dirs.keys() {
                                self.external_storage.insert(OsString::from(&key));
                            }
                        }
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

    fn is_directory_creation_valid(&self, save_file_name: &str) -> std::io::Result<()> {
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
        let mut path_to_directory = PathBuf::from(&self.path);
        path_to_directory.push(&self.new_directory_name);
        if let Err(error) = save_directory::read_save_file_content(
            &self.home_directory_path,
            &path_to_directory,
            save_file_name,
        ) {
            if let ErrorKind::Other = error.kind() {
                return Err(error);
            }
        }

        if self.checkbox_states.remove_original_file_name && self.filename_input.is_empty() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "If original file name is removed add custom name",
            ));
        }

        if let None = self.index_position {
            if self.checkbox_states.add_custom_name {
                return Err(std::io::Error::new(
                    ErrorKind::NotFound,
                    "Index position not found",
                ));
            }
        }

        Ok(())
    }

    fn create_directory_with_selected_files(
        &mut self,
        files_selected: BTreeMap<OsString, File>,
    ) -> std::io::Result<()> {
        if let Some(selected_directory) = self.root.get_mut_directory_by_path(&self.path) {
            if let Some(directories) = selected_directory.get_directories() {
                if !organize_files::is_directory_name_unique(&self.new_directory_name, directories)
                {
                    self.files_selected = files_selected;
                    return Err(std::io::Error::new(
                        ErrorKind::AlreadyExists,
                        "Directory name already exists.",
                    ));
                }
            }

            // In case of an error, put files_selected back to self
            let temp_files_selected = files_selected.clone();

            // Copy selected_files to files_organized
            for (filename, file) in files_selected.clone() {
                self.files_organized.insert(filename, file);
            }

            let data = organize_files::OrganizingData::new(
                files_selected,
                &self.checkbox_states,
                &self.replaceables,
                &self.new_directory_name,
                &self.filename_input,
                &self.order_of_filename_components,
                self.date_type_selected,
                self.index_position,
            );

            // Write directory path and checkbox states to a file
            if let Err(error) = organize_files::apply_rules_for_directory(
                &self.path,
                &mut self.files_organized,
                String::from(&self.new_directory_name),
                selected_directory,
                data,
            ) {
                self.files_selected = temp_files_selected;
                self.files_organized.clear();
                return Err(error);
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
        checkbox_states: CheckboxStates,
        date_type: Option<DateType>,
    ) -> std::io::Result<()> {
        if let Some(selected_dir) = self.root.get_mut_directory_by_path(&self.path) {
            while let Some((key, mut value)) = self.files_selected.pop_last() {
                let file_name = app_util::convert_os_str_to_str(&key)?;
                let mut renamed_file_name = String::new();
                let file_count = selected_dir.get_file_count();
                organize_files::rename_file_name(organize_files::RenameData::build(
                    &mut renamed_file_name,
                    &checkbox_states,
                    &self.replaceables,
                    &self.new_directory_name,
                    &self.filename_input,
                    file_count,
                    &self.order_of_filename_components,
                    file_name,
                    &value,
                    date_type,
                    self.index_position,
                ));
                organize_files::create_destination_path(&self.path, vec![], &mut value);
                self.files_organized
                    .insert(OsString::from(&renamed_file_name), value.clone());
                selected_dir.insert_file(OsString::from(renamed_file_name), value);
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
                self.checkbox_states.convert_uppercase_to_lowercase = toggle;
            }
            4 => {
                self.checkbox_states.replace_character = toggle;
                if toggle && self.replaceables.is_empty() {
                    self.replaceables.push(ReplacableSelection::new());
                }
            }
            5 => {
                self.checkbox_states.use_only_ascii = toggle;
            }
            6 => {
                self.checkbox_states.insert_directory_name_to_file_name = toggle;
                if toggle {
                    self.order_of_filename_components
                        .push(String::from(filename_components::DIRECTORY_NAME));
                } else {
                    self.filter_order_of_filename_components(String::from(
                        filename_components::DIRECTORY_NAME,
                    ));
                }
            }
            7 => {
                self.checkbox_states.insert_date_to_file_name = toggle;
                if toggle {
                    self.order_of_filename_components
                        .push(String::from(filename_components::DATE));
                } else {
                    self.filter_order_of_filename_components(String::from(
                        filename_components::DATE,
                    ));
                }
            }
            8 => {
                self.checkbox_states.remove_original_file_name = toggle;
                if toggle {
                    self.filter_order_of_filename_components(String::from(
                        filename_components::ORIGINAL_FILENAME,
                    ));
                    self.checkbox_states.add_custom_name = true;
                    if !self
                        .order_of_filename_components
                        .contains(&String::from(filename_components::CUSTOM_FILE_NAME))
                    {
                        self.order_of_filename_components
                            .push(String::from(filename_components::CUSTOM_FILE_NAME));
                    }
                } else {
                    self.order_of_filename_components
                        .push(String::from(filename_components::ORIGINAL_FILENAME));
                }
            }
            9 => {
                if self.checkbox_states.remove_original_file_name {
                    return;
                }
                self.checkbox_states.add_custom_name = toggle;
                if toggle {
                    self.order_of_filename_components
                        .push(String::from(filename_components::CUSTOM_FILE_NAME));
                } else {
                    self.filter_order_of_filename_components(String::from(
                        filename_components::CUSTOM_FILE_NAME,
                    ));
                }
            }
            _ => {}
        }
    }

    fn filter_order_of_filename_components(&mut self, filter_value: String) {
        self.order_of_filename_components = self
            .order_of_filename_components
            .iter()
            .filter_map(|element| {
                if *element == filter_value {
                    return None;
                }
                Some((*element).clone())
            })
            .collect();
    }

    fn select_multiple_files_from_directories(
        &mut self,
        new_file_name: &str,
        new_file_index: usize,
        directory_path: &PathBuf,
    ) -> std::io::Result<()> {
        // Select multiple files from_directories
        if self.multiple_selection.file_name.is_empty() {
            self.multiple_selection.file_name = String::from(new_file_name);
            self.multiple_selection.file_index = new_file_index;
            return Ok(());
        } else {
            // Do multiple select
            let mut files_selected = BTreeMap::new();
            let mut files_unselected = BTreeMap::new();
            if let Some(directory) = self.root.get_mut_directory_by_path(directory_path) {
                if let Some(mut files) = directory.get_mut_files().take() {
                    if self.multiple_selection.file_index > new_file_index {
                        // Select from bottom
                        (files_selected, files_unselected) = multiple_select_files(
                            &mut files,
                            &self.multiple_selection.file_name,
                            new_file_name,
                            SelectionDirection::Bottom,
                        );
                    } else {
                        // Select from top
                        (files_selected, files_unselected) = multiple_select_files(
                            &mut files,
                            &self.multiple_selection.file_name,
                            new_file_name,
                            SelectionDirection::Up,
                        );
                    }
                    directory.insert_empty_files();
                }
            }
            // Check for errors
            if let Err(error) = app_util::is_duplicate_files_in_directory_selection(
                &files_selected,
                &self.files_selected,
            ) {
                if let Some(directory) = self.root.get_mut_directory_by_path(directory_path) {
                    if let Some(files) = directory.get_mut_files() {
                        for (key, value) in files_selected {
                            files.insert(key, value);
                        }
                        for (key, value) in files_unselected {
                            files.insert(key, value);
                        }
                    }
                }
                return Err(error);
            }
            // Put files to files_selected
            if let Some(directory) = self.root.get_mut_directory_by_path(directory_path) {
                if let Some(files) = directory.get_mut_files() {
                    for (key, value) in files_unselected {
                        files.insert(key, value);
                    }

                    for (key, value) in files_selected {
                        self.files_selected.insert(key, value);
                    }
                }
            }
        }
        self.multiple_selection.file_index = 0;
        self.multiple_selection.file_name.clear();
        Ok(())
    }

    fn select_multiple_files_from_files_selected(
        &mut self,
        new_file_name: &str,
        new_file_index: usize,
        origin_directory_path: &PathBuf,
    ) -> std::io::Result<()> {
        if self.multiple_selection.file_name.is_empty() {
            self.multiple_selection.file_name = String::from(new_file_name);
            self.multiple_selection.file_index = new_file_index;
            return Ok(());
        } else {
            // Write logic for second click in files_selected
            let (files_selected, files_unselected) =
                if self.multiple_selection.file_index > new_file_index {
                    multiple_select_files(
                        &mut self.files_selected,
                        &self.multiple_selection.file_name,
                        new_file_name,
                        SelectionDirection::Bottom,
                    )
                } else {
                    multiple_select_files(
                        &mut self.files_selected,
                        &self.multiple_selection.file_name,
                        new_file_name,
                        SelectionDirection::Up,
                    )
                };

            // Do error checks
            if let Err(error) = app_util::is_duplicate_files_in_files_selected(
                &self.root,
                &self.files_selected,
                &origin_directory_path,
            ) {
                for (key, value) in files_selected {
                    self.files_selected.insert(key, value);
                }

                for (key, value) in files_unselected {
                    self.files_selected.insert(key, value);
                }
                return Err(error);
            }
            // Put files back to origin_directory or files_selected
            if let Some(origin_directory) =
                self.root.get_mut_directory_by_path(origin_directory_path)
            {
                if let Some(files) = origin_directory.get_mut_files() {
                    // Do some loops
                    for (key, value) in files_selected {
                        files.insert(key, value);
                    }

                    for (key, value) in files_unselected {
                        self.files_selected.insert(key, value);
                    }
                }
            }
            self.multiple_selection.file_index = 0;
            self.multiple_selection.file_name.clear();
            return Ok(());
        }
    }

    fn insert_files_to_selected_dir(&mut self) -> std::io::Result<()> {
        if let Some(selected_dir_path) = &self.directory_selected {
            if let Some(selected_dir) = self.root.get_mut_directory_by_path(selected_dir_path) {
                let (
                    checkbox_states,
                    date_type,
                    index_position,
                    replaceables,
                    order_of_filename_components,
                    custom_filename,
                ) = save_directory::read_directory_rules_from_file(
                    &self.home_directory_path,
                    selected_dir_path,
                )?;
                if let Some(last) = selected_dir_path.iter().last() {
                    let directory_name = app_util::convert_os_str_to_str(last)?;
                    organize_files::move_files_to_organized_directory(
                        &self.path,
                        &mut self.files_organized,
                        selected_dir,
                        organize_files::OrganizingData::new(
                            self.files_selected.clone(),
                            &checkbox_states,
                            &replaceables,
                            directory_name,
                            &custom_filename,
                            &order_of_filename_components,
                            date_type,
                            index_position,
                        ),
                    )?;
                    self.files_selected.clear();
                    return Ok(());
                }
            }
        }
        Err(std::io::Error::new(
            ErrorKind::NotFound,
            "Could not find selected directory.",
        ))
    }

    fn swap_filename_components(&mut self, index: usize) {
        if self.order_of_filename_components.len() >= index {
            let temp = self.order_of_filename_components[index - 1].clone();
            self.order_of_filename_components[index - 1] =
                self.order_of_filename_components[index].clone();
            self.order_of_filename_components[index] = temp;
        }
    }

    fn follow_directory_path(&self, current_path: &PathBuf) -> Option<&Directory> {
        let mut path_stack = PathBuf::new();
        let mut dir = None;
        for (i, component) in current_path.components().enumerate() {
            path_stack.push(component);
            if i == 0 {
                continue;
            }
            if std::env::consts::OS == "windows" {
                if i == 1 {
                    continue;
                }
            }
            let directory = self.root.get_directory_by_path(&path_stack);
            if directory.get_name() != Some(OsString::from("/")) {
                dir = Some(directory);
            } else {
                break;
            }
        }
        dir
    }

    fn path_has_only_prefix(&self, path: &str) -> bool {
        let mut contains_character = false;
        let mut contains_colon = false;
        for (i, character) in path.chars().enumerate() {
            for ch in 'A'..'Z' {
                if i == 0 && character == ch {
                    contains_character = true;
                }
            }
            for ch in 'a'..'z' {
                if i == 0 && character == ch {
                    contains_character = true;
                }
            }
            if i == 1 && character == ':' {
                contains_colon = true;
            }
        }
        if contains_character && contains_colon && path.len() == 2 || path.len() == 3 {
            return true;
        }
        if contains_character && path.len() == 1 {
            return true;
        }
        false
    }

    fn search_directories_from_path(&mut self) -> std::io::Result<String> {
        let current_path = PathBuf::from(&self.path_input);
        if std::env::consts::OS == "windows" {
            let current_path = app_util::convert_path_to_str(&current_path)?;
            if self.path_has_only_prefix(current_path) {
                let mut prefix_path = String::from(current_path);
                if prefix_path.len() == 2 {
                    prefix_path.push('\\');
                }
                if prefix_path.len() == 1 {
                    prefix_path.push(':');
                    prefix_path.push('\\');
                }
                return Ok(prefix_path);
            }
        }
        if let Some(last_component) = current_path.iter().last() {
            if let Some(directory) = self.follow_directory_path(&current_path) {
                let mut dir_with_greatest_score = None;
                let mut path_is_equal = false;

                if directory.get_name() == Some(OsString::from(last_component)) {
                    let last_component = app_util::convert_os_str_to_str(last_component)?;
                    dir_with_greatest_score = Some(last_component);
                    path_is_equal = true;
                } else if let Some(sub_directories) = directory.get_directories() {
                    let mut score = 0;
                    for dir_name in sub_directories.keys() {
                        if let Some((last_component, dir_name)) =
                            self.get_path_components_to_str(last_component, dir_name)
                        {
                            let count = app_util::is_substring(last_component, dir_name);
                            if count > score {
                                score = count;
                                dir_with_greatest_score = Some(dir_name);
                            }
                        }
                    }
                }
                if let Some(dir) = dir_with_greatest_score {
                    return self.add_new_dir_to_path_input(dir, path_is_equal);
                }
            }
            if current_path == PathBuf::from("/") {
                return Ok(String::from("/"));
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No match found.",
        ))
    }

    fn insert_slash(&self, path: &mut String) {
        match std::env::consts::OS {
            "windows" => path.push_str("\\"),
            "linux" | "macos" => path.push_str("/"),
            _ => {}
        }
    }

    fn add_new_dir_to_path_input(&self, dir: &str, path_is_equal: bool) -> std::io::Result<String> {
        let mut result_path = self.path_input.clone();
        if path_is_equal {
            if result_path.ends_with("/") || result_path.ends_with("\\") {
                return Ok(result_path);
            }
            self.insert_slash(&mut result_path);
            return Ok(result_path);
        }

        while let Some(ch) = result_path.pop() {
            if ch == '/' || ch == '\\' {
                break;
            }
        }

        self.insert_slash(&mut result_path);
        result_path.push_str(dir);
        self.insert_slash(&mut result_path);
        return Ok(result_path);
    }

    fn get_path_components_to_str<'a>(
        &'a self,
        last_component: &'a OsStr,
        dir_name: &'a OsStr,
    ) -> Option<(&'a str, &'a str)> {
        if let Some(dir_name) = dir_name.to_str() {
            if let Some(last_component) = last_component.to_str() {
                return Some((last_component, dir_name));
            }
        }
        None
    }

    fn rename_directory_name_based_on_rules(&mut self) {
        if self.checkbox_states.use_only_ascii {
            self.new_directory_name =
                organize_files::replace_non_ascii(self.new_directory_name.to_owned());
        }

        if self.checkbox_states.convert_uppercase_to_lowercase {
            self.new_directory_name = self.new_directory_name.to_lowercase();
        }
        if self.checkbox_states.replace_character {
            for replaceable in &self.replaceables {
                if let Some(replace) = replaceable.get_replaceable_selected() {
                    if let Some(replace_with) = replaceable.get_replace_with_selected() {
                        organize_files::replace_character_with(
                            &mut self.new_directory_name,
                            replace,
                            replace_with,
                        );
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum SelectionDirection {
    Up,
    Bottom,
}

pub fn multiple_select_files(
    files_holder: &mut BTreeMap<OsString, File>,
    previous_file_name: &str,
    new_file_name: &str,
    direction: SelectionDirection,
) -> (BTreeMap<OsString, File>, BTreeMap<OsString, File>) {
    let mut files_selected = BTreeMap::new();
    let mut files_unselected = BTreeMap::new();
    let mut in_boundaries = false;
    match direction {
        SelectionDirection::Bottom => {
            while let Some((key, value)) = files_holder.pop_last() {
                if OsString::from(previous_file_name) == key {
                    in_boundaries = true;
                }
                if in_boundaries {
                    files_selected.insert(key.to_owned(), value);
                } else {
                    files_unselected.insert(key.to_owned(), value);
                }
                if OsString::from(new_file_name) == key {
                    in_boundaries = false;
                }
            }
        }
        SelectionDirection::Up => {
            while let Some((key, value)) = files_holder.pop_first() {
                if OsString::from(previous_file_name) == key {
                    in_boundaries = true;
                }
                if in_boundaries {
                    files_selected.insert(key.to_owned(), value);
                } else {
                    files_unselected.insert(key.to_owned(), value);
                }
                if OsString::from(new_file_name) == key {
                    in_boundaries = false;
                }
            }
        }
    }
    (files_selected, files_unselected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::system_dir;
    use crate::metadata::Metadata;
    use std::fs;

    #[test]
    fn test_filter_order_of_file_name_components() {
        let mut app = App::default();
        app.order_of_filename_components
            .push(String::from(filename_components::DATE));
        app.order_of_filename_components
            .push(String::from(filename_components::ORIGINAL_FILENAME));
        app.order_of_filename_components
            .push(String::from(filename_components::DIRECTORY_NAME));
        app.order_of_filename_components
            .push(String::from(filename_components::CUSTOM_FILE_NAME));
        app.filter_order_of_filename_components(String::from(filename_components::DATE));
        assert_eq!(
            app.order_of_filename_components,
            vec![
                String::from(filename_components::ORIGINAL_FILENAME),
                String::from(filename_components::DIRECTORY_NAME),
                String::from(filename_components::CUSTOM_FILE_NAME)
            ]
        )
    }

    const TEST_SAVE_FILE: &str = ".test_save_file.csv";
    #[test]
    fn test_is_directory_creation_valid() {
        let mut app = App::default();
        if let Err(error) = app.is_directory_creation_valid(TEST_SAVE_FILE) {
            assert_eq!(error.to_string(), "No files selected.");
        }

        app.files_selected
            .insert(OsString::from("file1.txt"), File::new(Metadata::new()));
        app.files_selected
            .insert(OsString::from("file2.txt"), File::new(Metadata::new()));
        app.files_selected
            .insert(OsString::from("file3.txt"), File::new(Metadata::new()));

        if let Err(error) = app.is_directory_creation_valid(TEST_SAVE_FILE) {
            assert_eq!(error.to_string(), "Directory name not specified.");
        }

        app.new_directory_name = String::from("content");
        if let Err(error) = app.is_directory_creation_valid(TEST_SAVE_FILE) {
            assert_eq!(error.to_string(), "Directory name not specified.");
        }
        let home_path =
            system_dir::get_home_directory().expect("Could not get home directory path");
        let _save_file = save_directory::create_save_file(&home_path, TEST_SAVE_FILE)
            .expect("Failed to create temporary save file.");
        let mut path_to_file = PathBuf::from(home_path);
        path_to_file.push(TEST_SAVE_FILE);
        if let Err(error) = app.is_directory_creation_valid(TEST_SAVE_FILE) {
            fs::remove_file(path_to_file).expect("Failed to remove test save file");
            panic!(
                "is_directory_creation_valid could not read temporary save file: {}",
                error
            );
        }
        app.checkbox_states.remove_original_file_name = true;
        if let Err(error) = app.is_directory_creation_valid(TEST_SAVE_FILE) {
            assert_eq!(
                error.to_string(),
                "If original file name is removed add custom name"
            );
        }
        app.filename_input = String::from("filename");
        app.checkbox_states.add_custom_name = true;
        if let Err(error) = app.is_directory_creation_valid(TEST_SAVE_FILE) {
            assert_eq!(error.to_string(), "Index position not found");
        }

        app.index_position = Some(IndexPosition::Before);
        if let Err(error) = app.is_directory_creation_valid(TEST_SAVE_FILE) {
            fs::remove_file(path_to_file).expect("Failed to remove test save file");
            panic!("Directory creation should be valid: {}", error);
        }
        fs::remove_file(path_to_file).expect("Failed to remove test save file");
    }

    #[test]
    fn test_update_path_prefix() {
        let mut app = App::default();
        app.external_storage.insert(OsString::from("C:/"));
        app.external_storage.insert(OsString::from("D:/"));
        app.external_storage.insert(OsString::from("E:/"));
        app.update_path_prefix(&OsString::from("C:/"));
        assert_eq!(app.path, PathBuf::from("C:/"));
        app.update_path_prefix(&OsString::from("F:/"));
        assert_eq!(app.path, PathBuf::from("C:/"));
    }

    #[test]
    fn test_update_path_input() {
        let mut app = App::default();
        app.path = PathBuf::from("/home/verneri/rust");
        assert_eq!(app.path_input, String::from(""));
        app.update_path_input();
        assert_eq!(app.path_input, String::from("/home/verneri/rust"));
    }
}
