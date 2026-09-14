//! 伪随机数生成器

#![no_std]

use core::{convert::Infallible, ops::{Bound, RangeBounds}};

use rand_core::{Rng, TryRng};

pub struct CubeRng(pub u64);

impl CubeRng {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            let mut value = self.next_u64();
            for byte in chunk.iter_mut() {
                *byte = value as u8;
                value >>= 8;
            }
        }
    }

    pub fn random(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }

        let range_size = max - min;

        min + (self.next_u32() % range_size)
    }

    pub fn random_range(&mut self, range: impl RangeBounds<usize>) -> usize {
        let panic_empty_range = || panic!("empty range: {:?}..{:?}", range.start_bound(), range.end_bound());

        let low = match range.start_bound() {
            Bound::Unbounded => u32::MIN,
            Bound::Included(&x) => x as u32,
            Bound::Excluded(&x) => (x as u32).checked_add(1).unwrap_or_else(panic_empty_range),
        };

        let high = match range.end_bound() {
            Bound::Unbounded => u32::MAX,
            Bound::Included(&x) => x as u32,
            Bound::Excluded(&x) => (x as u32).checked_sub(1).unwrap_or_else(panic_empty_range),
        };

        if low > high {
            panic_empty_range();
        }

        let range_size = high - low;
        (low + (self.next_u32() % range_size)) as usize
    }
}

impl TryRng for CubeRng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        Ok(self.next_u64() as u32)
    }

    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        Ok(self.0)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Infallible> {
        self.fill_bytes(dest);
        Ok(())
    }
}
