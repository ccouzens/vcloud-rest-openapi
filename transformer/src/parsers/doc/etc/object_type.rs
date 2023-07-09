#[cfg(test)]
use super::r#type::Type;
use super::{
    field::Occurrences,
    simple_type::{str_to_simple_type_or_reference, SimpleType},
};
use crate::parsers::doc::etc::annotation::Annotation;
use crate::parsers::doc::etc::field::Field;
use crate::parsers::doc::etc::group_ref::GroupRef;
use crate::parsers::doc::etc::r#type::TypeParseError;
use crate::parsers::doc::etc::XML_SCHEMA_NS;
use openapiv3::Discriminator;
#[cfg(test)]
use serde_json::json;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub(super) struct ObjectType {
    pub(super) annotation: Option<Annotation>,
    pub(super) name: String,
    pub(super) fields: Vec<Field>,
    pub(super) parents: Vec<openapiv3::ReferenceOr<SimpleType>>,
    pub(super) descendants: Vec<String>,
}

impl
    TryFrom<(
        Option<&str>,
        &xmltree::XMLNode,
        &Vec<(Option<&str>, xmltree::XMLNode)>,
    )> for ObjectType
{
    type Error = TypeParseError;

    fn try_from(
        (ns, xml, types): (
            Option<&str>,
            &xmltree::XMLNode,
            &Vec<(Option<&str>, xmltree::XMLNode)>,
        ),
    ) -> Result<Self, Self::Error> {
        match xml {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                attributes,
                children,
                ..
            }) if namespace == XML_SCHEMA_NS
                && match name.as_str() {
                    "complexType" | "group" | "attributeGroup" => true,
                    _ => false,
                } =>
            {
                let mut annotations = Vec::new();
                // TODO: check qualified names in root by type and base attributes (PrefixedName = Prefix ':' LocalPart) see for details: https://www.w3.org/TR/xml-names11/#NT-QName
                let type_name = attributes.get("name");
                let name = type_name
                    .map(String::from)
                    .map(|type_name| {
                        ns.map_or(type_name.to_owned(), |ns| format!("{}_{}", ns, type_name))
                    })
                    .ok_or(TypeParseError::MissingName)?;
                annotations.extend(children.iter().filter_map(|c| Annotation::try_from(c).ok()));
                let mut fields = Vec::new();
                let mut parents = Vec::new();
                let descendants = type_name
                    // TODO: Clarify all possible discriminators types
                    // Filter out types with a property in the payload that holds the discriminator value.
                    .filter(|&type_name| match type_name.as_str() {
                        "QueryResultRecordType"
                        | "MetadataTypedValue"
                        | "HardwareVersionBaseType"
                        | "SupportedHardwareVersionBaseType"
                        | "Section_Type" => true,
                        _ => false,
                    })
                    .map(|type_name| {
                        types
                            .iter()
                            .flat_map(|(ns_type, xml)| {
                                xml.as_element()
                                    .map(|e| {
                                        e.children
                                            .iter()
                                            .flat_map(|child| match child {
                                                xmltree::XMLNode::Element(xmltree::Element {
                                                    attributes,
                                                    namespace: Some(_xml_schema_ns),
                                                    name,
                                                    children,
                                                    ..
                                                }) if name == "complexType" => children
                                                    .iter()
                                                    .find_map(|child| match child {
                                                        xmltree::XMLNode::Element(
                                                            xmltree::Element {
                                                                namespace: Some(_xml_schema_ns),
                                                                name,
                                                                children,
                                                                ..
                                                            },
                                                        ) if match name.as_str() {
                                                            "complexContent" | "simpleContent" => {
                                                                true
                                                            }
                                                            _ => false,
                                                        } =>
                                                        {
                                                            children.iter().find_map(|child| {
                                                                match child {
                                                        xmltree::XMLNode::Element(
                                                            xmltree::Element {
                                                                namespace: Some(_xml_schema_ns),
                                                                name,
                                                                attributes,
                                                                ..
                                                            },
                                                        ) if name == "extension" => attributes
                                                            .get("base")
                                                            .filter(|&name| {
                                                                match name.split_once(':') {
                                                                    Some((namespace, name)) => {
                                                                        ns.map_or(false, |ns| ns == namespace && type_name.eq(name))
                                                                    },
                                                                    None => type_name.eq(name),
                                                                }
                                                            })
                                                            .map(|name| match name.split_once(':') {
                                                                Some((_, _)) => name.into(),
                                                                None => ns_type.map_or(name.to_owned(), |ns| format!("{}:{}", ns, name))
                                                            }),
                                                        _ => None,
                                                    }
                                                            })
                                                        }
                                                        _ => None,
                                                    })
                                                    .and_then(|_| {
                                                        attributes
                                                            .get("name")
                                                             .map(|name| match name.split_once(':') {
                                                                Some((_, name)) => ns_type.map_or(name.into(), |ns| format!("{}:{}", ns, name)),
                                                                None => ns_type.map_or(name.into(), |ns| format!("{}:{}", ns, name))
                                                            })
                                                    }),
                                                _ => Default::default(),
                                            })
                                            .collect::<Vec<_>>()
                                    })
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                fields.extend(
                    children
                        .iter()
                        .flat_map(|xml| Field::try_from((ns, xml, types))),
                );

                let base_name = children.iter().find_map(|c| match c {
                    xmltree::XMLNode::Element(xmltree::Element { name, children, .. })
                        if name == "annotation" =>
                    {
                        children.iter().find_map(|c| match c {
                            xmltree::XMLNode::Element(xmltree::Element {
                                name, children, ..
                            }) if name == "appinfo" => children.iter().find_map(|c| match c {
                                xmltree::XMLNode::Element(xmltree::Element {
                                    name,
                                    attributes,
                                    ..
                                }) if name == "property" && attributes.get("name").is_some() => {
                                    attributes.get("name")
                                }
                                _ => None,
                            }),
                            _ => None,
                        })
                    }
                    _ => None,
                });

                for child in children {
                    match child {
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(_xml_schema_ns),
                            name,
                            children,
                            ..
                        }) if name == "sequence" => {
                            fields.extend(
                                children
                                    .iter()
                                    .flat_map(|xml| Field::try_from((ns, xml, types))),
                            );
                            parents.extend(
                                children.iter().flat_map(GroupRef::try_from).map(|g| {
                                    str_to_simple_type_or_reference(ns, &g.reference, None)
                                }),
                            );
                        }
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(_xml_schema_ns),
                            name,
                            children,
                            ..
                        }) if match name.as_str() {
                            "complexContent" | "simpleContent" => true,
                            _ => false,
                        } =>
                        {
                            annotations.extend(
                                children.iter().filter_map(|c| Annotation::try_from(c).ok()),
                            );
                            for child in children {
                                match child {
                                    xmltree::XMLNode::Element(xmltree::Element {
                                        attributes,
                                        namespace: Some(_xml_schema_ns),
                                        name,
                                        children,
                                        ..
                                    }) if name == "extension" => {
                                        if let Some(type_name) = attributes.get("base") {
                                            parents.push(str_to_simple_type_or_reference(
                                                ns,
                                                type_name,
                                                base_name.map(|r| r.into()),
                                            ));
                                        }
                                        fields.extend(
                                            children
                                                .iter()
                                                .flat_map(|xml| Field::try_from((ns, xml, types))),
                                        );
                                        parents.extend(
                                            children.iter().flat_map(GroupRef::try_from).map(|g| {
                                                str_to_simple_type_or_reference(
                                                    ns,
                                                    &g.reference,
                                                    None,
                                                )
                                            }),
                                        );
                                        for child in children {
                                            match child {
                                                xmltree::XMLNode::Element(xmltree::Element {
                                                    namespace: Some(_xml_schema_ns),
                                                    name,
                                                    children,
                                                    ..
                                                }) if name == "sequence" => {
                                                    fields.extend(children.iter().flat_map(
                                                        |xml| Field::try_from((ns, xml, types)),
                                                    ));
                                                    parents.extend(
                                                        children
                                                            .iter()
                                                            .flat_map(GroupRef::try_from)
                                                            .map(|g| {
                                                                str_to_simple_type_or_reference(
                                                                    ns,
                                                                    &g.reference,
                                                                    None,
                                                                )
                                                            }),
                                                    );
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }

                fields.extend(parents.iter().filter_map(|p| match p {
                    openapiv3::ReferenceOr::Reference { .. } => None,
                    openapiv3::ReferenceOr::Item(i @ SimpleType { name, .. }) => Some(Field {
                        annotation: Some(Annotation {
                            content_type: None,
                            deprecated: false,
                            description: None,
                            required: Some(true),
                            removed: false,
                        }),
                        name: name.as_ref().map_or("value".into(), |s| s.into()),
                        occurrences: Occurrences::One,
                        r#type: openapiv3::ReferenceOr::Item(i.clone()),
                    }),
                }));

                parents.retain(|p| match p {
                    openapiv3::ReferenceOr::Reference { .. } => true,
                    openapiv3::ReferenceOr::Item(_) => false,
                });

                let annotation: Option<Annotation> = annotations.into_iter().fold(
                    None,
                    |acc: Option<Annotation>, ann: Annotation| match acc {
                        None => Some(ann),
                        Some(acc) => Some(acc.merge(ann)),
                    },
                );

                Ok(ObjectType {
                    name,
                    annotation,
                    fields,
                    parents,
                    descendants,
                })
            }
            _ => Err(TypeParseError::NotTypeNode),
        }
    }
}

impl From<&ObjectType> for openapiv3::Schema {
    fn from(c: &ObjectType) -> Self {
        match &c {
            &ObjectType {
                name,
                parents,
                descendants,
                fields,
                annotation,
                ..
            } => {
                let mut schema_kind =
                    openapiv3::SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
                        properties: fields
                            .iter()
                            .map(|s| {
                                (
                                    s.name.clone(),
                                    match openapiv3::ReferenceOr::from(s) {
                                        openapiv3::ReferenceOr::Item(v) => {
                                            openapiv3::ReferenceOr::Item(Box::new(v))
                                        }
                                        openapiv3::ReferenceOr::Reference { reference } => {
                                            openapiv3::ReferenceOr::Reference { reference }
                                        }
                                    },
                                )
                            })
                            .collect(),
                        additional_properties: Some(openapiv3::AdditionalProperties::Any(false)),
                        required: fields
                            .iter()
                            .filter_map(|s| {
                                if s.annotation
                                    .as_ref()
                                    .and_then(|a| a.required)
                                    .unwrap_or(false)
                                {
                                    Some(s.name.clone())
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        ..Default::default()
                    }));
                let mut schema_data = openapiv3::SchemaData {
                    deprecated: annotation.as_ref().map(|a| a.deprecated).unwrap_or(false),
                    title: Some(name.clone()),
                    description: annotation.as_ref().and_then(|a| a.description.clone()),
                    ..Default::default()
                };
                if parents.len() > 0 {
                    let mut all_of = Vec::new();
                    all_of.extend(parents.iter().map(|reference| match reference {
                        openapiv3::ReferenceOr::Reference { reference } => {
                            openapiv3::ReferenceOr::Reference {
                                reference: format!("#/components/schemas/{}", reference),
                            }
                        }
                        openapiv3::ReferenceOr::Item(simple_type) => {
                            openapiv3::ReferenceOr::Item(simple_type.into())
                        }
                    }));

                    all_of.push(openapiv3::ReferenceOr::Item(openapiv3::Schema {
                        schema_kind,
                        schema_data: Default::default(),
                    }));

                    schema_kind = openapiv3::SchemaKind::AllOf { all_of }
                }
                if descendants.len() > 0 {
                    match schema_kind {
                        openapiv3::SchemaKind::Type(openapiv3::Type::Object(
                            openapiv3::ObjectType {
                                ref mut properties,
                                ref mut required,
                                ..
                            },
                        )) => {
                            properties.entry(String::from("_type")).or_insert_with(|| {
                                openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                                    schema_data: Default::default(),
                                    schema_kind: openapiv3::SchemaKind::Type(
                                        openapiv3::Type::String(openapiv3::StringType {
                                            ..Default::default()
                                        }),
                                    ),
                                })
                            });
                            required.push(String::from("_type"));
                        }
                        openapiv3::SchemaKind::AllOf { ref mut all_of } => {
                            if let Some(openapiv3::ReferenceOr::Item(openapiv3::Schema {
                                schema_kind:
                                    openapiv3::SchemaKind::Type(openapiv3::Type::Object(
                                        openapiv3::ObjectType {
                                            ref mut properties,
                                            ref mut required,
                                            ..
                                        },
                                    )),
                                ..
                            })) = all_of.into_iter().find(|kind| match kind {
                                openapiv3::ReferenceOr::Item(openapiv3::Schema {
                                    schema_kind:
                                        openapiv3::SchemaKind::Type(openapiv3::Type::Object(
                                            openapiv3::ObjectType { .. },
                                        )),
                                    ..
                                }) => true,
                                _ => false,
                            }) {
                                properties.entry(String::from("_type")).or_insert_with(|| {
                                    openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                                        schema_data: Default::default(),
                                        schema_kind: openapiv3::SchemaKind::Type(
                                            openapiv3::Type::String(openapiv3::StringType {
                                                ..Default::default()
                                            }),
                                        ),
                                    })
                                });
                                required.push(String::from("_type"));
                            }
                        }
                        _ => {}
                    }
                    schema_data.discriminator = Some(Discriminator {
                        property_name: String::from("_type"),
                        mapping: descendants
                            .iter()
                            .map(|type_name| match type_name.split_once(':') {
                                Some((ns, name)) => (
                                    name.replace("_", "").to_owned(),
                                    format!("#/components/schemas/{ns}_{name}"),
                                ),
                                None => (
                                    type_name.to_owned(),
                                    format!("#/components/schemas/{type_name}"),
                                ),
                            })
                            .collect(),
                        extensions: Default::default(),
                    });
                }

                openapiv3::Schema {
                    schema_data,
                    schema_kind,
                }
            }
        }
    }
}

#[test]
fn parse_attribute_group_test() {
    let xml: &[u8] = br#"
<xs:attributeGroup name="CommonAttributes" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
    <xs:attribute name="name" type="xs:string" use="required">
        <xs:annotation>
            <xs:documentation source="modifiable">
                always
            </xs:documentation>
            <xs:documentation xml:lang="en">
                The name that people should call you.
            </xs:documentation>
            <xs:documentation source="required">
                true
            </xs:documentation>
        </xs:annotation>
    </xs:attribute>
    <xs:attribute name="age" type="xs:int" use="required">
        <xs:annotation>
            <xs:documentation source="modifiable">
                none
            </xs:documentation>
            <xs:documentation xml:lang="en">
                Your age in years.
            </xs:documentation>
            <xs:documentation source="required">
                true
            </xs:documentation>
        </xs:annotation>
    </xs:attribute>
</xs:attributeGroup>
"#;

    let ns: Option<&str> = Some("test");
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!(
            {
              "title": "test_CommonAttributes",
              "type": "object",
              "properties": {
                "name": {
                  "description": "The name that people should call you.",
                  "type": "string"
                },
                "age": {
                  "description": "Your age in years.",
                  "format": "int32",
                  "type": "integer"
                }
              },
              "required": [
                "name",
                "age"
              ],
              "additionalProperties": false
            }
        )
    );
}

#[test]
fn parse_attribute_group_ref_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:complexContent>
            <xs:extension base="BaseType">
                <xs:attributeGroup ref="GroupReference"/>
                <xs:attribute name="optionalAttribute" type="xs:string">
                    <xs:annotation>
                        <xs:documentation source="modifiable">none</xs:documentation>
                        <xs:documentation>
                            A field that comes from an attribute.
                        </xs:documentation>
                    </xs:annotation>
                </xs:attribute>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>
    "#;
    let ns: Option<&str> = Some("test");
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "title": "test_TestType",
          "allOf": [
            {
              "$ref": "#/components/schemas/test_BaseType"
            },
            {
              "$ref": "#/components/schemas/test_GroupReference"
            },
            {
              "type": "object",
              "properties": {
                "optionalAttribute": {
                  "description": "A field that comes from an attribute.",
                  "type": "string"
                }
              },
              "additionalProperties": false
            }
          ]
        })
    );
}

