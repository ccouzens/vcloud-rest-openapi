const XML_SCHEMA_NS: &str = "http://www.w3.org/2001/XMLSchema";

pub mod annotation;
pub mod gen_xsd;
pub mod sequence_element;

use annotation::parse_annotation;
