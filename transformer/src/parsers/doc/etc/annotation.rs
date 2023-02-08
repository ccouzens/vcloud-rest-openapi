use crate::parsers::doc::etc::XML_SCHEMA_NS;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub(super) struct Annotation {
    pub(super) description: Option<String>,
    pub(super) required: Option<bool>,
    pub(super) deprecated: bool,
    pub(super) content_type: Option<String>,
    pub(super) removed: bool,
}

#[derive(Error, Debug, PartialEq)]
pub enum AnnotationParseError {
    #[error("not an annotation node")]
    NotAnnotationNode,
}

impl Annotation {
    pub(super) fn merge(self, b: Annotation) -> Self {
        Self {
            description: self.description.or(b.description),
            content_type: self.content_type.or(b.content_type),
            deprecated: self.deprecated || b.deprecated,
            required: self.required.or(b.required),
            removed: self.removed || b.removed,
        }
    }
}

impl TryFrom<&Vec<xmltree::XMLNode>> for Annotation {
    type Error = AnnotationParseError;

    fn try_from(children: &Vec<xmltree::XMLNode>) -> Result<Self, Self::Error> {
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
                    && (attributes.get("lang").map(String::as_str) == Some("en")
                        || attributes.is_empty()) =>
                {
                    match children.get(0) {
                        Some(xmltree::XMLNode::Text(doc)) => Some(doc),
                        _ => None,
                    }
                }
                _ => None,
            })
            .next()
            .map(|d| html2md::parse_html(d));
        let required = children
            .iter()
            .filter_map(|child| match child {
                xmltree::XMLNode::Element(xmltree::Element {
                    namespace: Some(_xml_schema_ns),
                    name,
                    children,
                    attributes,
                    ..
                }) if name == "documentation"
                    && attributes.get("source").map(String::as_str) == Some("required") =>
                {
                    match children.get(0) {
                        Some(xmltree::XMLNode::Text(r)) => match r.trim() {
                            "true" => Some(true),
                            "false" => Some(false),
                            _ => None,
                        },
                        _ => None,
                    }
                }
                _ => None,
            })
            .next();
        let deprecated = children.iter().any(|child| match child {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(_xml_schema_ns),
                name,
                attributes,
                ..
            }) if name == "documentation" => {
                attributes.get("source").map(String::as_str) == Some("deprecated")
            }
            _ => false,
        });
        let _xml_schema_ns_meta = String::from("http://www.vmware.com/vcloud/meta");
        let removed = children.iter().any(|child| match child {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(_xml_schema_ns),
                name,
                attributes,
                ..
            }) if name == "documentation" => {
                attributes.get("source").map(String::as_str) == Some("removed-in")
            }
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(_xml_schema_ns_meta),
                name,
                attributes,
                ..
            }) if name == "version" => attributes.contains_key("removed-in"),
            _ => false,
        });

        let content_type = children
            .iter()
            .filter_map(|child| match child {
                xmltree::XMLNode::Element(xmltree::Element {
                    namespace: Some(_xml_schema_ns),
                    name,
                    children,
                    ..
                }) if name == "appinfo" => children
                    .iter()
                    .filter_map(|child| match child {
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(_xml_schema_ns_meta),
                            name,
                            children,
                            ..
                        }) if name == "content-type" => match children.get(0) {
                            Some(xmltree::XMLNode::Text(ct)) => Some(ct.trim().to_owned()),
                            _ => None,
                        },
                        _ => None,
                    })
                    .next(),
                _ => None,
            })
            .next();
        Ok(Annotation {
            description,
            required,
            deprecated,
            content_type,
            removed,
        })
    }
}

impl TryFrom<&xmltree::XMLNode> for Annotation {
    type Error = AnnotationParseError;

