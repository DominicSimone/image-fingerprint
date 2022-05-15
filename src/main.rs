#![windows_subsystem = "windows"]

use iced::{Settings, pure::Application, window};
use image;

mod gui;

fn main() -> iced::Result {
    let image_bytes = include_bytes!("../icon.png");
    let icon = image::load_from_memory(image_bytes).unwrap().into_rgba8().to_vec();

    gui::Gui::run(Settings {
        window: window::Settings {
            size: (1300, 800),
            icon: Some(window::icon::Icon::from_rgba(icon, 128, 128).unwrap()),
            ..Default::default()
        },
        ..Settings::default()
    })
}