use crate::parsers::doc::etc::r#type::Type;
use crate::parsers::doc::etc::XML_SCHEMA_NS;
#[cfg(test)]
use serde_json::json;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Schema {
  includes: Vec<String>,
  types: Vec<Type>,
}

#[derive(Error, Debug, PartialEq)]
pub enum SchemaParseError {
  #[error("not a schema node")]
  NotSchemaNode,
}

#[derive(Error, Debug)]
pub enum SchemaFromBytesError {
  #[error("XML parse error")]
  XmlParse(#[from] xmltree::ParseError),
  #[error("XSD parse error")]
  XsdParse(#[from] SchemaParseError),
}

impl TryFrom<&[u8]> for Schema {
  type Error = SchemaFromBytesError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    Ok(Self::try_from(&xmltree::XMLNode::Element(
      xmltree::Element::parse(value)?,
    ))?)
  }
}

impl TryFrom<&xmltree::XMLNode> for Schema {
  type Error = SchemaParseError;

  fn try_from(value: &xmltree::XMLNode) -> Result<Self, Self::Error> {
    match value {
      xmltree::XMLNode::Element(xmltree::Element {
        namespace: Some(namespace),
        name,
        children,
        ..
      }) if namespace == XML_SCHEMA_NS && name == "schema" => Ok(Schema {
        types: children.iter().flat_map(Type::try_from).collect(),
        includes: children
          .iter()
          .filter_map(|child| match child {
            xmltree::XMLNode::Element(xmltree::Element {
              namespace: Some(namespace),
              name,
              attributes,
              ..
            }) if namespace == XML_SCHEMA_NS && name == "include" => {
              attributes.get("schemaLocation").cloned()
            }
            _ => None,
          })
          .collect(),
      }),
      _ => Err(SchemaParseError::NotSchemaNode),
    }
  }
}

impl From<&Schema> for Vec<openapiv3::Schema> {
  fn from(s: &Schema) -> Self {
    s.types.iter().map(openapiv3::Schema::from).collect()
  }
}

#[test]
fn base_schema_into_schemas_test() {
  let s = Schema::try_from(include_bytes!("test_base.xsd") as &[u8]).unwrap();
  let value = Vec::<openapiv3::Schema>::from(&s);

  assert_eq!(
    serde_json::to_value(value).unwrap(),
    json!([
        {
            "title": "BaseType",
            "description": "A base abstract type for all the types.",
            "type": "object",
            "properties": {
                "baseField": {
                    "description": "A base field for the base type",
                    "type": "string"
                }
            },
            "additionalProperties": false
        }
    ])
  );
}

#[test]
fn schema_into_schemas_test() {
  let s = Schema::try_from(include_bytes!("test.xsd") as &[u8]).unwrap();
  let value = Vec::<openapiv3::Schema>::from(&s);

  assert_eq!(
    serde_json::to_value(value).unwrap(),
    json!([
      {
        "title": "TestType",
        "description": "A simple type to test the parser",
        "allOf": [
          {
            "$ref": "#/components/schemas/BaseType"
          },
          {
            "type": "object",
            "properties": {
              "requiredAttribute": {
                "readOnly": true,
                "description": "A field that comes from an attribute.",
                "type": "string"
              },
              "optionalString": {
                "description": "String that may or may not be here",
                "type": "string"
              },
              "requiredString": {
                "description": "String that will be here",
                "type": "string"
              },
              "readOnlyString": {
                "readOnly": true,
                "description": "String that can not be modified",
                "type": "string"
              },
              "createOnlyString": {
                "description": "String that can only be modified on create",
                "type": "string"
              },
              "updateOnlyString": {
                "description": "String that can only be modified on update",
                "type": "string"
              },
              "booleanField": {
                "description": "Test boolean field",
                "type": "boolean"
              },
              "deprecatedField": {
                "deprecated": true,
                "description": "This field is unused and is deprecated.",
                "type": "string"
              },
              "multilineDoc": {
                "description": "This is multiple lines of documentation.",
                "type": "string"
              },
              "signedThirtyTwo": {
                "description": "A signed 32 bit value",
                "type": "integer",
                "format": "int32"
              },
              "boundedCustom2": {
                "description": "A reference to another type, but only one or none",
                "allOf": [
                  {
                    "$ref": "#/components/schemas/Custom2Type"
                  }
                ]
              },
              "unboundedCustom2": {
                "readOnly": true,
                "description": "A reference to many of another type",
                "type": "array",
                "items": {
                  "$ref": "#/components/schemas/Custom3Type"
                }
              }
            },
            "required": [
              "requiredAttribute",
              "requiredString"
            ],
            "additionalProperties": false
          }
        ]
      },
      {
        "title": "Custom2Type",
        "description": "Part of a test.",
        "allOf": [
          {
            "$ref": "#/components/schemas/BaseType"
          },
          {
            "type": "object",
            "properties": {
              "someField": {
                "description": "Foo",
                "type": "string"
              }
            },
            "additionalProperties": false
          }
        ]
      },
      {
        "title": "Custom3Type",
        "description": "Part of a test continued.",
        "allOf": [
          {
            "$ref": "#/components/schemas/BaseType"
          },
          {
            "type": "object",
            "properties": {
              "someField2": {
                "readOnly": true,
                "description": "Bar",
                "type": "string"
              }
            },
            "additionalProperties": false
          }
        ]
      }
    ])
  );
}