#[test]
fn parse_annotation_inside_complex_content_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:complexContent>
            <xs:annotation>
                <xs:appinfo>
                    <meta:content-type>application/vnd.ccouzens.test</meta:content-type>
                </xs:appinfo>
                <xs:documentation source="since">0.9</xs:documentation>
                <xs:documentation xml:lang="en">
                    A simple type to test the parser
                </xs:documentation>
            </xs:annotation>

            <xs:extension base="BaseType">
                <xs:attribute name="requiredAttribute" type="xs:string" use="required">
                    <xs:annotation>
                        <xs:documentation source="modifiable">none</xs:documentation>
                        <xs:documentation>
                            A field that comes from an attribute.
                        </xs:documentation>
                        <xs:documentation source="required">true</xs:documentation>
                    </xs:annotation>
                </xs:attribute>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>
    "#;
    let ns: Option<&str> = Some("test");
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "title": "test_TestType",
          "description": "A simple type to test the parser",
          "allOf": [
            {
              "$ref": "#/components/schemas/test_BaseType"
            },
            {
              "type": "object",
              "properties": {
                "requiredAttribute": {
                  "description": "A field that comes from an attribute.",
                  "type": "string"
                }
              },
              "required": [
                "requiredAttribute"
              ],
              "additionalProperties": false
            }
          ]
        })
    );
}

