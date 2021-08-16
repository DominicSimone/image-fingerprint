use iced::{
    button, executor, keyboard, pane_grid, pick_list, scrollable, text_input, Align, Application,
    Button, Checkbox, Clipboard, Color, Column, Command, Container, Element, HorizontalAlignment,
    Length, PaneGrid, Row, Scrollable, Text, TextInput,
};

use image::DynamicImage;
use rfd::FileDialog;
use std::fmt;
use std::path::PathBuf;

pub struct Gui {
    panes: pane_grid::State<Content>,
    files: Vec<PathBuf>,
    selected_file: Option<String>,
    image: Option<DynamicImage>,
    scroll: scrollable::State,
    select_button_state: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddFile,
    PasteImage,
    TaskMessage(usize, TaskMessage),
}

impl Application for Gui {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (panes, _) = pane_grid::State::new(Content::new(0));

        (
            Gui {
                panes,
                files: vec![],
                selected_file: None,
                image: None,
                scroll: scrollable::State::new(),
                select_button_state: button::State::new()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Pane grid - Iced")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::AddFile => {
                if let Some(file) = FileDialog::new()
                    .add_filter("fingerprint storage", &["fgs"])
                    .pick_file()
                {
                    self.files.push(file);
                }
                println!("Attempted to open file dialog");
            }
            Message::PasteImage => {}
            Message::TaskMessage(i, TaskMessage::Delete) => {
                self.files.remove(i);
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let Gui {
            files,
            scroll,
            select_button_state,
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

        let files: Element<_> = if files.len() > 0 {
            files
                .iter()
                .enumerate()
                .fold(Column::new(), |column, (i, file)| {
                    column.push(message(file.to_str().unwrap_or("No file")))
                })
                .into()
        } else {
            message("No fingerprint files specified")
        };

        let file_list = Column::new()
        .max_width(300)
        .push(button(
            select_button_state,
            "Open file",
            Message::AddFile,
            style::Button::Primary,
        ))
        .push(files);

        let row = Row::new().push(file_list);

        Container::new(row)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .center_y()
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
    .height(Length::Units(20))
    .center_y()
    .into()
}

struct Content {
    id: usize,
    scroll: scrollable::State,
    select_button_state: button::State,
}

impl Content {
    fn new(id: usize) -> Self {
        Content {
            id,
            scroll: scrollable::State::new(),
            select_button_state: button::State::new(),
        }
    }
    fn view(&mut self, files: Vec<PathBuf>) -> Element<Message> {
        let Content {
            scroll,
            select_button_state,
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

        let files: Element<_> = if files.len() > 0 {
            files
                .iter()
                .enumerate()
                .fold(Column::new().spacing(20), |column, (i, file)| {
                    column.push(message(file.to_str().unwrap_or("No file")))
                })
                .into()
        } else {
            message("No fingerprint files specified")
        };

        let file_list = Column::new().max_width(800).spacing(20).push(files);

        let content = Scrollable::new(scroll)
            .width(Length::Fill)
            .spacing(10)
            .align_items(Align::Center)
            .push(button(
                select_button_state,
                "Open file",
                Message::AddFile,
                style::Button::Primary,
            ));

        let row = Row::new().push(file_list).push(content);

        Container::new(row)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .center_y()
            .into()
    }
}

#[derive(Debug, Clone)]
struct Task {
    description: String,
    completed: bool,
    state: TaskState,
}

#[derive(Debug, Clone)]
pub enum TaskState {
    Idle { delete_button: button::State },
}

impl Default for TaskState {
    fn default() -> Self {
        TaskState::Idle {
            delete_button: button::State::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TaskMessage {
    Delete,
}

impl Task {
    fn new(description: String) -> Self {
        Task {
            description,
            completed: false,
            state: TaskState::Idle {
                delete_button: button::State::new(),
            },
        }
    }

    fn update(&mut self, message: TaskMessage) {
        match message {
            TaskMessage::Delete => {}
        }
    }

    fn view(&mut self) -> Element<TaskMessage> {
        match &mut self.state {
            TaskState::Idle { delete_button } => Row::new()
                .spacing(20)
                .align_items(Align::Center)
                .push(
                    Button::new(
                        delete_button,
                        Row::new().spacing(10).push(Text::new("Delete")),
                    )
                    .on_press(TaskMessage::Delete)
                    .padding(10)
                    .style(style::Button::Destructive),
                )
                .into(),
        }
    }
}

mod style {
    use iced::{button, container, Background, Color, Vector};

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

    pub struct TitleBar {
        pub is_focused: bool,
    }

    impl container::StyleSheet for TitleBar {
        fn style(&self) -> container::Style {
            let pane = Pane.style();

            container::Style {
                text_color: Some(Color::WHITE),
                background: Some(pane.border_color.into()),
                ..Default::default()
            }
        }
    }

    pub struct Pane;

    impl container::StyleSheet for Pane {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(SURFACE)),
                border_width: 1.0,
                border_color: Color::BLACK,
                ..Default::default()
            }
        }
    }

    pub enum Button {
        Primary,
        Destructive,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            let (background, text_color) = match self {
                Button::Primary => (Some(ACTIVE), Color::WHITE),
                Button::Destructive => (None, Color::from_rgb8(0xFF, 0x47, 0x47)),
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
            };

            button::Style {
                background: background.map(Background::Color),
                ..active
            }
        }
    }
}
