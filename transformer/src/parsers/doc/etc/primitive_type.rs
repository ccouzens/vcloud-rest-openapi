use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum PrimitiveType {
    AnyType,
    AnyUri,
    Base64Binary,
    Boolean,
    DateTime,
    Double,
    HexBinary,
    Int,
    Integer,
    Long,
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
            "xs:double" => PrimitiveType::Double,
            "xs:hexBinary" => PrimitiveType::HexBinary,
            "xs:int" => PrimitiveType::Int,
            "xs:integer" => PrimitiveType::Integer,
            "xs:long" => PrimitiveType::Long,
            "xs:string" => PrimitiveType::String,
            _ => return Err(ParsePrimitiveTypeError::NoMatch(s.to_owned())),
        })
    }
}
