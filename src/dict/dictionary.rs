use std::fs::File;
use std::io::{BufRead, BufReader};
use std::marker::Sync;
use std::rc::Rc;
use std::sync::Mutex;

#[warn(unused_imports)]
use once_cell;
use once_cell::sync::Lazy;

use crate::config::configuration::Configuration;
use crate::dict::hit::Hit;
use crate::dict::trie::Trie;

pub static GLOBAL_DICT: Lazy<Mutex<Dictionary>> = Lazy::new(|| {
    let mut dict = Dictionary::default();
    dict.load();
    Mutex::new(dict)
});

type Dict = Trie;

#[derive(Default)]
/// Dictionary Manager
pub struct Dictionary {
    // 主词典对象
    main_dict: Dict,
    // 停止词词典
    stop_word_dict: Dict,
    // 量词词典
    quantifier_dict: Dict,
    // 配置文件
    cfg: Option<Rc<dyn Configuration>>,
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
    pub fn match_in_main_dict(&mut self, word: &str) -> Vec<Hit> {
        self.main_dict.match_word(word.chars())
    }

    // 检索匹配主词典
    pub fn match_in_main_dict_with_offset(
        &mut self,
        word: &str,
        offset: usize,
        length: usize,
    ) -> Vec<Hit> {
        self.main_dict.match_word_with_offset(word.chars(), offset, length)
    }

    // 检索匹配量词词典
    pub fn match_in_quantifier_dict(
        &mut self,
        word: &str,
        offset: usize,
        length: usize,
    ) -> Vec<Hit> {
        self.quantifier_dict
            .match_word_with_offset(word.chars(), offset, length)
    }

    // 判断是否是停止词
    pub fn is_stop_word(&mut self, word: &str, offset: usize, length: usize) -> bool {
        let hits = self
            .stop_word_dict
            .match_word_with_offset(word.chars(), offset, length);
        for hit in hits.iter() {
            if hit.is_match() {
                return true;
            }
        }
        false
    }

    // 加载主词典及扩展词典
    fn load_main_dict(&mut self) -> bool {
        let main_dict_path = self.cfg.as_ref().unwrap().as_ref().get_main_dictionary();
        // 读取主词典文件
        let file = File::open(main_dict_path).expect("Open main_dict error!");
        let reader = BufReader::new(file);
        let mut total: usize = 0;
        for line in reader.lines() {
            match line {
                Ok(word) => {
                    self.main_dict.insert(word.trim().chars());
                    total += 1;
                }
                Err(e) => {
                    panic!("main dict read error:{}", e);
                }
            }
        }
        println!("load main_dict size = {}", total);
        // 加载扩展词典
        self.load_ext_dict()
    }

    // 加载用户配置的扩展词典到主词库表
    fn load_ext_dict(&mut self) -> bool {
        let ext_dict_files = self.cfg.as_ref().unwrap().get_ext_dictionaries();
        let mut total = 0;
        for ext_dict_file in ext_dict_files {
            let file = File::open(ext_dict_file).expect("open error");
            let reader = BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok(word) => {
                        self.main_dict.insert(word.trim().chars());
                        total += 1;
                    }
                    Err(e) => {
                        panic!("ext dict read error:{}", e);
                    }
                }
            }
        }
        println!("ext dict total size = {}", total);
        true
    }

    // 加载用户扩展的停止词词典
    fn load_stop_word_dict(&mut self) -> bool {
        // 加载扩展停止词典
        let ext_stop_word_dict_files = self
            .cfg
            .as_ref()
            .unwrap()
            .as_ref()
            .get_ext_stop_word_dictionaries();
        let mut total = 0_usize;
        for stop_file in ext_stop_word_dict_files {
            println!("{}", stop_file);
            let file = File::open(stop_file).expect("open error");
            let reader = BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok(word) => {
                        self.stop_word_dict.insert(word.trim().chars());
                        total += 1;
                    }
                    Err(e) => {
                        panic!("stop dict read error:{}", e);
                    }
                }
            }
        }
        println!("stop dict total size = {}", total);
        true
    }

    // 加载量词词典
    fn load_quantifier_dict(&mut self) -> bool {
        // 建立一个量词典实例
        let file_path = self
            .cfg
            .as_ref()
            .unwrap()
            .as_ref()
            .get_quantifier_dictionary();
        let file = File::open(&file_path[..]).expect("open error");
        let reader = BufReader::new(file);
        let mut total = 0_usize;
        for line in reader.lines() {
            match line {
                Ok(word) => {
                    self.quantifier_dict.insert(word.trim().chars());
                    total += 1;
                }
                Err(e) => {
                    panic!("quantifier dict read error:{}", e);
                }
            }
        }
        println!("quantifier_dict total size = {}", total);
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
        assert_eq!(true, initialized);
        let words = vec!["abcd", "blues"];
        dictionary.add_words(words);

        let vec_exist = vec!["一夕之间", "ab", "万般皆下品唯有读书高", "张三", "张"];
        println!("{}", "一夕之间".to_string().len());
        for word in vec_exist {
            let hits = dictionary.match_in_main_dict(word);
            assert_eq!(true, !hits.is_empty());
        }
    }
}
