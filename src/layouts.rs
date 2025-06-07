use std::{
    collections::BTreeSet,
    ffi::{OsStr, OsString},
    path::{Iter, PathBuf},
};

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, checkbox, column, container, radio, row, scrollable, text, text_input, Column,
        Container, Row,
    },
    Background, Color,
    Length::{Fill, FillPortion},
    Theme,
};

use crate::{
    app::{App, Message},
    directory::Directory,
    metadata::{DateType, Metadata},
};

#[derive(Debug, Clone)]
pub struct CheckboxStates {
    pub organize_by_filetype: bool,
    pub organize_by_date: bool,
    pub insert_date_to_file_name: bool,
    pub insert_directory_name_to_file_name: bool,
    pub remove_uppercase: bool,
    pub replace_spaces_with_underscores: bool,
    pub use_only_ascii: bool,
    pub remove_original_file_name: bool,
    pub add_custom_name: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexPosition {
    Before,
    After,
}

impl Default for CheckboxStates {
    fn default() -> Self {
        Self {
            organize_by_filetype: false,
            organize_by_date: false,
            insert_date_to_file_name: false,
            insert_directory_name_to_file_name: false,
            remove_uppercase: false,
            replace_spaces_with_underscores: false,
            use_only_ascii: false,
            remove_original_file_name: false,
            add_custom_name: false,
        }
    }
}

impl CheckboxStates {
    pub fn new(
        organize_by_filetype: bool,
        organize_by_date: bool,
        insert_date_to_file_name: bool,
        insert_directory_name_to_file_name: bool,
        remove_uppercase: bool,
        replace_spaces_with_underscores: bool,
        use_only_ascii: bool,
        remove_original_file_name: bool,
        add_custom_name: bool,
    ) -> Self {
        Self {
            organize_by_filetype,
            organize_by_date,
            insert_date_to_file_name,
            insert_directory_name_to_file_name,
            remove_uppercase,
            replace_spaces_with_underscores,
            use_only_ascii,
            remove_original_file_name,
            add_custom_name,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DirectoryView {
    List,
    DropDown,
}

#[derive(Debug, Clone)]
pub enum Layout {
    Main,
    DirectorySelectionLayout,
    DirectoryOrganizingLayout,
}

impl Layout {
    pub fn get_layout<'a>(&'a self, app: &'a App) -> Container<'a, Message> {
        match self {
            Layout::Main => self.main_layout(app),
            Layout::DirectorySelectionLayout => self.directory_tree_layout(app),
            Layout::DirectoryOrganizingLayout => self.directory_organizing_layout(app),
        }
    }

    fn main_layout<'a>(&self, _: &App) -> Container<'a, Message> {
        container(column![
            text("Filerganizer").size(50),
            row![
                button("Select directory to organize")
                    .style(directory_button_style)
                    .on_press(Message::SwitchLayout(Layout::DirectorySelectionLayout)),
                button("Exit")
                    .on_press(Message::Exit)
                    .style(directory_button_style)
            ]
            .spacing(10)
        ])
        .center(Fill)
    }

    fn directory_tree_layout<'a>(&'a self, app: &'a App) -> Container<'a, Message> {
        if let Some(path) = app.get_path().to_str() {
            container(
                column![
                    column![
                        button("Main Menu")
                            .on_press(Message::SwitchLayout(Layout::Main))
                            .style(directory_button_style),
                        row![
                            self.insert_search_bar(app, path),
                            self.insert_directory_view_buttons(app),
                            row![button("Select this directory").on_press(Message::SelectPath)]
                        ]
                        .spacing(10),
                        self.insert_external_storage(app),
                        button("Previous")
                            .on_press(Message::MoveUpDirectory)
                            .style(directory_button_style),
                        text(app.get_error()),
                    ]
                    .spacing(5),
                    self.display_directory_contents(app).spacing(5),
                ]
                .spacing(10)
                .padding(10),
            )
        } else {
            container(text("Could not find path"))
        }
    }

    fn directory_organizing_layout<'a>(&'a self, app: &'a App) -> Container<'a, Message> {
        let mut select_all_files_button = Column::new();
        if let Some(path) = app.get_path().to_str() {
            let selected_dir = app
                .get_root_directory()
                .get_directory_by_path(app.get_path());

            if let Some(files) = selected_dir.get_files() {
                if !files.is_empty() {
                    select_all_files_button = select_all_files_button
                        .push(button("Select all files").on_press(Message::SelectAllFiles));
                }
            }

            container(column![
                row![
                    column![button("Back").on_press(Message::Back)]
                        .width(FillPortion(1))
                        .align_x(Horizontal::Left),
                    column![text("Selected path: "), text(path)]
                        .width(FillPortion(1))
                        .align_x(Horizontal::Left),
                    column![button("Commit")]
                        .width(FillPortion(1))
                        .align_x(Horizontal::Right),
                ]
                .align_y(Vertical::Center),
                column![text(app.get_error())],
                select_all_files_button,
                row![
                    scrollable(self.display_selected_path_content(app)).width(FillPortion(2)),
                    scrollable(
                        column![
                            self.selected_directory_option(app),
                            self.insert_files_selected(app),
                        ]
                        .padding(10)
                    )
                    .width(FillPortion(2))
                    .spacing(5)
                ]
                .spacing(5)
            ])
        } else {
            container(text("Could not find path"))
        }
    }

    fn rules_for_directory<'a>(&'a self, app: &'a App) -> Column<Message> {
        let created = radio(
            "Created",
            DateType::Created,
            app.get_date_type_selected(),
            Message::DateTypeSelected,
        );
        let accessed = radio(
            "Accessed",
            DateType::Accessed,
            app.get_date_type_selected(),
            Message::DateTypeSelected,
        );
        let modified = radio(
            "Modified",
            DateType::Modified,
            app.get_date_type_selected(),
            Message::DateTypeSelected,
        );
        column![
            text("Rules for directory"),
            column![
                checkbox(
                    "Organize to directories by file type.",
                    app.get_checkbox_states().organize_by_filetype
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 1) }),
                checkbox(
                    "Organize to directories by date.",
                    app.get_checkbox_states().organize_by_date
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 2) }),
                column![text("Datetype"), created, accessed, modified].padding(10),
                checkbox(
                    "Remove uppercase",
                    app.get_checkbox_states().remove_uppercase
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 3) }),
                checkbox(
                    "Replace spaces with underscores",
                    app.get_checkbox_states().replace_spaces_with_underscores
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 4) }),
                checkbox(
                    "Use ascii characters only",
                    app.get_checkbox_states().use_only_ascii
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 5) }),
                checkbox(
                    "Insert directory name to file name",
                    app.get_checkbox_states().insert_directory_name_to_file_name
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 6) }),
                checkbox(
                    "Insert date to file name",
                    app.get_checkbox_states().insert_date_to_file_name
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 7) }),
                checkbox(
                    "Remove original file name",
                    app.get_checkbox_states().remove_original_file_name
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 8) }),
                row![
                    checkbox(
                        "Add custom name to file name",
                        app.get_checkbox_states().add_custom_name
                    )
                    .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 9) }),
                    self.custom_name_box(app)
                ]
                .align_y(Vertical::Center)
                .spacing(5)
            ],
            column![self.order_of_file_name_components(app)]
        ]
    }

    fn custom_name_box(&self, app: &App) -> Row<Message> {
        let index_before = radio(
            "Prefix",
            IndexPosition::Before,
            app.get_index_position(),
            Message::IndexPositionSelected,
        );

        let index_after = radio(
            "Suffix",
            IndexPosition::After,
            app.get_index_position(),
            Message::IndexPositionSelected,
        );
        if app.get_checkbox_states().add_custom_name {
            return row![
                text_input("Add custom file name", app.get_filename_input())
                    .on_input(Message::FilenameInput),
                column![index_before, index_after]
            ]
            .spacing(5)
            .align_y(Vertical::Center);
        }
        return row![];
    }

    fn order_of_file_name_components<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let mut column = Column::new();
        let mut row = Row::new();
        let order_of_filename_components = app.get_order_of_filename_components();
        if !order_of_filename_components.is_empty() {
            column = column.push(text("Order of filename components"));
        }
        for component in order_of_filename_components {
            row = row.push(text(component));
        }
        row = row.spacing(5).padding(5).align_y(Vertical::Center);
        column = column.push(row);
        return column;
    }

    fn selected_directory_option<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let mut column = Column::new();
        let directory_selected = app.get_directory_selected();
        if let Some(directory_path) = directory_selected {
            if let Some(last_component) = directory_path.iter().last() {
                if let Some(dir_name) = last_component.to_str() {
                    let row = row![
                        text("Selected directory").size(15),
                        button(dir_name).style(directory_button_style)
                    ]
                    .spacing(5)
                    .align_y(Vertical::Center);
                    column = column.push(row);
                    column = column.push(button("Extract directory content").on_press(
                        Message::ExtractContentFromDirectory(PathBuf::from(directory_path)),
                    ));
                    column = column.push(button("Extract all files from directory").on_press(
                        Message::ExtractAllContentFromDirectory(PathBuf::from(directory_path)),
                    ));
                    if app.get_directory_selected().is_some() {
                        column = column.push(
                            button("Insert selected files to selected directory")
                                .on_press(Message::InsertFilesToSelectedDirectory),
                        );
                    }
                    column = column.padding(10).spacing(10);
                }
            }
        }
        column
    }

    fn insert_files_selected<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let mut column = Column::new();

        let mut path_stack = PathBuf::from(app.get_path());
        for (i, (key, _)) in app.get_files_selected().iter().enumerate() {
            if i == 0 {
                column = column.push(row![
                    text_input("New directory name", app.get_new_directory_input())
                        .on_input(Message::InputNewDirectoryName),
                    button("Create directory with selected files")
                        .on_press(Message::CreateDirectoryWithSelectedFiles),
                ]);
                column = column.push(button("Just rename").on_press(Message::RenameFiles));
                column = column.push(self.rules_for_directory(app));

                column = column.push(
                    button("Remove all files from selected").on_press(Message::PutAllFilesBack),
                );
                column = column.push(text("Selected files").size(15));
            }
            if let Some(file_name) = key.to_str() {
                path_stack.push(key);
                column = column.push(
                    button(file_name)
                        .style(file_button_style)
                        .on_press(Message::SelectFile(PathBuf::from(&path_stack))),
                );
                path_stack.pop();
            }
        }
        column = column.spacing(10).padding(10);

        column
    }

    fn display_selected_path_content<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let mut column = Column::new();
        let root = app
            .get_root_directory()
            .get_directory_by_path(app.get_path());
        let mut path = PathBuf::from(app.get_path());
        let mut directories_selected_iter = app.get_directories_selected().iter();

        column = self.display_directories_as_dropdown(
            root,
            &mut path,
            &mut directories_selected_iter,
            0,
            column,
        );

        // Display files in the root directory
        column = self.append_files_to_column(root, &mut path, column, true);
        column = column.padding(10).spacing(10);
        column
    }
    // For when all sub directories are already read
    fn display_directories_as_dropdown<'a>(
        &'a self,
        current_directory: &'a Directory,
        path: &mut PathBuf,
        directories_selected_iter: &mut std::slice::Iter<'_, PathBuf>,
        call_count: usize,
        mut column: Column<'a, Message>,
    ) -> Column<'a, Message> {
        let next_dir_selected = directories_selected_iter.next();
        let directories = current_directory.get_directories();

        if let Some(directories) = directories {
            for (key, subdir) in directories {
                path.push(key);
                if let Some(directory_name) = key.to_str() {
                    let is_selected = next_dir_selected
                        .and_then(|p| p.iter().last())
                        .map_or(false, |last| last == key);
                    let drop_down_icon = if is_selected { "|" } else { ">" };
                    let button_row = self.create_directory_buttons_row(
                        call_count,
                        drop_down_icon,
                        directory_name,
                        path,
                    );

                    column = column.push(button_row);

                    if is_selected {
                        let mut new_column = Column::new();
                        path.push(key);
                        new_column = self.display_directories_as_dropdown(
                            subdir,
                            path,
                            directories_selected_iter,
                            call_count + 1,
                            new_column,
                        );
                        path.pop();
                        new_column = new_column.padding(20).spacing(10);
                        new_column = self.append_files_to_column(subdir, path, new_column, false);
                        column = column.push(new_column);
                    }
                }
                path.pop();
            }
        }

        column
    }

    fn append_files_to_column<'a>(
        &'a self,
        root: &'a Directory,
        path: &mut PathBuf,
        mut column: Column<'a, Message>,
        files_pressable: bool,
    ) -> Column<'a, Message> {
        if let Some(files) = root.get_files() {
            for key in files.keys() {
                if let Some(file_name) = key.to_str() {
                    path.push(key);

                    if files_pressable {
                        column = column.push(
                            button(file_name)
                                .style(file_button_style)
                                .on_press(Message::SelectFile(PathBuf::from(&path))),
                        );
                    } else {
                        column = column.push(text(file_name));
                    }

                    path.pop();
                }
            }
        }
        column
    }

    fn create_directory_buttons_row<'a>(
        &'a self,
        call_count: usize,
        drop_down_icon: &'a str,
        directory_name: &'a str,
        path: &PathBuf,
    ) -> Row<'a, Message> {
        if call_count == 0 {
            return row![
                button(drop_down_icon)
                    .style(directory_button_style)
                    .on_press(Message::ViewDirectory(PathBuf::from(&path))),
                button(directory_name)
                    .style(directory_button_style)
                    .on_press(Message::SelectDirectory(PathBuf::from(&path))),
            ];
        } else {
            return row![
                button(drop_down_icon)
                    .style(directory_button_style)
                    .on_press(Message::ViewDirectory(PathBuf::from(&path))),
                text(directory_name)
            ]
            .spacing(5)
            .align_y(Vertical::Center);
        }
    }

    fn insert_search_bar<'a>(&self, app: &'a App, path: &str) -> Row<'a, Message> {
        row![
            text_input(path, app.get_path_input())
                .on_input(Message::TextInput)
                .on_submit(Message::SearchPath),
            button("Search")
                .style(directory_button_style)
                .on_press(Message::SearchPath)
        ]
    }

    fn insert_directory_view_buttons<'a>(&self, app: &'a App) -> Row<'a, Message> {
        row![
            button("List view")
                .on_press(Message::SwitchDirectoryView(DirectoryView::List))
                .style(|theme: &Theme, _| {
                    let status = match app.get_directory_view() {
                        DirectoryView::List => button::Status::Disabled,
                        DirectoryView::DropDown => button::Status::Active,
                    };
                    directory_button_style(theme, status)
                }),
            button("Drop down")
                .on_press(Message::SwitchDirectoryView(DirectoryView::DropDown))
                .style(|theme: &Theme, _| {
                    let status = match app.get_directory_view() {
                        DirectoryView::List => button::Status::Active,
                        DirectoryView::DropDown => button::Status::Disabled,
                    };
                    directory_button_style(theme, status)
                }),
        ]
    }

    fn display_directory_contents<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        match app.get_directory_view() {
            DirectoryView::List => column![
                self.insert_header(),
                scrollable(self.display_directory_contents_as_list(app))
            ],
            DirectoryView::DropDown => {
                let path = PathBuf::from(app.get_path());
                let mut path_iter = path.iter();

                let mut path_stack = PathBuf::new();
                if let Some(root) = path_iter.next() {
                    path_stack.push(root);
                    if let std::env::consts::OS = "windows" {
                        if let Some(next) = path_iter.next() {
                            path_stack.push(next);
                        }
                    }
                }
                let root_dir = app.get_root_directory();
                return column![scrollable(self.insert_directory_content_as_dropdown(
                    root_dir,
                    &mut path_iter,
                    &mut path_stack,
                ))];
            }
        }
    }
    // For when all sub directories haven't been read
    fn insert_directory_content_as_dropdown<'a>(
        &'a self,
        current_directory: &'a Directory,
        full_path_iter: &mut Iter<'_>,
        path_stack: &mut PathBuf,
    ) -> Column<'a, Message> {
        let mut column = Column::new();
        if let Some(next) = full_path_iter.next() {
            if let Some(directories) = current_directory.get_directories() {
                for dir_key in directories.keys() {
                    column = self.insert_drop_down_directories(dir_key, path_stack, column);
                    if dir_key == next {
                        if let Some(selected) = directories.get(dir_key) {
                            path_stack.push(next);
                            let mut new_column = self.insert_directory_content_as_dropdown(
                                selected,
                                full_path_iter,
                                path_stack,
                            );
                            path_stack.pop();
                            new_column = new_column.padding(10);
                            new_column = new_column.spacing(10);
                            new_column = self.insert_drop_down_files(selected, new_column);
                            column = column.push(new_column);
                        }
                    }
                }
            }
        } else {
            if let Some(directories) = current_directory.get_directories() {
                for dir_key in directories.keys() {
                    column = self.insert_drop_down_directories(dir_key, path_stack, column);
                }
            }
        }
        column
    }

    fn display_directory_contents_as_list<'a>(&self, app: &'a App) -> Column<'a, Message> {
        let mut column = Column::new();
        let current_directory = app
            .get_root_directory()
            .get_directory_by_path(app.get_path());
        column = self.insert_directories(current_directory, column);
        column = self.insert_files(current_directory, column);
        column
    }

    fn insert_external_storage<'a>(&self, app: &'a App) -> Row<'a, Message> {
        let mut row = Row::new();
        let external_directories: &BTreeSet<OsString> = app.get_external_directories();
        for key in external_directories.iter() {
            if let Some(k) = key.to_str() {
                row = row.push(
                    button(k)
                        .style(directory_button_style)
                        .on_press(Message::MoveInExternalDirectory(OsString::from(key))),
                );
            }
        }
        row
    }

    fn insert_header<'a>(&self) -> Row<'a, Message> {
        let mut header: Row<Message> = Row::new();
        header = header.push(text("Name").width(FillPortion(1)));
        header = header.push(text("Created").width(FillPortion(1)));
        header = header.push(text("Accessed").width(FillPortion(1)));
        header = header.push(text("Modified").width(FillPortion(1)));
        header = header.push(text("Permissions").width(FillPortion(1)));
        header = header.push(text("Size").width(FillPortion(1)));
        header = header.padding(10);
        header
    }

    fn insert_directories<'a>(
        &self,
        root_dir: &'a Directory,
        mut column: Column<'a, Message>,
    ) -> Column<'a, Message> {
        if let Some(dirs) = root_dir.get_directories() {
            for (key, directory) in dirs.iter() {
                if let Some(dir_name) = key.to_str() {
                    if let Some(dir_metadata) = directory.get_metadata() {
                        let row = self.insert_formatted_metadata(dir_name, dir_metadata, 1);
                        column = column.push(
                            button(row)
                                .on_press(Message::MoveDownDirectory(OsString::from(key)))
                                .padding(10)
                                .style(directory_button_style),
                        );
                    }
                }
            }
        }
        column
    }

    fn insert_drop_down_directories<'a>(
        &'a self,
        selected_directory_key: &'a OsStr,
        path_stack: &PathBuf,
        mut column: Column<'a, Message>,
    ) -> Column<'a, Message> {
        let mut path_stack = PathBuf::from(&path_stack);

        if let Some(_last) = path_stack.iter().last() {
            path_stack.push(selected_directory_key);
        }

        if let Some(key) = selected_directory_key.to_str() {
            column = column.push(
                button(key)
                    .width(500)
                    .padding(5)
                    .style(directory_button_style)
                    .on_press(Message::DropDownDirectory(PathBuf::from(&path_stack))),
            );
        }
        column
    }

    fn insert_files<'a>(
        &self,
        root_dir: &'a Directory,
        mut column: Column<'a, Message>,
    ) -> Column<'a, Message> {
        if let Some(files) = root_dir.get_files() {
            for (key, file) in files.iter() {
                if let Some(file_name) = key.to_str() {
                    if let Some(file_metadata) = file.get_metadata() {
                        let row = self.insert_formatted_metadata(file_name, file_metadata, 1);
                        column = column.push(container(row).padding(10));
                    }
                }
            }
        }
        column
    }

    fn insert_drop_down_files<'a>(
        &'a self,
        root_dir: &'a Directory,
        mut column: Column<'a, Message>,
    ) -> Column<'a, Message> {
        if let Some(files) = root_dir.get_files() {
            for (key, _) in files.iter() {
                if let Some(file_name) = key.to_str() {
                    column = column.push(container(file_name).padding(5));
                }
            }
        }
        column
    }

    fn insert_formatted_metadata<'a>(
        &self,
        name: &'a str,
        metadata: &Metadata,
        fill_portion_amount: u16,
    ) -> Row<'a, Message> {
        let mut row = Row::new();
        row = row.push(text(name).width(FillPortion(fill_portion_amount)));
        if let Some(created) = metadata.get_created() {
            let formatted = created.format("%Y-%m-%d %H:%M:%S").to_string();
            row = row.push(text(formatted).width(FillPortion(fill_portion_amount)));
        }

        if let Some(accessed) = metadata.get_accessed() {
            let formatted = accessed.format("%Y-%m-%d %H:%M:%S").to_string();
            row = row.push(text(formatted).width(FillPortion(fill_portion_amount)));
        }
        if let Some(modified) = metadata.get_modified() {
            let formatted = modified.format("%Y-%m-%d %H:%M:%S").to_string();
            row = row.push(text(formatted).width(FillPortion(fill_portion_amount)));
        }

        if metadata.get_readonly() {
            row = row.push(text("No permission").width(FillPortion(fill_portion_amount)));
        } else {
            row = row.push(text("Allowed").width(FillPortion(fill_portion_amount)));
        }

        if let Some(size) = metadata.get_size() {
            let (divided_size, postfix) = round_size(size);
            let formatted_size = format!("{} {}", divided_size, postfix);
            row = row.push(text(formatted_size).width(FillPortion(fill_portion_amount)));
        } else {
            row = row.push(text("-").width(FillPortion(fill_portion_amount)));
        }
        row
    }
}

