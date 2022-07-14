use ::image::{DynamicImage, imageops::FilterType};
use clipboard_win::{formats, get_clipboard};
use iced::{
    alignment::{Horizontal, Vertical},
    executor,
    pure::{
        widget::{image, Button, Column, Container, Row, Text},
        Application, Element,
    },
    Alignment::Center,
    Command, Length, ProgressBar,
};
use lib::{fgs, ihash::{dhash, dhash_rotations}};
use rfd::FileDialog;
use std::{path::PathBuf, str::FromStr};

mod style;
mod hash_dir;

#[derive(Copy, Clone, Default)]
pub struct ProgressData {
    visible: bool,
    total: f32,
    value: f32,
}

pub struct Gui {
    progress_data: ProgressData,
    hashstore: fgs::HashStore,
    fingerprint_store_path: Option<PathBuf>,
    found_paths: Vec<String>,
    found_images: Vec<image::Handle>,
    image_to_process: DynamicImage,
    pasted_image: image::Handle,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartProgress(f32),
    IncrementProgress(f32),
    SaveImage,
    ClearImage,
    SaveHashstoreAs,
    HashDirectory,
    HashDirectoryStep,
    HashExistingImage,
    AddFile,
    PasteImage,
    Search,
}

impl Application for Gui {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Gui {
                progress_data: ProgressData::default(),
                hashstore: fgs::HashStore::new(),
                fingerprint_store_path: None,
                image_to_process: DynamicImage::new_rgb8(2, 2),
                found_paths: vec![],
                found_images: vec![],
                pasted_image: image::Handle::from_memory(include_bytes!("../icon.png").to_vec()),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Image Fingerprint v1.1")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::AddFile => {
                if let Some(file) = FileDialog::new()
                    .add_filter("Hash storage", &["json"])
                    .pick_file()
                {
                    self.hashstore = fgs::HashStore::from_file(file.to_str().unwrap()).unwrap();
                    self.fingerprint_store_path = Some(file)
                }
            }
            Message::PasteImage => {
                self.pasted_image = if let Ok(data) = get_clipboard(formats::Bitmap) {
                    self.image_to_process = ::image::load_from_memory(&data).unwrap();
                    image::Handle::from_memory(data)
                } else {
                    self.pasted_image.clone()
                }
            }
            Message::ClearImage => {
                self.pasted_image =
                    image::Handle::from_memory(include_bytes!("../icon.png").to_vec())
            }
            Message::SaveImage => {
                if let Some(path) = FileDialog::new().add_filter("", &["png"]).save_file() {
                    let spath = path.to_str().unwrap();
                    match self.image_to_process.save(spath) {
                        Ok(_) => {
                            let hash = dhash(&self.image_to_process);
                            self.hashstore.add_hash(&hash, &spath);
                            let _ = self.hashstore.save();
                        }
                        Err(_) => {}
                    }
                }
            }
            Message::Search => {
                let hashes = dhash_rotations(&self.image_to_process, FilterType::Triangle);
                self.found_paths = self.hashstore.find_many(&hashes, 5);
                self.found_images.clear();
                for path in self.found_paths.iter() {
                    let im = image::Handle::from_path(path);
                    self.found_images.push(im);
                }
            }
            Message::SaveHashstoreAs => {
                if let Some(path) = FileDialog::new().add_filter("", &["json"]).save_file() {
                    let spath = path.to_str().unwrap();
                    let _ = self.hashstore.to_file(spath);
                    self.fingerprint_store_path = Some(PathBuf::from_str(spath).unwrap());
                }
            }
            Message::HashExistingImage => {
                if let Some(paths) = FileDialog::new().pick_files() {
                    for path in paths {
                        if let Ok(image) = ::image::open(&path) {
                            let hash = dhash(&image);
                            self.hashstore.add_hash(&hash, &path.to_str().unwrap());
                        }
                    }
                    let _ = self.hashstore.save();
                }
            }
            Message::HashDirectory => {
                if let Some(path) = FileDialog::new().pick_folder() {
                    if let Ok(dir_iter) = std::fs::read_dir(path) {
                        for entry in dir_iter {
                            let entry = entry.unwrap();
                            if entry.file_type().unwrap().is_dir() {
                                continue;
                            }
                            let spath = entry.path();
                            if let Ok(image) = ::image::open(&spath) {
                                let hash = dhash(&image);
                                self.hashstore.add_hash(&hash, &spath.to_str().unwrap());
                            }
                        }
                        let _ = self.hashstore.save();
                    }
                }
            }
            Message::HashDirectoryStep => {}
            Message::StartProgress(target) => {
                self.progress_data.total = target;
                self.progress_data.value = 0f32;
            }
            Message::IncrementProgress(amount) => {
                self.progress_data.value += amount;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let Gui {
            progress_data,
            fingerprint_store_path,
            found_images,
            found_paths,
            ..
        } = self;

        let active_store_filename: Element<_> = match fingerprint_store_path {
            Some(file) => message(file.to_str().unwrap_or("No file")),
            None => message("No fingerprint file specified"),
        };

        let image_results: Element<_> = if !found_images.is_empty() {
            found_images
                .iter()
                .enumerate()
                .fold(Column::new(), |col, (i, image)| {
                    col.push(
                        Row::new()
                            .push(image::Image::new(image.clone()).height(Length::Units(150)))
                            .push(message(found_paths.get(i).unwrap())),
                    )
                })
                .into()
        } else {
            message("No images found")
        };

        let file_controls_list = Column::new()
            .max_width(300)
            .padding(10)
            .spacing(5)
            .align_items(Center)
            .push(
                Button::new(button_text("Fingerprint Directory Contents"))
                    .on_press(Message::HashDirectory)
                    .style(style::Button::Additive)
                    .width(Length::Fill),
            )
            .push(
                Button::new(button_text("Fingerprint Existing Image"))
                    .on_press(Message::HashExistingImage)
                    .style(style::Button::Additive)
                    .width(Length::Fill),
            )
            .push(
                Button::new(button_text("Save Fingerprints File As"))
                    .on_press(Message::SaveHashstoreAs)
                    .style(style::Button::Primary)
                    .width(Length::Fill),
            )
            .push(
                Button::new(button_text("Open Fingerprints File"))
                    .on_press(Message::AddFile)
                    .style(style::Button::Primary)
                    .width(Length::Fill),
            )
            .push(active_store_filename);

        let image_viewer = Column::new()
            .max_width(500)
            .padding(10)
            .spacing(5)
            .align_items(Center)
            .push(
                Row::new()
                    .spacing(5)
                    .push(
                        Button::new(button_text("Clear"))
                            .on_press(Message::ClearImage)
                            .style(style::Button::Destructive)
                            .width(Length::Fill),
                    )
                    .push(
                        Button::new(button_text("Paste Image"))
                            .on_press(Message::PasteImage)
                            .style(style::Button::Primary)
                            .width(Length::Fill),
                    ),
            )
            .push(
                Button::new(button_text("Save Image and Add Fingerprint"))
                    .on_press(Message::SaveImage)
                    .style(style::Button::Additive)
                    .width(Length::Fill),
            )
            .push(image::Image::new(self.pasted_image.clone()));

        let fingerprint_pane = Column::new()
            .max_width(500)
            .padding(10)
            .spacing(5)
            .align_items(Center)
            .push(
                Button::new(button_text("Search"))
                    .on_press(Message::Search)
                    .style(style::Button::Primary)
                    .width(Length::Fill),
            )
            .push(image_results);

        let progress_bar = ProgressBar::new(0f32..=progress_data.total, progress_data.value);

        let row = Row::new()
            .push(file_controls_list)
            .push(image_viewer)
            .push(fingerprint_pane);

        let col = match progress_data.visible {
            true => Column::new().push(progress_bar).push(row),
            false => Column::new().push(row),
        };

        Container::new(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .into()
    }
}

fn message<'a>(message: &str) -> Element<'a, Message> {
    Container::new(
        Text::new(message)
            .width(Length::Fill)
            .size(16)
            .vertical_alignment(Vertical::Top)
            .horizontal_alignment(Horizontal::Center)
            .color([0.5, 0.5, 0.5]),
    )
    .width(Length::Fill)
    .center_x()
    .into()
}

fn button_text<'a>(message: &str) -> Element<'a, Message> {
    Container::new(
        Text::new(message)
            .width(Length::Fill)
            .size(20)
            .vertical_alignment(Vertical::Top)
            .horizontal_alignment(Horizontal::Center)
            .color([1_f32, 1_f32, 1_f32]),
    )
    .width(Length::Fill)
    .center_x()
    .into()
}
