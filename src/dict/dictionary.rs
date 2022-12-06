use std::marker::Sync;
use std::sync::Mutex;

#[warn(unused_imports)]
use once_cell;
use once_cell::sync::Lazy;

static DEFAULT_MAIN_DICT: &str = include_str!("../../dict/main2012.dic");
static DEFAULT_QUANTIFIER_DICT: &str = include_str!("../../dict/quantifier.dic");
static DEFAULT_STOPWORD_DICT: &str = include_str!("../../dict/stopword.dic");

use crate::dict::hit::Hit;
use crate::dict::trie::Trie;

pub static GLOBAL_DICT: Lazy<Mutex<Dictionary>> = Lazy::new(|| {
    let mut dict = Dictionary::default();
    dict.load();
    Mutex::new(dict)
});

type Dict = Trie;

/// Dictionary Manager
pub struct Dictionary {
    // 主词典对象
    main_dict: Dict,
    // 停止词词典
    stop_word_dict: Dict,
    // 量词词典
    quantifier_dict: Dict,
}

impl Default for Dictionary {
    fn default() -> Self {
        Self {
            main_dict: Dict::default(),
            stop_word_dict: Dict::default(),
            quantifier_dict: Dict::default(),
        }
    }
}

unsafe impl Sync for Dictionary {}
unsafe impl Send for Dictionary {}

impl Dictionary {
    pub fn load(&mut self) -> bool {
        self.load_main_dict() && self.load_stop_word_dict() && self.load_quantifier_dict()
    }

    // 批量加载新词条
    pub fn add_words(&mut self, words: Vec<&str>) {
        for word in words {
            self.main_dict.insert(word.chars());
        }
    }

    // 批量移除（屏蔽）词条
    pub fn disable_words(&mut self, words: Vec<&str>) {
        for word in words {
            self.main_dict.delete(word.chars());
        }
    }

    // 检索匹配主词典
    pub fn match_in_main_dict<C: IntoIterator<Item = char>>(&mut self, word: C) -> Vec<Hit> {
        self.main_dict.match_word(word.into_iter())
    }

    // 检索匹配主词典
    pub fn match_in_main_dict_with_offset<C: IntoIterator<Item = char>>(
        &mut self,
        word: C,
        offset: usize,
        length: usize,
    ) -> Vec<Hit> {
        self.main_dict
            .match_word_with_offset(word.into_iter(), offset, length)
    }

    // 检索匹配量词词典
    pub fn match_in_quantifier_dict<C: IntoIterator<Item = char>>(
        &mut self,
        word: C,
        offset: usize,
        length: usize,
    ) -> Vec<Hit> {
        self.quantifier_dict
            .match_word_with_offset(word.into_iter(), offset, length)
    }

    // 判断是否是停止词
    pub fn is_stop_word<C: IntoIterator<Item = char>>(
        &mut self,
        word: C,
        offset: usize,
        length: usize,
    ) -> bool {
        let hits = self
            .stop_word_dict
            .match_word_with_offset(word.into_iter(), offset, length);
        for hit in hits.iter() {
            if hit.is_match() && hit.begin == offset && hit.end == offset + length - 1 {
                return true;
            }
        }
        false
    }

    // 加载主词典及扩展词典
    fn load_main_dict(&mut self) -> bool {
        let dict = DEFAULT_MAIN_DICT.split("\n");
        let mut total: usize = 0;
        for line in dict {
            self.main_dict.insert(line.trim().chars());
            total += 1;
        }
        log::trace!("load main dict size = {}", total);
        true
    }

    // 加载停用词词典
    fn load_stop_word_dict(&mut self) -> bool {
        let dict = DEFAULT_STOPWORD_DICT.split("\n");
        let mut total: usize = 0;
        for line in dict {
            self.stop_word_dict.insert(line.trim().chars());
            total += 1;
        }
        log::trace!("load stopword dict size = {}", total);
        true
    }

    // 加载量词词典
    fn load_quantifier_dict(&mut self) -> bool {
        // 建立一个量词典实例
        let dict = DEFAULT_QUANTIFIER_DICT.split("\n");
        let mut total: usize = 0;
        for line in dict {
            self.quantifier_dict.insert(line.trim().chars());
            total += 1;
        }
        log::trace!("load quantifier dict size = {}", total);
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_dictionary() {
        let mut dictionary = Dictionary::default();
        let initialized = dictionary.load();
        assert!(initialized);
        let words = vec!["abcd", "blues"];
        dictionary.add_words(words);

        let vec_exist = vec!["一夕之间", "ab", "万般皆下品唯有读书高", "张三", "张"];
        println!("{}", "一夕之间".to_string().len());
        for word in vec_exist {
            let hits = dictionary.match_in_main_dict(word.chars());
            assert!(!hits.is_empty());
        }
    }
}
