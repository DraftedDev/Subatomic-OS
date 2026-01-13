use core::ops::RangeInclusive;

/// A trait for random-number-generation algorithms.
///
/// # Comparison of Kernel RNG Algorithms
///
/// | Feature | PCG32 | Xoshiro256** | ChaCha20 |
/// | :--- | :--- | :--- | :--- |
/// | **State Size** | **64-bit** | 256-bit | 512-bit |
/// | **Secure** | No | No | **Yes** |
/// | **Performance** | High | **Extremely High** | Medium |
/// | **Code Size** | **Very Small** | Small | Medium |
/// | **Best Use Case** | General purpose utility | High-frequency simulations | Security-critical |
/// | **Weaknesses** | Statistical bias in small seeds | Zero-state trap | Slower for single values |
pub trait Rng {
    /// Create a new RNG by using the architecture-specific [seed](crate::api::seed) function.
    ///
    /// The `quality` flag specified if the seed should be high quality or fast.
    fn new(quality: bool) -> Self
    where
        Self: Sized,
    {
        Self::new_seed(crate::api::seed(quality))
    }

    /// Create a new RNG with the given `u64` seed.
    fn new_seed(seed: u64) -> Self;

    /// Seed the RNG with the given `u64`.
    fn seed(&mut self, seed: u64);

    /// Generate a random `u32`.
    ///
    /// Every other method of the [Rng] trait is built on this.
    /// so it needs to be carefully implemented.
    fn generate(&mut self) -> u32;

    /// Generate a random 32-bit unsigned integer in the specified range.
    fn uint(&mut self, range: RangeInclusive<u32>) -> u32 {
        let low = *range.start();
        let high = *range.end();

        // wrapping_sub handles the case where high - low + 1 is exactly 2^32
        let span = high.wrapping_sub(low).wrapping_add(1);

        if span == 0 {
            // A span of 0 in wrapping math means 2^32 (the full range)
            self.generate()
        } else {
            low + (self.generate() % span)
        }
    }

    /// Generate a random 32-bit signed integer in the specified range.
    fn int(&mut self, range: RangeInclusive<i32>) -> i32 {
        let low = *range.start();
        let high = *range.end();
        let diff = high.wrapping_sub(low) as u32;

        let span = diff.wrapping_add(1);

        if span == 0 {
            self.generate() as i32
        } else {
            let offset = self.generate() % span;
            low.wrapping_add(offset as i32)
        }
    }

    /// Generate a random 32-bit floating point integer ranging from `0.0` to `1.0`.
    fn float(&mut self) -> f32 {
        (self.generate() as f32) / (u32::MAX as f32)
    }

    /// Generate a random boolean.
    fn bool(&mut self) -> bool {
        (self.generate() & 1) == 1
    }
}

/// The PCG32 (XSH-RR variant) algorithm.
///
/// See [Rng] for a comparison of RNG-algorithms.
pub struct Pcg32Rng {
    state: u64,
    inc: u64,
}

impl Rng for Pcg32Rng {
    fn new_seed(seed: u64) -> Self {
        let mut pcg = Self {
            state: 0,
            inc: 0xda3e39cb94b95bdb,
        };
        pcg.seed(seed);
        pcg
    }

    fn seed(&mut self, seed: u64) {
        self.state = 0;
        self.state = self.state.wrapping_add(seed);
        self.generate();
    }

    fn generate(&mut self) -> u32 {
        let old_state = self.state;

        self.state = old_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.inc | 1);

        let rot = (old_state >> 59) as u32;
        let xor_shifted = (((old_state >> 18) ^ old_state) >> 27) as u32;

        (xor_shifted >> rot) | (xor_shifted << ((!rot).wrapping_add(1) & 31))
    }
}

/// The Xoshiro256 algorithm.
///
/// See [Rng] for a comparison of RNG-algorithms.
pub struct Xoshiro256 {
    state: [u64; 4],
}

impl Rng for Xoshiro256 {
    fn new_seed(seed: u64) -> Self {
        let mut rng = Self { state: [0; 4] };
        rng.seed(seed);
        rng
    }

