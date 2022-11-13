use crate::core::lexeme::Lexeme;

pub trait Segmenter {
    fn analyze(&mut self, input: &[char]) -> Vec<Lexeme>;
    fn name(&self) -> &str;
}
