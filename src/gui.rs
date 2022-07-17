use ::image::{imageops::FilterType, DynamicImage};
use clipboard_win::{formats, get_clipboard};
use iced::{
    alignment::{Horizontal, Vertical},
    executor,
    pure::{
        widget::{image, Button, Column, Container, Row, Text},
        Application, Element,
    },
    Alignment::{self, Center},
    Command, Length, ProgressBar, Subscription,
};
use lib::{
    fgs,
    ihash::{dhash, dhash_rotations},
};
use rfd::FileDialog;
use std::{path::PathBuf, str::FromStr, io::Write};

use self::hash_dir::{Progress, HashPair};

mod hash_dir;
mod style;

pub struct Gui {
    hashstore: fgs::HashStore,
    fingerprint_store_path: Option<PathBuf>,
    found_paths: Vec<String>,
    found_images: Vec<image::Handle>,
    image_to_process: DynamicImage,
    pasted_image: image::Handle,
    multihashes: Vec<MultiHash>,
    last_id: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    SaveImage,
    ClearImage,
    SaveHashstoreAs,
    HashDirectory,
    HashExistingImages,
    AddFile,
    PasteImage,
    Search,
    MultiHashProgressed((usize, Progress<Vec<HashPair>>)),
}

impl Application for Gui {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Gui {
                hashstore: fgs::HashStore::new(),
                fingerprint_store_path: None,
                image_to_process: DynamicImage::new_rgb8(2, 2),
                found_paths: vec![],
                found_images: vec![],
                pasted_image: image::Handle::from_memory(include_bytes!("../icon.png").to_vec()),
                multihashes: vec![],
                last_id: 0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Image Fingerprint v1.1")
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(self.multihashes.iter().map(MultiHash::subscription))
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
            Message::HashExistingImages => {
                if let Some(paths) = FileDialog::new().pick_files() {
                    self.multihashes.push(MultiHash::new(self.last_id, paths));
                    if let Some(multihash) = self.multihashes.get_mut(self.last_id) {
                        multihash.start();
                    }
                    self.last_id = self.last_id + 1;
                }
            }
            // TODO still blocks the main thread
            Message::HashDirectory => {
                if let Some(path) = FileDialog::new().pick_folder() {
                    let mut paths: Vec<PathBuf> = vec![];
                    if let Ok(dir_iter) = std::fs::read_dir(path) {
                        for entry in dir_iter {
                            let entry = entry.unwrap();
                            if entry.file_type().unwrap().is_dir() {
                                continue;
                            }
                            paths.push(entry.path());
                        }
                    }
                    self.multihashes.push(MultiHash::new(self.last_id, paths));
                    if let Some(multihash) = self.multihashes.get_mut(self.last_id) {
                        multihash.start();
                    }
                    self.last_id = self.last_id + 1;
                }
            }
            Message::MultiHashProgressed((id, progress)) => {
                if let Some(multihash) = self
                    .multihashes
                    .iter_mut()
                    .find(|multihash| multihash.id == id)
                {
                    if let Progress::Advanced(_, newHashes) = &progress {
                        for (hash, pathbuf) in newHashes {
                            self.hashstore.add_hash(hash, pathbuf.clone().to_str().unwrap())
                        }
                        self.hashstore.save();
                    }
                    
                    multihash.progress(progress)
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let Gui {
            fingerprint_store_path,
            found_images,
            found_paths,
            multihashes,
            ..
        } = self;

        let active_store_filename: Element<_> = match fingerprint_store_path {
            Some(file) => message(file.to_str().unwrap_or("No file")),
            None => message("No fingerprint file specified"),
        };

        let progress_bar = multihashes
            .iter()
            .rev()
            .find(|multihash| {
                match multihash.state {
                    State::Hashing { progress } => progress < 100.0,
                    _ => false
                }
            })
            .map(|multihash| Column::new().spacing(20).push(multihash.view())).unwrap_or(
                Column::new()
                .spacing(10)
                .padding(10)
                .push(ProgressBar::new(0.0..=100.0, 100.0))
            );

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
                Button::new(button_text("Fingerprint Existing Images"))
                    .on_press(Message::HashExistingImages)
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

        let row = Row::new()
            .push(file_controls_list)
            .push(image_viewer)
            .push(fingerprint_pane);

        let col = Column::new().push(progress_bar).push(row);

        Container::new(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .into()
    }
}

#[derive(Debug)]
struct MultiHash {
    id: usize,
    state: State,
    paths: Vec<PathBuf>,
}

#[derive(Debug)]
enum State {
    Idle,
    Hashing { progress: f32 },
    Finished,
    Errored,
}

impl MultiHash {
    pub fn new(id: usize, paths: Vec<PathBuf>) -> Self {
        MultiHash {
            id,
            state: State::Idle,
            paths,
        }
    }

    pub fn start(&mut self) {
        match self.state {
            State::Idle { .. } | State::Finished { .. } | State::Errored { .. } => {
                self.state = State::Hashing { progress: 0.0 };
            }
            _ => {}
        }
    }

    pub fn progress<T>(&mut self, new_progress: hash_dir::Progress<T>) {
        match &mut self.state {
            State::Hashing { progress } => match new_progress {
                hash_dir::Progress::Started => {
                    *progress = 0.0;
                }
                hash_dir::Progress::Advanced(percentage, vec) => {
                    *progress = percentage;
                }
                hash_dir::Progress::Finished => self.state = State::Finished,
                hash_dir::Progress::Errored => self.state = State::Errored,
            },
            _ => {}
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Hashing { .. } => {
                hash_dir::files(self.id, self.paths.clone()).map(Message::MultiHashProgressed)
            }
            _ => Subscription::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let current_progress = match &self.state {
            State::Idle { .. } => 0.0,
            State::Hashing { progress } => *progress,
            State::Finished { .. } => 100.0,
            State::Errored { .. } => 0.0,
        };

        let progress_bar = ProgressBar::new(0.0..=100.0, current_progress);

        Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Center)
            .push(progress_bar)
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
