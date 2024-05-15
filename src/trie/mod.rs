use crate::dictionary::Filter;
use std::collections::{HashMap, HashSet};

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

    /// Find matching string in the trie graph returning it index num if found or None otherwise.
    ///
    pub fn find_match(&self, s: &str) -> Option<u32> {
        let mut curr = self;
        for c in s.chars() {
            curr = curr.nodes.get(&c)?;
        }
        curr.num
    }

    fn append_inner(&self, nums: &mut HashSet<u32>) {
        for (_, next) in self.nodes.iter() {
            if let Some(num) = next.num {
                nums.insert(num);
            }
            next.append_inner(nums);
        }
    }

    fn collect_numbers(&self, numbers: &mut HashSet<u32>) {
        if let Some(number) = self.num {
            numbers.insert(number);
        }
        
        let mut nodes : Vec<&Node> = Vec::new();
        for (_, node) in &self.nodes { 
            nodes.push(node);
        }

        while !nodes.is_empty() {
            let mut temp : Vec<&Node> = Vec::new();

            for element in &nodes {
                if let Some(number) = element.num { 
                    numbers.insert(number);
                }

                for (_, node) in &element.nodes {
                    temp.push(node);
                }
            }

            nodes = temp;
        }
    }
}

impl Filter for Node {
    /// Push string in to the trie graph giving it a num index.
    /// Num index shall be unique and it is not the case of trie to validate it uniqueness.
    ///
    fn push(&mut self, s: &str, num: u32) {
        let mut curr = self;
        for c in s.chars() {
            curr = curr.nodes.entry(c).or_insert_with(|| Box::new(Node::new()));
        }
        curr.num = Some(num);
    }

    /// Finds all index nums with matching string prefix.
    ///
    fn find_prefix(&self, s: &str) -> HashSet<u32> {
        let mut curr = self;
        let mut nums = HashSet::new();
        for c in s.chars() {
            let Some(node) = curr.nodes.get(&c) else {
                return nums;
            };
            curr = node;
        }
        
        curr.collect_numbers(&mut nums);
        nums
    }

    fn find_prefix_case_insensitive(&self, s: &str) -> HashSet<u32> {
        let mut result = HashSet::new();

        for (idx, char) in s.chars().enumerate() {
            let Some(upper) = char.to_uppercase().next() else {
                return result;
            };
            if let Some(node) = self.nodes.get(&upper) {
                if let Some(num) = node.num {
                    result.insert(num);
                }
                node.find_prefix_case_insensitive(&s[idx + 1..])
                    .iter()
                    .for_each(|el| {
                        result.insert(*el);
                    });
            }

            let Some(lower) = char.to_lowercase().next() else {
                return result;
            };
            if let Some(node) = self.nodes.get(&lower) {
                if let Some(num) = node.num {
                    result.insert(num);
                }
                node.find_prefix_case_insensitive(&s[idx + 1..])
                    .iter()
                    .for_each(|el| {
                        result.insert(*el);
                    });
            }
        }

        if s.is_empty() {
            self.append_inner(&mut result);
        }

        result
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

    const TEST_WORDS_NOT_PUSH: [&str; 11] = [
        "a",
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
    fn on_find_match_of_pushed_words_should_find_all_matching_words() {
        let mut root = Node::new();
        TEST_WORDS_PUSH
            .iter()
            .enumerate()
            .for_each(|w| root.push(w.1, w.0 as u32));

        for (i, w) in TEST_WORDS_PUSH.iter().enumerate() {
            assert_eq!(root.find_match(w).unwrap(), i as u32);
        }
    }

    #[test]
    fn on_find_match_of_not_pushed_words_should_find_no_matching_words() {
        let mut root = Node::new();
        for (i, w) in TEST_WORDS_PUSH.iter().enumerate() {
            root.push(w, i as u32);
        }

        for (_, w) in TEST_WORDS_NOT_PUSH.iter().enumerate() {
            assert!(root.find_match(w).is_none());
        }
    }

    #[test]
    fn on_find_prefix_should_find_all_matching_words_case_sensitive() {
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
    fn bench_push() {
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
    fn bench_find_match() {
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
    fn bench_find_prefix() {
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

    #[test]
    fn on_find_prefix_should_find_matches_when_case_insensitive() {
        let mut root = Node::new();
        vec![
            ("ALA", 0),
            ("noise", 1),
            ("ala", 2),
            ("Ala", 3),
            ("Abba", 4),
            ("Aaala", 5),
        ]
        .iter()
        .for_each(|el| root.push(el.0, el.1));
        let expected = HashSet::from_iter([0, 2, 3]);

        let actual = root.find_prefix_case_insensitive("al");

        assert_eq!(expected, actual);
    }

    #[test]
    fn on_collect_numbers_should_retrieve_numbers_from_decendent_nodes() {
        let mut root = Node::new();
        vec![
            ("inn", 0),
            ("in", 1),
            ("inner", 2),
            ("i", 3),
            ("innest", 4),
        ].iter().for_each(|(s, idx)| root.push(s, *idx));
        let mut node = &root;
        "inn".chars().for_each(|char| node = node.nodes.get(&char).unwrap());

        println!("{:#?}", node);
        let mut actual = HashSet::new();
        node.collect_numbers(&mut actual);

        assert_eq!(HashSet::from_iter([0, 2, 4]), actual);
    }
}
