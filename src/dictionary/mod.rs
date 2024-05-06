use scanf::sscanf;
use sqlx::Error as ErrorSql;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{LineWriter, Result as ResultStd, Write};

/// Stores Serializer in Self.
///
pub trait SerializerSaver {
    async fn save(&self, s: &Module) -> Result<(), ErrorSql>;
}

/// Reads stored Serializer in Self.
///
pub trait SerializerReader {
    async fn read(&self) -> Result<Module, ErrorSql>;
}

/// Serializer serialize the log in to the binary format.
///
#[derive(Debug, Clone)]
pub struct Module {
    words_to_numbers: HashMap<String, u32>,
    nums_to_words: HashMap<u32, String>,
    last_available_number: u32,
}

impl Module {
    /// Creates new Book that holds no values yet.
    ///
    #[inline]
    pub fn new() -> Self {
        Self {
            words_to_numbers: HashMap::new(),
            nums_to_words: HashMap::new(),
            last_available_number: 0,
        }
    }

    /// Sets a new words to numbers map from given dataset.
    ///
    #[inline(always)]
    pub fn set_map_from(&mut self, m: HashMap<String, u32>) {
        self.last_available_number = 0;
        self.words_to_numbers = m;
        self.words_to_numbers.iter().for_each(|(_, n)| {
            if *n > self.last_available_number {
                self.last_available_number = *n
            }
        });
        self.nums_from_words();
    }

    /// Serializes the value in to the numeric representation of data.
    ///
    #[inline]
    pub fn serialize(&mut self, log: &str) -> Vec<u32> {
        log.split_whitespace()
            .into_iter()
            .map(|token| {
                let Some(num) = self.words_to_numbers.get(token) else {
                    self.last_available_number += 1;
                    self.words_to_numbers
                        .insert(token.to_string(), self.last_available_number);
                    self.nums_to_words
                        .insert(self.last_available_number, token.to_string());
                    return self.last_available_number;
                };
                *num
            })
            .collect()
    }

    /// Deserializes numeric representation of data to String.
    ///
    #[inline(always)]
    pub fn deserialize(&self, buffer: &[u32]) -> String {
        let mut msg = "".to_string();
        for candidate in buffer.into_iter() {
            match self.nums_to_words.get(candidate) {
                Some(w) => msg.push_str(w),
                None => msg.push_str("[?]"),
            }
            msg.push(' ');
        }

        msg.trim().to_string()
    }

    /// Allows to iterate over inner words to num collection.
    ///
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&String, &u32)> {
        self.words_to_numbers.iter()
    }

    /// Saves schema to a file.
    ///
    #[inline]
    pub fn save_schema_to_file(&self, path: &str) -> ResultStd<()> {
        let file = File::create(path)?;
        let mut file = LineWriter::new(file);
        for (w, n) in self.words_to_numbers.iter() {
            file.write_all(format!("{} : {}\n", *w, *n).as_bytes())?;
        }
        file.flush()?;

        Ok(())
    }

    /// Reads schema from a file.
    ///
    #[inline]
    pub fn read_schema_from_file(path: &str) -> ResultStd<Self> {
        let mut serializer = Self::new();
        let mut file = File::open(path)?;
        let mut reader = BufReader::new(file);

        for line in reader.lines() {
            let mut n = 0;
            let mut w = String::new();
            sscanf!(&line?, "{} : {}", w, n)?;
            if serializer.last_available_number < n {
                serializer.last_available_number = n;
            }
            serializer.words_to_numbers.insert(w.to_string(), n);
        }
        serializer.nums_from_words();

        Ok(serializer)
    }

    #[inline(always)]
    fn nums_from_words(&mut self) {
        for (k, v) in self.words_to_numbers.iter() {
            self.nums_to_words.insert(*v, k.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    const text: &str = "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem. Ut enim ad minima veniam, quis nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut aliquid ex ea commodi consequatur? Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur, vel illum qui dolorem eum fugiat quo voluptas nulla pariatur";
    const bench_loops: usize = 10000;
    const path: &str = "./test.schema";

    #[test]
    fn test_serialize_once() {
        let mut serialize = Module::new();
        let buffer = serialize.serialize(text);
        assert!(buffer.len() > 0);
    }

    #[test]
    fn test_serialize_bench() {
        let mut book = Module::new();
        let start = Instant::now();
        for _ in 0..bench_loops {
            book.serialize(text);
        }
        let duration = start.elapsed();

        println!(
            "Time elapsed in test_serialize_bench is: {:?}",
            duration / bench_loops as u32
        );
    }

    #[test]
    fn test_deserialize_once() {
        let mut module = Module::new();
        let buffer = module.serialize(text);
        let log = module.deserialize(&buffer);
        assert_eq!(text, log);
    }

    #[test]
    fn test_serialize_save_read() {
        let mut module = Module::new();
        let buffer = module.serialize(text);
        if let Err(_) = module.save_schema_to_file(path) {
            assert!(false);
        }
        let Ok(new_serializer) = Module::read_schema_from_file(path) else {
            assert!(false);
            return;
        };

        assert_eq!(
            module.last_available_number,
            new_serializer.last_available_number
        );

        for (k, v) in module.words_to_numbers.iter() {
            match new_serializer.words_to_numbers.get(k) {
                Some(n_v) => assert_eq!(*v, *n_v),
                None => assert!(false),
            };
        }
    }

    #[test]
    fn test_deserialize_bench() {
        let mut module = Module::new();
        let buffer = module.serialize(text);
        let start = Instant::now();
        for _ in 0..bench_loops {
            let _ = module.deserialize(&buffer);
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in test_deserialize_once is: {:?}",
            duration / bench_loops as u32
        );
    }
}
