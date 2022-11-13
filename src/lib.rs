#[allow(dead_code)]
pub mod config;
#[allow(dead_code)]
pub mod core;
#[allow(dead_code)]
pub mod dict;

use std::sync::Mutex;

use once_cell::sync::Lazy;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream, Tokenizer};

use crate::core::ik_segmenter::{IKSegmenter, TokenMode};

pub static GLOBAL_IK: Lazy<Mutex<IKSegmenter>> = Lazy::new(|| {
    let ik = IKSegmenter::new();
    Mutex::new(ik)
});

#[derive(Clone)]
pub struct IkTokenizer {
    mode: TokenMode,
}

pub struct IkTokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl TokenStream for IkTokenStream {
    fn advance(&mut self) -> bool {
        if self.index < self.tokens.len() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index - 1]
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.index - 1]
    }
}

impl IkTokenizer {
    pub fn new(mode: TokenMode) -> Self {
        Self { mode }
    }
}

impl Tokenizer for IkTokenizer {
    fn token_stream<'a>(&self, text: &'a str) -> BoxTokenStream<'a> {
        let mut indices = text.char_indices().collect::<Vec<_>>();
        indices.push((text.len(), '\0'));
        let orig_tokens = GLOBAL_IK.lock().unwrap().tokenize(text, self.mode);
        let mut tokens = Vec::new();
        for token in orig_tokens.iter() {
            tokens.push(Token {
                offset_from: indices[token.get_begin_position()].0,
                offset_to: indices[token.get_end_position()].0,
                position: token.get_begin(),
                text: String::from(
                    &text[(indices[token.get_begin_position()].0)
                        ..(indices[token.get_end_position()].0)],
                ),
                position_length: token.get_length(),
            });
        }
        BoxTokenStream::from(IkTokenStream { tokens, index: 0 })
    }
}

#[cfg(test)]
mod tests {
    use crate::TokenMode;

    fn test_once(text: &str, mode: TokenMode, expect_tokens: Vec<&str>) {
        use tantivy::tokenizer::*;
        let tokenizer = crate::IkTokenizer::new(mode);
        let mut token_stream = tokenizer.token_stream(text);
        let mut token_text = Vec::new();
        while let Some(token) = token_stream.next() {
            token_text.push(token.text.clone());
        }

        assert_eq!(token_text, expect_tokens);
    }

    #[test]
    fn tantivy_ik_works() {
        const TEXT: &str =
            "张华考上了北京大学；李萍进了中等技术学校；我在百货公司当售货员：我们都有光明的前途";
        test_once(
            TEXT,
            TokenMode::SEARCH,
            vec![
                "张华",
                "考",
                "上了",
                "北京大学",
                "李萍",
                "进了",
                "中等",
                "技术学校",
                "我",
                "在",
                "百货公司",
                "当",
                "售货员",
                "我们",
                "都有",
                "光明",
                "的",
                "前途",
            ],
        );
    }

    #[test]
    fn test_index_tokenizer() {
        const TEXT: &str =
            "张华考上了北京大学；李萍进了中等技术学校；我在百货公司当售货员：我们都有光明的前途";
        test_once(
            TEXT,
            TokenMode::INDEX,
            vec![
                "张华",
                "考上",
                "上了",
                "北京大学",
                "北京大",
                "北京",
                "大学",
                "李萍",
                "进了",
                "中等",
                "技术学校",
                "技术",
                "学校",
                "我",
                "在",
                "百货公司",
                "百货",
                "百",
                "货",
                "公司",
                "当",
                "售货员",
                "售货",
                "货员",
                "我们",
                "我",
                "们",
                "都有",
                "有",
                "光明",
                "的",
                "前途",
            ],
        );
    }

    #[test]
    fn test_cn_quantifier() {
        const TEXT: &str = "一二三四五六七八九十";
        test_once(
            TEXT,
            TokenMode::INDEX,
            vec![
                "一二三四五六七八九十",
                "二三",
                "四五",
                "六七",
                "七八",
                "八九",
                "十",
            ],
        );
    }

    #[test]
    fn test_letters() {
        test_once(
            "Lark Search 综搜质量小分队",
            TokenMode::SEARCH,
            vec!["Lark", "Search", "综", "搜", "质量", "小分队"],
        );
    }
}
