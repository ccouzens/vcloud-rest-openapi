const XML_SCHEMA_NS: &str = "http://www.w3.org/2001/XMLSchema";

#[derive(Debug, PartialEq)]
enum Modifiable {
    Create,
    Update,
    Always,
    None,
}

#[derive(Debug, PartialEq)]
struct Annotation {
    description: String,
    required: Option<bool>,
    deprecated: bool,
    modifiable: Option<Modifiable>,
    content_type: Option<String>,
}

fn parse_annotation(input: &xmltree::XMLNode) -> Option<Annotation> {
    match input {
        xmltree::XMLNode::Element(xmltree::Element {
            namespace: Some(namespace),
            name,
            children,
            ..
        }) if namespace == XML_SCHEMA_NS && name == "annotation" => {
            let description = html2md::parse_html(
                children
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
                    .next()?,
            );
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
            let modifiable = children
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
                        && attributes.get("source").map(String::as_str) == Some("modifiable") =>
                    {
                        match children.get(0) {
                            Some(xmltree::XMLNode::Text(r)) => match r.as_str() {
                                "create" => Some(Modifiable::Create),
                                "update" => Some(Modifiable::Update),
                                "always" => Some(Modifiable::Always),
                                "none" => Some(Modifiable::None),
                                _ => None,
                            },
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .next();

            let content_type = children
                .iter()
                .filter_map(|child| match child {
                    xmltree::XMLNode::Element(xmltree::Element {
                        namespace: Some(namespace),
                        name,
                        children,
                        ..
                    }) if namespace == XML_SCHEMA_NS && name == "appinfo" => children
                        .iter()
                        .filter_map(|child| match child {
                            xmltree::XMLNode::Element(xmltree::Element {
                                namespace: Some(namespace),
                                name,
                                children,
                                ..
                            }) if namespace == "http://www.vmware.com/vcloud/meta"
                                && name == "content-type" =>
                            {
                                match children.get(0) {
                                    Some(xmltree::XMLNode::Text(ct)) => Some(ct),
                                    _ => None,
                                }
                            }
                            _ => None,
                        })
                        .next(),
                    _ => None,
                })
                .next()
                .cloned();

            Some(Annotation {
                description,
                required,
                deprecated,
                modifiable,
                content_type,
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
            deprecated: false,
            modifiable: None,
            content_type: None
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
            deprecated: false,
            modifiable: None,
            content_type: None
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
            deprecated: false,
            modifiable: None,
            content_type: None
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
            deprecated: true,
            modifiable: None,
            content_type: None
        })
    );
}

#[test]
fn test_parse_annotation_modifiable_create() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:documentation source="modifiable">create</xs:documentation>
        <xs:documentation xml:lang="en">
            A field that is only settable on &lt;i&gt;create&lt;/i&gt;.
        </xs:documentation>
        </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_annotation(&xmltree::XMLNode::Element(tree)),
        Some(Annotation {
            description: "A field that is only settable on *create*.".to_owned(),
            required: None,
            deprecated: false,
            modifiable: Some(Modifiable::Create),
            content_type: None
        })
    );
}

#[test]
fn test_parse_annotation_modifiable_update() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:documentation source="modifiable">update</xs:documentation>
        <xs:documentation xml:lang="en">
            A field that is only settable on &lt;i&gt;update&lt;/i&gt;.
        </xs:documentation>
        </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_annotation(&xmltree::XMLNode::Element(tree)),
        Some(Annotation {
            description: "A field that is only settable on *update*.".to_owned(),
            required: None,
            deprecated: false,
            modifiable: Some(Modifiable::Update),
            content_type: None
        })
    );
}

#[test]
fn test_parse_annotation_modifiable_always() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:documentation source="modifiable">always</xs:documentation>
        <xs:documentation xml:lang="en">
            A field that is &lt;i&gt;always&lt;/i&gt; settable.
        </xs:documentation>
        </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_annotation(&xmltree::XMLNode::Element(tree)),
        Some(Annotation {
            description: "A field that is *always* settable.".to_owned(),
            required: None,
            deprecated: false,
            modifiable: Some(Modifiable::Always),
            content_type: None
        })
    );
}

#[test]
fn test_parse_annotation_modifiable_none() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:documentation source="modifiable">none</xs:documentation>
        <xs:documentation xml:lang="en">
            A field that is &lt;i&gt;read only&lt;/i&gt;.
        </xs:documentation>
        </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_annotation(&xmltree::XMLNode::Element(tree)),
        Some(Annotation {
            description: "A field that is *read only*.".to_owned(),
            required: None,
            deprecated: false,
            modifiable: Some(Modifiable::None),
            content_type: None
        })
    );
}

#[test]
fn test_parse_content_type() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
        <xs:appinfo>
            <meta:content-type>application/vnd.ccouzens.test</meta:content-type>
        </xs:appinfo>
        <xs:documentation xml:lang="en">
            A type with a content type.
        </xs:documentation>
        </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        parse_annotation(&xmltree::XMLNode::Element(tree)),
        Some(Annotation {
            description: "A type with a content type.".to_owned(),
            required: None,
            deprecated: false,
            modifiable: None,
            content_type: Some("application/vnd.ccouzens.test".to_owned())
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
