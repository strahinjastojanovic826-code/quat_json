use crate::ast::QuatTokenType;

pub struct QuatQueryEngine<'a> {
    tokens: &'a [QuatTokenType],
}

impl<'a> QuatQueryEngine<'a> {
    /// Initialize the query engine with decoded quaternary tokens
    pub fn new(tokens: &'a [QuatTokenType]) -> Self {
        Self { tokens }
    }

    /// Count occurrences of a specific quaternary token type (e.g., ObjectStart, ValueNode)
    pub fn count_token_type(&self, target: QuatTokenType) -> usize {
        self.tokens.iter().filter (|&t| *t == target).count()
    }

    /// Find keys/values sequentially without full DOM overhead
    pub fn find_keys(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        for (i, token) in self.tokens.iter().enumerate() {
            // Q2 (ValueNode/Separator context depending on AST mapping)
            if *token == QuatTokenType::ValueNode {
                indices.push(i);
            }
        }
        indices
    }
}