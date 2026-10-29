use crate::quat::{pack_quats, QuatBuffer};
use crate::ast::{traverse_json_value, QuatTokenType};
use serde_json::Value;

#[derive(Debug)]
pub enum AdvancedCodecError {
    JsonParseError(serde_json::Error),
    EmptyInput,
    InvalidBufferData,
}

impl std::fmt::Display for AdvancedCodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonParseError(e) => write!(f, "JSON syntax error: {}", e),
            Self::EmptyInput => write!(f, "Input string cannot be empty"),
            Self::InvalidBufferData => write!(f, "Corrupted or invalid quaternary buffer"),
        }
    }
}

impl std::error::Error for AdvancedCodecError {}

pub fn encode_json(json_str: &str) -> Result<QuatBuffer, AdvancedCodecError> {
    if json_str.trim().is_empty() {
        return Err(AdvancedCodecError::EmptyInput);
    }

    let parsed_json: Value = serde_json::from_str(json_str)
        .map_err(AdvancedCodecError::JsonParseError)?;

    let mut quats = Vec::new();
    traverse_json_value(&parsed_json, &mut quats);

    Ok(pack_quats(&quats))
}

pub fn decode_quats(data: &[u8], total_quats: usize) -> Result<Vec<QuatTokenType>, AdvancedCodecError> {
    // Validate buffer length against expected total_quats (4 quats per byte)
    let required_bytes = (total_quats + 3) / 4;
    if data.len() < required_bytes {
        return Err(AdvancedCodecError::InvalidBufferData);
    }

    let mut tokens = Vec::with_capacity(total_quats);
    let iter = crate::quat::QuatIter::new(data, total_quats);

    for quat in iter {
        let token_type = QuatTokenType::from_quat(quat);
        tokens.push(token_type);
    }

    if tokens.len() != total_quats {
        return Err(AdvancedCodecError::InvalidBufferData);
    }

    Ok(tokens)
}