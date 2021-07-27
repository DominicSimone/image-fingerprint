use iced::{
    executor, scrollable, Application, Button, Clipboard, Color, Column, Command, Element, Text,
};
pub struct Gui;
mod file_list;

impl Application for Gui {
    type Executor = executor::Default;
    type Message = ();
    type Flags = ();

    fn new(_flags: ()) -> (Gui, Command<Self::Message>) {
        (Gui, Command::none())
    }

    fn title(&self) -> String {
        String::from("Image Fingerprint")
    }

    fn update(
        &mut self,
        _message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Text::new("Hello, world!").into()
    }

    fn background_color(&self) -> Color {
        Color {
            r: 0.117,
            g: 0.117,
            b: 0.117,
            a: 0.,
        }
    }
}
