use crate::image::Color;

const ALIGNER_COLOR: Color = Color::red();
const VERTICAL_BAR_COLOR: Color = Color::blue();
const HORIZONTAL_BAR_COLOR: Color = Color::magenta();
const DEBUG_TARGET_COLOR: Color = Color::green();

const ALIGNER_FULLNESS: f64 = 0.41;
const BAR_FULLNESS: f64 = 0.9;
const FULLNESS_TOLERANCE: f64 = 0.2;
const TARGET_MIN_AREA: usize = 1000;
const TARGET_MAX_AREA: usize = 100_000;


#[derive(Clone)]
pub struct Target {
    kind: TargetKind,
    pub top: usize,
    pub bottom: usize,
    pub right: usize,
    pub left: usize,
    pub pixels_filled: usize,
    pub mean_x: usize,
    pub mean_y: usize,
}

impl Target {
    pub fn new(top: usize, bottom: usize, right: usize, left: usize, mean_x: usize, mean_y: usize, pixels_filled: usize) -> Option<Target> {
        Some(Target {
            top,
            bottom,
            right,
            left,
            pixels_filled,
            mean_x,
            mean_y,
            kind: TargetKind::classify(top, bottom, right, left, pixels_filled)?
        })
    }

    pub fn is_aligner(&self) -> bool {
        self.kind == TargetKind::Aligner
    }

    pub fn get_color(&self) -> Color {
        self.kind.get_color()
    }
}


#[derive(Clone, Copy, PartialEq)]
pub enum TargetKind {
    HorizontalBar,
    VerticalBar,
    Aligner,
    Debug,
}


impl TargetKind {
    fn classify(top: usize, bottom: usize, right: usize, left: usize, pixels_filled: usize) -> Option<TargetKind> {
        if pixels_filled < TARGET_MIN_AREA || pixels_filled > TARGET_MAX_AREA {
            return None;
        }


        let width = (right - left + 1) as f64;
        let height = (bottom - top + 1) as f64;
        let fullness = pixels_filled as f64 / (width * height);


        let is_bar = (BAR_FULLNESS - fullness).abs() < FULLNESS_TOLERANCE;
        let is_aligner = (ALIGNER_FULLNESS - fullness).abs() < FULLNESS_TOLERANCE;

        if is_aligner {
            println!("alligner area: {} fullness: {}", pixels_filled, fullness);
        }

        if is_bar {
            println!("bar area {} fullness: {}", pixels_filled, fullness);
        }

        match (is_bar, is_aligner) {
            (false, false) => {
                println!("mystery fullness {}", fullness);
                Some(TargetKind::Debug)
            },
            (true, true) => None,
            (false, true) => Some(TargetKind::Aligner),
            (true, false) if height > width => Some(TargetKind::VerticalBar),
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

