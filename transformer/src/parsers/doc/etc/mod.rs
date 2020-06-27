const XML_SCHEMA_NS: &str = "http://www.w3.org/2001/XMLSchema";

pub mod annotation;
pub mod complex_type;
pub mod schema;
pub mod sequence_element;

use annotation::parse_annotation;
use sequence_element::parse_sequence_element;
