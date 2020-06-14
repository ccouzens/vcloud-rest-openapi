use crate::parsers::doc::etc::XML_SCHEMA_NS;

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
