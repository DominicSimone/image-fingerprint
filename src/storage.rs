use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

pub struct LocalStorage {
    contents: HashMap<String, String>,
    file: File,
}

impl Storage for LocalStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        let value = self.contents.get(key);
        match value {
            None => None,
            Some(v) => Some(v.clone()),
        }
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.contents.insert(key.to_string(), value);
    }

    fn flush(&mut self) {
        let string_contents = serde_json::to_vec(&self.contents);
        match string_contents {
            Ok(s) => {
                self.file.write(&s).ok();
            }
            Err(_) => {}
        }
    }
}

pub fn init(filename: &str) -> Option<LocalStorage> {
    match File::open(filename) {
        Err(_) => {
            match File::create(filename) {
                Err(_) => None,
                Ok(new_file) => Some(LocalStorage {
                    file: new_file,
                    contents: HashMap::new()
                })
            }
        },
        Ok(mut f) => {
            let mut string_contents = String::new();
            f.read_to_string(&mut string_contents).ok();
            Some(LocalStorage {
                file: f,
                contents: serde_json::from_str(&string_contents).unwrap_or(HashMap::new())
            })
        }
    }
}
