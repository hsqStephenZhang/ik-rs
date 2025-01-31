use crate::core::char_util::{char_type_of, CharType};
use crate::core::lexeme::{Lexeme, LexemeType};
use crate::core::segmentor::Segmenter;
use crate::dict::dictionary::GLOBAL_DICT;

const SEGMENTER_NAME: &str = "CJK_SEGMENTER";

// 中文-日韩文子分词器
#[derive(Debug, Default)]
pub struct CJKSegmenter {}

impl Segmenter for CJKSegmenter {
    fn analyze(&mut self, chars: &[char]) -> Vec<Lexeme> {
        let mut new_lexemes: Vec<Lexeme> = Vec::new();
        let char_count = chars.len();
        for (cursor, curr_char) in chars.iter().enumerate() {
            let curr_char_type = char_type_of(curr_char);
            if CharType::USELESS != curr_char_type {
                let hit_options = GLOBAL_DICT.lock().unwrap().match_in_main_dict_with_offset(
                    chars.iter().copied(),
                    cursor,
                    char_count - cursor,
                );
                for hit in hit_options.iter() {
                    if hit.is_match() {
                        // 输出当前的词
                        let new_lexeme =
                            Lexeme::new(0, hit.begin, hit.end - hit.begin + 1, LexemeType::CNWORD);
                        new_lexemes.push(new_lexeme);
                    }
                }
            }
        }
        new_lexemes
    }

    fn name(&self) -> &str {
        SEGMENTER_NAME
    }
}

impl CJKSegmenter {
    pub fn new() -> Self {
        CJKSegmenter {}
    }
}
