use crate::app::Message;
use crate::app::App;
use iced::Subscription;
use iced::keyboard::{on_key_press, Key, Modifiers, key};

fn key_press(key: Key, _: Modifiers) -> Option<Message> {
   match key {
        Key::Named(named) => {
            match named {
                key::Named::Tab => Some(Message::TabKeyPressed), 
                _ => None
            }
        },
        _ => None
   } 
}

pub fn subscription(_: &App) -> Subscription<Message> {
   on_key_press(key_press) 
}

