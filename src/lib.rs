pub mod ihash {

    use image::{imageops::FilterType, DynamicImage};

    pub fn dhash_with(image: &DynamicImage, filter: FilterType) -> String {
        let mut hash = String::from("");
        let gray = image.resize_exact(9, 8, filter).to_luma8();
        for (_, mut row) in gray.enumerate_rows() {
            if let Some((_, _, mut prev)) = row.next() {
                for (_, _, pixel) in row {
                    hash.push(if pixel.0 > prev.0 { '1' } else { '0' });
                    prev = pixel;
                }
            }
        }
        hash
    }

    pub fn dhash(image: &DynamicImage) -> String {
        dhash_with(image, FilterType::Triangle)
    }

    pub fn dist(first: &str, second: &str) -> usize {
        first
            .chars()
            .zip(second.chars())
            .fold(0, |acc: usize, (c1, c2)| match c1 == c2 {
                true => acc,
                false => acc + 1,
            })
    }
}

pub mod fgs {

    use std::cmp::Ordering;
    use std::collections::BinaryHeap;
    use std::fs::File;
    use std::io::Error;

    #[derive(Clone, Eq, PartialEq, Debug)]
    struct Comparison {
        similarity: usize,
        path: String,
    }

    impl PartialOrd for Comparison {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.similarity.partial_cmp(&other.similarity)
        }
    }

    impl Ord for Comparison {
        fn cmp(&self, other: &Self) -> Ordering {
            self.similarity.cmp(&other.similarity)
        }
    }

    #[derive(Default)]
    pub struct HashStore {
        hashes: Vec<(String, String)>,
        path: Option<String>,
    }

    impl HashStore {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn from_file(path: &str) -> Result<Self, Error> {
            let file = File::open(path)?;
            let data: Vec<(String, String)> = serde_json::from_reader(file)?;
            Ok(Self {
                hashes: data,
                path: Some(path.to_string()),
            })
        }

        pub fn to_file(&self, path: &str) -> Result<&Self, Error> {
            let file = File::create(path)?;
            serde_json::to_writer(file, &self.hashes)?;
            Ok(self)
        }

        pub fn save(&self) -> Result<&Self, Error> {
            if let Some(p) = &self.path {
                return self.to_file(&p);
            }
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No path set",
            ))
        }

        pub fn add_hash(&mut self, hash: &str, path: &str) {
            self.hashes.push((hash.to_string(), path.to_string()))
        }

        pub fn find(&self, hash: &str) -> Option<&str> {
            for (h, p) in self.hashes.iter() {
                if super::ihash::dist(h, hash) == 0 {
                    return Some(p);
                }
            }
            None
        }

        pub fn find_many(&self, hash: &str, size: usize) -> Vec<String> {
            let mut bheap: BinaryHeap<Comparison> = BinaryHeap::new();
            for (h, p) in self.hashes.iter() {
                bheap.push(Comparison {
                    similarity: 100 - super::ihash::dist(h, hash),
                    path: String::from(p),
                })
            }

            let mut result: Vec<String> = vec![];
            for _ in 0..size {
                if let Some(comp) = bheap.pop() {
                    result.push(comp.path);
                }
            }
            result
        }
    }
}

#[test]
fn distance_test() {
    assert_eq!(
        35,
        ihash::dist(
            "0000001100000011000000110010001100000011000110110010101100101111",
            "0011000000110000001100000010000011110001101100000011000011110000"
        )
    );
    assert_eq!(1, ihash::dist("1001", "1000"));
    assert_eq!(4, ihash::dist("0111", "1000"));
}

#[test]
fn dhash_test() {
    use image::io::Reader;

    let image = Reader::open("./test/pokemon/bulbasaur.png")
        .unwrap()
        .decode()
        .unwrap();
    ihash::dhash(&image);
}

#[test]
fn hashstore_read_write() {
    use fgs::HashStore;
    use std::fs::remove_file;

    let fname = "./test/data.json";
    let mut store = HashStore::default();
    store.add_hash("1001", "./test/pokemon/nonexistant.png");
    let _ = store.to_file(fname);

    let store_fs = HashStore::from_file(fname).unwrap_or(HashStore::default());
    assert_eq!(
        store_fs.find("1001").unwrap(),
        "./test/pokemon/nonexistant.png"
    );
    let _ = remove_file(fname);
}
