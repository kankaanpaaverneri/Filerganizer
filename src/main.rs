mod app;
mod directory;
mod file;
mod layouts;
mod metadata;

use app::App;
use iced;

fn main() -> iced::Result {
    iced::run("Filerganizer", App::update, App::view)
}
