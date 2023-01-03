use std::fs;
use void_n_cluster::BlueNoise;

fn main() {
    let rng = rand::thread_rng();
    let width = 124;
    let mut blue_noise = BlueNoise::new(width, true, rng);
    blue_noise.init();
    let _ = fs::create_dir("./out");
    blue_noise.write_pattern_iteration_pngs("./out/b1_pattern");
    blue_noise.write_noise_png("./out/b1.png");
}
