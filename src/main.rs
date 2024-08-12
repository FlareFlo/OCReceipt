use std::cmp::Ordering;
use std::time::Instant;
use image::{ImageReader, Rgb, RgbImage};
use ocr_receipt::{add_contrast_filter, blob_find, draw_bounding_boxes_for_row, get_row_candidates, print_bounds, BoundBox};

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

    let mut all_bounds = all_bounds.iter().map(BoundBox::from).collect::<Vec<_>>();

    all_bounds.sort_by(|a, b| {
        if a.top_left.y < b.top_left.y {
            Ordering::Less
        } else if a.top_left.x == b.top_left.x && a.top_left.y == b.top_left.y {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    });

    for bound in &all_bounds {
        print_bounds(bound);
    }

    let begin_row_candidates = Instant::now();
    let row_candidates = get_row_candidates(&mut img, &all_bounds);
    let end_row_candidates = begin_row_candidates.elapsed();

    println!("Rows length: {}", row_candidates.len());
    //draw_bounding_boxes_for_row(&mut img, &row_candidates[20]);

    // Post process the rows
    let post_process_start = Instant::now();

    let post_process_end = post_process_start.elapsed();

    let mut colors = [
        [255, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        //[47, 116, 89],
        //[147, 68, 40],
        [0, 255, 255],
    ]
        .iter()
        .cycle();
    for i in 0..row_candidates.len() {
        let color = colors.next().unwrap().clone();
        let color = Rgb(color);
        draw_bounding_boxes_for_row(&mut img, &row_candidates[i], color);
    }

    //draw_bounding_boxes_for_row(&mut img, &rows_cleaned[2], Rgb([255, 0, 0]));

    let mut csv = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open("balls.csv")
        .unwrap();

    for bound in &all_bounds {
        let _x = bound.top_left.x;
        let y = bound.top_left.y;
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