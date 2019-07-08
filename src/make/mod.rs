pub mod scan_sheet_elements;
pub mod scan_sheet_layout;

use crate::make::scan_sheet_layout::{HighLevelPageDescription, HighLevelField, HighLevelKind};


pub fn dummy() -> HighLevelPageDescription {
    let page_description = HighLevelPageDescription {
        document_title: String::from("test1"),
        fields: vec![
            HighLevelField {
                kind: HighLevelKind::Boolean,
                descriptor: String::from("boom"),
            },
            HighLevelField {
                kind: HighLevelKind::Boolean,
                descriptor: String::from("another"),
            },
            HighLevelField {
                kind: HighLevelKind::SevenSegmentDisplay(2),
                descriptor: String::from("another one"),
            },
            HighLevelField {
                kind: HighLevelKind::Boolean,
                descriptor: String::from("hardcore"),
            },
        ]
    };


    page_description
}
