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
#[cfg(test)]
use serde_json::json;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub(super) struct ObjectType {
    pub(super) annotation: Option<Annotation>,
    pub(super) name: String,
    pub(super) fields: Vec<Field>,
    pub(super) parents: Vec<openapiv3::ReferenceOr<SimpleType>>,
}

impl TryFrom<(&xmltree::XMLNode, &str)> for ObjectType {
    type Error = TypeParseError;

    fn try_from((xml, schema_namespace): (&xmltree::XMLNode, &str)) -> Result<Self, Self::Error> {
        match xml {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                attributes,
                children,
                ..
            }) if namespace == XML_SCHEMA_NS
                && (name == "complexType" || name == "group" || name == "attributeGroup") =>
            {
                let mut annotations = Vec::new();
                let name = attributes
                    .get("name")
                    .map(|n| format!("{}_{}", schema_namespace, n))
                    .ok_or(TypeParseError::MissingName)?;
                annotations.extend(children.iter().filter_map(|c| Annotation::try_from(c).ok()));
                let mut fields = Vec::new();
                let mut parents = Vec::new();
                fields.extend(
                    children
                        .iter()
                        .flat_map(|xml| Field::try_from((xml, schema_namespace))),
                );
                for child in children {
                    match child {
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(namespace),
                            name,
                            children,
                            ..
                        }) if namespace == XML_SCHEMA_NS && name == "sequence" => {
                            fields.extend(
                                children
                                    .iter()
                                    .flat_map(|xml| Field::try_from((xml, schema_namespace))),
                            );
                            parents.extend(children.iter().flat_map(GroupRef::try_from).map(|g| {
                                str_to_simple_type_or_reference(schema_namespace, &g.reference)
                            }));
                        }
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(namespace),
                            name,
                            children,
                            ..
                        }) if namespace == XML_SCHEMA_NS
                            && (name == "complexContent" || name == "simpleContent") =>
                        {
                            annotations.extend(
                                children.iter().filter_map(|c| Annotation::try_from(c).ok()),
                            );

                            for child in children {
                                match child {
                                    xmltree::XMLNode::Element(xmltree::Element {
                                        attributes,
                                        namespace: Some(namespace),
                                        name,
                                        children,
                                        ..
                                    }) if namespace == XML_SCHEMA_NS && name == "extension" => {
                                        if let Some(type_name) = attributes.get("base") {
                                            parents.push(str_to_simple_type_or_reference(
                                                schema_namespace,
                                                type_name,
                                            ));
                                        }
                                        fields.extend(children.iter().flat_map(|xml| {
                                            Field::try_from((xml, schema_namespace))
                                        }));
                                        parents.extend(
                                            children.iter().flat_map(GroupRef::try_from).map(|g| {
                                                str_to_simple_type_or_reference(
                                                    schema_namespace,
                                                    &g.reference,
                                                )
                                            }),
                                        );
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
                                                    fields.extend(children.iter().flat_map(
                                                        |xml| {
                                                            Field::try_from((xml, schema_namespace))
                                                        },
                                                    ));
                                                    parents.extend(
                                                        children
                                                            .iter()
                                                            .flat_map(GroupRef::try_from)
                                                            .map(|g| {
                                                                str_to_simple_type_or_reference(
                                                                    schema_namespace,
                                                                    &g.reference,
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
                    openapiv3::ReferenceOr::Item(i) => Some(Field {
                        annotation: Some(Annotation {
                            content_type: None,
                            deprecated: false,
                            description: None,
                            modifiable: None,
                            required: Some(true),
                            removed: false,
                        }),
                        name: "value".into(),
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
                })
            }
            _ => Err(TypeParseError::NotTypeNode),
        }
    }
}

impl From<&ObjectType> for openapiv3::Schema {
    fn from(c: &ObjectType) -> Self {
        let schema_kind =
            openapiv3::SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
                properties: c
                    .fields
                    .iter()
                    .map(|s| {
                        (
                            s.name.clone(),
                            openapiv3::ReferenceOr::boxed_item(openapiv3::Schema::from(s)),
                        )
                    })
                    .collect(),
                additional_properties: Some(openapiv3::AdditionalProperties::Any(false)),
                required: c
                    .fields
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
        let schema_data = openapiv3::SchemaData {
            deprecated: c.annotation.as_ref().map(|a| a.deprecated).unwrap_or(false),
            title: Some(c.name.clone()),
            description: c.annotation.as_ref().and_then(|a| a.description.clone()),
            ..Default::default()
        };
        match &c.parents.is_empty() {
            true => openapiv3::Schema {
                schema_data,
                schema_kind,
            },
            false => {
                let mut all_of = Vec::new();
                all_of.extend(c.parents.iter().map(|reference| match reference {
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

                openapiv3::Schema {
                    schema_data,
                    schema_kind: openapiv3::SchemaKind::AllOf { all_of },
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

    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((&xmltree::XMLNode::Element(tree), "test")).unwrap();
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
                  "type": "integer",
                  "readOnly": true
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
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((&xmltree::XMLNode::Element(tree), "test")).unwrap();
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
                  "readOnly": true,
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
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((&xmltree::XMLNode::Element(tree), "test")).unwrap();
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
                  "readOnly": true,
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
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((&xmltree::XMLNode::Element(tree), "test")).unwrap();
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