    fn seed(&mut self, seed: u64) {
        let mut sm_state = seed;

        for i in 0..4 {
            sm_state = sm_state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = sm_state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            self.state[i] = z ^ (z >> 31);
        }

        if self.state.iter().all(|&x| x == 0) {
            self.state[0] = seed | 1;
        }
    }

    fn generate(&mut self) -> u32 {
        let result = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);

        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);

        (result >> 32) as u32
    }
}

/// The ChaCha20 algorithm.
///
/// See [Rng] for a comparison of RNG-algorithms.
pub struct ChaCha20Rng {
    state: [u32; 16],
    buffer: [u32; 16],
    cursor: usize,
}

impl ChaCha20Rng {
    fn next_block(&mut self) {
        let mut x = self.state;

        macro_rules! qr {
            ($a:expr, $b:expr, $c:expr, $d:expr) => {
                x[$a] = x[$a].wrapping_add(x[$b]);
                x[$d] ^= x[$a];
                x[$d] = x[$d].rotate_left(16);
                x[$c] = x[$c].wrapping_add(x[$d]);
                x[$b] ^= x[$c];
                x[$b] = x[$b].rotate_left(12);
                x[$a] = x[$a].wrapping_add(x[$b]);
                x[$d] ^= x[$a];
                x[$d] = x[$d].rotate_left(8);
                x[$c] = x[$c].wrapping_add(x[$d]);
                x[$b] ^= x[$c];
                x[$b] = x[$b].rotate_left(7);
            };
        }

        // Perform 20 rounds
        for _ in 0..10 {
            // Column rounds
            qr!(0, 4, 8, 12);
            qr!(1, 5, 9, 13);
            qr!(2, 6, 10, 14);
            qr!(3, 7, 11, 15);
            // Diagonal rounds
            qr!(0, 5, 10, 15);
            qr!(1, 6, 11, 12);
            qr!(2, 7, 8, 13);
            qr!(3, 4, 9, 14);
        }

        // Add original state to scrambled state (feed-forward)
        for (i, val) in x.iter().enumerate() {
            self.buffer[i] = val.wrapping_add(self.state[i]);
        }

        // Increment the 64-bit counter (state indices 12 and 13)
        let (n, overflow) = self.state[12].overflowing_add(1);
        self.state[12] = n;
        if overflow {
            self.state[13] = self.state[13].wrapping_add(1);
        }

        self.cursor = 0;
    }
}

impl Rng for ChaCha20Rng {
    fn new_seed(seed: u64) -> Self {
        let mut rng = Self {
            state: [0; 16],
            buffer: [0; 16],
            cursor: 16, // Ensure we generate a block on first call
        };
        rng.seed(seed);
        rng
    }

    fn seed(&mut self, seed: u64) {
        // Magic constants "expand 32-byte k"
        self.state[0] = 0x61707865;
        self.state[1] = 0x3320646e;
        self.state[2] = 0x79622d32;
        self.state[3] = 0x6b206574;

        // Key setup (split u64 seed across the first two key slots)
        self.state[4] = seed as u32;
        self.state[5] = (seed >> 32) as u32;

        // Fill the rest of the key (6-11) with deterministic constants derived from seed
        // This ensures the 256-bit key space is fully utilized.
        let mut mix = seed;
        for i in 6..12 {
            mix = mix.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0x1);
            self.state[i] = (mix ^ (mix >> 32)) as u32;
        }

        // Block counter (12-13)
        self.state[12] = 0;
        self.state[13] = 0;

        // Use RDTSC for the Nonce (14-15)
        let tsc = unsafe { core::arch::x86_64::_rdtsc() };
        self.state[14] = tsc as u32;
        self.state[15] = (tsc >> 32) as u32;

        self.next_block();
    }

    fn generate(&mut self) -> u32 {
        if self.cursor >= 16 {
            self.next_block();
        }
        let val = self.buffer[self.cursor];
        self.cursor += 1;
        val
    }
}