const KB: f64 = 1_000.0;
const MB: f64 = 1_000_000.0;
const GB: f64 = 1_000_000_000.0;
const TB: f64 = 1_000_000_000_000.0;
const PB: f64 = 1_000_000_000_000_000.0;
const EB: f64 = 1_000_000_000_000_000_000.0;

fn round_size(size: f64) -> (f64, String) {
    let mut divided_size = size;
    let mut postfix = String::from("B");

    if size > EB {
        divided_size /= EB;
        postfix = String::from("EB");
    } else if size > PB {
        divided_size /= PB;
        postfix = String::from("PB")
    } else if size > TB {
        divided_size /= TB;
        postfix = String::from("TB");
    } else if size > GB {
        divided_size /= GB;
        postfix = String::from("GB");
    } else if size > MB {
        divided_size /= MB;
        postfix = String::from("MB");
    } else if size > KB {
        divided_size /= KB;
        postfix = String::from("KB");
    }
    divided_size = (divided_size * 10.0).ceil() / 10.0;
    (divided_size, postfix)
}

fn directory_button_style(_: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => {
            let mut style = button::Style::default().with_background(Background::Color(
                get_directory_button_background_color(1.0),
            ));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
        button::Status::Hovered => {
            let mut style = button::Style::default().with_background(Background::Color(
                get_directory_button_background_color(0.7),
            ));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
        button::Status::Disabled => {
            let mut style = button::Style::default().with_background(Background::Color(
                get_directory_button_background_color(0.1),
            ));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
        button::Status::Pressed => {
            let mut style = button::Style::default().with_background(Background::Color(
                get_directory_button_background_color(0.4),
            ));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
    }
}

fn file_button_style(_: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => {
            let mut style = button::Style::default()
                .with_background(Background::Color(get_file_button_background_color(1.0)));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
        button::Status::Hovered => {
            let mut style = button::Style::default()
                .with_background(Background::Color(get_file_button_background_color(0.7)));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
        button::Status::Disabled => {
            let mut style = button::Style::default()
                .with_background(Background::Color(get_file_button_background_color(0.1)));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
        button::Status::Pressed => {
            let mut style = button::Style::default()
                .with_background(Background::Color(get_file_button_background_color(0.7)));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
    }
}

fn get_directory_button_background_color(alpha_value: f32) -> Color {
    Color {
        r: 0.42,
        g: 0.53,
        b: 0.671,
        a: alpha_value,
    }
}

fn get_file_button_background_color(alpha_value: f32) -> Color {
    Color {
        r: 0.4,
        g: 0.4,
        b: 0.4,
        a: alpha_value,
    }
}
