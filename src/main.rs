#![allow(dead_code)]

use iced::{Application, Settings, window};
use image::io::Reader as ImageReader;
use image::{DynamicImage};

mod gui;

fn main() -> iced::Result {
    let icon: Vec<u8> = open("./icon.png").as_rgba8().unwrap().clone().into_raw();
    


    gui::Gui::run(Settings {
        window: window::Settings {
            size: (1300, 800),
            icon: Some(window::icon::Icon::from_rgba(icon, 128, 128).unwrap()),
            ..Default::default()
        },
        ..Settings::default()
    })
}


fn open(path: &str) -> DynamicImage {
    ImageReader::open(path).unwrap().decode().unwrap()
}
