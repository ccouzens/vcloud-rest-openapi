use super::detail_page::{DefinitionListValue, DetailPage, DetailPageFromStrError};
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryParseError {
    #[error("Detail Page Parse Error `{0}`")]
    DetailPageParseError(#[from] DetailPageFromStrError),
    #[error("Cannot find type name")]
    CannotFindTypeName,
}

pub struct Query {
    pub name: String,
}

impl TryFrom<&str> for Query {
    type Error = QueryParseError;

    fn try_from(html: &str) -> Result<Self, Self::Error> {
        Self::try_from(DetailPage::try_from(html)?)
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
        Ok(Query { name })
    }
}
