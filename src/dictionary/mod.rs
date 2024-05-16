use scanf::sscanf;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{LineWriter, Result as ResultStd, Write};

/// Offers finding mechanism for matching words with numeric representation.
///
pub trait Filter: Send + Sync {
    fn push(&mut self, s: &str, num: u32);
    fn find_prefix(&self, s: &str) -> HashSet<u32>;
    fn find_prefix_case_insensitive(&self, s: &str) -> HashSet<u32>;
}

/// Stores Serializer in Self.
///
pub trait SerializerSaver {
    async fn save(&self, s: &Module) -> ResultStd<()>;
}

/// Reads stored Serializer in Self.
///
pub trait SerializerReader {
    async fn read(&self) -> ResultStd<Module>;
}

/// Serializer serialize the log in to the binary format.
///
pub struct Module {
    words_to_numbers: HashMap<String, u32>,
    nums_to_words: HashMap<u32, String>,
    last_available_number: u32,
    filter: Box<dyn Filter>,
}

impl Module {
    /// Creates new Book that holds no values yet.
    ///
    #[inline]
    pub fn new(f: impl Filter + 'static) -> Self {
        Self {
            words_to_numbers: HashMap::new(),
            nums_to_words: HashMap::new(),
            last_available_number: 0,
            filter: Box::new(f),
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
                    self.filter.push(token, self.last_available_number);
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

    /// Filters buffers based on matching prefix.
    ///
    #[inline(always)]
    pub fn filter_prefixed(&self, word: &str, buffers: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
        let mut filtered = Vec::new();
        let set = self.filter.find_prefix(word);
        'outer: for buf in buffers.iter() {
            for member in buf.iter() {
                if set.contains(member) {
                    filtered.push(buf.to_vec());
                    continue 'outer;
                }
            }
        }

        filtered
    }

    /// Filters buffers based on matching full word from slice of words.
    ///
    #[inline(always)]
    pub fn filter_word(&self, words: &[String], buffers: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
        let mut filtered = Vec::new();
        let mut set: HashSet<u32> = HashSet::new();
        for w in words.iter() {
            if let Some(num) = self.words_to_numbers.get(w) {
                set.insert(*num);
            }
        }
        'outer: for buf in buffers.iter() {
            for member in buf.iter() {
                if set.contains(member) {
                    filtered.push(buf.to_vec());
                    continue 'outer;
                }
            }
        }

        filtered
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
    pub fn read_schema_from_file(path: &str, f: impl Filter + 'static) -> ResultStd<Self> {
        let mut serializer = Self::new(f);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

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
    use std::error::Error;

    use super::*;
    use std::time::Instant;

    const TEXT: &str = "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem. Ut enim ad minima veniam, quis nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut aliquid ex ea commodi consequatur? Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur, vel illum qui dolorem eum fugiat quo voluptas nulla pariatur";
    const BENCH_LOOP: usize = 10000;

    struct MyFilterMock {}
    impl MyFilterMock {
        fn new() -> Self {
            Self {}
        }
    }

    impl Filter for MyFilterMock {
        fn push(&mut self, _: &str, _: u32) {}
        fn find_prefix(&self, _: &str) -> HashSet<u32> {
            HashSet::new()
        }
        fn find_prefix_case_insensitive(&self, _: &str) -> HashSet<u32> {
            HashSet::new()
        }
    }

    #[test]
    fn test_serialize_once() {
        let mock = MyFilterMock::new();
        let mut serialize = Module::new(mock);

        let buffer = serialize.serialize(TEXT);
        
        assert!(buffer.len() > 0);
    }

    #[test]
    fn test_serialize_bench() {
        let mock = MyFilterMock::new();
        let mut serialize = Module::new(mock);
        let start = Instant::now();
        for _ in 0..BENCH_LOOP {
            serialize.serialize(TEXT);
        }
        let duration = start.elapsed();

        println!(
            "Time elapsed in test_serialize_bench is: {:?}",
            duration / BENCH_LOOP as u32
        );
    }

    #[test]
    fn test_deserialize_once() {
        let mock = MyFilterMock::new();

        let mut serialize = Module::new(mock);
        let buffer = serialize.serialize(TEXT);
        let log = serialize.deserialize(&buffer);
        assert_eq!(TEXT, log);
    }

    #[test]
    fn test_serialize_save_read() -> Result<(), Box<dyn Error>> {
        let path = "./save_read.schema";
        let expected = Module::new(MyFilterMock::new());
        expected.save_schema_to_file(path)?;

        let actual = Module::read_schema_from_file(path, MyFilterMock::new())?;

        let result = expected.words_to_numbers.iter()
            .all(|(k, v)| *v == *actual.words_to_numbers.get(k).unwrap());
        assert!(result);
        assert_eq!(
            expected.last_available_number,
            actual.last_available_number
        );
        Ok(())
    }

    #[test]
    fn test_find_prefixes() {
        let path = "./test.schema";
        struct MyFindPrefixTestFilterMock {
            h: HashSet<u32>,
        }
        impl MyFindPrefixTestFilterMock {
            fn new(h: HashSet<u32>) -> Self {
                Self { h }
            }
        }

        impl Filter for MyFindPrefixTestFilterMock {
            fn push(&mut self, _: &str, num: u32) {}
            fn find_prefix(&self, _: &str) -> HashSet<u32> {
                self.h.clone()
            }
            fn find_prefix_case_insensitive(&self, _: &str) -> HashSet<u32> {
                HashSet::new()
            }
        }

        let mut hs = HashSet::new();
        hs.insert(1);

        let mock = MyFindPrefixTestFilterMock::new(hs);

        let mut serialize = Module::new(mock);
        let buffer = serialize.serialize(TEXT);
        if let Err(_) = serialize.save_schema_to_file(path) {
            assert!(false);
        }

        let mut buffers = Vec::new();
        buffers.push(buffer.clone());
        buffers.push(vec![11111]);

        let result = serialize.filter_prefixed("Se", buffers);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].clone(), buffer);
    }

    #[test]
    fn test_deserialize_bench() {
        let mock = MyFilterMock::new();

        let mut serialize = Module::new(mock);
        let buffer = serialize.serialize(TEXT);
        let start = Instant::now();
        for _ in 0..BENCH_LOOP {
            let _ = serialize.deserialize(&buffer);
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in test_deserialize_once is: {:?}",
            duration / BENCH_LOOP as u32
        );
    }
}
