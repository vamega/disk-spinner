use std::io;

use rand::RngCore as _;

use super::GarbageGenerator;

pub struct ShishuaGenerator {
    rng: ::shishua::ShiShuARng,
}

impl GarbageGenerator for ShishuaGenerator {}

impl ShishuaGenerator {
    pub(super) fn new(seed: u64) -> Self {
        Self {
            rng: ::shishua::ShiShuARng::new(to_shishua_seed(seed)),
        }
    }
}

impl io::Read for ShishuaGenerator {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.rng.fill_bytes(buf);
        Ok(buf.len())
    }
}

fn to_shishua_seed(seed: u64) -> [u64; 4] {
    // Match the old CLI-backed generator, which passed this u64 as a single
    // hex word to `shishua --seed`.
    [seed, 0, 0, 0]
}

#[cfg(test)]
mod test {
    use std::io::Read as _;

    use super::{to_shishua_seed, ShishuaGenerator};

    #[test]
    fn maps_disk_spinner_seed_like_the_old_cli_wrapper() {
        assert_eq!(
            to_shishua_seed(0x1234_5678_9ABC_DEF0),
            [0x1234_5678_9ABC_DEF0, 0, 0, 0]
        );
    }

    #[test]
    fn produces_repeatable_output_for_the_same_seed() {
        let mut first = ShishuaGenerator::new(0x1234_5678_9ABC_DEF0);
        let mut second = ShishuaGenerator::new(0x1234_5678_9ABC_DEF0);
        let mut first_bytes = [0; 257];
        let mut second_bytes = [0; 257];

        first.read_exact(&mut first_bytes).unwrap();
        second.read_exact(&mut second_bytes).unwrap();

        assert_eq!(first_bytes, second_bytes);
    }

    #[test]
    fn consumes_successive_bytes_across_reads() {
        let mut one_read = ShishuaGenerator::new(42);
        let mut split_reads = ShishuaGenerator::new(42);
        let mut expected = [0; 128];
        let mut actual = [0; 128];

        one_read.read_exact(&mut expected).unwrap();
        split_reads.read_exact(&mut actual[..17]).unwrap();
        split_reads.read_exact(&mut actual[17..]).unwrap();

        assert_eq!(actual, expected);
    }
}
