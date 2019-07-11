use crate::make::scan_sheet_elements::{ScanSheetElements, ElementKind, BAR_WIDTH, BAR_LENGTH, FIELD_FONT_SIZE, ALIGNER_OUTER_RADIUS};
use crate::make::scan_sheet_elements::Element;
use svg;
use std::collections::{HashSet};
use crate::parse::BarsFound;

const TEXT_WIDTH_MULTIPLIER: f64 = 0.6; // characters are how many times wider than they are tall
const TEXT_GAP: f64 = 0.02; // how many pixels between the end of the text and the start of the field

const FIELD_START_X: f64 = 0.2;

pub const ALIGNER_DISTANCE_FROM_CORNER: f64 = 0.05;

const BAR_DISTANCE_THRESHOLD: f64 = 0.01;

const TITLE_X: f64 = 0.3;
const TITLE_Y: f64 = 0.05;

const DIGIT_GAP: f64 = BAR_LENGTH; // the gap between seven segment display digits
const BAR_SPACE: f64 = 0.003; //

const VERTICAL_FIELD_START: f64 = 0.3;
const VERTICAL_FIELD_SPACE: f64 = 0.1;
const BAR_VERTICAL_OFFSET: f64 = 0.03;
const SEVEN_SEGMENT_DISPLAY_OFFSET: f64 = 0.0;

// describes offset from the top left of the digit
const SEVEN_SEGMENT_BAR_OFFSETS: [(f64, f64, bool); 7] = [ // (x, y, is_horizontal)
    (BAR_WIDTH+BAR_SPACE, 0.0, true), // top
    (BAR_WIDTH+BAR_LENGTH+2.0* BAR_SPACE, BAR_WIDTH+ BAR_SPACE, false),
    (BAR_WIDTH+BAR_LENGTH+2.0* BAR_SPACE, 2.0*BAR_WIDTH+BAR_LENGTH+3.0* BAR_SPACE, false),
    (BAR_WIDTH+ BAR_SPACE, 2.0*BAR_WIDTH+2.0*BAR_LENGTH+4.0* BAR_SPACE, true), // bottom
    (0.0, 2.0*BAR_WIDTH+BAR_LENGTH+3.0* BAR_SPACE, false),
    (0.0, BAR_WIDTH+ BAR_SPACE, false),
    (BAR_WIDTH+ BAR_SPACE, BAR_WIDTH+BAR_LENGTH+2.0* BAR_SPACE, true), // middle section
];




pub struct HighLevelPageDescription {
    pub document_title: String,
    pub fields: Vec<HighLevelField>,
}

impl HighLevelPageDescription {
    pub fn layout(&self) -> PageLayout {
        let mut id_generator = BarIdGenerator::new();
        let mut layout = PageLayout::new(self.document_title.clone());

        let mut current_y = VERTICAL_FIELD_START;

        for field in self.fields.iter() {

            // this is how far the actual field must be shifted over so it doesn't overlap with the text
            let text_x_offset =  FIELD_START_X + TEXT_WIDTH_MULTIPLIER*FIELD_FONT_SIZE*field.descriptor.len() as f64 + TEXT_GAP;

            let new_entry = match field.kind {
                HighLevelKind::Boolean =>
                    LayoutEntry::Boolean(Bar::new(text_x_offset, current_y+BAR_VERTICAL_OFFSET, true, &mut id_generator)),
                HighLevelKind::SevenSegmentDisplay(digit_count) =>
                    LayoutEntry::SevenSegmentDisplay(SevenSegmentDisplay::new(text_x_offset, current_y+SEVEN_SEGMENT_DISPLAY_OFFSET, digit_count, &mut id_generator)),
            };

            layout.add_entry(new_entry, field.descriptor.clone(), FIELD_START_X, current_y);

            current_y += VERTICAL_FIELD_SPACE;
        }

        layout
    }
}

pub struct HighLevelField {
    pub kind: HighLevelKind,
    pub descriptor: String,
}

pub enum HighLevelKind {
    Boolean,
    SevenSegmentDisplay(u8), // digit count
}



pub struct PageLayout {
    document_title: String,
    fields: Vec<LayoutEntry>, // FIXME: un-public
    descriptors: Vec<(f64, f64, String)>, // x, y, text
}

#[derive(Debug)]
enum LayoutEntry { // TODO: replace this with a Field trait with elements_iter, interpret_found_target, && make not public
    Boolean(Bar),
    SevenSegmentDisplay(SevenSegmentDisplay), // this actually consists of bars
}



impl PageLayout {
    fn new(title: String) -> PageLayout {
        PageLayout { document_title: title, fields: Vec::new(), descriptors: Vec::new() }
    }

    fn add_entry(&mut self, entry: LayoutEntry, descriptor: String, x: f64, y: f64) {
        self.fields.push(entry);
        self.descriptors.push((x, y, descriptor));
    }

