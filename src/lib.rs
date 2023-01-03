use bitvec::prelude::{bitvec, BitVec};
use rand_core::RngCore;
use std::fs::File;
use std::io::BufWriter;

const SIGMA: f32 = 1.9;
const TWO_SIGMA_SQUARED: f32 = 2.0 * SIGMA * SIGMA;

pub struct BlueNoise<R: RngCore> {
    width: usize,
    pattern: BitVec,
    pattern_iterations: Vec<BitVec>,
    should_capture_iterations: bool,
    lut: Vec<f32>,
    rng: R,
    ranks: Vec<usize>,
    noise: Vec<u8>,
}

impl<R: RngCore> BlueNoise<R> {
    pub fn new(width: usize, should_capture_iterations: bool, rng: R) -> Self {
        let len = width * width;
        Self {
            width,
            pattern: bitvec![1; len],
            pattern_iterations: Vec::new(),
            should_capture_iterations,
            lut: vec![0.0f32; len],
            rng,
            ranks: vec![0; len],
            noise: vec![0; len],
        }
    }

    pub fn init(&mut self) {
        self.make_seed();
        self.make_initial_pattern();
        self.make_lut(true);
        self.phase_1();
        self.phase_2();
        self.make_lut(false);
        self.phase_3();
        self.make_blue_noise();
    }

    fn make_seed(&mut self) {
        let starting_amount = self.pattern.len() / 10;

        for _ in 0..starting_amount {
            let index = self.rng.next_u32() as usize % self.pattern.len();
            self.write_pattern_value(index, false);
        }

        if self.should_capture_iterations {
            self.pattern_iterations.push(self.pattern.clone());
        }
    }

    fn make_initial_pattern(&mut self) {
        loop {
            // TODO: Panicless?
            let tightest_cluster_index = self.find_tightest_cluster().unwrap();
            self.write_pattern_value(tightest_cluster_index, false);

            // TODO: Panicless?
            let largest_void_index = self.find_largest_void().unwrap();
            self.write_pattern_value(largest_void_index, true);

            if self.should_capture_iterations {
                self.pattern_iterations.push(self.pattern.clone());
            }

            if largest_void_index == tightest_cluster_index {
                break;
            }
        }
    }

    fn make_lut(&mut self, write_ones: bool) {
        self.lut.fill(0.0);

        for i in 0..self.pattern.len() {
            if self.pattern[i] == write_ones {
                self.write_pattern_value(i, write_ones);
            }
        }
    }

    fn find_tightest_cluster(&self) -> Option<usize> {
        self.find_lut_winner(true)
    }

    fn find_largest_void(&self) -> Option<usize> {
        self.find_lut_winner(false)
    }

    fn find_lut_winner(&self, cluster: bool) -> Option<usize> {
        let mut best_value = if cluster { -f32::MAX } else { f32::MAX };
        let mut best_indices = Vec::new();

        for i in 0..self.lut.len() {
            let energy = self.lut[i];
            let bit = self.pattern[i];
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

    pub fn write_pattern_value(&mut self, index: usize, value: bool) {
        // set bit
        self.pattern.set(index, value);

        // update LUT
        let x = index % self.width;
        let y = index / self.width;
        let len = self.width * self.width;

        for i in 0..len {
            let px = i % self.width;
            let py = i / self.width;

            let mut disty = (py as f32 - y as f32).abs();

            if disty > (self.width / 2) as f32 {
                disty = self.width as f32 - disty;
            }

            let mut distx = (px as f32 - x as f32).abs();

            if distx > (self.width / 2) as f32 {
                distx = self.width as f32 - distx;
            }

            let distance_squared = (distx * distx) as f32 + (disty * disty) as f32;

            let energy =
                (-distance_squared / TWO_SIGMA_SQUARED).exp() * if value { 1.0 } else { -1.0 };

            self.lut[i] += energy;
        }
    }

    pub fn phase_1(&mut self) {
        let mut ones = self.pattern.count_ones();
        // let starting_ones = ones;
        while ones > 0 {
            // TODO: Panicless?
            let tightest_cluster_index = self.find_tightest_cluster().unwrap();
            self.write_pattern_value(tightest_cluster_index, false);
            ones -= 1;
            self.ranks[tightest_cluster_index] = ones;
        }
    }

    pub fn phase_2(&mut self) {
        let mut ones = self.pattern.count_ones();
        // let starting_ones = ones;
        while ones <= self.pattern.len() / 2 {
            // let ones_done = ones - starting_ones;
            // TODO: Panicless?
            let largest_void_index = self.find_largest_void().unwrap();
            self.write_pattern_value(largest_void_index, true);
            self.ranks[largest_void_index] = ones;
            ones += 1;
        }
    }

    pub fn phase_3(&mut self) {
        let mut ones = self.pattern.count_ones();

        while let Some(largest_void_index) = self.find_largest_void() {
            self.write_pattern_value(largest_void_index, true);
            self.ranks[largest_void_index] = ones;
            ones += 1;
        }
    }

    pub fn make_blue_noise(&mut self) {
        for i in 0..self.pattern.len() {
            self.noise[i] = (self.ranks[i] * 256 / self.pattern.len()) as u8;
        }
    }

    pub fn write_noise_png(&self, file_name: &str) {
        let file = File::create(file_name).unwrap();
        let file_writer = BufWriter::new(file);

        let width = self.width as u32;
        let mut encoder = png::Encoder::new(file_writer, width, width);

        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);

        let mut png_writer = encoder.write_header().unwrap();

        png_writer.write_image_data(&self.noise).unwrap();
    }

    pub fn write_pattern_iteration_pngs(&self, file_prefix: &str) {
        for (i, pattern) in self.pattern_iterations.iter().enumerate() {
            let file = File::create(format!("{}{}.png", file_prefix, i)).unwrap();
            let file_writer = BufWriter::new(file);

            let width = self.width as u32;
            let mut encoder = png::Encoder::new(file_writer, width, width);

            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);

            let mut png_writer = encoder.write_header().unwrap();

            let image: Vec<u8> = pattern
                .iter()
                .map(|x| if *x.as_ref() { 255 } else { 0 })
                .collect();

            png_writer.write_image_data(&image).unwrap();
        }
    }
}
