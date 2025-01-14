use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ptr::NonNull;

use crate::core::lexeme::Lexeme;
use crate::core::ordered_linked_list::{Node, OrderedLinkedList};

// Lexeme链（路径）
pub struct LexemePath {
    // 起始位置
    pub path_begin: i32,
    // 结束
    pub path_end: i32,
    // 词元链的有效字符长度
    pub payload_length: usize,
    pub lexeme_list: OrderedLinkedList<Lexeme>,
}

impl Default for LexemePath {
    fn default() -> Self {
        Self::new()
    }
}

impl LexemePath {
    pub fn new() -> Self {
        LexemePath {
            path_begin: -1,
            path_end: -1,
            payload_length: 0,
            lexeme_list: OrderedLinkedList::new(),
        }
    }

    // 向LexemePath追加相交的Lexeme
    // 应当针按照 Lexeme 的顺序依次调用此函数
    // 如果 lexeme 和 lexeme_list 没有冲突，则不添加到 lexeme_list 中， 否则将更新 lexeme_list
    pub fn add_cross_lexeme(&mut self, lexeme: &Lexeme) -> bool {
        // lexeme_list 为空
        if self.lexeme_list.is_empty() {
            self.lexeme_list
                .insert(lexeme.clone())
                .expect("add cross lexeme error!");
            self.path_begin = lexeme.get_begin() as i32;
            self.path_end = (lexeme.get_begin() + lexeme.get_length()) as i32;
            self.payload_length += lexeme.get_length();
            true
        } else if self.check_cross(lexeme) {
            // 当前 lexeme 和 lexeme_list 冲突
            self.lexeme_list
                .insert(lexeme.clone())
                .expect("add cross lexeme error!");
            if (lexeme.get_begin() + lexeme.get_length()) as i32 > self.path_end {
                self.path_end = (lexeme.get_begin() + lexeme.get_length()) as i32;
            }
            self.payload_length = (self.path_end - self.path_begin) as usize;
            return true;
        } else {
            return false;
        }
    }

    //  向LexemePath追加不相交的Lexeme
    pub fn add_not_cross_lexeme(&mut self, lexeme: &Lexeme) -> bool {
        if self.lexeme_list.is_empty() {
            self.lexeme_list
                .insert(lexeme.clone())
                .expect("add not cross lexeme error");
            self.path_begin = lexeme.get_begin() as i32;
            self.path_end = (lexeme.get_begin() + lexeme.get_length()) as i32;
            self.payload_length += lexeme.get_length();
            true
        } else if self.check_cross(lexeme) {
            return false;
        } else {
            self.lexeme_list
                .insert(lexeme.clone())
                .expect("add no cross lexeme error");
            self.payload_length += lexeme.get_length();
            let head = self.lexeme_list.peek_front(); //  peekFirst();
            self.path_begin = head.unwrap().get_begin() as i32;
            let tail = self.lexeme_list.peek_back(); //  peekLast();
            self.path_end =
                (tail.unwrap().get_begin() as i32) + (tail.unwrap().get_length() as i32);
            return true;
        }
    }

    /// 移除尾部的Lexeme
    pub fn remove_tail(&mut self) -> Option<Lexeme> {
        let tail = self.lexeme_list.pop_back();
        if self.lexeme_list.is_empty() {
            self.path_begin = -1;
            self.path_end = -1;
            self.payload_length = 0;
        } else {
            self.payload_length -= tail.as_ref().unwrap().get_length();
            let new_tail = self.lexeme_list.peek_back();
            self.path_end = (new_tail.as_ref().unwrap().get_begin() as i32)
                + (new_tail.as_ref().unwrap().get_length() as i32);
        }
        tail
    }

    // 检测词元位置交叉（有歧义的切分）
    pub fn check_cross(&self, lexeme: &Lexeme) -> bool {
        let l_begin = lexeme.get_begin() as i32;
        let l_length = lexeme.get_length() as i32;

        (l_begin >= self.path_begin && l_begin < self.path_end)
            || (self.path_begin >= l_begin && self.path_begin < l_begin + l_length)
    }

    pub fn get_path_begin(&self) -> i32 {
        self.path_begin
    }

    pub fn get_path_end(&self) -> i32 {
        self.path_end
    }

    pub fn get_payload_length(&self) -> usize {
        self.payload_length
    }

    pub fn get_path_length(&self) -> usize {
        (self.path_end - self.path_begin) as usize
    }

    // X权重（词元长度积）
    pub fn get_xweight(&self) -> i32 {
        let mut product = 1;
        for lexeme in self.lexeme_list.iter() {
            product *= lexeme.get_length();
        }
        product as i32
    }

    // 词元位置权重
    pub fn get_pweight(&self) -> i32 {
        let mut p_weight = 0;
        let mut p = 0;
        for lexeme in self.lexeme_list.iter() {
            p += 1;
            p_weight += p * lexeme.get_length();
        }
        p_weight as i32
    }

    pub fn size(&self) -> usize {
        self.lexeme_list.length()
    }

    pub fn poll_first(&mut self) -> Option<Lexeme> {
        self.lexeme_list.pop_front()
    }

    pub fn get_head(&self) -> Option<&NonNull<Node<Lexeme>>> {
        self.lexeme_list.head_node()
    }
}

impl Display for LexemePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "path_begin:{}, path_end:{}, payload_length:{}, lexeme_list:{}",
            self.path_begin, self.path_end, self.payload_length, self.lexeme_list
        )
    }
}

impl Clone for LexemePath {
    fn clone(&self) -> Self {
        let mut the_copy = LexemePath::new();
        the_copy.path_begin = self.path_begin;
        the_copy.path_end = self.path_end;
        the_copy.payload_length = self.payload_length;
        for lexeme in self.lexeme_list.iter() {
            the_copy
                .lexeme_list
                .insert(lexeme.clone())
                .expect("clone insert error");
        }
        the_copy
    }
}

impl Ord for LexemePath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd<Self> for LexemePath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 比较有效文本长度
        Some(match self.payload_length.cmp(&other.payload_length) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => match self.size().cmp(&other.size()) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => match self.get_path_length().cmp(&other.get_path_length()) {
                    Ordering::Less => Ordering::Greater,
                    Ordering::Greater => Ordering::Less,
                    Ordering::Equal => match self.path_end.cmp(&other.path_end) {
                        Ordering::Less => Ordering::Greater,
                        Ordering::Greater => Ordering::Less,
                        Ordering::Equal => match self.get_xweight().cmp(&other.get_xweight()) {
                            Ordering::Less => Ordering::Greater,
                            Ordering::Greater => Ordering::Less,
                            Ordering::Equal => other.get_pweight().cmp(&self.get_pweight()),
                        },
                    },
                },
            },
        })
    }
}

impl Eq for LexemePath {}
impl PartialEq for LexemePath {
    fn eq(&self, other: &Self) -> bool {
        if self.path_begin == other.path_begin
            && self.path_end == other.path_end
            && self.payload_length == other.payload_length
            && self.lexeme_list.length() == other.lexeme_list.length()
        {
            for _ in 0..self.lexeme_list.length() {
                let a = self.lexeme_list.iter().next().unwrap();
                let b = other.lexeme_list.iter().next().unwrap();
                if !a.eq(b) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}
