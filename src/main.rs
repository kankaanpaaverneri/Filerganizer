mod app;
mod app_util;
mod directory;
mod file;
mod filesystem;
mod layouts;
mod metadata;
mod organize_files;
mod save_directory;
mod subscription;

use app::App;
use iced::Theme;

fn main() -> iced::Result {
    let mut window_settings = iced::window::Settings::default();
    window_settings.size = iced::Size::new(1000.0, 700.0);
    iced::application("Filerganizer", App::update, App::view)
        .window(window_settings)
        .subscription(subscription::subscription)
        .theme(theme)
        .run()
}

fn theme(_: &App) -> Theme {
    Theme::Dark
}
