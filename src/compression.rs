use crate::quat::{Quat, QuatBuffer, pack_quats};
use crate::codec::AdvancedCodecError;

pub struct ScalarCompressor;

impl ScalarCompressor {
    /// Compresses a slice of small integer scalars (0 to 3) into an ultra-compact QuatBuffer
    pub fn compress_scalars(scalars: &[u8]) -> Result<QuatBuffer, AdvancedCodecError> {
        if scalars.is_empty() {
            return Err(AdvancedCodecError::EmptyInput);
        }

        let mut quats = Vec::with_capacity(scalars.len());
        for &val in scalars {
            let q = match val & 0b11 {
                0b00 => Quat::Q0,
                0b01 => Quat::Q1,
                0b10 => Quat::Q2,
                _ => Quat::Q3,
            };
            quats.push(q);
        }

        Ok(pack_quats(&quats))
    }

    /// Decompresses a QuatBuffer back into raw scalar values (0-3)
    pub fn decompress_scalars(buffer: &QuatBuffer) -> Vec<u8> {
        let quats = crate::quat::unpack_quats(&buffer.data, buffer.total_quats);
        quats.into_iter().map(|q| q as u8).collect()
    }
}