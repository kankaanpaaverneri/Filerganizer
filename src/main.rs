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

use crate::directory::system_dir;
const ICON: &str = "icon.png";

fn main() -> iced::Result {
    let mut window_settings = iced::window::Settings::default();
    window_settings.size = iced::Size::new(1000.0, 700.0);
    let mut current_dir = system_dir::get_current_dir();

    if let Some(current_dir_path) = &mut current_dir {
        current_dir_path.push(ICON);
        let icon = iced::window::icon::from_file(current_dir_path);
        match icon {
            Ok(icon) => {
                window_settings.icon = Some(icon);
            }
            Err(error) => {
                eprintln!("Failed to load icon: {}", error);
            }
        }
    }
    iced::application("Filerganizer", App::update, App::view)
        .window(window_settings)
        .subscription(subscription::subscription)
        .theme(theme)
        .run()
}

fn theme(_: &App) -> Theme {
    Theme::Dark
}
