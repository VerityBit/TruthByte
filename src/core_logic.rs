use std::num::Wrapping;

struct Xorshift32 {
    state: Wrapping<u32>,
}

impl Xorshift32 {
    fn new(seed: u32) -> Self {
        let initial = if seed == 0 { 1 } else { seed };
        Self {
            state: Wrapping(initial),
        }
    }

    #[inline(always)]
    fn next_u8(&mut self) -> u8 {
        let mut x = self.state.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = Wrapping(x);
        x as u8
    }
}

pub fn fill_block(offset: u64, buffer: &mut [u8]) {
    let seed = (offset as u32) ^ ((offset >> 32) as u32);
    let mut rng = Xorshift32::new(seed);

    for byte in buffer.iter_mut() {
        *byte = rng.next_u8();
    }
}

pub fn verify_block(offset: u64, buffer: &[u8]) -> Result<(), usize> {
    let seed = (offset as u32) ^ ((offset >> 32) as u32);
    let mut rng = Xorshift32::new(seed);

    for (index, &actual_byte) in buffer.iter().enumerate() {
        let expected_byte = rng.next_u8();
        if actual_byte != expected_byte {
            return Err(index);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{fill_block, verify_block};

    #[test]
    fn test_determinism() {
        let mut block1 = vec![0u8; 1024];
        let mut block2 = vec![0u8; 1024];
        fill_block(100, &mut block1);
        fill_block(100, &mut block2);
        assert_eq!(block1, block2);
    }

    #[test]
    fn test_offset_variance() {
        let mut block1 = vec![0u8; 1024];
        let mut block2 = vec![0u8; 1024];
        fill_block(100, &mut block1);
        fill_block(200, &mut block2);
        assert_ne!(block1, block2);
    }

    #[test]
    fn test_verification_logic() {
        let offset = 500;
        let mut data = vec![0u8; 256];

        fill_block(offset, &mut data);
        assert!(verify_block(offset, &data).is_ok());

        data[10] = data[10].wrapping_add(1);
        assert_eq!(verify_block(offset, &data), Err(10));
    }
}
