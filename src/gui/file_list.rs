use iced::{scrollable, Container, Scrollable, Column, Length};

pub struct FileList {
    files: Vec<String>,
    scroll: scrollable::State,
}

#[derive(Debug, Clone)]
pub enum FileListMessage {
    AddFile(String),
    RemoveFile(u32),
}

impl Default for FileList {
    fn default() -> Self {
        FileList {
            files: vec![],
            scroll: scrollable::State::new()
        }
    }
}

impl FileList {
    pub fn update(&mut self, FileListMessage: FileListMessage) {
        match FileListMessage {
            FileListMessage::AddFile(_file) => {}
            FileListMessage::RemoveFile(_position) => {}
        }
    }

    pub fn view(&mut self) -> Scrollable<FileListMessage> {
        let content = Column::new().max_width(800).spacing(20);

        Scrollable::new(&mut self.scroll)
            .padding(40)
            .push(Container::new(content).width(Length::Fill).center_x())
    }
}
