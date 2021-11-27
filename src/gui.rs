use iced::{
    button, executor, image, Align, Application, Button, Clipboard, Column, Command, Container,
    Element, HorizontalAlignment, Length, Row, Text, VerticalAlignment,
};

use ::image::DynamicImage;
use clipboard_win::{formats, get_clipboard};
use lib::{fgs, ihash};
use rfd::FileDialog;
use std::{path::PathBuf, str::FromStr};

pub struct Gui {
    hashstore: fgs::HashStore,
    ruimprint_files: Option<RuimprintFile>,
    found_paths: Vec<String>,
    found_images: Vec<(image::Handle, image::viewer::State)>,
    image_to_process: DynamicImage,
    image: image::Handle,
    image_viewer: image::viewer::State,
    select_button_state: button::State,
    paste_image_button_state: button::State,
    search_button_state: button::State,
    save_image_button_state: button::State,
    hash_directory_button_state: button::State,
    save_as_hashstore_button_state: button::State,
    save_hashstore_button_state: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    SaveImage,
    SaveHashstoreAs,
    SaveHashstore,
    HashDirectory,
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
                hashstore: fgs::HashStore::new(),
                ruimprint_files: None,
                image_to_process: DynamicImage::new_rgb8(2, 2),
                found_paths: vec![],
                found_images: vec![],
                image: image::Handle::from_memory(include_bytes!("../icon.png").to_vec()),
                image_viewer: image::viewer::State::new(),
                select_button_state: button::State::new(),
                paste_image_button_state: button::State::new(),
                search_button_state: button::State::new(),
                save_image_button_state: button::State::new(),
                hash_directory_button_state: button::State::new(),
                save_as_hashstore_button_state: button::State::new(),
                save_hashstore_button_state: button::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Ruimprint")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
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
            Message::SaveImage => {
                if let Some(path) = FileDialog::new().add_filter("", &["png"]).save_file() {
                    let spath = path.to_str().unwrap();
                    match self.image_to_process.save(spath) {
                        Ok(_) => {
                            let hash = ihash::dhash(&self.image_to_process);
                            self.hashstore.add_hash(&hash, &spath);
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
                    let state = image::viewer::State::new();
                    self.found_images.push((im, state));
                }
            }
            Message::SaveHashstore => {
                let _ = self.hashstore.save();
            }
            Message::SaveHashstoreAs => {
                if let Some(path) = FileDialog::new().add_filter("", &["json"]).save_file() {
                    let spath = path.to_str().unwrap();
                    let _ = self.hashstore.to_file(spath);
                    self.ruimprint_files =
                        Some(RuimprintFile::new(PathBuf::from_str(spath).unwrap()));
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
                                let hash = ihash::dhash(&image);
                                self.hashstore.add_hash(&hash, &spath.to_str().unwrap());
                            }
                        }
                    }
                }
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let Gui {
            ruimprint_files,
            found_images,
            found_paths,
            select_button_state,
            paste_image_button_state,
            search_button_state,
            save_image_button_state,
            hash_directory_button_state,
            save_as_hashstore_button_state,
            save_hashstore_button_state,
            image_viewer,
            ..
        } = self;

        let button = |state, label, message, style| {
            Button::new(
                state,
                Text::new(label)
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .size(16),
            )
            .width(Length::Fill)
            .padding(8)
            .on_press(message)
            .style(style)
        };

        let files_element: Element<_> = match ruimprint_files {
            Some(file) => message(file.file.to_str().unwrap_or("No file")),
            None => message("No fingerprint file specified"),
        };

        let image_results: Element<_> = if !found_images.is_empty() {
            found_images
                .iter_mut()
                .enumerate()
                .fold(Column::new(), |col, (i, (image, state))| {
                    col.push(
                        Row::new()
                            .push(
                                image::Viewer::new(state, image.clone()).height(Length::Units(150)),
                            )
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
            .push(button(
                hash_directory_button_state,
                "Fingerprint a Directory",
                Message::HashDirectory,
                style::Button::Primary,
            ))
            .push(button(
                save_as_hashstore_button_state,
                "Save Fingerprints File As",
                Message::SaveHashstoreAs,
                style::Button::Primary,
            ))
            .push(button(
                save_hashstore_button_state,
                "Save Fingerprints File",
                Message::SaveHashstore,
                style::Button::Primary,
            ))
            .push(button(
                select_button_state,
                "Open Fingerprints File",
                Message::AddFile,
                style::Button::Primary,
            ))
            .push(files_element);

        let image_viewer = Column::new()
            .max_width(500)
            .padding(10)
            .spacing(5)
            .align_items(Align::Center)
            .push(button(
                paste_image_button_state,
                "Paste Image",
                Message::PasteImage,
                style::Button::Primary,
            ))
            .push(image::Viewer::new(image_viewer, self.image.clone()))
            .push(button(
                save_image_button_state,
                "Save Image",
                Message::SaveImage,
                style::Button::Primary,
            ));

        let fingerprint_pane = Column::new()
            .max_width(500)
            .padding(10)
            .spacing(5)
            .align_items(Align::Center)
            .push(button(
                search_button_state,
                "Search",
                Message::Search,
                style::Button::Primary,
            ))
            .push(image_results);

        let row = Row::new()
            .push(file_list)
            .push(image_viewer)
            .push(fingerprint_pane);

        Container::new(row)
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
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Center)
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
    Idle {
        delete_button: button::State,
        mark_button: button::State,
    },
}

impl Default for RuimprintFileState {
    fn default() -> Self {
        RuimprintFileState::Idle {
            delete_button: button::State::new(),
            mark_button: button::State::new(),
        }
    }
}

impl RuimprintFile {
    fn new(file: PathBuf) -> Self {
        RuimprintFile {
            marked: false,
            file,
            state: RuimprintFileState::Idle {
                delete_button: button::State::new(),
                mark_button: button::State::new(),
            },
        }
    }
}

mod style {
    use iced::{button, Background, Color, Vector};

    const ACTIVE: Color = Color::from_rgb(
        0x72 as f32 / 255.0,
        0x89 as f32 / 255.0,
        0xDA as f32 / 255.0,
    );

    const HOVERED: Color = Color::from_rgb(
        0x67 as f32 / 255.0,
        0x7B as f32 / 255.0,
        0xC4 as f32 / 255.0,
    );

    pub enum Button {
        Primary,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            let (background, text_color) = match self {
                Button::Primary => (Some(ACTIVE), Color::WHITE),
            };

            button::Style {
                text_color,
                background: background.map(Background::Color),
                border_radius: 5.0,
                shadow_offset: Vector::new(0.0, 0.0),
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            let active = self.active();

            let background = match self {
                Button::Primary => Some(HOVERED),
            };

            button::Style {
                background: background.map(Background::Color),
                ..active
            }
        }
    }
}
