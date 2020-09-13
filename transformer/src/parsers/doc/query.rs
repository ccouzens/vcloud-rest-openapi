use super::detail_page::{DefinitionListValue, DetailPage, DetailPageFromStrError};
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryParseError {
    #[error("Detail Page Parse Error `{0}`")]
    DetailPageParseError(#[from] DetailPageFromStrError),
    #[error("Cannot find type name")]
    CannotFindTypeName,
    #[error("Cannot find description")]
    CannotFindDescription,
    #[error("Cannot find record result")]
    CannotFindRecordResult,
}

#[derive(Debug)]
pub struct Query {
    pub name: String,
    description: String,
    record_result: String,
}

impl TryFrom<&str> for Query {
    type Error = QueryParseError;

    fn try_from(html: &str) -> Result<Self, Self::Error> {
        Ok(Self::try_from(DetailPage::try_from(html)?)?)
    }
}

impl<'a> TryFrom<DetailPage> for Query {
    type Error = QueryParseError;

    fn try_from(p: DetailPage) -> Result<Self, Self::Error> {
        let name = p
            .definition_list
            .find("Type Name:")
            .and_then(DefinitionListValue::as_text)
            .ok_or(Self::Error::CannotFindTypeName)?
            .to_string();
        let description = p
            .definition_list
            .find("Description:")
            .and_then(DefinitionListValue::text_to_markdown)
            .ok_or(Self::Error::CannotFindDescription)?;
        let record_result = p
            .definition_list
            .find("Record Result:")
            .and_then(DefinitionListValue::as_text)
            .and_then(|t| t.strip_suffix("</a>)"))
            .and_then(|t| t.split(">").nth(1))
            .ok_or(Self::Error::CannotFindRecordResult)?
            .to_string();
        Ok(Query {
            name,
            description,
            record_result,
        })
    }
}
