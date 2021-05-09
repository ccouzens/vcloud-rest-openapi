use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PrimitiveType {
    AnyType,
    AnyUri,
    Base64Binary,
    Boolean,
    DateTime,
    Decimal,
    Double,
    Float,
    HexBinary,
    Int,
    Integer,
    Long,
    NormalizedString,
    Short,
    String,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParsePrimitiveTypeError {
    #[error("No match for input: `{0}`")]
    NoMatch(String),
}

impl FromStr for PrimitiveType {
    type Err = ParsePrimitiveTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "xs:anyType" => PrimitiveType::AnyType,
            "xs:anyURI" => PrimitiveType::AnyUri,
            "xs:base64Binary" => PrimitiveType::Base64Binary,
            "xs:boolean" => PrimitiveType::Boolean,
            "xs:dateTime" => PrimitiveType::DateTime,
            "xs:decimal" => PrimitiveType::Decimal,
            "xs:double" => PrimitiveType::Double,
            "xs:float" => PrimitiveType::Float,
            "xs:hexBinary" => PrimitiveType::HexBinary,
            "xs:int" => PrimitiveType::Int,
            "xs:integer" => PrimitiveType::Integer,
            "xs:long" => PrimitiveType::Long,
            "xs:normalizedString" => PrimitiveType::NormalizedString,
            "xs:short" => PrimitiveType::Short,
            "xs:string" => PrimitiveType::String,
            _ => return Err(ParsePrimitiveTypeError::NoMatch(s.to_owned())),
        })
    }
}

#[derive(Debug, PartialEq)]
pub(super) struct RestrictedPrimitiveType<'a> {
    pub(super) r#type: PrimitiveType,
    pub(super) pattern: &'a Option<String>,
    pub(super) enumeration: &'a Vec<String>,
    pub(super) min_inclusive: &'a Option<String>,
}

impl<'a> From<&RestrictedPrimitiveType<'a>> for openapiv3::Type {
    fn from(t: &RestrictedPrimitiveType) -> Self {
        match &t.r#type {
            PrimitiveType::AnyType
            | PrimitiveType::Decimal // verify decimal is encoded as a string
            | PrimitiveType::HexBinary
            | PrimitiveType::NormalizedString
            | PrimitiveType::String => Self::String(openapiv3::StringType {
                enumeration: t.enumeration.clone(),
                pattern: t.pattern.clone(),
                ..Default::default()
            }),
            PrimitiveType::AnyUri => Self::String(openapiv3::StringType {
                enumeration: t.enumeration.clone(),
                pattern: t.pattern.clone(),
                format: openapiv3::VariantOrUnknownOrEmpty::Unknown("uri".to_owned()),
                ..Default::default()
            }),
            PrimitiveType::Base64Binary => Self::String(openapiv3::StringType {
                enumeration: t.enumeration.clone(),
                pattern: t.pattern.clone(),
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::Byte),
                ..Default::default()
            }),
            PrimitiveType::Boolean => Self::Boolean {},
            PrimitiveType::DateTime => Self::String(openapiv3::StringType {
                enumeration: t.enumeration.clone(),
                pattern: t.pattern.clone(),
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::DateTime),
                ..Default::default()
            }),
            PrimitiveType::Double => Self::Number(openapiv3::NumberType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::NumberFormat::Double),
                minimum: t.min_inclusive.as_ref().and_then(|m| m.parse().ok()),
                enumeration: t
                    .enumeration
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect(),
                ..Default::default()
            }),
            PrimitiveType::Float => Self::Number(openapiv3::NumberType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::NumberFormat::Float),
                minimum: t.min_inclusive.as_ref().and_then(|m| m.parse().ok()),
                enumeration: t
                    .enumeration
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect(),
                ..Default::default()
            }),
            PrimitiveType::Int => Self::Integer(openapiv3::IntegerType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::IntegerFormat::Int32),
                minimum: t.min_inclusive.as_ref().and_then(|m| m.parse().ok()),
                enumeration: t
                    .enumeration
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect(),
                ..Default::default()
            }),
            PrimitiveType::Integer | PrimitiveType::Short => {
                Self::Integer(openapiv3::IntegerType {
                    minimum: t.min_inclusive.as_ref().and_then(|m| m.parse().ok()),
                    enumeration: t
                        .enumeration
                        .iter()
                        .filter_map(|s| s.parse().ok())
                        .collect(),
                    ..Default::default()
                })
            }
            PrimitiveType::Long => Self::Integer(openapiv3::IntegerType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::IntegerFormat::Int64),
                minimum: t.min_inclusive.as_ref().and_then(|m| m.parse().ok()),
                enumeration: t
                    .enumeration
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect(),
                ..Default::default()
            }),
        }
    }
}
