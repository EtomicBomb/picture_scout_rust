use std::cmp::{max, min};

use crate::target::{Target};
use crate::boolean_matrix::BooleanMatrix;
use crate::image::{Image, Color};


pub struct TargetMesh {
    inner: Vec<Target>,
}


impl TargetMesh {
    pub fn add_to_image(&self, image: &mut Image) {
        for target in self.inner.iter() {
            let color = target.get_color();

            for x in target.left..=target.right {
                for y in target.top..=target.bottom {
                    image.set_color(x, y, color);
                }
            }

            for x in target.mean_x-20..=target.mean_x+20 {
                for y in target.mean_y-20..=target.mean_y+20 {
                    image.set_color(x, y, Color::yellow());
                }
            }
        }
    }


    pub fn get_aligner_centers(&self) -> Vec<(f64, f64)> {
        let mut aligners: Vec<Target> = self.inner.iter()
            .filter(|t| t.is_aligner())
            .cloned()
            .collect();

        aligners.sort_by_key(|t| t.pixels_filled);
        aligners.truncate(4);

        assert_eq!(aligners.len(), 4);

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
        let (width, height) = target_candidates.width_height();
        let mut has_seen = BooleanMatrix::all_false(width, height);

        let mut targets = Vec::new(); // we add coordinates of the targets here

        for y in 0..height {
            for x in 0..width {
                if !target_candidates.is_set(x, y) || has_seen.is_set(x, y) { continue } // this pixel isn't a target, or we've already seen it

                if let Some(target) = flood_fill(x, y, target_candidates, &mut has_seen) {
                    targets.push(target);
                }
            }
        }

        TargetMesh { inner: targets }
    }
}


fn flood_fill(x: usize, y: usize, target_candidates: &BooleanMatrix, has_seen: &mut BooleanMatrix) -> Option<Target> {
    // returns the topmost, rightmost, bottommost, leftmost point, and the total pixels filled

    assert!(!has_seen.is_set(x, y));
    assert!(target_candidates.is_set(x, y));

    let (width, height) = target_candidates.width_height();

    let mut top = y;
    let mut bottom = y;
    let mut left = x;
    let mut right = x;

    let mut x_sum = 0;
    let mut y_sum = 0;

    let mut pixels_filled = 0;

    let mut stack = Vec::new();
    stack.push((x, y));

    while let Some((x, y)) = stack.pop() {
        if has_seen.is_set(x, y) { continue } // after this was pushed on, this pixel was colored

        has_seen.set(x, y);


        pixels_filled += 1;

        x_sum += x;
        y_sum += y;

        left = min(left, x);
        top = min(top, y);
        right = max(right, x);
        bottom = max(bottom, y);

        // process the neighbors
        for (new_x, new_y) in around(x, y, width, height) {
            if target_candidates.is_set(new_x, new_y) {
                stack.push((new_x, new_y));
            }
        }
    }

    let mean_x = x_sum / pixels_filled;
    let mean_y = y_sum / pixels_filled;

    Target::new(top, bottom, right, left, mean_x, mean_y, pixels_filled)
}

fn around(x: usize, y: usize, width: usize, height: usize) -> impl Iterator<Item=(usize, usize)> {
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

