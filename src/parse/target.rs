use crate::parse::image::Color;
use crate::util::squareness;

const ALIGNER_COLOR: Color = Color::red();
const VERTICAL_BAR_COLOR: Color = Color::blue();
const HORIZONTAL_BAR_COLOR: Color = Color::cyan();
const DEBUG_TARGET_COLOR: Color = Color::green();

const ALIGNER_FULLNESS: f64 = 0.4;
const FULLNESS_TOLERANCE: f64 = 0.2;
const ALIGNER_SQUARE_TOLERANCE: f64 = 2.0;

const MAX_SQUARENESS: f64 = 5.0;
const NOISE_FULLNESS_THRESHOLD: f64 = 0.1; // this means less than one percent of the target is filled in
const NOISE_IMAGE_FILLED_THRESHOLD: f64 = 0.0001;

const BAR_TARGET_AREA: f64 = 0.0003;
const BAR_TARGET_AREA_TOLERANCE: f64 = 0.0002;
const BAR_SQUARENESS: f64 = 3.0;
const BAR_SQUARENESS_TOLERANCE: f64 = 2.0;

#[derive(Clone, Debug)]
pub struct Target {
    kind: TargetKind,
    pub top: f64,
    pub bottom: f64,
    pub right: f64,
    pub left: f64,
    pub fraction_of_image_filled: f64, // total fraction of the image filled by the target
    pub mean_x: f64,
    pub mean_y: f64,
}

impl Target {
    pub fn new(left: usize, right: usize, top: usize, bottom: usize, pixels_filled: usize, mean_x: usize, mean_y: usize, image_base: usize, image_height: usize) -> Option<Target> {
        let kind = TargetKind::classify(left, right, top, bottom, pixels_filled, image_base, image_height)?;

        let b = image_base as f64;
        let h = image_height as f64;

        // we need to convert our absolute coordinates into fractions
        Some(Target {
            kind,
            top: top as f64 / h,
            bottom: bottom as f64 / h,
            right: right as f64 / b,
            left: left as f64 / b,
            fraction_of_image_filled: pixels_filled as f64 / (b*h),
            mean_x: mean_x as f64 / b,
            mean_y: mean_y as f64 / h,
        })
    }

    pub fn is_bar(&self) -> bool {
        match self.kind {
            TargetKind::HorizontalBar => true,
            TargetKind::VerticalBar => true,
            _ => false,
        }
    }

    pub fn center_position(&self) -> (f64, f64) {
        fn mean(a: f64, b: f64) -> f64 {
            (a as f64 + b as f64)/2.0
        }

        let mean_x = mean(self.left, self.right);
        let mean_y = mean(self.top, self.bottom);

        (mean_x, mean_y)
    }

    pub fn is_aligner(&self) -> bool {
        self.kind == TargetKind::Aligner
    }

    pub fn get_color(&self) -> Color {
        self.kind.get_color()
    }
}


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TargetKind {
    HorizontalBar,
    VerticalBar,
    Aligner,
    Debug,
}


impl TargetKind {
    fn classify(left: usize, right: usize, top: usize, bottom: usize, pixels_filled: usize, image_base: usize, image_height: usize) -> Option<TargetKind> {
        let target_base = right - left +1;
        let target_height = bottom - top +1;

        let target_area = (target_base*target_height) as f64;
        let target_area_fraction = target_area / (image_base*image_height) as f64;

        let fraction_image_filled = pixels_filled as f64 / (image_base*image_height) as f64;

        let squareness = squareness(target_base, target_height);
        let fullness = pixels_filled as f64 / target_area;

        let is_tall = target_height > target_base;

        if fullness < NOISE_FULLNESS_THRESHOLD || fraction_image_filled < NOISE_IMAGE_FILLED_THRESHOLD || squareness > MAX_SQUARENESS {
            return None; // this target isn't a real target, its just noise
        }

        let is_aligner = squareness < ALIGNER_SQUARE_TOLERANCE && (fullness-ALIGNER_FULLNESS).abs() < FULLNESS_TOLERANCE;
        //let is_bar = (fullness-BAR_FULLNESS).abs() < FULLNESS_TOLERANCE;
        let is_bar = (target_area_fraction-BAR_TARGET_AREA).abs() < BAR_TARGET_AREA_TOLERANCE
            && (squareness-BAR_SQUARENESS).abs() < BAR_SQUARENESS_TOLERANCE;

        if is_bar {
            println!("bar fullness: {:.03}\t\tsquareness: {}\t\timage_filled: {}", fullness, squareness, fraction_image_filled);
        }

        match (is_bar, is_aligner) {
            (false, false) => {
                println!("mystery fullness: {:.03}\t\tsquareness: {}\t\ttarget_area: {}", fullness, squareness, target_area_fraction);
                Some(TargetKind::Debug)
            },
            (true, true) => None,
            (false, true) => Some(TargetKind::Aligner),
            (true, false) if is_tall => Some(TargetKind::VerticalBar),
            (true, false) => Some(TargetKind::HorizontalBar),
        }
    }

    fn get_color(self) -> Color {
        match self {
            TargetKind::HorizontalBar => HORIZONTAL_BAR_COLOR,
            TargetKind::VerticalBar => VERTICAL_BAR_COLOR,
            TargetKind::Aligner => ALIGNER_COLOR,
            TargetKind::Debug => DEBUG_TARGET_COLOR,
        }
    }
}

