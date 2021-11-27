use iced::{
    button, executor, image, scrollable, Align, Application, Button, Clipboard, Column, Command,
    Container, Element, HorizontalAlignment, Length, Row, Text,
};

use ::image::DynamicImage;
use clipboard_win::{formats, get_clipboard};
use lib::{fgs, ihash};
use rfd::FileDialog;
use std::path::PathBuf;

pub struct Gui {
    hashstore: fgs::HashStore,
    ruimprint_files: Option<RuimprintFile>,
    selected_file: Option<String>,
    image_to_process: DynamicImage,
    image: image::Handle,
    image_viewer: image::viewer::State,
    scroll: scrollable::State,
    select_button_state: button::State,
    paste_image_button_state: button::State,
    search_button_state: button::State,
    save_button_state: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    SaveImage,
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
                selected_file: None,
                image_to_process: DynamicImage::new_rgb8(2, 2),
                image: image::Handle::from_path("icon.png"),
                image_viewer: image::viewer::State::new(),
                scroll: scrollable::State::new(),
                select_button_state: button::State::new(),
                paste_image_button_state: button::State::new(),
                search_button_state: button::State::new(),
                save_button_state: button::State::new(),
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
                println!("Attempted to open file dialog");
            }
            Message::PasteImage => {
                dbg!("attempting to access clipboard");
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
                            let _ = self.hashstore.save();
                        }
                        Err(_) => {}
                    }
                }
            }
            Message::Search => {
                // TODO 
                let hash = ihash::dhash(&self.image_to_process);
                println!("Hash: {}", hash);
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let Gui {
            ruimprint_files,
            select_button_state,
            paste_image_button_state,
            search_button_state,
            save_button_state,
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
            None => message("No fingerprint files specified"),
        };

        let file_list = Column::new()
            .max_width(300)
            .push(button(
                select_button_state,
                "Open file",
                Message::AddFile,
                style::Button::Primary,
            ))
            .push(files_element);

        let image_viewer = Column::new()
            .max_width(500)
            .align_items(Align::Center)
            .push(button(
                paste_image_button_state,
                "Paste Image",
                Message::PasteImage,
                style::Button::Primary,
            ))
            .push(image::Viewer::new(image_viewer, self.image.clone()))
            .push(button(
                save_button_state,
                "Save Image",
                Message::SaveImage,
                style::Button::Primary,
            ));

        let fingerprint_pane = Column::new()
            .max_width(500)
            .align_items(Align::Center)
            .push(button(
                search_button_state,
                "Search",
                Message::Search,
                style::Button::Primary,
            ));

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
            .horizontal_alignment(HorizontalAlignment::Center)
            .color([0.5, 0.5, 0.5]),
    )
    .width(Length::Fill)
    .height(Length::Units(40))
    .center_y()
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

    const SURFACE: Color = Color::from_rgb(
        0xF2 as f32 / 255.0,
        0xF3 as f32 / 255.0,
        0xF5 as f32 / 255.0,
    );

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
        Destructive,
        Secondary,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            let (background, text_color) = match self {
                Button::Primary => (Some(ACTIVE), Color::WHITE),
                Button::Destructive => (None, Color::from_rgb8(0xFF, 0x47, 0x47)),
                Button::Secondary => (None, Color::BLACK),
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
                Button::Destructive => Some(Color {
                    a: 0.2,
                    ..active.text_color
                }),
                Button::Secondary => Some(Color {
                    a: 0.2,
                    ..active.text_color
                }),
            };

            button::Style {
                background: background.map(Background::Color),
                ..active
            }
        }
    }
}
