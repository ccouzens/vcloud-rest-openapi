#[derive(Debug, PartialEq)]
struct Annotation {
    description: String,
}

fn parse_annotation(input: &xmltree::XMLNode) -> Option<Annotation> {
    match input {
        xmltree::XMLNode::Element(xmltree::Element {
            namespace: Some(namespace),
            name,
            children,
            ..
        }) if namespace == "http://www.w3.org/2001/XMLSchema" && name == "annotation" => {
            let description = children
                .iter()
                .filter_map(|child| match child {
                    xmltree::XMLNode::Element(xmltree::Element {
                        namespace: Some(namespace),
                        name,
                        children,
                        attributes,
                        ..
                    }) if namespace == "http://www.w3.org/2001/XMLSchema"
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
            let description = html2md::parse_html(description);
            Some(Annotation { description })
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
            description: "A base abstract **type** for all the  \ntypes.".to_owned()
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
        }) if namespace == "http://www.w3.org/2001/XMLSchema" && name == "schema" => {
            for child in children {
                match child {
                    xmltree::XMLNode::Element(xmltree::Element {
                        namespace: Some(namespace),
                        name,
                        children,
                        attributes,
                        ..
                    }) if namespace == "http://www.w3.org/2001/XMLSchema"
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
                            })) if name == "annotation"
                                && namespace == "http://www.w3.org/2001/XMLSchema" =>
                            {
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
