use std::fs;

use std::fs::File;
use std::io::BufWriter;

fn main() {
    let rng = rand::thread_rng();
    let size = [256, 256];
    let noise = void_n_cluster::create_blue_noise(size, rng);

    let _ = fs::create_dir("./out");

    write_noise_png(&noise, size, "./out/noise.png");
}

fn write_noise_png(noise: &Vec<u8>, size: [u32; 2], file_name: &str) {
    let file = File::create(file_name).unwrap();
    let file_writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(file_writer, size[0], size[1]);

    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);

    let mut png_writer = encoder.write_header().unwrap();

    png_writer.write_image_data(&noise).unwrap();
}
