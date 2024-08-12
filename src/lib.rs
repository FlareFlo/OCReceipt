use image::{Rgb, RgbImage};
use std::array;
use std::collections::HashSet;
use std::ops::Index;
use std::sync::LazyLock;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Point {
	pub x: u32,
	pub y: u32,
}
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct BoundBox {
	pub top_left: Point,
	pub bottom_right: Point,
}

impl BoundBox {
	pub fn height(&self) -> u32 {
		self.bottom_right.y - self.top_left.y
	}

	pub fn width(&self) -> u32 {
		self.bottom_right.y - self.top_left.y
	}

	pub fn middle_point(&self) -> Point {
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
			top_left: Point {
				x: value.0 .0,
				y: value.0 .1,
			},
			bottom_right: Point {
				x: value.1 .0,
				y: value.1 .1,
			},
		}
	}
}



// TODO: Version 1 of row algorithm, would need a post-process step
pub fn get_row_candidates<'a>(
	img: &mut RgbImage,
	all_bounds: &'a Vec<BoundBox>,
) -> Vec<Vec<&'a BoundBox>> {
	let (width, height) = img.dimensions();
	let mut row_candidates = Vec::new();
	let mut visited = HashSet::new();
	let mut row_box: BoundBox;
	for bound in all_bounds {
		if visited.contains(bound) {
			continue;
		}
		row_box = (
			(0, bound.top_left.y.saturating_sub(25)),
			(width, bound.bottom_right.y + 25),
		)
			.into();

		let mut row = Vec::new();
		for other_bound in all_bounds {
			if visited.contains(other_bound) {
				continue;
			}
			if is_inside(&row_box, other_bound) {
				row.push(other_bound);
				visited.insert(other_bound);
			}
		}

		row_candidates.push(row);
	}

	row_candidates
}

// TODO: also refactor this?
pub fn distance(a: Point, b: Point) -> f32 {
	let x = (b.x - a.x).pow(2) as f32;
	let y = (b.y - a.y).pow(2) as f32;
	(x + y).sqrt()
}

// TODO: Refactor this for like box.is_inside(other_box) or smth
pub fn is_inside(row_box: &BoundBox, other_bound: &BoundBox) -> bool {
	(other_bound.top_left.x as i32 - row_box.top_left.x as i32) >= 0
		&& (other_bound.top_left.y as i32 - row_box.top_left.y as i32) >= 0
		&& (row_box.bottom_right.x as i32 - other_bound.bottom_right.x as i32) >= 0
		&& (row_box.bottom_right.y as i32 - other_bound.bottom_right.y as i32) >= 0
}

pub fn draw_bounding_boxes(img: &mut RgbImage, all_bounds: &Vec<BoundBox>) {
	let (width, height) = img.dimensions();
	let all_bounds = all_bounds.iter().map(|b| (b.top_left, b.bottom_right));
	let box_color = Rgb([0, 255, 0]);
	for (top_left, bottom_right) in all_bounds {
		for x in top_left.x..=bottom_right.x {
			if x > 0 && x < width {
				img.put_pixel(x, top_left.y, box_color.clone());
				img.put_pixel(x, bottom_right.y, box_color.clone());
			}
		}
		for y in top_left.y..=bottom_right.y {
			if y > 0 && y < height {
				img.put_pixel(top_left.x, y, box_color.clone());
				img.put_pixel(bottom_right.x, y, box_color.clone());
			}
		}
	}
}

pub fn draw_bounding_boxes_for_row(img: &mut RgbImage, row: &Vec<&BoundBox>, color: Rgb<u8>) {
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

pub fn add_contrast_filter(img: &mut RgbImage) {
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

pub fn blob_find(img: &RgbImage) -> Vec<((u32, u32), (u32, u32))> {
	let mut black_pixels = img.enumerate_pixels().filter(|p| is_black(&p.2 .0));
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
					stack.push((curr_x, curr_y - 1));
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
pub fn print_bounds(bounds: &BoundBox) {
	let (top_left, bottom_right) = (bounds.top_left, bounds.bottom_right);
	let size = bounds_size(&bounds);
	println!(
		"top left: ({}, {}), bottom right: ({}, {}), size: {}",
		top_left.x, top_left.y, bottom_right.x, bottom_right.y, size,
	)
}

pub fn bounds_size(bounds: &BoundBox) -> u32 {
	let (top_left, bottom_right) = (bounds.top_left, bounds.bottom_right);
	(bottom_right.x - top_left.x) * (bottom_right.y - top_left.y)
}

pub fn update_bounds((top_left, bottom_right): &mut ((u32, u32), (u32, u32)), (x, y): (u32, u32)) {
	if x < top_left.0 {
		top_left.0 = x
	}
	if y < top_left.1 {
		top_left.1 = y
	}
	if x > bottom_right.0 {
		bottom_right.0 = x
	}
	if y > bottom_right.1 {
		bottom_right.1 = y
	}
}

pub fn new_bounds() -> ((u32, u32), (u32, u32)) {
	((u32::MAX, u32::MAX), (u32::MIN, u32::MIN))
}

struct Square {
	x: u32,
	y: u32,
	pixels: [[[u8; 3]; 3]; 3],
}

impl Square {
	pub fn xy(&self) -> (u32, u32) {
		(self.x, self.y)
	}

	pub fn x(&self) -> u32 {
		self.x
	}

	pub fn y(&self) -> u32 {
		self.y
	}

	pub fn contains_black(&self) -> bool {
		self.count_blacks() > 0
	}

	pub fn count_blacks(&self) -> u8 {
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
pub fn get_square(img: &RgbImage, x: u32, y: u32) -> Square {
	let pixels = [
		[
			img.get_pixel(x, y).0,
			img.get_pixel(x + 1, y).0,
			img.get_pixel(x + 2, y).0,
		],
		[
			img.get_pixel(x, y + 1).0,
			img.get_pixel(x + 1, y + 1).0,
			img.get_pixel(x + 2, y + 1).0,
		],
		[
			img.get_pixel(x, y + 2).0,
			img.get_pixel(x + 1, y + 2).0,
			img.get_pixel(x + 2, y + 2).0,
		],
	];

	Square { x, y, pixels }
}

pub fn is_black(rgb: &[u8; 3]) -> bool {
	(rgb[0] as u16 + rgb[1] as u16 + rgb[2] as u16) == 0
}
