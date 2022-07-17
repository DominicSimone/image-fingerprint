use iced_native::subscription;
use lib::ihash::{dhash, IHash};
use std::{
    hash::Hash,
    path::PathBuf,
    sync::mpsc::{Receiver, TryRecvError},
};

pub type HashPair = (IHash, PathBuf);

// Just a little utility function
pub fn files<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
    paths: Vec<PathBuf>,
) -> iced::Subscription<(I, Progress<Vec<HashPair>>)> {
    subscription::unfold(id, State::Ready(paths), move |state| {
        multihash(id, state)
    })
}

pub fn hash_files(paths: Vec<PathBuf>) -> Response<HashPair> {
    let (tx, rx) = std::sync::mpsc::channel();
    let num_files = paths.len();
    std::thread::spawn(move || {
        for path in paths {
            if let Ok(image) = ::image::open(&path) {
                let hash = dhash(&image);
                tx.send((hash, path));
            }
        }
    });
    Response {
        num_files,
        receiver: rx,
        contents: vec![],
        complete: false,
    }
}

#[derive(Debug, Hash, Clone)]
pub struct MultiHash<I> {
    id: I,
    paths: Vec<PathBuf>,
}

async fn multihash<I: Copy>(id: I, state: State) -> (Option<(I, Progress<Vec<HashPair>>)>, State) {
    match state {
        State::Ready(paths) => {
            let response = hash_files(paths);

            let total = response.content_length();
            (
                Some((id, Progress::Started)),
                State::Hashing {
                    response,
                    total,
                    num_hashed: 0,
                },
            )
        }
        State::Hashing {
            mut response,
            total,
            num_hashed,
        } => {
            let chunk = response.chunk();
            let num_hashed = num_hashed + chunk.len();

            let percentage = (num_hashed as f32 / total as f32) * 100.0;

            if !response.complete {
                (
                    Some((id, Progress::Advanced(percentage, chunk))),
                    State::Hashing {
                        response,
                        total,
                        num_hashed,
                    },
                )
            } else {
                (Some((id, Progress::Finished)), State::Finished(response))
            }
        }
        State::Finished(_) => {
            // We do not let the stream die, as it would start a
            // new download repeatedly if the user is not careful
            // in case of errors.
            iced::futures::future::pending().await
        }
    }
}

#[derive(Debug, Clone)]
pub enum Progress<T> {
    Started,
    Advanced(f32, T),
    Finished,
    Errored,
}

#[derive(Debug)]
pub struct Response<T: Clone> {
    receiver: Receiver<T>,
    contents: Vec<T>,
    num_files: usize,
    complete: bool,
}

impl<T: Clone> Response<T> {
    pub fn chunk(&mut self) -> Vec<T> {
        let mut vec: Vec<T> = vec![];
        loop {
            match self.receiver.try_recv() {
                Ok(tuple) => {
                    self.contents.push(tuple.clone());
                    vec.push(tuple);
                }
                Err(TryRecvError::Disconnected) => {
                    self.complete = true;
                    break;
                }
                Err(TryRecvError::Empty) => break,
            }
        }
        vec
    }

    pub fn content_length(&self) -> usize {
        self.num_files
    }
}

pub enum State {
    Ready(Vec<PathBuf>),
    Hashing {
        response: Response<(IHash, PathBuf)>,
        total: usize,
        num_hashed: usize,
    },
    Finished(Response<(IHash, PathBuf)>),
}