    pub fn to_svg(&self) -> svg::Document {
        let mut elements = ScanSheetElements::empty();
        elements.add_element(Element { // document title
            x: TITLE_X,
            y: TITLE_Y,
            kind: ElementKind::Title(self.document_title.clone()),
        });
        elements.add_element(Element { // top left
            x: ALIGNER_DISTANCE_FROM_CORNER,
            y: ALIGNER_DISTANCE_FROM_CORNER,
            kind: ElementKind::Aligner,
        });
        elements.add_element(Element { // top right
            x: 1.0-2.0*ALIGNER_OUTER_RADIUS-ALIGNER_DISTANCE_FROM_CORNER,
            y: ALIGNER_DISTANCE_FROM_CORNER,
            kind: ElementKind::Aligner,
        });
        elements.add_element(Element { // bottom right
            x: 1.0-2.0*ALIGNER_OUTER_RADIUS-ALIGNER_DISTANCE_FROM_CORNER,
            y: 1.0-2.0*ALIGNER_OUTER_RADIUS-ALIGNER_DISTANCE_FROM_CORNER,
            kind: ElementKind::Aligner,
        });
        elements.add_element(Element { // bottom left
            x: ALIGNER_DISTANCE_FROM_CORNER,
            y: 1.0-2.0*ALIGNER_OUTER_RADIUS-ALIGNER_DISTANCE_FROM_CORNER,
            kind: ElementKind::Aligner,
        });


        for &(x, y, ref text) in self.descriptors.iter() {
            elements.add_element(Element {
                x,
                y,
                kind: ElementKind::FieldDescriptor(text.clone()),
            });
        }

        for entry in self.fields.iter() {
            match *entry {
                LayoutEntry::Boolean(ref b) => elements.add_element(b.to_element()),
                LayoutEntry::SevenSegmentDisplay(ref n) =>
                    for element in n.elements_iter() {
                        elements.add_element(element);
                    }
            }
        }


        elements.to_svg()
    }

    pub fn interpret_targets(&self, targets_found: &BarsFound) -> Result<LayoutResult, LayoutResultError> {
        // avoid silently double counting bars, hard error instead
        let mut already_found = HashSet::new();

        let mut result = Vec::new();

        for entry in self.fields.iter() {
            match *entry {
                LayoutEntry::Boolean(ref bar) => {
                    let is_set = bar.is_set(targets_found, &mut already_found)?;
                    result.push(LayoutResultOption::Boolean(is_set))
                },
                LayoutEntry::SevenSegmentDisplay(ref number) => {
                    let n = number.as_number(targets_found, &mut already_found)?;
                    result.push(LayoutResultOption::Number(n));
                }
            }
        }

        Ok(LayoutResult { result })
    }
}

#[derive(Debug)]
pub enum LayoutResultError {
    BarConflictError,
    SevenSegmentError(SevenSegmentError),
}

impl From<BarConflictError> for LayoutResultError {
    fn from(_error: BarConflictError) -> LayoutResultError {
        LayoutResultError::BarConflictError
    }
}

impl From<SevenSegmentError> for LayoutResultError {
    fn from(error: SevenSegmentError) -> LayoutResultError {
        LayoutResultError::SevenSegmentError(error)
    }
}



pub struct LayoutResult {
    result: Vec<LayoutResultOption>,
}

impl LayoutResult {
    pub fn describe_results(&self, page_description: &HighLevelPageDescription) {
        for (i, (field, result)) in page_description.fields.iter().zip(self.result.iter()).enumerate() {
            println!("field #{} - '{}' has value {:?}", i, field.descriptor, result);
        }
    }
}

#[derive(Debug)]
enum LayoutResultOption {
    Boolean(bool),
    Number(u64),
}


#[derive(Debug)]
struct SevenSegmentDisplay {
    digits: Vec<SevenSegmentDigit>,
}

impl SevenSegmentDisplay {
    fn new(x: f64, y: f64, digit_count: u8, id_generator: &mut BarIdGenerator) -> SevenSegmentDisplay {
        let mut digits = Vec::new();

        let mut current_x = x;

        for _ in 0..digit_count {
            digits.push(SevenSegmentDigit::new(current_x, y, id_generator));

            // we want to space it out
            current_x += BAR_WIDTH+BAR_LENGTH+BAR_WIDTH+DIGIT_GAP;
        }

        SevenSegmentDisplay { digits }
    }

    fn elements_iter(&self) -> impl Iterator<Item=Element>+'_ {
        let mut digit_index = 0;
        let mut bar_index = 0;

