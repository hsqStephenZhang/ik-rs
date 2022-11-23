#[allow(dead_code)]
pub mod config;
#[allow(dead_code)]
pub mod core;
#[allow(dead_code)]
pub mod dict;

use std::sync::Mutex;

use once_cell::sync::Lazy;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream, Tokenizer};

use crate::core::char_util::regularize_str;
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
        let regular_str = regularize_str(text);
        let text = regular_str.as_str();
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
                "都有",
                "光明",
                "的",
                "前途",
            ],
        );

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
        test_once(TEXT, TokenMode::SEARCH, vec!["一二三四五六七八九十"]);
    }

    #[test]
    fn test_regularize() {
        test_once("Ａｄｅ", TokenMode::INDEX, vec!["Ade"])
    }

    #[test]
    fn test_full1() {
        test_once(
            "我家的后面有",
            TokenMode::INDEX,
            vec!["我家", "的", "后面", "面有"],
        );
        test_once(
            "我家的后面有",
            TokenMode::SEARCH,
            vec!["我家", "的", "后", "面有"],
        );
    }

    #[test]
    fn test_full2() {
        test_once(
            "一块根",
            TokenMode::INDEX,
            vec!["一块", "一", "块根", "块", "根"],
        );
        test_once("一块根", TokenMode::SEARCH, vec!["一", "块根"]);
    }

    #[test]
    fn test_full3() {
        test_once(
            "蒙在小说的绣像上一个个描下来，象习字时候的影写一样",
            TokenMode::INDEX,
            vec![
                "蒙在",
                "小说",
                "的",
                "绣像",
                "上一个",
                "一个个",
                "一个",
                "一",
                "个个",
                "个",
                "个",
                "描",
                "下来",
                "象",
                "习字",
                "时候",
                "的",
                "影",
                "写",
                "一样",
                "一",
                "样",
            ],
        );
        test_once(
            "蒙在小说的绣像上一个个描下来，象习字时候的影写一样",
            TokenMode::SEARCH,
            vec![
                "蒙在",
                "小说",
                "的",
                "绣像",
                "上",
                "一个个",
                "描",
                "下来",
                "象",
                "习字",
                "时候",
                "的",
                "影",
                "写",
                "一样",
            ],
        );
    }

    // “十八” 这个量词既在 main_dict 出现，也在量词中出现，发生冲突
    #[test]
    #[should_panic]
    fn test_full4() {
        test_once("十八日", TokenMode::INDEX, vec!["十八日", "十八", "八日"]);
    }

    // 合并了量词
    #[test]
    #[should_panic]
    fn test_full5() {
        test_once("一两天", TokenMode::INDEX, vec!["一两", "两天", "两", "天"]);
    }

    #[test]
    fn test_stop_word() {
        test_once("is：issue：feed", TokenMode::INDEX, vec!["issue", "feed"]);
    }
}
