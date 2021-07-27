use iced::{Element};

pub struct FileList {
    files: Vec<String>
}

#[derive(Debug, Clone)]
pub enum Message {
    AddFile(String),
    RemoveFile(usize)
}

impl Default for FileList {
    fn default() -> Self {
        todo!()
    }
}

impl FileList {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::AddFile(_file) => {},
            Message::RemoveFile(_position) => {}
        }
    }

    pub fn view<'a>(&'a mut self) -> Element<'a, Message> {
        todo!()
    }
}