use std::{
    collections::BTreeSet,
    ffi::{OsStr, OsString},
    path::{Iter, PathBuf},
};

use iced::{
    widget::{
        button, checkbox, column, container, row, scrollable, text, text_input, Column, Container,
        Row,
    },
    Background, Color,
    Length::{Fill, FillPortion},
    Theme,
};

use crate::{
    app::{App, Message},
    directory::Directory,
    metadata::Metadata,
};
pub struct CheckboxStates {
    pub organize_by_filetype: bool,
    pub organize_by_date: bool,
    pub insert_date_to_file_name: bool,
    pub insert_directory_name_to_file_name: bool,
}

impl Default for CheckboxStates {
    fn default() -> Self {
        Self {
            organize_by_filetype: false,
            organize_by_date: false,
            insert_date_to_file_name: false,
            insert_directory_name_to_file_name: false,
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
                    text("Directory Tree").size(50),
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
        if let Some(path) = app.get_path().to_str() {
            container(column![
                row![text("Selected path: "), text(path)]
                    .spacing(5)
                    .padding(10),
                row![
                    scrollable(self.display_selected_path_content(app)),
                    column![
                        text("Selected files"),
                        self.insert_files_selected(app),
                        self.rules_for_directory(app)
                    ]
                    .spacing(5)
                ]
                .spacing(5)
            ])
        } else {
            container(text("Could not find path"))
        }
    }

    fn rules_for_directory(&self, app: &App) -> Column<Message> {
        column![
            text("Rules"),
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
                checkbox(
                    "Insert date to file name",
                    app.get_checkbox_states().insert_date_to_file_name
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 3) }),
                checkbox(
                    "Insert directory name to file name",
                    app.get_checkbox_states().insert_directory_name_to_file_name
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 4) })
            ]
        ]
    }

    fn insert_files_selected<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let mut column = Column::new();
        for (key, _) in app.get_files_selected() {
            if let Some(file_name) = key.to_str() {
                column = column.push(text(file_name));
            }
        }
        column = column.push(row![
            text_input("New directory name", app.get_new_directory_input())
                .on_input(Message::InputNewDirectoryName),
            button("Create directory with selected items")
                .on_press(Message::CreateDirectoryWithSelectedFiles),
        ]);
        column
    }

    fn display_selected_path_content<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let mut column = Column::new();
        let root = app
            .get_root_directory()
            .get_directory_by_path(app.get_path());
        let path_stack = PathBuf::from(app.get_path());
        let mut path_component_iter: std::slice::Iter<'_, OsString> =
            app.get_directories_selected().iter();
        column = self.display_directories_as_dropdown(
            app,
            root,
            &path_stack,
            &mut path_component_iter,
            column,
        );
        if let Some(files) = root.get_files() {
            for (key, file) in files {
                if let Some(file_name) = key.to_str() {
                    column = column.push(
                        button(file_name)
                            .style(file_button_style)
                            .on_press(Message::SelectFile(OsString::from(&key), file.clone())),
                    )
                }
            }
        }
        column
    }
    // For when all sub directories are already read
    fn display_directories_as_dropdown<'a>(
        &'a self,
        app: &'a App,
        current_directory: &'a Directory,
        path_stack: &PathBuf,
        path_component_iter: &mut std::slice::Iter<'_, OsString>,
        mut column: Column<'a, Message>,
    ) -> Column<'a, Message> {
        if let Some(next) = path_component_iter.next() {
            if let Some(directories) = current_directory.get_directories() {
                for (key, directory) in directories {
                    if let Some(directory_name) = key.to_str() {
                        column = column.push(
                            button(directory_name)
                                .on_press(Message::SelectDirectory(OsString::from(key))),
                        );
                    }

                    if next == key {
                        let mut new_column = Column::new();
                        new_column = self.display_directories_as_dropdown(
                            app,
                            directory,
                            path_stack,
                            path_component_iter,
                            new_column,
                        );
                        new_column = new_column.padding(10);
                        if let Some(files) = directory.get_files() {
                            for (key, _file) in files {
                                if let Some(file_name) = key.to_str() {
                                    column = column.push(text(file_name))
                                }
                            }
                        }
                        column = column.push(new_column);
                    }
                }
            }
        } else {
            if let Some(directories) = current_directory.get_directories() {
                for (key, _) in directories {
                    if let Some(directory_name) = key.to_str() {
                        column = column.push(
                            button(directory_name)
                                .on_press(Message::SelectDirectory(OsString::from(key))),
                        );
                    }
                }
            }
        }
        column
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
                    &path,
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
        full_path: &PathBuf,
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
                                full_path,
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
                    column = self.insert_drop_down_directories(dir_key, full_path, column);
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
        full_path: &PathBuf,
        mut column: Column<'a, Message>,
    ) -> Column<'a, Message> {
        let mut path_stack = PathBuf::from(&full_path);

        if let Some(last) = path_stack.iter().last() {
            if last != selected_directory_key {
                path_stack.push(selected_directory_key);
            }
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