#[test]
fn removed_field_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" abstract="true" name="BaseType">
        <xs:annotation>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A base abstract type for all the types.
            </xs:documentation>
        </xs:annotation>

        <xs:sequence>
            <xs:element name="FieldA" type="xs:string" minOccurs="0">
                <xs:annotation>
                    <xs:appinfo><meta:version added-in="1.0" removed-in="5.1"/></xs:appinfo>
                    <xs:documentation source="modifiable">always</xs:documentation>
                    <xs:documentation xml:lang="en">
                        A field that has been removed
                    </xs:documentation>
                    <xs:documentation source="required">false</xs:documentation>
                </xs:annotation>
            </xs:element>
            <xs:element name="FieldB" type="xs:string" minOccurs="0">
            <xs:annotation>
                <xs:documentation source="modifiable">always</xs:documentation>
                <xs:documentation xml:lang="en">
                    A field that has not been removed
                </xs:documentation>
                <xs:documentation source="required">false</xs:documentation>
            </xs:annotation>
        </xs:element>

        </xs:sequence>
    </xs:complexType>
    "#;
    let ns: Option<&str> = Some("test");
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "title": "test_BaseType",
          "description": "A base abstract type for all the types.",
          "type": "object",
          "properties": {
            "fieldB": {
              "description": "A field that has not been removed",
              "type": "string"
            }
          },
          "additionalProperties": false
        })
    );
}
