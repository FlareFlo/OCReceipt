use std::collections::HashSet;
use std::ops::Index;
use std::time::Instant;
use image::{GenericImage, GenericImageView, ImageReader, Rgb, RgbImage};

fn main() {
    let img = ImageReader::open("receipt.jpg").unwrap().decode().unwrap();
    let img = img.to_rgb8();
    let mut img: RgbImage = image::imageops::rotate270(&img).into();

    let start_contrast = Instant::now();
    add_contrast_filter(&mut img);
    let end_contrast = start_contrast.elapsed();

    img.save("receipt_contrast.png").unwrap();

    let start_blob = Instant::now();
    let all_bounds = blob_find(&img);
    let end_blob = start_blob.elapsed();

    let start_boxes = Instant::now();
    draw_bounding_boxes(&mut img, &all_bounds);
    let end_boxes = start_boxes.elapsed();

    img.save("receipt_with_bounding_boxes.png").unwrap();

    let time_contrast = end_contrast.as_millis();
    let time_blob = end_blob.as_millis();
    let time_boxes = end_boxes.as_millis();

    println!("Contrast took {time_contrast} millis");
    println!("Blob find took {time_blob} millis");
    println!("Boxes took {time_boxes} millis");
}

fn draw_bounding_boxes(img: &mut RgbImage, all_bounds: &HashSet<((u32, u32), (u32, u32))>) {
    let (width, height) = img.dimensions();
    let box_color = Rgb([0, 255, 0]);
    for (top_left, bottom_right) in all_bounds {
        for x in (top_left.0..=bottom_right.0) {
            if x > 0 && x < width {
                img.put_pixel(x, top_left.1, box_color.clone());
                img.put_pixel(x, bottom_right.1, box_color.clone());
            }
        }
        for y in (top_left.1..=bottom_right.1) {
            if y > 0 && y < height {
                img.put_pixel(top_left.0, y, box_color.clone());
                img.put_pixel(bottom_right.0, y, box_color.clone());
            }
        }
    }
}

fn add_contrast_filter(img: &mut RgbImage) {
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

fn blob_find(img: &RgbImage) -> HashSet<((u32, u32), (u32, u32))> {
    let mut black_pixels: Vec<_> = img
        .enumerate_pixels()
        .filter(|p| is_black(&p.2.0))
        .collect();
    let mut all_bounds = HashSet::new();
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    while !black_pixels.is_empty() {
        let pixel = black_pixels.pop().unwrap(); // it exists (!stack.is_empty())
        stack.push((pixel.0, pixel.1));
        if visited.contains(&(pixel.0, pixel.1)) {
            continue;
        }
        let mut bounds = new_bounds();
        while !stack.is_empty() {
            let (curr_x, curr_y) = stack.pop().unwrap(); // it exists (!stack.is_empty())
            if visited.contains(&(curr_x, curr_y)) {
                continue;
            }

            let pixel = img.get_pixel_checked(curr_x, curr_y);
            if let Some(pixel) = pixel {
                if is_black(&pixel.0) {
                    update_bounds(&mut bounds, (curr_x, curr_y));
                    stack.push((curr_x + 1, curr_y));
                    stack.push((curr_x, curr_y + 1));
                    stack.push((curr_x - 1, curr_y));
                    stack.push((curr_x , curr_y - 1));
                }
            }

            visited.insert((curr_x, curr_y));
        }

        //print_bounds(&bounds);
        all_bounds.insert(bounds);
        stack.clear();
    }

    all_bounds
}

fn print_bounds(
    (top_left, bottom_right): &((u32, u32), (u32, u32))
) {
    println!(
        "top left: ({}, {}), bottom right: ({}, {})",
        top_left.0,
        top_left.1,
        bottom_right.0,
        bottom_right.1,
    )
}

fn update_bounds(
    (top_left, bottom_right): &mut ((u32, u32), (u32, u32)),
    (x, y): (u32, u32)
) {
    if x < top_left.0 { top_left.0 = x }
    if y < top_left.1 { top_left.1 = y }
    if x > bottom_right.0 { bottom_right.0 = x }
    if y > bottom_right.1 { bottom_right.1 = y }
}

fn new_bounds() -> ((u32, u32), (u32, u32)) {
    ((u32::MAX, u32::MAX), (u32::MIN, u32::MIN))
}

struct Square {
    x: u32,
    y: u32,
    pixels: [[[u8; 3]; 3]; 3]
}

impl Square {
    fn xy(&self) -> (u32, u32) {
        (self.x, self.y)
    }
    
    fn x(&self) -> u32 {
        self.x
    }
    
    fn y(&self) -> u32 {
        self.y
    }
    
    fn contains_black(&self) -> bool {
        self.count_blacks() > 0
    }

    fn count_blacks(&self) -> u8 {
        let mut sum = 0;
        for row in self.pixels {
            for item in row {
                sum += item[0]
            }
        }

        sum / 255
    }
}

impl Index<usize> for Square {
    type Output = [[u8; 3]; 3];

    fn index(&self, y: usize) -> &Self::Output {
        &self.pixels[y]
    }
}

// x, y is the top left corner of the square
fn get_square(img: &RgbImage, x: u32, y: u32) -> Square {
    let pixels = [
        [img.get_pixel(x, y).0, img.get_pixel(x + 1, y).0, img.get_pixel(x + 2, y).0],
        [img.get_pixel(x, y + 1).0, img.get_pixel(x + 1, y + 1).0, img.get_pixel(x + 2, y + 1).0],
        [img.get_pixel(x, y + 2).0, img.get_pixel(x + 1, y + 2).0, img.get_pixel(x + 2, y + 2).0],
    ];

    Square {
        x,
        y,
        pixels
    }
}

fn is_black(rgb: &[u8;3]) -> bool {
    (rgb[0] as u16 + rgb[1] as u16 + rgb[2] as u16) == 0
}

