use crate::parsers::doc::etc::annotation::Annotation;
use crate::parsers::doc::etc::annotation::Modifiable;
use crate::parsers::doc::etc::parse_annotation;
use crate::parsers::doc::etc::parse_sequence_element;
use crate::parsers::doc::etc::sequence_element::{Occurrences, SequenceElement};
use crate::parsers::doc::etc::XML_SCHEMA_NS;

#[derive(Debug, PartialEq)]
pub(super) struct ComplexType {
    pub(super) annotation: Annotation,
    pub(super) name: String,
    pub(super) sequence_elements: Vec<SequenceElement>,
    pub(super) parent: Option<String>,
}

pub(super) fn parse_complex_type(input: &xmltree::XMLNode) -> Option<ComplexType> {
    match input {
        xmltree::XMLNode::Element(xmltree::Element {
            namespace: Some(namespace),
            name,
            attributes,
            children,
            ..
        }) if namespace == XML_SCHEMA_NS && name == "complexType" => {
            let name = attributes.get("name")?.clone();
            let annotation = children.iter().filter_map(parse_annotation).next()?;
            let mut sequence_elements = Vec::new();
            let mut parent = None;

            for child in children {
                match child {
                    xmltree::XMLNode::Element(xmltree::Element {
                        namespace: Some(namespace),
                        name,
                        children,
                        ..
                    }) if namespace == XML_SCHEMA_NS && name == "sequence" => {
                        sequence_elements
                            .extend(children.iter().filter_map(parse_sequence_element));
                    }
                    xmltree::XMLNode::Element(xmltree::Element {
                        namespace: Some(namespace),
                        name,
                        children,
                        ..
                    }) if namespace == XML_SCHEMA_NS && name == "complexContent" => {
                        for child in children {
                            match child {
                                xmltree::XMLNode::Element(xmltree::Element {
                                    attributes,
                                    namespace: Some(namespace),
                                    name,
                                    children,
                                    ..
                                }) if namespace == XML_SCHEMA_NS && name == "extension" => {
                                    parent = attributes.get("base").cloned();
                                    for child in children {
                                        match child {
                                            xmltree::XMLNode::Element(xmltree::Element {
                                                namespace: Some(namespace),
                                                name,
                                                children,
                                                ..
                                            }) if namespace == XML_SCHEMA_NS
                                                && name == "sequence" =>
                                            {
                                                sequence_elements.extend(
                                                    children
                                                        .iter()
                                                        .filter_map(parse_sequence_element),
                                                );
                                            }
                                            _ => {}
                                        }
                                    }
                                    sequence_elements
                                        .extend(children.iter().filter_map(parse_sequence_element));
                                }
                                _ => {}
                            }
                        }
                    }

                    _ => {}
                }
            }

            Some(ComplexType {
                name,
                annotation,
                sequence_elements,
                parent,
            })
        }
        _ => None,
    }
}

#[test]
fn parse_base_type_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" abstract="true" name="BaseType">
        <xs:annotation>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A base abstract type for all the types.
            </xs:documentation>
        </xs:annotation>

        <xs:sequence>
            <xs:element name="BaseField" type="xs:string" minOccurs="0">
                <xs:annotation>
                    <xs:documentation source="modifiable">always</xs:documentation>
                    <xs:documentation xml:lang="en">
                        A base field for the base type
                    </xs:documentation>
                    <xs:documentation source="required">false</xs:documentation>
                </xs:annotation>
            </xs:element>
        </xs:sequence>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_complex_type(&xmltree::XMLNode::Element(tree)),
        Some(ComplexType {
            annotation: Annotation {
                description: "A base abstract type for all the types.".to_owned(),
                required: None,
                deprecated: false,
                modifiable: None,
                content_type: None
            },
            name: "BaseType".to_owned(),
            sequence_elements: vec![SequenceElement {
                name: "baseField".to_owned(),
                r#type: "xs:string".to_owned(),
                occurrences: Occurrences::Optional,
                annotation: Some(Annotation {
                    description: "A base field for the base type".to_owned(),
                    required: Some(false),
                    deprecated: false,
                    modifiable: Some(Modifiable::Always),
                    content_type: None
                })
            }],
            parent: None
        })
    );
}

#[test]
fn parse_type_wth_parent_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:annotation>
            <xs:appinfo>
                <meta:content-type>application/vnd.ccouzens.test</meta:content-type>
            </xs:appinfo>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A simple type to test the parser
            </xs:documentation>
        </xs:annotation>
        <xs:complexContent>
            <xs:extension base="BaseType">
                <xs:sequence>
                    <xs:element name="OptionalString" type="xs:string" minOccurs="0">
                        <xs:annotation>
                            <xs:documentation source="modifiable">always</xs:documentation>
                            <xs:documentation xml:lang="en">
                                String that may or may not be here
                            </xs:documentation>
                            <xs:documentation source="required">false</xs:documentation>
                        </xs:annotation>
                    </xs:element>
                    <xs:element name="RequiredString" type="xs:string" minOccurs="1">
                        <xs:annotation>
                            <xs:documentation source="modifiable">always</xs:documentation>
                            <xs:documentation xml:lang="en">
                                String that will be here
                            </xs:documentation>
                            <xs:documentation source="required">true</xs:documentation>
                        </xs:annotation>
                    </xs:element>
                </xs:sequence>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_complex_type(&xmltree::XMLNode::Element(tree)),
        Some(ComplexType {
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
                    name: "requiredString".to_string(),
                    r#type: "xs:string".to_owned(),
                    occurrences: Occurrences::One
                }
            ],
            parent: Some("BaseType".to_owned())
        })
    );
}
