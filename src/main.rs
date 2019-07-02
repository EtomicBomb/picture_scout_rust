mod image;
use image::{Image};

mod target;

mod boolean_matrix;
use boolean_matrix::BooleanMatrix;

mod target_mesh;

mod perspective_transform;
use perspective_transform::perspective_transform;

use crate::target_mesh::TargetMesh;

const SAMPLE_IMAGE_NAME: &'static str = "sample-images/beans.png"; // this is just one of my test images

const DARK_THRESHOLD: u8 = 110; // all pixels darker than this are target candidates

fn main() {
    let mut unprocessed = Image::read_from_file(SAMPLE_IMAGE_NAME);

    let target_candidates = BooleanMatrix::from_image(&unprocessed, DARK_THRESHOLD);
    target_candidates.as_image().output_to_file("monocolor.png");

    let target_mesh = TargetMesh::from_matrix(&target_candidates);
    let aligner_centers = target_mesh.get_aligner_centers();

    let mut targets_image = unprocessed.clone();
    target_mesh.add_to_image(&mut targets_image);
    targets_image.output_to_file("targets.png");

    perspective_transform(&unprocessed, &aligner_centers).output_to_file("transformed.png");
}