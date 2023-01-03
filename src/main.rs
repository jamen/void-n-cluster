mod blue_noise;

use blue_noise::BlueNoise;

use rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;

fn main() {
    let rng = Xoshiro256Plus::from_entropy();
    let width = 32;
    let mut blue_noise = BlueNoise::new(width, false, rng);
    blue_noise.init();
    blue_noise.write_pattern_iteration_pngs("./out/blue_noise_iter_");
    blue_noise.write_noise_png("./out/blue_noise.png");
}
