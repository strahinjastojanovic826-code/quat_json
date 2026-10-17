#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Quat {
    Q0 = 0b00,
    Q1 = 0b01,
    Q2 = 0b10,
    Q3 = 0b11,
}

impl Quat {
    #[inline]
    pub const fn from_u8(val: u8) -> Option<Self> {
        match val & 0b11 {
            0b00 => Some(Self::Q0),
            0b01 => Some(Self::Q1),
            0b10 => Some(Self::Q2),
            0b11 => Some(Self::Q3),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct QuatBuffer {
    pub data: Vec<u8>,
    pub total_quats: usize,
}

#[inline]
pub fn pack_quats(quats: &[Quat]) -> QuatBuffer {
    let total_quats = quats.len();
    let mut data = Vec::with_capacity((total_quats + 3) / 4);
    
    for chunk in quats.chunks(4) {
        let mut byte = 0u8;
        for (i, &q) in chunk.iter().enumerate() {
            byte |= (q as u8) << (i * 2);
        }
        data.push(byte);
    }

    QuatBuffer { data, total_quats }
}

#[inline]
pub fn unpack_quats(bytes: &[u8], total_quats: usize) -> Vec<Quat> {
    let mut quats = Vec::with_capacity(total_quats);
    let mut count = 0;

    for &byte in bytes {
        for i in 0..4 {
            if count >= total_quats {
                break;
            }
            let val = (byte >> (i * 2)) & 0b11;
            if let Some(q) = Quat::from_u8(val) {
                quats.push(q);
            }
            count += 1;
        }
    }
    quats
}