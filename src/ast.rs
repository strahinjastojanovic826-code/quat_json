use crate::quat::Quat;
use serde_json::Value;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QuatTokenType {
    ObjectStart = 0b00, // Q0 -> Object boundary
    ArrayStart  = 0b01, // Q1 -> Array boundary
    Separator   = 0b10, // Q2 -> Field/Key separator
    ValueNode   = 0b11, // Q3 -> Scalar literal/value
}

impl QuatTokenType {
    #[inline]
    pub const fn to_quat(self) -> Quat {
        match self {
            Self::ObjectStart => Quat::Q0,
            Self::ArrayStart  => Quat::Q1,
            Self::Separator   => Quat::Q2,
            Self::ValueNode   => Quat::Q3,
        }
    }

    #[inline]
    pub const fn from_quat(q: Quat) -> Self {
        match q {
            Quat::Q0 => Self::ObjectStart,
            Quat::Q1 => Self::ArrayStart,
            Quat::Q2 => Self::Separator,
            Quat::Q3 => Self::ValueNode,
        }
    }
}

pub fn traverse_json_value(val: &Value, quats: &mut Vec<Quat>) {
    match val {
        Value::Object(map) => {
            quats.push(QuatTokenType::ObjectStart.to_quat());
            for (_, v) in map {
                quats.push(QuatTokenType::ValueNode.to_quat());
                quats.push(QuatTokenType::Separator.to_quat());
                traverse_json_value(v, quats);
            }
        }
        Value::Array(arr) => {
            quats.push(QuatTokenType::ArrayStart.to_quat());
            for item in arr {
                traverse_json_value(item, quats);
            }
        }
        _ => {
            quats.push(QuatTokenType::ValueNode.to_quat());
        }
    }
}