use phf::{phf_set, Set};
use unicode_blocks;

#[derive(Debug, PartialEq)]
pub enum CharType {
    USELESS,
    SPECIAL,
    ARABIC,
    ENGLISH,
    CHINESE,
    OtherCjk,
}

static SPECIAL_CHARS: Set<char> = phf_set! {
    // 英文特殊字符
    '!',
    '"',
    '#',
    '$',
    '%',
    '&',
    '(',
    ')',
    '+',
    ',',
    '-',
    '/',
    ';',
    '<',
    '=',
    '>',
    '?',
    '@',
    '[',
    '\\',
    ']',
    '^',
    '`',
    '{',
    '|',
    '}',
    '~',
    '¥',
    // 中文特殊字符
    '、',
    '。',
    '《',
    '》',
    '「',
    '」',
    '【',
    '】',
    '！',
    '（',
    '）',
    '，',
    '；',
    '？',
    '～',
};

// identify CharType Of char
pub fn char_type_of(input: &char) -> CharType {
    if ('0'..='9').contains(input) {
        return CharType::ARABIC;
    } else if ('a'..='z').contains(input) || ('A'..='Z').contains(input) {
        return CharType::ENGLISH;
    } else if SPECIAL_CHARS.contains(input) {
        return CharType::SPECIAL;
    } else {
        let ub = unicode_blocks::find_unicode_block(*input).unwrap();
        if ub == unicode_blocks::CJK_UNIFIED_IDEOGRAPHS
            || ub == unicode_blocks::CJK_COMPATIBILITY_IDEOGRAPHS
            || ub == unicode_blocks::CJK_UNIFIED_IDEOGRAPHS_EXTENSION_A
        {
            // 目前已知的中文字符UTF-8集合
            return CharType::CHINESE;
        } else if ub == unicode_blocks::HALFWIDTH_AND_FULLWIDTH_FORMS //全角数字字符和日韩字符
                    //韩文字符集
                    || ub == unicode_blocks::HANGUL_SYLLABLES
                    || ub == unicode_blocks::HANGUL_JAMO
                    || ub == unicode_blocks::HANGUL_COMPATIBILITY_JAMO
                    //日文字符集
                    || ub == unicode_blocks::HIRAGANA //平假名
                    || ub == unicode_blocks::KATAKANA //片假名
                    || ub == unicode_blocks::KATAKANA_PHONETIC_EXTENSIONS
        {
            return CharType::OtherCjk;
        }
    }
    CharType::USELESS
}

// full char -> half char && lowercase
pub fn regularize(input: char) -> char {
    let mut input_code = input as u32;
    if input_code == 12288 {
        input_code -= 12256; // 空格
    } else if (65281..=65374).contains(&input_code) {
        input_code -= 65248; // 全角字符
    } else if input_code >= 'A' as u32 && input_code <= 'Z' as u32 {
        input_code += 32; // lowercase
    }

    char::from_u32(input_code).unwrap()
}

pub fn regularize_str(input: &str) -> String {
    let mut regular_str = "".to_string();
    for c in input.chars() {
        regular_str.push(regularize(c));
    }
    regular_str
}

pub fn utf8_slice(s: &str, begin: usize, end: usize) -> &str {
    if end < begin {
        return "";
    }
    s.char_indices()
        .nth(begin)
        .and_then(|(start_pos, _)| {
            if end >= utf8_len(s) {
                return Some(&s[start_pos..]);
            }
            s[start_pos..]
                .char_indices()
                .nth(end - begin)
                .map(|(end_pos, _)| &s[start_pos..start_pos + end_pos])
        })
        .unwrap_or("")
}

pub fn utf8_from(s: &str, begin: usize) -> &str {
    utf8_slice(s, begin, utf8_len(s))
}

pub fn utf8_till(s: &str, end: usize) -> &str {
    utf8_slice(s, 0, end)
}

pub fn utf8_len(s: &str) -> usize {
    s.chars().count()
}