        std::iter::from_fn(move || {
            if bar_index == 7 {
                digit_index += 1;
                bar_index = 0;
            }

            if digit_index == self.digits.len() {
                return None;
            }

            let bar = &self.digits[digit_index].bars[bar_index];

            bar_index += 1;

            Some(bar.to_element())
        })
    }

    fn as_number(&self, targets_found: &BarsFound, already_found: &mut HashSet<BarId>) -> Result<u64, SevenSegmentError> {
        // returns None if no segments are filled, or we have an invalid digit
        // we are looking at these digits from right to left

        let mut sum = 0;
        let mut power_of_ten = 1;

        let mut has_seen_empty = false; // there's no problem with seeing empty if the number has finished
        let mut all_are_empty = true;

        for digit in self.digits.iter().rev() {
            match digit.get_digit(targets_found, already_found) {
                Ok(_) if has_seen_empty => return Err(SevenSegmentError::Empty), // this situation looks like: 5523_23 or something
                Err(SevenSegmentError::Empty) => has_seen_empty = true, // something like _23
                Err(SevenSegmentError::Invalid(n)) => return Err(SevenSegmentError::Invalid(n)),
                Err(SevenSegmentError::BarConflict) => return Err(SevenSegmentError::BarConflict),
                Ok(d) => {
                    all_are_empty = false;
                    sum += d*power_of_ten;
                },
            }

            power_of_ten *= 10;
        }

        // make sure that we have seen at least one digit
        if all_are_empty {
            Err(SevenSegmentError::Empty)
        } else {
            Ok(sum)
        }
    }
}

#[derive(Debug)]
struct SevenSegmentDigit {
    bars: Vec<Bar>, // specified in the order mentioned (a..f) in https://en.wikipedia.org/wiki/Seven-segment_display#Displaying_letters
}

impl SevenSegmentDigit {
    fn new(x: f64, y: f64, id_generator: &mut BarIdGenerator) -> SevenSegmentDigit {
        let bars = (0..7)
            .map(|i| {
                let (x_offset, y_offset, is_horizontal) = SEVEN_SEGMENT_BAR_OFFSETS[i];
                Bar::new(x+x_offset, y+y_offset, is_horizontal, id_generator)
            })
            .collect();

        SevenSegmentDigit { bars }
    }

    fn get_digit(&self, targets_found: &BarsFound, already_found: &mut HashSet<BarId>) -> Result<u64, SevenSegmentError> {
        use SevenSegmentError::*;
        
        let mut bars_set = 0; // default value
        for (i, bar) in self.bars.iter().rev().enumerate() {
            let is_set = bar.is_set(targets_found, already_found)? as usize;
            bars_set |= is_set << i;
        }

        match bars_set {
            0b0000000 => Err(Empty),
            0b1111110 => Ok(0),
            0b0110000 => Ok(1),
            0b1101101 => Ok(2),
            0b1111001 => Ok(3),
            0b0110011 => Ok(4),
            0b1011011 => Ok(5),
            0b1011111 | 0b0011111 => Ok(6), // includes alternate 6 representation without top bar
            0b1110000 => Ok(7),
            0b1111111 => Ok(8),
            0b1111011 | 0b1110011 => Ok(9), // alternate repr of 9 without bottom bar
            _ => Err(Invalid(bars_set)),
        }
    }
}

#[derive(Debug)]
pub enum SevenSegmentError {
    Empty, // just a digit with no bars set
    Invalid(usize), // an invalid set of bars filled
    BarConflict, // a bar was found that we thought belonged to the display, but apparently it is owned by another target (bad error)
}

#[derive(Debug)]
pub struct Bar { // FIXME: unpublic
    x: f64,
    y: f64,
    is_horizontal: bool,
    id: BarId,
}

impl Bar {
    fn new(x: f64, y: f64, is_horizontal: bool, id_generator: &mut BarIdGenerator) -> Bar {
        let id = id_generator.next();

        Bar { x, y, is_horizontal, id }
    }

    fn mean_position(&self) -> (f64, f64) {
        // targets_found iterates over the mean position of the image targets, we have to get the mean position out here too
        let (base, height) = if self.is_horizontal { (BAR_LENGTH, BAR_WIDTH) } else { (BAR_WIDTH, BAR_LENGTH ) };

        (self.x + base/2.0, self.y + height/2.0)
    }

    fn is_set(&self, targets_found: &BarsFound, already_found: &mut HashSet<BarId>) -> Result<bool, BarConflictError> {

        let (mean_x, mean_y) = self.mean_position();

        for &(target_x, target_y) in targets_found.bars.iter() {
            let distance = (target_x-mean_x).hypot(target_y-mean_y);

            if distance < BAR_DISTANCE_THRESHOLD {
                if already_found.contains(&self.id) {
                    return Err(BarConflictError)
                } else {
                    return Ok(true);
                }
            } // else: this isn't even relavant to this target
        }

        Ok(false)
    }

    fn to_element(&self) -> Element {
        Element {
            x: self.x,
            y: self.y,
            kind: if self.is_horizontal { ElementKind::HorizontalBar } else { ElementKind::VerticalBar }
        }
    }
}

#[derive(Copy, Clone)]
struct BarConflictError;

impl From<BarConflictError> for SevenSegmentError {
    fn from(_error: BarConflictError) -> SevenSegmentError {
        SevenSegmentError::BarConflict
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct BarId {
    inner: u64,
}

struct BarIdGenerator {
    inner: u64,
}

impl BarIdGenerator {
    fn new() -> BarIdGenerator {
        BarIdGenerator {
            inner: 0,
        }
    }

    fn next(&mut self) -> BarId {
        let ret = BarId { inner: self.inner };
        self.inner += 1;
        ret
    }
}