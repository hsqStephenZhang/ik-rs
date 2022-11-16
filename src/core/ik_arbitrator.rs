use std::collections::{BTreeSet, HashMap};
use std::ptr::NonNull;

use crate::core::ik_segmenter::TokenMode;
use crate::core::lexeme::Lexeme;
use crate::core::lexeme_path::LexemePath;
use crate::core::ordered_linked_list::{Node, OrderedLinkedList};

// IK分词歧义裁决器
#[derive(Clone, Default)]
pub struct IKArbitrator {}

impl IKArbitrator {
    pub fn new() -> Self {
        IKArbitrator {}
    }

    // 分词歧义处理
    pub fn process(
        &mut self,
        org_lexemes: &mut OrderedLinkedList<Lexeme>,
        mode: TokenMode,
    ) -> HashMap<usize, LexemePath> {
        let mut path_map = HashMap::<usize, LexemePath>::new();
        let mut cross_path = LexemePath::new();
        let mut cur_node = org_lexemes.head_node();

        let mut handle_once = |path_map: &mut HashMap<usize, LexemePath>,
                           cross_path: LexemePath| {
            if cross_path.size() == 1 || !(mode == TokenMode::SEARCH) {
                // crossPath没有歧义 或者 不做歧义处理
                // 直接输出当前crossPath
                path_map.insert(cross_path.get_path_begin() as usize, cross_path);
            } else {
                // 对当前的crossPath进行歧义处理
                let judge_result = self.judge(cross_path.get_head());
                // 输出歧义处理结果judgeResult
                path_map.insert(
                    judge_result.as_ref().unwrap().get_path_begin() as usize,
                    judge_result.unwrap(),
                );
            }
        };

        while let Some(inner) = cur_node {
            // safety: we own the ordered linked list, so deref the NonNull node is safe
            let org_lexeme = unsafe { &(inner.as_ref().val) };
            if !cross_path.add_cross_lexeme(org_lexeme) {
                // 找到与crossPath不相交的下一个crossPath
                handle_once(&mut path_map, cross_path);
                // 把orgLexeme加入新的crossPath中
                cross_path = LexemePath::new();
                cross_path.add_cross_lexeme(org_lexeme);
            }
            // safety: we own the ordered linked list
            unsafe {
                cur_node = inner.as_ref().next.as_ref();
            }
        }

        // 处理最后的path
        handle_once(&mut path_map, cross_path);
        path_map
    }

    /// 歧义识别
    ///
    /// @param lexeme_cell     歧义路径链表头
    /// @param fullTextLength 歧义路径文本长度
    pub fn judge(&mut self, cur_node: Option<&NonNull<Node<Lexeme>>>) -> Option<LexemePath> {
        // 候选路径集合
        let mut path_options = BTreeSet::new();
        // 候选结果路径
        let mut option_path = LexemePath::new();
        // 对crossPath进行一次遍历,同时返回本次遍历中有冲突的Lexeme栈
        let mut lexeme_stack = self.forward_path(cur_node, &mut option_path);
        // 当前词元链并非最理想的，加入候选路径集合
        path_options.insert(option_path.clone());
        while let Some(c) = lexeme_stack.pop() {
            // rollback path
            self.backward_path(c, &mut option_path);
            // forward path
            self.forward_path(c, &mut option_path);
            path_options.insert(option_path.clone());
        }
        // 返回集合中的最优方案
        path_options.iter().next().cloned()
    }

    // 向前遍历，添加词元，构造一个无歧义词元组合
    // option_path: 无歧义的路径
    // ret: 歧义，待裁决的路径
    pub fn forward_path<'a>(
        &'a self,
        cur_node: Option<&'a NonNull<Node<Lexeme>>>,
        option_path: &mut LexemePath,
    ) -> Vec<Option<&NonNull<Node<Lexeme>>>> {
        // 发生冲突的Lexeme栈
        let mut conflict_stack: Vec<Option<&NonNull<Node<Lexeme>>>> = Vec::new();
        // 迭代遍历Lexeme链表
        let mut cur = cur_node;
        // safety: cur is Some
        while let Some(inner) = cur.as_ref() {
            unsafe {
                let c = &(inner.as_ref().val);
                if !option_path.add_not_cross_lexeme(c) {
                    // 词元交叉，添加失败则加入lexemeStack栈
                    conflict_stack.push(cur);
                }
                cur = inner.as_ref().next.as_ref();
            }
        }
        conflict_stack
    }

    // 回滚词元链，直到它能够接受指定的词元
    pub fn backward_path(&self, l: Option<&NonNull<Node<Lexeme>>>, option: &mut LexemePath) {
        if let Some(lexeme) = l {
            unsafe {
                let lexeme = &lexeme.as_ref().val;
                while option.check_cross(lexeme) {
                    option.remove_tail();
                }
            }
        }
    }
}
