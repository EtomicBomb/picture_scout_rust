use std::cmp::{max, min, Reverse};

use crate::parse::boolean_matrix::BooleanMatrix;
use crate::parse::image::{Image, Color};
use crate::parse::target::Target;

use ordered_float::OrderedFloat;


#[derive(Debug)]
pub struct TargetMesh {
    pub targets: Vec<Target>, // coordinates stored are between 0 and 1 as height percentage
}


impl TargetMesh {
    pub fn add_to_image(&self, image: &mut Image) {
        let target_center_square_size = image.height / 200;

        for target in self.targets.iter() {
            let color = target.get_color();

            let top = (target.top * image.height as f64) as usize;
            let bottom = (target.bottom * image.height as f64) as usize;
            let left = (target.left * image.base as f64) as usize;
            let right = (target.right * image.base as f64) as usize;
            let mean_x = (target.mean_x * image.base as f64) as usize;
            let mean_y = (target.mean_y * image.height as f64) as usize;

            for y in top..=bottom {
                for x in left..=right {
                    // we need to scale this up to the size of the image
                    image.set_color(x, y, color);
                }
            }

            for y in mean_y-target_center_square_size..=mean_y+target_center_square_size {
                for x in mean_x-target_center_square_size..=mean_x+target_center_square_size {
                    if x < image.base && y < image.height {
                        image.set_color(x, y, Color::yellow());
                    }
                }
            }
        }
    }

    pub fn get_bar_centers(&self) -> Vec<(f64, f64)> {
        self.targets.iter()
            .filter(|t| t.is_bar())
            .map(|t| t.center_position())
            .collect()
    }

    pub fn get_aligner_centers(&self) -> Vec<(f64, f64)> {
        let mut aligners: Vec<Target> = self.targets.iter()
            .filter(|t| t.is_aligner())
            .cloned()
            .collect();

        aligners.sort_by_key(|t| Reverse(OrderedFloat(t.fraction_of_image_filled)));
        aligners.truncate(4);

        assert_eq!(aligners.len(), 4, "Fewer than four aligners were found");

        let mut centers: Vec<(f64, f64)>  = aligners.into_iter()
            .map(|t| (t.mean_x as f64, t.mean_y as f64))
            .collect();

        // we want to
        let mut sorted_centers = Vec::with_capacity(4);


        sorted_centers.push(remove_max_by(&mut centers, |&(x0, y0), &(x1, y1)| x0+y0 < x1+y1)); // top left
        sorted_centers.push(remove_max_by(&mut centers, |&(x0, y0), &(x1, y1)| x0-y0 > x1-y1)); // top right
        sorted_centers.push(remove_max_by(&mut centers, |&(x0, y0), &(x1, y1)| x0+y0 > x1+y1)); // bottom right
        sorted_centers.push(remove_max_by(&mut centers, |&(x0, y0), &(x1, y1)| x0-y0 < x1-y1)); // bottom left

        sorted_centers
    }

    pub fn from_matrix(target_candidates: &BooleanMatrix) -> TargetMesh {
        // lets iterate through all of the `dark` pixels
        let (base, height) = target_candidates.base_height();

        let mut has_seen = BooleanMatrix::all_false(height, height);

        let mut targets = Vec::new(); // we add coordinates of the targets here

        for y in 0..height {
            for x in 0..base {
                if !target_candidates.is_set(x, y) || has_seen.is_set(x, y) { continue } // this pixel isn't a target, or we've already seen it

                if let Some(target) = flood_fill(x, y, target_candidates, &mut has_seen) {
                    targets.push(target);
                }
            }
        }

        TargetMesh { targets }
    }
}


