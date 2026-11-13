use crate::quat::Quat;

/// Represents 4-valued logic states:
/// Q0 = False (00)
/// Q1 = True (01)
/// Q2 = Unknown / Null (10)
/// Q3 = High-Impedance / Error / Reserved (11)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QuatLogicState {
    False = 0b00,
    True = 0b01,
    Unknown = 0b10,
    Error = 0b11,
}

impl QuatLogicState {
    pub fn to_quat(self) -> Quat {
        match self {
            Self::False => Quat::Q0,
            Self::True => Quat::Q1,
            Self::Unknown => Quat::Q2,
            Self::Error => Quat::Q3,
        }
    }

    pub fn from_quat(q: Quat) -> Self {
        match q {
            Quat::Q0 => Self::False,
            Quat::Q1 => Self::True,
            Quat::Q2 => Self::Unknown,
            Quat::Q3 => Self::Error,
        }
    }

    /// Basic Quaternary NOT operation (Cyclic shift or inversion)
    pub fn quat_not(self) -> Self {
        match self {
            Self::False => Self::True,
            Self::True => Self::False,
            Self::Unknown => Self::Error,
            Self::Error => Self::Unknown,
        }
    }

    /// Quaternary AND (conservative minimum logic)
    pub fn quat_and(a: Self, b: Self) -> Self {
        match (a, b) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::True, Self::True) => Self::True,
            _ => Self::Unknown,
        }
    }

    /// Quaternary OR (conservative maximum logic)
    pub fn quat_or(a: Self, b: Self) -> Self {
        match (a, b) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::False, Self::False) => Self::False,
            _ => Self::Unknown,
        }
    }
}