    fn try_from(value: &xmltree::XMLNode) -> Result<Self, Self::Error> {
        match value {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                children,
                ..
            }) if namespace == XML_SCHEMA_NS && name == "annotation" => {
                let annotation_a = children
                    .iter()
                    .filter_map(|n| match n {
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(namespace),
                            name,
                            children,
                            ..
                        }) if namespace == XML_SCHEMA_NS && name == "appinfo" => {
                            Annotation::try_from(children).ok()
                        }
                        _ => None,
                    })
                    .next();
                let annotation_b = Annotation::try_from(children);
                match (annotation_a, annotation_b) {
                    (None, Ok(b)) => Ok(b),
                    (None, Err(e)) => Err(e),
                    (Some(a), Ok(b)) => Ok(a.merge(b)),
                    (Some(a), Err(_)) => Ok(a),
                }
            }
            _ => Err(AnnotationParseError::NotAnnotationNode),
        }
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
        Annotation::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Annotation {
            description: Some("A base abstract **type** for all the  \ntypes.".to_owned()),
            required: None,
            deprecated: false,
            content_type: None,
            removed: false
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
        Annotation::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Annotation {
            description: Some("A field that is *required*.".to_owned()),
            required: Some(true),
            deprecated: false,
            content_type: None,
            removed: false
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
        <xs:documentation source="required">
          false
        </xs:documentation>
    </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        Annotation::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Annotation {
            description: Some("A field that is *not required*.".to_owned()),
            required: Some(false),
            deprecated: false,
            content_type: None,
            removed: false
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
        Annotation::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Annotation {
            description: Some("A field that is *deprecated*.".to_owned()),
            required: None,
            deprecated: true,
            content_type: None,
            removed: false
        })
    );
}

#[test]
fn test_parse_content_type() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
        <xs:appinfo>
            <meta:content-type>
              application/vnd.ccouzens.test
            </meta:content-type>
        </xs:appinfo>
        <xs:documentation xml:lang="en">
            A type with a content type.
        </xs:documentation>
        </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        Annotation::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Annotation {
            description: Some("A type with a content type.".to_owned()),
            required: None,
            deprecated: false,
            content_type: Some("application/vnd.ccouzens.test".to_owned()),
            removed: false
        })
    );
}

#[test]
fn test_parse_annotation_inside_app_info() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema">
      <xs:appinfo>
        <xs:documentation source="since">0.9</xs:documentation>
        <xs:documentation xml:lang="en">
            A base abstract &lt;b&gt;type&lt;/b&gt; for
            all the&lt;br/&gt;types.
        </xs:documentation>
      </xs:appinfo>
    </xs:annotation>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        Annotation::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Annotation {
            description: Some("A base abstract **type** for all the  \ntypes.".to_owned()),
            required: None,
            deprecated: false,
            content_type: None,
            removed: false
        })
    );
}

#[test]
fn test_annotation_indicating_removal() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
        <xs:appinfo><meta:version removed-in="API_VERSION_POST9_1_UPDATE"/></xs:appinfo>
        <xs:documentation source="modifiable">always</xs:documentation>
        <xs:documentation xml:lang="en">
            This field has been removed
        </xs:documentation>
        <xs:documentation source="required">false</xs:documentation>
        <xs:documentation source="deprecated">6.0</xs:documentation>
        <xs:documentation source="removed-in">API_VERSION_POST9_1_UPDATE</xs:documentation>
    </xs:annotation>"#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        Annotation::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Annotation {
            description: Some("This field has been removed".to_owned()),
            required: Some(false),
            deprecated: true,
            content_type: None,
            removed: true
        })
    );
}

#[test]
fn test_alternative_removal_syntax() {
    let xml: &[u8] = br#"
    <xs:annotation xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
        <xs:appinfo><meta:version removed-in="API_VERSION_POST9_1_UPDATE"/></xs:appinfo>
        <xs:documentation source="modifiable">always</xs:documentation>
        <xs:documentation xml:lang="en">
            This field has been removed
        </xs:documentation>
        <xs:documentation source="required">false</xs:documentation>
        <xs:documentation source="deprecated">6.0</xs:documentation>
    </xs:annotation>"#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        Annotation::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Annotation {
            description: Some("This field has been removed".to_owned()),
            required: Some(false),
            deprecated: true,
            content_type: None,
            removed: true
        })
    );
}
