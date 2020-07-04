use crate::parsers::doc::etc::XML_SCHEMA_NS;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub(super) struct GroupRef {
    pub(super) reference: String,
}

#[derive(Error, Debug, PartialEq)]
pub enum GroupRefParseError {
    #[error("missing ref attribute")]
    MissingRefAttribute,
    #[error("not a group node")]
    NotGroupNode,
}

impl TryFrom<&xmltree::XMLNode> for GroupRef {
    type Error = GroupRefParseError;

    fn try_from(value: &xmltree::XMLNode) -> Result<Self, Self::Error> {
        match value {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                attributes,
                ..
            }) if namespace == XML_SCHEMA_NS && name == "group" => {
                let reference = attributes
                    .get("ref")
                    .ok_or(GroupRefParseError::MissingRefAttribute)?
                    .clone();
                Ok(GroupRef { reference })
            }
            _ => Err(GroupRefParseError::NotGroupNode),
        }
    }
}
