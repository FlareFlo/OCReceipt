use std::array;
use std::sync::LazyLock;
use image::{ImageReader, Rgb, RgbImage};
use ocr_receipt::{add_contrast_filter, blob_find};
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

#[divan::bench]
fn contrast_filter_flo_bencher(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let img = ImageReader::open("receipt.jpg").unwrap().decode().unwrap();
            let img = img.to_rgb8();
            let img: RgbImage = image::imageops::rotate270(&img).into();
            img
        })
        .counter(get_image_byte_count())
        .bench_values(|mut img| add_contrast_filter_flo(&mut img));
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

fn add_contrast_filter_flo(img: &mut RgbImage) {
    static LLT: LazyLock<[f32; u8::MAX as usize * 3]> = LazyLock::new(|| {
        array::from_fn(|e| match e {
            0..255 => e as f32 * 0.2126f32,
            255..512 => e as f32 * 0.7152,
            512.. => e as f32 * 0.0722,
        })
    });

    let is_luma = |[r, g, b]: [u8; 3]| {
        let r = LLT[r as usize];
        let g = LLT[g as usize + 255];
        let b = LLT[b as usize + 255 * 2];
        (r + g + b) < 112.5
    };

    for pixel in img.enumerate_pixels_mut() {
        let rgb_values = &mut pixel.2 .0;
        if is_luma(*rgb_values) {
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
