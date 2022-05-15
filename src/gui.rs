use ::image::DynamicImage;
use clipboard_win::{formats, get_clipboard};
use iced::{
    alignment::{Horizontal, Vertical},
    executor,
    pure::{
        widget::{image, Button, Column, Container, Row, Text},
        Application, Element,
    },
    Alignment, Command, Length,
};
use lib::{fgs, ihash};
use rfd::FileDialog;
use std::{fs::ReadDir, path::PathBuf, str::FromStr};

mod style;

#[derive(Copy, Clone, Default)]
pub struct ProgressData {
    total: f32,
    value: f32,
}

pub enum State {
    InProgress(f32),
    Idle,
}

pub struct DirectoryData {
    dir_iter: Option<ReadDir>,
    state: State,
}

pub struct Gui {
    directory_data: DirectoryData,
    progress_data: ProgressData,
    hashstore: fgs::HashStore,
    ruimprint_files: Option<RuimprintFile>,
    found_paths: Vec<String>,
    found_images: Vec<image::Handle>,
    image_to_process: DynamicImage,
    image: image::Handle,
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
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Gui {
                directory_data: DirectoryData {
                    dir_iter: None,
                    state: State::Idle,
                },
                progress_data: ProgressData::default(),
                hashstore: fgs::HashStore::new(),
                ruimprint_files: None,
                image_to_process: DynamicImage::new_rgb8(2, 2),
                found_paths: vec![],
                found_images: vec![],
                image: image::Handle::from_memory(include_bytes!("../icon.png").to_vec()),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Image Fingerprint")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::AddFile => {
                if let Some(file) = FileDialog::new()
                    .add_filter("fingerprint storage", &["json"])
                    .pick_file()
                {
                    self.hashstore = fgs::HashStore::from_file(file.to_str().unwrap()).unwrap();
                    self.ruimprint_files = Some(RuimprintFile::new(file))
                }
            }
            Message::PasteImage => {
                self.image = if let Ok(data) = get_clipboard(formats::Bitmap) {
                    self.image_to_process = ::image::load_from_memory(&data).unwrap();
                    image::Handle::from_memory(data)
                } else {
                    self.image.clone()
                }
            }
            Message::ClearImage => {
                self.image = image::Handle::from_memory(include_bytes!("../icon.png").to_vec())
            }
            Message::SaveImage => {
                if let Some(path) = FileDialog::new().add_filter("", &["png"]).save_file() {
                    let spath = path.to_str().unwrap();
                    match self.image_to_process.save(spath) {
                        Ok(_) => {
                            let hash = ihash::dhash(&self.image_to_process);
                            self.hashstore.add_hash(&hash, &spath);
                            let _ = self.hashstore.save();
                        }
                        Err(_) => {}
                    }
                }
            }
            Message::Search => {
                let hash = ihash::dhash(&self.image_to_process);
                self.found_paths = self.hashstore.find_many(&hash, 5);
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
                    self.ruimprint_files =
                        Some(RuimprintFile::new(PathBuf::from_str(spath).unwrap()));
                }
            }
            Message::HashExistingImage => {
                if let Some(path) = FileDialog::new().pick_file() {
                    if let Ok(image) = ::image::open(&path) {
                        let hash = ihash::dhash(&image);
                        self.hashstore.add_hash(&hash, &path.to_str().unwrap());
                        let _ = self.hashstore.save();
                    }
                }
            }
            // See about splitting this up into finer grained messages so it doesn't block the main thread
            Message::HashDirectory => {
                if let Some(path) = FileDialog::new().pick_folder() {
                    if let Ok(dir_iter) = std::fs::read_dir(path) {
                        // self.directory_data = DirectoryData {
                        //     dir_iter: Some(dir_iter),
                        //     state: State::InProgress(0.0)
                        // };
                        for entry in dir_iter {
                            let entry = entry.unwrap();
                            if entry.file_type().unwrap().is_dir() {
                                continue;
                            }
                            let spath = entry.path();
                            if let Ok(image) = ::image::open(&spath) {
                                let hash = ihash::dhash(&image);
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
            ruimprint_files,
            found_images,
            found_paths,
            ..
        } = self;

        let files_element: Element<_> = match ruimprint_files {
            Some(file) => message(file.file.to_str().unwrap_or("No file")),
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

        let file_list = Column::new()
            .max_width(300)
            .padding(10)
            .spacing(5)
            .push(
                Button::new("Fingerprint Directory")
                    .on_press(Message::HashDirectory)
                    .style(style::Button::Additive),
            )
            .push(
                Button::new("Fingerprint Existing Image")
                    .on_press(Message::HashExistingImage)
                    .style(style::Button::Additive),
            )
            .push(
                Button::new("Save Fingerprints File As")
                    .on_press(Message::SaveHashstoreAs)
                    .style(style::Button::Primary),
            )
            .push(
                Button::new("Open Fingerprints File")
                    .on_press(Message::AddFile)
                    .style(style::Button::Primary),
            )
            .push(files_element);

        let image_viewer = Column::new()
            .max_width(500)
            .padding(10)
            .spacing(5)
            .align_items(Alignment::Center)
            .push(
                Row::new()
                    .spacing(5)
                    .push(
                        Button::new("Clear")
                            .on_press(Message::ClearImage)
                            .style(style::Button::Destructive),
                    )
                    .push(
                        Button::new("Paste Image")
                            .on_press(Message::PasteImage)
                            .style(style::Button::Primary),
                    ),
            )
            .push(
                Button::new("Save Image and Add Fingerprint")
                    .on_press(Message::SaveImage)
                    .style(style::Button::Additive),
            )
            .push(image::Image::new(self.image.clone()));

        let fingerprint_pane = Column::new()
            .max_width(500)
            .padding(10)
            .spacing(5)
            .align_items(Alignment::Center)
            .push(
                Button::new("Search")
                    .on_press(Message::Search)
                    .style(style::Button::Primary),
            )
            .push(image_results);

        // let progress_bar = ProgressBar::new(0f32..=progress_data.total, progress_data.value);

        let row = Row::new()
            .push(file_list)
            .push(image_viewer)
            .push(fingerprint_pane);

        let col = Column::new().push(row);

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

#[derive(Debug, Clone)]
struct RuimprintFile {
    file: PathBuf,
    marked: bool,
    state: RuimprintFileState,
}

#[derive(Debug, Clone)]
pub enum RuimprintFileState {
    Idle,
}

impl Default for RuimprintFileState {
    fn default() -> Self {
        RuimprintFileState::Idle
    }
}

impl RuimprintFile {
    fn new(file: PathBuf) -> Self {
        RuimprintFile {
            marked: false,
            file,
            state: RuimprintFileState::Idle,
        }
    }
}

