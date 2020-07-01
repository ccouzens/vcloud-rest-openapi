#[cfg(test)]
use crate::parsers::doc::etc::annotation::{Annotation, Modifiable};
#[cfg(test)]
use crate::parsers::doc::etc::object_type::ObjectType;
use crate::parsers::doc::etc::r#type::Type;
#[cfg(test)]
use crate::parsers::doc::etc::sequence_element::{Occurrences, SequenceElement};
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
fn test_parse_base_schema() {
    assert_eq!(
        Schema::try_from(include_bytes!("test_base.xsd") as &[u8]).unwrap(),
        Schema {
            includes: vec![],
            types: vec![Type::ObjectType(ObjectType {
                annotation: Annotation {
                    description: "A base abstract type for all the types.".to_owned(),
                    required: None,
                    deprecated: false,
                    modifiable: None,
                    content_type: None
                },
                name: "BaseType".to_owned(),
                sequence_elements: vec![SequenceElement {
                    annotation: Some(Annotation {
                        description: "A base field for the base type".to_owned(),
                        required: Some(false),
                        deprecated: false,
                        modifiable: Some(Modifiable::Always),
                        content_type: None
                    }),
                    name: "baseField".to_owned(),
                    r#type: "xs:string".to_owned(),
                    occurrences: Occurrences::Optional
                }],
                parent: None
            })]
        }
    );
}

