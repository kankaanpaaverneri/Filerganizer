use std::{
    collections::BTreeSet,
    ffi::{OsStr, OsString},
    path::{Iter, PathBuf},
};

use iced::{
    alignment::Vertical,
    widget::{
        button, checkbox, column, container, mouse_area, pick_list, radio, row, scrollable, text,
        text_input, Button, Column, Container, Row,
    },
    Alignment::Center,
    Background, Color,
    Length::{Fill, FillPortion},
    Theme,
};

use chrono::{DateTime, Local};

use crate::{
    app::{filename_components, App, Message, ReplacableSelection, SelectedDirectoryRules},
    directory::Directory,
    metadata::{DateType, Metadata},
    organize_files,
};

#[derive(Debug, Clone, PartialEq, Copy, Eq)]
pub enum Replaceable {
    Dash,
    Space,
    Comma,
}

const MAX_REPLACEABLE_OPTIONS: usize = 3;

impl std::fmt::Display for Replaceable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Replaceable::Dash => "Dash",
            Replaceable::Space => "Space",
            Replaceable::Comma => "Comma",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Eq)]
pub enum ReplaceWith {
    Underscore,
    Nothing,
}

impl std::fmt::Display for ReplaceWith {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ReplaceWith::Nothing => "Nothing",
            ReplaceWith::Underscore => "Underscore",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckboxStates {
    pub organize_by_filetype: bool,
    pub organize_by_date: bool,
    pub insert_date_to_file_name: bool,
    pub insert_directory_name_to_file_name: bool,
    pub convert_uppercase_to_lowercase: bool,
    pub replace_character: bool,
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
            convert_uppercase_to_lowercase: false,
            replace_character: false,
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
        convert_uppercase_to_lowercase: bool,
        replace_character: bool,
        use_only_ascii: bool,
        remove_original_file_name: bool,
        add_custom_name: bool,
    ) -> Self {
        Self {
            organize_by_filetype,
            organize_by_date,
            insert_date_to_file_name,
            insert_directory_name_to_file_name,
            convert_uppercase_to_lowercase,
            replace_character,
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

#[derive(Debug, Clone)]
pub enum FileSelectedLocation {
    FromDirectory(PathBuf),
    FromFilesSelected(PathBuf),
}

impl Layout {
    pub fn get_layout<'a>(&'a self, app: &'a App) -> Container<'a, Message> {
        match self {
            Layout::Main => self.main_layout(app),
            _ => self.directory_tree_layout(app),
        }
    }

    fn main_layout<'a>(&self, app: &App) -> Container<'a, Message> {
        let files_have_been_organized = match app.get_files_have_been_organized() {
            true => "Your files have been Filerganized",
            false => "",
        };
        container(column![
            row![text("Filerganizer").size(50)].spacing(10).padding(10),
            row![
                button("Select directory to organize")
                    .style(directory_button_style)
                    .on_press(Message::SwitchLayout(Layout::DirectorySelectionLayout)),
                button("Exit")
                    .on_press(Message::Exit)
                    .style(directory_button_style)
            ]
            .spacing(10)
            .padding(10),
            row![text(files_have_been_organized)
                .color(Color::from_rgb(0.0, 0.5, 0.1))
                .center()
                .size(25)]
            .spacing(10)
            .padding(10)
        ])
        .padding(10)
        .center(Fill)
    }

    fn directory_tree_layout<'a>(&'a self, app: &'a App) -> Container<'a, Message> {
        if let Some(path) = app.get_path().to_str() {
            let mut main_row = Row::new();
            let mut header_column = Column::new();
            header_column = header_column.push(
                button("Main Menu")
                    .on_press(Message::SwitchLayout(Layout::Main))
                    .style(directory_button_style),
            );
            let mut header_column_row = Row::new();
            main_row = main_row.push(self.display_directory_contents(app).spacing(5));
            if let Layout::DirectoryOrganizingLayout = self {
                header_column_row = header_column_row.push(self.insert_search_bar(app, path));
                header_column_row = header_column_row
                    .push(self.insert_directory_view_buttons(app))
                    .spacing(5);
                if !app.get_files_organized().is_empty() {
                    header_column_row =
                        header_column_row.push(button("Commit").on_press(Message::Commit))
                }
                main_row = main_row.push(
                    scrollable(
                        column![
                            self.selected_directory_option(app),
                            self.insert_files_selected(app),
                        ]
                        .padding(10),
                    )
                    .width(FillPortion(2))
                    .spacing(5),
                );
            }
            if let Layout::DirectorySelectionLayout = self {
                header_column_row = header_column_row.push(self.insert_search_bar(app, path));
                header_column_row = header_column_row
                    .push(self.insert_directory_view_buttons(app))
                    .spacing(5);
                header_column_row = header_column_row.push(
                    button("Show Directory rule options")
                        .style(directory_button_style)
                        .on_press(Message::SelectPath),
                );
            }
            header_column = header_column.push(header_column_row);

            container(
                column![
                    header_column,
                    column![
                        self.insert_external_storage(app),
                        button("Previous")
                            .on_press(Message::DropDownDirectory(PathBuf::from(path)))
                            .style(directory_button_style),
                        text(app.get_error()),
                    ]
                    .spacing(5),
                    main_row
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
            let mut directory_content_fill_portion = FillPortion(1);
            if app.get_files_selected().is_empty() {
                directory_content_fill_portion = Fill;
            }

            container(column![
                column![text(app.get_error())],
                select_all_files_button,
                row![
                    scrollable(self.display_selected_path_content(app))
                        .width(directory_content_fill_portion),
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

    fn insert_replaceables<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let mut column = Column::new();
        if !app.get_checkbox_states().replace_character {
            return column;
        }
        for (i, replaceable_rule) in app.get_replaceables().iter().enumerate() {
            column = column.push(
                row![
                    text("Replace"),
                    pick_list(
                        app.get_replaceable_options(),
                        replaceable_rule.get_replaceable_selected(),
                        move |replaceable| Message::SelectReplaceable(replaceable, i),
                    )
                    .width(150),
                    text("With"),
                    pick_list(
                        app.get_replace_with_options(),
                        replaceable_rule.get_replace_with_selected(),
                        move |replace_with| { Message::SelectReplaceWith(replace_with, i) }
                    ),
                    button("Remove").on_press(Message::RemoveReplaceable(i))
                ]
                .spacing(5)
                .padding(5)
                .align_y(Center),
            );
        }
        if !app.get_replaceable_options().is_empty()
            && app.get_replaceables().len() < MAX_REPLACEABLE_OPTIONS
        {
            column = column.push(row![button("Add new").on_press(Message::AddNewReplaceable)]);
        }
        column
    }

    fn rules_for_directory<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
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
        let replaceables = self.insert_replaceables(app);
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
                    "Convert uppercase to lowercase.",
                    app.get_checkbox_states().convert_uppercase_to_lowercase
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 3) }),
                checkbox(
                    "Replace character with.",
                    app.get_checkbox_states().replace_character
                )
                .on_toggle(|toggle| { Message::CheckboxToggled(toggle, 4) }),
                replaceables,
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

    fn get_custom_name_example(&self, app: &App, index_position: &IndexPosition) -> String {
        match index_position {
            IndexPosition::Before => {
                let mut filename_input = String::new();
                filename_input.push_str("1 ");
                if !app.get_filename_input().is_empty() {
                    filename_input.push_str(app.get_filename_input());
                } else {
                    filename_input.push_str("Custom Name");
                }

                self.convert_text_by_checkbox_states(app, filename_input)
            }
            IndexPosition::After => {
                let mut filename_input = String::new();
                if !app.get_filename_input().is_empty() {
                    filename_input.push_str(app.get_filename_input());
                } else {
                    filename_input.push_str("Custom Name");
                }
                filename_input.push_str(" 1");
                self.convert_text_by_checkbox_states(app, filename_input)
            }
        }
    }

    fn convert_text_by_checkbox_states(&self, app: &App, text: String) -> String {
        let mut converted = text;
        if app.get_checkbox_states().convert_uppercase_to_lowercase {
            converted = converted.to_lowercase();
        }
        if app.get_checkbox_states().replace_character {
            for replaceable in app.get_replaceables() {
                if let Some(replace) = replaceable.get_replaceable_selected() {
                    if let Some(replace_with) = replaceable.get_replace_with_selected() {
                        organize_files::replace_character_with(
                            &mut converted,
                            replace,
                            replace_with,
                        );
                    }
                }
            }
        }
        converted
    }

    fn order_of_file_name_components<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let mut column = Column::new();
        let mut row = Row::new();
        let order_of_filename_components = app.get_order_of_filename_components();
        if !order_of_filename_components.is_empty() {
            column = column.push(text("Order of filename components example"));
        }
        for (i, component) in order_of_filename_components.iter().enumerate() {
            let example_component = match component.as_str() {
                filename_components::DATE => {
                    let current_date: DateTime<Local> = Local::now();
                    let formatted = current_date.format("%Y%m%d");
                    formatted.to_string()
                }
                filename_components::ORIGINAL_FILENAME => {
                    let original_filename = String::from("Original Filename");
                    self.convert_text_by_checkbox_states(app, original_filename)
                }
                filename_components::DIRECTORY_NAME => {
                    let mut directory_name = String::new();
                    let new_directory_name = app.get_new_directory_input();
                    if !app.get_new_directory_input().is_empty() {
                        directory_name.push_str(new_directory_name);
                    } else {
                        directory_name.push_str("Directory Name");
                    }
                    self.convert_text_by_checkbox_states(app, directory_name)
                }
                filename_components::CUSTOM_FILE_NAME => {
                    let mut custom_name = String::from("Custom Name");
                    let filename_input = app.get_filename_input();
                    if !filename_input.is_empty() {
                        custom_name = String::from(filename_input);
                    }
                    let mut custom_file_name =
                        self.convert_text_by_checkbox_states(app, custom_name);
                    if let Some(position) = app.get_index_position() {
                        custom_file_name = self.get_custom_name_example(app, &position)
                    }
                    custom_file_name
                }
                _ => String::new(),
            };
            if i > 0 {
                row = row.push(button("swap").on_press(Message::SwapFileNameComponents(i)));
            }
            row = row.push(text(example_component).size(12));
        }
        row = row.push(text(".filetype").size(12));
        row = row.spacing(2).padding(5).align_y(Vertical::Center);
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
                    if let Some(rules) = app.get_selected_directory_rules() {
                        column = column.push(column![
                            text("Rules for selected directory"),
                            self.selected_directory_rules(rules).padding(10)
                        ])
                    }
                    column = column.padding(10).spacing(10);
                }
            }
        }
        column
    }

    fn selected_directory_rules<'a>(
        &'a self,
        rules: &'a SelectedDirectoryRules,
    ) -> Column<'a, Message> {
        let mut column = Column::new();
        let checkbox_states = rules.get_checkbox_states();
        let replaceables = rules.get_replaceables();
        column =
            column.push(self.insert_checkbox_states_for_directory(checkbox_states, replaceables));
        let date_type_selected = rules.get_date_type_selected();
        column = column.push(self.insert_date_type_selected_for_directory(date_type_selected));
        let index_position = rules.get_index_position();
        column = column.push(self.insert_index_position_for_directory(index_position));
        let order_of_filename_components = rules.get_order_of_filename_components();
        column = column.push(
            self.insert_order_of_filename_components_for_directory(order_of_filename_components),
        );
        let custom_filename = rules.get_custom_filename();
        column = column.push(self.insert_custom_filename(custom_filename));
        column
    }

