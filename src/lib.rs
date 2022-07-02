pub mod ihash {

    use image::{imageops::FilterType, DynamicImage, GrayImage};
    use serde::{Serialize, Deserialize};

    pub enum HashMethod {
        DHashRotations
    }

    #[derive(Default, Clone, Copy, Serialize, Deserialize)]
    pub struct IHash {
        value: u64,
    }

    impl IHash {
        pub fn new(hash: u64) -> Self {
            IHash { value: hash }
        }

        pub fn from_str(string: &str) -> Self {
            IHash {
                value: u64::from_str_radix(string, 10).unwrap(),
            }
        }

        pub fn to_str(self) -> String {
            self.value.to_string()
        }

        // Also checks mirrored hash
        pub fn comp(hash1: &Self, hash2: &Self) -> u32 {
            let xor: u64 = hash1.value ^ hash2.value;
            let xor_inv: u64 = hash1.value ^ u64::MAX ^ hash2.value;
            u32::min(xor.count_ones(), xor_inv.count_ones())
        }

        pub fn dist(self, hash2: &Self) -> u32 {
            IHash::comp(&self, hash2)
        }
    }

    pub fn dhash_small_luma(small: &GrayImage) -> IHash {
        let mut hash: u64 = 0;
        for (_, mut row) in small.enumerate_rows() {
            if let Some((_, _, mut prev)) = row.next() {
                for (_, _, pixel) in row {
                    if pixel.0 > prev.0 {
                        hash <<= 1;
                        hash += 1;
                    } else {
                        hash <<= 1;
                    }
                    prev = pixel;
                }
            }
        }
        IHash::new(hash)
    }

    pub fn dhash_rotations(image: &DynamicImage, filter: FilterType) -> Vec<IHash> {
        let mut vec: Vec<IHash> = vec![];
        let gray = DynamicImage::ImageLuma8(image.to_luma8());
        vec.push(dhash_small_luma(&gray.resize_exact(9, 8, filter).into_luma8()));
        for _ in 0..=2 {
            gray.rotate90();
            vec.push(dhash_small_luma(&gray.resize_exact(9, 8, filter).into_luma8()));
        }
        vec
    }

    pub fn dhash_once(image: &DynamicImage, filter: FilterType) -> IHash {
        // Not sure if resizing first or grayscaling first is faster
        // let gray = image.resize_exact(9, 8, filter).to_luma8();
        let gray = DynamicImage::ImageLuma8(image.to_luma8()).resize_exact(9, 8, filter).into_luma8();
        dhash_small_luma(&gray)
    }

    pub fn dhash(image: &DynamicImage) -> IHash {
        dhash_once(image, FilterType::Triangle)
    }
}

pub mod fgs {

    use std::cmp::Ordering;
    use std::collections::BinaryHeap;
    use std::fs::File;
    use std::io::Error;

    use crate::ihash::IHash;

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
        hashes: Vec<(IHash, String)>,
        path: Option<String>,
    }

    impl HashStore {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn from_file(path: &str) -> Result<Self, Error> {
            let file = File::open(path)?;
            let data: Vec<(IHash, String)> = serde_json::from_reader(file)?;
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

        pub fn add_hash(&mut self, hash: &IHash, path: &str) {
            self.hashes.push((hash.clone(), path.to_string()))
        }

        pub fn find(&self, hash: &IHash) -> Option<&str> {
            for (h, p) in self.hashes.iter() {
                if hash.dist(h) == 0 {
                    return Some(p);
                }
            }
            None
        }

        pub fn find_many(&self, hash_list: &Vec<IHash>, size: usize) -> Vec<String> {
            let mut bheap: BinaryHeap<Comparison> = BinaryHeap::new();
            for (h, p) in self.hashes.iter() {
                let mut smallest = 100 - hash_list[0].dist(h);
                for hash in hash_list {
                    let d = 100 - hash.dist(h);
                    if d < smallest {
                        smallest = d;
                    }
                }
                bheap.push(Comparison {
                    similarity: smallest as usize,
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

        pub fn find_heap(&self, hash: &IHash, size: usize) -> Vec<String> {
            let mut bheap: BinaryHeap<Comparison> = BinaryHeap::new();
            for (h, p) in self.hashes.iter() {
                bheap.push(Comparison {
                    similarity: 100 - hash.dist(h) as usize,
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
    use ihash::IHash;
    assert_eq!(
        29,
        IHash::comp(
            &IHash::from_str("217020655954766639"),
            &IHash::from_str("3472328230754595056")
        )
    );
    assert_eq!(1, IHash::comp(&IHash::from_str("9"), &IHash::from_str("8")));
    assert_eq!(4, IHash::comp(&IHash::from_str("7"), &IHash::from_str("8")));
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
    use ihash::IHash;

    let fname = "./test/data.json";
    let mut store = HashStore::default();
    store.add_hash(&IHash::from_str("9"), "./test/pokemon/nonexistant.png");
    let _ = store.to_file(fname);

    let store_fs = HashStore::from_file(fname).unwrap_or(HashStore::default());
    assert_eq!(
        store_fs.find(&IHash::from_str("9")).unwrap(),
        "./test/pokemon/nonexistant.png"
    );
    let _ = remove_file(fname);
}