fn flood_fill(x: usize, y: usize, target_candidates: &BooleanMatrix, has_seen: &mut BooleanMatrix) -> Option<Target> {
    // returns the topmost, rightmost, bottommost, leftmost point, and the total pixels filled

    let (image_base, image_height) = target_candidates.base_height();

    assert!(!has_seen.is_set(x, y));
    assert!(target_candidates.is_set(x, y));

//    let mut top = y as f64 / image_height as f64;
//    let mut bottom = y as f64 / image_height as f64;
//    let mut left = x as f64 / image_base as f64;
//    let mut right = x as f64 / image_base as f64;
    let mut top = y;
    let mut bottom = y;
    let mut left = x;
    let mut right = x;

//    let mut x_sum = 0.0;
//    let mut y_sum = 0.0;
    let mut x_sum = 0;
    let mut y_sum = 0;

    let mut pixels_filled = 0;

    let mut stack = Vec::new();
    stack.push((x, y));

    while let Some((x, y)) = stack.pop() {
        if has_seen.is_set(x, y) { continue } // after this was pushed on, this pixel was colored
        has_seen.set(x, y);


//        let x_frac = x as f64 / image_base as f64;
//        let y_frac = y as f64 / image_height as f64;


        pixels_filled += 1;
//
//        x_sum += x_frac;
//        y_sum += y_frac;
//
//        left = min(left, x_frac);
//        top = min(top, y_frac);
//        right = max(right, x_frac);
//        bottom = max(bottom, y_frac);
        x_sum += x;
        y_sum += y;

        left = min(left, x);
        top = min(top, y);
        right = max(right, x);
        bottom = max(bottom, y);

        // process the neighbors
        for (new_x, new_y) in neighbors(x, y, image_base, image_height) {
            if target_candidates.is_set(new_x, new_y) {
                stack.push((new_x, new_y));
            }
        }
    }

//    let mean_x = x_sum / pixels_filled as f64;
//    let mean_y = y_sum / pixels_filled as f64;


    // Target::new shouldn't care about the image_base or image_height of our image. That's why we want to convert all of our numbers
    // to fractions between 0 and 1

//    let (b, h) = (image_base as f64, image_height as f64); // just to be clear, these are the image_base and image_height of the image
//
//    // this is the 'center of mass' of our target
//    let mean_x = x_sum / pixels_filled;
//    let mean_y = y_sum / pixels_filled;
//
//    let mut fraction_of_image_filled = pixels_filled as f64 / (b*h);
//
//    // why add one: because otherwise if there is a target only one pixel, then right-left == 0, and so image_base would be reported as 0
//    let target_base_unmapped = right-left+1; // we add one, because
//    let target_height_unmapped = bottom-left+1;
//    let squareness = squareness(target_base_unmapped, target_height_unmapped);
//    let fullness = pixels_filled as f64 / (target_base_unmapped*target_height_unmapped) as f64;
//
//    Target::new(
//        top as f64 / h,
//        bottom as f64 / h,
//        right as f64 / b,
//        left as f64 / b,
//        mean_x as f64 / b,
//        mean_y as f64 / h,
//        fraction_of_image_filled,
//        squareness,
//        fullness,
//    )
    let mean_x = x_sum / pixels_filled;
    let mean_y = y_sum / pixels_filled;

    Target::new(left, right, top, bottom, pixels_filled, mean_x, mean_y, image_base, image_height)
}

fn neighbors(x: usize, y: usize, width: usize, height: usize) -> impl Iterator<Item=(usize, usize)> {
    // returns an iterator over the 4 pixels surrounding (x,y) respecting the edges
    let mut ret = Vec::with_capacity(4);

    if x > 0 {
        ret.push((x-1, y));
    }
    if y > 0 {
        ret.push((x, y-1));
    }
    if x < width-1 {
        ret.push((x+1, y));
    }
    if y < height-1 {
        ret.push((x, y+1));
    }

    ret.into_iter()
}


fn remove_max_by<T>(vec: &mut Vec<T>, greater_than: impl Fn(&T, &T) -> bool) -> T {
    let mut max_index = 0;
    let mut max = &vec[0];
    for (i, k) in vec.iter().enumerate().skip(1) {
        if greater_than(k, max) {
            max = k;
            max_index = i;
        }
    }

    return vec.remove(max_index);
}

