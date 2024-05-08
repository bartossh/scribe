use std::collections::HashMap;

/// Node is a part of tries graph.
/// First node is a root of the tree and Contains None number and \0 char.
/// Root node task is to be the entry point in to the graph.
///
#[derive(Debug, Clone)]
pub struct Node {
    num: Option<u32>,
    nodes: HashMap<char, Box<Node>>,
}

impl Node {
    /// Crates a new root node that is an entrance to the graph.
    ///
    pub fn new() -> Self {
        Self {
            num: None,
            nodes: HashMap::new(),
        }
    }

    /// Push string in to the trie graph giving it a num index.
    /// Num index shall be unique and it is not the case of trie to validate it uniqueness.
    ///
    pub fn push(&mut self, s: &str, num: u32) {
        let mut curr = self;
        for c in s.chars() {
            curr = curr.nodes.entry(c).or_insert_with(|| Box::new(Node::new()));
        }
        curr.num = Some(num);
    }

    /// Find matching string in the trie graph returning it index num if found or None otherwise.
    ///
    pub fn find_match(&self, s: &str) -> Option<u32> {
        let mut curr = self;
        for c in s.chars() {
            if let Some(node) = curr.nodes.get(&c) {
                curr = node;
            } else {
                return None;
            }
        }
        curr.num
    }

    /// Finds all index nums with matching string prefix.
    ///
    pub fn find_prefix(&self, s: &str) -> Vec<u32> {
        let mut curr = self;
        let mut nums = Vec::new();
        for c in s.chars() {
            if let Some(node) = curr.nodes.get(&c) {
                if let Some(num) = node.num {
                    nums.push(num);
                }
                curr = node;
            } else {
                return nums;
            }
        }
        curr.append_inner(&mut nums);
        nums
    }

    fn append_inner(&self, nums: &mut Vec<u32>) {
        for (_, next) in self.nodes.iter() {
            if let Some(num) = next.num {
                nums.push(num);
            }
            next.append_inner(nums);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::time::Instant;

    const BENCH_LOOP_SIZE: usize = 100000;
    const BENCH_WORD_SIZE: usize = 12;

    const TEST_WORDS_PUSH: [&str; 10] = [
        "aba",
        "abacus",
        "abacusa",
        "abac",
        "abacusasa",
        "ab",
        "avacusasatasa",
        "ole",
        "oleum",
        "oleole",
    ];

    const TEST_WORDS_NOT_PUSH: [&str; 10] = [
        "abaa",
        "abacusaa",
        "abacusa3",
        "abac2",
        "abacusasaooo",
        "abx",
        "avacusasatasaxcz",
        "olezsa",
        "elo",
        "aloes",
    ];

    fn create_random_str(size: usize) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(size)
            .map(char::from)
            .collect()
    }

    #[test]
    fn test_find_match() {
        let mut root = Node::new();
        for (i, w) in TEST_WORDS_PUSH.iter().enumerate() {
            root.push(w, i as u32);
        }

        for (i, w) in TEST_WORDS_PUSH.iter().enumerate() {
            if let Some(n) = root.find_match(w) {
                assert_eq!(n, i as u32);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_not_find_match() {
        let mut root = Node::new();
        for (i, w) in TEST_WORDS_PUSH.iter().enumerate() {
            root.push(w, i as u32);
        }

        for (_, w) in TEST_WORDS_NOT_PUSH.iter().enumerate() {
            let res = root.find_match(w);
            assert_eq!(res, None);
        }
    }

    #[test]
    fn test_find_prefix() {
        let mut root = Node::new();
        for (i, w) in TEST_WORDS_PUSH.iter().enumerate() {
            root.push(w, i as u32);
        }

        let result = root.find_prefix(TEST_WORDS_PUSH[5]);

        for i in result.iter() {
            if *i as usize > TEST_WORDS_PUSH.len() {
                assert!(false);
                return;
            }
            let w = TEST_WORDS_PUSH[*i as usize];
            assert_eq!(w[0..2], TEST_WORDS_PUSH[5][0..2]);
        }
    }

    #[test]
    fn test_bench_push() {
        let mut words = Vec::new();
        for _ in 0..BENCH_LOOP_SIZE {
            words.push(create_random_str(BENCH_WORD_SIZE));
        }

        let mut root = Node::new();

        let start = Instant::now();
        for (i, w) in words.iter().enumerate() {
            root.push(w, i as u32);
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in test_bench_push is: {:?} for one word in {} words of length {}, total {:?}.",
            duration / BENCH_LOOP_SIZE as u32,
            words.len(),
            BENCH_WORD_SIZE,
            duration
        );
    }

    #[test]
    fn test_bench_find_match() {
        let mut words = Vec::new();
        for _ in 0..BENCH_LOOP_SIZE {
            words.push(create_random_str(BENCH_WORD_SIZE));
        }

        let mut root = Node::new();

        for (i, w) in words.iter().enumerate() {
            root.push(w, i as u32);
        }

        let start = Instant::now();

        for (i, w) in words.iter().enumerate() {
            if let Some(n) = root.find_match(w) {
                assert_eq!(n, i as u32);
            } else {
                assert!(false);
            }
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in test_bench_find_match is: {:?} for one word in {} words of length {}, total: {:?}.",
            duration / BENCH_LOOP_SIZE as u32,
            words.len(),
            BENCH_WORD_SIZE,
            duration
        );
    }

    #[test]
    fn test_bench_find_prefix() {
        let mut words = Vec::new();
        for _ in 0..BENCH_LOOP_SIZE {
            words.push(create_random_str(BENCH_WORD_SIZE));
        }

        let mut root = Node::new();

        for (i, w) in words.iter().enumerate() {
            root.push(w, i as u32);
        }

        let start = Instant::now();

        for (_, w) in words.iter().enumerate() {
            let _ = root.find_prefix(&w[0..BENCH_WORD_SIZE / 2 as usize]);
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in test_bench_find_prefix of size {} is: {:?} for one word in {} words of length {}, total: {:?}.",
            BENCH_WORD_SIZE / 2,
            duration / BENCH_LOOP_SIZE as u32,
            words.len(),
            BENCH_WORD_SIZE,
            duration
        );
    }
}
