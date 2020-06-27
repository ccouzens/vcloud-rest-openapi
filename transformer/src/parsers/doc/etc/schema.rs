use crate::parsers::doc::etc::annotation::{Annotation, Modifiable};
use crate::parsers::doc::etc::complex_type::parse_complex_type;
use crate::parsers::doc::etc::complex_type::ComplexType;
use crate::parsers::doc::etc::sequence_element::{Occurrences, SequenceElement};
use crate::parsers::doc::etc::XML_SCHEMA_NS;

#[derive(Debug, PartialEq)]
pub(super) struct Schema {
    pub(super) includes: Vec<String>,
    pub(super) complex_types: Vec<ComplexType>,
}

pub(super) fn parse_schema(input: &xmltree::XMLNode) -> Option<Schema> {
    match input {
        xmltree::XMLNode::Element(xmltree::Element {
            namespace: Some(namespace),
            name,
            children,
            ..
        }) if namespace == XML_SCHEMA_NS && name == "schema" => Some(Schema {
            complex_types: children.iter().filter_map(parse_complex_type).collect(),
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
        _ => None,
    }
}

#[test]
fn test_parse_base_schema() {
    let tree = xmltree::Element::parse(include_bytes!("test_base.xsd") as &[u8]).unwrap();
    assert_eq!(
        parse_schema(&xmltree::XMLNode::Element(tree)),
        Some(Schema {
            includes: vec![],
            complex_types: vec![ComplexType {
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
            }]
        })
    );
}

#[test]
fn test_parse_schema() {
    let tree = xmltree::Element::parse(include_bytes!("test.xsd") as &[u8]).unwrap();
    assert_eq!(
        parse_schema(&xmltree::XMLNode::Element(tree)),
        Some(Schema {
            includes: vec!["test_base.xsd".to_owned()],
            complex_types: vec![
                ComplexType {
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
                },
                ComplexType {
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
                },
                ComplexType {
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
                }
            ]
        })
    );
}
