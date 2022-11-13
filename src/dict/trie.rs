use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::dict::hit::Hit;

#[derive(Debug, Default)]
pub struct TrieNode {
    value: Option<char>,
    final_state: bool,
    child_nodes: HashMap<char, TrieNode>,
}

impl Display for TrieNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TrieNode[value:{:?}, final_state:{}, childs:{}]",
            self.value,
            self.final_state,
            self.child_nodes.len()
        )
    }
}

impl TrieNode {
    pub fn new(c: char, final_state: bool) -> Self {
        TrieNode {
            value: Some(c),
            final_state,
            child_nodes: HashMap::new(),
        }
    }

    pub fn has_childs(&self) -> bool {
        !self.child_nodes.is_empty()
    }

    pub fn is_final_state(&self) -> bool {
        self.final_state
    }

    pub fn check_value(self, c: char) -> bool {
        self.value == Some(c)
    }

    pub fn add_child(&mut self, c: char, final_state: bool) {
        self.child_nodes.insert(c, TrieNode::new(c, final_state));
    }

    pub fn exist<C: Iterator<Item = char>>(&self, chars: C) -> bool {
        let mut current_node = self;
        for c in chars {
            if !current_node.child_nodes.contains_key(&c) {
                return false;
            }
            current_node = current_node.child_nodes.get(&c).unwrap();
        }
        current_node.final_state
    }

    pub fn delete<C: Iterator<Item = char>>(&mut self, chars: C) -> bool {
        let mut current_node = self;
        for c in chars {
            if !current_node.child_nodes.contains_key(&c) {
                return true;
            }
            current_node = current_node.child_nodes.get_mut(&c).unwrap();
        }
        current_node.final_state = false;
        true
    }

    pub fn insert<C: Iterator<Item = char>>(&mut self, chars: C) {
        let mut current_node = self;
        let char_list: Vec<char> = chars.collect();
        let mut final_state = false;

        for (idx, c) in char_list.iter().enumerate() {
            if !current_node.child_nodes.contains_key(c) {
                if idx == char_list.len() - 1 {
                    final_state = true;
                }
                current_node.add_child(*c, final_state);
            }
            current_node = current_node.child_nodes.get_mut(c).unwrap();
        }
    }

    pub fn match_with_offset(
        &self,
        char_list: Vec<char>,
        offset: usize,
        length: usize,
    ) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut current_node = self;
        if offset + length <= char_list.len() {
            let mut end = offset;
            for (counter, c) in char_list.iter().enumerate().skip(offset).take(length) {
                if !current_node.child_nodes.contains_key(c) {
                    break;
                }
                if current_node.final_state {
                    let mut hit = Hit::new();
                    hit.begin = offset;
                    hit.end = end;
                    hit.set_match();
                    if current_node.has_childs() {
                        hit.set_prefix();
                    }
                    hits.push(hit);
                }
                current_node = current_node.child_nodes.get(c).unwrap();
                end = counter;
            }
            if current_node.value.is_some() {
                let mut hit = Hit::new();
                hit.begin = offset;
                hit.end = end;
                if current_node.final_state {
                    hit.set_match();
                }
                if current_node.has_childs() {
                    hit.set_prefix();
                }
                hits.push(hit);
            }
        }
        hits
    }
}

#[derive(Debug, Default)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn insert<C: Iterator<Item = char>>(&mut self, chars: C) {
        let current_node = &mut self.root;
        current_node.insert(chars)
    }

    pub fn delete<C: Iterator<Item = char>>(&mut self, chars: C) -> bool {
        let current_node = &mut self.root;
        current_node.delete(chars)
    }

    pub fn exist<C: Iterator<Item = char>>(&mut self, chars: C) -> bool {
        let current_node = &mut self.root;
        current_node.exist(chars)
    }

    pub fn match_word<C: Iterator<Item = char>>(&mut self, chars: C) -> Vec<Hit> {
        let root_node = &mut self.root;
        let char_list: Vec<char> = chars.collect();
        let length = char_list.len();
        root_node.match_with_offset(char_list, 0, length)
    }

    pub fn match_word_with_offset<C: Iterator<Item = char>>(
        &mut self,
        chars: C,
        offset: usize,
        length: usize,
    ) -> Vec<Hit> {
        let root_node = &mut self.root;
        let char_list = chars.collect();
        root_node.match_with_offset(char_list, offset, length)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn trie_exist() {
        let mut trie = Trie::default();
        trie.insert("Test".chars());
        trie.insert("Tea".chars());
        trie.insert("Background".chars());
        trie.insert("Back".chars());
        trie.insert("Brown".chars());
        trie.insert("申艳超".chars());
        trie.insert("blues小站".chars());

        assert!(!trie.exist("Testing".chars()));
        assert!(trie.exist("Brown".chars()));
        assert!(trie.exist("申艳超".chars()));
        assert!(!trie.exist("申超".chars()));
    }

    #[test]
    fn trie_search() {
        let mut trie = Trie::default();
        trie.insert("Test".chars());
        trie.insert("Tea".chars());
        trie.insert("Background".chars());
        trie.insert("Back".chars());
        trie.insert("Brown".chars());
        trie.insert("申艳超".chars());

        let hits = trie.match_word("申艳超".chars());
        assert_eq!(1, hits.len());
        let hits = trie.match_word("Tea".chars());
        for hit in hits.iter() {
            println!("{:?}", hit);
        }
    }
}
