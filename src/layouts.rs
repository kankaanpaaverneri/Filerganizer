use std::{collections::BTreeSet, ffi::OsString};

use iced::{
    widget::{
        button, column, container, row, scrollable, text, text_input, Column, Container, Row,
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

#[derive(Debug, Clone)]
pub enum Layout {
    Main,
    DirectoryExploringLayout,
}

impl Layout {
    pub fn get_layout<'a>(&self, app: &'a App) -> Container<'a, Message> {
        match self {
            Layout::Main => self.main_layout(app),
            Layout::DirectoryExploringLayout => self.directory_exploring_layout(app),
        }
    }

    fn main_layout<'a>(&self, _: &App) -> Container<'a, Message> {
        container(column![
            text("Filerganizer").size(50),
            row![
                button("Select directory to organize")
                    .style(button_style)
                    .on_press(Message::SwitchLayout(Layout::DirectoryExploringLayout)),
                button("Exit").on_press(Message::Exit).style(button_style)
            ]
            .spacing(10)
        ])
        .center(Fill)
    }

    fn directory_exploring_layout<'a>(&self, app: &'a App) -> Container<'a, Message> {
        if let Some(path) = app.get_path().to_str() {
            container(
                column![
                    text("File explorer").size(50),
                    column![
                        button("Main Menu")
                            .on_press(Message::SwitchLayout(Layout::Main))
                            .style(button_style),
                        row![
                            row![
                                text_input(path, app.get_path_input())
                                    .on_input(Message::TextInput)
                                    .on_submit(Message::SearchPath),
                                button("Search")
                                    .style(button_style)
                                    .on_press(Message::SearchPath)
                            ],
                            row![
                                button("List view").style(button_style),
                                button("Drop down").style(button_style)
                            ],
                        ]
                        .spacing(10),
                        self.insert_external_storage(app),
                        button("Previous")
                            .on_press(Message::MoveUpDirectory)
                            .style(button_style),
                        text(app.get_error()),
                    ]
                    .spacing(5),
                    self.insert_header(),
                    scrollable(
                        column![self.display_directory_contents(app).spacing(5)].spacing(10)
                    )
                ]
                .padding(10),
            )
        } else {
            container(text("Could not find path"))
        }
    }

    fn display_directory_contents<'a>(&self, app: &'a App) -> Column<'a, Message> {
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
                        .style(button_style)
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
                        let row = self.insert_formatted_metadata(dir_name, dir_metadata);
                        column = column.push(
                            button(row)
                                .on_press(Message::MoveDownDirectory(OsString::from(key)))
                                .padding(10)
                                .style(button_style),
                        );
                    }
                }
            }
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
                        let row = self.insert_formatted_metadata(file_name, file_metadata);
                        column = column.push(container(row).padding(10));
                    }
                }
            }
        }
        column
    }

    fn insert_formatted_metadata<'a>(
        &self,
        name: &'a str,
        metadata: &Metadata,
    ) -> Row<'a, Message> {
        let mut row = Row::new();
        row = row.push(text(name).width(FillPortion(1)));
        if let Some(created) = metadata.get_created() {
            let formatted = created.format("%Y-%m-%d %H:%M:%S").to_string();
            row = row.push(text(formatted).width(FillPortion(1)));
        }
        if let Some(accessed) = metadata.get_accessed() {
            let formatted = accessed.format("%Y-%m-%d %H:%M:%S").to_string();
            row = row.push(text(formatted).width(FillPortion(1)));
        }
        if let Some(modified) = metadata.get_modified() {
            let formatted = modified.format("%Y-%m-%d %H:%M:%S").to_string();
            row = row.push(text(formatted).width(FillPortion(1)));
        }

        if metadata.get_readonly() {
            row = row.push(text("No permission").width(FillPortion(1)));
        } else {
            row = row.push(text("Allowed").width(FillPortion(1)));
        }

        if let Some(size) = metadata.get_size() {
            let (divided_size, postfix) = round_size(size);
            let formatted_size = format!("{} {}", divided_size, postfix);
            row = row.push(text(formatted_size).width(FillPortion(1)));
        } else {
            row = row.push(text("-").width(FillPortion(1)));
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

fn button_style(_: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => {
            let mut style = button::Style::default()
                .with_background(Background::Color(get_button_background_color(1.0)));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
        button::Status::Hovered => {
            let mut style = button::Style::default()
                .with_background(Background::Color(get_button_background_color(0.7)));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
        button::Status::Disabled => {
            let mut style = button::Style::default()
                .with_background(Background::Color(get_button_background_color(0.1)));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
        button::Status::Pressed => {
            let mut style = button::Style::default()
                .with_background(Background::Color(get_button_background_color(0.4)));
            style.text_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
            style
        }
    }
}

fn get_button_background_color(alpha_value: f32) -> Color {
    Color {
        r: 0.42,
        g: 0.53,
        b: 0.671,
        a: alpha_value,
    }
}
