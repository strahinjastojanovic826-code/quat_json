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

    /// Counts how many object or array boundaries exist in the token slice.
    pub fn container_count(&self) -> usize {
        self.tokens
            .iter()
            .filter(|t| matches!(t, QuatTokenType::ObjectStart | QuatTokenType::ArrayStart))
            .count()
    }

    /// Returns a zero-copy subslice starting at `start_idx` clamped to `tokens.len()`.
    pub fn slice_from(&self, start_idx: usize) -> &'a [QuatTokenType] {
        if start_idx >= self.tokens.len() {
            &[]
        } else {
            &self.tokens[start_idx..]
        }
    }

    /// Advances past a container starting at `start_idx` and returns the next index.
    pub fn skip_container(&self, start_idx: usize) -> usize {
        if start_idx >= self.tokens.len() {
            return self.tokens.len();
        }
        let mut depth: usize = 0;
        let mut i = start_idx;
        while i < self.tokens.len() {
            match self.tokens[i] {
                QuatTokenType::ObjectStart | QuatTokenType::ArrayStart => depth += 1,
                QuatTokenType::Separator => {
                    if depth > 0 {
                        depth -= 1;
                        if depth == 0 {
                            return i + 1;
                        }
                    }
                }
                QuatTokenType::ValueNode => {
                    if depth == 0 {
                        return i + 1;
                    }
                }
            }
            i += 1;
        }
        self.tokens.len()
    }
    
}