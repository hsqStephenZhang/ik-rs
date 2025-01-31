use std::cmp::Ordering;

use crate::core::char_util::utf8_slice;

// lexemeType常量
#[derive(Debug, PartialEq, Clone)]
pub enum LexemeType {
    // 未知 0
    UNKNOWN,
    // 英文 1
    ENGLISH,
    // 数字2
    ARABIC,
    // 英文数字混合3
    LETTER,
    // 中文词元4
    CNWORD,
    // 中文单字64
    CNCHAR,
    // 日韩文字8
    OtherCJK,
    // 中文数词16
    CNUM,
    // 中文量词32
    COUNT,
    // 中文数量词48
    CQUAN,
}

/// IK词元对象
#[derive(Debug, Clone)]
pub struct Lexeme {
    // 词元的起始位移
    offset: usize,
    // 词元的相对起始位置
    begin: usize,
    // 词元的长度
    length: usize,
    // 词元文本
    lexeme_text: String,
    // 词元类型
    pub(crate) lexeme_type: LexemeType,
}

impl PartialEq for Lexeme {
    // 判断词元相等算法: 起始位置偏移、起始位置、终止位置相同
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.begin == other.begin && self.length == other.length
    }
}

impl Eq for Lexeme {}

impl PartialOrd for Lexeme {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 起始位置优先
        Some(self.cmp(other))
    }
}

impl Ord for Lexeme {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.begin.cmp(&other.begin) {
            // 词元长度优先
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => other.length.cmp(&self.length),
        }
    }
}

impl Lexeme {
    pub fn new(offset: usize, begin: usize, length: usize, lexeme_type: LexemeType) -> Self {
        Lexeme {
            offset,
            begin,
            length,
            lexeme_type,
            lexeme_text: String::from(""),
        }
    }

    pub fn get_begin(&self) -> usize {
        self.begin
    }

    // 获取词元在文本中的起始位置
    pub fn get_begin_position(&self) -> usize {
        self.offset + self.begin
    }

    // 获取词元在文本中的结束位置
    pub fn get_end_position(&self) -> usize {
        self.get_begin_position() + self.length
    }

    //  获取词元的字符长度
    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn set_length(&mut self, length: usize) {
        self.length = length;
    }

    // 获取词元的文本内容
    pub fn get_lexeme_text(&self) -> &str {
        &self.lexeme_text
    }

    pub fn parse_lexeme_text(&mut self, input: &str) {
        let sub_text = utf8_slice(input, self.begin, self.begin + self.length);
        self.lexeme_text = sub_text.to_string();
    }

    // 获取词元类型标示字符串
    pub fn get_lexeme_type_string(&self) -> &str {
        match &self.lexeme_type {
            LexemeType::ENGLISH => "ENGLISH",
            LexemeType::ARABIC => "ARABIC",
            LexemeType::LETTER => "LETTER",
            LexemeType::CNWORD => "CN_WORD",
            LexemeType::CNCHAR => "CN_CHAR",
            LexemeType::OtherCJK => "OtherCjk",
            LexemeType::COUNT => "COUNT",
            LexemeType::CNUM => "TYPE_CNUM",
            LexemeType::CQUAN => "TYPE_CQUAN",
            _ => "UNKNOW",
        }
    }

    // 合并两个相邻的词元, 返回 词元是否成功合并
    pub fn append(&mut self, l: &Lexeme, lexeme_type: LexemeType) -> bool {
        if self.get_end_position() == l.get_begin_position() {
            self.length += l.get_length();
            self.lexeme_type = lexeme_type;
            return true;
        }
        false
    }
}
