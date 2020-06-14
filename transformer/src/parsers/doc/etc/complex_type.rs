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
                    _ => {}
                }
            }

            Some(ComplexType {
                name,
                annotation,
                sequence_elements,
                parent: None,
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
