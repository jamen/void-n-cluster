use bitvec::prelude::{bitvec, BitVec};
use rand_core::RngCore;

const SIGMA: f32 = 1.9;
const TWO_SIGMA_SQUARED: f32 = 2.0 * SIGMA * SIGMA;

struct Pattern {
    size: [u32; 2],
    bits: BitVec,
    lut: Vec<f32>,
}

impl Pattern {
    fn new(size: [u32; 2]) -> Self {
        let len = size[0] * size[1];
        Self {
            size,
            bits: bitvec![0; len as usize],
            lut: vec![0.0; len as usize],
        }
    }

    fn make_lut(&mut self, ones: bool) {
        self.lut.fill(0.0);
        for i in 0..self.bits.len() {
            self.set(i, self.bits[i] == ones);
        }
    }

    fn set(&mut self, index: usize, value: bool) {
        // set bit
        self.bits.set(index, value);

        // update lut
        let [w, h] = self.size;

        for target_index in 0..(w * h) {
            self.lut[target_index as usize] +=
                energy(index as u32, target_index, self.size) * if value { 1.0 } else { -1.0 }
        }
    }

    fn len(&self) -> usize {
        self.bits.len()
    }

    fn tightest_cluster(&self) -> Option<usize> {
        self.find_lut_winner(true)
    }

    fn largest_void(&self) -> Option<usize> {
        self.find_lut_winner(false)
    }

    fn find_lut_winner(&self, cluster: bool) -> Option<usize> {
        let mut best_value = if cluster { -f32::MAX } else { f32::MAX };
        let mut best_indices = Vec::new();

        for i in 0..self.lut.len() {
            let energy = self.lut[i];
            let bit = self.bits[i];
            if bit == cluster {
                if energy == best_value {
                    best_indices.push(i)
                } else if (cluster == true && energy > best_value)
                    || (cluster == false && energy < best_value)
                {
                    best_value = energy;
                    best_indices.clear();
                    best_indices.push(i);
                }
            }
        }

        best_indices.first().copied()
    }
}

pub fn create_blue_noise<R: RngCore>(size: [u32; 2], mut rng: R) -> Vec<u8> {
    let mut pattern = Pattern::new(size);

    // Random seed

    let starting_amount = pattern.len() / 10;

    for _ in 0..starting_amount {
        let index = rng.next_u32() as usize % pattern.len();
        pattern.set(index, true);
    }

    // Initial bits

    loop {
        let tightest_cluster_index = pattern.tightest_cluster().unwrap();
        pattern.set(tightest_cluster_index, false);

        let largest_void_index = pattern.largest_void().unwrap();
        pattern.set(largest_void_index, true);

        if largest_void_index == tightest_cluster_index {
            break;
        }
    }

    // Initial LUT

    pattern.make_lut(true);

    let mut ranks = vec![0; pattern.len()];

    // Phase 1

    let mut ones = pattern.bits.count_ones();

    while ones > 0 {
        let tightest_cluster_index = pattern.tightest_cluster().unwrap();
        pattern.set(tightest_cluster_index, false);
        ones -= 1;
        ranks[tightest_cluster_index] = ones;
    }

    // Phase 2

    let mut ones = pattern.bits.count_ones();

    while ones <= pattern.len() / 2 {
        let index = pattern.largest_void().unwrap();
        pattern.set(index, true);
        ranks[index] = ones;
        ones += 1;
    }

    pattern.make_lut(false);

    // Phase 3

    let mut ones = pattern.bits.count_ones();

    while let Some(largest_void_index) = pattern.largest_void() {
        pattern.set(largest_void_index, true);
        ranks[largest_void_index] = ones;
        ones += 1;
    }

    // Finalize

    let mut noise = vec![0u8; pattern.len()];

    for i in 0..pattern.len() {
        noise[i] = (ranks[i] * 256 / pattern.len()) as u8;
    }

    noise
}

fn energy(a: u32, b: u32, [w, h]: [u32; 2]) -> f32 {
    let ax = a % w;
    let ay = a / h;
    let bx = b % w;
    let by = b / h;

    let mut dx = (bx as f32 - ax as f32).abs();

    if dx > (w / 2) as f32 {
        dx = w as f32 - dx;
    }

    let mut dy = (by as f32 - ay as f32).abs();

    if dy > (h / 2) as f32 {
        dy = h as f32 - dy;
    }

    let distance_squared = (dx * dx) as f32 + (dy * dy) as f32;

    (-distance_squared / TWO_SIGMA_SQUARED).exp()
}
