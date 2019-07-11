use rulinalg::matrix::Matrix;
use rulinalg::vector::Vector;

use crate::parse::image::{Image, Color};
use crate::make::scan_sheet_elements::ALIGNER_OUTER_RADIUS;
use crate::make::scan_sheet_layout::ALIGNER_DISTANCE_FROM_CORNER;

pub fn perspective_transform(old: &Image, aligner_centers: &[(f64, f64)], destination_centers: &[(f64, f64)], new_image_side: usize)-> Image {
    // https://www.pyimagesearch.com/2014/08/25/4-point-opencv-getperspective-transform-example/
    // https://docs.opencv.org/2.4/modules/imgproc/doc/geometric_transformations.html?highlight=getperspectivetransform#void%20warpPerspective(InputArray%20src,%20OutputArray%20dst,%20InputArray%20M,%20Size%20dsize,%20int%20flags,%20int%20borderMode,%20const%20Scalar&%20borderValue)
    // https://github.com/opencv/opencv/blob/11b020b9f9e111bddd40bffe3b1759aa02d966f0/modules/imgproc/src/imgwarp.cpp


    //let m = get_perspective_shift_matrix(aligner_centers, old.width as f64, old.height as f64);
    //let m = get_perspective_shift_matrix(aligner_centers, width, height);
    let m = get_perspective_shift_matrix(aligner_centers, destination_centers);

    // actually produce a new image with our transformation
    //Image::from_fn(old.width, old.height, |x: usize, y: usize| {
    Image::from_fn(new_image_side, new_image_side, |x: usize, y: usize| {
        let x = x as f64;
        let y = y as f64;

        let new_x = (m[[0,0]]*x + m[[0,1]]*y + m[[0,2]]) / (m[[2,0]]*x + m[[2,1]]*y + m[[2,2]]);
        let new_y = (m[[1,0]]*x + m[[1,1]]*y + m[[1,2]]) / (m[[2,0]]*x + m[[2,1]]*y + m[[2,2]]);

        old.get_color_checked(new_x as usize, new_y as usize)
            .unwrap_or(Color::magenta()) // our debug value
    })
}

fn get_perspective_shift_matrix(aligner_centers: &[(f64, f64)], destination_centers: &[(f64, f64)]) -> Matrix<f64> {
    assert_eq!(aligner_centers.len(), 4);

    let src = aligner_centers; // rename
    let dst = destination_centers; // rename


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
