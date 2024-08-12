use image::{ImageReader, Rgb, RgbImage};
use divan::counter::BytesCount;
use divan::AllocProfiler;
use divan::Bencher;


fn main() {
    let img = ImageReader::open("receipt.jpg").unwrap().decode().unwrap();
    let img = img.to_rgb8();
    let mut img: RgbImage = image::imageops::rotate270(&img).into();

    divan::main();
}



#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn get_image_byte_count() -> BytesCount {
    let img = ImageReader::open("receipt.jpg").unwrap().decode().unwrap();
    let img = img.to_rgb8();
    let img: RgbImage = image::imageops::rotate270(&img).into();

    let (width, height) = img.dimensions();
    let bytes = width * height * 3 * 4;
    BytesCount::new(bytes)
}

#[divan::bench]
fn contrast_filter_ints_bencher(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let img = ImageReader::open("receipt.jpg").unwrap().decode().unwrap();
            let img = img.to_rgb8();
            let img: RgbImage = image::imageops::rotate270(&img).into();
            img
        })
        .counter(get_image_byte_count())
        .bench_values(|mut img| add_contrast_filter_ints(&mut img));
}


#[divan::bench]
fn contrast_filter_floats_bencher(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let img = ImageReader::open("receipt.jpg").unwrap().decode().unwrap();
            let img = img.to_rgb8();
            let img: RgbImage = image::imageops::rotate270(&img).into();
            img
        })
        .counter(get_image_byte_count())
        .bench_values(|mut img| add_contrast_filter_floats(&mut img));
}

fn add_contrast_filter_ints(img: &mut RgbImage) {
    for pixel in img.enumerate_pixels_mut() {
        let rgb_values = &mut pixel.2.0;
        let luma = (rgb_values[0] as u32) * 2126
            + (rgb_values[1] as u32) * 7152
            + (rgb_values[2] as u32) * 722;
        if luma < 1125000 {
            rgb_values[0] = 0;
            rgb_values[1] = 0;
            rgb_values[2] = 0;
        } else {
            rgb_values[0] = 255;
            rgb_values[1] = 255;
            rgb_values[2] = 255;
        }
    }
}

fn add_contrast_filter_floats(img: &mut RgbImage) {
    for pixel in img.enumerate_pixels_mut() {
        let rgb_values = &mut pixel.2.0;
        let luma = 0.2126f32 * rgb_values[0] as f32
            + 0.7152 * rgb_values[1] as f32
            + 0.0722 * rgb_values[2] as f32;
        if luma < 112.5 {
            rgb_values[0] = 0;
            rgb_values[1] = 0;
            rgb_values[2] = 0;
        } else {
            rgb_values[0] = 255;
            rgb_values[1] = 255;
            rgb_values[2] = 255;
        }
    }
}