#[test]
fn test_parse_schema() {
    assert_eq!(
        Schema::try_from(include_bytes!("test.xsd") as &[u8]).unwrap(),
        Schema {
            includes: vec!["test_base.xsd".to_owned()],
            types: vec![
                Type::ObjectType(ObjectType {
                    annotation: Annotation {
                        description: "A simple type to test the parser".to_owned(),
                        required: None,
                        deprecated: false,
                        modifiable: None,
                        content_type: Some("application/vnd.ccouzens.test".to_owned())
                    },
                    name: "TestType".to_owned(),
                    sequence_elements: vec![
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "A field that comes from an attribute.".to_owned(),
                                required: Some(true),
                                deprecated: false,
                                modifiable: Some(Modifiable::None),
                                content_type: None
                            }),
                            name: "requiredAttribute".to_owned(),
                            r#type: "xs:string".to_owned(),
                            occurrences: Occurrences::One
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "String that may or may not be here".to_owned(),
                                required: Some(false),
                                deprecated: false,
                                modifiable: Some(Modifiable::Always),
                                content_type: None
                            }),
                            name: "optionalString".to_owned(),
                            r#type: "xs:string".to_owned(),
                            occurrences: Occurrences::Optional
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "String that will be here".to_owned(),
                                required: Some(true),
                                deprecated: false,
                                modifiable: Some(Modifiable::Always),
                                content_type: None
                            }),
                            name: "requiredString".to_owned(),
                            r#type: "xs:string".to_owned(),
                            occurrences: Occurrences::One
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "String that can not be modified".to_owned(),
                                required: Some(false),
                                deprecated: false,
                                modifiable: Some(Modifiable::None),
                                content_type: None
                            }),
                            name: "readOnlyString".to_owned(),
                            r#type: "xs:string".to_owned(),
                            occurrences: Occurrences::Optional
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "String that can only be modified on create"
                                    .to_owned(),
                                required: Some(false),
                                deprecated: false,
                                modifiable: Some(Modifiable::Create),
                                content_type: None
                            }),
                            name: "createOnlyString".to_owned(),
                            r#type: "xs:string".to_owned(),
                            occurrences: Occurrences::Optional
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "String that can only be modified on update"
                                    .to_owned(),
                                required: Some(false),
                                deprecated: false,
                                modifiable: Some(Modifiable::Update),
                                content_type: None
                            }),
                            name: "updateOnlyString".to_owned(),
                            r#type: "xs:string".to_owned(),
                            occurrences: Occurrences::Optional
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "Test boolean field".to_owned(),
                                required: Some(false),
                                deprecated: false,
                                modifiable: Some(Modifiable::Always),
                                content_type: None
                            }),
                            name: "booleanField".to_owned(),
                            r#type: "xs:boolean".to_owned(),
                            occurrences: Occurrences::Optional
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "This field is unused and is deprecated.".to_owned(),
                                required: Some(false),
                                deprecated: true,
                                modifiable: Some(Modifiable::Always),
                                content_type: None
                            }),
                            name: "deprecatedField".to_owned(),
                            r#type: "xs:string".to_owned(),
                            occurrences: Occurrences::Optional
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "This is multiple lines of documentation.".to_owned(),
                                required: Some(false),
                                deprecated: false,
                                modifiable: Some(Modifiable::Always),
                                content_type: None
                            }),
                            name: "multilineDoc".to_owned(),
                            r#type: "xs:string".to_owned(),
                            occurrences: Occurrences::Optional
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "A signed 32 bit value".to_owned(),
                                required: Some(false),
                                deprecated: false,
                                modifiable: Some(Modifiable::Always),
                                content_type: None
                            }),
                            name: "signedThirtyTwo".to_owned(),
                            r#type: "xs:int".to_owned(),
                            occurrences: Occurrences::Optional
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "A reference to another type, but only one or none"
                                    .to_owned(),
                                required: Some(false),
                                deprecated: false,
                                modifiable: Some(Modifiable::Always),
                                content_type: None
                            }),
                            name: "boundedCustom2".to_owned(),
                            r#type: "Custom2Type".to_owned(),
                            occurrences: Occurrences::Optional
                        },
                        SequenceElement {
                            annotation: Some(Annotation {
                                description: "A reference to many of another type".to_owned(),
                                required: Some(false),
                                deprecated: false,
                                modifiable: Some(Modifiable::None),
                                content_type: None
                            }),
                            name: "unboundedCustom2".to_owned(),
                            r#type: "Custom3Type".to_owned(),
                            occurrences: Occurrences::Array
                        }
                    ],
                    parent: Some("BaseType".to_owned())
                }),
                Type::ObjectType(ObjectType {
                    annotation: Annotation {
                        description: "Part of a test.".to_owned(),
                        required: None,
                        deprecated: false,
                        modifiable: None,
                        content_type: Some("application/vnd.ccouzens.custom2".to_owned())
                    },
                    name: "Custom2Type".to_owned(),
                    sequence_elements: vec![SequenceElement {
                        annotation: Some(Annotation {
                            description: "Foo".to_owned(),
                            required: Some(false),
                            deprecated: false,
                            modifiable: Some(Modifiable::Always),
                            content_type: None
                        }),
                        name: "someField".to_owned(),
                        r#type: "xs:string".to_owned(),
                        occurrences: Occurrences::Optional
                    }],
                    parent: Some("BaseType".to_owned())
                }),
                Type::ObjectType(ObjectType {
                    annotation: Annotation {
                        description: "Part of a test continued.".to_owned(),
                        required: None,
                        deprecated: false,
                        modifiable: None,
                        content_type: None
                    },
                    name: "Custom3Type".to_owned(),
                    sequence_elements: vec![SequenceElement {
                        annotation: Some(Annotation {
                            description: "Bar".to_owned(),
                            required: Some(false),
                            deprecated: false,
                            modifiable: Some(Modifiable::None),
                            content_type: None
                        }),
                        name: "someField2".to_owned(),
                        r#type: "xs:string".to_owned(),
                        occurrences: Occurrences::Optional
                    }],
                    parent: Some("BaseType".to_owned())
                })
            ]
        }
    );
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
                        "nullable": true,
                        "description": "A base field for the base type",
                        "type": "string"
                    }
                },
                "required": [
                    "baseField"
                ],
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
                    "nullable": true,
                    "description": "String that may or may not be here",
                    "type": "string"
                  },
                  "requiredString": {
                    "description": "String that will be here",
                    "type": "string"
                  },
                  "readOnlyString": {
                    "nullable": true,
                    "readOnly": true,
                    "description": "String that can not be modified",
                    "type": "string"
                  },
                  "createOnlyString": {
                    "nullable": true,
                    "description": "String that can only be modified on create",
                    "type": "string"
                  },
                  "updateOnlyString": {
                    "nullable": true,
                    "description": "String that can only be modified on update",
                    "type": "string"
                  },
                  "booleanField": {
                    "nullable": true,
                    "description": "Test boolean field",
                    "type": "boolean"
                  },
                  "deprecatedField": {
                    "nullable": true,
                    "deprecated": true,
                    "description": "This field is unused and is deprecated.",
                    "type": "string"
                  },
                  "multilineDoc": {
                    "nullable": true,
                    "description": "This is multiple lines of documentation.",
                    "type": "string"
                  },
                  "signedThirtyTwo": {
                    "nullable": true,
                    "description": "A signed 32 bit value",
                    "type": "integer",
                    "format": "int32"
                  },
                  "boundedCustom2": {
                    "nullable": true,
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
                  "optionalString",
                  "requiredString",
                  "readOnlyString",
                  "createOnlyString",
                  "updateOnlyString",
                  "booleanField",
                  "deprecatedField",
                  "multilineDoc",
                  "signedThirtyTwo",
                  "boundedCustom2",
                  "unboundedCustom2"
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
                    "nullable": true,
                    "description": "Foo",
                    "type": "string"
                  }
                },
                "required": [
                  "someField"
                ],
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
                    "nullable": true,
                    "readOnly": true,
                    "description": "Bar",
                    "type": "string"
                  }
                },
                "required": [
                  "someField2"
                ],
                "additionalProperties": false
              }
            ]
          }
        ])
    );
}
