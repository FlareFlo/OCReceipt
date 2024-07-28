use std::cmp::Ordering;
use std::collections::HashSet;
use std::ops::Index;
use std::time::Instant;
use image::{ImageReader, Rgb, RgbImage};

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
struct Point {
    pub x: u32,
    pub y: u32,
}
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
struct BoundBox {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl BoundBox {
    fn height(&self) -> u32 {
        self.bottom_right.y - self.top_left.y
    }

    fn width(&self) -> u32 {
        self.bottom_right.y - self.top_left.y
    }

    fn middle_point(&self) -> Point {
        let x = self.bottom_right.x - (self.width() / 2);
        let y = self.bottom_right.y - (self.height() / 2);
        Point { x, y }
    }
}

impl From<((u32, u32), (u32, u32))> for BoundBox {
    fn from(value: ((u32, u32), (u32, u32))) -> Self {
        Self::from(&value)
    }
}

impl From<&((u32, u32), (u32, u32))> for BoundBox {
    fn from(value: &((u32, u32), (u32, u32))) -> Self {
        Self {
            top_left: Point { x: value.0.0, y: value.0.1 },
            bottom_right: Point { x: value.1.0, y: value.1.1 },
        }
    }
}

fn main() {
    let img = ImageReader::open("receipt.jpg").unwrap().decode().unwrap();
    let img = img.to_rgb8();
    let mut img: RgbImage = image::imageops::rotate270(&img).into();

    let start_contrast = Instant::now();
    add_contrast_filter(&mut img);
    let end_contrast = start_contrast.elapsed();

    img.save("receipt_contrast.png").unwrap();

    let start_blob = Instant::now();
    let mut all_bounds = blob_find(&img);
    let end_blob = start_blob.elapsed();


    all_bounds.sort_by(|a, b| {
        if a.0.1 < b.0.1 {
            Ordering::Less
        } else if a.0.0 == b.0.0 && a.0.1 == b.0.1 {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    });

    for bound in &all_bounds {
        print_bounds(bound);
    }

    let begin_row_candidates = Instant::now();
    let (width, height) = img.dimensions();
    let mut row_candidates = Vec::new();
    let mut visited = HashSet::new();
    let mut row_box: BoundBox;
    for bound in &all_bounds {
        if visited.contains(bound) { continue; }
        row_box = ((0, bound.0.1.saturating_sub(25)), (width, bound.1.1 + 25)).into();

        let mut row = Vec::new();
        for other_bound in &all_bounds {
            if visited.contains(other_bound) { continue }
            if is_inside(&row_box, &other_bound.into()) {
                let bound_box: BoundBox = other_bound.into();
                row.push(bound_box);
                visited.insert(other_bound);
            }
        }

        row_candidates.push(row);
    }
    let end_row_candidates = begin_row_candidates.elapsed();

    println!("Rows length: {}", row_candidates.len());
    //draw_bounding_boxes_for_row(&mut img, &row_candidates[20]);

    fn distance(a: Point, b: Point) -> f32 {
        let x = (b.x - a.x).pow(2) as f32;
        let y = (b.y - a.y).pow(2) as f32;
        (x + y).sqrt()
    }

    let mut heights = Vec::new(); // TODO: remove; used for debugging
    // Post process the rows
    let post_process_start = Instant::now();
    let mut visited = HashSet::new();
    let mut rows_cleaned = Vec::new();
    for candidate in &row_candidates {
        let average_y = candidate.iter()
            .map(|b| (b.bottom_right.y - (b.height() / 2)))
            .sum::<u32>() / candidate.len() as u32;
        let average_height = candidate.iter()
            .map(|b| b.height())
            .sum::<u32>() / candidate.len() as u32;
        let mut cleaned_row = Vec::new();
        for bound_box in &all_bounds {
            let bound_box: BoundBox = bound_box.into();
            if visited.contains(&bound_box) { continue }
            let distance_to_average_y = bound_box.middle_point().y.abs_diff(average_y);
            if distance_to_average_y < average_height {
                visited.insert(bound_box.clone());
                cleaned_row.push(bound_box);
            }
        }

        if !cleaned_row.is_empty() {
            rows_cleaned.push(cleaned_row);
            heights.push(average_y);
            println!("average_y {average_y}");
        }
    }
    let post_process_end = post_process_start.elapsed();

    let mut colors = [[255, 0, 0], [0, 255, 0], [0, 0, 255]].iter().cycle();
    for i in 0..rows_cleaned.len() {
        let color = colors.next().unwrap().clone();
        let color = Rgb(color);
        draw_bounding_boxes_for_row(&mut img, &rows_cleaned[i], color);
        rows_cleaned[i].sort_by(|a, b| {
            a.top_left.x.cmp(&b.top_left.x)
        });
        let a = rows_cleaned[i][0].top_left.y;
        println!("Height: {}, top_left: {a}", heights[i]);
        // TODO: remove; for debugging
        for j in 0..width {
            img.put_pixel(j, heights[i], color.clone());
        }
    }

    //draw_bounding_boxes_for_row(&mut img, &rows_cleaned[2], Rgb([255, 0, 0]));


    let mut csv = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open("balls.csv").unwrap();

    for bound in &all_bounds {
        let ((x, y), _) = bound;
        use std::io::Write;
        writeln!(csv, "{y}").unwrap();
    }


    // Gedankendump:
    /*
        Nimm die ersten Buchstaben die irgendwie groß genug sind maybe, maybe Größe erstmal
        ignorieren.
        Als nächstes erstmal ne Linie ziehen vom ersten Buchstaben oben links nach ganz rechts,
        dannach eine Linie ziehen vom ersten Buchstaben unten links nach ganz rechts.
        Alles was dazwischen ist, ist in der Row included.
        Alles was davon geschnitten wird muss nach irgend einer heuristic auch in der Row included
        sein, das kann zum Beispiel vorkommen, wenn das Komme unter dem ersten Buchstaben liegt.
        Es sollte nie komplett drunter liegen, aber kann knapp werden.
        Die Kommata sind eigentlich die einzigen special cases.
        Ggf. wenn man eine Row fertig hat noch im Anschluss von der niedrigsten Box nochmal ne Linie
        unten ziehen, falls das nicht reicht. Ansonsten vielleicht einfach eine Linie
        durch den Mittelpunkt aller Boxen (statistischer Average oder so) und dann anhand dem
        Abstand zum Mittelpunkt der Boxen entscheiden oder so
     */


    let start_boxes = Instant::now();
    //draw_bounding_boxes(&mut img, &all_bounds);
    let end_boxes = start_boxes.elapsed();

    img.save("receipt_with_bounding_boxes.png").unwrap();

    let time_contrast = end_contrast.as_millis();
    let time_blob = end_blob.as_millis();
    let time_boxes = end_boxes.as_millis();
    let row_candidates = end_row_candidates.as_micros();
    let post_process = post_process_end.as_micros();

    println!("Contrast took {time_contrast} millis");
    println!("Blob find took {time_blob} millis");
    println!("Boxes took {time_boxes} millis");
    println!("Row candidates took {row_candidates} micros");
    println!("Post process took {post_process} micros");
}


// TODO: Refactor this for like box.is_inside(other_box) or smth
fn is_inside(row_box: &BoundBox, other_bound: &BoundBox) -> bool {
    (other_bound.top_left.x as i32 - row_box.top_left.x as i32) >= 0
        && (other_bound.top_left.y as i32 - row_box.top_left.y as i32) >= 0
        && (row_box.bottom_right.x as i32 - other_bound.bottom_right.x as i32) >= 0
        && (row_box.bottom_right.y as i32 - other_bound.bottom_right.y as i32) >= 0
}

fn draw_bounding_boxes(img: &mut RgbImage, all_bounds: &Vec<((u32, u32), (u32, u32))>) {
    let (width, height) = img.dimensions();
    let box_color = Rgb([0, 255, 0]);
    for (top_left, bottom_right) in all_bounds {
        for x in top_left.0..=bottom_right.0 {
            if x > 0 && x < width {
                img.put_pixel(x, top_left.1, box_color.clone());
                img.put_pixel(x, bottom_right.1, box_color.clone());
            }
        }
        for y in top_left.1..=bottom_right.1 {
            if y > 0 && y < height {
                img.put_pixel(top_left.0, y, box_color.clone());
                img.put_pixel(bottom_right.0, y, box_color.clone());
            }
        }
    }
}

fn draw_bounding_boxes_for_row(img: &mut RgbImage, row: &Vec<BoundBox>, color: Rgb<u8>) {
    let (width, height) = img.dimensions();
    for bounding_box in row {
        for x in bounding_box.top_left.x..=bounding_box.bottom_right.x {
            if x > 0 && x < width {
                img.put_pixel(x, bounding_box.top_left.y, color.clone());
                img.put_pixel(x, bounding_box.bottom_right.y, color.clone());
            }
        }
        for y in bounding_box.top_left.y..=bounding_box.bottom_right.y {
            if y > 0 && y < height {
                img.put_pixel(bounding_box.top_left.x, y, color.clone());
                img.put_pixel(bounding_box.bottom_right.x, y, color.clone());
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

fn blob_find(img: &RgbImage) -> Vec<((u32, u32), (u32, u32))> {
    let mut black_pixels = img
        .enumerate_pixels()
        .filter(|p| is_black(&p.2.0));
    let mut all_bounds = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    while let Some(pixel) = black_pixels.next() {
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
        all_bounds.push(bounds);
        stack.clear();
    }

    all_bounds
}

#[allow(unused)]
fn print_bounds(
    (top_left, bottom_right): &((u32, u32), (u32, u32))
) {
    let size = bounds_size(&(*top_left, *bottom_right));
    println!(
        "top left: ({}, {}), bottom right: ({}, {}), size: {}",
        top_left.0,
        top_left.1,
        bottom_right.0,
        bottom_right.1,
        size,
    )
}

fn bounds_size(
    (top_left, bottom_right): &((u32, u32), (u32, u32))
) -> u32 {
    (bottom_right.0 - top_left.0) * (bottom_right.1 - top_left.1)
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

