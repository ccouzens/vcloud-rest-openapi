use crate::parsers::doc::etc::annotation::Annotation;
use crate::parsers::doc::etc::annotation::Modifiable;
use crate::parsers::doc::etc::parse_annotation;
use crate::parsers::doc::etc::XML_SCHEMA_NS;

#[derive(Debug, PartialEq)]
pub(super) enum MinOccurrences {
    Zero,
    One,
}

#[derive(Debug, PartialEq)]
pub(super) enum MaxOccurrences {
    One,
    Unbounded,
}

#[derive(Debug, PartialEq)]
pub(super) struct SequenceElement {
    pub(super) annotation: Option<Annotation>,
    pub(super) name: String,
    pub(super) r#type: String,
    pub(super) min_occurs: Option<MinOccurrences>,
    pub(super) max_occurs: Option<MaxOccurrences>,
}

pub(super) fn parse_sequence_element(input: &xmltree::XMLNode) -> Option<SequenceElement> {
    match input {
        xmltree::XMLNode::Element(xmltree::Element {
            namespace: Some(namespace),
            name,
            attributes,
            children,
            ..
        }) if namespace == XML_SCHEMA_NS && name == "element" => {
            // Name comes from the attribute name.
            // In the XML the fields start with a uppercase letter.
            // But in the JSON, the first letter is lowercase.
            let name = attributes
                .get("name")?
                .chars()
                .enumerate()
                .map(|(i, c)| if i == 0 { c.to_ascii_lowercase() } else { c })
                .collect();
            let r#type = attributes.get("type")?.to_owned();
            let min_occurs = match attributes.get("minOccurs").map(String::as_str) {
                Some("0") => Some(MinOccurrences::Zero),
                Some("1") => Some(MinOccurrences::One),
                _ => None,
            };
            let annotation = children.iter().filter_map(parse_annotation).next();
            Some(SequenceElement {
                annotation,
                name,
                r#type,
                min_occurs,
                max_occurs: None,
            })
        }
        _ => None,
    }
}

#[test]
fn test_parse_sequence_element() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:string" minOccurs="0">
        <xs:annotation>
            <xs:documentation source="modifiable">always</xs:documentation>
            <xs:documentation xml:lang="en">
                A base field for the base type
            </xs:documentation>
            <xs:documentation source="required">false</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_sequence_element(&xmltree::XMLNode::Element(tree)),
        Some(SequenceElement {
            annotation: Some(Annotation {
                description: "A base field for the base type".to_owned(),
                required: Some(false),
                deprecated: false,
                modifiable: Some(Modifiable::Always),
                content_type: None
            }),
            name: "baseField".to_owned(),
            r#type: "xs:string".to_owned(),
            min_occurs: Some(MinOccurrences::Zero),
            max_occurs: None
        })
    );
}
