use crate::quat::Quat;

pub struct QuatStreamEncoder {
    buffer: Vec<u8>,
    current_byte: u8,
    quat_count: usize,
    total_quats: usize,
}

impl QuatStreamEncoder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            current_byte: 0,
            quat_count: 0,
            total_quats: 0,
        }
    }

    /// Push a single quaternary value (Q0, Q1, Q2, Q3) into the stream
    pub fn push_quat(&mut self, q: Quat) {
        let shift = (self.quat_count % 4) * 2;
        self.current_byte |= (q as u8) << shift;
        self.quat_count += 1;
        self.total_quats += 1;

        if self.quat_count == 4 {
            self.buffer.push(self.current_byte);
            self.current_byte = 0;
            self.quat_count = 0;
        }
    }

    /// Finalize stream and return the packed buffer bytes and total quats
    pub fn finalize(mut self) -> (Vec<u8>, usize) {
        if self.quat_count > 0 {
            self.buffer.push(self.current_byte);
        }
        (self.buffer, self.total_quats)
    }
}