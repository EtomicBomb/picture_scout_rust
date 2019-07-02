use std::cmp::{max, min};


use rulinalg::matrix::Matrix;
use rulinalg::vector::Vector;

mod image;
use image::{Image, Color};

mod target;
use target::{Target};

mod boolean_matrix;

use std::time::Instant;

const SAMPLE_IMAGE_NAME: &'static str = "sample-images/test.png";


const DARK_THRESHOLD: u8 = 110; // all pixels darker than this are target candidates

fn main() {
    let mut unprocessed = Image::read_from_file(SAMPLE_IMAGE_NAME);

    let target_candidates = BooleanMatrix::from_image(&unprocessed, DARK_THRESHOLD);

    target_candidates.as_image().output_to_file("monocolor.png");

    let targets = extract_targets(&target_candidates);

    add_targets_to_image(&mut unprocessed, &targets);
    unprocessed.output_to_file("targets.png");


    let aligner_centers = get_aligner_centers(&targets);

    let start = Instant::now();
    perspective_transform(&unprocessed, &aligner_centers).output_to_file("transformed.png");
    dbg!(start.elapsed());
}

fn perspective_transform(old: &Image, aligner_centers: &[(f64, f64)])-> Image {
    // https://www.pyimagesearch.com/2014/08/25/4-point-opencv-getperspective-transform-example/
    // https://docs.opencv.org/2.4/modules/imgproc/doc/geometric_transformations.html?highlight=getperspectivetransform#void%20warpPerspective(InputArray%20src,%20OutputArray%20dst,%20InputArray%20M,%20Size%20dsize,%20int%20flags,%20int%20borderMode,%20const%20Scalar&%20borderValue)
    // https://github.com/opencv/opencv/blob/11b020b9f9e111bddd40bffe3b1759aa02d966f0/modules/imgproc/src/imgwarp.cpp

    let m = get_perspective_shift_matrix(aligner_centers, old.width as f64, old.height as f64);

    // actually produce a new image with our transformation
    Image::from_fn(old.width, old.height, |x: usize, y: usize| {
        let x = x as f64;
        let y = y as f64;

        let new_x = (m[[0,0]]*x + m[[0,1]]*y + m[[0,2]])/ (m[[2,0]]*x + m[[2,1]]*y + m[[2,2]]);
        let new_y = (m[[1,0]]*x + m[[1,1]]*y + m[[1,2]])/ (m[[2,0]]*x + m[[2,1]]*y + m[[2,2]]);

        old.get_color_checked(new_x as usize, new_y as usize)
            .unwrap_or(Color::magenta()) // our debug value
    })
}

fn get_perspective_shift_matrix(aligner_centers: &[(f64, f64)], width: f64, height: f64) -> Matrix<f64> {
    assert_eq!(aligner_centers.len(), 4);

    let src = aligner_centers; // rename
    let dst = [(0.0, 0.0), (width, 0.0), (width, height), (0.0, height)];

    let mut a = Matrix::zeros(8, 8);
    let mut b = Vector::zeros(8);
    for i in 0..4 {
        let t = src[i].0;
        a[[i+4,3]] = t;
        a[[i,0]] = t;

        let t = src[i].1;
        a[[i+4,4]] = t;
        a[[i,1]] = t;

        a[[i+4,5]] = 1.0;
        a[[i,2]] = 1.0;

        a[[i,6]] = -src[i].0*dst[i].0;
        a[[i,7]] = -src[i].1*dst[i].0;
        a[[i+4,6]] = -src[i].0*dst[i].1;
        a[[i+4,7]] = -src[i].1*dst[i].1;

        b[i] = dst[i].0;
        b[i+4] = dst[i].1;
    }

    // calculate the transformation matrix m such that a*m = b
    let cs = a.solve(b).unwrap();
    let mut data = cs.into_vec();
    data.push(1.0);
    Matrix::<f64>::new(3, 3, data).inverse().unwrap()
}


fn add_targets_to_image(image: &mut Image, targets: &[Target]) {
    for target in targets {
        let color = target.kind.get_color();

        for x in target.left..=target.right {
            for y in target.top..=target.bottom {
                image.set_color(x, y, color);
            }
        }

        for x in target.mean_x-20..=target.mean_x+20 {
            for y in target.mean_y-20..=target.mean_y+20 {
                image.set_color(x, y, TARGET_CENTER_COLOR);
            }
        }
    }
}


fn remove_max_by<T>(vec: &mut Vec<T>, prefer_first: impl Fn(&T, &T) -> bool) -> T {
    let mut max_index = 0;
    let mut max = &vec[0];
    for (i, k) in vec.iter().enumerate().skip(1) {
        if prefer_first(k, max) {
            max = k;
            max_index = i;
        }
    }

    return vec.remove(max_index);
}



fn get_aligner_centers(targets: &[Target]) -> Vec<(f64, f64)> {
    let mut aligners: Vec<Target> = targets.iter()
        .filter(|t| t.kind == TargetKind::Aligner)
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


fn extract_targets(is_target_candidate: &BooleanMatrix) -> Vec<Target> {
    // lets iterate through all of the `dark` pixels
    let width = is_target_candidate.width;
    let height = is_target_candidate.height;

    let mut has_seen = BooleanMatrix::all_false(width, height);

    let mut targets = Vec::new(); // we add coordinates of the targets here

    for y in 0..height {
        for x in 0..width {
            if !is_target_candidate.is_set(x, y) || has_seen.is_set(x, y) { continue } // this pixel isn't a target, or we've already seen it

            if let Some(target) = flood_fill(x, y, is_target_candidate, &mut has_seen) {
                targets.push(target);
            }
        }
    }

    targets
}



fn around(x: usize, y: usize, width: usize, height: usize) -> impl Iterator<Item=(usize, usize)> {
    // TODO: this code is ugly with all of these casts. Figure out a better way to do this

    let x = x as i32;
    let y = y as i32;
    let w = width as i32;
    let h = height as i32;


    let d = vec![
        (x, y-1), // top
        (x, y+1), // bottom
        (x-1, y), // right
        (x+1, y), // left
    ];

    d.into_iter()
        .filter(move |&(x, y)| {
            x >= 0 && x < w && y >= 0 && y < h
        })
        .map(move |(x, y)| (x as usize, y as usize))
}

fn flood_fill(x: usize, y: usize, target_candidates: &BooleanMatrix, has_seen: &mut BooleanMatrix) -> Option<Target> {
    // returns the topmost, rightmost, bottommost, leftmost point, and the total pixels filled

    assert!(!has_seen.is_set(x, y));
    assert!(target_candidates.is_set(x, y));

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
        for (new_x, new_y) in around(x, y, target_candidates.width, target_candidates.height) {
            if target_candidates.is_set(new_x, new_y) {
                stack.push((new_x, new_y));
            }
        }
    }
    
    let mean_x = x_sum / pixels_filled;
    let mean_y = y_sum / pixels_filled;

    Target::new(top, bottom, right, left, mean_x, mean_y, pixels_filled)
}



