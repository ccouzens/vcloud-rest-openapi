use super::detail_page::{DefinitionListValue, DetailPage, DetailPageFromStrError};
use indexmap::IndexSet;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Type {
    pub r#type: String,
    pub name: String,
    pub elements: IndexSet<String>,
    pub namespace: String,
    pub description: String,
    pub media_types: Option<IndexSet<String>>,
    pub extends: String,
}

#[derive(Error, Debug)]
pub enum TypeParseError {
    #[error("Detail Page Parse Error `{0}`")]
    DetailPageParseError(#[from] DetailPageFromStrError),
    #[error("Cannot find elements")]
    CannotFindElementsError,
}

fn html_to_mimes(html: &str) -> impl Iterator<Item = String> + '_ {
    html.split("<br>")
        .filter(|&t| !(t.is_empty() || t == "None"))
        .map(String::from)
}

impl TryFrom<&str> for Type {
    type Error = TypeParseError;

    fn try_from(html: &str) -> Result<Self, Self::Error> {
        Self::try_from(DetailPage::try_from(html)?)
    }
}

impl TryFrom<DetailPage> for Type {
    type Error = TypeParseError;

    fn try_from(p: DetailPage) -> Result<Self, Self::Error> {
        let r#type = p.h1;
        let elements = p
            .definition_list
            .find("Element:")
            .or(p.definition_list.find("Elements:"))
            .and_then(DefinitionListValue::as_text)
            .map(|v| v.split(',').map(|e| e.trim().into()).collect())
            .ok_or(Self::Error::CannotFindElementsError)?;
        let namespace = p
            .definition_list
            .find("Namespace:")
            .and_then(DefinitionListValue::to_inner_text)
            .unwrap_or_default();
        let description = p
            .definition_list
            .find("Description:")
            .and_then(DefinitionListValue::as_text)
            .unwrap_or_default()
            .to_string();
        let media_types = p
            .definition_list
            .find("Media type(s):")
            .and_then(DefinitionListValue::as_text)
            .map(|t| html_to_mimes(t).collect());
        let extends = p
            .definition_list
            .find("Extends:")
            .and_then(DefinitionListValue::to_inner_text)
            .unwrap_or_default();
        let name = match namespace.as_str() {
            "http://www.vmware.com/vcloud/extension/v1.5" => format!("vcloud-ext_{}", r#type),
            "http://www.vmware.com/vcloud/versions" => format!("versioning_{}", r#type),
            "http://www.vmware.com/vcloud/v1.5" => format!("vcloud_{}", r#type),
            "http://schemas.dmtf.org/ovf/envelope/1" => format!("ovf_{}", r#type),
            _ => r#type.to_string(),
        };
        Ok(Self {
            r#type,
            name,
            elements,
            namespace,
            description,
            media_types,
            extends,
        })
    }
}
