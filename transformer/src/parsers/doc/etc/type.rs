use crate::parsers::doc::etc::object_type::ObjectType;
use crate::parsers::doc::etc::primitive_type::ParsePrimitiveTypeError;
use crate::parsers::doc::etc::simple_type::SimpleType;
#[cfg(test)]
use serde_json::json;

use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub(super) enum Type {
    ObjectType(ObjectType),
    SimpleType(SimpleType),
}

impl Type {
    pub fn content_type(&self) -> Option<&str> {
        match self {
            Type::ObjectType(o) => o.annotation.as_ref(),
            Type::SimpleType(s) => s.annotation.as_ref(),
        }
        .and_then(|a| a.content_type.as_deref())
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Type::ObjectType(o) => Some(o.name.as_str()),
            Type::SimpleType(s) => s.name.as_deref(),
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum TypeParseError {
    #[error("not a complex or simple type node")]
    NotTypeNode,
    #[error("missing name attribute")]
    MissingName,
    #[error("failure to parse PrimitiveType")]
    PrimitiveTypeParseError(#[from] ParsePrimitiveTypeError),
    #[error("Missing base attribute")]
    MissingBase,
    #[error("Missing item type attribute")]
    MissingItemTypeValue,
}

impl TryFrom<(Option<&str>, &xmltree::XMLNode, &Vec<(Option<&str>, &xmltree::XMLNode)>)> for Type {
    type Error = TypeParseError;
    fn try_from((ns, xml, types): (Option<&str>, &xmltree::XMLNode, &Vec<(Option<&str>, &xmltree::XMLNode)>)) -> Result<Self, Self::Error> {
        match ObjectType::try_from((ns, xml, types)) {
            Err(TypeParseError::NotTypeNode) => Ok(Type::SimpleType(SimpleType::try_from((ns, xml))?)),
            Ok(object) => Ok(Type::ObjectType(object)),
            Err(e) => Err(e),
        }
    }
}

impl From<&Type> for openapiv3::Schema {
    fn from(t: &Type) -> Self {
        match t {
            Type::ObjectType(c) => Self::from(c),
            Type::SimpleType(s) => Self::from(s),
        }
    }
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
    </xs:complexType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
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
                "optionalString": {
                  "description": "String that may or may not be here",
                  "type": "string"
                },
                "requiredString": {
                  "description": "String that will be here",
                  "type": "string"
                }
              },
              "required": [
                "requiredString"
              ],
              "additionalProperties": false
            }
          ]
        }
        )
    );
}

#[test]
fn parse_type_that_is_attribute_test() {
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
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
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
fn parse_type_that_is_attribute_but_not_required_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:complexContent>
            <xs:extension base="BaseType">
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
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
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
fn parse_type_that_is_attribute_but_not_extension_test() {
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
        <xs:attribute name="requiredAttribute" type="xs:string" use="required">
            <xs:annotation>
                <xs:documentation source="modifiable">none</xs:documentation>
                <xs:documentation>
                    A field that comes from an attribute.
                </xs:documentation>
                <xs:documentation source="required">true</xs:documentation>
            </xs:annotation>
        </xs:attribute>
    </xs:complexType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "title": "test_TestType",
          "description": "A simple type to test the parser",
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
        )
    );
}

#[test]
fn simple_type_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType name="CoinType" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
        <xs:annotation>
            <xs:appinfo><meta:version added-in="5.6"/></xs:appinfo>
            <xs:documentation source="since">5.6</xs:documentation>
            <xs:documentation xml:lang="en">
                An enumeration of the sides of a coin
            </xs:documentation>
        </xs:annotation>
        <xs:restriction base="xs:string">
            <xs:enumeration value="Heads"/>
            <xs:enumeration value="Tails"/>
        </xs:restriction>
    </xs:simpleType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "An enumeration of the sides of a coin",
            "type": "string",
            "enum": ["Heads", "Tails"],
            "title": "test_CoinType"
        })
    );
}

#[test]
fn simple_type_with_pattern_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType name="HttpsType"xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:restriction base="xs:anyURI">
            <xs:pattern value="https://.+"/>
        </xs:restriction>
    </xs:simpleType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "type": "string",
            "format": "uri",
            "pattern": "https://.+",
            "title": "test_HttpsType"
        })
    );
}

#[test]
fn simple_type_with_min_inclusive_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType name="DaysInMonth" xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:restriction base="xs:int">
            <xs:minInclusive value="28"/>
        </xs:restriction>
    </xs:simpleType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "type": "integer",
            "format": "int32",
            "minimum": 28,
            "title": "test_DaysInMonth"
        })
    );
}

#[test]
fn simple_type_with_list_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType name="FavouriteFoods" xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:list itemType="xs:string"/>
    </xs:simpleType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "type": "array",
            "items": { "type": "string" },
            "title": "test_FavouriteFoods"
        })
    );
}

#[test]
fn simplest_simple_type_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:restriction base="xs:string">
        </xs:restriction>
    </xs:simpleType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({"type": "string"})
    );
}

#[test]
fn base_type_into_schema_test() {
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
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "additionalProperties": false,
            "description": "A base abstract type for all the types.",
            "properties": {
                "baseField": {
                    "description": "A base field for the base type",
                    "type": "string"
                }
            },
            "title": "test_BaseType",
            "type": "object"
        })
    );
}

