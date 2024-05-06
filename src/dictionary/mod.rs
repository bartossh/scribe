use scanf::sscanf;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{LineWriter, Result, Write};

/// Serializer serialize the log in to the binary format.
///
#[derive(Debug, Clone)]
pub struct Serializer {
    words_to_numbers: HashMap<String, u32>,
    last_available_number: u32,
}

impl Serializer {
    /// Creates new Book that holds no values yet.
    ///
    #[inline]
    pub fn new() -> Self {
        Self {
            words_to_numbers: HashMap::new(),
            last_available_number: 0,
        }
    }

    /// Serializes the value in to the numeric representation of data.
    ///
    #[inline(always)]
    pub fn serialize(&mut self, log: &str) -> Vec<u32> {
        log.split_whitespace()
            .into_iter()
            .map(|token| {
                let Some(num) = self.words_to_numbers.get(token) else {
                    self.last_available_number += 1;
                    self.words_to_numbers
                        .insert(token.to_string(), self.last_available_number);
                    return self.last_available_number;
                };
                *num
            })
            .collect()
    }

    /// Saves schema to a file.
    ///
    #[inline]
    pub fn save_schema_to_file(&self, path: &str) -> Result<()> {
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
    pub fn read_schema_from_file(path: &str) -> Result<Self> {
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

        Ok(serializer)
    }

    #[inline(always)]
    pub fn save_log(&self, log: &[u32]) -> Result<()> {
        Ok(())
    }
}

/// Deserializer deserializes logs from binary format.
///
#[derive(Debug, Clone)]
pub struct Deserializer {
    nums_to_words: HashMap<u32, String>,
}

impl Deserializer {
    #[inline]
    pub fn from(s: &Serializer) -> Self {
        let mut nums_to_words = HashMap::new();
        for (k, v) in s.words_to_numbers.iter() {
            nums_to_words.insert(*v, k.clone());
        }
        Self { nums_to_words }
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
        let mut serialize = Serializer::new();
        let buffer = serialize.serialize(text);
        println!("BUFFER:\n {:?} \n", buffer);
    }

    #[test]
    fn test_serialize_bench() {
        let mut book = Serializer::new();
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
        let mut serialize = Serializer::new();
        let buffer = serialize.serialize(text);
        let deserialize = Deserializer::from(&serialize);
        let log = deserialize.deserialize(&buffer);
        assert_eq!(text, log);
    }

    #[test]
    fn test_serialize_save_read() {
        let mut serialize = Serializer::new();
        let buffer = serialize.serialize(text);
        if let Err(_) = serialize.save_schema_to_file(path) {
            assert!(false);
        }
        let Ok(new_serializer) = Serializer::read_schema_from_file(path) else {
            assert!(false);
            return;
        };

        assert_eq!(
            serialize.last_available_number,
            new_serializer.last_available_number
        );

        for (k, v) in serialize.words_to_numbers.iter() {
            match new_serializer.words_to_numbers.get(k) {
                Some(n_v) => assert_eq!(*v, *n_v),
                None => assert!(false),
            };
        }
    }

    #[test]
    fn test_deserialize_bench() {
        let mut serialize = Serializer::new();
        let buffer = serialize.serialize(text);
        let deserialize = Deserializer::from(&serialize);
        let mut book = Serializer::new();
        let start = Instant::now();
        for _ in 0..bench_loops {
            let _ = deserialize.deserialize(&buffer);
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in test_deserialize_once is: {:?}",
            duration / bench_loops as u32
        );
    }
}
