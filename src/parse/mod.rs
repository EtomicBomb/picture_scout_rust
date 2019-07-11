use crate::parse::image::Image;
use crate::parse::boolean_matrix::BooleanMatrix;
use crate::parse::target_mesh::TargetMesh;

mod boolean_matrix;
pub mod image;
mod target;
mod target_mesh;

use crate::make::scan_sheet_elements::{ALIGNER_OUTER_RADIUS};
use crate::make::scan_sheet_layout::ALIGNER_DISTANCE_FROM_CORNER;

const DARK_THRESHOLD: u8 = 110; // all pixels darker than this are target candidates

#[derive(Debug)]
pub struct BarsFound {
    pub bars: Vec<(f64, f64)>,
}

impl BarsFound {
    pub fn from_image(input_image: &Image) -> BarsFound {
        let target_candidates = BooleanMatrix::from_image(&input_image, DARK_THRESHOLD);

        target_candidates.as_image().output_to_file("bruh.png");

        let mesh = TargetMesh::from_matrix(&target_candidates);
        let mut debug_image = input_image.clone();
        mesh.add_to_image(&mut debug_image);
        debug_image.output_to_file("debug.png");
        let mut aligner_centers = mesh.get_aligner_centers();




        let d = ALIGNER_OUTER_RADIUS+ALIGNER_DISTANCE_FROM_CORNER;
        let mut destination_centers = [(d, d), (1.0-d, d), (1.0-d, 1.0-d), (d, 1.0-d)];


        dbg!();

        let new_image_height = 500; // why not?
        // we have to scale our transformation centers to the size of the new image
        for (x, y) in aligner_centers.iter_mut() {
            *x *= input_image.base as f64;
            *y *= input_image.height as f64;
        }
        for (x, y) in destination_centers.iter_mut() {
            *x *= new_image_height as f64;
            *y *= new_image_height as f64;
        }


        let transformed_image = input_image.perspective_transform(&aligner_centers, &destination_centers, new_image_height, new_image_height);
        transformed_image.output_to_file("transformed.png");
        let transformed_image_matrix = BooleanMatrix::from_image(&transformed_image, DARK_THRESHOLD);

        let new_target_mesh = TargetMesh::from_matrix(&transformed_image_matrix);

        let mut output_image_2 = transformed_image.clone();
        new_target_mesh.add_to_image(&mut output_image_2);
        output_image_2.output_to_file("debug2.png");

        BarsFound { bars: new_target_mesh.get_bar_centers() }
    }
}

