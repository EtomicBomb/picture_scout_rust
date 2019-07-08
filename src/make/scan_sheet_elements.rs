use svg::{Document};
use svg::node::element::Circle;
use svg::node::element::Rectangle;
use svg::node;
use svg::node::element;

use std::f64::consts::SQRT_2;

// we are trying to do a4 paper
pub const DOCUMENT_BASE: f64 = 100.0;
pub const DOCUMENT_HEIGHT: f64 = 100.0*SQRT_2;

pub const BAR_WIDTH: f64 = 2.0;
pub const BAR_LENGTH: f64 = 6.0;

const ALIGNER_INNER_RADIUS: f64 = 7.0;
pub const ALIGNER_OUTER_RADIUS: f64 = 10.0;

const TEMPLATE_COLOR: &'static str = "#CFE2F3"; // blue light enough that the image parser will ignore it

const TITLE_FONT_SIZE: f64 = 10.0;
pub const FIELD_FONT_SIZE: f64 = 5.0;

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
            .set("width", DOCUMENT_BASE)
            .set("height", DOCUMENT_HEIGHT);

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
                    .set("cx", self.x+ALIGNER_OUTER_RADIUS)
                    .set("cy", self.y+ALIGNER_OUTER_RADIUS)
                    .set("r", ALIGNER_OUTER_RADIUS)
                    .set("fill", "black");

                let inner = Circle::new()
                    .set("cx", self.x+ALIGNER_OUTER_RADIUS)
                    .set("cy", self.y+ALIGNER_OUTER_RADIUS)
                    .set("r", ALIGNER_INNER_RADIUS)
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
                    .set("x", self.x)
                    .set("y", self.y)
                    .set("width", w)
                    .set("height", h)
                    .set("fill", TEMPLATE_COLOR);

                doc.add(rect)
            },
            ElementKind::FieldDescriptor(ref s) | ElementKind::Title(ref s) => {
                let font_size = if self.kind.is_title() { TITLE_FONT_SIZE } else { FIELD_FONT_SIZE };

                let text = element::Text::new()
                    .add(node::Text::new(s.clone()))
                    .set("x", self.x)
                    .set("y", self.y+font_size) // for some reason text position is relative to bottom left corner
                    .set("fill", TEMPLATE_COLOR)
                    .set("font-size", font_size)
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