#[test]
fn parent_type_into_schema_test() {
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
    </xs:complexType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A simple type to test the parser",
            "allOf": [
                {
                    "$ref": "#/components/schemas/test_BaseType"
                },
                {
                  "additionalProperties": false,
                  "properties": {
                    "optionalString": {
                        "description": "String that may or may not be here",
                        "type": "string"
                    },
                    "requiredString": {
                        "description": "String that will be here",
                        "type": "string"
                    }
                  },
                  "required": ["requiredString"],
                  "type": "object"
              }
            ],
            "title": "test_TestType",
        })
    );
}

#[test]
fn parse_group_test() {
    let xml: &[u8] = br#"
    <xs:group xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="SimpleGroup">
        <xs:sequence>
            <xs:element name="Field1" type="xs:int" minOccurs="1" maxOccurs="1">
                <xs:annotation>
                    <xs:documentation source="modifiable">always</xs:documentation>
                    <xs:documentation xml:lang="en">
                        The first field in the group.
                    </xs:documentation>
                    <xs:documentation source="required">true</xs:documentation>
                </xs:annotation>
            </xs:element>
            <xs:element name="Field2" type="xs:string" minOccurs="0" maxOccurs="1">
                <xs:annotation>
                    <xs:documentation source="modifiable">always</xs:documentation>
                    <xs:documentation xml:lang="en">
                        The second field in the group.
                    </xs:documentation>
                    <xs:documentation source="required">false</xs:documentation>
                </xs:annotation>
            </xs:element>
        </xs:sequence>
    </xs:group>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "title": "test_SimpleGroup",
          "type": "object",
          "properties": {
            "field1": {
              "description": "The first field in the group.",
              "type": "integer",
              "format": "int32"
            },
            "field2": {
              "description": "The second field in the group.",
              "type": "string"
            }
          },
          "required": [
            "field1"
          ],
          "additionalProperties": false
        })
    );
}

#[test]
fn parse_group_ref_test() {
    let xml: &[u8] = br#"
    <xs:complexType name="ObjectWithTwoBases" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
        <xs:annotation>
            <xs:documentation xml:lang="en">
                Object with 2 bases
            </xs:documentation>
        </xs:annotation>
        <xs:complexContent>
            <xs:extension base="Base1">
                <xs:sequence>
                    <xs:group ref="Base2"/>
                </xs:sequence>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "title": "test_ObjectWithTwoBases",
            "description": "Object with 2 bases",
            "allOf": [
                {
                    "$ref": "#/components/schemas/test_Base1"
                },
                {
                    "$ref": "#/components/schemas/test_Base2"
                },
                {
                    "type": "object",
                    "additionalProperties": false
                }
            ]
        })
    );
}

#[test]
fn parse_group_ref_no_sequence_test() {
    let xml: &[u8] = br#"
    <xs:complexType name="ObjectWithTwoBases" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
        <xs:annotation>
            <xs:documentation xml:lang="en">
                Object with 2 bases
            </xs:documentation>
        </xs:annotation>
        <xs:complexContent>
            <xs:extension base="Base1">
                <xs:group ref="Base2"/>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "title": "test_ObjectWithTwoBases",
            "description": "Object with 2 bases",
            "allOf": [
                {
                    "$ref": "#/components/schemas/test_Base1"
                },
                {
                    "$ref": "#/components/schemas/test_Base2"
                },
                {
                    "type": "object",
                    "additionalProperties": false
                }
            ]
        })
    );
}

#[test]
fn parse_group_ref_one_parent_test() {
    let xml: &[u8] = br#"
    <xs:complexType name="ObjectWithOneBase" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
        <xs:annotation>
            <xs:documentation xml:lang="en">
                Object with 1 base
            </xs:documentation>
        </xs:annotation>
        <xs:sequence>
            <xs:group ref="Base1"/>
        </xs:sequence>
    </xs:complexType>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, &xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "title": "test_ObjectWithOneBase",
            "description": "Object with 1 base",
            "allOf": [
                {
                    "$ref": "#/components/schemas/test_Base1"
                },
                {
                    "type": "object",
                    "additionalProperties": false
                }
            ]
        })
    );
}

#[test]
fn parse_complex_simple_extension_type() {
    let xml: &[u8] = br###"
    <xs:complexType name="ExtendedString" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
      <xs:annotation>
        <xs:appinfo><meta:version added-in="5.1"/></xs:appinfo>
        <xs:documentation xml:lang="en">
           Base shows up as value
        </xs:documentation>
      </xs:annotation>
      <xs:simpleContent>
        <xs:extension base="xs:string">
            <xs:attribute name="field" type="xs:string" use="required">
                <xs:annotation>
                    <xs:appinfo><meta:version added-in="5.1"/></xs:appinfo>
                    <xs:documentation source="required">true</xs:documentation>
                    <xs:documentation xml:lang="en">
                        field shows up as itself
                    </xs:documentation>
                </xs:annotation>
            </xs:attribute>
        </xs:extension>
      </xs:simpleContent>
    </xs:complexType>
        "###;
        let ns: Option<&str> = None;
        let tree = xmltree::Element::parse(xml).unwrap();
        let types = xmltree::Element::parse(xml).unwrap();
        let c = Type::try_from((
            ns,
            &xmltree::XMLNode::Element(tree),
            &vec![(ns, &xmltree::XMLNode::Element(types))],
        ))
        .unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "title": "test_ExtendedString",
          "description": "Base shows up as value",
          "type": "object",
          "properties": {
            "field": {
              "description": "field shows up as itself",
              "type": "string"
            },
            "value": {
              "type": "string"
            }
          },
          "required": [
            "field",
            "value"
          ],
          "additionalProperties": false
        }
        )
    );
}
