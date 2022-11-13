use std::collections::HashSet;

use crate::core::char_util::{char_type_of, CharType};
use crate::core::lexeme::{Lexeme, LexemeType};
use crate::core::segmentor::Segmenter;
use crate::dict::dictionary::GLOBAL_DICT;

const SEGMENTER_NAME: &str = "QUAN_SEGMENTER";

#[derive(Debug)]
pub struct CnQuantifierSegmenter {
    n_start: i32,
    n_end: i32,
    chn_number_chars: HashSet<char>,
}

impl Segmenter for CnQuantifierSegmenter {
    fn analyze(&mut self, input: &[char]) -> Vec<Lexeme> {
        // 处理中文数词
        let a = self.process_cnumber(input);
        // 处理中文量词
        let b = self.process_count(input);
        let mut new_lexemes: Vec<Lexeme> = Vec::with_capacity(a.len() + b.len());
        new_lexemes.extend(a);
        new_lexemes.extend(b);
        new_lexemes
    }
    fn name(&self) -> &str {
        SEGMENTER_NAME
    }
}

impl Default for CnQuantifierSegmenter {
    fn default() -> Self {
        Self::new()
    }
}

impl CnQuantifierSegmenter {
    pub fn new() -> Self {
        CnQuantifierSegmenter {
            n_start: -1,
            n_end: -1,
            chn_number_chars: HashSet::from([
                '一', '二', '两', '三', '四', '五', '六', '七', '八', '九', '十', '零', '壹', '贰',
                '叁', '肆', '伍', '陆', '柒', '捌', '玖', '拾', '百', '千', '万', '亿', '拾', '佰',
                '仟', '萬', '億', '兆', '卅', '廿',
            ]),
        }
    }

    // 处理数词
    pub fn process_cnumber(&mut self, input: &[char]) -> Vec<Lexeme> {
        let mut new_lexemes = Vec::new();
        let input_length = input.len();
        for (cursor, curr_char) in input.iter().enumerate() {
            let curr_char_type = char_type_of(curr_char);
            if self.n_start == -1 && self.n_end == -1 {
                // 初始状态
                if CharType::CHINESE == curr_char_type && self.chn_number_chars.contains(curr_char)
                {
                    // 记录数词的起始、结束位置
                    self.n_start = cursor as i32;
                    self.n_end = cursor as i32;
                }
            } else {
                // 正在处理状态
                if CharType::CHINESE == curr_char_type && self.chn_number_chars.contains(curr_char)
                {
                    // 记录数词的结束位置
                    self.n_end = cursor as i32;
                } else {
                    // 输出数词
                    let new_lexeme = Lexeme::new(
                        0,
                        self.n_start as usize,
                        (self.n_end - self.n_start + 1) as usize,
                        LexemeType::CNUM,
                    );
                    new_lexemes.push(new_lexeme);
                    // 重置头尾指针
                    self.n_start = -1;
                    self.n_end = -1;
                }
            }

            // 缓冲区已经用完，还有尚未输出的数词
            if cursor == input_length - 1 && self.n_start != -1 && self.n_end != -1 {
                // 输出数词
                let new_lexeme = Lexeme::new(
                    0,
                    self.n_start as usize,
                    (self.n_end - self.n_start + 1) as usize,
                    LexemeType::CNUM,
                );
                new_lexemes.push(new_lexeme);
                // 重置头尾指针
                self.n_start = -1;
                self.n_end = -1;
            }
        }
        new_lexemes
    }

    //  处理中文量词
    pub fn process_count(&mut self, chars: &[char]) -> Vec<Lexeme> {
        let mut new_lexemes = Vec::new();
        // 判断是否需要启动量词扫描
        if self.need_count_scan() {
            let char_count = chars.len();
            for (cursor, curr_char) in chars.iter().enumerate() {
                let curr_char_type = char_type_of(curr_char);
                if CharType::CHINESE == curr_char_type {
                    let hit_options = GLOBAL_DICT.lock().unwrap().match_in_quantifier_dict(
                        chars.iter().copied(),
                        cursor,
                        char_count - cursor,
                    );
                    for hit in hit_options.iter() {
                        if hit.is_match() {
                            // 输出当前的词
                            let new_lexeme = Lexeme::new(
                                0,
                                hit.begin,
                                hit.end - hit.begin + 1,
                                LexemeType::COUNT,
                            );
                            new_lexemes.push(new_lexeme);
                        }
                    }
                }
            }
        }
        new_lexemes
    }

    // 判断是否需要扫描量词
    fn need_count_scan(&self) -> bool {
        if self.n_start == -1 || self.n_end == -1 {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn t1() {
        let chars = "一二三四五".chars().collect::<Vec<_>>();
        let mut s = CnQuantifierSegmenter::new();
        let r = s.analyze(&chars);
        assert_ne!(r, Vec::new());
    }
}
