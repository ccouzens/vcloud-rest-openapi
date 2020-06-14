const XML_SCHEMA_NS: &str = "http://www.w3.org/2001/XMLSchema";

#[derive(Debug, PartialEq)]
struct Annotation {
    description: String,
    required: Option<bool>,
    deprecated: bool,
}

fn parse_annotation(input: &xmltree::XMLNode) -> Option<Annotation> {
    match input {
        xmltree::XMLNode::Element(xmltree::Element {
            namespace: Some(namespace),
            name,
            children,
            ..
        }) if namespace == XML_SCHEMA_NS && name == "annotation" => {
            let description = children
                .iter()
                .filter_map(|child| match child {
                    xmltree::XMLNode::Element(xmltree::Element {
                        namespace: Some(namespace),
                        name,
                        children,
                        attributes,
                        ..
                    }) if namespace == XML_SCHEMA_NS
                        && name == "documentation"
                        && attributes.get("lang").map(String::as_str) == Some("en") =>
                    {
                        match children.get(0) {
                            Some(xmltree::XMLNode::Text(doc)) => Some(doc),
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .next()?;
            let required = children
                .iter()
                .filter_map(|child| match child {
                    xmltree::XMLNode::Element(xmltree::Element {
                        namespace: Some(namespace),
                        name,
                        children,
                        attributes,
                        ..
                    }) if namespace == XML_SCHEMA_NS
                        && name == "documentation"
                        && attributes.get("source").map(String::as_str) == Some("required") =>
                    {
                        match children.get(0) {
                            Some(xmltree::XMLNode::Text(r)) if r == "true" => Some(true),
                            Some(xmltree::XMLNode::Text(r)) if r == "false" => Some(false),
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .next();
            let deprecated = children.iter().any(|child| match child {
                xmltree::XMLNode::Element(xmltree::Element {
                    namespace: Some(namespace),
                    name,
                    attributes,
                    ..
                }) if namespace == XML_SCHEMA_NS
                    && name == "documentation"
                    && attributes.get("source").map(String::as_str) == Some("deprecated") =>
                {
                    true
                }
                _ => false,
            });

            let description = html2md::parse_html(description);
            Some(Annotation {
                description,
                required,
                deprecated,
            })
        }
        _ => None,
    }
}

#[test]
fn test_parse_annotation() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:documentation source="since">0.9</xs:documentation>
        <xs:documentation xml:lang="en">
            A base abstract &lt;b&gt;type&lt;/b&gt; for
            all the&lt;br/&gt;types.
        </xs:documentation>
    </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_annotation(&xmltree::XMLNode::Element(tree)),
        Some(Annotation {
            description: "A base abstract **type** for all the  \ntypes.".to_owned(),
            required: None,
            deprecated: false
        })
    );
}

#[test]
fn test_parse_annotation_required() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:documentation source="since">0.9</xs:documentation>
        <xs:documentation xml:lang="en">
            A field that is &lt;i&gt;required&lt;/i&gt;.
        </xs:documentation>
        <xs:documentation source="required">true</xs:documentation>
    </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_annotation(&xmltree::XMLNode::Element(tree)),
        Some(Annotation {
            description: "A field that is *required*.".to_owned(),
            required: Some(true),
            deprecated: false
        })
    );
}

#[test]
fn test_parse_annotation_not_required() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:documentation source="since">0.9</xs:documentation>
        <xs:documentation xml:lang="en">
            A field that is &lt;i&gt;not required&lt;/i&gt;.
        </xs:documentation>
        <xs:documentation source="required">false</xs:documentation>
    </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_annotation(&xmltree::XMLNode::Element(tree)),
        Some(Annotation {
            description: "A field that is *not required*.".to_owned(),
            required: Some(false),
            deprecated: false
        })
    );
}
#[test]
fn test_parse_annotation_deprecated() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:documentation source="since">0.9</xs:documentation>
        <xs:documentation source="deprecated">34.0</xs:documentation>
        <xs:documentation xml:lang="en">
            A field that is &lt;i&gt;deprecated&lt;/i&gt;.
        </xs:documentation>
        </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_annotation(&xmltree::XMLNode::Element(tree)),
        Some(Annotation {
            description: "A field that is *deprecated*.".to_owned(),
            required: None,
            deprecated: true
        })
    );
}

#[test]
fn basic() {
    let file: &[u8] = include_bytes!("test_base.xsd");
    let tree = xmltree::Element::parse(file);

    match tree {
        Ok(xmltree::Element {
            namespace: Some(namespace),
            name,
            children,
            ..
        }) if namespace == XML_SCHEMA_NS && name == "schema" => {
            for child in children {
                match child {
                    xmltree::XMLNode::Element(xmltree::Element {
                        namespace: Some(namespace),
                        name,
                        children,
                        attributes,
                        ..
                    }) if namespace == XML_SCHEMA_NS
                        && name == "complexType"
                        && attributes.contains_key("name") =>
                    {
                        dbg!(&attributes["name"]);
                        match &children.get(0) {
                            Some(xmltree::XMLNode::Element(xmltree::Element {
                                name,
                                namespace: Some(namespace),
                                children,
                                ..
                            })) if name == "annotation" && namespace == XML_SCHEMA_NS => {
                                dbg!(children);
                            }
                            _ => panic!("no annotations"),
                        }
                    }
                    _ => panic!("not matched"),
                }
            }
        }
        _ => panic!("didn't match"),
    }
}
