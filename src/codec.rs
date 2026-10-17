use crate::quat::{pack_quats, unpack_quats, QuatBuffer};
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
    if data.is_empty() && total_quats > 0 {
        return Err(AdvancedCodecError::InvalidBufferData);
    }
    let quats = unpack_quats(data, total_quats);
    let tokens = quats.into_iter().map(QuatTokenType::from_quat).collect();
    Ok(tokens)
}