    fn insert_custom_filename<'a>(&'a self, custom_filename: &'a str) -> Column<'a, Message> {
        let mut column = Column::new();
        column = column.push(row![text("Custom filename: "), text(custom_filename)]);

        column
    }

    fn insert_order_of_filename_components_for_directory<'a>(
        &'a self,
        order_of_filename_components: &'a Vec<String>,
    ) -> Column<'a, Message> {
        let mut column = Column::new();
        column = column.push(text("Order of filename components: "));
        let mut row = Row::new();
        for component in order_of_filename_components {
            row = row.push(text(component));
        }
        row = row.spacing(10);
        column = column.push(row);
        column
    }

    fn insert_index_position_for_directory(
        &self,
        index_position: &Option<IndexPosition>,
    ) -> Column<Message> {
        let mut column = Column::new();
        if let Some(index_position) = index_position {
            let index_position_text = match index_position {
                IndexPosition::After => "After",
                IndexPosition::Before => "Before",
            };
            column = column.push(row![text("Index position: "), text(index_position_text)]);
        }
        column
    }

    fn insert_date_type_selected_for_directory(
        &self,
        date_type_selected: &Option<DateType>,
    ) -> Column<Message> {
        let mut column = Column::new();
        if let Some(date_type) = date_type_selected {
            let date_type_text = match date_type {
                DateType::Created => "Created",
                DateType::Accessed => "Accessed",
                DateType::Modified => "Modified",
            };
            column = column.push(row![text("Date type: "), text(date_type_text)]);
        }
        column
    }

    fn insert_checkbox_states_for_directory(
        &self,
        checkbox_states: &CheckboxStates,
        replaceables: &Vec<ReplacableSelection>,
    ) -> Column<Message> {
        let mut column = Column::new();
        let checkbox_state_array: [&bool; 9] = [
            &checkbox_states.organize_by_filetype,
            &checkbox_states.organize_by_date,
            &checkbox_states.convert_uppercase_to_lowercase,
            &checkbox_states.replace_character,
            &checkbox_states.use_only_ascii,
            &checkbox_states.insert_directory_name_to_file_name,
            &checkbox_states.insert_date_to_file_name,
            &checkbox_states.remove_original_file_name,
            &checkbox_states.add_custom_name,
        ];
        let checkbox_text: [&str; 9] = [
            "Organize by filetype",
            "Organize by date",
            "Convert uppercase to lowercase",
            "Replace character",
            "Use only ascii",
            "Insert directory name to filename",
            "Insert date to filename",
            "Remove original filename",
            "Add a custom name",
        ];
        for (i, checkbox_state) in checkbox_state_array.iter().enumerate() {
            if **checkbox_state {
                column = column.push(text(checkbox_text[i]));
                if i == 3 {
                    column = column.push(self.insert_replaceable_rules(replaceables).padding(10));
                }
            }
        }
        column
    }

    fn insert_replaceable_rules(&self, replaceables: &Vec<ReplacableSelection>) -> Column<Message> {
        let mut column = Column::new();
        for replaceable in replaceables {
            if let Some(replace) = replaceable.get_replaceable_selected() {
                if let Some(replace_with) = replaceable.get_replace_with_selected() {
                    let replace_text = match replace {
                        Replaceable::Dash => "Dash",
                        Replaceable::Space => "Space",
                        Replaceable::Comma => "Comma",
                    };

                    let replace_with_text = match replace_with {
                        ReplaceWith::Nothing => "Nothing",
                        ReplaceWith::Underscore => "Underscore",
                    };
                    let row = row![
                        text!("Replace "),
                        text(replace_text),
                        text(" With "),
                        text(replace_with_text)
                    ];
                    column = column.push(row);
                }
            }
        }
        column
    }

    fn insert_files_selected<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let mut column = Column::new();

        let mut path_stack = PathBuf::from(app.get_path());
        for (i, (key, file)) in app.get_files_selected().iter().enumerate() {
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
                let files_selected_count = app.get_files_selected().len();
                let formatted_count = format!("{}", files_selected_count);
                column = column.push(text(formatted_count));
            }
            if let Some(file_name) = key.to_str() {
                path_stack.push(key);
                if let Some(metadata) = file.get_metadata() {
                    if let Some(origin_path) = metadata.get_origin_path() {
                        column = column.push(
                            mouse_area(button(file_name).style(file_button_style).on_press(
                                Message::SelectFile(FileSelectedLocation::FromFilesSelected(
                                    origin_path.to_owned(),
                                )),
                            ))
                            .on_right_press(
                                Message::SelectMultipleFiles(
                                    i,
                                    FileSelectedLocation::FromFilesSelected(origin_path),
                                ),
                            ),
                        );
                    }
                }
                path_stack.pop();
            }
        }
        column = column.spacing(10).padding(10);

        column
    }

    fn display_selected_path_content<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let current_directory = app.get_root_directory();
        let path = PathBuf::from(app.get_path());
        let mut path_iter = path.iter();
        path_iter.next();

        let mut column =
            self.display_directories_as_dropdown(current_directory, path_iter, path.clone());

        // Display files in the root directory
        //column = self.append_files_to_column(current_directory, &mut path, column, true);
        column = column.padding(10).spacing(10);
        column
    }

    // For when directory has been selected
    fn display_directories_as_dropdown<'a>(
        &'a self,
        current_directory: &'a Directory,
        mut path_iter: Iter,
        mut path_stack: PathBuf,
    ) -> Column<'a, Message> {
        let mut column = Column::new();

        column
    }

    fn append_files_to_column<'a>(
        &'a self,
        root: &'a Directory,
        path: &mut PathBuf,
        mut column: Column<'a, Message>,
        files_selectable: bool,
    ) -> Column<'a, Message> {
        if let Some(files) = root.get_files() {
            for (i, (key, value)) in files.iter().enumerate() {
                if let Some(file_name) = key.to_str() {
                    path.push(key);
                    if let Some(metadata) = value.get_metadata() {
                        if let Some(origin_path) = metadata.get_origin_path() {
                            if files_selectable {
                                column = column.push(
                                    mouse_area(
                                        button(file_name)
                                            .style(file_button_style)
                                            .width(Fill)
                                            .on_press(Message::SelectFile(
                                                FileSelectedLocation::FromDirectory(
                                                    path.to_owned(),
                                                ),
                                            )),
                                    )
                                    .on_right_press(
                                        Message::SelectMultipleFiles(
                                            i,
                                            FileSelectedLocation::FromDirectory(path.to_owned()),
                                        ),
                                    ),
                                );
                            } else {
                                column = column.push(
                                    mouse_area(
                                        button(file_name)
                                            .width(Fill)
                                            .style(inner_file_button_style)
                                            .on_press(Message::SelectFile(
                                                FileSelectedLocation::FromDirectory(
                                                    path.to_owned(),
                                                ),
                                            )),
                                    )
                                    .on_right_press(
                                        Message::SelectMultipleFiles(
                                            i,
                                            FileSelectedLocation::FromDirectory(path.to_owned()),
                                        ),
                                    ),
                                );
                            }
                        }
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
                .id(app.get_path_input_id())
                .on_input(Message::TextInput)
                .on_submit(Message::SearchPath(true)),
            button("Search")
                .style(directory_button_style)
                .on_press(Message::SearchPath(false))
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
                skip_prefix_in_path(&mut path_iter, &mut path_stack);
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
                            new_column = new_column.padding(10);
                            new_column = new_column.spacing(10);
                            new_column =
                                self.insert_drop_down_files(path_stack, selected, new_column);
                            path_stack.pop();
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

    fn display_directory_contents_as_list<'a>(&'a self, app: &'a App) -> Column<'a, Message> {
        let root_directory = app.get_root_directory();
        let path = app.get_path();
        let mut path_stack = PathBuf::new();
        let column = self.insert_directory_contents_as_list(root_directory, &path, &mut path_stack);
        column
    }

    fn insert_directory_contents_as_list<'a>(
        &self,
        current_directory: &'a Directory,
        full_path: &PathBuf,
        path_stack: &mut PathBuf,
    ) -> Column<'a, Message> {
        let mut column = Column::new();
        let mut dir = current_directory;
        for (i, component) in full_path.components().enumerate() {
            match std::env::consts::OS {
                "windows" => {
                    if i <= 1 {
                        path_stack.push(component.as_os_str());
                        continue;
                    }
                }
                "macos" | "linux" => {
                    if i == 0 {
                        path_stack.push(component.as_os_str());
                        continue;
                    }
                }
                _ => {}
            }
            if let Some(directories) = dir.get_directories() {
                for (key, value) in directories {
                    if key == component.as_os_str() {
                        path_stack.push(key);
                        dir = value;
                    }
                }
            }
        }
        if let Some(directories) = dir.get_directories() {
            for key in directories.keys() {
                path_stack.push(key);
                if let Some(dir_name) = key.to_str() {
                    column = column.push(
                        button(dir_name)
                            .style(directory_button_style)
                            .on_press(Message::DropDownDirectory(path_stack.to_owned())),
                    );
                }
                path_stack.pop();
            }
        }
        if let Some(files) = dir.get_files() {
            for (i, key) in files.keys().enumerate() {
                path_stack.push(key);
                if let Some(file_name) = key.to_str() {
                    column = column.push(
                        mouse_area(button(file_name).style(file_button_style).on_press(
                            Message::SelectFile(FileSelectedLocation::FromDirectory(
                                path_stack.to_owned(),
                            )),
                        ))
                        .on_right_press(Message::SelectMultipleFiles(
                            i,
                            FileSelectedLocation::FromDirectory(path_stack.to_owned()),
                        )),
                    )
                }
                path_stack.pop();
            }
        }
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
        current_directory: &'a Directory,
        path: &PathBuf,
        mut column: Column<'a, Message>,
    ) -> Column<'a, Message> {
        if let Some(dirs) = current_directory.get_directories() {
            for (key, directory) in dirs.iter() {
                let mut path = PathBuf::from(path);
                path.push(key);
                if let Some(dir_name) = key.to_str() {
                    if let Some(dir_metadata) = directory.get_metadata() {
                        let row = self.insert_formatted_metadata(dir_name, dir_metadata, 1);
                        column = column.push(
                            button(row)
                                .on_press(Message::DropDownDirectory(path))
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

        path_stack.push(selected_directory_key);

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
        file_path: &PathBuf,
        mut column: Column<'a, Message>,
    ) -> Column<'a, Message> {
        if let Some(files) = root_dir.get_files() {
            let mut iterator = 0;
            for (key, file) in files.iter() {
                if let Some(file_name) = key.to_str() {
                    let mut file_path = PathBuf::from(file_path);
                    file_path.push(file_name);
                    if let Some(file_metadata) = file.get_metadata() {
                        let row = self
                            .insert_formatted_metadata(file_name, file_metadata, 1)
                            .padding(10);
                        let button = Button::new(row).style(file_button_style).on_press(
                            Message::SelectFile(FileSelectedLocation::FromDirectory(
                                file_path.to_owned(),
                            )),
                        );
                        column = column.push(mouse_area(button).on_right_press(
                            Message::SelectMultipleFiles(
                                iterator,
                                FileSelectedLocation::FromDirectory(file_path.to_owned()),
                            ),
                        ));
                    }
                }
                iterator += 1;
            }
        }
        column
    }

    fn insert_drop_down_files<'a>(
        &'a self,
        current_path: &PathBuf,
        selected: &'a Directory,
        mut column: Column<'a, Message>,
    ) -> Column<'a, Message> {
        if let Some(files) = selected.get_files() {
            let mut iterator = 0;
            for (key, _value) in files.iter() {
                if let Some(file_name) = key.to_str() {
                    let mut path_to_file = PathBuf::from(current_path);
                    path_to_file.push(file_name);
                    column = column.push(
                        mouse_area(
                            button(file_name)
                                .style(file_button_style)
                                .on_press(Message::SelectFile(FileSelectedLocation::FromDirectory(
                                    path_to_file.to_owned(),
                                )))
                                .padding(5),
                        )
                        .on_right_press(Message::SelectMultipleFiles(
                            iterator,
                            FileSelectedLocation::FromDirectory(path_to_file.to_owned()),
                        )),
                    );
                }
                iterator += 1;
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

fn inner_file_button_style(_: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => {
            let mut style = button::Style::default()
                .with_background(Background::Color(get_file_button_background_color(0.0)));
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
                .with_background(Background::Color(get_file_button_background_color(0.0)));
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

fn skip_prefix_in_path(path_iter: &mut Iter<'_>, path_stack: &mut PathBuf) {
    if let Some(root) = path_iter.next() {
        path_stack.push(root);
        if let std::env::consts::OS = "windows" {
            if let Some(next) = path_iter.next() {
                path_stack.push(next);
            }
        }
    }
}
