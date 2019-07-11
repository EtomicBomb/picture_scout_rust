use svg::{Document};
use svg::node::element::Circle;
use svg::node::element::Rectangle;
use svg::node;
use svg::node::element;

// lets do this on an 8.5 by 8.5 square cause itll fit on my paper
const DOCUMENT_HEIGHT: f64 = 8.5;

pub const BAR_WIDTH: f64 = 0.01; // fractions
pub const BAR_LENGTH: f64 = 0.03;

const ALIGNER_INNER_RADIUS: f64 = 0.05;
pub const ALIGNER_OUTER_RADIUS: f64 = 0.05*10./7.;

const TEMPLATE_COLOR: &'static str = "#CFE2F3"; // blue light enough that the image parser will ignore it

const TITLE_FONT_SIZE: f64 = 0.13;
pub const FIELD_FONT_SIZE: f64 = 0.05;

// numbers in here are expressed as percentages of the document width

pub struct ScanSheetElements {
    elements: Vec<Element>,
}

impl ScanSheetElements {
    pub fn empty() -> ScanSheetElements {
        ScanSheetElements {
            elements: Vec::new()
        }
    }

    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn to_svg(&self) -> Document {
        let mut doc = Document::new()
            .set("width", format!("{}in", DOCUMENT_HEIGHT))
            .set("height", format!("{}in", DOCUMENT_HEIGHT));

        for element in self.elements.iter() {
            doc = element.add_to_document(doc);
        }

        doc
    }
}

pub struct Element {
    pub x: f64,
    pub y: f64,
    pub kind: ElementKind,
}

impl Element {
    fn add_to_document(&self, doc: Document) -> Document {
        match self.kind {
            ElementKind::Aligner => {
                let outer = Circle::new()
                    .set("cx", percentize(self.x+ALIGNER_OUTER_RADIUS))
                    .set("cy", percentize(self.y+ALIGNER_OUTER_RADIUS))
                    .set("r", percentize(ALIGNER_OUTER_RADIUS))
                    .set("fill", "black");

                let inner = Circle::new()
                    .set("cx", percentize(self.x+ALIGNER_OUTER_RADIUS))
                    .set("cy", percentize(self.y+ALIGNER_OUTER_RADIUS))
                    .set("r", percentize(ALIGNER_INNER_RADIUS))
                    .set("fill", "white");

                doc.add(outer).add(inner)
            },
            ElementKind::HorizontalBar | ElementKind::VerticalBar => {
                let (w, h) = if let ElementKind::VerticalBar = self.kind {
                    (BAR_WIDTH, BAR_LENGTH)
                } else {
                    (BAR_LENGTH, BAR_WIDTH)
                };

                let rect = Rectangle::new()
                    .set("x", percentize(self.x))
                    .set("y", percentize(self.y))
                    .set("width", percentize(w))
                    .set("height", percentize(h))
                    .set("fill", TEMPLATE_COLOR);

                doc.add(rect)
            },
            ElementKind::FieldDescriptor(ref s) | ElementKind::Title(ref s) => {
                let font_size = if self.kind.is_title() { TITLE_FONT_SIZE } else { FIELD_FONT_SIZE };

                let text = element::Text::new()
                    .add(node::Text::new(s.clone()))
                    .set("x", percentize(self.x))
                    .set("y", percentize(self.y+font_size)) // for some reason text position is relative to bottom left corner
                    .set("fill", TEMPLATE_COLOR)
                    .set("font-size", to_points(font_size))
                    .set("font-family", "monospace");

                doc.add(text)
            },
        }
    }
}

#[derive(PartialEq)]
pub enum ElementKind {
    Aligner,
    HorizontalBar,
    VerticalBar,
    FieldDescriptor(String),
    Title(String),
}

impl ElementKind {
    fn is_title(&self) -> bool {
        match *self {
            ElementKind::Title(_) => true,
            _ => false,
        }
    }
}

// n is a fraction between 0 and 1
fn percentize(n: f64) -> String {
    format!("{:.2}%", 100.0*n) // the text of our svg is less readable if we don't truncate
}

fn to_points(fraction: f64) -> String {
    // the input is a height fraction from 0 to 1, and the output is the size of text in points
    // 72 to points is one inch, and our square is 8.5 inches

    let height = 8.5*fraction; // in inches
    format!("{}", 72.0*height)
}
