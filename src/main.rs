mod app;
mod directory;
mod file;
mod layouts;
mod metadata;

use app::App;
use iced;
use iced::Theme;

fn main() -> iced::Result {
    iced::application("Filerganizer", App::update, App::view).theme(theme).run()
}

fn theme(_: &App) -> Theme {
    Theme::Dark
}
