use regex::{Captures, Regex};
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub struct DefinitionList(pub Vec<(String, DefinitionListValue)>);

#[derive(Error, Debug)]
pub enum DefinitionListFromElError {
    #[error("CSS Selector parse error `{0}`")]
    SelectorParseError(String),
    #[error("Unexpected entry in dt, dd sequence")]
    UnexpectedEntry,
    #[error("Failed to parse entry `{0}`")]
    DefinitionListValueError(#[from] DefinitionListValueFromElError),
}

impl DefinitionList {
    pub fn find<'a>(&'a self, search_key: &'a str) -> Option<&DefinitionListValue> {
        self.filter(search_key).next()
    }

    pub fn filter<'a>(&'a self, search_key: &'a str) -> impl Iterator<Item = &DefinitionListValue> {
        self.0.iter().filter_map(
            move |(key, value)| {
                if search_key == key {
                    Some(value)
                } else {
                    None
                }
            },
        )
    }
}

impl<'a> TryFrom<&scraper::ElementRef<'a>> for DefinitionList {
    type Error = DefinitionListFromElError;

    fn try_from(el: &scraper::ElementRef) -> Result<Self, Self::Error> {
        let top_selector = scraper::Selector::parse(":scope > dt, :scope > dd")
            .map_err(|e| Self::Error::SelectorParseError(format!("{:?}", e)))?;
        Ok(el
            .select(&top_selector)
            .try_fold(
                (None, Self(Default::default())),
                |(title, mut acc), el| match (title, el.value().name()) {
                    (_, "dt") => Ok((Some(el.text().collect()), acc)),
                    (Some(title), "dd") => {
                        acc.0.push((title, DefinitionListValue::try_from(&el)?));
                        Ok((None, acc))
                    }
                    _ => Err(Self::Error::UnexpectedEntry),
                },
            )?
            .1)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DefinitionListValue {
    Text(String),
    TextAndSublist {
        preamble: Option<String>,
        sublist: Box<DefinitionList>,
    },
}

#[derive(Error, Debug)]
pub enum DefinitionListValueFromElError {
    #[error("CSS Selector parse error `{0}`")]
    SelectorParseError(String),

    #[error("Failed to parse definition list `{0}`")]
    DefinitionListParseError(#[from] Box<DefinitionListFromElError>),
}

impl<'a> TryFrom<&scraper::ElementRef<'a>> for DefinitionListValue {
    type Error = DefinitionListValueFromElError;

    fn try_from(el: &scraper::ElementRef) -> Result<Self, Self::Error> {
        let top_selector = scraper::Selector::parse(":scope > dl")
            .map_err(|e| Self::Error::SelectorParseError(format!("{:?}", e)))?;
        let text = el.inner_html().split("<dl>").next().and_then(|t| {
            let trim = t.trim();
            if trim.is_empty() {
                None
            } else {
                Some(trim.to_string())
            }
        });
        let child = el.select(&top_selector).next();
        match (child, child.map(|c| c.value().name())) {
            (Some(child), Some("dl")) => Ok(Self::TextAndSublist {
                preamble: text,
                sublist: Box::new(DefinitionList::try_from(&child).map_err(Box::new)?),
            }),
            _ => Ok(Self::Text(text.unwrap_or_default())),
        }
    }
}

impl DefinitionListValue {
    pub fn text_to_markdown(&self) -> Option<String> {
        self.as_text().map(|h| html2md::parse_html(h).trim().into())
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            DefinitionListValue::Text(html) => Some(html.as_str()),
            DefinitionListValue::TextAndSublist { preamble, .. } => preamble.as_deref(),
        }
    }

    pub fn as_sublist(&self) -> Option<&DefinitionList> {
        match self {
            DefinitionListValue::Text(_) => None,
            DefinitionListValue::TextAndSublist { sublist, .. } => Some(sublist),
        }
    }

    pub fn to_inner_text(&self) -> Option<String> {
        self.as_text().map(|t| {
            scraper::Html::parse_fragment(t)
                .root_element()
                .text()
                .collect()
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct DetailPage {
    pub title: String,
    pub h1: String,
    pub definition_list: DefinitionList,
}

#[derive(Error, Debug)]
pub enum DetailPageFromStrError {
    #[error("CSS Selector parse error `{0}`")]
    SelectorParseError(String),
    #[error("Title not found error")]
    TitleNotFound,
    #[error("H1 not found error")]
    H1NotFound,
    #[error("Dl not found error")]
    DlNotFound,
    #[error("Failed to parse dl element `{0}`")]
    DefinitionListParseError(#[from] DefinitionListFromElError),
}

impl TryFrom<&str> for DetailPage {
    type Error = DetailPageFromStrError;

    fn try_from(html: &str) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref UNINTENDED_XML: Regex = Regex::new(r"<([A-Z][a-zA-Z]+)>").unwrap();
        }
        let html = UNINTENDED_XML
            .replace_all(html, |caps: &Captures| format!("&lt;{}&gt;", &caps[1]))
            .to_string();

        lazy_static! {
            static ref UNINTENDED_DD: Regex = Regex::new(r"(\S)<dd>").unwrap();
        }
        let html = UNINTENDED_DD.replace_all(&html, |caps: &Captures| format!("{}</dd>", &caps[1]));

        let document = scraper::Html::parse_document(&html);
        let title_selector = scraper::Selector::parse("title")
            .map_err(|e| Self::Error::SelectorParseError(format!("{:?}", e)))?;
        let h1_selector = scraper::Selector::parse("h1")
            .map_err(|e| Self::Error::SelectorParseError(format!("{:?}", e)))?;

        let dl_selector = scraper::Selector::parse("body > dl")
            .map_err(|e| Self::Error::SelectorParseError(format!("{:?}", e)))?;

        let title = document
            .select(&title_selector)
            .next()
            .ok_or(Self::Error::TitleNotFound)?
            .text()
            .collect();
        let h1 = document
            .select(&h1_selector)
            .next()
            .ok_or(Self::Error::H1NotFound)?
            .text()
            .collect();

        let definition_list = DefinitionList::try_from(
            &document
                .select(&dl_selector)
                .next()
                .ok_or(Self::Error::DlNotFound)?,
        )?;

        Ok(Self {
            title,
            h1,
            definition_list,
        })
    }
}

#[test]
fn parse_operation_test() {
    let actual = DetailPage::try_from(include_str!("operations/PUT-Test.html")).unwrap();
    assert_eq!(
        actual,
        DetailPage {
            title: "VMware Cloud Director API - PUT-Test".into(),
            h1: "PUT /admin/test/{id}".into(),
            definition_list: DefinitionList(
                [
                    (
                        "Operation:".into(),
                        DefinitionListValue::Text("PUT /admin/test/{id}".into())
                    ),
                    (
                        "Description:".into(),
                        DefinitionListValue::Text("Update a test.".into())
                    ),
                    ("Since:".into(), DefinitionListValue::Text("0.9".into())),
                    (
                        "Input parameters".into(),
                        DefinitionListValue::TextAndSublist{preamble: None, sublist: Box::new(DefinitionList(
                            [
                                (
                                    "Consume media type(s):".into(),
                                    DefinitionListValue::Text(
                                        "application/vnd.vmware.admin.test+xml<br>application/vnd.vmware.admin.test+json<br>"
                                            .into()
                                    )
                                ),
                                (
                                    "Input type:".into(),
                                    DefinitionListValue::Text ("<a href=\"..//types/AdminTestType.html\">AdminTestType</a>".into())
                                )
                            ]
                            .iter()
                            .cloned()
                            .collect()
                        ))}
                    ),
                    (
                        "Query parameters".into(),
                        DefinitionListValue::TextAndSublist{preamble: None, sublist: Box::new(DefinitionList(
                            [
                                (
                                    "Parameter".into(),
                                    DefinitionListValue::Text(
                                        "force".into()
                                    )
                                ),
                                (
                                    "Documentation".into(),
                                    DefinitionListValue::Text("Documentation for force".into())
                                ),
                                (
                                    "Parameter".into(),
                                    DefinitionListValue::Text(
                                        "recursive".into()
                                    )
                                ),
                                (
                                    "Documentation".into(),
                                    DefinitionListValue::Text("".into())
                                )
                            ]
                            .iter()
                            .cloned()
                            .collect()
                        ))}
                    ),
                    (
                        "Output parameters".into(),
                        DefinitionListValue::TextAndSublist{preamble: Some("AdminTestType<br><br>\n            <p>Extended description<br><br>\n            </p>".into()), sublist: Box::new(DefinitionList(
                            [
                                (
                                    "Produce media type(s):".into(),
                                    DefinitionListValue::Text(
                                        "application/vnd.vmware.admin.testo+xml<br>application/vnd.vmware.admin.testo+json<br>"
                                            .into()
                                    )
                                ),
                                (
                                    "Output type:".into(),
                                    DefinitionListValue::Text("<a href=\"..//types/AdminTestTypeO.html\">AdminTestTypeO</a>".into())
                                )
                            ]
                            .iter()
                            .cloned()
                            .collect()
                        ))}
                    ),
                    (
                        "Examples".into(),
                        DefinitionListValue::TextAndSublist{preamble: None, sublist: Box::new(DefinitionList(Default::default()))}
                    )
                ]
                .iter()
                .cloned()
                .collect()
            )
        }
    )
}
