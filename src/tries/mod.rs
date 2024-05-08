use std::borrow::Borrow;

/// Node is a part of tries graph.
/// First node is a root of the tree and Contains None number and \0 char.
/// Root node task is to be the entry point in to the graph.
///
#[derive(Debug, Clone)]
pub struct Node {
    c: char,
    num: Option<u32>,
    nodes: Vec<Node>,
}

impl Node {
    /// Crates a new root node that is an entrance to the graph.
    ///
    pub fn new() -> Self {
        Self {
            c: '\0',
            num: None,
            nodes: Vec::new(),
        }
    }

    pub fn push(&mut self, s: &str, num: u32) {
        let mut curr = self;
        for c in s.chars() {
            curr = match curr.nodes.iter_mut().position(|n| n.c == c) {
                Some(idx) => &mut curr.nodes[idx],
                None => {
                    curr.nodes.push(Node {
                        c: c,
                        num: None,
                        nodes: Vec::new(),
                    });
                    curr.nodes.last_mut().unwrap()
                }
            }
        }
        curr.num = Some(num);
    }

    pub fn find_match(&self, s: &str) -> Option<u32> {
        let mut curr: &Node = self;
        let empty: &Node = &Node {
            c: '\0',
            num: None,
            nodes: Vec::new(),
        };
        for c in s.chars() {
            curr = match curr.nodes.iter().position(|n| n.c == c) {
                Some(idx) => &curr.nodes[idx],
                None => empty,
            }
        }
        curr.num
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::time::Instant;

    const BENCH_LOOP_SIZE: usize = 10000;
    const BENCH_WORD_SIZE: usize = 8;

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
    fn test_push() {
        let mut root = Node::new();
        for (i, w) in TEST_WORDS_PUSH.iter().enumerate() {
            root.push(w, i as u32);
        }

        println!("{:?}", root);
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

        for (i, w) in TEST_WORDS_NOT_PUSH.iter().enumerate() {
            let res = root.find_match(w);
            assert_eq!(res, None);
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
            "Time elapsed in test_bench_push is: {:?} for {} words.",
            duration / BENCH_LOOP_SIZE as u32,
            words.len(),
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
            "Time elapsed in test_bench_find_match is: {:?} for {} words.",
            duration / BENCH_LOOP_SIZE as u32,
            words.len(),
        );
    }
}
