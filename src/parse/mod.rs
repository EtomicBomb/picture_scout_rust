use crate::parse::image::Image;
use crate::parse::boolean_matrix::BooleanMatrix;
use crate::parse::target_mesh::TargetMesh;

mod boolean_matrix;
pub mod image;
mod perspective_transform;
mod target;
mod target_mesh;

use perspective_transform::perspective_transform;
use crate::util::map;
use crate::make::scan_sheet_elements::{DOCUMENT_BASE, DOCUMENT_HEIGHT};

const DARK_THRESHOLD: u8 = 110; // all pixels darker than this are target candidates

#[derive(Debug)]
pub struct TargetsFound {
    pub targets: Vec<(f64, f64)>,
}

impl TargetsFound {
    pub fn from_image(input_image: &Image) -> TargetsFound {
        let target_candidates = BooleanMatrix::from_image(&input_image, DARK_THRESHOLD);

        let target_mesh = TargetMesh::from_matrix(&target_candidates);
        let aligner_centers = target_mesh.get_aligner_centers();

        let transformed_image = perspective_transform(&input_image, &aligner_centers);

        let transformed_image_matrix = BooleanMatrix::from_image(&transformed_image, DARK_THRESHOLD);

        let new_target_mesh = TargetMesh::from_matrix(&transformed_image_matrix);


        // TODO: remove
        let mut debug_image = transformed_image.clone();
        new_target_mesh.add_to_image(&mut debug_image);
        debug_image.output_to_file("debug.png");

        let (width, height) = transformed_image_matrix.width_height();

        let mut targets = Vec::new();
        for target in new_target_mesh.targets.iter().filter(|t| t.is_bar()) {
            let (mean_x, mean_y) = target.mean_position();

            // this corodinates are in image space, not scan sheet coordinate space
            let sheet_x = map(mean_x, 0.0, width as f64, 0.0, DOCUMENT_BASE);
            let sheet_y = map(mean_y, 0.0, height as f64, 0.0, DOCUMENT_HEIGHT);
            targets.push((sheet_x, sheet_y));
        }

        TargetsFound { targets }
    }
}

